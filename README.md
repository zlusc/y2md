# YouTube to Markdown Transcriber (y2md)

A Rust CLI tool that converts YouTube videos to Markdown transcripts using speech-to-text and caption extraction, with powerful LLM integration for enhanced formatting.

## Features

### Core Features
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

### LLM Integration Features
- ✅ **Multiple LLM Providers**: Ollama (local), OpenAI, Anthropic Claude, LM Studio, Custom (OpenAI-compatible)
- ✅ **Multi-Provider Management**: Configure and switch between multiple providers
- ✅ **OAuth2 Authentication**: Device code flow for OpenAI and Anthropic (coming soon for Anthropic)
- ✅ **Secure Credential Storage**: API keys and OAuth tokens stored in system keychain
- ✅ **Automatic Token Refresh**: OAuth tokens refreshed automatically before expiration
- ✅ **Model Management**: Automatic downloads with progress tracking
- ✅ **Provider Testing**: Test LLM connections before use
- ✅ **Configuration Management**: XDG-compliant configuration system

## Requirements

### Essential Dependencies

- **FFmpeg**: Required for audio format conversion
  - Ubuntu/Debian: `sudo apt install ffmpeg`
  - macOS: `brew install ffmpeg`
  - Windows: Download from [FFmpeg.org](https://ffmpeg.org/)

- **yt-dlp**: Required for YouTube downloads and metadata extraction
  - Via pip: `python3 -m pip install yt-dlp`
  - Ubuntu/Debian: `sudo apt install yt-dlp`
  - macOS: `brew install yt-dlp`
  - Standalone binary: `sudo curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && sudo chmod a+rx /usr/local/bin/yt-dlp`

- **Whisper models**: Downloaded automatically via `download_model.sh`

### Optional: LLM Services

- **Ollama** (for local LLM): Install from [ollama.ai](https://ollama.ai)
- **OpenAI API Key**: For GPT-4, GPT-3.5, etc.
- **Anthropic API Key**: For Claude 3 models
- **LM Studio**: For local OpenAI-compatible server

## Installation

```bash
# Clone and build
cargo build --release

# Download whisper models (for STT)
./download_model.sh

# Optional: Install Ollama for local LLM
# Visit https://ollama.ai for installation instructions
```

## Quick Start

### Basic Transcription (No LLM)

```bash
# Simple transcription
y2md https://www.youtube.com/watch?v=VIDEO_ID

# With options
y2md https://youtu.be/VIDEO_ID \
  --out-dir ./transcripts \
  --timestamps \
  --lang es
```

### Enhanced Transcription with LLM

```bash
# Using local Ollama
y2md https://youtube.com/watch?v=VIDEO_ID --use-llm

# Using configured provider
y2md https://youtube.com/watch?v=VIDEO_ID --use-llm
```

## LLM Provider Setup

### Option 1: Local LLM with Ollama (Recommended)

**Pros**: Free, private, no API keys needed  
**Cons**: Requires local resources, slower than cloud APIs

```bash
# 1. Install Ollama
# Visit https://ollama.ai

# 2. Add Ollama provider
y2md provider add ollama-local \
  --provider-type ollama \
  --model mistral-nemo:12b-instruct-2407-q5_0

# 3. Set as active provider
y2md provider set-active ollama-local

# 4. Use it
y2md https://youtube.com/watch?v=VIDEO_ID --use-llm
```

### Option 2: OpenAI with API Key

**Pros**: Fast, high quality, reliable  
**Cons**: Requires API key, costs money per use

```bash
# 1. Add OpenAI provider
y2md provider add my-openai \
  --provider-type openai \
  --model gpt-4-turbo

# 2. Set API key (will prompt securely)
y2md provider set-api-key my-openai

# 3. Set as active provider
y2md provider set-active my-openai

# 4. Use it
y2md https://youtube.com/watch?v=VIDEO_ID --use-llm
```

**Alternative: Use environment variable**
```bash
export Y2MD_MY_OPENAI_API_KEY="sk-your-api-key-here"
```

### Option 3: OpenAI with OAuth (For Subscribers)

**Pros**: No need to manage API keys, automatic token refresh  
**Cons**: Requires OpenAI subscription, browser access for initial setup

```bash
# 1. Add OpenAI provider
y2md provider add my-openai \
  --provider-type openai \
  --model gpt-4-turbo

# 2. Login with OAuth device code flow
y2md auth login my-openai
# 🔐 Starting OpenAI OAuth authentication...
# 
# Please visit: https://auth0.openai.com/activate
# And enter code: ABCD-EFGH
#
# Waiting for authentication...
# ✅ Authentication successful!

# 3. Set as active provider
y2md provider set-active my-openai

# 4. Use it
y2md https://youtube.com/watch?v=VIDEO_ID --use-llm

# Check authentication status
y2md auth status my-openai
```

### Option 4: Anthropic Claude with API Key

**Pros**: High quality formatting, good context understanding  
**Cons**: Requires API key, costs money per use

```bash
# 1. Add Anthropic provider
y2md provider add claude \
  --provider-type anthropic \
  --model claude-3-sonnet-20240229

# 2. Set API key (will prompt securely)
y2md provider set-api-key claude

# 3. Set as active provider
y2md provider set-active claude

# 4. Use it
y2md https://youtube.com/watch?v=VIDEO_ID --use-llm
```

### Option 5: LM Studio (Local OpenAI-compatible)

**Pros**: Free, private, OpenAI-compatible, local control  
**Cons**: Requires setup, slower than cloud APIs

```bash
# 1. Install and run LM Studio
# Visit https://lmstudio.ai

# 2. Add LM Studio provider
y2md provider add lmstudio-local \
  --provider-type lmstudio \
  --model local-model \
  --endpoint http://localhost:1234/v1

# 3. Set as active provider
y2md provider set-active lmstudio-local

# 4. Use it
y2md https://youtube.com/watch?v=VIDEO_ID --use-llm
```

### Option 6: Custom OpenAI-Compatible API (Groq, Together AI, etc.)

**Pros**: Flexible, various price/performance options  
**Cons**: Requires API key, varying quality

```bash
# Example: Groq
y2md provider add groq \
  --provider-type custom \
  --model mixtral-8x7b-32768 \
  --endpoint https://api.groq.com/openai/v1

y2md provider set-api-key groq
y2md provider set-active groq
```

## Provider Management Commands

### Managing Multiple Providers

```bash
# List all configured providers
y2md provider list

# Add a new provider
y2md provider add <name> \
  --provider-type <ollama|openai|anthropic|lmstudio|custom> \
  --model <model-name> \
  [--endpoint <url>]

# Show provider details
y2md provider show <name>

# Switch active provider
y2md provider set-active <name>

# Test provider connection
y2md provider test [name]

# Remove a provider
y2md provider remove <name>
```

### Managing API Keys

```bash
# Set API key (prompts securely)
y2md provider set-api-key <provider-name>

# Set API key directly (not recommended for security)
y2md provider set-api-key <provider-name> --api-key sk-...

# Remove API key
y2md provider remove-api-key <provider-name>

# Use environment variable instead
export Y2MD_<PROVIDER>_API_KEY="your-key"
```

### OAuth Authentication

```bash
# Login with OAuth device code flow
y2md auth login <provider-name>

# Check authentication status
y2md auth status                    # All providers
y2md auth status <provider-name>    # Specific provider

# Logout (remove OAuth tokens)
y2md auth logout <provider-name>
```

## Configuration Management

### Configuration Commands

```bash
# Show current configuration
y2md config show

# Set LLM provider (backward compatibility)
y2md config set-llm-provider <ollama|openai|anthropic|lmstudio|custom>

# Set LLM model
y2md config set-llm-model <model-name>

# Set LLM endpoint
y2md config set-llm-endpoint <url>

# Set LLM API key (stored in keychain)
y2md config set-llm-api-key <key>

# Set default language
y2md config set-language <en|es|fr|de|...>

# Set output directory
y2md config set-output-dir <path>

# Set paragraph length (sentences per paragraph)
y2md config set-paragraph-length <number>

# Reset to defaults
y2md config reset
```

### Configuration File

Configuration is stored in `~/.config/y2md/config.toml` (XDG-compliant):

```toml
prefer_captions = true
default_language = "en"
timestamps = false
compact = false
paragraph_length = 4

[llm]
provider = "ollama"
model = "mistral-nemo:12b-instruct-2407-q5_0"

active_provider = "my-openai"

[providers.my-openai]
name = "my-openai"
provider_type = "openai"
model = "gpt-4-turbo"

[providers.claude]
name = "claude"
provider_type = "anthropic"
model = "claude-3-sonnet-20240229"
```

**Important**: API keys and OAuth tokens are NOT stored in the config file. They are securely stored in your system keychain:
- **macOS**: Keychain Access
- **Windows**: Credential Manager
- **Linux**: Secret Service (gnome-keyring, kwallet, etc.)

## Model Management (Ollama)

### Model Commands

```bash
# Check model status and availability
y2md model status

# Download the current configured model
y2md model download

# Download a specific model (updates config)
y2md model download <model-name>

# List locally installed models
y2md model list-local

# List available models in Ollama library
y2md model list-available [search]

# Remove a model
y2md model remove <model-name>
```

### Automatic Model Downloads

When setting a new LLM model, y2md automatically checks availability and offers to download:

```bash
y2md config set-llm-model llama2:13b
# ⚠️  Model 'llama2:13b' is not available locally.
#    Do you want to download it now? [y/N]
```

## Command-Line Options

```bash
y2md [OPTIONS] [URL]

Options:
  -o, --out-dir <DIR>            Output directory (default: current)
  --prefer-captions              Use captions when available (default: true)
  --lang <CODE>                  Language code (en, es, fr, de, it, pt, ru, ja, zh, ko, ar, hi)
  --timestamps                   Include timestamps in transcript
  --compact                      Compact output format
  --paragraph-length <N>         Sentences per paragraph (default: 4)
  --force-formatting             Force enhanced formatting for music content
  --cookies <FILE>               Cookies file for restricted content
  --model <SIZE>                 Whisper model size (default: small)
  --threads <N>                  Number of threads for STT (default: 4)
  -v, --verbose                  Verbose output
  --use-llm                      Use LLM for enhanced transcript formatting
  --dry-run                      Preview without writing files
  --save-raw                     Save raw transcript to separate txt file
  -h, --help                     Print help
  -V, --version                  Print version
```

## Examples

### Multiple Provider Workflow

```bash
# Setup multiple providers
y2md provider add ollama-local --provider-type ollama --model mistral-nemo:12b
y2md provider add gpt4 --provider-type openai --model gpt-4-turbo
y2md provider add claude --provider-type anthropic --model claude-3-opus-20240229

# Set API keys
y2md provider set-api-key gpt4
y2md provider set-api-key claude

# Use different providers for different tasks
y2md provider set-active ollama-local
y2md https://youtube.com/watch?v=short-video --use-llm  # Fast, free

y2md provider set-active gpt4
y2md https://youtube.com/watch?v=long-video --use-llm   # High quality

y2md provider set-active claude
y2md https://youtube.com/watch?v=technical-video --use-llm  # Best for technical content

# Check all provider status
y2md provider list
y2md auth status
```

### Advanced Usage

```bash
# Spanish video with timestamps
y2md https://youtube.com/watch?v=VIDEO_ID \
  --lang es \
  --timestamps \
  --use-llm

# Long video with custom paragraph length
y2md https://youtube.com/watch?v=VIDEO_ID \
  --paragraph-length 6 \
  --use-llm \
  --out-dir ./long-videos

# Save both formatted and raw transcripts
y2md https://youtube.com/watch?v=VIDEO_ID \
  --use-llm \
  --save-raw

# Dry run to preview before saving
y2md https://youtube.com/watch?v=VIDEO_ID \
  --use-llm \
  --dry-run
```

## What LLMs Do for Transcript Formatting

When you use `--use-llm`, the LLM enhances the transcript by:

- ✅ Organizing content into logical paragraphs
- ✅ Fixing grammar and punctuation errors
- ✅ Removing filler words and repetitions
- ✅ Improving sentence structure
- ✅ Maintaining original meaning and tone
- ✅ Making the transcript more readable

**Before LLM:**
```
so today we're gonna talk about rust and uh you know it's a really interesting language and um it has memory safety without garbage collection which is pretty cool right and uh the borrow checker is something that...
```

**After LLM:**
```
Today we're going to talk about Rust, a really interesting language. It has memory safety without garbage collection, which is quite impressive. The borrow checker is a unique feature that...
```

## Supported Languages

- **English**: Optimized English-only Whisper model
- **Spanish, French, German, Italian, Portuguese**: Multi-language model
- **Russian, Japanese, Chinese, Korean, Arabic, Hindi**: Multi-language model

## Output Format

Transcripts are saved as Markdown files with YAML front matter:

```markdown
---
title: "Video Title"
channel: "Channel Name"
url: "https://youtube.com/watch?v=..."
video_id: "VIDEO_ID"
duration: "12:34"
source: "captions"
language: "en"
extracted_at: "2024-03-20T10:30:00Z"
---

# Video Title

[Formatted transcript content...]
```

## Troubleshooting

### LLM Issues

**Problem**: "Ollama service not available"
```bash
# Check if Ollama is running
systemctl --user status ollama

# Start Ollama
systemctl --user start ollama

# Or run Ollama directly
ollama serve
```

**Problem**: "Model not found in Ollama"
```bash
# List available models
y2md model list-local

# Download missing model
y2md model download mistral-nemo:12b-instruct-2407-q5_0
```

**Problem**: "OpenAI API returned error 401"
```bash
# Check API key is set
y2md provider show my-openai

# Reset API key
y2md provider set-api-key my-openai
```

**Problem**: "OAuth token expired"
```bash
# Re-login with OAuth
y2md auth login my-openai

# Or switch to API key
y2md provider set-api-key my-openai
```

### Provider Issues

**Problem**: "Provider not found"
```bash
# List all providers
y2md provider list

# Check active provider
y2md config show
```

**Problem**: "No active provider set"
```bash
# Set active provider
y2md provider set-active <name>
```

### General Issues

**Problem**: "yt-dlp not found"
```bash
# Install yt-dlp
python3 -m pip install yt-dlp
```

**Problem**: "FFmpeg not found"
```bash
# Ubuntu/Debian
sudo apt install ffmpeg

# macOS
brew install ffmpeg
```

## Security & Privacy

### Credential Storage

- **API Keys**: Stored encrypted in system keychain
- **OAuth Tokens**: Stored encrypted in system keychain with automatic refresh
- **Environment Variables**: Supported as alternative (less secure)
- **Config Files**: Never contain credentials

### Data Privacy

- **Local Processing**: Whisper STT runs locally on your machine
- **LLM Options**: 
  - **Local (Ollama, LM Studio)**: Data never leaves your machine
  - **Cloud (OpenAI, Anthropic)**: Transcripts sent to API for formatting
- **Audio Caching**: Audio files cached locally in temp directory

## Architecture

1. **URL Processing**: Extract video ID and validate YouTube URL
2. **Metadata Fetching**: Get video title, channel, duration via yt-dlp
3. **Captions Check**: Look for available subtitles
4. **Audio Download**: Download audio using yt-dlp with caching
5. **Audio Conversion**: Convert to WAV format using FFmpeg
6. **Transcription**: Use Whisper speech-to-text or extract captions
7. **LLM Formatting** (optional): Enhance transcript with configured LLM provider
8. **Output**: Generate Markdown with YAML front matter

## Building from Source

```bash
# Clone repository
git clone https://github.com/yourusername/y2md.git
cd y2md

# Build
cargo build --release

# Run tests
cargo test

# Install
cargo install --path .

# Run
y2md --help
```

## Contributing

Contributions welcome! See `AGENTS.md` for development guidelines.

## License

MIT OR Apache-2.0

## Additional Resources

- **Development Guidelines**: See `AGENTS.md`
- **Full Specification**: See `spec.md`
- **GitHub Issues**: Report bugs and feature requests
- **Ollama Models**: https://ollama.ai/library
- **OpenAI Platform**: https://platform.openai.com
- **Anthropic Claude**: https://console.anthropic.com
