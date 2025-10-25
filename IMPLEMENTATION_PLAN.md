# Y2MD Usability Improvements - Implementation Plan

Detailed technical implementation plan for all usability improvements documented in `USABILITY_IMPROVEMENTS.md`.

---

## Phase 1: Essential Improvements (2-3 weeks)

### 1.1 Diagnostic Command (`y2md doctor`)

**Priority**: 🔴 Critical  
**Effort**: Medium  
**Impact**: High

#### Implementation Details

**New Module**: `src/diagnostics.rs`

```rust
pub struct Diagnostic {
    name: String,
    status: DiagnosticStatus,
    message: String,
    fix_command: Option<String>,
}

pub enum DiagnosticStatus {
    Success,
    Warning,
    Error,
    Info,
}

pub async fn run_diagnostics() -> DiagnosticReport {
    // Check all system requirements
}
```

**Checks to Implement**:
1. External dependencies (yt-dlp, FFmpeg)
   - Use `Command::new("yt-dlp").arg("--version")` to check presence
   - Parse version numbers
   - Compare against minimum required versions
   
2. Whisper models
   - Check `~/.local/share/y2md/models/` directory
   - Verify file sizes match expected values
   - List available vs. required models

3. LLM providers
   - Ollama: HTTP health check to `localhost:11434/api/tags`
   - Cloud APIs: Check keyring for API keys without revealing them
   - Test connectivity with minimal requests

4. Configuration
   - Parse config file and validate schema
   - Check for deprecated settings
   - Verify paths exist and are writable

5. System resources
   - Check disk space in output directory
   - Verify write permissions
   - Check temp directory availability

**CLI Integration**:
```rust
Commands::Doctor => {
    let report = run_diagnostics().await?;
    print_diagnostic_report(&report);
    std::process::exit(if report.has_errors() { 1 } else { 0 });
}
```

**Files to Modify**:
- Create `src/diagnostics.rs`
- Modify `src/main.rs` to add `Doctor` command
- Update `src/lib.rs` to export diagnostics module

**Testing**:
- Unit tests for each diagnostic check
- Integration test with missing dependencies
- Test on different OS platforms

---

### 1.2 Setup Wizard (`y2md init`)

**Priority**: 🔴 Critical  
**Effort**: Medium  
**Impact**: High

#### Implementation Details

**New Module**: `src/setup.rs`

```rust
pub struct SetupWizard {
    config: AppConfig,
}

impl SetupWizard {
    pub async fn run() -> Result<AppConfig> {
        // Interactive prompts
        let output_dir = prompt_output_directory()?;
        let language = prompt_default_language()?;
        let llm_choice = prompt_llm_setup().await?;
        
        // Build config
        let config = AppConfig {
            output_dir,
            default_language: language,
            llm: llm_choice,
            ..Default::default()
        };
        
        // Validate and save
        config.validate()?;
        config.save()?;
        
        Ok(config)
    }
}
```

**User Flow**:
1. Welcome message
2. Output directory selection (with default suggestion)
3. Language preference
4. LLM provider setup (with explanations and cost info)
5. Optional Whisper model download
6. Save configuration
7. Run quick validation
8. Show next steps

**CLI Integration**:
```rust
Commands::Init { force } => {
    if !force && AppConfig::exists() {
        println!("Config already exists. Use --force to overwrite.");
        return Ok(());
    }
    
    let config = SetupWizard::run().await?;
    println!("✓ Setup complete! Config saved to: {}", config.path());
}
```

**Dependencies**:
- `dialoguer` crate for interactive prompts
- `console` crate for colored output
- `indicatif` for progress bars (already in project)

**Files to Modify**:
- Create `src/setup.rs`
- Modify `src/main.rs` to add `Init` command
- Modify `Cargo.toml` to add `dialoguer` and `console`

**Testing**:
- Mock stdin/stdout for automated testing
- Test all prompt paths
- Verify config generation

---

### 1.3 Improved Error Messages

**Priority**: 🔴 Critical  
**Effort**: Low  
**Impact**: High

#### Implementation Details

**New Module**: `src/errors.rs`

```rust
#[derive(Error, Debug)]
pub enum Y2mdError {
    #[error("yt-dlp not found\n\n{}", installation_help("yt-dlp"))]
    YtDlpNotFound,
    
    #[error("FFmpeg not found\n\n{}", installation_help("ffmpeg"))]
    FFmpegNotFound,
    
    // ... other errors
}

fn installation_help(tool: &str) -> String {
    let os = std::env::consts::OS;
    
    match (tool, os) {
        ("yt-dlp", "linux") => {
            "To install yt-dlp:\n\n  \
            Ubuntu/Debian:  sudo apt install yt-dlp\n  \
            Fedora:         sudo dnf install yt-dlp\n  \
            Arch:           sudo pacman -S yt-dlp\n  \
            pip:            python3 -m pip install yt-dlp\n\n\
            After installation: y2md doctor"
        }
        ("yt-dlp", "macos") => {
            "To install yt-dlp:\n\n  \
            Homebrew:       brew install yt-dlp\n  \
            MacPorts:       sudo port install yt-dlp\n  \
            pip:            python3 -m pip install yt-dlp\n\n\
            After installation: y2md doctor"
        }
        // ... more combinations
    }
}
```

**Error Context Enhancement**:
```rust
// Wrap all external command errors with context
.map_err(|e| {
    if e.kind() == std::io::ErrorKind::NotFound {
        Y2mdError::YtDlpNotFound
    } else {
        Y2mdError::CommandFailed {
            command: "yt-dlp",
            error: e,
            suggestion: "Run 'y2md doctor' to diagnose issues",
        }
    }
})?
```

**Files to Modify**:
- Refactor `src/lib.rs` error definitions
- Update all `Command::new()` calls with better error context
- Add OS detection helper functions

**Testing**:
- Test error messages on each supported OS
- Verify all external commands have proper error wrapping

---

### 1.4 Better Progress Indicators

**Priority**: 🟡 High  
**Effort**: Medium  
**Impact**: Medium

#### Implementation Details

**Enhanced Progress Tracking**:

```rust
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

pub struct TranscriptionProgress {
    multi: MultiProgress,
    download_bar: ProgressBar,
    transcribe_bar: ProgressBar,
    format_bar: ProgressBar,
}

impl TranscriptionProgress {
    pub fn new() -> Self {
        let multi = MultiProgress::new();
        
        let download_bar = multi.add(ProgressBar::new(100));
        download_bar.set_style(
            ProgressStyle::default_bar()
                .template("{prefix} [{bar:40.cyan/blue}] {percent}% {msg}")
                .unwrap()
        );
        download_bar.set_prefix("Download");
        
        // Similar for transcribe and format bars
        
        Self { multi, download_bar, transcribe_bar, format_bar }
    }
    
    pub fn update_download(&self, percent: u64, speed: &str) {
        self.download_bar.set_position(percent);
        self.download_bar.set_message(format!("({speed})"));
    }
}
```

**Time Estimation**:
```rust
pub struct TimeEstimator {
    start_time: Instant,
    total_work: u64,
}

impl TimeEstimator {
    pub fn estimate_remaining(&self, completed: u64) -> Duration {
        let elapsed = self.start_time.elapsed();
        let rate = completed as f64 / elapsed.as_secs_f64();
        let remaining = self.total_work - completed;
        Duration::from_secs_f64(remaining as f64 / rate)
    }
}
```

**Files to Modify**:
- Create `src/progress.rs`
- Update `src/lib.rs` download/transcription functions
- Modify progress bar styles throughout codebase

**Testing**:
- Visual testing with real transcriptions
- Test time estimation accuracy
- Verify multi-step progress display

---

### 1.5 Improved Output Directory and Filenames

**Priority**: 🟡 High  
**Effort**: Low  
**Impact**: Medium

#### Implementation Details

**Filename Generation**:

```rust
pub enum FilenameFormat {
    Slug,           // "never-gonna-give-you-up.md"
    DatedSlug,      // "2025-10-24-never-gonna-give-you-up.md"
    VideoId,        // "dQw4w9WgXcQ.md"
    Full,           // "2025-10-24_dQw4w9WgXcQ_never-gonna-give-you-up.md"
}

impl FilenameFormat {
    pub fn generate(&self, metadata: &VideoMetadata) -> String {
        match self {
            Self::Slug => format!("{}.md", slugify(&metadata.title)),
            Self::DatedSlug => format!(
                "{}-{}.md",
                chrono::Utc::now().format("%Y-%m-%d"),
                slugify(&metadata.title)
            ),
            Self::VideoId => format!("{}.md", metadata.video_id),
            Self::Full => {
                // Current format
                format!(
                    "{}_{}_{}",
                    chrono::Utc::now().format("%Y-%m-%d"),
                    metadata.video_id,
                    slugify(&metadata.title)
                )
            }
        }
    }
}

fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            ' ' | '-' | '_' => '-',
            _ => '-',
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
```

**Default Output Directory**:

```rust
pub fn default_output_directory() -> PathBuf {
    if let Some(docs_dir) = dirs::document_dir() {
        docs_dir.join("y2md-transcripts")
    } else if let Some(home_dir) = dirs::home_dir() {
        home_dir.join("y2md-transcripts")
    } else {
        PathBuf::from(".")
    }
}
```

**CLI Options**:
```rust
#[arg(long, default_value = "dated-slug")]
filename_format: FilenameFormat,

#[arg(long)]
open: bool,  // Open file after generation
```

**Files to Modify**:
- Add `slugify` function to `src/lib.rs`
- Modify filename generation in `src/main.rs`
- Update `AppConfig` default `output_dir`
- Add `dirs` crate to `Cargo.toml`

**Testing**:
- Test slugification with various titles
- Test Unicode handling
- Verify path creation on all platforms

---

## Phase 2: High Impact Features (3-4 weeks)

### 2.1 LLM Setup Wizard (`y2md setup-llm`)

**Priority**: 🟡 High  
**Effort**: High  
**Impact**: High

#### Implementation Details

**Interactive Flow**:

```rust
pub async fn llm_setup_wizard() -> Result<LlmSettings> {
    use dialoguer::Select;
    
    let providers = vec![
        "Local (Ollama) - Free, private",
        "OpenAI - Fast, high quality (~$0.01-0.02/video)",
        "Anthropic Claude - Excellent quality (~$0.015/video)",
        "DeepSeek - Good quality, competitive pricing (~$0.008/video)",
        "Custom - OpenAI-compatible API",
        "None - Standard formatting",
    ];
    
    let selection = Select::new()
        .with_prompt("Choose your LLM provider")
        .items(&providers)
        .default(0)
        .interact()?;
    
    match selection {
        0 => setup_ollama().await?,
        1 => setup_openai().await?,
        2 => setup_anthropic().await?,
        3 => setup_deepseek().await?,
        4 => setup_custom().await?,
        5 => return Ok(LlmSettings::disabled()),
        _ => unreachable!(),
    }
}

async fn setup_ollama() -> Result<LlmSettings> {
    println!("\n🤖 Setting up Ollama (Local LLM)\n");
    
    // Check if Ollama is running
    let ollama = OllamaManager::new(None);
    
    if !ollama.is_available().await {
        println!("⚠️  Ollama is not running or not installed.\n");
        
        let actions = vec![
            "Install Ollama (opens browser)",
            "I've already installed it, start the service",
            "Skip for now",
        ];
        
        let choice = Select::new()
            .with_prompt("What would you like to do?")
            .items(&actions)
            .interact()?;
        
        match choice {
            0 => {
                open::that("https://ollama.ai")?;
                println!("\nAfter installing, run: y2md setup-llm");
                return Err(anyhow!("Please install Ollama first"));
            }
            1 => {
                start_ollama_service()?;
                if !ollama.is_available().await {
                    return Err(anyhow!("Could not start Ollama service"));
                }
            }
            2 => return Err(anyhow!("Setup cancelled")),
            _ => unreachable!(),
        }
    }
    
    println!("✓ Ollama is running\n");
    
    // List available models
    let models = ollama.get_local_models().await?;
    
    if models.is_empty() {
        println!("No models installed. Recommended: llama3.2:3b (fast) or mistral-nemo:12b (quality)\n");
        
        let models_to_choose = vec![
            "llama3.2:3b - Fast, 2GB download",
            "mistral-nemo:12b - High quality, 7GB download",
            "Other (I'll specify)",
        ];
        
        let model_choice = Select::new()
            .with_prompt("Select model to download")
            .items(&models_to_choose)
            .interact()?;
        
        let model_name = match model_choice {
            0 => "llama3.2:3b",
            1 => "mistral-nemo:12b-instruct-2407-q5_0",
            2 => {
                let custom: String = Input::new()
                    .with_prompt("Model name")
                    .interact_text()?;
                &custom
            }
            _ => unreachable!(),
        };
        
        println!("\n📥 Downloading {}...", model_name);
        ollama.download_model(model_name).await?;
        println!("✓ Model downloaded\n");
        
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
        // Select from available models
        let model = Select::new()
            .with_prompt("Select model to use")
            .items(&models)
            .interact()?;
        
        Ok(LlmSettings {
            enabled: true,
            provider: LlmProviderType::Local,
            local: LocalLlmConfig {
                endpoint: "http://localhost:11434".to_string(),
                model: models[model].clone(),
            },
            ..Default::default()
        })
    }
}

async fn setup_openai() -> Result<LlmSettings> {
    println!("\n🤖 Setting up OpenAI\n");
    
    // Prompt for API key
    let api_key: String = Input::new()
        .with_prompt("OpenAI API Key")
        .interact_text()?;
    
    if api_key.trim().is_empty() {
        return Err(anyhow!("API key cannot be empty"));
    }
    
    // Test the API key
    println!("\n🔄 Testing API key...");
    
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(anyhow!("Invalid API key or OpenAI API error"));
    }
    
    println!("✓ API key is valid\n");
    
    // Store in keyring
    let cred_manager = CredentialManager::new();
    cred_manager.set_api_key(&LlmProviderType::OpenAI, &api_key)?;
    
    // Select model
    let models = vec![
        "gpt-4o - Latest, best quality",
        "gpt-4-turbo - Fast and capable",
        "gpt-3.5-turbo - Fastest, cheapest",
    ];
    
    let model_choice = Select::new()
        .with_prompt("Select model")
        .items(&models)
        .default(0)
        .interact()?;
    
    let model_name = match model_choice {
        0 => "gpt-4o",
        1 => "gpt-4-turbo",
        2 => "gpt-3.5-turbo",
        _ => unreachable!(),
    };
    
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
```

**CLI Integration**:
```rust
Commands::SetupLlm => {
    let llm_settings = llm_setup_wizard().await?;
    
    // Update config
    let mut config = AppConfig::load().unwrap_or_default();
    config.llm = llm_settings;
    config.save()?;
    
    println!("\n✓ LLM setup complete!");
    println!("Test it: y2md <URL> --llm");
}
```

**Files to Modify**:
- Create interactive wizard in `src/setup.rs`
- Add `SetupLlm` command to `src/main.rs`
- Add `dialoguer`, `console`, `open` crates to `Cargo.toml`

**Testing**:
- Mock interactive prompts
- Test each provider setup path
- Verify API key storage

---

### 2.2 Workflow Aliases

**Priority**: 🟡 High  
**Effort**: Low  
**Impact**: Medium

#### Implementation Details

**New Commands**:

```rust
#[derive(Subcommand, Debug)]
enum Commands {
    // ... existing commands
    
    /// Quick transcription (captions only, no LLM)
    Quick {
        /// YouTube URL
        url: String,
        
        #[arg(short, long, default_value = ".")]
        out_dir: String,
    },
    
    /// Best quality transcription (Whisper + best LLM)
    Best {
        /// YouTube URL
        url: String,
        
        #[arg(short, long, default_value = ".")]
        out_dir: String,
    },
    
    /// Process multiple URLs from a file
    Batch {
        /// File containing URLs (one per line)
        file: PathBuf,
        
        #[arg(short, long, default_value = ".")]
        out_dir: String,
        
        #[arg(long)]
        llm: Option<Option<String>>,
        
        #[arg(long)]
        continue_on_error: bool,
    },
    
    /// Transcribe entire YouTube playlist
    Playlist {
        /// YouTube playlist URL
        url: String,
        
        #[arg(short, long, default_value = ".")]
        out_dir: String,
        
        #[arg(long)]
        llm: Option<Option<String>>,
    },
}
```

**Implementation**:

```rust
Commands::Quick { url, out_dir } => {
    println!("🚀 Quick transcription mode (captions only, no LLM)\n");
    
    // Validate URL
    let video_id = validate_youtube_url(&url)?;
    let metadata = fetch_video_metadata(&video_id).await?;
    
    println!("Transcribing: {}", metadata.title);
    
    // Force captions, no LLM, compact format
    let (transcript, source, raw) = transcribe_video(
        &video_id,
        true,  // prefer_captions
        None,  // language
        &out_dir,
        8,     // paragraph_length (longer for compact)
        false, // force_formatting
    ).await?;
    
    let markdown = format_markdown(
        &metadata,
        &transcript,
        &source,
        false, // timestamps
        true,  // compact
        8,     // paragraph_length
        false, // use_llm
        None,  // llm_provider
    ).await;
    
    save_transcript(&markdown, &metadata, &out_dir, "slug")?;
    println!("\n✓ Quick transcription complete!");
}

Commands::Best { url, out_dir } => {
    println!("⭐ Best quality mode (Whisper STT + best available LLM)\n");
    
    let video_id = validate_youtube_url(&url)?;
    let metadata = fetch_video_metadata(&video_id).await?;
    let config = AppConfig::load()?;
    
    // Use Whisper, best LLM, timestamps, detailed formatting
    let (transcript, source, raw) = transcribe_video(
        &video_id,
        false, // prefer Whisper over captions
        None,
        &out_dir,
        4,
        true,
    ).await?;
    
    let markdown = format_markdown(
        &metadata,
        &transcript,
        &source,
        true,  // timestamps
        false, // compact
        4,
        true,  // use_llm
        Some(config.llm.provider.clone()),
    ).await;
    
    save_transcript(&markdown, &metadata, &out_dir, "dated-slug")?;
    println!("\n✓ Best quality transcription complete!");
}

Commands::Batch { file, out_dir, llm, continue_on_error } => {
    println!("📦 Batch processing mode\n");
    
    // Read URLs from file
    let urls = std::fs::read_to_string(&file)?
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .collect::<Vec<_>>();
    
    println!("Found {} URLs to process\n", urls.len());
    
    let mut results = Vec::new();
    
    for (i, url) in urls.iter().enumerate() {
        println!("[{}/{}] Processing: {}", i + 1, urls.len(), url);
        
        match process_single_video(url, &out_dir, llm.as_ref()).await {
            Ok(path) => {
                println!("✓ Saved to: {}\n", path.display());
                results.push((url, Ok(path)));
            }
            Err(e) => {
                println!("✗ Error: {}\n", e);
                results.push((url, Err(e)));
                
                if !continue_on_error {
                    println!("Stopping due to error. Use --continue-on-error to skip failures.");
                    break;
                }
            }
        }
    }
    
    // Print summary
    let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
    let fail_count = results.len() - success_count;
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Batch Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✓ Successful: {}", success_count);
    println!("✗ Failed:     {}", fail_count);
    println!("  Total:      {}", results.len());
    
    if fail_count > 0 {
        println!("\nFailed URLs:");
        for (url, result) in results.iter().filter(|(_, r)| r.is_err()) {
            println!("  - {}", url);
        }
    }
}
```

**Batch Processing Helper**:
```rust
async fn process_single_video(
    url: &str,
    out_dir: &str,
    llm: Option<&Option<String>>,
) -> Result<PathBuf> {
    let video_id = validate_youtube_url(url)?;
    let metadata = fetch_video_metadata(&video_id).await?;
    
    let (transcript, source, _) = transcribe_video(
        &video_id,
        true,
        None,
        out_dir,
        4,
        false,
    ).await?;
    
    let use_llm = llm.is_some();
    let provider = llm.and_then(|l| l.as_ref().and_then(|p| p.parse().ok()));
    
    let markdown = format_markdown(
        &metadata,
        &transcript,
        &source,
        false,
        false,
        4,
        use_llm,
        provider,
    ).await;
    
    save_transcript(&markdown, &metadata, out_dir, "dated-slug")
}
```

**Files to Modify**:
- Add new commands to `src/main.rs`
- Create `batch.rs` for batch processing logic
- Update help text with examples

**Testing**:
- Test each workflow alias
- Test batch processing with failures
- Verify playlist extraction

---

### 2.3 Enhanced Help and Examples

**Priority**: 🟢 Medium  
**Effort**: Low  
**Impact**: Medium

#### Implementation Details

**Examples Command**:

```rust
Commands::Examples { category } => {
    print_examples(category.as_deref());
}

fn print_examples(category: Option<&str>) {
    match category {
        Some("basic") => print_basic_examples(),
        Some("llm") => print_llm_examples(),
        Some("batch") => print_batch_examples(),
        Some("advanced") => print_advanced_examples(),
        None => print_all_examples(),
        _ => {
            println!("Unknown category. Available: basic, llm, batch, advanced");
        }
    }
}

fn print_basic_examples() {
    println!(r#"
Basic Usage Examples
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Simple transcription (just the URL):

  y2md https://youtu.be/dQw4w9WgXcQ

Save to specific directory:

  y2md https://youtu.be/VIDEO_ID --out-dir ~/transcripts

Spanish video:

  y2md https://youtu.be/VIDEO_ID --lang es

Include timestamps:

  y2md https://youtu.be/VIDEO_ID --timestamps

More examples: y2md examples llm
"#);
}

fn print_llm_examples() {
    println!(r#"
LLM Formatting Examples
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Use default LLM provider:

  y2md https://youtu.be/VIDEO_ID --llm

Use specific provider:

  y2md https://youtu.be/VIDEO_ID --llm openai
  y2md https://youtu.be/VIDEO_ID --llm anthropic
  y2md https://youtu.be/VIDEO_ID --llm local

Setup LLM provider:

  y2md setup-llm

Test LLM connection:

  y2md llm test
  y2md llm test anthropic

More examples: y2md examples batch
"#);
}
```

**Enhanced --help**:

Modify clap attributes to include examples:

```rust
#[command(
    version,
    about,
    long_about = "YouTube to Markdown Transcriber

EXAMPLES:
  # Basic transcription
  y2md https://youtu.be/dQw4w9WgXcQ

  # With LLM formatting
  y2md https://youtu.be/VIDEO_ID --llm

  # Quick mode (fastest)
  y2md quick https://youtu.be/VIDEO_ID

  # Best quality mode
  y2md best https://youtu.be/VIDEO_ID

  # Process multiple videos
  y2md batch urls.txt --llm

For more examples: y2md examples
Full documentation: https://github.com/yourusername/y2md"
)]
```

**Files to Modify**:
- Add `Examples` command to `src/main.rs`
- Create `src/examples.rs` with categorized examples
- Update clap command attributes

---

### 2.4 Config Validation and Diff

**Priority**: 🟢 Medium  
**Effort**: Low  
**Impact**: Low

#### Implementation Details

**Config Validation**:

```rust
impl AppConfig {
    pub fn validate(&self) -> Result<Vec<ConfigWarning>> {
        let mut warnings = Vec::new();
        
        // Check output directory
        if !PathBuf::from(&self.output_dir).exists() {
            warnings.push(ConfigWarning::new(
                ConfigLevel::Warning,
                "Output directory does not exist",
                &format!("Create it: mkdir -p {}", self.output_dir),
            ));
        }
        
        // Check LLM config
        if self.llm.enabled {
            let provider = &self.llm.provider;
            
            if *provider == LlmProviderType::Local {
                // Check Ollama availability
                if !is_ollama_running(&self.llm.local.endpoint) {
                    warnings.push(ConfigWarning::new(
                        ConfigLevel::Error,
                        "LLM enabled but Ollama not running",
                        "Start Ollama: ollama serve",
                    ));
                }
            } else {
                // Check API key
                let cred_manager = CredentialManager::new();
                if !cred_manager.has_api_key(provider) {
                    warnings.push(ConfigWarning::new(
                        ConfigLevel::Error,
                        &format!("LLM enabled but no API key for {}", provider),
                        &format!("Set key: y2md llm set-key {}", provider),
                    ));
                }
            }
        }
        
        // Check paragraph length
        if self.paragraph_length == 0 {
            warnings.push(ConfigWarning::new(
                ConfigLevel::Error,
                "paragraph_length cannot be 0",
                "Set to 4 (recommended)",
            ));
        }
        
        Ok(warnings)
    }
}

pub struct ConfigWarning {
    level: ConfigLevel,
    message: String,
    suggestion: String,
}

pub enum ConfigLevel {
    Error,
    Warning,
    Info,
}
```

**Config Diff**:

```rust
Commands::Config { action } => {
    match action {
        ConfigCommands::Validate => {
            let config = AppConfig::load()?;
            let warnings = config.validate()?;
            
            if warnings.is_empty() {
                println!("✓ Configuration is valid");
            } else {
                for warning in warnings {
                    warning.print();
                }
            }
        }
        
        ConfigCommands::Diff => {
            let current = AppConfig::load()?;
            let default = AppConfig::default();
            
            print_config_diff(&current, &default);
        }
        
        // ... other commands
    }
}

fn print_config_diff(current: &AppConfig, default: &AppConfig) {
    println!("Changes from default configuration:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let mut changes = 0;
    
    if current.output_dir != default.output_dir {
        println!("  + output_dir = \"{}\"", current.output_dir);
        println!("  - output_dir = \"{}\"", default.output_dir);
        println!();
        changes += 1;
    }
    
    // ... compare all fields
    
    if changes == 0 {
        println!("No changes from default configuration");
    } else {
        println!("({} changes)", changes);
    }
}
```

**Files to Modify**:
- Add validation methods to `src/lib.rs` `AppConfig`
- Add `Validate` and `Diff` to `ConfigCommands`
- Update config command handler in `src/main.rs`

---

## Phase 3: Polish Features (2-3 weeks)

### 3.1 Shell Completions

**Priority**: 🟢 Medium  
**Effort**: Low  
**Impact**: Low

#### Implementation Details

Use `clap_complete`:

```rust
use clap_complete::{generate, Shell};

#[derive(Subcommand)]
enum Commands {
    // ... existing
    
    /// Generate shell completions
    Completions {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },
}

Commands::Completions { shell } => {
    let mut cmd = Args::command();
    let bin_name = cmd.get_name().to_string();
    generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
}
```

**Installation Instructions**:

Add to README:
```bash
# Bash
y2md completions bash > /etc/bash_completion.d/y2md

# Zsh
y2md completions zsh > ~/.zsh/completions/_y2md

# Fish
y2md completions fish > ~/.config/fish/completions/y2md.fish
```

**Files to Modify**:
- Add `clap_complete` to `Cargo.toml`
- Add `Completions` command to `src/main.rs`

---

### 3.2 Auto-detect Language

**Priority**: 🟢 Medium  
**Effort**: Medium  
**Impact**: Medium

#### Implementation Details

```rust
pub async fn detect_video_language(video_id: &str) -> Result<String> {
    let metadata_json = fetch_video_metadata_json(video_id).await?;
    
    // Check multiple fields
    let lang = metadata_json["language"]
        .as_str()
        .or_else(|| metadata_json["defaultAudioLanguage"].as_str())
        .or_else(|| {
            // Check caption languages
            metadata_json["subtitles"]
                .as_object()
                .and_then(|subs| subs.keys().next())
                .map(|s| s.as_str())
        })
        .unwrap_or("en");
    
    Ok(lang.to_string())
}
```

**Integration**:

```rust
// In transcribe_video
let language = language.or_else(|| {
    println!("🔍 Detecting video language...");
    detect_video_language(video_id).await.ok()
});

if let Some(detected_lang) = &language {
    println!("📝 Language: {}", detected_lang);
}
```

---

### 3.3 First-Run Onboarding

**Priority**: 🟢 Medium  
**Effort**: Medium  
**Impact**: Medium

#### Implementation Details

```rust
pub fn is_first_run() -> bool {
    !AppConfig::config_path()
        .map(|p| p.exists())
        .unwrap_or(false)
}

async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // Check for first run
    if is_first_run() && args.url.is_some() {
        println!("Welcome to y2md! 🎉\n");
        println!("This appears to be your first time using y2md.");
        println!("Let's set things up quickly.\n");
        
        if Confirm::new()
            .with_prompt("Run setup wizard now?")
            .default(true)
            .interact()?
        {
            SetupWizard::run().await?;
        } else {
            println!("Creating default configuration...");
            let config = AppConfig::default();
            config.save()?;
        }
        
        println!("\n✓ Ready to go! Continuing with transcription...\n");
    }
    
    // ... rest of main
}
```

---

## Phase 4: Advanced Features (4-6 weeks)

### 4.1 Caching and Resume

**Priority**: 🔵 Nice-to-have  
**Effort**: High  
**Impact**: Medium

#### Implementation Details

**Cache Structure**:

```rust
pub struct TranscriptionCache {
    cache_dir: PathBuf,
}

impl TranscriptionCache {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow!("Could not determine cache directory"))?
            .join("y2md");
        
        std::fs::create_dir_all(&cache_dir)?;
        
        Ok(Self { cache_dir })
    }
    
    pub fn get_audio(&self, video_id: &str) -> Option<PathBuf> {
        let audio_path = self.cache_dir.join(format!("{}.audio", video_id));
        if audio_path.exists() {
            Some(audio_path)
        } else {
            None
        }
    }
    
    pub fn store_audio(&self, video_id: &str, audio_data: &[u8]) -> Result<PathBuf> {
        let audio_path = self.cache_dir.join(format!("{}.audio", video_id));
        std::fs::write(&audio_path, audio_data)?;
        Ok(audio_path)
    }
    
    pub fn get_raw_transcript(&self, video_id: &str) -> Option<String> {
        let transcript_path = self.cache_dir.join(format!("{}.transcript", video_id));
        std::fs::read_to_string(transcript_path).ok()
    }
    
    pub fn clear(&self) -> Result<()> {
        std::fs::remove_dir_all(&self.cache_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }
    
    pub fn size(&self) -> Result<u64> {
        let mut total = 0;
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            total += entry.metadata()?.len();
        }
        Ok(total)
    }
}
```

**Cache Commands**:

```rust
Commands::Cache { action } => {
    match action {
        CacheCommands::Info => {
            let cache = TranscriptionCache::new()?;
            let size = cache.size()?;
            println!("Cache location: {}", cache.cache_dir.display());
            println!("Cache size: {}", format_bytes(size));
        }
        CacheCommands::Clear => {
            let cache = TranscriptionCache::new()?;
            cache.clear()?;
            println!("✓ Cache cleared");
        }
    }
}
```

---

### 4.2 Playlist Support

**Priority**: 🔵 Nice-to-have  
**Effort**: High  
**Impact**: Medium

#### Implementation Details

**Playlist Extraction**:

```rust
pub async fn extract_playlist_videos(playlist_url: &str) -> Result<Vec<String>> {
    let output = Command::new("yt-dlp")
        .args([
            "--flat-playlist",
            "--print", "id",
            playlist_url,
        ])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow!("Failed to extract playlist"));
    }
    
    let video_ids = String::from_utf8(output.stdout)?
        .lines()
        .map(|s| s.trim().to_string())
        .collect();
    
    Ok(video_ids)
}
```

**Playlist Command**:

```rust
Commands::Playlist { url, out_dir, llm } => {
    println!("📺 Extracting playlist...\n");
    
    let video_ids = extract_playlist_videos(&url).await?;
    
    println!("Found {} videos in playlist\n", video_ids.len());
    
    for (i, video_id) in video_ids.iter().enumerate() {
        println!("[{}/{}] Processing video: {}", i + 1, video_ids.len(), video_id);
        
        match process_single_video(
            &format!("https://youtu.be/{}", video_id),
            &out_dir,
            llm.as_ref(),
        ).await {
            Ok(path) => println!("✓ Saved: {}\n", path.display()),
            Err(e) => println!("✗ Error: {}\n", e),
        }
    }
}
```

---

### 4.3 Multiple Output Formats

**Priority**: 🔵 Nice-to-have  
**Effort**: High  
**Impact**: Low

#### Implementation Details

**Output Formatters**:

```rust
pub trait OutputFormatter {
    fn format(&self, metadata: &VideoMetadata, transcript: &str) -> Result<String>;
    fn extension(&self) -> &str;
}

pub struct MarkdownFormatter;
impl OutputFormatter for MarkdownFormatter {
    fn format(&self, metadata: &VideoMetadata, transcript: &str) -> Result<String> {
        // Current markdown formatting
    }
    fn extension(&self) -> &str { "md" }
}

pub struct PlainTextFormatter;
impl OutputFormatter for PlainTextFormatter {
    fn format(&self, metadata: &VideoMetadata, transcript: &str) -> Result<String> {
        Ok(format!("{}\n\n{}", metadata.title, transcript))
    }
    fn extension(&self) -> &str { "txt" }
}

pub struct JsonFormatter;
impl OutputFormatter for JsonFormatter {
    fn format(&self, metadata: &VideoMetadata, transcript: &str) -> Result<String> {
        let output = serde_json::json!({
            "title": metadata.title,
            "channel": metadata.channel,
            "url": metadata.url,
            "transcript": transcript,
        });
        Ok(serde_json::to_string_pretty(&output)?)
    }
    fn extension(&self) -> &str { "json" }
}
```

---

## Testing Strategy

### Unit Tests
- All new modules require >80% coverage
- Mock external dependencies (yt-dlp, FFmpeg, Ollama)
- Test error handling paths

### Integration Tests
- Test full workflows end-to-end
- Use small test videos
- Verify output file generation

### Platform Testing
- Test on Ubuntu, macOS, Windows
- Verify dependency detection on each platform
- Test config file paths

### User Acceptance Testing
- Recruit 5-10 beta testers
- Track time-to-first-transcription
- Collect feedback on error messages

---

## Documentation Updates

### README.md
- Update quick start with new commands
- Add setup wizard instructions
- Document all workflow aliases

### AGENTS.md
- Add new commands to build/test section
- Document new configuration options

### New Files
- `TROUBLESHOOTING.md` - Common issues and solutions
- `CONTRIBUTING.md` - Development setup guide

---

## Rollout Plan

### Week 1-2: Phase 1 Foundation
- Implement `y2md doctor`
- Implement `y2md init`
- Improve error messages
- Update progress indicators

### Week 3-4: Phase 1 Completion
- Improve output handling
- Write unit tests
- Beta test with small group
- Fix critical issues

### Week 5-7: Phase 2 High Impact
- Implement `y2md setup-llm`
- Add workflow aliases
- Enhanced help/examples
- Config validation

### Week 8-9: Phase 2 Polish
- Integration testing
- Documentation updates
- Beta testing round 2
- Bug fixes

### Week 10-11: Phase 3 Quality
- Shell completions
- Auto-detect features
- First-run experience
- Final polish

### Week 12+: Phase 4 Advanced
- Implement based on user feedback
- Prioritize most-requested features
- Consider community contributions

---

## Success Criteria

Each phase should meet these criteria before moving to next:

✅ All unit tests pass  
✅ Integration tests pass on all platforms  
✅ Documentation updated  
✅ Beta testers report positive experience  
✅ No critical bugs  
✅ Performance acceptable (no regressions)  

---

## Resource Requirements

### Development
- 1 senior developer (full-time, 12 weeks)
- 1 junior developer (part-time, testing support)

### Testing
- 5-10 beta testers per phase
- CI/CD pipeline (GitHub Actions)
- Test infrastructure (Linux, macOS, Windows VMs)

### Documentation
- Technical writer (part-time, weeks 4, 8, 11)
- Video tutorial creator (optional, week 12)

---

**Plan Version**: 1.0  
**Created**: 2025-10-24  
**Status**: Ready for implementation
