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
- **Multiple Providers**: Ollama (local), OpenAI, Anthropic Claude, LM Studio, Custom (OpenAI-compatible)
- **Provider Management**: `y2md provider` commands for managing multiple provider configurations
- **Configurable Models**: Set via `y2md config set-llm-model` or per-provider configuration
- **Endpoint Configuration**: Customizable API endpoints for all providers
- **Secure Credential Storage**: API keys stored in system keychain (not config files)
- **API Key Support**: Secure storage via system keychain with environment variable fallback
- **Model Validation**: Checks model availability before use (Ollama)
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

## Provider Management System
- **Multi-Provider Support**: Configure and manage multiple LLM providers simultaneously
- **Provider Commands**: `y2md provider` subcommands for complete provider lifecycle
  - `y2md provider list` - List all configured providers
  - `y2md provider add` - Add a new provider configuration
  - `y2md provider remove` - Remove a provider
  - `y2md provider set-active` - Switch between providers
  - `y2md provider show` - View provider details
  - `y2md provider set-api-key` - Securely store API key in system keychain
  - `y2md provider remove-api-key` - Remove stored API key
  - `y2md provider test` - Test provider connection
- **Credential Security**: API keys stored in system keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- **Environment Variables**: Support for `Y2MD_<PROVIDER>_API_KEY` environment variables
- **Active Provider**: Set one provider as active for LLM formatting operations
- **Provider Types**: ollama, openai, anthropic, lmstudio, custom (OpenAI-compatible)

## OAuth2 Authentication System
- **Device Code Flow**: User-friendly CLI authentication for OpenAI and Anthropic
- **Auth Commands**: `y2md auth` subcommands for authentication management
  - `y2md auth login <provider>` - Login to a provider using OAuth
  - `y2md auth logout <provider>` - Logout and remove OAuth tokens
  - `y2md auth status [provider]` - Show authentication status for providers
- **Token Management**: Automatic token refresh before expiration
- **Secure Storage**: OAuth tokens stored in system keychain with encryption
- **Fallback**: Automatic fallback to API key authentication if OAuth not available
- **Token Validation**: Checks token expiration and automatically refreshes when needed
- **Multiple Auth Methods**: Support both OAuth and API key authentication per provider

## Supported LLM Providers
1. **Ollama** (Local)
   - Default provider for local LLM execution
   - No API key required
   - Model management via `y2md model` commands
   
2. **OpenAI**
   - Supports GPT-4, GPT-3.5-turbo, etc.
   - Requires API key (stored in system keychain)
   - Custom endpoint support
   
3. **Anthropic Claude**
   - Supports Claude 3 models (Opus, Sonnet, Haiku)
   - Requires API key (stored in system keychain)
   - Uses Anthropic Messages API
   
4. **LM Studio** (Local)
   - OpenAI-compatible local server
   - No API key required
   - Custom endpoint configuration
   
5. **Custom (OpenAI-compatible)**
   - For any OpenAI-compatible API (Groq, Together AI, etc.)
   - Requires custom endpoint URL
   - Optional API key support