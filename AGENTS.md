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
- `whisper-rs` for STT (optional - requires cmake)
- `reqwest` for HTTP, `serde` for serialization
- `indicatif` for progress bars
- `symphonia` for audio processing

## LLM Integration
- Uses Ollama REST API for local LLM formatting
- Model: `mistral-nemo:12b-instruct-2407-q5_0`
- Endpoint: `http://localhost:11434/api/generate`
- Fallback to standard formatting if LLM unavailable