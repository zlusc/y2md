# YouTube → Markdown Transcriber (Rust) — Requirement Specification

## 1) Summary

A small cross-platform CLI tool that, given a single YouTube URL, retrieves the video’s spoken content and saves a clean Markdown transcript to a chosen folder. The tool prefers official captions when available; otherwise it downloads audio and runs offline speech-to-text (Whisper via `whisper-rs`). ([Google for Developers][1])

---

## 2) Goals & Non-Goals

**Goals**

- Input a YouTube link → Output a `.md` file with title, metadata, and transcript.
- Use official captions if present; fall back to high-quality offline transcription.
- Produce readable Markdown with optional timestamps and simple sections.
- Work offline for STT once audio is downloaded and the model is present.
- Be easy to install and ship as a single static binary when possible.

**Non-Goals**

- Editing captions, diarization/speaker labeling beyond “Unknown Speaker”.
- Multi-URL batch processing (could be a future enhancement).
- Full subtitle formatting (ASS/SSA), translation, or summarization.

---

## 3) User Stories

- **As a researcher**, I paste a YouTube URL and get a Markdown transcript I can search and quote.
- **As a developer**, I want a deterministic CLI that works without cloud keys.
- **As a traveler with spotty internet**, I want offline transcription when captions are missing.

---

## 4) Inputs & Outputs

**Input**

- One YouTube URL (any standard watch/shorts/share link).

**Output**

- `YYYY-MM-DD_<video-id>_<sanitized-title>.md` saved under the target folder.
- Markdown header block:
  - Title
  - Channel name (if available)
  - Original URL
  - Video ID
  - Duration
  - Transcript source: `captions` | `whisper`
  - Language detected/used
  - Timestamp of extraction

**Transcript body**

- Option `--timestamps`: prepend `[hh:mm:ss]` per line/segment.
- Option `--compact`: merge short lines for readability.
- Option `--no-metadata`: omit header block.

---

## 5) Functional Requirements

### 5.1 URL Handling

- Normalize & validate URL; extract `videoId` (robust against `youtu.be`, `shorts`, query params).
- Option `--cookies <file>` to support age-restricted/region-locked content (passed to downloader).

### 5.2 Metadata Fetch

- Retrieve lightweight metadata (title, channel, duration) via downloader JSON or web response.
- Fail gracefully if only partial metadata available.

### 5.3 Captions First Strategy

1. **Official captions path:**
   - Try YouTube Data API v3 Captions if user provides API key & OAuth (optional). ([Google for Developers][1])
   - Also attempt unauthenticated timed-text endpoint for public tracks when possible. ([Stack Overflow][2])
   - If a caption track is found (preferred language or auto-generated), download, convert to Markdown, done.

2. **Fallback STT path:**
   - Download **audio only** (m4a/webm) using `yt-dlp` (via Rust wrapper) with `--no-playlist`, `--extract-audio`, `--audio-format wav/m4a`. ([Crates.io][3])
   - Transcribe locally via Whisper (`whisper-rs` binding to `whisper.cpp`) using a configurable GGUF model (e.g., `base`, `small`, `medium`, `large-v3`, `large-v3-turbo`). ([Crates.io][4])
   - Optional VAD (voice activity detection) to segment before transcription for quality. ([Docs.rs][5])

### 5.4 Language Handling

- Auto-detect transcript language from captions or Whisper; allow `--lang <code>` override.

### 5.5 Markdown Formatting

- Convert captions/segments into Markdown paragraphs with optional timestamps.
- Escape Markdown special characters in text content.
- Append a “Provenance” footer noting the source and processing steps.

### 5.6 Error Handling

- Clear errors for: invalid URL, no network, geo/age restrictions, empty audio, model missing, or insufficient disk space.
- Non-zero exit codes with actionable messages.
- `--retry` policy for transient network failures.

---

## 6) Non-Functional Requirements

- **Accuracy:** Prefer official captions; otherwise Whisper (recognized as highly accurate among open-source ASR). ([Deepgram][6])
- **Performance:** Reasonable speed on consumer CPUs; allow `--threads N`, `--model-size`, and `--beam-size`.
- **Resource Use:** Stream audio where possible; avoid loading full video; respect temp folder limits.
- **Portability:** Linux (x86_64/aarch64), macOS (Intel/Apple Silicon), Windows (x86_64).
- **Reliability:** Deterministic output given same inputs & model.
- **Compliance:** Respect YouTube terms and local laws; user supplies cookies/API keys where needed.

---

## 7) Architecture & Tech Choices (Rust)

### 7.1 Runtime & CLI

- **Rust 1.78+**, async with **Tokio** for IO.
- **CLI**: `clap` v4 (derive API) with subcommands hidden initially (`transcribe`, `probe`).
- **Logging**: `tracing` + `tracing-subscriber` (pretty env filter).
- **Errors**: `anyhow` + `thiserror`.
- **HTTP**: `reqwest` (native TLS) for metadata/fallback calls.
- **Serde**: `serde`, `serde_json` for metadata and config.
- **Progress**: `indicatif` for spinners/bars.
- **FS paths**: `camino` or `path-absolutize`; config in `directories`.

### 7.2 Download Layer

- Prefer **`yt-dlp`** via Rust wrapper:
  - Option A: `yt-dlp` wrapper crate (`yt-dlp` / `ytdlp_bindings`) to manage the binary and invoke with structured options. ([Crates.io][3])
  - Option B: shell out to system `yt-dlp` if wrapper is insufficient; verify version at runtime.

- **FFmpeg**: required by `yt-dlp` for audio extraction; detect presence and error nicely.

### Installation Requirements

- **yt-dlp**: Must be installed and available in system PATH
  - Install via pip: `python3 -m pip install yt-dlp`
  - Install via package manager (Arch: `sudo pacman -S yt-dlp`, Ubuntu/Debian: `sudo apt install yt-dlp`, macOS: `brew install yt-dlp`)
  - Or download standalone binary: `sudo curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && sudo chmod a+rx /usr/local/bin/yt-dlp`

- **FFmpeg**: Required for audio processing
  - Install via package manager (Ubuntu/Debian: `sudo apt install ffmpeg`, macOS: `brew install ffmpeg`)
  - Or download from [FFmpeg.org](https://ffmpeg.org/)

### 7.3 Captions Layer

- Attempt **YouTube Captions API** when configured (OAuth/API key). ([Google for Developers][1])
- Attempt **timed-text** endpoint for public captions when accessible. ([Stack Overflow][2])
- Parse XML/JSON → internal segment model → Markdown.

### 7.4 STT Layer

- **Whisper via `whisper-rs`** (FFI to `whisper.cpp`):
  - Ship instructions to download GGUF models at first run; cache in `~/.local/share/y2md/models`. ([Crates.io][4])
  - Expose `--model` (`tiny/base/small/medium/large-v3/large-v3-turbo`) and `--device` (`cpu` only; GPU via `whisper.cpp` build flags if available).
  - Note: “large-v3-turbo” is substantially faster with competitive accuracy for many cases. ([Modal][7])

### 7.5 Configuration

- `y2md.toml` in XDG config dir:
  - `default_lang`, `prefer_captions = true`, `downloader = "yt-dlp"`, `audio_format = "wav"`, `model = "small"`, `output_dir`, `timestamps = true`, `cookies_file`, `api_key` (optional).

---

## 8) Command Line Design

```
y2md <YOUTUBE_URL>
  [--out-dir <PATH>]
  [--prefer-captions | --no-prefer-captions]
  [--lang <xx>]
  [--timestamps | --no-timestamps]
  [--compact]
  [--cookies <cookies.txt>]
  [--model <tiny|base|small|medium|large-v3|large-v3-turbo>]
  [--threads <N>]
  [--beam-size <N>]
  [--verbose]
  [--dry-run]
```

**Examples**

- `y2md https://youtu.be/abc123 -o transcripts/ --timestamps`
- `y2md https://youtube.com/watch?v=xyz --no-prefer-captions --model small`

---

## 9) File & Folder Layout

```
~/.config/y2md/y2md.toml        # config
~/.local/share/y2md/models/     # GGUF models cache
./transcripts/
  2025-10-07_abc123_Title.md
.tmp/y2md/                      # temp audio & intermediates
```

---

## 10) Markdown Format (example skeleton)

```markdown
---
title: "<Video Title>"
channel: "<Channel Name>"
url: "https://www.youtube.com/watch?v=<id>"
video_id: "<id>"
duration: "01:23:45"
source: "captions" # or "whisper"
language: "en"
extracted_at: "2025-10-07T10:00:00-07:00"
---

# <Video Title>

[00:00:00] Intro and overview…

[00:02:15] Key concept #1…

[00:05:42] …
```

---

## 11) Edge Cases & Policies

- **No captions & STT model missing** → prompt to download model or fail with instructions.
- **Age-restricted/“sign in”** → support `--cookies` (Netscape format) and document usage.
- **Rate limits** on caption endpoints → automatic backoff; allow `--retries`.
- **Regional blocks** → surface clear error; suggest cookies or VPN (user responsibility).
- **Terms of Service**: make the user explicitly confirm they are authorized to download/transcribe for personal use.

---

## 12) Packaging & Distribution

- Build static where possible:
  - Linux (musl): `cargo zigbuild` or `cross`.
  - macOS (Universal2 if feasible).
  - Windows MSVC.

- Optional `cargo-dist` to automate release artifacts & checksums.
- Pre-flight checks at runtime for dependencies (`yt-dlp`, `ffmpeg`) with version hints.

---

## 13) Quality: Logging, Telemetry, Tests

- **Logging**: `RUST_LOG=y2md=debug` with `tracing`.
- **Unit tests**: URL parsing, filename sanitization, caption parsing.
- **Integration tests**: public sample videos (short/long), captions vs. whisper, multilingual.
- **Golden files**: snapshot Markdown outputs for stability across changes.

---

## 14) Performance Targets

- Cold start (captions path): < 3s on broadband if captions exist.
- Whisper fallback:
  - `small` model transcribes 10-min talk on 8-core CPU in acceptable time (documented table in README).
  - `large-v3-turbo` recommended for speed/quality tradeoff. ([Modal][7])

---

## 15) Security & Privacy

- Do not store URLs or transcripts unless asked; default to local filesystem only.
- Never upload audio/transcripts; no telemetry.
- Warn users about sharing cookies; restrict permissions to 0600.

---

## 16) Roadmap (Post-v1)

- Batch mode (accept file of URLs).
- Speaker diarization & punctuation refinement.
- Optional summary or keyword extraction.
- GPU acceleration via `whisper.cpp` build flags.
- Simple GUI wrapper (Tauri) reusing the CLI core.
- Additional providers (e.g., Faster-Whisper backend) if Rust bindings mature. ([Deepgram][6])

---

## 17) Implementation Notes & References

- **Whisper in Rust:** `whisper-rs` (bindings to `whisper.cpp`) with VAD helpers and examples. ([Crates.io][4])
- **Download layer:** `yt-dlp` Rust wrapper and/or bindings; proven approach to robustly fetch audio/metadata. ([Crates.io][3])
- **Captions:** Official YouTube Captions API (needs OAuth scope) and public timed-text endpoint when available. ([Google for Developers][1])
- **ASR tradeoffs:** Open-source benchmarks consistently place Whisper near top for accuracy; v3-turbo variant noted for speed. ([Deepgram][6])
- **Rust CLI best practices:** Modern stacks emphasize `clap` + `tracing` + `anyhow` and thoughtful error handling; plenty of up-to-date guides. ([Chris Woody Woodruff][8])

---

## 18) Acceptance Criteria

- Given a valid public YouTube URL **with captions**, running `y2md <url>` produces a Markdown file where `source: "captions"` and text matches the source captions.
- Given a valid public YouTube URL **without captions**, running `y2md <url>` downloads audio, runs Whisper locally, and produces a Markdown transcript with `source: "whisper"`.
- `--timestamps` adds `[hh:mm:ss]` markers; `--compact` reduces line breaks.
- Missing dependencies (`yt-dlp`, `ffmpeg`, or model) trigger clear guidance and non-zero exit.
- Works on Linux/macOS/Windows in CI release artifacts.

---

### Nice-to-Have Developer Ergonomics

- `y2md probe <url>`: print JSON of what will happen (captions available? which languages?).
- `y2md doctor`: verify environment (ffmpeg, yt-dlp, model cache).
- `y2md cache --purge`: remove temp files and models.

---

If you’d like, I can turn this into a starter repo layout (Cargo workspace + modules) and a minimal `main.rs` that wires `clap`, downloader stubs, captions parser, and the Whisper transcribe path.

[1]: https://developers.google.com/youtube/v3/docs/captions?utm_source=chatgpt.com "Captions | YouTube Data API"

[2]: https://stackoverflow.com/questions/14061195/how-to-get-transcript-in-youtube-api-v3?utm_source=chatgpt.com "How to get \"transcript\" in youtube-api v3"
[3]: https://crates.io/crates/yt-dlp?utm_source=chatgpt.com "yt-dlp - crates.io: Rust Package Registry"
[4]: https://crates.io/crates/whisper-rs?utm_source=chatgpt.com "whisper-rs - crates.io: Rust Package Registry"
[5]: https://docs.rs/whisper-rs?utm_source=chatgpt.com "whisper_rs - Rust"
[6]: https://deepgram.com/learn/benchmarking-top-open-source-speech-models?utm_source=chatgpt.com "3 Best Open-Source ASR Models Compared: Whisper, ..."
[7]: https://modal.com/blog/open-source-stt?utm_source=chatgpt.com "The Top Open Source Speech-to-Text (STT) Models in 2025"
[8]: https://www.woodruff.dev/building-a-cli-app-in-rust-my-first-project/?utm_source=chatgpt.com "Building a CLI App in Rust: My First Project"
