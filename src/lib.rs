use config::{Config, File, FileFormat};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    #[error("LLM configuration error: {0}")]
    LlmConfig(String),
    #[error("Config parsing error: {0}")]
    ConfigParse(#[from] config::ConfigError),
}

/// LLM Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LlmProvider {
    #[serde(rename = "ollama")]
    #[default]
    Ollama,
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "lmstudio")]
    LMStudio,
    #[serde(rename = "custom")]
    Custom,
}

impl std::fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmProvider::Ollama => write!(f, "ollama"),
            LlmProvider::OpenAI => write!(f, "openai"),
            LlmProvider::Anthropic => write!(f, "anthropic"),
            LlmProvider::LMStudio => write!(f, "lmstudio"),
            LlmProvider::Custom => write!(f, "custom"),
        }
    }
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub model: String,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        LlmConfig {
            provider: LlmProvider::Ollama,
            model: "mistral-nemo:12b-instruct-2407-q5_0".to_string(),
            endpoint: None,
            api_key: None,
        }
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub llm: LlmConfig,
    pub active_provider: Option<String>,
    pub providers: HashMap<String, ProviderConfig>,
    pub prefer_captions: bool,
    pub default_language: String,
    pub output_dir: Option<String>,
    pub timestamps: bool,
    pub compact: bool,
    pub paragraph_length: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            llm: LlmConfig::default(),
            active_provider: None,
            providers: HashMap::new(),
            prefer_captions: true,
            default_language: "en".to_string(),
            output_dir: None,
            timestamps: false,
            compact: false,
            paragraph_length: 4,
        }
    }
}

impl AppConfig {
    /// Load configuration from file or return default
    pub fn load() -> Result<Self, Y2mdError> {
        let config_dir = directories::ProjectDirs::from("com", "y2md", "y2md")
            .ok_or_else(|| Y2mdError::Config("Could not determine config directory".to_string()))?;

        let config_path = config_dir.config_dir().join("config.toml");

        let mut config_builder = Config::builder();

        // Add default configuration
        config_builder =
            config_builder.add_source(config::Config::try_from(&AppConfig::default())?);

        // Add configuration file if it exists
        if config_path.exists() {
            config_builder =
                config_builder.add_source(File::from(config_path).format(FileFormat::Toml));
        }

        let config = config_builder.build()?;

        config
            .try_deserialize()
            .map_err(|e| Y2mdError::Config(format!("Failed to parse configuration: {}", e)))
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), Y2mdError> {
        let config_dir = directories::ProjectDirs::from("com", "y2md", "y2md")
            .ok_or_else(|| Y2mdError::Config("Could not determine config directory".to_string()))?;

        // Create config directory if it doesn't exist
        std::fs::create_dir_all(config_dir.config_dir())
            .map_err(|e| Y2mdError::Config(format!("Failed to create config directory: {}", e)))?;

        let config_path = config_dir.config_dir().join("config.toml");
        let config_toml = toml::to_string_pretty(self)
            .map_err(|e| Y2mdError::Config(format!("Failed to serialize configuration: {}", e)))?;

        std::fs::write(&config_path, config_toml)
            .map_err(|e| Y2mdError::Config(format!("Failed to write configuration file: {}", e)))?;

        Ok(())
    }

    /// Get configuration file path
    pub fn config_path() -> Result<PathBuf, Y2mdError> {
        let config_dir = directories::ProjectDirs::from("com", "y2md", "y2md")
            .ok_or_else(|| Y2mdError::Config("Could not determine config directory".to_string()))?;

        Ok(config_dir.config_dir().join("config.toml"))
    }

    pub fn add_provider(&mut self, provider: ProviderConfig) -> Result<(), Y2mdError> {
        if self.providers.contains_key(&provider.name) {
            return Err(Y2mdError::Config(format!(
                "Provider '{}' already exists",
                provider.name
            )));
        }
        self.providers.insert(provider.name.clone(), provider);
        Ok(())
    }

    pub fn remove_provider(&mut self, name: &str) -> Result<(), Y2mdError> {
        if !self.providers.contains_key(name) {
            return Err(Y2mdError::Config(format!("Provider '{}' not found", name)));
        }

        if self.active_provider.as_deref() == Some(name) {
            self.active_provider = None;
        }

        self.providers.remove(name);
        Ok(())
    }

    pub fn get_provider(&self, name: &str) -> Result<&ProviderConfig, Y2mdError> {
        self.providers
            .get(name)
            .ok_or_else(|| Y2mdError::Config(format!("Provider '{}' not found", name)))
    }

    pub fn get_active_provider(&self) -> Result<&ProviderConfig, Y2mdError> {
        if let Some(active_name) = &self.active_provider {
            self.get_provider(active_name)
        } else {
            Err(Y2mdError::Config("No active provider set".to_string()))
        }
    }

    pub fn set_active_provider(&mut self, name: &str) -> Result<(), Y2mdError> {
        if !self.providers.contains_key(name) {
            return Err(Y2mdError::Config(format!("Provider '{}' not found", name)));
        }
        self.active_provider = Some(name.to_string());
        Ok(())
    }

    pub fn list_providers(&self) -> Vec<&ProviderConfig> {
        self.providers.values().collect()
    }

    pub fn get_llm_config_for_provider(
        &self,
        provider: &ProviderConfig,
        cred_manager: &CredentialManager,
    ) -> Result<LlmConfig, Y2mdError> {
        let mut api_key = None;

        if let Some(token) = cred_manager.get_oauth_token(&provider.name)? {
            if !token.is_expired() {
                api_key = Some(token.access_token);
            } else if token.refresh_token.is_some() {
                return Err(Y2mdError::Config(format!(
                    "OAuth token expired for provider '{}'. Please login again: y2md auth login {}",
                    provider.name, provider.name
                )));
            }
        }

        if api_key.is_none() {
            api_key = cred_manager.get_api_key(&provider.name)?;
        }

        Ok(LlmConfig {
            provider: provider.provider_type.clone(),
            model: provider.model.clone(),
            endpoint: provider.endpoint.clone(),
            api_key,
        })
    }
}

/// Extract video ID from various YouTube URL formats
pub fn extract_video_id(url: &str) -> Result<String, Y2mdError> {
    let url = url.trim();

    // Handle youtu.be short URLs
    if url.contains("youtu.be/") {
        if let Some(id) = url.split("youtu.be/").nth(1) {
            return Ok(id.split('?').next().unwrap_or(id).to_string());
        }
    }

    // Handle youtube.com URLs
    if url.contains("youtube.com") {
        let parsed_url =
            reqwest::Url::parse(url).map_err(|_| Y2mdError::InvalidUrl(url.to_string()))?;

        // Handle /watch?v= format
        if let Some(query) = parsed_url.query() {
            let params: HashMap<_, _> = form_urlencoded::parse(query.as_bytes()).collect();
            if let Some(v) = params.get("v") {
                return Ok(v.to_string());
            }
        }

        // Handle /shorts/ format
        if let Some(segments) = parsed_url.path_segments() {
            let segments: Vec<_> = segments.collect();
            if segments.len() == 2 && segments[0] == "shorts" {
                return Ok(segments[1].to_string());
            }
        }
    }

    // Handle direct video ID (11 characters, alphanumeric + underscore)
    if url.len() == 11
        && url
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Ok(url.to_string());
    }

    Err(Y2mdError::VideoIdExtraction)
}

/// Validate YouTube URL format
pub fn validate_youtube_url(url: &str) -> Result<String, Y2mdError> {
    let video_id = extract_video_id(url)?;

    // YouTube video IDs are typically 11 characters
    if video_id.len() != 11 {
        return Err(Y2mdError::InvalidUrl(format!(
            "Invalid video ID length: {}",
            video_id
        )));
    }

    Ok(video_id)
}

/// Fetch video metadata from YouTube
pub async fn fetch_video_metadata(video_id: &str) -> Result<VideoMetadata, Y2mdError> {
    let url = format!("https://www.youtube.com/watch?v={}", video_id);

    // Use yt-dlp to get video metadata
    let output = Command::new("yt-dlp")
        .args(["--dump-json", "--no-download", &url])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Y2mdError::Config(
                    "yt-dlp not found. Please install yt-dlp: https://github.com/yt-dlp/yt-dlp"
                        .to_string(),
                )
            } else {
                Y2mdError::Io(e)
            }
        })?;

    if !output.status.success() {
        return Err(Y2mdError::Config(
            "Failed to fetch metadata with yt-dlp".to_string(),
        ));
    }

    // Parse JSON output
    let metadata_json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| Y2mdError::Config(format!("Failed to parse metadata JSON: {}", e)))?;

    // Extract fields from JSON
    let title = metadata_json["title"]
        .as_str()
        .unwrap_or("Unknown Title")
        .to_string();

    let channel = metadata_json["uploader"].as_str().map(|s| s.to_string());

    let duration_seconds = metadata_json["duration"].as_f64().unwrap_or(0.0);

    let duration = if duration_seconds > 0.0 {
        Some(format_duration(duration_seconds))
    } else {
        None
    };

    let metadata = VideoMetadata {
        title,
        channel,
        duration,
        video_id: video_id.to_string(),
        url,
    };

    Ok(metadata)
}

/// Format duration in seconds to HH:MM:SS
fn format_duration(seconds: f64) -> String {
    let total_seconds = seconds as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

/// Check if captions are available for a video
pub async fn check_captions_available(video_id: &str) -> Result<bool, Y2mdError> {
    let url = format!("https://www.youtube.com/watch?v={}", video_id);

    // Use yt-dlp to list available captions
    let output = Command::new("yt-dlp")
        .args(["--list-subs", "--no-download", &url])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Y2mdError::Config(
                    "yt-dlp not found. Please install yt-dlp: https://github.com/yt-dlp/yt-dlp"
                        .to_string(),
                )
            } else {
                Y2mdError::Io(e)
            }
        })?;

    if !output.status.success() {
        return Ok(false);
    }

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Check if there are any available captions
    // Look for language codes in the output - both automatic and manual captions
    Ok(output_str.contains("Available subtitles")
        && output_str
            .lines()
            .any(|line| line.contains("en") || line.contains("English")))
}

/// Extract captions from YouTube video
pub async fn extract_captions(
    video_id: &str,
    language: Option<&str>,
    force_formatting: bool,
) -> Result<(String, String), Y2mdError> {
    let url = format!("https://www.youtube.com/watch?v={}", video_id);
    let lang = language.unwrap_or("en");

    // Use yt-dlp to download captions
    let output = Command::new("yt-dlp")
        .args([
            "--write-sub",
            "--write-auto-sub",
            "--sub-lang",
            lang,
            "--skip-download",
            "--convert-subs",
            "srt",
            "-o",
            "%(id)s_captions",
            &url,
        ])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Y2mdError::Config(
                    "yt-dlp not found. Please install yt-dlp: https://github.com/yt-dlp/yt-dlp"
                        .to_string(),
                )
            } else {
                Y2mdError::Io(e)
            }
        })?;

    if !output.status.success() {
        return Err(Y2mdError::Config("Failed to extract captions".to_string()));
    }

    // Look for the generated caption file
    let caption_filename = format!("{}_captions.{}.srt", video_id, lang);

    if !std::path::Path::new(&caption_filename).exists() {
        return Err(Y2mdError::Config(
            "Caption file not found after extraction".to_string(),
        ));
    }

    // Read the caption file
    let caption_content = std::fs::read_to_string(&caption_filename)?;

    // Clean up the temporary file
    let _ = std::fs::remove_file(&caption_filename);

    // Convert SRT to plain text
    let raw_text = srt_to_plain_text(&caption_content);

    // Only apply enhanced formatting if the text doesn't contain music notation
    // or other special formatting that should be preserved
    let formatted_text = if force_formatting {
        // Force enhanced formatting regardless of content
        println!("Applying enhanced formatting to captions...");
        let result = format_transcript(&raw_text, false, 4);
        println!("Formatting completed");
        result
    } else if raw_text.contains('♪') || raw_text.contains('[') {
        // Preserve original formatting for music videos and special content
        println!("Preserving original formatting for music/special content");
        raw_text.clone()
    } else {
        // Apply enhanced formatting for regular speech
        println!("Applying enhanced formatting to captions...");
        let result = format_transcript(&raw_text, false, 4);
        println!("Formatting completed");
        result
    };

    Ok((formatted_text, raw_text))
}

/// Convert SRT subtitle format to plain text
fn srt_to_plain_text(srt_content: &str) -> String {
    let mut plain_text = String::new();
    let mut in_text_block = false;

    for line in srt_content.lines() {
        if line.trim().is_empty() {
            in_text_block = false;
            continue;
        }

        // Skip subtitle numbers and timestamps
        if line
            .trim()
            .chars()
            .next()
            .map(|c| c.is_numeric())
            .unwrap_or(false)
        {
            continue;
        }

        // Skip timestamp lines (contain -->)
        if line.contains("-->") {
            continue;
        }

        // This should be subtitle text
        if !in_text_block {
            if !plain_text.is_empty() {
                plain_text.push(' ');
            }
            in_text_block = true;
        }

        plain_text.push_str(line.trim());
        plain_text.push(' ');
    }

    plain_text.trim().to_string()
}

/// Download audio from YouTube video
pub async fn download_audio(video_id: &str, output_dir: &str) -> Result<PathBuf, Y2mdError> {
    let url = format!("https://www.youtube.com/watch?v={}", video_id);

    // Create output directory if it doesn't exist
    let output_path = PathBuf::from(output_dir);
    if !output_path.exists() {
        std::fs::create_dir_all(&output_path)?;
    }

    // First, check if audio file already exists in cache
    let _pattern = format!("{}_audio.*", video_id);
    let mut cached_audio_path = None;

    for entry in std::fs::read_dir(&output_path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        if let Some(name) = file_name.to_str() {
            if name.starts_with(&format!("{}_audio.", video_id)) {
                let path = entry.path();
                // Check if file is not empty
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if metadata.len() > 0 {
                        cached_audio_path = Some(path);
                        println!("Using cached audio file: {:?}", cached_audio_path);
                        break;
                    }
                }
            }
        }
    }

    if let Some(cached_path) = cached_audio_path {
        return Ok(cached_path);
    }

    // Create progress bar for download
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"]),
    );
    progress_bar.set_message("Downloading audio from YouTube...");
    progress_bar.enable_steady_tick(std::time::Duration::from_millis(100));

    // Use yt-dlp to download audio as WAV
    let output_template = output_path.join(format!("{}_audio", video_id));

    let status = Command::new("yt-dlp")
        .args([
            "-x", // Extract audio
            "--audio-format",
            "best", // Use best available format
            "--audio-quality",
            "0", // Best quality
            "-o",
            output_template.to_str().unwrap(),
            &url,
        ])
        .status()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Y2mdError::Config(
                    "yt-dlp not found. Please install yt-dlp: https://github.com/yt-dlp/yt-dlp"
                        .to_string(),
                )
            } else {
                Y2mdError::Io(e)
            }
        })?;

    if !status.success() {
        return Err(Y2mdError::Config(
            "Failed to download audio with yt-dlp".to_string(),
        ));
    }

    // Find the downloaded file (yt-dlp adds extension)
    // Look for files matching the pattern: {video_id}_audio.*
    let pattern = format!("{}_audio.*", video_id);
    let mut audio_path = None;

    println!("Looking for audio files matching pattern: {}", pattern);
    for entry in std::fs::read_dir(&output_path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        if let Some(name) = file_name.to_str() {
            println!("Found file: {}", name);
            if name.starts_with(&format!("{}_audio.", video_id)) {
                let path = entry.path();
                // Skip empty files
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if metadata.len() > 0 {
                        audio_path = Some(path);
                        println!("Selected audio file: {:?}", audio_path);
                        break;
                    } else {
                        println!("Skipping empty file: {:?}", path);
                    }
                }
            }
        }
    }

    let audio_path = audio_path.ok_or_else(|| {
        Y2mdError::Config(format!(
            "Downloaded audio file not found for pattern: {}",
            pattern
        ))
    })?;

    progress_bar.finish_with_message("Audio download completed");

    println!("Audio downloaded to: {:?}", audio_path);

    Ok(audio_path)
}

/// Transcribe YouTube video using captions or STT
pub async fn transcribe_video(
    video_id: &str,
    prefer_captions: bool,
    language: Option<&str>,
    output_dir: &str,
    paragraph_length: usize,
    force_formatting: bool,
) -> Result<(String, String, String), Y2mdError> {
    let mut source = "whisper".to_string();
    let transcript;

    let raw_transcript;

    if prefer_captions {
        match check_captions_available(video_id).await {
            Ok(true) => {
                let (formatted, raw) =
                    extract_captions(video_id, language, force_formatting).await?;
                transcript = formatted;
                raw_transcript = raw;
                source = "captions".to_string();
                println!("Using captions for transcription");
            }
            Ok(false) => {
                println!("No captions available, falling back to STT");
                let audio_path = download_audio(video_id, output_dir).await?;
                let (formatted, raw) =
                    transcribe_audio(&audio_path, language, paragraph_length).await?;
                transcript = formatted;
                raw_transcript = raw;
            }
            Err(e) => {
                println!("Error checking captions: {}, falling back to STT", e);
                let audio_path = download_audio(video_id, output_dir).await?;
                let (formatted, raw) =
                    transcribe_audio(&audio_path, language, paragraph_length).await?;
                transcript = formatted;
                raw_transcript = raw;
            }
        }
    } else {
        println!("Using STT for transcription");
        let audio_path = download_audio(video_id, output_dir).await?;
        let (formatted, raw) = transcribe_audio(&audio_path, language, paragraph_length).await?;
        transcript = formatted;
        raw_transcript = raw;
    }

    Ok((transcript, source, raw_transcript))
}

/// Transcribe audio file using STT
pub async fn transcribe_audio(
    audio_path: &PathBuf,
    language: Option<&str>,
    paragraph_length: usize,
) -> Result<(String, String), Y2mdError> {
    // Check if audio file exists
    if !audio_path.exists() {
        return Err(Y2mdError::Config(format!(
            "Audio file not found: {:?}",
            audio_path
        )));
    }

    // Use whisper-rs for real transcription
    println!("Transcribing audio with Whisper...");

    // Create progress bar for transcription
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    progress_bar.set_message("Transcribing audio...");
    progress_bar.enable_steady_tick(std::time::Duration::from_millis(100));

    // Determine which model to use based on language
    let (model_path, whisper_lang) = determine_model_and_language(language)?;

    if !std::path::Path::new(&model_path).exists() {
        return Err(Y2mdError::Whisper(format!(
            "Whisper model not found at: {}. Please run download_model.sh",
            model_path
        )));
    }

    // Load the whisper model
    let ctx_params = whisper_rs::WhisperContextParameters::default();
    let ctx = whisper_rs::WhisperContext::new_with_params(&model_path, ctx_params)
        .map_err(|e| Y2mdError::Whisper(format!("Failed to load whisper model: {}", e)))?;

    // Create state for transcription
    let mut state = ctx
        .create_state()
        .map_err(|e| Y2mdError::Whisper(format!("Failed to create state: {}", e)))?;

    // Convert audio to the format whisper expects
    let audio_data = convert_audio_for_whisper(audio_path).await?;

    // Set up transcription parameters
    let mut params =
        whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(&whisper_lang));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    // Transcribe the audio
    state
        .full(params, &audio_data[..])
        .map_err(|e| Y2mdError::Whisper(format!("Transcription failed: {}", e)))?;

    // Update progress bar
    progress_bar.set_message("Processing transcription segments...");

    // Collect all segments into a transcript
    let mut raw_transcript = String::new();
    for segment in state.as_iter() {
        let segment_text = segment.to_string();
        if !raw_transcript.is_empty() {
            raw_transcript.push(' ');
        }
        raw_transcript.push_str(&segment_text);
    }

    // Finish progress bar
    progress_bar.finish_with_message("Transcription completed");

    if raw_transcript.trim().is_empty() {
        return Err(Y2mdError::Whisper(
            "Transcription produced empty result".to_string(),
        ));
    }

    println!(
        "Transcription completed successfully (language: {})",
        whisper_lang
    );

    // Apply formatting to STT output
    println!("Applying formatting to transcript...");
    let formatted_transcript = format_transcript(&raw_transcript, false, paragraph_length);
    println!("Formatting completed");
    Ok((formatted_transcript, raw_transcript))
}

/// Determine which whisper model and language to use
fn determine_model_and_language(language: Option<&str>) -> Result<(String, String), Y2mdError> {
    let base_model_dir = shellexpand::tilde("~/.local/share/y2md/models/");
    let base_model_dir = base_model_dir.to_string();

    // Default to English if no language specified
    let lang = language.unwrap_or("en");

    // Map language codes to whisper model names
    let (model_name, whisper_lang) = match lang {
        "en" => ("ggml-base.en.bin", "en"),
        "es" => ("ggml-base.bin", "es"),
        "fr" => ("ggml-base.bin", "fr"),
        "de" => ("ggml-base.bin", "de"),
        "it" => ("ggml-base.bin", "it"),
        "pt" => ("ggml-base.bin", "pt"),
        "ru" => ("ggml-base.bin", "ru"),
        "ja" => ("ggml-base.bin", "ja"),
        "zh" => ("ggml-base.bin", "zh"),
        "ko" => ("ggml-base.bin", "ko"),
        "ar" => ("ggml-base.bin", "ar"),
        "hi" => ("ggml-base.bin", "hi"),
        _ => {
            // For unsupported languages, fall back to English model
            println!(
                "Warning: Language '{}' not explicitly supported, falling back to English model",
                lang
            );
            ("ggml-base.en.bin", "en")
        }
    };

    let model_path = format!("{}{}", base_model_dir, model_name);
    Ok((model_path, whisper_lang.to_string()))
}

/// Format transcript as Markdown with metadata
pub async fn format_markdown(
    metadata: &VideoMetadata,
    transcript: &str,
    source: &str,
    include_timestamps: bool,
    compact: bool,
    paragraph_length: usize,
    use_llm: bool,
) -> String {
    let mut markdown = String::new();

    // Add YAML front matter
    markdown.push_str("---\n");
    markdown.push_str(&format!(
        "title: \"{}\"\n",
        escape_markdown(&metadata.title)
    ));
    if let Some(channel) = &metadata.channel {
        markdown.push_str(&format!("channel: \"{}\"\n", escape_markdown(channel)));
    }
    markdown.push_str(&format!("url: \"{}\"\n", metadata.url));
    markdown.push_str(&format!("video_id: \"{}\"\n", metadata.video_id));
    if let Some(duration) = &metadata.duration {
        markdown.push_str(&format!("duration: \"{}\"\n", duration));
    }
    markdown.push_str(&format!("source: \"{}\"\n", source));
    markdown.push_str("language: \"en\"\n"); // TODO: Detect actual language from transcription
    markdown.push_str(&format!(
        "extracted_at: \"{}\"\n",
        chrono::Utc::now().to_rfc3339()
    ));
    markdown.push_str("---\n\n");

    // Add title
    markdown.push_str(&format!("# {}\n\n", escape_markdown(&metadata.title)));

    // Add transcript
    if include_timestamps {
        // For now, add placeholder timestamps
        markdown.push_str("[00:00:00] ");
    }

    // Use enhanced formatting for better readability
    let formatted_transcript = if use_llm {
        println!("Using LLM for enhanced formatting...");
        match format_with_llm(transcript).await {
            Ok(llm_formatted) => {
                println!("LLM formatting completed successfully");
                llm_formatted
            }
            Err(e) => {
                println!(
                    "LLM formatting failed: {}, falling back to standard formatting",
                    e
                );
                println!("Tip: Check your LLM configuration with 'y2md config show'");
                format_transcript(transcript, compact, paragraph_length)
            }
        }
    } else {
        format_transcript(transcript, compact, paragraph_length)
    };
    markdown.push_str(&formatted_transcript);

    markdown
}

/// Convert audio file to format expected by whisper
async fn convert_audio_for_whisper(audio_path: &PathBuf) -> Result<Vec<f32>, Y2mdError> {
    // First, try to convert the audio to WAV format using FFmpeg for better compatibility
    let converted_path = convert_audio_to_wav(audio_path).await?;

    // Then process the converted WAV file with symphonia
    use symphonia::core::audio::{AudioBufferRef, Signal};
    use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    // Open the converted audio file
    let file = std::fs::File::open(&converted_path)
        .map_err(|e| Y2mdError::Config(format!("Failed to open converted audio file: {}", e)))?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // Create a hint to help the format registry guess the format
    let mut hint = Hint::new();
    hint.with_extension("wav");

    // Use the default options for metadata and format
    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    // Probe the media source
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(|e| Y2mdError::Config(format!("Failed to probe audio format: {}", e)))?;

    // Get the format reader
    let mut format = probed.format;

    // Find the first audio track with a known codec
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| Y2mdError::Config("No supported audio tracks found".to_string()))?;

    // Create a decoder for the track
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| Y2mdError::Config(format!("Failed to create decoder: {}", e)))?;

    // Store all audio samples
    let mut all_samples = Vec::new();

    // Decode the audio packets
    while let Ok(packet) = format.next_packet() {
        match decoder.decode(&packet) {
            Ok(decoded) => {
                match decoded {
                    AudioBufferRef::F32(buf) => {
                        // For stereo, average the channels
                        if buf.spec().channels.count() == 2 {
                            for i in 0..buf.frames() {
                                let sample = (buf.chan(0)[i] + buf.chan(1)[i]) / 2.0;
                                all_samples.push(sample);
                            }
                        } else {
                            // For mono, just copy the samples
                            for i in 0..buf.frames() {
                                all_samples.push(buf.chan(0)[i]);
                            }
                        }
                    }
                    AudioBufferRef::S16(buf) => {
                        // Convert i16 to f32
                        if buf.spec().channels.count() == 2 {
                            for i in 0..buf.frames() {
                                let sample =
                                    (buf.chan(0)[i] as f32 + buf.chan(1)[i] as f32) / 2.0 / 32768.0;
                                all_samples.push(sample);
                            }
                        } else {
                            for i in 0..buf.frames() {
                                all_samples.push(buf.chan(0)[i] as f32 / 32768.0);
                            }
                        }
                    }
                    _ => {
                        return Err(Y2mdError::Config(
                            "Unsupported audio format (only F32 and S16 are supported)".to_string(),
                        ));
                    }
                }
            }
            Err(_) => {
                // Skip decoding errors
                continue;
            }
        }
    }

    // Clean up the temporary converted file
    let _ = std::fs::remove_file(&converted_path);

    if all_samples.is_empty() {
        return Err(Y2mdError::Config(
            "No audio samples were decoded".to_string(),
        ));
    }

    Ok(all_samples)
}

/// Convert audio file to WAV format using FFmpeg for better compatibility
async fn convert_audio_to_wav(audio_path: &PathBuf) -> Result<PathBuf, Y2mdError> {
    let temp_dir = std::env::temp_dir();
    let temp_filename = format!("y2md_converted_{}.wav", uuid::Uuid::new_v4());
    let output_path = temp_dir.join(temp_filename);

    // Create progress bar for conversion
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.yellow} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    progress_bar.set_message("Converting audio format...");
    progress_bar.enable_steady_tick(std::time::Duration::from_millis(100));

    println!(
        "Converting audio to WAV format: {:?} -> {:?}",
        audio_path, output_path
    );

    // Use FFmpeg to convert to WAV format
    let status = std::process::Command::new("ffmpeg")
        .args([
            "-i",
            audio_path.to_str().unwrap(),
            "-ac",
            "1", // Convert to mono
            "-ar",
            "16000", // 16kHz sample rate (optimal for whisper)
            "-acodec",
            "pcm_f32le", // 32-bit float PCM
            "-y",        // Overwrite output file
            output_path.to_str().unwrap(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| Y2mdError::Config(format!("Failed to execute ffmpeg: {}", e)))?;

    if !status.success() {
        return Err(Y2mdError::Config("FFmpeg conversion failed".to_string()));
    }

    // Verify the converted file exists and has content
    if !output_path.exists() {
        return Err(Y2mdError::Config(
            "Converted audio file was not created".to_string(),
        ));
    }

    let metadata = std::fs::metadata(&output_path)
        .map_err(|e| Y2mdError::Config(format!("Failed to get file metadata: {}", e)))?;

    if metadata.len() == 0 {
        return Err(Y2mdError::Config(
            "Converted audio file is empty".to_string(),
        ));
    }

    progress_bar.finish_with_message("Audio conversion completed");
    println!("Audio conversion successful");
    Ok(output_path)
}

/// Format transcript for better readability
pub fn format_transcript(transcript: &str, compact: bool, paragraph_length: usize) -> String {
    if compact {
        // Simple paragraph format for compact mode
        return format_paragraphs(transcript, paragraph_length); // More sentences per paragraph
    }

    // Enhanced formatting for better readability
    let cleaned = clean_transcript(transcript);
    // Use configured paragraph length (default 3-5 sentences per paragraph)
    format_paragraphs(&cleaned, paragraph_length)
}

/// Apply LLM formatting to transcript using configured LLM
pub async fn format_with_llm(transcript: &str) -> Result<String, Y2mdError> {
    let config = AppConfig::load()?;

    // Validate LLM configuration
    validate_llm_config(&config.llm)?;

    match config.llm.provider {
        LlmProvider::Ollama => format_with_ollama(transcript, &config.llm).await,
        LlmProvider::OpenAI => format_with_openai(transcript, &config.llm).await,
        LlmProvider::Anthropic => format_with_anthropic(transcript, &config.llm).await,
        LlmProvider::LMStudio => format_with_lmstudio(transcript, &config.llm).await,
        LlmProvider::Custom => format_with_custom(transcript, &config.llm).await,
    }
}

/// Validate LLM configuration
fn validate_llm_config(llm_config: &LlmConfig) -> Result<(), Y2mdError> {
    if llm_config.model.trim().is_empty() {
        return Err(Y2mdError::LlmConfig(
            "LLM model name cannot be empty".to_string(),
        ));
    }

    match llm_config.provider {
        LlmProvider::OpenAI | LlmProvider::Anthropic => {
            if llm_config.api_key.is_none() {
                return Err(Y2mdError::LlmConfig(format!(
                    "{} provider requires an API key",
                    llm_config.provider
                )));
            }
        }
        LlmProvider::Custom => {
            if llm_config.endpoint.is_none() {
                return Err(Y2mdError::LlmConfig(
                    "Custom provider requires an endpoint URL".to_string(),
                ));
            }
        }
        LlmProvider::Ollama | LlmProvider::LMStudio => {}
    }

    Ok(())
}

/// Apply LLM formatting using Ollama
async fn format_with_ollama(transcript: &str, llm_config: &LlmConfig) -> Result<String, Y2mdError> {
    let endpoint = llm_config
        .endpoint
        .as_deref()
        .unwrap_or("http://localhost:11434");

    // Check if Ollama service is available
    let client = reqwest::Client::new();
    let health_check = client.get(format!("{}/api/tags", endpoint)).send().await;

    if health_check.is_err() {
        return Err(Y2mdError::LlmConfig(format!(
            "Ollama service not available at {}. Make sure Ollama is running",
            endpoint
        )));
    }

    // Check if model is available
    let model_check = client.get(format!("{}/api/tags", endpoint)).send().await;
    if let Ok(response) = model_check {
        if response.status().is_success() {
            let models_json: serde_json::Value = response.json().await.map_err(|e| {
                Y2mdError::LlmConfig(format!("Failed to parse Ollama models: {}", e))
            })?;

            let models = models_json["models"].as_array().ok_or_else(|| {
                Y2mdError::LlmConfig(
                    "Invalid response format from Ollama models endpoint".to_string(),
                )
            })?;

            let model_exists = models.iter().any(|model| {
                model["name"]
                    .as_str()
                    .map(|name| name.contains(&llm_config.model))
                    .unwrap_or(false)
            });

            if !model_exists {
                return Err(Y2mdError::LlmConfig(format!(
                    "Model '{}' not found in Ollama. Available models: {}",
                    llm_config.model,
                    models
                        .iter()
                        .filter_map(|m| m["name"].as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
            }
        }
    }

    // Prepare the prompt for the LLM
    let prompt = format!(
        "Please format the following transcript into well-structured markdown. 
        Keep the original content but improve readability by:
        - Organizing into logical paragraphs
        - Fixing any grammar or punctuation issues
        - Removing filler words if appropriate
        - Maintaining the original meaning and tone
        
        Transcript:\n\n{}
        
        Formatted markdown:",
        transcript
    );

    // Prepare the request payload
    let request_body = serde_json::json!({
        "model": llm_config.model,
        "prompt": prompt,
        "stream": false
    });

    // Send request to Ollama with timeout
    let response = client
        .post(format!("{}/api/generate", endpoint))
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(120)) // 2 minute timeout
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                Y2mdError::LlmConfig("LLM request timed out after 2 minutes".to_string())
            } else {
                Y2mdError::LlmConfig(format!("Failed to connect to Ollama: {}", e))
            }
        })?;

    if !response.status().is_success() {
        return Err(Y2mdError::LlmConfig(format!(
            "Ollama API returned error: {}",
            response.status()
        )));
    }

    // Parse the response
    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| Y2mdError::LlmConfig(format!("Failed to parse Ollama response: {}", e)))?;

    // Extract the generated text
    let formatted_text = response_json["response"]
        .as_str()
        .ok_or_else(|| Y2mdError::LlmConfig("Invalid response format from Ollama".to_string()))?
        .trim()
        .to_string();

    if formatted_text.is_empty() {
        return Err(Y2mdError::LlmConfig(
            "Ollama returned empty response".to_string(),
        ));
    }

    Ok(formatted_text)
}

/// Apply LLM formatting using OpenAI-compatible API
async fn format_with_openai(transcript: &str, llm_config: &LlmConfig) -> Result<String, Y2mdError> {
    let endpoint = llm_config
        .endpoint
        .as_deref()
        .unwrap_or("https://api.openai.com/v1");

    let client = reqwest::Client::new();

    // Prepare the prompt for the LLM
    let prompt = format!(
        "Please format the following transcript into well-structured markdown. 
        Keep the original content but improve readability by:
        - Organizing into logical paragraphs
        - Fixing any grammar or punctuation issues
        - Removing filler words if appropriate
        - Maintaining the original meaning and tone
        
        Transcript:\n\n{}",
        transcript
    );

    // Prepare the request payload
    let request_body = serde_json::json!({
        "model": llm_config.model,
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant that formats transcripts into well-structured markdown."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": 0.1
    });

    // Send request to OpenAI-compatible API with timeout
    let mut request_builder = client
        .post(format!("{}/chat/completions", endpoint))
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(120)); // 2 minute timeout

    if let Some(api_key) = &llm_config.api_key {
        request_builder = request_builder.header("Authorization", format!("Bearer {}", api_key));
    }

    let response = request_builder.send().await.map_err(|e| {
        if e.is_timeout() {
            Y2mdError::LlmConfig("LLM request timed out after 2 minutes".to_string())
        } else {
            Y2mdError::LlmConfig(format!("Failed to connect to OpenAI API: {}", e))
        }
    })?;

    if !response.status().is_success() {
        return Err(Y2mdError::LlmConfig(format!(
            "OpenAI API returned error: {}",
            response.status()
        )));
    }

    // Parse the response
    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| Y2mdError::LlmConfig(format!("Failed to parse OpenAI response: {}", e)))?;

    // Extract the generated text
    let formatted_text = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| Y2mdError::LlmConfig("Invalid response format from OpenAI".to_string()))?
        .trim()
        .to_string();

    if formatted_text.is_empty() {
        return Err(Y2mdError::LlmConfig(
            "OpenAI returned empty response".to_string(),
        ));
    }

    Ok(formatted_text)
}

/// Apply LLM formatting using LM Studio
async fn format_with_lmstudio(
    transcript: &str,
    llm_config: &LlmConfig,
) -> Result<String, Y2mdError> {
    format_with_openai(transcript, llm_config).await
}

async fn format_with_anthropic(
    transcript: &str,
    llm_config: &LlmConfig,
) -> Result<String, Y2mdError> {
    let endpoint = llm_config
        .endpoint
        .as_deref()
        .unwrap_or("https://api.anthropic.com/v1");

    let client = reqwest::Client::new();

    let prompt = format!(
        "Please format the following transcript into well-structured markdown. 
        Keep the original content but improve readability by:
        - Organizing into logical paragraphs
        - Fixing any grammar or punctuation issues
        - Removing filler words if appropriate
        - Maintaining the original meaning and tone
        
        Transcript:\n\n{}",
        transcript
    );

    let request_body = serde_json::json!({
        "model": llm_config.model,
        "max_tokens": 4096,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let mut request_builder = client
        .post(format!("{}/messages", endpoint))
        .header("anthropic-version", "2023-06-01")
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(120));

    if let Some(api_key) = &llm_config.api_key {
        request_builder = request_builder.header("x-api-key", api_key);
    }

    let response = request_builder.send().await.map_err(|e| {
        if e.is_timeout() {
            Y2mdError::LlmConfig("LLM request timed out after 2 minutes".to_string())
        } else {
            Y2mdError::LlmConfig(format!("Failed to connect to Anthropic API: {}", e))
        }
    })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(Y2mdError::LlmConfig(format!(
            "Anthropic API returned error {}: {}",
            status, error_text
        )));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| Y2mdError::LlmConfig(format!("Failed to parse Anthropic response: {}", e)))?;

    let formatted_text = response_json["content"][0]["text"]
        .as_str()
        .ok_or_else(|| Y2mdError::LlmConfig("Invalid response format from Anthropic".to_string()))?
        .trim()
        .to_string();

    if formatted_text.is_empty() {
        return Err(Y2mdError::LlmConfig(
            "Anthropic returned empty response".to_string(),
        ));
    }

    Ok(formatted_text)
}

async fn format_with_custom(transcript: &str, llm_config: &LlmConfig) -> Result<String, Y2mdError> {
    let _endpoint = llm_config
        .endpoint
        .as_ref()
        .ok_or_else(|| Y2mdError::LlmConfig("Custom provider requires endpoint".to_string()))?;

    format_with_openai(transcript, llm_config).await
}

/// Clean and normalize transcript text
fn clean_transcript(text: &str) -> String {
    let mut result = String::new();
    let words: Vec<&str> = text.split_whitespace().collect();

    for (i, word) in words.iter().enumerate() {
        if !result.is_empty() {
            result.push(' ');
        }

        // Capitalize first word of sentence
        if i == 0 || result.ends_with(['.', '!', '?']) {
            result.push_str(&capitalize_first_letter(word));
        } else {
            result.push_str(word);
        }

        // Add punctuation if missing at natural breaks
        if should_add_punctuation(word, i, words.len()) {
            result.push('.');
        }
    }

    result
}

/// Format text into readable paragraphs
fn format_paragraphs(text: &str, sentences_per_paragraph: usize) -> String {
    let mut result = String::new();
    let sentences: Vec<&str> = text
        .split(['.', '!', '?'])
        .filter(|s| !s.trim().is_empty())
        .collect();

    let mut sentence_count = 0;
    let mut current_paragraph = String::new();

    for sentence in sentences {
        let trimmed = sentence.trim();
        if trimmed.is_empty() {
            continue;
        }

        if !current_paragraph.is_empty() {
            current_paragraph.push(' ');
        }
        current_paragraph.push_str(&capitalize_first_letter(trimmed));
        current_paragraph.push('.');

        sentence_count += 1;

        // Start new paragraph after N sentences
        if sentence_count >= sentences_per_paragraph {
            if !result.is_empty() {
                result.push_str("\n\n");
            }
            result.push_str(&current_paragraph);
            current_paragraph.clear();
            sentence_count = 0;
        }
    }

    // Add remaining sentences
    if !current_paragraph.is_empty() {
        if !result.is_empty() {
            result.push_str("\n\n");
        }
        result.push_str(&current_paragraph);
    }

    result
}

/// Capitalize first letter of a string
fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Determine if punctuation should be added
fn should_add_punctuation(word: &str, index: usize, total_words: usize) -> bool {
    // Don't add punctuation if it already ends with one
    if word.ends_with(['.', '!', '?']) {
        return false;
    }

    // Add punctuation at natural sentence boundaries
    let is_long_phrase = index > 0 && index.is_multiple_of(12); // Every ~12 words
    let is_near_end = index == total_words - 1;

    is_long_phrase || is_near_end
}

/// Escape Markdown special characters
fn escape_markdown(text: &str) -> String {
    text.replace('*', "\\*")
        .replace('_', "\\_")
        .replace('`', "\\`")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('#', "\\#")
        .replace('+', "\\+")
        .replace('-', "\\-")
        .replace('.', "\\.")
        .replace('!', "\\!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_video_id_youtube_com() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        assert_eq!(extract_video_id(url).unwrap(), "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_youtu_be() {
        let url = "https://youtu.be/dQw4w9WgXcQ";
        assert_eq!(extract_video_id(url).unwrap(), "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_shorts() {
        let url = "https://www.youtube.com/shorts/abc123def45";
        assert_eq!(extract_video_id(url).unwrap(), "abc123def45");
    }

    #[test]
    fn test_extract_video_id_with_params() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=42";
        assert_eq!(extract_video_id(url).unwrap(), "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_direct() {
        let url = "dQw4w9WgXcQ";
        assert_eq!(extract_video_id(url).unwrap(), "dQw4w9WgXcQ");
    }

    #[test]
    fn test_validate_youtube_url() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        assert_eq!(validate_youtube_url(url).unwrap(), "dQw4w9WgXcQ");
    }

    #[test]
    fn test_invalid_url() {
        let url = "https://example.com";
        assert!(extract_video_id(url).is_err());
    }

    #[test]
    fn test_capitalize_first_letter() {
        assert_eq!(capitalize_first_letter("hello"), "Hello");
        assert_eq!(capitalize_first_letter("world"), "World");
        assert_eq!(capitalize_first_letter(""), "");
    }

    #[test]
    fn test_format_transcript_compact() {
        let transcript = "this is a test sentence. this is another sentence.";
        let formatted = format_transcript(transcript, true, 8);
        assert!(formatted.contains("This is a test sentence."));
        assert!(formatted.contains("This is another sentence."));
    }

    #[test]
    fn test_format_transcript_enhanced() {
        let transcript = "this is a test sentence. this is another sentence.";
        let formatted = format_transcript(transcript, false, 4);
        assert!(formatted.contains("This is a test sentence."));
        assert!(formatted.contains("This is another sentence."));
    }

    #[test]
    fn test_clean_transcript() {
        let transcript = "hello world how are you";
        let cleaned = clean_transcript(transcript);
        assert_eq!(cleaned, "Hello world how are you.");
    }

    #[test]
    fn test_format_paragraphs() {
        let text = "first. second. third. fourth. fifth.";
        let formatted = format_paragraphs(text, 2);
        // Should create paragraphs with 2 sentences each
        assert!(formatted.contains("First. Second."));
        assert!(formatted.contains("Third. Fourth."));
        assert!(formatted.contains("Fifth."));
    }

    #[test]
    fn test_formatting_pipeline() {
        // Test the complete formatting pipeline
        let raw_transcript = "hello world this is a test sentence how are you doing today i hope you are doing well this is another test sentence to demonstrate the formatting capabilities of our system";

        // Test compact mode
        let compact = format_transcript(raw_transcript, true, 8);
        assert!(compact.contains("Hello world this is a test sentence"));
        assert!(compact.contains("how are you doing today"));

        // Test enhanced mode
        let enhanced = format_transcript(raw_transcript, false, 4);
        assert!(enhanced.contains("Hello world this is a test sentence"));
        assert!(enhanced.contains("how are you doing today"));

        // Verify they produce different outputs
        assert_ne!(compact, enhanced);
    }

    #[test]
    fn test_paragraph_length_customization() {
        let transcript = "first sentence. second sentence. third sentence. fourth sentence. fifth sentence. sixth sentence. seventh sentence. eighth sentence. ninth sentence. tenth sentence. eleventh sentence. twelfth sentence.";

        // Test different paragraph lengths in compact mode
        let compact_short = format_transcript(transcript, true, 2);
        let compact_long = format_transcript(transcript, true, 5);

        println!("Compact short (2): '{}'", compact_short);
        println!("Compact long (5): '{}'", compact_long);
        println!(
            "Compact short paragraphs: {}",
            compact_short.matches("\n\n").count() + 1
        );
        println!(
            "Compact long paragraphs: {}",
            compact_long.matches("\n\n").count() + 1
        );

        // They should be different due to different paragraph lengths
        assert_ne!(compact_short, compact_long);

        // Test different paragraph lengths in enhanced mode
        let enhanced_short = format_transcript(transcript, false, 2);
        let enhanced_long = format_transcript(transcript, false, 5);

        println!("Enhanced short (2): '{}'", enhanced_short);
        println!("Enhanced long (5): '{}'", enhanced_long);
        println!(
            "Enhanced short paragraphs: {}",
            enhanced_short.matches("\n\n").count() + 1
        );
        println!(
            "Enhanced long paragraphs: {}",
            enhanced_long.matches("\n\n").count() + 1
        );

        // They should be different due to different paragraph lengths
        assert_ne!(enhanced_short, enhanced_long);
    }
}

// ============================================================================
// Ollama Model Management
// ============================================================================

use std::sync::Arc;
use tokio::sync::Mutex;

/// Ollama model management
#[derive(Debug, Clone)]
pub struct OllamaManager {
    client: reqwest::Client,
    endpoint: String,
    cache: Arc<Mutex<ModelCache>>,
}

#[derive(Debug, Clone, Default)]
struct ModelCache {
    local_models: Vec<String>,
    last_updated: Option<std::time::SystemTime>,
}

impl OllamaManager {
    /// Create a new Ollama manager
    pub fn new(endpoint: Option<String>) -> Self {
        let endpoint = endpoint.unwrap_or_else(|| "http://localhost:11434".to_string());
        Self {
            client: reqwest::Client::new(),
            endpoint,
            cache: Arc::new(Mutex::new(ModelCache::default())),
        }
    }

    /// Check if Ollama service is available
    pub async fn is_available(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await
            .is_ok()
    }

    /// Get list of locally available models
    pub async fn get_local_models(&self) -> Result<Vec<String>, Y2mdError> {
        let mut cache = self.cache.lock().await;

        // Use cache if recently updated (within 30 seconds)
        if let Some(last_updated) = cache.last_updated {
            if last_updated.elapsed().unwrap_or_default().as_secs() < 30 {
                return Ok(cache.local_models.clone());
            }
        }

        let response = self
            .client
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await
            .map_err(|e| Y2mdError::LlmConfig(format!("Failed to connect to Ollama: {}", e)))?;

        if !response.status().is_success() {
            return Err(Y2mdError::LlmConfig(
                "Ollama service not available".to_string(),
            ));
        }

        let models_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Y2mdError::LlmConfig(format!("Failed to parse Ollama models: {}", e)))?;

        let models = models_json["models"].as_array().ok_or_else(|| {
            Y2mdError::LlmConfig("Invalid response format from Ollama".to_string())
        })?;

        let model_names: Vec<String> = models
            .iter()
            .filter_map(|model| model["name"].as_str().map(|s| s.to_string()))
            .collect();

        // Update cache
        cache.local_models = model_names.clone();
        cache.last_updated = Some(std::time::SystemTime::now());

        Ok(model_names)
    }

    /// Check if a specific model is available locally
    pub async fn is_model_available(&self, model_name: &str) -> Result<bool, Y2mdError> {
        let local_models = self.get_local_models().await?;
        Ok(local_models.iter().any(|name| name.contains(model_name)))
    }

    /// Get model information including size
    pub async fn get_model_info(&self, model_name: &str) -> Result<ModelInfo, Y2mdError> {
        // First check if model exists locally
        let local_models = self.get_local_models().await?;
        if let Some(full_name) = local_models.iter().find(|name| name.contains(model_name)) {
            return Ok(ModelInfo {
                name: full_name.clone(),
                size: None, // Size not available from local models endpoint
                available: true,
            });
        }

        // For remote models, we'd need to query Ollama's model library
        // This is a simplified implementation
        Ok(ModelInfo {
            name: model_name.to_string(),
            size: None, // Would need to query Ollama's model library
            available: false,
        })
    }

    /// Download a model with progress indication
    pub async fn download_model(
        &self,
        model_name: &str,
        progress_callback: Option<Box<dyn Fn(String, u64, u64) + Send + Sync>>,
    ) -> Result<(), Y2mdError> {
        let response = self
            .client
            .post(format!("{}/api/pull", self.endpoint))
            .json(&serde_json::json!({
                "name": model_name,
                "stream": true
            }))
            .send()
            .await
            .map_err(|e| Y2mdError::LlmConfig(format!("Failed to start model download: {}", e)))?;

        if !response.status().is_success() {
            let status_code = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Y2mdError::LlmConfig(format!(
                "Failed to download model: {} - {}",
                status_code, error_text
            )));
        }

        if let Some(callback) = &progress_callback {
            callback("Starting download...".to_string(), 0, 0);
        }

        // Stream the response line by line
        let mut download_completed = false;

        // Read the response as text and process line by line
        let response_text = response.text().await.map_err(|e| {
            Y2mdError::LlmConfig(format!("Failed to read download response: {}", e))
        })?;

        for line in response_text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(status) = json["status"].as_str() {
                    if let Some(callback) = &progress_callback {
                        callback(status.to_string(), 0, 0);
                    }

                    // Check for completion indicators
                    if status == "success" || status.contains("complete") || status.contains("done")
                    {
                        download_completed = true;
                    }
                }
            }
        }

        // If we didn't get a clear completion signal, wait a bit and check
        if !download_completed {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }

        if let Some(callback) = &progress_callback {
            callback("Download complete".to_string(), 100, 100);
        }

        // Verify the model was actually downloaded
        let mut attempts = 0;
        while attempts < 5 {
            let available = self.is_model_available(model_name).await?;
            if available {
                break;
            }
            attempts += 1;
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }

        let final_available = self.is_model_available(model_name).await?;

        if !final_available {
            return Err(Y2mdError::LlmConfig(format!(
                "Model '{}' was not installed after download. Please check if the model name is correct and try again.",
                model_name
            )));
        }

        // Invalidate cache since we added a new model
        let mut cache = self.cache.lock().await;
        cache.last_updated = None;

        Ok(())
    }

    /// Remove a model
    pub async fn remove_model(&self, model_name: &str) -> Result<(), Y2mdError> {
        let response = self
            .client
            .delete(format!("{}/api/delete", self.endpoint))
            .json(&serde_json::json!({
                "name": model_name
            }))
            .send()
            .await
            .map_err(|e| Y2mdError::LlmConfig(format!("Failed to remove model: {}", e)))?;

        if !response.status().is_success() {
            return Err(Y2mdError::LlmConfig(format!(
                "Failed to remove model: {}",
                response.status()
            )));
        }

        // Invalidate cache
        let mut cache = self.cache.lock().await;
        cache.last_updated = None;

        Ok(())
    }
}

/// Model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub size: Option<u64>, // Size in bytes
    pub available: bool,
}

impl ModelInfo {
    /// Get human-readable size
    pub fn size_human(&self) -> Option<String> {
        self.size.map(|bytes| {
            const KB: u64 = 1024;
            const MB: u64 = KB * 1024;
            const GB: u64 = MB * 1024;

            if bytes >= GB {
                format!("{:.1} GB", bytes as f64 / GB as f64)
            } else if bytes >= MB {
                format!("{:.1} MB", bytes as f64 / MB as f64)
            } else if bytes >= KB {
                format!("{:.1} KB", bytes as f64 / KB as f64)
            } else {
                format!("{} bytes", bytes)
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub provider_type: LlmProvider,
    pub model: String,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
    pub token_type: String,
}

impl OAuthToken {
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now >= expires_at
        } else {
            false
        }
    }

    pub fn needs_refresh(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now >= expires_at - 300
        } else {
            false
        }
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

    pub fn get_api_key(&self, provider_name: &str) -> Result<Option<String>, Y2mdError> {
        let env_var_name = format!("Y2MD_{}_API_KEY", provider_name.to_uppercase());
        if let Ok(key) = std::env::var(&env_var_name) {
            return Ok(Some(key));
        }

        let entry = keyring::Entry::new(&self.service_name, provider_name)
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

    pub fn set_api_key(&self, provider_name: &str, api_key: &str) -> Result<(), Y2mdError> {
        let entry = keyring::Entry::new(&self.service_name, provider_name)
            .map_err(|e| Y2mdError::Config(format!("Failed to access keyring: {}", e)))?;

        entry
            .set_password(api_key)
            .map_err(|e| Y2mdError::Config(format!("Failed to store API key in keyring: {}", e)))?;

        Ok(())
    }

    pub fn delete_api_key(&self, provider_name: &str) -> Result<(), Y2mdError> {
        let entry = keyring::Entry::new(&self.service_name, provider_name)
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

    pub fn has_api_key(&self, provider_name: &str) -> bool {
        self.get_api_key(provider_name).ok().flatten().is_some()
    }

    pub fn get_oauth_token(&self, provider_name: &str) -> Result<Option<OAuthToken>, Y2mdError> {
        let token_key = format!("{}_oauth_token", provider_name);
        let entry = keyring::Entry::new(&self.service_name, &token_key)
            .map_err(|e| Y2mdError::Config(format!("Failed to access keyring: {}", e)))?;

        match entry.get_password() {
            Ok(token_json) => {
                let token: OAuthToken = serde_json::from_str(&token_json).map_err(|e| {
                    Y2mdError::Config(format!("Failed to parse OAuth token: {}", e))
                })?;
                Ok(Some(token))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(Y2mdError::Config(format!(
                "Failed to retrieve OAuth token from keyring: {}",
                e
            ))),
        }
    }

    pub fn set_oauth_token(
        &self,
        provider_name: &str,
        token: &OAuthToken,
    ) -> Result<(), Y2mdError> {
        let token_key = format!("{}_oauth_token", provider_name);
        let entry = keyring::Entry::new(&self.service_name, &token_key)
            .map_err(|e| Y2mdError::Config(format!("Failed to access keyring: {}", e)))?;

        let token_json = serde_json::to_string(token)
            .map_err(|e| Y2mdError::Config(format!("Failed to serialize OAuth token: {}", e)))?;

        entry.set_password(&token_json).map_err(|e| {
            Y2mdError::Config(format!("Failed to store OAuth token in keyring: {}", e))
        })?;

        Ok(())
    }

    pub fn delete_oauth_token(&self, provider_name: &str) -> Result<(), Y2mdError> {
        let token_key = format!("{}_oauth_token", provider_name);
        let entry = keyring::Entry::new(&self.service_name, &token_key)
            .map_err(|e| Y2mdError::Config(format!("Failed to access keyring: {}", e)))?;

        match entry.delete_password() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(Y2mdError::Config(format!(
                "Failed to delete OAuth token from keyring: {}",
                e
            ))),
        }
    }

    pub fn has_oauth_token(&self, provider_name: &str) -> bool {
        self.get_oauth_token(provider_name).ok().flatten().is_some()
    }

    pub async fn get_valid_token(
        &self,
        provider_name: &str,
        provider_type: &LlmProvider,
    ) -> Result<Option<String>, Y2mdError> {
        if let Some(mut token) = self.get_oauth_token(provider_name)? {
            if token.needs_refresh() && token.refresh_token.is_some() {
                token = self
                    .refresh_oauth_token(provider_name, provider_type, &token)
                    .await?;
                self.set_oauth_token(provider_name, &token)?;
            }

            if !token.is_expired() {
                return Ok(Some(token.access_token));
            }
        }

        self.get_api_key(provider_name)
    }

    async fn refresh_oauth_token(
        &self,
        _provider_name: &str,
        provider_type: &LlmProvider,
        token: &OAuthToken,
    ) -> Result<OAuthToken, Y2mdError> {
        let refresh_token = token
            .refresh_token
            .as_ref()
            .ok_or_else(|| Y2mdError::Config("No refresh token available".to_string()))?;

        match provider_type {
            LlmProvider::OpenAI => self.refresh_openai_token(refresh_token).await,
            LlmProvider::Anthropic => self.refresh_anthropic_token(refresh_token).await,
            _ => Err(Y2mdError::Config(format!(
                "OAuth not supported for provider type: {}",
                provider_type
            ))),
        }
    }

    async fn refresh_openai_token(&self, _refresh_token: &str) -> Result<OAuthToken, Y2mdError> {
        Err(Y2mdError::Config(
            "OpenAI OAuth refresh not yet implemented".to_string(),
        ))
    }

    async fn refresh_anthropic_token(&self, _refresh_token: &str) -> Result<OAuthToken, Y2mdError> {
        Err(Y2mdError::Config(
            "Anthropic OAuth refresh not yet implemented".to_string(),
        ))
    }
}

pub struct OAuthManager {
    client: reqwest::Client,
}

impl OAuthManager {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn device_code_flow(
        &self,
        provider_type: &LlmProvider,
        client_id: &str,
    ) -> Result<OAuthToken, Y2mdError> {
        match provider_type {
            LlmProvider::OpenAI => self.openai_device_code_flow(client_id).await,
            LlmProvider::Anthropic => self.anthropic_device_code_flow(client_id).await,
            _ => Err(Y2mdError::Config(format!(
                "OAuth not supported for provider type: {}",
                provider_type
            ))),
        }
    }

    async fn openai_device_code_flow(&self, client_id: &str) -> Result<OAuthToken, Y2mdError> {
        println!("🔐 Starting OpenAI OAuth authentication...\n");

        let device_code_response = self
            .client
            .post("https://auth0.openai.com/oauth/device/code")
            .json(&serde_json::json!({
                "client_id": client_id,
                "scope": "openid profile email offline_access",
                "audience": "https://api.openai.com/v1"
            }))
            .send()
            .await
            .map_err(|e| {
                Y2mdError::Config(format!("Failed to initiate device code flow: {}", e))
            })?;

        if !device_code_response.status().is_success() {
            let error_text = device_code_response.text().await.unwrap_or_default();
            return Err(Y2mdError::Config(format!(
                "Failed to get device code: {}",
                error_text
            )));
        }

        let device_code_json: serde_json::Value =
            device_code_response.json().await.map_err(|e| {
                Y2mdError::Config(format!("Failed to parse device code response: {}", e))
            })?;

        let user_code = device_code_json["user_code"]
            .as_str()
            .ok_or_else(|| Y2mdError::Config("Missing user_code in response".to_string()))?;
        let verification_uri = device_code_json["verification_uri"]
            .as_str()
            .ok_or_else(|| Y2mdError::Config("Missing verification_uri in response".to_string()))?;
        let device_code = device_code_json["device_code"]
            .as_str()
            .ok_or_else(|| Y2mdError::Config("Missing device_code in response".to_string()))?;
        let interval = device_code_json["interval"].as_u64().unwrap_or(5);

        println!("Please visit: {}", verification_uri);
        println!("And enter code: {}\n", user_code);
        println!("Waiting for authentication...");

        self.poll_for_token(
            "https://auth0.openai.com/oauth/token",
            client_id,
            device_code,
            interval,
        )
        .await
    }

    async fn anthropic_device_code_flow(&self, _client_id: &str) -> Result<OAuthToken, Y2mdError> {
        Err(Y2mdError::Config(
            "Anthropic OAuth device code flow not yet implemented. Please use API key authentication.".to_string()
        ))
    }

    async fn poll_for_token(
        &self,
        token_url: &str,
        client_id: &str,
        device_code: &str,
        interval: u64,
    ) -> Result<OAuthToken, Y2mdError> {
        let mut attempts = 0;
        let max_attempts = 120;

        loop {
            if attempts >= max_attempts {
                return Err(Y2mdError::Config("Authentication timeout".to_string()));
            }

            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;

            let response = self
                .client
                .post(token_url)
                .json(&serde_json::json!({
                    "client_id": client_id,
                    "device_code": device_code,
                    "grant_type": "urn:ietf:params:oauth:grant-type:device_code"
                }))
                .send()
                .await
                .map_err(|e| Y2mdError::Config(format!("Failed to poll for token: {}", e)))?;

            if response.status().is_success() {
                let token_json: serde_json::Value = response.json().await.map_err(|e| {
                    Y2mdError::Config(format!("Failed to parse token response: {}", e))
                })?;

                let access_token = token_json["access_token"]
                    .as_str()
                    .ok_or_else(|| {
                        Y2mdError::Config("Missing access_token in response".to_string())
                    })?
                    .to_string();

                let refresh_token = token_json["refresh_token"].as_str().map(|s| s.to_string());

                let expires_in = token_json["expires_in"].as_u64();
                let expires_at = expires_in.map(|secs| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        + secs
                });

                let token_type = token_json["token_type"]
                    .as_str()
                    .unwrap_or("Bearer")
                    .to_string();

                println!("✅ Authentication successful!\n");

                return Ok(OAuthToken {
                    access_token,
                    refresh_token,
                    expires_at,
                    token_type,
                });
            }

            let error_json: serde_json::Value = response
                .json()
                .await
                .map_err(|e| Y2mdError::Config(format!("Failed to parse error response: {}", e)))?;

            let error = error_json["error"].as_str().unwrap_or("unknown_error");

            match error {
                "authorization_pending" => {
                    attempts += 1;
                    continue;
                }
                "slow_down" => {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    attempts += 1;
                    continue;
                }
                "expired_token" => {
                    return Err(Y2mdError::Config(
                        "Device code expired. Please try again.".to_string(),
                    ));
                }
                "access_denied" => {
                    return Err(Y2mdError::Config("Access denied by user".to_string()));
                }
                _ => {
                    return Err(Y2mdError::Config(format!("OAuth error: {}", error)));
                }
            }
        }
    }
}
