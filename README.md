	# YouTube to Markdown Transcriber (y2md)

A Rust CLI tool that converts YouTube videos to Markdown transcripts using speech-to-text and caption extraction.

## Features

- ✅ URL validation and video ID extraction
- ✅ Video metadata fetching via yt-dlp
- ✅ Captions-first strategy with STT fallback
- ✅ Whisper speech-to-text integration
- ✅ Multi-language transcription support
- ✅ Audio format conversion via FFmpeg
- ✅ Progress indicators for long operations
- ✅ Audio file caching to avoid re-downloads
- ✅ Markdown formatting with YAML front matter
- ✅ Command-line interface with comprehensive options
- ✅ Cross-platform support
- ✅ Local LLM integration via Ollama for enhanced formatting
- ✅ Advanced model management system with automatic downloads
- ✅ Multi-provider LLM support (Ollama, OpenAI, LM Studio)
- ✅ Configuration management with XDG compliance
- ✅ Streaming model downloads with progress tracking
- ✅ Automatic config updates when downloading specific models
- ✅ Model availability checking with caching
- ✅ Enhanced error handling and user confirmation
- ✅ Advanced model management system with automatic downloads
- ✅ Multi-provider LLM support (Ollama, OpenAI, LM Studio)
- ✅ Configuration management with XDG compliance
- ✅ Streaming model downloads with progress tracking
- ✅ Automatic config updates when downloading specific models
- ✅ Model availability checking with caching
- ✅ Enhanced error handling and user confirmation

## Requirements

- **FFmpeg**: Required for audio format conversion
  - Install on Ubuntu/Debian: `sudo apt install ffmpeg`
  - Install on macOS: `brew install ffmpeg`
  - Install on Windows: Download from [FFmpeg.org](https://ffmpeg.org/)

- **yt-dlp**: Required for YouTube downloads and metadata extraction
  - Install via pip: `python3 -m pip install yt-dlp`
  - Install on Arch Linux: `sudo pacman -S yt-dlp`
  - Install on Ubuntu/Debian: `sudo apt install yt-dlp`
  - Install on macOS: `brew install yt-dlp`
  - Or download standalone binary: `sudo curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && sudo chmod a+rx /usr/local/bin/yt-dlp`

- **Whisper models**: Downloaded automatically via `download_model.sh`

## Installation

```bash
# Clone and build
cargo build --release

# Download whisper models and setup Ollama
./download_model.sh
```

## Usage

```bash
# Basic usage
y2md https://www.youtube.com/watch?v=VIDEO_ID

# With options
y2md https://youtu.be/VIDEO_ID \
  --out-dir ./transcripts \
  --timestamps \
  --compact \
  --verbose

# Specify language
y2md https://www.youtube.com/watch?v=VIDEO_ID --lang es

# Dry run to preview
y2md https://www.youtube.com/watch?v=VIDEO_ID --dry-run
```

## Options

- `-o, --out-dir`: Output directory (default: current directory)
- `--prefer-captions`: Use captions when available (default: true)
- `--lang`: Language code override (en, es, fr, de, it, pt, ru, ja, zh, ko, ar, hi)
- `--timestamps`: Include timestamps in transcript
- `--compact`: Compact output format
- `--cookies`: Cookies file for restricted content
- `--model`: Whisper model size (default: small)
- `--threads`: Number of threads for STT (default: 4)
- `-v, --verbose`: Verbose output
- `--use-llm`: Use local LLM (Ollama) for enhanced transcript formatting
- `--dry-run`: Preview without writing files

## Supported Languages

- **English**: Uses optimized English-only model
- **Spanish, French, German, Italian, Portuguese**: Uses multi-language model
- **Russian, Japanese, Chinese, Korean, Arabic, Hindi**: Uses multi-language model

## Configuration

Y2MD supports a flexible configuration system that allows you to customize various settings:

### Configuration Commands

```bash
# Show current configuration
y2md config show

# Set LLM provider (ollama, openai, lmstudio)
y2md config set-llm-provider ollama

# Set LLM model (automatically downloads if not available)
y2md config set-llm-model mistral-nemo:12b-instruct-2407-q5_0

# Set LLM endpoint
y2md config set-llm-endpoint http://localhost:11434

# Set LLM API key (for OpenAI)
y2md config set-llm-api-key your-api-key

# Set default language
y2md config set-language en

# Set output directory
y2md config set-output-dir ./transcripts

# Set paragraph length
y2md config set-paragraph-length 4

# Reset to defaults
y2md config reset
```

### Configuration System Features

The configuration system now includes:

- **XDG Compliance**: Configuration files are stored in standard XDG directories (`~/.config/y2md/config.toml`)
- **Automatic Model Validation**: When setting LLM models, the system validates model availability
- **Interactive Downloads**: Missing models trigger interactive download prompts with size warnings
- **Provider-Specific Validation**: Each LLM provider has specific validation rules (e.g., OpenAI requires API keys)
- **Graceful Fallbacks**: If LLM formatting fails, the system falls back to standard formatting
- **Timeout Handling**: LLM requests have 2-minute timeouts with clear error messages

### Model Management Commands

```bash
# Check model status and availability
y2md model status

# Download the current configured model
y2md model download

# Download a specific model (automatically updates config)
y2md model download <model-name>

# List available models in Ollama library
y2md model list-available [search-term]

# List locally installed models
y2md model list-local

# Remove a model from Ollama
y2md model remove <model-name>
```

### Enhanced Model Management Features

The model management system now includes:

- **Automatic Downloads**: When setting a new LLM model via `y2md config set-llm-model`, the system automatically checks if the model is available and offers to download it if not
- **Streaming Downloads**: Model downloads use Ollama's streaming API with real-time progress tracking
- **Config Auto-Update**: When downloading a specific model with `y2md model download <model-name>`, the configuration is automatically updated to use that model
- **Smart Caching**: Model availability is cached for 30 seconds to improve performance
- **User Confirmation**: Large downloads require user confirmation to prevent accidental downloads
- **Error Recovery**: Clear error messages with actionable suggestions when downloads fail
- **Availability Verification**: System verifies model installation after download completion

### Supported LLM Providers

- **Ollama**: Local LLM service (default)
- **OpenAI**: OpenAI-compatible APIs
- **LM Studio**: Local LLM service with OpenAI-compatible API

## LLM Integration

For enhanced transcript formatting, you can use various LLM providers. The setup script (`download_model.sh`) automatically installs Ollama and downloads the required model.

```bash
# Use LLM formatting (after running download_model.sh)
cargo run -- https://www.youtube.com/watch?v=VIDEO_ID --use-llm
```

The LLM will:
- Organize content into logical paragraphs
- Fix grammar and punctuation
- Remove filler words when appropriate
- Improve overall readability while maintaining original meaning

### Ollama Service Management

After running `download_model.sh`, the Ollama service should be running. If you need to manage it manually:

```bash
# Start Ollama service
systemctl --user start ollama

# Stop Ollama service
systemctl --user stop ollama

# Check Ollama status
systemctl --user status ollama

# View available models
ollama list
```

## Building

```bash
cargo build
cargo test
cargo run -- --help
```

## Architecture

1. **URL Processing**: Extract video ID and validate YouTube URL
2. **Metadata Fetching**: Get video title, channel, duration via yt-dlp
3. **Captions Check**: Look for available subtitles
4. **Audio Download**: Download audio using yt-dlp with caching
5. **Audio Conversion**: Convert to WAV format using FFmpeg
6. **Transcription**: Use Whisper speech-to-text
7. **Formatting**: Generate Markdown with YAML front matter

## Next Steps

See `spec.md` for complete requirements and `AGENTS.md` for development guidelines.
