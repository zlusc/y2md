use clap::{Parser, Subcommand};
use std::fs;
use std::io::Write;
use y2md::{
    fetch_video_metadata, format_markdown, transcribe_video, validate_youtube_url, AppConfig,
    CredentialManager, LlmProviderType, OllamaManager,
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

    /// Use LLM for enhanced transcript formatting (optional: specify provider)
    #[arg(long, value_name = "PROVIDER")]
    llm: Option<Option<String>>,

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
        action: Option<ConfigCommands>,
    },
    /// LLM management
    Llm {
        #[command(subcommand)]
        action: LlmCommands,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    /// Show current configuration (default)
    Show,
    /// Open config file in editor
    Edit,
    /// Show config file path
    Path,
    /// Reset configuration to defaults
    Reset,
}

#[derive(Subcommand, Debug)]
enum LlmCommands {
    /// List locally installed models (Ollama)
    List,
    /// Download a model (Ollama)
    Pull {
        /// Model name to download
        model: String,
    },
    /// Remove a model (Ollama)
    Remove {
        /// Model name to remove
        model: String,
    },
    /// Test LLM connection
    Test {
        /// Provider to test (uses default if not specified)
        provider: Option<String>,
    },
    /// Set API key for a provider
    SetKey {
        /// Provider name (openai, anthropic, custom)
        provider: String,
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
            Commands::Llm { action } => {
                return handle_llm_command(action).await;
            }
        }
    }

    // If no URL provided, show help
    let url = args.url.ok_or_else(|| {
        anyhow::anyhow!("YouTube URL is required. Use --help for usage information.")
    })?;

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
        &config.output_dir
    };
    let paragraph_length = args.paragraph_length;
    let timestamps = args.timestamps || config.timestamps;
    let compact = args.compact || config.compact;

    // Determine if we should use LLM and which provider
    let (use_llm, llm_provider) = match &args.llm {
        Some(Some(provider_str)) => {
            // --llm <provider> specified
            let provider = provider_str.parse::<LlmProviderType>().map_err(|e| {
                anyhow::anyhow!(
                    "Invalid provider: {}. Valid providers: local, openai, anthropic, custom",
                    e
                )
            })?;
            (true, Some(provider))
        }
        Some(None) => {
            // --llm flag without provider (use default from config)
            (true, None)
        }
        None => {
            // No --llm flag (check config)
            (config.llm.enabled, None)
        }
    };

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
        llm_provider,
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
        } else {
            fs::write(&raw_output_path, &raw_transcript)?;
            println!("Raw transcript saved to: {}", raw_output_path.display());
        }
    }

    // Calculate formatting statistics
    let word_count = transcript.split_whitespace().count();
    let char_count = transcript.chars().count();
    let paragraph_count = markdown.matches("\n\n").count() + 1;

    println!("Transcription completed using: {}", source);
    println!("Formatting statistics:");
    println!("  - Word count: {}", word_count);
    println!("  - Character count: {}", char_count);
    println!("  - Paragraph count: {}", paragraph_count);

    Ok(())
}

/// Handle configuration commands
async fn handle_config_command(action: Option<ConfigCommands>) -> anyhow::Result<()> {
    match action.unwrap_or(ConfigCommands::Show) {
        ConfigCommands::Show => {
            let config = AppConfig::load()?;
            println!("Current configuration:");
            println!("  Output directory: {}", config.output_dir);
            println!("  Default language: {}", config.default_language);
            println!("  Prefer captions: {}", config.prefer_captions);
            println!("  Timestamps: {}", config.timestamps);
            println!("  Compact: {}", config.compact);
            println!("  Paragraph length: {}", config.paragraph_length);
            println!("\nLLM Settings:");
            println!("  Enabled: {}", config.llm.enabled);
            println!("  Default provider: {}", config.llm.provider);
            println!("  Local model: {}", config.llm.local.model);
            println!("  Local endpoint: {}", config.llm.local.endpoint);
            println!("  OpenAI model: {}", config.llm.openai.model);
            println!("  Anthropic model: {}", config.llm.anthropic.model);
            if !config.llm.custom.endpoint.is_empty() {
                println!("  Custom endpoint: {}", config.llm.custom.endpoint);
                println!("  Custom model: {}", config.llm.custom.model);
            }

            let config_path = AppConfig::config_path()?;
            println!("\nConfiguration file: {}", config_path.display());
            println!("\nTo edit: y2md config edit");
            println!("Or edit directly: {}", config_path.display());
        }
        ConfigCommands::Edit => {
            let config_path = AppConfig::config_path()?;

            // Create config if it doesn't exist
            if !config_path.exists() {
                let config = AppConfig::default();
                config.save()?;
                println!("Created default configuration file");
            }

            // Open in editor
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            let status = std::process::Command::new(&editor)
                .arg(&config_path)
                .status()?;

            if !status.success() {
                anyhow::bail!("Editor exited with error");
            }

            // Validate the edited config
            match AppConfig::load() {
                Ok(_) => println!("✓ Configuration is valid"),
                Err(e) => {
                    eprintln!("✗ Configuration has errors: {}", e);
                    eprintln!("Please fix the errors in: {}", config_path.display());
                    anyhow::bail!("Invalid configuration");
                }
            }
        }
        ConfigCommands::Path => {
            let config_path = AppConfig::config_path()?;
            println!("{}", config_path.display());
        }
        ConfigCommands::Reset => {
            let default_config = AppConfig::default();
            default_config.save()?;
            println!("✓ Configuration reset to defaults");
            let config_path = AppConfig::config_path()?;
            println!("  Location: {}", config_path.display());
        }
    }
    Ok(())
}

/// Handle LLM management commands
async fn handle_llm_command(command: LlmCommands) -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let ollama_manager = OllamaManager::new(Some(config.llm.local.endpoint.clone()));
    let cred_manager = CredentialManager::new();

    match command {
        LlmCommands::List => {
            println!("Checking local Ollama models...");

            if !ollama_manager.is_available().await {
                anyhow::bail!(
                    "Ollama service is not available at: {}\nMake sure Ollama is running",
                    config.llm.local.endpoint
                );
            }

            match ollama_manager.get_local_models().await {
                Ok(models) => {
                    if models.is_empty() {
                        println!("No local models found.");
                        println!("\nTo download a model, use: y2md llm pull <model-name>");
                    } else {
                        println!("Local models ({} total):", models.len());
                        for model in models {
                            let marker = if model.contains(&config.llm.local.model) {
                                " (configured)"
                            } else {
                                ""
                            };
                            println!("  - {}{}", model, marker);
                        }
                    }
                }
                Err(e) => {
                    anyhow::bail!("Failed to list models: {}", e);
                }
            }
        }
        LlmCommands::Pull { model } => {
            println!("Downloading model: {}", model);

            if !ollama_manager.is_available().await {
                anyhow::bail!(
                    "Ollama service is not available at: {}\nMake sure Ollama is running",
                    config.llm.local.endpoint
                );
            }

            // Check if model already exists
            if ollama_manager.is_model_available(&model).await? {
                println!("✓ Model '{}' is already available", model);
                return Ok(());
            }

            println!(
                "\n⚠️  This will download '{}' from Ollama's library.",
                model
            );
            println!("   This may take several minutes. Continue? [y/N]");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") && !input.trim().eq_ignore_ascii_case("yes")
            {
                println!("Download cancelled.");
                return Ok(());
            }

            println!("\n📥 Downloading model...");
            match ollama_manager.download_model(&model).await {
                Ok(()) => {
                    println!("✓ Model '{}' downloaded successfully", model);
                }
                Err(e) => {
                    anyhow::bail!("Download failed: {}", e);
                }
            }
        }
        LlmCommands::Remove { model } => {
            println!("⚠️  This will permanently remove the model '{}'.", model);
            println!("   Continue? [y/N]");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") && !input.trim().eq_ignore_ascii_case("yes")
            {
                println!("Removal cancelled.");
                return Ok(());
            }

            match ollama_manager.remove_model(&model).await {
                Ok(()) => {
                    println!("✓ Model '{}' removed successfully", model);
                }
                Err(e) => {
                    anyhow::bail!("Removal failed: {}", e);
                }
            }
        }
        LlmCommands::Test { provider } => {
            let provider_type = if let Some(p) = provider {
                p.parse::<LlmProviderType>()
                    .map_err(|e| anyhow::anyhow!("Invalid provider: {}", e))?
            } else {
                config.llm.provider.clone()
            };

            println!("Testing provider: {}", provider_type);

            let test_transcript =
                "This is a test transcript to verify the LLM connection is working properly.";

            match y2md::format_with_llm(test_transcript, Some(provider_type)).await {
                Ok(result) => {
                    println!("✓ Provider test successful!");
                    println!("\nTest output preview:");
                    println!("{}", &result[..result.len().min(200)]);
                    if result.len() > 200 {
                        println!("...");
                    }
                }
                Err(e) => {
                    anyhow::bail!("Provider test failed: {}", e);
                }
            }
        }
        LlmCommands::SetKey { provider } => {
            let provider_type = provider.parse::<LlmProviderType>().map_err(|e| {
                anyhow::anyhow!(
                    "Invalid provider: {}. Valid providers: openai, anthropic, custom",
                    e
                )
            })?;

            if provider_type == LlmProviderType::Local {
                anyhow::bail!("Local provider (Ollama) does not require an API key");
            }

            print!("Enter API key for '{}': ", provider);
            std::io::stdout().flush()?;

            let key = rpassword::read_password()?;

            if key.trim().is_empty() {
                anyhow::bail!("API key cannot be empty");
            }

            cred_manager.set_api_key(&provider_type, &key)?;
            println!("✓ API key set for provider '{}'", provider);
            println!("\nThe API key is securely stored in your system keychain.");
        }
    }

    Ok(())
}
