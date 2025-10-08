use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

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
    let output = Command::new("/tmp/yt-dlp-venv/bin/yt-dlp")
        .args(["--dump-json", "--no-download", &url])
        .output()?;

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
    let output = Command::new("/tmp/yt-dlp-venv/bin/yt-dlp")
        .args(["--list-subs", "--no-download", &url])
        .output()?;

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
    let output = Command::new("/tmp/yt-dlp-venv/bin/yt-dlp")
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
        .output()?;

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

    let status = Command::new("/tmp/yt-dlp-venv/bin/yt-dlp")
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
        .status()?;

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
                let (formatted, raw) = extract_captions(video_id, language, force_formatting).await?;
                transcript = formatted;
                raw_transcript = raw;
                source = "captions".to_string();
                println!("Using captions for transcription");
            }
            Ok(false) => {
                println!("No captions available, falling back to STT");
                let audio_path = download_audio(video_id, output_dir).await?;
                let (formatted, raw) = transcribe_audio(&audio_path, language, paragraph_length).await?;
                transcript = formatted;
                raw_transcript = raw;
            }
            Err(e) => {
                println!("Error checking captions: {}, falling back to STT", e);
                let audio_path = download_audio(video_id, output_dir).await?;
                let (formatted, raw) = transcribe_audio(&audio_path, language, paragraph_length).await?;
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

/// Apply LLM formatting to transcript using Ollama
pub async fn format_with_llm(transcript: &str) -> Result<String, Y2mdError> {
    // Check if Ollama service is available
    let client = reqwest::Client::new();
    let health_check = client.get("http://localhost:11434/api/tags").send().await;

    if health_check.is_err() {
        return Err(Y2mdError::Config(
            "Ollama service not available. Make sure Ollama is running on localhost:11434"
                .to_string(),
        ));
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
        "model": "mistral-nemo:12b-instruct-2407-q5_0",
        "prompt": prompt,
        "stream": false
    });

    // Send request to Ollama
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| Y2mdError::Config(format!("Failed to connect to Ollama: {}", e)))?;

    if !response.status().is_success() {
        return Err(Y2mdError::Config(format!(
            "Ollama API returned error: {}",
            response.status()
        )));
    }

    // Parse the response
    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| Y2mdError::Config(format!("Failed to parse Ollama response: {}", e)))?;

    // Extract the generated text
    let formatted_text = response_json["response"]
        .as_str()
        .ok_or_else(|| Y2mdError::Config("Invalid response format from Ollama".to_string()))?
        .trim()
        .to_string();

    if formatted_text.is_empty() {
        return Err(Y2mdError::Config(
            "Ollama returned empty response".to_string(),
        ));
    }

    Ok(formatted_text)
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
    let is_long_phrase = index > 0 && index % 12 == 0; // Every ~12 words
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
