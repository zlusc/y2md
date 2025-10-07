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

## Requirements

- **FFmpeg**: Required for audio format conversion
  - Install on Ubuntu/Debian: `sudo apt install ffmpeg`
  - Install on macOS: `brew install ffmpeg`
  - Install on Windows: Download from [FFmpeg.org](https://ffmpeg.org/)

- **yt-dlp**: Required for YouTube downloads (automatically installed)
- **Whisper models**: Downloaded automatically via `download_model.sh`

## Installation

```bash
# Clone and build
cargo build --release

# Download whisper models
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
- `--dry-run`: Preview without writing files

## Supported Languages

- **English**: Uses optimized English-only model
- **Spanish, French, German, Italian, Portuguese**: Uses multi-language model
- **Russian, Japanese, Chinese, Korean, Arabic, Hindi**: Uses multi-language model

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