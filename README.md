# YouTube to Markdown Transcriber (y2md)

A powerful Rust CLI tool that converts YouTube videos to well-formatted Markdown transcripts using speech-to-text and caption extraction, with optional LLM enhancement.

## ✨ Features

- 🎯 **Simple & Intuitive** - One command to transcribe any YouTube video
- 📝 **Smart Transcription** - Captions-first with Whisper STT fallback
- 🤖 **LLM Enhancement** - Optional AI-powered formatting for better readability
- 🔄 **Multiple LLM Providers** - Local (Ollama), OpenAI, Anthropic, or custom
- 🌍 **Multi-language Support** - Transcribe in English, Spanish, French, German, and more
- 🔒 **Secure** - API keys stored in system keychain
- ⚙️ **Configurable** - Simple TOML config file you can edit directly
- 📦 **Cross-platform** - Works on Linux, macOS, and Windows

## 🚀 Quick Start

```bash
# Basic transcription (no LLM needed)
y2md https://www.youtube.com/watch?v=VIDEO_ID

# With LLM enhancement (local Ollama)
y2md https://www.youtube.com/watch?v=VIDEO_ID --llm

# With specific provider
y2md https://www.youtube.com/watch?v=VIDEO_ID --llm openai
```

That's it! Simple, right? 😊

## 📋 Requirements

### Essential
- **FFmpeg** - Audio processing
  ```bash
  # Ubuntu/Debian
  sudo apt install ffmpeg
  
  # macOS
  brew install ffmpeg
  ```

- **yt-dlp** - YouTube downloads
  ```bash
  # Via pip
  python3 -m pip install yt-dlp
  
  # Or standalone
  sudo curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp \
    -o /usr/local/bin/yt-dlp && sudo chmod a+rx /usr/local/bin/yt-dlp
  ```

### Optional (for LLM features)
- **Ollama** - For local LLM (free, private)
  ```bash
  # Visit https://ollama.ai for installation
  ```

- **OpenAI/Anthropic API Key** - For cloud LLMs (paid)

## 🛠️ Installation

```bash
# Clone and build
git clone https://github.com/yourusername/y2md.git
cd y2md
cargo build --release

# Download Whisper models (for speech-to-text)
./download_model.sh

# Optional: Install globally
cargo install --path .
```

## 📖 Usage

### Basic Commands

```bash
# Transcribe a video
y2md <YOUTUBE_URL>

# Transcribe with LLM formatting (uses configured provider)
y2md <YOUTUBE_URL> --llm

# Transcribe with specific LLM provider
y2md <YOUTUBE_URL> --llm local      # Use local Ollama
y2md <YOUTUBE_URL> --llm openai     # Use OpenAI
y2md <YOUTUBE_URL> --llm anthropic  # Use Anthropic

# Common options
y2md <URL> --out-dir ./transcripts  # Save to specific directory
y2md <URL> --lang es                # Spanish transcription
y2md <URL> --timestamps             # Include timestamps
y2md <URL> --save-raw               # Save raw + formatted transcripts
```

### Configuration

```bash
# Show current configuration
y2md config

# Edit configuration file
y2md config edit

# Show config file location
y2md config path

# Reset to defaults
y2md config reset
```

### LLM Management

```bash
# List local Ollama models
y2md llm list

# Download an Ollama model
y2md llm pull llama3.2:1b

# Remove a model
y2md llm remove llama3.2:1b

# Test LLM connection
y2md llm test          # Test default provider
y2md llm test openai   # Test specific provider

# Set API key for a provider
y2md llm set-key openai     # Prompts securely
y2md llm set-key anthropic
```

## ⚙️ Configuration

Configuration is stored in `~/.config/y2md/config.toml`. You can edit it directly!

```toml
# =============================================================================
# Y2MD Configuration
# Edit this file directly or use: y2md config edit
# =============================================================================

# Basic Settings
output_dir = "."
default_language = "en"
prefer_captions = true
timestamps = false
compact = false
paragraph_length = 4

# LLM Configuration
[llm]
enabled = false                     # Use LLM by default?
provider = "local"                  # local, openai, anthropic, custom

# Local LLM (Ollama) - Free, private
[llm.local]
endpoint = "http://localhost:11434"
model = "mistral-nemo:12b-instruct-2407-q5_0"

# OpenAI - Fast, high quality (requires API key)
[llm.openai]
endpoint = "https://api.openai.com/v1"
model = "gpt-4-turbo-preview"

# Anthropic Claude - High quality (requires API key)
[llm.anthropic]
endpoint = "https://api.anthropic.com/v1"
model = "claude-3-sonnet-20240229"

# Custom OpenAI-compatible API
[llm.custom]
endpoint = ""
model = ""

# Advanced Settings
[advanced]
whisper_model = "base"
whisper_threads = 4
cache_audio = true
```

See `config.example.toml` for a complete reference.

## 🤖 LLM Provider Setup

### Option 1: Local LLM with Ollama (Recommended)

**Pros**: Free, private, no API keys  
**Cons**: Requires local resources

```bash
# 1. Install Ollama (visit https://ollama.ai)

# 2. Pull a model
ollama pull mistral-nemo:12b-instruct-2407-q5_0

# 3. Use it!
y2md <URL> --llm local
```

### Option 2: OpenAI

**Pros**: Fast, high quality  
**Cons**: Costs money

```bash
# 1. Set API key
y2md llm set-key openai
# Enter your API key when prompted

# 2. Use it!
y2md <URL> --llm openai

# Alternative: Use environment variable
export Y2MD_OPENAI_API_KEY="sk-your-key-here"
```

### Option 3: Anthropic Claude

**Pros**: Excellent quality  
**Cons**: Costs money

```bash
# 1. Set API key
y2md llm set-key anthropic
# Enter your API key when prompted

# 2. Use it!
y2md <URL> --llm anthropic

# Alternative: Use environment variable
export Y2MD_ANTHROPIC_API_KEY="sk-ant-your-key-here"
```

### Option 4: Custom OpenAI-Compatible API

```bash
# Edit config file
y2md config edit

# Set your custom endpoint and model
[llm.custom]
endpoint = "https://api.groq.com/openai/v1"
model = "mixtral-8x7b-32768"

# Set API key if needed
y2md llm set-key custom

# Use it!
y2md <URL> --llm custom
```

## 📊 What LLM Enhancement Does

When you use `--llm`, the transcript is enhanced with:

- ✅ Better paragraph organization
- ✅ Grammar and punctuation fixes
- ✅ Filler words removed
- ✅ Improved readability
- ✅ Original meaning preserved

**Before LLM:**
```
so today we're gonna talk about rust and uh you know it's a really 
interesting language and um it has memory safety without garbage 
collection which is pretty cool right...
```

**After LLM:**
```
Today we're going to talk about Rust, a really interesting language. 
It has memory safety without garbage collection, which is quite 
impressive. The borrow checker is a unique feature that...
```

## 📝 Output Format

Transcripts are saved as Markdown files with YAML front matter containing comprehensive metadata:

```markdown
---
title: "Video Title"
channel: "Channel Name"
url: "https://youtube.com/watch?v=..."
video_id: "VIDEO_ID"
duration: "12:34"
source: "captions"              # or "whisper" for STT
language: "en"
extracted_at: "2024-03-20T10:30:00Z"
formatted_by: "llm"             # or "standard" for non-LLM
llm_provider: "anthropic"       # Provider used (if LLM formatting applied)
llm_model: "claude-3-sonnet-20240229"  # Specific model (if LLM formatting applied)
---

# Video Title

[Well-formatted transcript content...]
```

### Metadata Fields

- **title**: Video title
- **channel**: Channel/creator name
- **url**: Original YouTube URL
- **video_id**: YouTube video ID
- **duration**: Video length (HH:MM:SS)
- **source**: Transcript source (`captions` or `whisper`)
- **language**: Transcript language code
- **extracted_at**: ISO 8601 timestamp of extraction
- **formatted_by**: Formatting method (`llm` or `standard`)
- **llm_provider**: LLM provider used (only if `formatted_by: "llm"`)
- **llm_model**: Specific model name (only if `formatted_by: "llm"`)

This metadata allows you to:
- Track which LLM provider and model processed each transcript
- Reproduce results with the same configuration
- Audit processing quality across different providers
- Organize transcripts by processing method

## 🌍 Supported Languages

- English (optimized model)
- Spanish, French, German, Italian, Portuguese
- Russian, Japanese, Chinese, Korean
- Arabic, Hindi

## 🔧 Advanced Usage

```bash
# Spanish video with timestamps
y2md <URL> --lang es --timestamps --llm

# Custom paragraph length
y2md <URL> --paragraph-length 6 --llm

# Save both raw and formatted
y2md <URL> --llm --save-raw

# Dry run (preview without saving)
y2md <URL> --llm --dry-run

# Compact formatting
y2md <URL> --compact

# Force formatting for music videos
y2md <URL> --force-formatting
```

## 🔒 Security & Privacy

### Credential Storage
- **API keys**: Encrypted in system keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- **Config files**: Never contain sensitive credentials
- **Environment variables**: Supported as alternative

### Data Privacy
- **Local (Ollama)**: Data never leaves your machine
- **Cloud (OpenAI, Anthropic)**: Transcripts sent to API for formatting
- **Audio**: Cached locally, not shared with anyone

## 🐛 Troubleshooting

### "Ollama service not available"
```bash
# Check if Ollama is running
ollama serve

# Or start as service
systemctl --user start ollama
```

### "Model not found"
```bash
# List available models
y2md llm list

# Download model
y2md llm pull mistral-nemo:12b-instruct-2407-q5_0
```

### "API key not set"
```bash
# Set API key
y2md llm set-key openai

# Or use environment variable
export Y2MD_OPENAI_API_KEY="sk-..."
```

### "yt-dlp not found"
```bash
pip3 install yt-dlp
```

### "FFmpeg not found"
```bash
# Ubuntu/Debian
sudo apt install ffmpeg

# macOS
brew install ffmpeg
```

## 📚 Examples

```bash
# Quick transcription
y2md https://youtu.be/dQw4w9WgXcQ

# Professional quality with GPT-4
y2md https://youtube.com/watch?v=VIDEO_ID --llm openai

# Free local LLM
y2md https://youtube.com/watch?v=VIDEO_ID --llm local

# Spanish conference talk
y2md https://youtube.com/watch?v=VIDEO_ID --lang es --llm

# Technical video with timestamps
y2md https://youtube.com/watch?v=VIDEO_ID --timestamps --llm anthropic

# Batch processing with custom output
for url in $(cat urls.txt); do
  y2md "$url" --llm --out-dir ./transcripts
done
```

## 🏗️ Architecture

1. **URL Processing** - Extract and validate video ID
2. **Metadata Fetching** - Get video info via yt-dlp
3. **Caption Check** - Look for available subtitles
4. **Audio Download** - Download audio with caching
5. **Transcription** - Captions or Whisper STT
6. **LLM Enhancement** (optional) - Format with configured provider
7. **Output** - Save as Markdown

## 🤝 Contributing

Contributions welcome! See `AGENTS.md` for development guidelines.

## 📄 License

MIT OR Apache-2.0

## 🔗 Resources

- **Documentation**: See `REFACTORING_PLAN.md` and `IMPLEMENTATION_STATUS.md`
- **Configuration Example**: `config.example.toml`
- **GitHub Issues**: Report bugs and feature requests
- **Ollama**: https://ollama.ai
- **OpenAI**: https://platform.openai.com
- **Anthropic**: https://console.anthropic.com

## 🎯 Why y2md?

- **Simple**: Just one command to transcribe
- **Flexible**: Local or cloud LLMs, your choice
- **Private**: Local processing option available
- **Powerful**: Professional-quality transcripts
- **Free**: Works great without any paid services
- **Open**: Source code available, extensible

---

Made with ❤️ using Rust
