use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Input, Select};
use y2md::{
    AnthropicConfig, AppConfig, CredentialManager, CustomLlmConfig, DeepSeekConfig,
    LlmProviderType, LlmSettings, LocalLlmConfig, OllamaManager, OpenAiConfig,
};

pub struct SetupWizard;

impl SetupWizard {
    pub async fn run() -> Result<AppConfig> {
        println!("\n{}", style("Welcome to y2md Setup! 🎉").bold().cyan());
        println!("{}", style("Let's configure y2md for your needs.\n").dim());

        let output_dir = Self::prompt_output_directory()?;
        let default_language = Self::prompt_default_language()?;
        let llm_settings = Self::prompt_llm_setup().await?;

        let config = AppConfig {
            output_dir,
            default_language,
            llm: llm_settings,
            prefer_captions: true,
            timestamps: false,
            compact: false,
            paragraph_length: 4,
            ..Default::default()
        };

        config.save()?;

        println!("\n{}", style("✓ Setup complete!").bold().green());
        println!(
            "Configuration saved to: {}",
            AppConfig::config_path()?.display()
        );
        println!("\n{}", style("Next steps:").bold());
        println!("  1. Test your setup: {}", style("y2md doctor").cyan());
        println!(
            "  2. Transcribe a video: {}",
            style("y2md <YOUTUBE_URL>").cyan()
        );

        Ok(config)
    }

    fn prompt_output_directory() -> Result<String> {
        println!("{}", style("Output Directory").bold());
        println!("Where should transcripts be saved?");

        let default_dir = dirs::document_dir()
            .and_then(|d| d.join("y2md-transcripts").to_str().map(String::from))
            .unwrap_or_else(|| "./transcripts".to_string());

        let output_dir: String = Input::new()
            .with_prompt("Output directory")
            .default(default_dir.clone())
            .interact_text()?;

        let expanded = shellexpand::tilde(&output_dir).to_string();

        if !std::path::Path::new(&expanded).exists() {
            if Confirm::new()
                .with_prompt(format!(
                    "Directory '{}' doesn't exist. Create it?",
                    expanded
                ))
                .default(true)
                .interact()?
            {
                std::fs::create_dir_all(&expanded)?;
                println!("  {} Created directory", style("✓").green());
            }
        }

        println!();
        Ok(output_dir)
    }

    fn prompt_default_language() -> Result<String> {
        println!("{}", style("Default Language").bold());
        println!("Which language will you transcribe most often?");

        let languages = vec![
            "English (en)",
            "Spanish (es)",
            "French (fr)",
            "German (de)",
            "Italian (it)",
            "Portuguese (pt)",
            "Russian (ru)",
            "Japanese (ja)",
            "Chinese (zh)",
            "Korean (ko)",
            "Other",
        ];

        let selection = Select::new()
            .with_prompt("Select language")
            .items(&languages)
            .default(0)
            .interact()?;

        let lang_code = match selection {
            0 => "en",
            1 => "es",
            2 => "fr",
            3 => "de",
            4 => "it",
            5 => "pt",
            6 => "ru",
            7 => "ja",
            8 => "zh",
            9 => "ko",
            10 => {
                let custom: String = Input::new()
                    .with_prompt("Enter language code (e.g., 'ar' for Arabic)")
                    .interact_text()?;
                return Ok(custom);
            }
            _ => "en",
        };

        println!();
        Ok(lang_code.to_string())
    }

    async fn prompt_llm_setup() -> Result<LlmSettings> {
        println!("{}", style("LLM Formatting (Optional)").bold());
        println!("LLMs can improve transcript readability by fixing grammar,");
        println!("removing filler words, and organizing content.\n");

        let providers = vec![
            "Local (Ollama) - Free, private, runs on your machine",
            "OpenAI - Fast, high quality (~$0.01-0.02 per video)",
            "Anthropic Claude - Excellent quality (~$0.015 per video)",
            "DeepSeek - Good quality, competitive pricing (~$0.008 per video)",
            "Custom - Any OpenAI-compatible API",
            "None - Use standard formatting (no LLM)",
        ];

        let selection = Select::new()
            .with_prompt("Choose your LLM provider")
            .items(&providers)
            .default(5)
            .interact()?;

        println!();

        match selection {
            0 => Self::setup_ollama().await,
            1 => Self::setup_openai().await,
            2 => Self::setup_anthropic().await,
            3 => Self::setup_deepseek().await,
            4 => Self::setup_custom().await,
            5 => {
                println!("  {} LLM formatting disabled", style("ℹ").cyan());
                println!(
                    "  You can enable it later with: {}\n",
                    style("y2md setup-llm").cyan()
                );
                Ok(LlmSettings {
                    enabled: false,
                    ..Default::default()
                })
            }
            _ => Ok(LlmSettings::default()),
        }
    }

    async fn setup_ollama() -> Result<LlmSettings> {
        println!("{}", style("Setting up Ollama (Local LLM)").bold());
        println!();

        let ollama = OllamaManager::new(Some("http://localhost:11434".to_string()));

        if !ollama.is_available().await {
            println!(
                "  {} Ollama is not running or not installed.",
                style("⚠").yellow()
            );
            println!();

            let actions = vec![
                "Install Ollama (opens browser)",
                "I've already installed it, let me start it",
                "Skip for now",
            ];

            let choice = Select::new()
                .with_prompt("What would you like to do?")
                .items(&actions)
                .interact()?;

            match choice {
                0 => {
                    println!("\n  Opening https://ollama.ai in your browser...");
                    let _ = open::that("https://ollama.ai");
                    println!(
                        "\n  After installing Ollama, run: {}",
                        style("y2md init").cyan()
                    );
                    return Err(anyhow::anyhow!("Please install Ollama first"));
                }
                1 => {
                    println!("\n  Waiting for Ollama to start...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                    if !ollama.is_available().await {
                        println!("  {} Could not connect to Ollama", style("✗").red());
                        println!(
                            "  Start Ollama manually, then run: {}",
                            style("y2md init").cyan()
                        );
                        return Err(anyhow::anyhow!("Ollama not available"));
                    }
                }
                2 => {
                    println!("  {} Skipping Ollama setup\n", style("ℹ").cyan());
                    return Ok(LlmSettings {
                        enabled: false,
                        ..Default::default()
                    });
                }
                _ => unreachable!(),
            }
        }

        println!("  {} Ollama is running", style("✓").green());
        println!();

        let models = ollama.get_local_models().await.unwrap_or_default();

        if models.is_empty() {
            println!("No models installed yet.");
            println!("Recommended models:");
            println!(
                "  • {} - Fast, small download (2GB)",
                style("llama3.2:3b").cyan()
            );
            println!(
                "  • {} - High quality (7GB)",
                style("mistral-nemo:12b").cyan()
            );
            println!();

            let model_choices = vec![
                "llama3.2:3b - Fast, 2GB download",
                "mistral-nemo:12b-instruct-2407-q5_0 - High quality, 7GB",
                "I'll download a model later",
            ];

            let model_choice = Select::new()
                .with_prompt("Select model")
                .items(&model_choices)
                .interact()?;

            let model_name = match model_choice {
                0 => "llama3.2:3b",
                1 => "mistral-nemo:12b-instruct-2407-q5_0",
                2 => {
                    println!(
                        "\n  {} Download a model later with: {}",
                        style("ℹ").cyan(),
                        style("y2md llm pull <model-name>").cyan()
                    );
                    return Ok(LlmSettings {
                        enabled: false,
                        ..Default::default()
                    });
                }
                _ => unreachable!(),
            };

            println!("\n  Downloading {}...", style(model_name).cyan());
            println!("  This may take several minutes depending on your connection.\n");

            if let Err(e) = ollama.download_model(model_name).await {
                println!("  {} Failed to download model: {}", style("✗").red(), e);
                println!(
                    "  Try downloading manually: {}",
                    style(format!("ollama pull {}", model_name)).cyan()
                );
                return Ok(LlmSettings {
                    enabled: false,
                    ..Default::default()
                });
            }

            println!("\n  {} Model downloaded successfully", style("✓").green());

            Ok(LlmSettings {
                enabled: true,
                provider: LlmProviderType::Local,
                local: LocalLlmConfig {
                    endpoint: "http://localhost:11434".to_string(),
                    model: model_name.to_string(),
                },
                ..Default::default()
            })
        } else {
            println!("Available models:");
            for model in &models {
                println!("  • {}", style(model).cyan());
            }
            println!();

            let model_idx = Select::new()
                .with_prompt("Select model to use")
                .items(&models)
                .default(0)
                .interact()?;

            println!(
                "  {} Using model: {}",
                style("✓").green(),
                style(&models[model_idx]).cyan()
            );
            println!();

            Ok(LlmSettings {
                enabled: true,
                provider: LlmProviderType::Local,
                local: LocalLlmConfig {
                    endpoint: "http://localhost:11434".to_string(),
                    model: models[model_idx].clone(),
                },
                ..Default::default()
            })
        }
    }

    async fn setup_openai() -> Result<LlmSettings> {
        println!("{}", style("Setting up OpenAI").bold());
        println!();
        println!("You'll need an OpenAI API key from: https://platform.openai.com/api-keys");
        println!();

        let api_key: String = Input::new().with_prompt("OpenAI API Key").interact_text()?;

        if api_key.trim().is_empty() {
            return Err(anyhow::anyhow!("API key cannot be empty"));
        }

        println!("\n  Testing API key...");

        let client = reqwest::Client::new();
        let response = client
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", api_key.trim()))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                println!("  {} API key is valid", style("✓").green());
            }
            Ok(resp) => {
                println!(
                    "  {} Invalid API key or API error: {}",
                    style("✗").red(),
                    resp.status()
                );
                return Err(anyhow::anyhow!("Invalid API key"));
            }
            Err(e) => {
                println!("  {} Could not connect to OpenAI: {}", style("✗").red(), e);
                return Err(anyhow::anyhow!("Connection error"));
            }
        }

        let cred_manager = CredentialManager::new();
        cred_manager.set_api_key(&LlmProviderType::OpenAI, api_key.trim())?;

        let models = vec![
            "gpt-4o - Latest, best quality",
            "gpt-4-turbo-preview - Fast and capable",
            "gpt-3.5-turbo - Fastest, cheapest",
        ];

        let model_choice = Select::new()
            .with_prompt("Select model")
            .items(&models)
            .default(0)
            .interact()?;

        let model_name = match model_choice {
            0 => "gpt-4o",
            1 => "gpt-4-turbo-preview",
            2 => "gpt-3.5-turbo",
            _ => "gpt-4o",
        };

        println!(
            "\n  {} OpenAI configured with {}",
            style("✓").green(),
            style(model_name).cyan()
        );
        println!();

        Ok(LlmSettings {
            enabled: true,
            provider: LlmProviderType::OpenAI,
            openai: OpenAiConfig {
                endpoint: "https://api.openai.com/v1".to_string(),
                model: model_name.to_string(),
            },
            ..Default::default()
        })
    }

    async fn setup_anthropic() -> Result<LlmSettings> {
        println!("{}", style("Setting up Anthropic Claude").bold());
        println!();
        println!("You'll need an Anthropic API key from: https://console.anthropic.com/");
        println!();

        let api_key: String = Input::new()
            .with_prompt("Anthropic API Key")
            .interact_text()?;

        if api_key.trim().is_empty() {
            return Err(anyhow::anyhow!("API key cannot be empty"));
        }

        println!("\n  Testing API key...");

        let client = reqwest::Client::new();
        let test_body = serde_json::json!({
            "model": "claude-3-haiku-20240307",
            "max_tokens": 10,
            "messages": [{"role": "user", "content": "Hi"}]
        });

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("anthropic-version", "2023-06-01")
            .header("x-api-key", api_key.trim())
            .json(&test_body)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                println!("  {} API key is valid", style("✓").green());
            }
            Ok(resp) => {
                println!(
                    "  {} Invalid API key or API error: {}",
                    style("✗").red(),
                    resp.status()
                );
                return Err(anyhow::anyhow!("Invalid API key"));
            }
            Err(e) => {
                println!(
                    "  {} Could not connect to Anthropic: {}",
                    style("✗").red(),
                    e
                );
                return Err(anyhow::anyhow!("Connection error"));
            }
        }

        let cred_manager = CredentialManager::new();
        cred_manager.set_api_key(&LlmProviderType::Anthropic, api_key.trim())?;

        let models = vec![
            "claude-3-opus-20240229 - Most capable",
            "claude-3-sonnet-20240229 - Balanced (recommended)",
            "claude-3-haiku-20240307 - Fast and efficient",
        ];

        let model_choice = Select::new()
            .with_prompt("Select model")
            .items(&models)
            .default(1)
            .interact()?;

        let model_name = match model_choice {
            0 => "claude-3-opus-20240229",
            1 => "claude-3-sonnet-20240229",
            2 => "claude-3-haiku-20240307",
            _ => "claude-3-sonnet-20240229",
        };

        println!(
            "\n  {} Anthropic configured with {}",
            style("✓").green(),
            style(model_name).cyan()
        );
        println!();

        Ok(LlmSettings {
            enabled: true,
            provider: LlmProviderType::Anthropic,
            anthropic: AnthropicConfig {
                endpoint: "https://api.anthropic.com/v1".to_string(),
                model: model_name.to_string(),
            },
            ..Default::default()
        })
    }

    async fn setup_deepseek() -> Result<LlmSettings> {
        println!("{}", style("Setting up DeepSeek").bold());
        println!();
        println!("You'll need a DeepSeek API key from: https://platform.deepseek.com/");
        println!();

        let api_key: String = Input::new()
            .with_prompt("DeepSeek API Key")
            .interact_text()?;

        if api_key.trim().is_empty() {
            return Err(anyhow::anyhow!("API key cannot be empty"));
        }

        let cred_manager = CredentialManager::new();
        cred_manager.set_api_key(&LlmProviderType::DeepSeek, api_key.trim())?;

        println!("\n  {} DeepSeek configured", style("✓").green());
        println!();

        Ok(LlmSettings {
            enabled: true,
            provider: LlmProviderType::DeepSeek,
            deepseek: DeepSeekConfig {
                endpoint: "https://api.deepseek.com/v1".to_string(),
                model: "deepseek-chat".to_string(),
            },
            ..Default::default()
        })
    }

    async fn setup_custom() -> Result<LlmSettings> {
        println!(
            "{}",
            style("Setting up Custom OpenAI-compatible API").bold()
        );
        println!();

        let endpoint: String = Input::new()
            .with_prompt("API Endpoint URL")
            .interact_text()?;

        let model: String = Input::new().with_prompt("Model name").interact_text()?;

        let needs_key = Confirm::new()
            .with_prompt("Does this API require an API key?")
            .default(true)
            .interact()?;

        if needs_key {
            let api_key: String = Input::new().with_prompt("API Key").interact_text()?;

            let cred_manager = CredentialManager::new();
            cred_manager.set_api_key(&LlmProviderType::Custom, api_key.trim())?;
        }

        println!("\n  {} Custom API configured", style("✓").green());
        println!();

        Ok(LlmSettings {
            enabled: true,
            provider: LlmProviderType::Custom,
            custom: CustomLlmConfig { endpoint, model },
            ..Default::default()
        })
    }
}
