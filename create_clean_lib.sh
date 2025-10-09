#!/bin/bash

# This script creates a clean lib.rs by extracting only what we need

cat > src/lib.rs << 'ENDOFFILE'
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;
use url::form_urlencoded;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub title: String,
    pub channel: Option<String>,
    pub duration: Option<String>,
    pub video_id: String,
    pub url: String,
}

#[derive(Error, Debug)]
pub enum Y2mdError {
    #[error("Invalid YouTube URL: {0}")]
    InvalidUrl(String),
    #[error("Failed to extract video ID from URL")]
    VideoIdExtraction,
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Whisper error: {0}")]
    Whisper(String),
    #[error("LLM error: {0}")]
    Llm(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProviderType {
    Local,
    OpenAI,
    Anthropic,
    Custom,
}

impl Default for LlmProviderType {
    fn default() -> Self {
        LlmProviderType::Local
    }
}

impl std::fmt::Display for LlmProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmProviderType::Local => write!(f, "local"),
            LlmProviderType::OpenAI => write!(f, "openai"),
            LlmProviderType::Anthropic => write!(f, "anthropic"),
            LlmProviderType::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for LlmProviderType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local" => Ok(LlmProviderType::Local),
            "openai" => Ok(LlmProviderType::OpenAI),
            "anthropic" => Ok(LlmProviderType::Anthropic),
            "custom" => Ok(LlmProviderType::Custom),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalLlmConfig {
    pub endpoint: String,
    pub model: String,
}

impl Default for LocalLlmConfig {
    fn default() -> Self {
        LocalLlmConfig {
            endpoint: "http://localhost:11434".to_string(),
            model: "mistral-nemo:12b-instruct-2407-q5_0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub endpoint: String,
    pub model: String,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        OpenAiConfig {
            endpoint: "https://api.openai.com/v1".to_string(),
            model: "gpt-4-turbo-preview".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub endpoint: String,
    pub model: String,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        AnthropicConfig {
            endpoint: "https://api.anthropic.com/v1".to_string(),
            model: "claude-3-sonnet-20240229".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomLlmConfig {
    pub endpoint: String,
    pub model: String,
}

impl Default for CustomLlmConfig {
    fn default() -> Self {
        CustomLlmConfig {
            endpoint: "".to_string(),
            model: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSettings {
    pub enabled: bool,
    pub provider: LlmProviderType,
    pub local: LocalLlmConfig,
    pub openai: OpenAiConfig,
    pub anthropic: AnthropicConfig,
    pub custom: CustomLlmConfig,
}

impl Default for LlmSettings {
    fn default() -> Self {
        LlmSettings {
            enabled: false,
            provider: LlmProviderType::Local,
            local: LocalLlmConfig::default(),
            openai: OpenAiConfig::default(),
            anthropic: AnthropicConfig::default(),
            custom: CustomLlmConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSettings {
    pub whisper_model: String,
    pub whisper_threads: usize,
    pub cache_audio: bool,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        AdvancedSettings {
            whisper_model: "base".to_string(),
            whisper_threads: 4,
            cache_audio: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub output_dir: String,
    pub default_language: String,
    pub prefer_captions: bool,
    pub timestamps: bool,
    pub compact: bool,
    pub paragraph_length: usize,
    pub llm: LlmSettings,
    pub advanced: AdvancedSettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            output_dir: ".".to_string(),
            default_language: "en".to_string(),
            prefer_captions: true,
            timestamps: false,
            compact: false,
            paragraph_length: 4,
            llm: LlmSettings::default(),
            advanced: AdvancedSettings::default(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, Y2mdError> {
        let config_dir = directories::ProjectDirs::from("com", "y2md", "y2md")
            .ok_or_else(|| Y2mdError::Config("Could not determine config directory".to_string()))?;

        let config_path = config_dir.config_dir().join("config.toml");

        if !config_path.exists() {
            return Ok(AppConfig::default());
        }

        let config_content = std::fs::read_to_string(&config_path)
            .map_err(|e| Y2mdError::Config(format!("Failed to read config file: {}", e)))?;

        toml::from_str::<AppConfig>(&config_content)
            .map_err(|e| Y2mdError::Config(format!("Failed to parse config: {}\n\nPlease check your config file at: {}", e, config_path.display())))
    }

    pub fn save(&self) -> Result<(), Y2mdError> {
        let config_dir = directories::ProjectDirs::from("com", "y2md", "y2md")
            .ok_or_else(|| Y2mdError::Config("Could not determine config directory".to_string()))?;

        std::fs::create_dir_all(config_dir.config_dir())
            .map_err(|e| Y2mdError::Config(format!("Failed to create config directory: {}", e)))?;

        let config_path = config_dir.config_dir().join("config.toml");
        
        let header = r#"# =============================================================================
# Y2MD Configuration
# Edit this file directly or use: y2md config edit
# =============================================================================

"#;
        
        let config_toml = toml::to_string_pretty(self)
            .map_err(|e| Y2mdError::Config(format!("Failed to serialize configuration: {}", e)))?;

        std::fs::write(&config_path, format!("{}{}", header, config_toml))
            .map_err(|e| Y2mdError::Config(format!("Failed to write configuration file: {}", e)))?;

        Ok(())
    }

    pub fn config_path() -> Result<PathBuf, Y2mdError> {
        let config_dir = directories::ProjectDirs::from("com", "y2md", "y2md")
            .ok_or_else(|| Y2mdError::Config("Could not determine config directory".to_string()))?;

        Ok(config_dir.config_dir().join("config.toml"))
    }
}

pub struct CredentialManager {
    service_name: String,
}

impl CredentialManager {
    pub fn new() -> Self {
        Self {
            service_name: "y2md".to_string(),
        }
    }

    pub fn get_api_key(&self, provider_type: &LlmProviderType) -> Result<Option<String>, Y2mdError> {
        let provider_name = provider_type.to_string();
        let env_var_name = format!("Y2MD_{}_API_KEY", provider_name.to_uppercase());
        
        if let Ok(key) = std::env::var(&env_var_name) {
            return Ok(Some(key));
        }

        let entry = keyring::Entry::new(&self.service_name, &provider_name)
            .map_err(|e| Y2mdError::Config(format!("Failed to access keyring: {}", e)))?;

        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(Y2mdError::Config(format!(
                "Failed to retrieve API key from keyring: {}",
                e
            ))),
        }
    }

    pub fn set_api_key(&self, provider_type: &LlmProviderType, api_key: &str) -> Result<(), Y2mdError> {
        let provider_name = provider_type.to_string();
        let entry = keyring::Entry::new(&self.service_name, &provider_name)
            .map_err(|e| Y2mdError::Config(format!("Failed to access keyring: {}", e)))?;

        entry
            .set_password(api_key)
            .map_err(|e| Y2mdError::Config(format!("Failed to store API key in keyring: {}", e)))?;

        Ok(())
    }

    pub fn delete_api_key(&self, provider_type: &LlmProviderType) -> Result<(), Y2mdError> {
        let provider_name = provider_type.to_string();
        let entry = keyring::Entry::new(&self.service_name, &provider_name)
            .map_err(|e| Y2mdError::Config(format!("Failed to access keyring: {}", e)))?;

        match entry.delete_password() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(Y2mdError::Config(format!(
                "Failed to delete API key from keyring: {}",
                e
            ))),
        }
    }

    pub fn has_api_key(&self, provider_type: &LlmProviderType) -> bool {
        self.get_api_key(provider_type).ok().flatten().is_some()
    }
}

ENDOFFILE

# Now append the working functions from the backup
tail -n +345 src/lib.rs.backup | head -n 1400 >> src/lib.rs

echo "Clean lib.rs created successfully"

