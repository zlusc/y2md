use crate::{AppConfig, CredentialManager, LlmProviderType, OllamaManager};
use console::{style, Emoji};
use std::path::PathBuf;
use std::process::Command;

static CHECKMARK: Emoji = Emoji("✓", "+");
static CROSS: Emoji = Emoji("✗", "x");
static WARNING: Emoji = Emoji("⚠", "!");
static INFO: Emoji = Emoji("ℹ", "i");

#[derive(Debug, Clone)]
pub enum DiagnosticStatus {
    Success,
    Warning,
    Error,
    Info,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub name: String,
    pub status: DiagnosticStatus,
    pub message: String,
    pub fix_command: Option<String>,
}

impl Diagnostic {
    pub fn new(
        name: String,
        status: DiagnosticStatus,
        message: String,
        fix_command: Option<String>,
    ) -> Self {
        Self {
            name,
            status,
            message,
            fix_command,
        }
    }

    pub fn success(name: String, message: String) -> Self {
        Self::new(name, DiagnosticStatus::Success, message, None)
    }

    pub fn warning(name: String, message: String, fix: Option<String>) -> Self {
        Self::new(name, DiagnosticStatus::Warning, message, fix)
    }

    pub fn error(name: String, message: String, fix: Option<String>) -> Self {
        Self::new(name, DiagnosticStatus::Error, message, fix)
    }

    pub fn info(name: String, message: String) -> Self {
        Self::new(name, DiagnosticStatus::Info, message, None)
    }
}

#[derive(Debug)]
pub struct DiagnosticReport {
    pub dependencies: Vec<Diagnostic>,
    pub llm_providers: Vec<Diagnostic>,
    pub configuration: Vec<Diagnostic>,
    pub system: Vec<Diagnostic>,
}

impl DiagnosticReport {
    pub fn new() -> Self {
        Self {
            dependencies: Vec::new(),
            llm_providers: Vec::new(),
            configuration: Vec::new(),
            system: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        let mut all_diagnostics = self
            .dependencies
            .iter()
            .chain(self.llm_providers.iter())
            .chain(self.configuration.iter())
            .chain(self.system.iter());

        all_diagnostics.any(|d| matches!(d.status, DiagnosticStatus::Error))
    }

    pub fn has_warnings(&self) -> bool {
        let mut all_diagnostics = self
            .dependencies
            .iter()
            .chain(self.llm_providers.iter())
            .chain(self.configuration.iter())
            .chain(self.system.iter());

        all_diagnostics.any(|d| matches!(d.status, DiagnosticStatus::Warning))
    }
}

pub async fn run_diagnostics() -> DiagnosticReport {
    let mut report = DiagnosticReport::new();

    report.dependencies = check_dependencies().await;
    report.llm_providers = check_llm_providers().await;
    report.configuration = check_configuration().await;
    report.system = check_system().await;

    report
}

async fn check_dependencies() -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.push(check_ytdlp());
    diagnostics.push(check_ffmpeg());
    diagnostics.push(check_whisper_models());

    diagnostics
}

fn check_ytdlp() -> Diagnostic {
    match Command::new("yt-dlp").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Diagnostic::success("yt-dlp".to_string(), format!("v{} (installed)", version))
        }
        _ => {
            let install_help = get_installation_help("yt-dlp");
            Diagnostic::error(
                "yt-dlp".to_string(),
                "not found".to_string(),
                Some(install_help),
            )
        }
    }
}

fn check_ffmpeg() -> Diagnostic {
    match Command::new("ffmpeg").arg("-version").output() {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let version = output_str
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(2))
                .unwrap_or("unknown");

            Diagnostic::success("FFmpeg".to_string(), format!("v{} (installed)", version))
        }
        _ => {
            let install_help = get_installation_help("ffmpeg");
            Diagnostic::error(
                "FFmpeg".to_string(),
                "not found".to_string(),
                Some(install_help),
            )
        }
    }
}

fn check_whisper_models() -> Diagnostic {
    let model_dir = shellexpand::tilde("~/.local/share/y2md/models/");
    let model_path_en = format!("{}ggml-base.en.bin", model_dir);
    let model_path_multi = format!("{}ggml-base.bin", model_dir);

    let en_exists = std::path::Path::new(&model_path_en).exists();
    let multi_exists = std::path::Path::new(&model_path_multi).exists();

    if en_exists || multi_exists {
        let mut models = Vec::new();
        if en_exists {
            models.push("base.en");
        }
        if multi_exists {
            models.push("base");
        }
        Diagnostic::success(
            "Whisper models".to_string(),
            format!("{} (installed)", models.join(", ")),
        )
    } else {
        Diagnostic::warning(
            "Whisper models".to_string(),
            "not found".to_string(),
            Some("Run ./download_model.sh to download Whisper models".to_string()),
        )
    }
}

async fn check_llm_providers() -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let config = AppConfig::load().ok();

    diagnostics.push(check_ollama(&config).await);
    diagnostics.push(check_api_key("OpenAI", &LlmProviderType::OpenAI));
    diagnostics.push(check_api_key("Anthropic", &LlmProviderType::Anthropic));
    diagnostics.push(check_api_key("DeepSeek", &LlmProviderType::DeepSeek));

    diagnostics
}

async fn check_ollama(config: &Option<AppConfig>) -> Diagnostic {
    let endpoint = config
        .as_ref()
        .map(|c| c.llm.local.endpoint.clone())
        .unwrap_or_else(|| "http://localhost:11434".to_string());

    let ollama = OllamaManager::new(Some(endpoint.clone()));

    if ollama.is_available().await {
        Diagnostic::success("Ollama".to_string(), format!("running at {}", endpoint))
    } else {
        Diagnostic::info(
            "Ollama".to_string(),
            "not running or not installed".to_string(),
        )
    }
}

fn check_api_key(provider_name: &str, provider_type: &LlmProviderType) -> Diagnostic {
    let cred_manager = CredentialManager::new();

    if cred_manager.has_api_key(provider_type) {
        Diagnostic::success(
            format!("{} API Key", provider_name),
            "configured".to_string(),
        )
    } else {
        Diagnostic::info(format!("{} API Key", provider_name), "not set".to_string())
    }
}

async fn check_configuration() -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    match AppConfig::config_path() {
        Ok(path) => {
            if path.exists() {
                match AppConfig::load() {
                    Ok(config) => {
                        diagnostics.push(Diagnostic::success(
                            "Config file".to_string(),
                            format!("{} (valid)", path.display()),
                        ));

                        let output_dir = PathBuf::from(&config.output_dir);
                        if output_dir.exists() {
                            if is_writable(&output_dir) {
                                diagnostics.push(Diagnostic::success(
                                    "Output dir".to_string(),
                                    format!("{} (writable)", config.output_dir),
                                ));
                            } else {
                                diagnostics.push(Diagnostic::error(
                                    "Output dir".to_string(),
                                    format!("{} (not writable)", config.output_dir),
                                    Some(format!(
                                        "Fix permissions: chmod u+w {}",
                                        config.output_dir
                                    )),
                                ));
                            }
                        } else {
                            diagnostics.push(Diagnostic::warning(
                                "Output dir".to_string(),
                                format!("{} (does not exist)", config.output_dir),
                                Some(format!("Create it: mkdir -p {}", config.output_dir)),
                            ));
                        }
                    }
                    Err(e) => {
                        diagnostics.push(Diagnostic::error(
                            "Config file".to_string(),
                            format!("{} (invalid: {})", path.display(), e),
                            Some("Fix config: y2md config edit".to_string()),
                        ));
                    }
                }
            } else {
                diagnostics.push(Diagnostic::info(
                    "Config file".to_string(),
                    "not found (using defaults)".to_string(),
                ));
            }
        }
        Err(e) => {
            diagnostics.push(Diagnostic::error(
                "Config file".to_string(),
                format!("could not determine config path: {}", e),
                None,
            ));
        }
    }

    diagnostics
}

async fn check_system() -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.push(check_disk_space());

    diagnostics
}

fn check_disk_space() -> Diagnostic {
    match get_available_space(".") {
        Ok(space_bytes) => {
            let space_gb = space_bytes as f64 / 1_073_741_824.0;

            if space_gb < 1.0 {
                Diagnostic::warning(
                    "Disk space".to_string(),
                    format!("{:.1} GB available", space_gb),
                    Some("Low disk space - transcriptions may fail".to_string()),
                )
            } else {
                Diagnostic::success(
                    "Disk space".to_string(),
                    format!("{:.0} GB available", space_gb),
                )
            }
        }
        Err(_) => Diagnostic::info("Disk space".to_string(), "could not determine".to_string()),
    }
}

fn is_writable(path: &PathBuf) -> bool {
    if let Ok(metadata) = std::fs::metadata(path) {
        !metadata.permissions().readonly()
    } else {
        false
    }
}

fn get_available_space(_path: &str) -> Result<u64, std::io::Error> {
    #[cfg(target_os = "linux")]
    {
        use std::ffi::CString;
        use std::mem;

        #[repr(C)]
        struct statvfs {
            f_bsize: u64,
            f_frsize: u64,
            f_blocks: u64,
            f_bfree: u64,
            f_bavail: u64,
            f_files: u64,
            f_ffree: u64,
            f_favail: u64,
            f_fsid: u64,
            f_flag: u64,
            f_namemax: u64,
        }

        extern "C" {
            fn statvfs(path: *const i8, buf: *mut statvfs) -> i32;
        }

        let path_c = CString::new(_path)?;
        let mut stat: statvfs = unsafe { mem::zeroed() };

        let result = unsafe { statvfs(path_c.as_ptr(), &mut stat) };

        if result == 0 {
            Ok(stat.f_bavail * stat.f_bsize)
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok(100_000_000_000)
    }
}

fn get_installation_help(tool: &str) -> String {
    let os = std::env::consts::OS;

    match (tool, os) {
        ("yt-dlp", "linux") => "Ubuntu/Debian:  sudo apt install yt-dlp
Fedora:         sudo dnf install yt-dlp
Arch:           sudo pacman -S yt-dlp
pip:            python3 -m pip install yt-dlp

After installation: y2md doctor"
            .to_string(),
        ("yt-dlp", "macos") => "Homebrew:       brew install yt-dlp
MacPorts:       sudo port install yt-dlp
pip:            python3 -m pip install yt-dlp

After installation: y2md doctor"
            .to_string(),
        ("yt-dlp", _) => "pip:            python3 -m pip install yt-dlp
More info:      https://github.com/yt-dlp/yt-dlp

After installation: y2md doctor"
            .to_string(),
        ("ffmpeg", "linux") => "Ubuntu/Debian:  sudo apt install ffmpeg
Fedora:         sudo dnf install ffmpeg
Arch:           sudo pacman -S ffmpeg

After installation: y2md doctor"
            .to_string(),
        ("ffmpeg", "macos") => "Homebrew:       brew install ffmpeg
MacPorts:       sudo port install ffmpeg

After installation: y2md doctor"
            .to_string(),
        ("ffmpeg", _) => "More info:      https://ffmpeg.org/download.html

After installation: y2md doctor"
            .to_string(),
        _ => "Please install manually".to_string(),
    }
}

pub fn print_diagnostic_report(report: &DiagnosticReport) {
    use console::Term;
    let term = Term::stdout();

    let _ = term.write_line("");
    let _ = term.write_line(&style("y2md System Diagnostics").bold().to_string());
    let _ = term.write_line(&"━".repeat(60));
    let _ = term.write_line("");

    print_section("Required Dependencies", &report.dependencies, &term);
    print_section("LLM Providers", &report.llm_providers, &term);
    print_section("Configuration", &report.configuration, &term);
    print_section("System", &report.system, &term);

    let _ = term.write_line(&"━".repeat(60));

    let status_text = if report.has_errors() {
        style("Overall Status: ✗ Issues found").red().bold()
    } else if report.has_warnings() {
        style("Overall Status: ⚠ Ready with warnings")
            .yellow()
            .bold()
    } else {
        style("Overall Status: ✓ All systems ready").green().bold()
    };

    let _ = term.write_line(&status_text.to_string());
    let _ = term.write_line("");

    print_suggestions(report, &term);
}

fn print_section(title: &str, diagnostics: &[Diagnostic], term: &console::Term) {
    let _ = term.write_line(&style(title).bold().to_string());

    for diagnostic in diagnostics {
        let (symbol, color_fn): (
            String,
            fn(console::StyledObject<String>) -> console::StyledObject<String>,
        ) = match diagnostic.status {
            DiagnosticStatus::Success => (format!("  {} ", CHECKMARK), |s| s.green()),
            DiagnosticStatus::Warning => (format!("  {} ", WARNING), |s| s.yellow()),
            DiagnosticStatus::Error => (format!("  {} ", CROSS), |s| s.red()),
            DiagnosticStatus::Info => (format!("  {} ", INFO), |s| s.cyan()),
        };

        let line = format!("{}{:<20} {}", symbol, diagnostic.name, diagnostic.message);
        let _ = term.write_line(&color_fn(style(line)).to_string());
    }

    let _ = term.write_line("");
}

fn print_suggestions(report: &DiagnosticReport, term: &console::Term) {
    let all_diagnostics = report
        .dependencies
        .iter()
        .chain(report.llm_providers.iter())
        .chain(report.configuration.iter())
        .chain(report.system.iter());

    let suggestions: Vec<_> = all_diagnostics
        .filter_map(|d| d.fix_command.as_ref())
        .collect();

    if !suggestions.is_empty() {
        let _ = term.write_line(&style("Suggested Actions:").bold().to_string());

        for (i, suggestion) in suggestions.iter().enumerate() {
            let _ = term.write_line(&format!("  {}. {}", i + 1, suggestion));
        }
        let _ = term.write_line("");
    }
}
