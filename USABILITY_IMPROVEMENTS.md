# Y2MD Usability Improvements

Comprehensive recommendations to make y2md easier to use for all users, from beginners to power users.

---

## 1. UX/Interface Improvements

### 1.1 Simplify Default Workflow
**Problem**: Users must explicitly add `--llm` flag even when Ollama is running locally.

**Solutions**:
- Auto-detect if Ollama is running and available
- If detected, offer to use it with a simple prompt: "Ollama detected. Use LLM formatting? [Y/n]"
- Fall back gracefully to standard formatting if Ollama times out
- Add `--no-llm` flag to explicitly disable when auto-detection would enable it

**Impact**: Reduces friction for local LLM users

---

### 1.2 Interactive First-Time Setup
**Problem**: New users must manually edit config files or remember CLI flags.

**Solutions**:
- Create `y2md init` command with interactive wizard
- Guide users through:
  - Default output directory selection
  - Language preference
  - LLM provider choice (local/cloud/none)
  - API key setup for cloud providers
  - Whisper model download
- Save choices to config automatically
- Validate configuration before saving

**Impact**: Dramatically improves first-run experience

---

### 1.3 Better Error Messages & Guidance
**Problem**: Cryptic errors when dependencies are missing or misconfigured.

**Current Examples**:
```
yt-dlp not found. Please install yt-dlp: https://github.com/yt-dlp/yt-dlp
```

**Improved Examples**:
```
✗ yt-dlp not found

To install yt-dlp on your system:

  Ubuntu/Debian:    sudo apt install yt-dlp
  macOS:            brew install yt-dlp
  pip:              python3 -m pip install yt-dlp
  
For more options: https://github.com/yt-dlp/yt-dlp

After installation, run: y2md doctor
```

**Solutions**:
- Detect OS and provide platform-specific installation instructions
- Include copy-pasteable commands
- Add troubleshooting links
- Show what command failed and why
- Suggest next steps (e.g., "Run `y2md doctor` to verify installation")

**Impact**: Reduces support burden and user frustration

---

### 1.4 Diagnostic Command
**Problem**: Users don't know what's wrong with their setup.

**Solution**: Create `y2md doctor` command that checks:
- ✓ yt-dlp installation and version
- ✓ FFmpeg installation and version
- ✓ Whisper models availability
- ✓ Ollama service status (if configured)
- ✓ API keys presence (without revealing them)
- ✓ Network connectivity to API endpoints
- ✓ Config file validity
- ✓ Write permissions for output directory
- ✓ Disk space availability

**Output Example**:
```
y2md System Diagnostics
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Required Dependencies
  ✓ yt-dlp          v2024.10.7  (installed)
  ✓ FFmpeg          v6.0        (installed)
  ✗ Whisper model   base.en     (missing - run: y2md setup-whisper)

LLM Providers
  ✓ Ollama          running at http://localhost:11434
  ✓ OpenAI API Key  configured
  ✗ Anthropic       no API key set

Configuration
  ✓ Config file     ~/.config/y2md/config.toml (valid)
  ✓ Output dir      ~/Documents/Transcripts (writable)
  ✓ Disk space      142 GB available

Overall Status: ⚠ Ready with warnings
  → Run 'y2md setup-whisper' to download Whisper model
  → Run 'y2md llm set-key anthropic' to enable Anthropic
```

**Impact**: Empowers users to self-diagnose issues

---

### 1.5 Simplify LLM Setup
**Problem**: Multiple steps required to set up LLM providers.

**Solution**: Create `y2md setup-llm` interactive wizard:
```
LLM Provider Setup
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Choose your preferred LLM provider:

  1. Local (Ollama)     - Free, private, runs on your machine
  2. OpenAI             - Fast, high quality, ~$0.01-0.02 per video
  3. Anthropic Claude   - Excellent quality, ~$0.015 per video
  4. DeepSeek           - Good quality, competitive pricing
  5. Custom             - Any OpenAI-compatible API
  6. None               - Use standard formatting (no LLM)

Your choice [1-6]: 1

✓ Ollama selected

Checking Ollama installation...
✗ Ollama not running

Would you like to:
  1. Install Ollama now (opens browser)
  2. Start Ollama service (if already installed)
  3. Skip for now

Your choice [1-3]: 
```

**Features**:
- Auto-detect existing setup
- Show estimated costs for cloud providers
- Test connection before saving
- Download recommended models
- Set as default provider

**Impact**: Reduces setup time from minutes to seconds

---

### 1.6 Better Progress Feedback
**Problem**: Long operations appear frozen with minimal feedback.

**Current**:
```
⠋ Transcribing audio...
```

**Improved**:
```
Transcribing audio...
  └─ Processing: ██████████░░░░░░░░░░ 48% (2m 15s / ~4m 40s)
  └─ Model: whisper-base.en
  └─ Speed: 1.2x realtime
```

**Solutions**:
- Show percentage complete for all long operations
- Display estimated time remaining
- Show actual speed metrics (e.g., "processing at 1.2x realtime")
- Add elapsed time counter
- Show current operation step in multi-step processes

**Impact**: Reduces perceived wait time and anxiety

---

### 1.7 Output Improvements
**Problem**: Files saved to current directory with unwieldy names.

**Current Output**:
- Location: `./` (current directory)
- Filename: `2025-10-24_dQw4w9WgXcQ_Never_Gonna_Give_You_Up.md`

**Improved Output**:
- Default location: `~/Documents/y2md/` or `~/y2md-transcripts/`
- Filename: `never-gonna-give-you-up.md`
- Alternative with date: `2025-10-24-never-gonna-give-you-up.md`

**Solutions**:
- Create sensible default output directory on first run
- Generate URL-friendly slugs from titles
- Add `--filename-format` option: `slug`, `dated-slug`, `video-id`, `full`
- Add `--open` flag to automatically open transcript after generation
- Organize by channel/date with `--organize` flag

**Filename Format Options**:
- `slug`: `never-gonna-give-you-up.md`
- `dated-slug`: `2025-10-24-never-gonna-give-you-up.md`
- `video-id`: `dQw4w9WgXcQ.md`
- `full`: `2025-10-24_dQw4w9WgXcQ_never-gonna-give-you-up.md` (current)

**Impact**: Cleaner file organization, easier to find transcripts

---

## 2. Documentation & Discovery

### 2.1 Interactive Help
**Problem**: Users struggle to discover features and usage patterns.

**Solutions**:

#### Enhanced `--help` Output
```
y2md https://youtube.com/watch?v=VIDEO_ID

Examples:
  # Basic transcription
  y2md https://youtu.be/dQw4w9WgXcQ

  # With LLM formatting
  y2md https://youtu.be/dQw4w9WgXcQ --llm

  # Spanish video with timestamps
  y2md https://youtu.be/VIDEO_ID --lang es --timestamps

  # Use specific LLM provider
  y2md https://youtu.be/VIDEO_ID --llm anthropic

More examples: y2md examples
Documentation: https://github.com/yourusername/y2md
```

#### New `y2md examples` Command
Shows categorized real-world examples:
- Basic usage
- LLM formatting
- Language options
- Batch processing
- Custom configuration
- Troubleshooting

#### New `y2md quickstart` Command
Interactive tutorial that:
1. Explains what y2md does
2. Checks dependencies
3. Runs through a simple example
4. Shows common use cases
5. Points to documentation

**Impact**: Reduces learning curve

---

### 2.2 Better Config Discovery
**Problem**: Users don't know where config file is or how to edit it.

**Solutions**:
- On first run, display: "Creating config at: ~/.config/y2md/config.toml"
- Show config location in `y2md config` output
- Add more detailed inline comments in generated configs
- Create `y2md config validate` to check for errors
- Show warnings for deprecated or invalid options
- Add `y2md config explain <key>` to explain specific settings

**Example Output**:
```
$ y2md config explain llm.provider

llm.provider = "local"
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Sets the default LLM provider for transcript formatting.

Options:
  - local      Use Ollama (free, private, requires local resources)
  - openai     Use OpenAI GPT models (fast, paid)
  - anthropic  Use Anthropic Claude (high quality, paid)
  - deepseek   Use DeepSeek models (good quality, competitive pricing)
  - custom     Use any OpenAI-compatible API

To change: y2md config edit
To test:   y2md llm test <provider>

Current cost estimate (per 1hr video):
  - local:     $0.00 (free)
  - openai:    ~$0.01-0.02
  - anthropic: ~$0.015
  - deepseek:  ~$0.008
```

**Impact**: Makes configuration transparent and accessible

---

### 2.3 In-App Documentation
**Problem**: Users must leave the terminal to read docs.

**Solutions**:
- Add `y2md help <topic>` for detailed help on specific topics
- Topics: `setup`, `llm`, `config`, `troubleshooting`, `batch`, etc.
- Include examples and common patterns
- Show related commands at the end

**Impact**: Keeps users in flow

---

## 3. Installation & Setup

### 3.1 Dependency Management
**Problem**: Manual installation of multiple dependencies is error-prone.

**Solutions**:

#### Installation Script
Create `install.sh` that:
- Detects operating system
- Checks for missing dependencies
- Offers to install them automatically
- Downloads Whisper models
- Runs initial setup wizard
- Verifies installation with `y2md doctor`

#### New `y2md install-deps` Command
For post-installation dependency management:
```
$ y2md install-deps

Checking dependencies...

Required Dependencies
  ✓ yt-dlp already installed
  ✗ FFmpeg not found

Install FFmpeg now? [Y/n]: y

Installing FFmpeg...
  → Running: sudo apt install ffmpeg
  [sudo] password: 
  ✓ FFmpeg installed successfully

Optional Dependencies
  ✗ Whisper model (base.en) not found

Download Whisper model (85 MB)? [Y/n]: y
  ██████████████████████████ 100% (85.2 MB / 85.2 MB)
  ✓ Whisper model installed

All dependencies installed! Run: y2md doctor
```

**Impact**: One-command dependency resolution

---

### 3.2 First-Run Experience
**Problem**: Overwhelming for new users with no guidance.

**Solution**: Create guided onboarding flow:

```
$ y2md https://youtu.be/dQw4w9WgXcQ

Welcome to y2md! 🎉

This appears to be your first time using y2md.
Let's set things up quickly (< 1 minute).

Default output directory: ~/Documents/y2md-transcripts
Change it? [y/N]: n
  ✓ Output directory created

Preferred language: [en]: 
  ✓ English selected

LLM formatting improves transcript readability.
Options:
  1. Local (Ollama) - Free, private
  2. Cloud (OpenAI/Anthropic) - Paid, faster
  3. None - Standard formatting

Your choice [1-3]: 3
  ✓ Standard formatting selected
  💡 Tip: Enable LLM later with --llm flag

Download Whisper model for offline transcription (85 MB)? [Y/n]: y
  ██████████████████████████ 100% 
  ✓ Setup complete!

Starting transcription...
```

**Impact**: Smooth onboarding, no confusion

---

### 3.3 Pre-compiled Binaries
**Problem**: Rust compilation takes time and requires build tools.

**Solutions**:
- Provide pre-compiled binaries for major platforms
- Create installers for Windows (.msi), macOS (.dmg), Linux (.deb, .rpm)
- Add to package managers: Homebrew, AUR, apt repositories
- Include minimal dependencies in binary where possible

**Impact**: Instant installation for most users

---

## 4. Command Simplification

### 4.1 Smarter Defaults
**Problem**: Too many flags required for common operations.

**Solutions**:

#### Auto-detect Video Language
- Query YouTube metadata for language
- Use detected language for transcription
- Override with `--lang` if needed
- Show detected language in output

#### Intelligent Timestamps
- Auto-enable timestamps for:
  - Tutorial videos (detected from title/description)
  - Long-form content (>30 minutes)
  - Educational content
- Disable for music, vlogs, entertainment
- Override with `--timestamps` / `--no-timestamps`

#### Smart Provider Selection
- If only one API key is set, use that provider automatically
- If Ollama is running, prefer it for local processing
- Fall back to standard formatting if all LLM options fail
- Show which provider was auto-selected in output

**Impact**: Most common use case becomes just `y2md <URL>`

---

### 4.2 Workflow Aliases
**Problem**: Common workflows require remembering multiple flags.

**Solutions**:

#### `y2md quick <URL>`
Fastest transcription possible:
- Captions only (no Whisper fallback)
- No LLM formatting
- Minimal metadata
- Fast output

```bash
# Equivalent to:
y2md <URL> --prefer-captions --no-llm --compact
```

#### `y2md best <URL>`
Highest quality transcription:
- Prefer Whisper STT over captions
- Use best available LLM provider
- Include timestamps
- Detailed formatting

```bash
# Equivalent to:
y2md <URL> --prefer-captions=false --llm --timestamps
```

#### `y2md batch <file.txt>`
Process multiple URLs:
- Read URLs from file (one per line)
- Show overall progress
- Continue on errors
- Generate summary report

```bash
# Process all URLs in urls.txt
y2md batch urls.txt --llm --out-dir ./transcripts

# With options for each video
y2md batch urls.txt --llm anthropic --lang es
```

#### `y2md playlist <URL>`
Transcribe entire YouTube playlist:
- Extract all video URLs from playlist
- Process in sequence or parallel
- Skip already transcribed videos
- Organize by playlist name

```bash
y2md playlist https://youtube.com/playlist?list=PLxxx
```

**Impact**: Common workflows become memorable commands

---

### 4.3 Better Config Commands
**Problem**: Config management is opaque.

**Solutions**:

#### `y2md config show` Improvements
Show what's different from defaults:
```
Configuration (showing non-default values only)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  output_dir = "~/Documents/y2md"  (default: ".")
  llm.enabled = true               (default: false)
  llm.provider = "anthropic"       (default: "local")

All other settings are at default values.

Config file: ~/.config/y2md/config.toml
Edit:        y2md config edit
Reset:       y2md config reset
```

#### `y2md config validate`
Check configuration without running:
```
$ y2md config validate

Validating configuration...

  ✓ Syntax valid
  ✓ All keys recognized
  ⚠ Warning: llm.enabled=true but no API key set for provider 'anthropic'
    Fix: y2md llm set-key anthropic
  ✓ Output directory exists and is writable
  ✗ Error: llm.openai.model 'gpt-5' does not exist
    Available models: gpt-4-turbo-preview, gpt-3.5-turbo

Overall: ⚠ Valid with warnings
```

#### `y2md config diff`
Show differences from default config:
```
$ y2md config diff

Changes from default configuration:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  + llm.enabled = true
  - llm.enabled = false

  + llm.provider = "anthropic"
  - llm.provider = "local"

  + paragraph_length = 6
  - paragraph_length = 4

(3 changes)
```

**Impact**: Configuration becomes transparent and manageable

---

## 5. Advanced Features

### 5.1 Caching & Resume
**Problem**: Failed transcriptions must restart from scratch.

**Solutions**:
- Cache intermediate results (audio, raw transcript, formatted transcript)
- Add `--resume` flag to continue interrupted transcriptions
- Show cache size and location with `y2md cache info`
- Add `y2md cache clear` to free up space
- Implement `--skip-existing` for batch operations

---

### 5.2 Quality Presets
**Problem**: Users don't know optimal settings combinations.

**Solutions**:
Add `--preset` option:
- `fast`: Captions, no LLM, compact formatting
- `balanced`: Captions with LLM formatting (default)
- `quality`: Whisper STT, best LLM, detailed formatting
- `offline`: Everything local (Whisper + Ollama)

```bash
y2md <URL> --preset quality
```

---

### 5.3 Output Format Options
**Problem**: Markdown is the only output format.

**Solutions**:
- Add `--format` flag: `markdown`, `txt`, `pdf`, `html`, `json`
- Generate multiple formats with `--format md,pdf,txt`
- Include metadata in all formats
- Add templates for custom formatting

---

### 5.4 Watch & Auto-Process
**Problem**: Manual processing for each new video in a channel.

**Solutions**:
```bash
# Watch a channel for new uploads
y2md watch --channel @channelname --llm

# Watch a playlist
y2md watch --playlist <URL> --interval 1h

# Process new videos automatically
y2md watch --subscription-file subs.txt
```

---

## 6. Quality of Life

### 6.1 Shell Completions
Generate completions for:
- Bash
- Zsh
- Fish
- PowerShell

```bash
y2md completions bash > /etc/bash_completion.d/y2md
```

---

### 6.2 Update Notifications
- Check for new versions on startup (opt-in)
- Show what's new in updates
- Add `y2md update` command (if installed via script)

---

### 6.3 Telemetry (Opt-in)
- Collect anonymous usage statistics
- Help improve error messages
- Understand common workflows
- Fully transparent and opt-in

---

## 7. Platform-Specific Enhancements

### 7.1 macOS
- Native notifications when transcription completes
- Integration with Quick Look for preview
- Spotlight indexing for transcripts
- Shortcut/Automator actions

### 7.2 Windows
- Windows Terminal integration
- Context menu "Transcribe with y2md"
- Windows notifications
- Windows Defender exclusions guide

### 7.3 Linux
- Desktop notifications
- Nautilus/Dolphin integration
- systemd service for watching channels
- AppImage distribution

---

## Implementation Priority

### Phase 1: Essential (2-3 weeks)
1. ✅ `y2md doctor` command
2. ✅ `y2md init` setup wizard
3. ✅ Better error messages with OS-specific instructions
4. ✅ Improved progress indicators
5. ✅ Better default output directory and filenames

### Phase 2: High Impact (3-4 weeks)
6. ✅ `y2md setup-llm` wizard
7. ✅ Workflow aliases (`quick`, `best`, `batch`)
8. ✅ Enhanced `--help` and `examples` command
9. ✅ Config validation and diff commands
10. ✅ Installation script

### Phase 3: Polish (2-3 weeks)
11. ✅ Shell completions
12. ✅ Auto-detect language and smart defaults
13. ✅ First-run onboarding experience
14. ✅ `y2md install-deps` command
15. ✅ Quality presets

### Phase 4: Advanced (4-6 weeks)
16. ⚡ Caching and resume functionality
17. ⚡ Playlist support
18. ⚡ Multiple output formats
19. ⚡ Platform-specific integrations
20. ⚡ Watch channels for new videos

---

## Success Metrics

Track improvements through:
- ⏱ Time to first successful transcription
- ❌ Error rate reduction
- 📚 Documentation page views
- 💬 Support requests decrease
- ⭐ User satisfaction (GitHub stars, feedback)
- 🚀 Adoption rate (downloads, active users)

---

## User Testing Plan

1. **Beginner Users** (never used y2md)
   - Can they transcribe a video within 5 minutes?
   - Do they understand error messages?
   - Can they set up LLM without documentation?

2. **Intermediate Users** (occasional use)
   - Can they remember common commands?
   - Do they find features discoverable?
   - Can they customize settings easily?

3. **Power Users** (daily use)
   - Do workflow aliases save them time?
   - Is batch processing efficient?
   - Are advanced features accessible?

---

**Document Version**: 1.0
**Last Updated**: 2025-10-24
**Status**: Ready for implementation planning
