# Changelog

All notable changes to y2md will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **DeepSeek LLM Provider**: Added support for DeepSeek as a new LLM provider option
  - Configure via `[llm.deepseek]` section in config.toml
  - Set API key with `y2md llm set-key deepseek` or `DEEPSEEK_API_KEY` environment variable
  - Uses OpenAI-compatible API endpoint at `https://api.deepseek.com/v1`
  - Default model: `deepseek-chat`

- **Enhanced Metadata Tracking**: Output markdown files now include comprehensive metadata about the formatting process:
  - `formatted_by`: Indicates whether the transcript was formatted by `"llm"` or `"standard"` fallback
  - `llm_provider`: The LLM provider used for formatting (e.g., `"local"`, `"openai"`, `"anthropic"`, `"deepseek"`, `"custom"`)
  - `llm_model`: The specific model name used for formatting (e.g., `"claude-3-sonnet-20240229"`, `"gpt-4-turbo-preview"`, `"deepseek-chat"`)
  - This metadata allows users to:
    - Track which LLM provider and model processed each transcript
    - Reproduce results with the same configuration
    - Audit processing quality across different providers
    - Organize transcripts by processing method

### Changed
- Markdown output now includes additional YAML front matter fields for better traceability

## [0.1.1] - 2025-10-09

### Added
- Simplified configuration system with self-documenting TOML structure
- Multiple LLM provider support (Local/Ollama, OpenAI, Anthropic, Custom)
- LLM management commands (`llm list`, `llm pull`, `llm remove`, `llm test`, `llm set-key`)
- Configuration management commands (`config show`, `config edit`, `config path`, `config reset`)
- Secure API key storage in system keychain
- Progress indicators for downloads and processing
- Audio caching for faster re-processing

### Changed
- Renamed `ollama` provider to `local` for clarity
- Simplified CLI structure with intuitive command grouping
- Improved error messages with actionable suggestions
- Configuration is now directly editable via TOML file

### Removed
- OAuth authentication (simplified to API keys only)
- Complex provider management commands (replaced with direct config editing)

### Fixed
- Model availability checking for Ollama
- Fallback to standard formatting when LLM fails
- Better timeout handling for LLM requests

## [0.1.0] - 2025-10-08

### Added
- Initial release
- YouTube video transcription using captions or Whisper STT
- Basic LLM formatting support
- Multi-language support
- Markdown output with metadata

[Unreleased]: https://github.com/yourusername/y2md/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/yourusername/y2md/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/yourusername/y2md/releases/tag/v0.1.0
