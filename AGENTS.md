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
- **Configuration Commands**: 
  - `y2md config show` - Display current configuration
  - `y2md config edit` - Open config in default editor
  - `y2md config path` - Show config file path
  - `y2md config reset` - Reset to default configuration
- **LLM Providers**: Local (Ollama), OpenAI, Anthropic, DeepSeek, Custom (OpenAI-compatible)
- **Direct Configuration**: Edit config.toml directly for all settings

## LLM Integration
- **Multiple Providers**: Local (Ollama), OpenAI, Anthropic Claude, DeepSeek, Custom (OpenAI-compatible)
- **Simple Provider Selection**: Use `--llm [provider]` flag or set default in config
- **Provider Configuration**: Configure all providers in config.toml under `[llm.local]`, `[llm.openai]`, etc.
- **API Key Management**: Set via `y2md llm set-key <provider>` or environment variables
- **Environment Variables**: Support for `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `DEEPSEEK_API_KEY`
- **Model Validation**: Checks model availability before use (Local/Ollama only)
- **Timeout Handling**: 2-minute timeout with graceful fallback
- **Fallback System**: Automatic fallback to standard formatting if LLM fails
- **Metadata Tracking**: Records which provider and model formatted each transcript in the output YAML front matter

## LLM Management Commands
- **List Models**: `y2md llm list` - List available local models (Ollama)
- **Pull Models**: `y2md llm pull <model>` - Download a model (Ollama)
- **Remove Models**: `y2md llm remove <model>` - Remove a model (Ollama)
- **Test Provider**: `y2md llm test [provider]` - Test LLM provider connection
- **Set API Key**: `y2md llm set-key <provider>` - Set API key for a provider
- **Progress Indicators**: Shows download progress with spinner
- **Error Recovery**: Clear error messages with actionable suggestions

## Supported LLM Providers
1. **Local** (Ollama)
   - Default provider for local LLM execution
   - No API key required
   - Model management via `y2md llm` commands
   - Configure endpoint and model in `[llm.local]`
   
2. **OpenAI**
   - Supports GPT-4, GPT-3.5-turbo, etc.
   - Requires API key (set via `y2md llm set-key openai` or `OPENAI_API_KEY`)
   - Configure endpoint and model in `[llm.openai]`
   
3. **Anthropic**
   - Supports Claude 3 models (Opus, Sonnet, Haiku)
   - Requires API key (set via `y2md llm set-key anthropic` or `ANTHROPIC_API_KEY`)
   - Configure endpoint and model in `[llm.anthropic]`
   
4. **Custom** (OpenAI-compatible)
   - For any OpenAI-compatible API (Groq, Together AI, LM Studio, etc.)
   - Optional API key support
   - Configure endpoint and model in `[llm.custom]`
## Output Metadata
All generated markdown files include comprehensive YAML front matter:
- **title**: Video title
- **channel**: Channel/creator name
- **url**: Original YouTube URL
- **video_id**: YouTube video ID
- **duration**: Video length (HH:MM:SS)
- **source**: Transcript source (`captions` or `whisper`)
- **language**: Transcript language code
- **extracted_at**: ISO 8601 timestamp of extraction
- **formatted_by**: Formatting method (`llm` or `standard`)
- **llm_provider**: LLM provider used (only when `formatted_by: "llm"`)
- **llm_model**: Specific model name (only when `formatted_by: "llm"`)

This metadata enables:
- Reproducibility of results
- Auditing of processing quality across providers
- Organization of transcripts by processing method
- Tracking of which tools and models were used for each transcript
