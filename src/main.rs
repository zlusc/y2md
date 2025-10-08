use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use y2md::{
    fetch_video_metadata, format_markdown, transcribe_video, validate_youtube_url, AppConfig,
    LlmProvider,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// YouTube URL to transcribe
    url: Option<String>,

    /// Output directory for transcript
    #[arg(short, long, default_value = ".")]
    out_dir: String,

    /// Prefer captions over STT
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    prefer_captions: bool,

    /// Language code override
    #[arg(long)]
    lang: Option<String>,

    /// Include timestamps in transcript
    #[arg(long, default_value_t = false)]
    timestamps: bool,

    /// Compact output format
    #[arg(long, default_value_t = false)]
    compact: bool,

    /// Paragraph length for enhanced formatting (sentences per paragraph)
    #[arg(long, default_value_t = 4)]
    paragraph_length: usize,

    /// Force enhanced formatting even for music content
    #[arg(long, default_value_t = false)]
    force_formatting: bool,

    /// Cookies file for restricted content
    #[arg(long)]
    cookies: Option<String>,

    /// Whisper model size
    #[arg(long, default_value = "small")]
    model: String,

    /// Number of threads for STT
    #[arg(long, default_value_t = 4)]
    threads: usize,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Use local LLM for enhanced transcript formatting
    #[arg(long, default_value_t = false)]
    use_llm: bool,

    /// Dry run - don't write files
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// Save raw transcript to separate txt file
    #[arg(long, default_value_t = false)]
    save_raw: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
    /// Model management
    Model {
        #[command(subcommand)]
        action: ModelCommands,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set LLM provider
    SetLlmProvider {
        /// LLM provider (ollama, openai, lmstudio)
        provider: String,
    },
    /// Set LLM model
    SetLlmModel {
        /// Model name
        model: String,
    },
    /// Set LLM endpoint
    SetLlmEndpoint {
        /// API endpoint URL
        endpoint: String,
    },
    /// Set LLM API key
    SetLlmApiKey {
        /// API key
        api_key: String,
    },
    /// Set default language
    SetLanguage {
        /// Language code (en, es, fr, etc.)
        language: String,
    },
    /// Set output directory
    SetOutputDir {
        /// Default output directory
        output_dir: String,
    },
    /// Set paragraph length
    SetParagraphLength {
        /// Sentences per paragraph
        length: usize,
    },
    /// Reset configuration to defaults
    Reset,
}

#[derive(Subcommand, Debug)]
enum ModelCommands {
    /// Show model status and availability
    Status,
    /// Download a model (defaults to current configured model)
    Download {
        /// Optional model name to download
        model: Option<String>,
    },
    /// List available models in Ollama library
    ListAvailable {
        /// Optional search term
        search: Option<String>,
    },
    /// List locally installed models
    ListLocal,
    /// Remove a model
    Remove {
        /// Model name to remove
        model: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Handle subcommands
    if let Some(command) = args.command {
        match command {
            Commands::Config { action } => {
                return handle_config_command(action).await;
            }
            Commands::Model { action } => {
                return handle_model_command(action).await;
            }
        }
    }

    // If no URL provided, show help
    let url = args.url.ok_or_else(|| {
        anyhow::anyhow!("YouTube URL is required. Use --help for usage information.")
    })?;

    // Initialize logging
    if args.verbose {
        tracing_subscriber::fmt::init();
    }

    // Validate URL and extract video ID
    let video_id = validate_youtube_url(&url)?;

    // Fetch video metadata
    let metadata = fetch_video_metadata(&video_id).await?;

    println!("Transcribing: {}", metadata.title);
    println!(
        "Channel: {}",
        metadata.channel.as_deref().unwrap_or("Unknown")
    );
    println!("Video ID: {}", video_id);
    println!("Output directory: {}", args.out_dir);

    // Load configuration
    let config = AppConfig::load()?;

    // Use configuration values with CLI args as overrides
    let prefer_captions = args.prefer_captions;
    let language = args.lang.as_deref().or(Some(&config.default_language));
    let output_dir = if args.out_dir != "." {
        &args.out_dir
    } else {
        config.output_dir.as_deref().unwrap_or(".")
    };
    let paragraph_length = args.paragraph_length;
    let timestamps = args.timestamps || config.timestamps;
    let compact = args.compact || config.compact;
    let use_llm = args.use_llm;

    // Perform transcription
    let (transcript, source, raw_transcript) = transcribe_video(
        &video_id,
        prefer_captions,
        language,
        output_dir,
        paragraph_length,
        args.force_formatting,
    )
    .await?;

    // Format as Markdown
    let markdown = format_markdown(
        &metadata,
        &transcript,
        &source,
        timestamps,
        compact,
        paragraph_length,
        use_llm,
    )
    .await;

    // Generate filename
    let sanitized_title = metadata
        .title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let filename = format!(
        "{}_{}_{}.md",
        chrono::Utc::now().format("%Y-%m-%d"),
        video_id,
        sanitized_title
    );
    let output_path = std::path::Path::new(&args.out_dir).join(&filename);

    if args.dry_run {
        println!("Dry run - would save to: {}", output_path.display());
        println!(
            "Markdown preview (first 500 chars):\n{}",
            &markdown[..markdown.len().min(500)]
        );
    } else {
        // Save to file
        fs::write(&output_path, &markdown)?;
        println!("Transcription saved to: {}", output_path.display());
    }

    // Save raw transcript if requested
    if args.save_raw {
        let raw_filename = format!(
            "{}_{}_{}_raw.txt",
            chrono::Utc::now().format("%Y-%m-%d"),
            video_id,
            sanitized_title
        );
        let raw_output_path = std::path::Path::new(&args.out_dir).join(&raw_filename);

        if args.dry_run {
            println!(
                "Dry run - would save raw transcript to: {}",
                raw_output_path.display()
            );
            println!(
                "Raw transcript preview (first 500 chars):\n{}",
                &raw_transcript[..raw_transcript.len().min(500)]
            );
        } else {
            fs::write(&raw_output_path, &raw_transcript)?;
            println!("Raw transcript saved to: {}", raw_output_path.display());
        }
    }

    // Calculate formatting statistics
    let word_count = transcript.split_whitespace().count();
    let char_count = transcript.chars().count();
    let paragraph_count = markdown.matches("\n\n").count() + 1;
    let formatting_type = if args.compact { "compact" } else { "enhanced" };

    println!("Transcription completed using: {}", source);
    println!("Formatting statistics:");
    println!("  - Formatting type: {}", formatting_type);
    println!("  - Paragraph length: {} sentences", args.paragraph_length);
    println!("  - Word count: {}", word_count);
    println!("  - Character count: {}", char_count);
    println!("  - Paragraph count: {}", paragraph_count);

    Ok(())
}

/// Handle configuration commands
async fn handle_config_command(action: ConfigCommands) -> anyhow::Result<()> {
    match action {
        ConfigCommands::Show => {
            let config = AppConfig::load()?;
            println!("Current configuration:");
            println!("  LLM Provider: {}", config.llm.provider);
            println!("  LLM Model: {}", config.llm.model);
            if let Some(endpoint) = &config.llm.endpoint {
                println!("  LLM Endpoint: {}", endpoint);
            }
            if let Some(api_key) = &config.llm.api_key {
                println!("  LLM API Key: {}...", &api_key[..api_key.len().min(8)]);
            }
            println!("  Prefer Captions: {}", config.prefer_captions);
            println!("  Default Language: {}", config.default_language);
            if let Some(output_dir) = &config.output_dir {
                println!("  Output Directory: {}", output_dir);
            }
            println!("  Timestamps: {}", config.timestamps);
            println!("  Compact: {}", config.compact);
            println!("  Paragraph Length: {}", config.paragraph_length);

            let config_path = AppConfig::config_path()?;
            println!("\nConfiguration file: {}", config_path.display());
        }
        ConfigCommands::SetLlmProvider { provider } => {
            let mut config = AppConfig::load()?;
            let provider = match provider.to_lowercase().as_str() {
                "ollama" => LlmProvider::Ollama,
                "openai" => LlmProvider::OpenAI,
                "lmstudio" => LlmProvider::LMStudio,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Invalid provider. Must be one of: ollama, openai, lmstudio"
                    ))
                }
            };
            config.llm.provider = provider;
            config.save()?;
            println!("LLM provider set to: {}", config.llm.provider);
        }
        ConfigCommands::SetLlmModel { model } => {
            let mut config = AppConfig::load()?;

            // Check if the model is available before setting
            let ollama_manager = y2md::OllamaManager::new(config.llm.endpoint.clone());

            if ollama_manager.is_available().await {
                match ollama_manager.is_model_available(&model).await {
                    Ok(true) => {
                        // Model exists, update config
                        config.llm.model = model;
                        config.save()?;
                        println!("✅ LLM model set to: {}", config.llm.model);
                    }
                    Ok(false) => {
                        // Model doesn't exist, offer to download
                        println!("⚠️  Model '{}' is not available locally.", model);
                        println!("   Do you want to download it now? [y/N]");

                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;

                        if input.trim().eq_ignore_ascii_case("y")
                            || input.trim().eq_ignore_ascii_case("yes")
                        {
                            println!("\n📥 Downloading model...");

                            let pb = ProgressBar::new_spinner();
                            pb.set_style(
                                ProgressStyle::default_spinner()
                                    .template("{spinner} {msg}")
                                    .unwrap()
                                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
                            );
                            pb.set_message("Starting download...");
                            pb.enable_steady_tick(std::time::Duration::from_millis(100));

                            // Use a simpler approach without callback for now
                            match ollama_manager.download_model(&model, None).await {
                                Ok(()) => {
                                    pb.finish_with_message("✅ Model downloaded successfully!");
                                    config.llm.model = model;
                                    config.save()?;
                                    println!("✅ LLM model set to: {}", config.llm.model);
                                }
                                Err(e) => {
                                    pb.finish_with_message("❌ Download failed");
                                    println!("Error: {}", e);
                                    println!("⚠️  Model not set due to download failure.");
                                    println!("   You can manually download it later with: y2md model download");
                                }
                            }
                        } else {
                            // User declined download, still set the model
                            config.llm.model = model;
                            config.save()?;
                            println!("⚠️  LLM model set to: {}", config.llm.model);
                            println!("   Model is not available locally. Run 'y2md model download' to download it.");
                        }
                    }
                    Err(e) => {
                        println!("❌ Error checking model availability: {}", e);
                        println!("⚠️  Setting model anyway, but it may not be available.");
                        config.llm.model = model;
                        config.save()?;
                        println!("LLM model set to: {}", config.llm.model);
                    }
                }
            } else {
                // Ollama not available, set model anyway
                println!("⚠️  Ollama service is not available.");
                println!("   Setting model anyway, but it won't work until Ollama is running.");
                config.llm.model = model;
                config.save()?;
                println!("LLM model set to: {}", config.llm.model);
            }
        }
        ConfigCommands::SetLlmEndpoint { endpoint } => {
            let mut config = AppConfig::load()?;
            config.llm.endpoint = Some(endpoint);
            config.save()?;
            println!(
                "LLM endpoint set to: {}",
                config.llm.endpoint.as_ref().unwrap()
            );
        }
        ConfigCommands::SetLlmApiKey { api_key } => {
            let mut config = AppConfig::load()?;
            config.llm.api_key = Some(api_key);
            config.save()?;
            println!("LLM API key set");
        }
        ConfigCommands::SetLanguage { language } => {
            let mut config = AppConfig::load()?;
            config.default_language = language;
            config.save()?;
            println!("Default language set to: {}", config.default_language);
        }
        ConfigCommands::SetOutputDir { output_dir } => {
            let mut config = AppConfig::load()?;
            config.output_dir = Some(output_dir);
            config.save()?;
            println!(
                "Output directory set to: {}",
                config.output_dir.as_ref().unwrap()
            );
        }
        ConfigCommands::SetParagraphLength { length } => {
            let mut config = AppConfig::load()?;
            config.paragraph_length = length;
            config.save()?;
            println!("Paragraph length set to: {}", config.paragraph_length);
        }
        ConfigCommands::Reset => {
            let default_config = AppConfig::default();
            default_config.save()?;
            println!("Configuration reset to defaults");
        }
    }
    Ok(())
}

/// Handle model management commands
async fn handle_model_command(command: ModelCommands) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let ollama_manager = y2md::OllamaManager::new(config.llm.endpoint.clone());

    match command {
        ModelCommands::Status => {
            println!("Checking model status...");

            // Check if Ollama is available
            if !ollama_manager.is_available().await {
                println!("❌ Ollama service is not available");
                println!(
                    "   Make sure Ollama is running at: {}",
                    config
                        .llm
                        .endpoint
                        .as_deref()
                        .unwrap_or("http://localhost:11434")
                );
                return Ok(());
            }

            println!("✅ Ollama service is available");

            // Check current model
            println!("\nCurrent configured model: {}", config.llm.model);

            match ollama_manager.is_model_available(&config.llm.model).await {
                Ok(true) => {
                    println!("✅ Model is available locally");
                }
                Ok(false) => {
                    println!("❌ Model is not available locally");
                    println!("   Run 'y2md model download' to download it");
                }
                Err(e) => {
                    println!("❌ Error checking model: {}", e);
                }
            }

            // Show local models count
            match ollama_manager.get_local_models().await {
                Ok(models) => {
                    println!("\n📦 Local models: {} models available", models.len());
                    if !models.is_empty() {
                        println!("\nAvailable models:");
                        for model in models.iter().take(10) {
                            println!("  - {}", model);
                        }
                        if models.len() > 10 {
                            println!("  ... and {} more", models.len() - 10);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Error listing local models: {}", e);
                }
            }
        }
        ModelCommands::Download { model } => {
            let model_to_download = model.as_deref().unwrap_or(&config.llm.model);
            println!("Downloading model: {}", model_to_download);

            // Check if model already exists
            if ollama_manager.is_model_available(model_to_download).await? {
                println!("✅ Model '{}' is already available", model_to_download);
                return Ok(());
            }

            // Show confirmation prompt
            println!(
                "\n⚠️  This will download '{}' from Ollama's library.",
                model_to_download
            );
            println!("   This may take several minutes and require significant disk space.");
            println!("   Do you want to continue? [y/N]");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") && !input.trim().eq_ignore_ascii_case("yes")
            {
                println!("Download cancelled.");
                return Ok(());
            }

            println!("\n📥 Downloading model...");

            // Create progress bar
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner} {msg}")
                    .unwrap()
                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
            );
            pb.set_message("Starting download...");
            pb.enable_steady_tick(std::time::Duration::from_millis(100));

            // Use a simpler approach without callback for now
            match ollama_manager.download_model(model_to_download, None).await {
                Ok(()) => {
                    pb.finish_with_message("✅ Model downloaded successfully!");

                    // If a specific model was provided and download succeeded, update config
                    if let Some(model_name) = &model {
                        let mut config = AppConfig::load()?;
                        config.llm.model = model_name.clone();
                        config.save()?;
                        println!("✅ LLM model set to: {}", config.llm.model);
                    }
                }
                Err(e) => {
                    pb.finish_with_message("❌ Download failed");
                    println!("Error: {}", e);
                    return Err(e.into());
                }
            }
        }
        ModelCommands::ListAvailable { search } => {
            println!("Available models in Ollama library:");
            println!("\nNote: This feature requires querying Ollama's model library.");
            println!("For now, you can browse models at: https://ollama.ai/library");

            if let Some(term) = search {
                println!("\nSearch for models containing: '{}'", term);
            }

            // In a real implementation, we'd query Ollama's model library API
            println!("\nTo download a model, use: y2md config set-llm-model <model-name>");
        }
        ModelCommands::ListLocal => match ollama_manager.get_local_models().await {
            Ok(models) => {
                if models.is_empty() {
                    println!("No local models found.");
                } else {
                    println!("Local models ({} total):", models.len());
                    for model in models {
                        println!("  - {}", model);
                    }
                }
            }
            Err(e) => {
                println!("❌ Error listing local models: {}", e);
            }
        },
        ModelCommands::Remove { model } => {
            println!("Removing model: {}", model);

            // Show confirmation prompt
            println!("\n⚠️  This will permanently remove the model '{}'.", model);
            println!("   This action cannot be undone.");
            println!("   Do you want to continue? [y/N]");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") && !input.trim().eq_ignore_ascii_case("yes")
            {
                println!("Removal cancelled.");
                return Ok(());
            }

            match ollama_manager.remove_model(&model).await {
                Ok(()) => {
                    println!("✅ Model '{}' removed successfully", model);
                }
                Err(e) => {
                    println!("❌ Error removing model: {}", e);
                }
            }
        }
    }
    Ok(())
}
