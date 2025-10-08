# AGENTS.md - YouTube to Markdown Transcriber

## Build/Test Commands
- `cargo build` - Build the project
- `cargo test` - Run all tests
- `cargo test -- --nocapture` - Run tests with output
- `cargo clippy` - Lint with Clippy
- `cargo fmt` - Format code
- `cargo run -- <YOUTUBE_URL>` - Run with YouTube URL

## Code Style Guidelines
- **Imports**: Group std, external, internal imports with blank lines
- **Formatting**: Use `cargo fmt` with default Rustfmt settings
- **Types**: Prefer explicit types, use `thiserror` for custom errors
- **Naming**: snake_case for variables/functions, PascalCase for types
- **Error Handling**: Use `anyhow` + `thiserror`, propagate with `?`
- **Async**: Use Tokio runtime, prefer async/await patterns
- **Logging**: Use `tracing` with structured logging
- **Documentation**: Include doc comments for public APIs

## Project Structure
- Main binary in `src/main.rs`
- Core logic in `src/lib.rs` modules
- Configuration in XDG directories
- Models cached in `~/.local/share/y2md/models/`

## Key Dependencies
- `clap` for CLI, `tracing` for logging
- `yt-dlp` wrapper for downloads and captions extraction
  - **Must be installed separately**: `python3 -m pip install yt-dlp` or via package manager
- `whisper-rs` for STT (optional - requires cmake)
- `reqwest` for HTTP, `serde` for serialization
- `indicatif` for progress bars
- `symphonia` for audio processing

## External Dependencies
- **yt-dlp**: Required for YouTube downloads and metadata
  - Install: `python3 -m pip install yt-dlp` or `sudo apt install yt-dlp` (Ubuntu/Debian)
- **FFmpeg**: Required for audio processing
  - Install: `sudo apt install ffmpeg` (Ubuntu/Debian) or `brew install ffmpeg` (macOS)

## Configuration System
- **Configuration File**: `~/.config/y2md/config.toml`
- **Configuration Commands**: `y2md config` subcommands for management
- **LLM Providers**: Ollama (default), OpenAI, LM Studio
- **Configuration Overrides**: CLI arguments override config file settings

## LLM Integration
- **Multiple Providers**: Ollama, OpenAI, LM Studio
- **Configurable Models**: Set via `y2md config set-llm-model`
- **Endpoint Configuration**: Customizable API endpoints
- **API Key Support**: For OpenAI-compatible providers
- **Model Validation**: Checks model availability before use
- **Timeout Handling**: 2-minute timeout with graceful fallback
- **Fallback System**: Automatic fallback to standard formatting if LLM fails

## Model Management System
- **Automatic Downloads**: `set-llm-model` automatically downloads missing models
- **Interactive Confirmation**: Asks for confirmation before downloading large models
- **Progress Indicators**: Shows download progress with spinner
- **Model Cache**: Caches local model list for 30 seconds
- **Management Commands**: `y2md model` subcommands for full control
- **Ollama Integration**: Uses Ollama API for model operations
- **Error Recovery**: Clear error messages with actionable suggestions