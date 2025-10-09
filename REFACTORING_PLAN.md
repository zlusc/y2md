# Y2MD Refactoring Plan
## Simplification and UX Improvement

**Date**: 2025-10-09  
**Version**: 1.0  
**Status**: Planning Phase

---

## Executive Summary

This document outlines a comprehensive refactoring plan to simplify the y2md tool's configuration and command structure. The current system has become overly complex with multiple layers of abstraction (providers, models, config, auth) that confuse users. The goal is to create an intuitive, transparent tool that matches the user's mental model.

---

## Current Problems

### 1. **Overly Complex Command Structure**
- Too many subcommands: `config`, `provider`, `model`, `auth`
- Unclear hierarchy and relationships
- Users need to constantly check `--help`
- Command names don't match user intent

### 2. **Confusing Configuration System**
- Mix of legacy `llm` config and new `providers` system
- Unclear which system is active
- No easy way to see or edit full configuration
- Multiple ways to configure the same thing

### 3. **Non-Intuitive User Flow**
```
Current (confusing):
User wants LLM → Must understand providers → Add provider → Set API key → Set active → Use --use-llm

Desired (intuitive):
User wants LLM → Run with --llm openai → Enter API key if prompted → Done
```

### 4. **Poor Documentation**
- Complex configuration requires extensive documentation
- Users can't easily understand what's configured
- Config file location and format unclear

---

## Design Goals

1. **Simplicity First**: Common tasks should be simple commands
2. **Transparency**: Configuration should be visible and editable
3. **Progressive Disclosure**: Complexity only when needed
4. **Mental Model Alignment**: Commands match how users think
5. **Self-Documenting**: Config file explains itself

---

## Proposed Solution

### 1. Simplified Command Structure

#### Core Commands
```bash
# Main functionality
y2md <URL>                          # Basic transcription
y2md <URL> --llm                    # Use LLM (from config)
y2md <URL> --llm local              # Use local LLM (Ollama)
y2md <URL> --llm openai             # Use OpenAI
y2md <URL> --llm anthropic          # Use Anthropic
y2md <URL> --llm custom             # Use custom endpoint

# Configuration management
y2md config                         # Show current config
y2md config edit                    # Open in $EDITOR
y2md config path                    # Show config file path
y2md config reset                   # Reset to defaults
y2md config init                    # Interactive setup wizard

# LLM helpers
y2md llm list                       # List local models
y2md llm pull <model>               # Download local model
y2md llm remove <model>             # Remove local model
y2md llm test [provider]            # Test LLM connection
y2md llm set-key <provider>         # Set API key for provider
```

#### Removed Commands
```bash
# These are removed
y2md provider ...                   # ❌ Too abstract
y2md model ...                      # ❌ Merged into llm
y2md auth ...                       # ❌ Handled automatically
y2md config set-llm-provider        # ❌ Just edit config
y2md config set-llm-model           # ❌ Just edit config
y2md config show                    # ❌ Now just "config"
```

### 2. Simplified Configuration File

**Location**: `~/.config/y2md/config.toml`

```toml
# =============================================================================
# Y2MD Configuration
# Edit this file directly or use: y2md config edit
# =============================================================================

# -----------------------------------------------------------------------------
# Basic Settings
# -----------------------------------------------------------------------------
output_dir = "."                    # Where to save transcripts
default_language = "en"             # Default language code
prefer_captions = true              # Try captions before STT

# -----------------------------------------------------------------------------
# Formatting Options
# -----------------------------------------------------------------------------
timestamps = false                  # Include timestamps in output
compact = false                     # Use compact formatting
paragraph_length = 4                # Sentences per paragraph

# -----------------------------------------------------------------------------
# LLM Configuration
# -----------------------------------------------------------------------------
[llm]
enabled = false                     # Use LLM formatting by default
provider = "local"                  # Default: "local", "openai", "anthropic", "custom"

# Local LLM (Ollama)
[llm.local]
endpoint = "http://localhost:11434"
model = "mistral-nemo:12b-instruct-2407-q5_0"

# OpenAI
[llm.openai]
model = "gpt-4-turbo-preview"
endpoint = "https://api.openai.com/v1"
# API key stored in system keychain for security

# Anthropic Claude
[llm.anthropic]
model = "claude-3-sonnet-20240229"
endpoint = "https://api.anthropic.com/v1"
# API key stored in system keychain for security

# Custom OpenAI-compatible API
[llm.custom]
endpoint = ""                       # Your API endpoint
model = ""                          # Model name
# API key stored in system keychain for security

# -----------------------------------------------------------------------------
# Advanced Options
# -----------------------------------------------------------------------------
[advanced]
whisper_model = "base"              # Whisper model for STT
whisper_threads = 4                 # Threads for STT
cache_audio = true                  # Cache downloaded audio
```

### 3. New Data Structures

#### Configuration Structure (Rust)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // Basic settings
    pub output_dir: String,
    pub default_language: String,
    pub prefer_captions: bool,
    
    // Formatting
    pub timestamps: bool,
    pub compact: bool,
    pub paragraph_length: usize,
    
    // LLM configuration
    pub llm: LlmSettings,
    
    // Advanced
    pub advanced: AdvancedSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSettings {
    pub enabled: bool,
    pub provider: LlmProviderType,  // enum: Local, OpenAI, Anthropic, Custom
    pub local: LocalLlmConfig,
    pub openai: OpenAiConfig,
    pub anthropic: AnthropicConfig,
    pub custom: CustomLlmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalLlmConfig {
    pub endpoint: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub endpoint: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub endpoint: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomLlmConfig {
    pub endpoint: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSettings {
    pub whisper_model: String,
    pub whisper_threads: usize,
    pub cache_audio: bool,
}
```

---

## Detailed Changes

### Phase 1: Configuration System Refactoring

#### File Changes
- `src/lib.rs`:
  - Remove `ProviderConfig` struct
  - Remove `providers` HashMap from `AppConfig`
  - Remove `active_provider` field
  - Add new `LlmSettings` struct
  - Add `LocalLlmConfig`, `OpenAiConfig`, `AnthropicConfig`, `CustomLlmConfig` structs
  - Simplify `AppConfig::load()` to use new structure
  - Update `AppConfig::default()` with new defaults

#### New Files
- Create migration function to upgrade old configs to new format

#### Removed Code
- `ProviderConfig` struct and all related methods
- `add_provider()`, `remove_provider()`, `get_provider()`, etc.
- OAuth-related code (out of scope for MVP)

### Phase 2: CLI Command Restructuring

#### File Changes
- `src/main.rs`:
  - Remove `ProviderCommands` enum
  - Remove `ModelCommands` enum  
  - Remove `AuthCommands` enum
  - Simplify `ConfigCommands` enum to: `Show`, `Edit`, `Path`, `Reset`, `Init`
  - Add new `LlmCommands` enum: `List`, `Pull`, `Remove`, `Test`, `SetKey`
  - Update `Args` struct to use `--llm <provider>` instead of `--use-llm`
  - Remove all provider/model/auth command handlers
  - Add new simplified command handlers

#### Command Mapping
```
Old → New
------------------------------
config show                 → config
config set-llm-provider     → (edit config file)
config set-llm-model        → (edit config file)
config set-llm-endpoint     → (edit config file)
config reset                → config reset
(no equivalent)             → config edit
(no equivalent)             → config path
(no equivalent)             → config init

model status                → llm test
model download              → llm pull
model list-local            → llm list
model list-available        → (removed - use ollama library)
model remove                → llm remove

provider list               → (removed)
provider add                → (removed)
provider remove             → (removed)
provider set-active         → (edit config file)
provider show               → config
provider set-api-key        → llm set-key
provider test               → llm test

auth login                  → (removed - out of scope)
auth logout                 → (removed - out of scope)
auth status                 → (removed - out of scope)
```

### Phase 3: LLM Integration Simplification

#### File Changes
- `src/lib.rs`:
  - Simplify `format_with_llm()` to take provider type directly
  - Remove provider validation complexity
  - Streamline API key retrieval
  - Remove OAuth token handling
  - Simplify error messages

#### Logic Flow
```
Old:
format_with_llm(transcript)
  → load config
  → get active provider
  → validate provider exists
  → get provider config
  → check OAuth token
  → fallback to API key
  → check env variables
  → call provider API

New:
format_with_llm(transcript, provider_type)
  → load config
  → get provider config (llm.openai, llm.local, etc.)
  → get API key from keychain if needed
  → call provider API
```

### Phase 4: User Experience Improvements

#### New Features
1. **Config Edit Command**
   - Open config file in $EDITOR or default editor
   - Validate config after editing
   - Show helpful error messages

2. **Config Init Wizard**
   - Interactive prompts for first-time setup
   - Detect Ollama availability
   - Offer to set up API keys
   - Generate complete config file

3. **Better Error Messages**
   - Clear, actionable error messages
   - Suggest next steps
   - Point to config file location

4. **Config Path Command**
   - Show where config file is located
   - Helpful for troubleshooting

---

## Migration Strategy

### Automatic Migration
When user runs any command with old config format:
1. Detect old format (has `providers` field or missing `llm.local`)
2. Create backup: `config.toml.backup.TIMESTAMP`
3. Migrate to new format:
   - If `active_provider` exists, use its config as default
   - If using old `llm` config, keep as `llm.local`
   - Preserve all existing settings
4. Save new config
5. Show migration message to user

### Manual Migration
Users can also manually edit their config file - it's just TOML.

---

## Work Plan

### Phase 1: Foundation (Day 1)
**Goal**: New config structure working

Tasks:
1. Create new config structs in `lib.rs`
2. Update `AppConfig::load()` to support new format
3. Update `AppConfig::save()` to write new format
4. Create `AppConfig::default()` with new defaults
5. Write config migration function
6. Test config loading/saving

**Deliverable**: Config system works with new structure

### Phase 2: CLI Refactoring (Day 1-2)
**Goal**: New command structure working

Tasks:
1. Update `Commands` enum in `main.rs`
2. Remove `ProviderCommands`, `ModelCommands`, `AuthCommands`
3. Simplify `ConfigCommands`
4. Add new `LlmCommands`
5. Update `Args` struct for `--llm <provider>`
6. Implement new command handlers
7. Remove old command handlers

**Deliverable**: New CLI commands work

### Phase 3: LLM Integration (Day 2)
**Goal**: LLM formatting works with new system

Tasks:
1. Simplify `format_with_llm()` function
2. Remove provider abstraction layer
3. Update `format_with_ollama()` to use new config
4. Update `format_with_openai()` to use new config
5. Update `format_with_anthropic()` to use new config
6. Update `format_with_custom()` to use new config
7. Remove OAuth-related code
8. Simplify credential management

**Deliverable**: LLM formatting works with all providers

### Phase 4: Polish & Testing (Day 2-3)
**Goal**: Production-ready release

Tasks:
1. Implement `config edit` command
2. Implement `config init` wizard
3. Implement `config path` command
4. Update all error messages
5. Test all command combinations
6. Test config migration
7. Update README.md
8. Update AGENTS.md
9. Test build and run

**Deliverable**: Fully working, polished tool

### Phase 5: Documentation (Day 3)
**Goal**: Users can understand and use the tool

Tasks:
1. Update README with new commands
2. Add configuration examples
3. Add troubleshooting guide
4. Update AGENTS.md with new structure
5. Create migration guide for existing users
6. Add example config files

**Deliverable**: Complete documentation

---

## Testing Checklist

### Configuration Tests
- [ ] Default config loads correctly
- [ ] Config file can be saved and reloaded
- [ ] Migration from old format works
- [ ] Invalid config shows helpful error
- [ ] Config edit opens editor
- [ ] Config init wizard works
- [ ] Config path shows correct location
- [ ] Config reset restores defaults

### CLI Tests
- [ ] Basic transcription works: `y2md <URL>`
- [ ] LLM with default provider: `y2md <URL> --llm`
- [ ] LLM with local: `y2md <URL> --llm local`
- [ ] LLM with OpenAI: `y2md <URL> --llm openai`
- [ ] LLM with Anthropic: `y2md <URL> --llm anthropic`
- [ ] LLM with custom: `y2md <URL> --llm custom`
- [ ] Config commands all work
- [ ] LLM commands all work

### LLM Integration Tests
- [ ] Local Ollama formatting works
- [ ] OpenAI formatting works
- [ ] Anthropic formatting works
- [ ] Custom endpoint works
- [ ] API key from keychain works
- [ ] Missing API key shows helpful error
- [ ] Invalid provider shows helpful error
- [ ] LLM timeout handled gracefully
- [ ] Fallback to basic formatting works

### Edge Cases
- [ ] No config file (first run)
- [ ] Corrupted config file
- [ ] Missing Ollama service
- [ ] Invalid API key
- [ ] Network errors
- [ ] Empty responses from LLM
- [ ] Very long transcripts

---

## Risk Assessment

### Low Risk
- Config structure changes (can migrate automatically)
- Command renames (old commands will show error with suggestion)
- Removing provider system (was confusing anyway)

### Medium Risk
- LLM integration changes (need thorough testing)
- Credential management (ensure API keys still work)
- Config migration (need to handle edge cases)

### Mitigation
- Create automatic backups before migration
- Extensive testing with real API keys
- Clear error messages with recovery steps
- Keep ability to manually edit config

---

## Success Criteria

1. **Simplicity**: New user can use tool in under 5 minutes
2. **Transparency**: Config file is self-documenting
3. **Flexibility**: Power users can still customize everything
4. **Reliability**: All existing functionality still works
5. **Documentation**: README covers all common use cases

---

## Rollback Plan

If critical issues arise:
1. Revert commits to pre-refactor state
2. Create hotfix release
3. Address issues in separate branch
4. Re-release when stable

Backup strategy:
- Git history preserves all old code
- Users' config files backed up during migration
- Old config format still loadable (migration code)

---

## Post-Launch

### Monitoring
- Watch for GitHub issues
- Monitor user feedback
- Track common error patterns

### Future Improvements
- Add more LLM providers (Groq, Together AI, etc.)
- Streaming LLM responses for long transcripts
- Better progress indicators
- Transcript caching
- Batch processing multiple URLs

---

## Approval Required

**Before proceeding, please confirm:**
1. ✅ Overall design direction
2. ✅ Config file structure
3. ✅ Command simplification
4. ✅ Migration strategy
5. ✅ Work plan phases

**Questions for review:**
1. Should we keep OAuth support or defer to v2?
   - **Recommendation**: Remove for simplicity, add later if needed
   
2. Should `--llm` flag auto-enable LLM or require provider?
   - **Recommendation**: Auto-enable with default provider from config
   
3. Should we keep advanced Whisper config?
   - **Recommendation**: Yes, but move to `[advanced]` section

---

## Next Steps

Upon approval:
1. Create feature branch: `refactor/simplify-config-cli`
2. Begin Phase 1: Foundation
3. Regular commits with clear messages
4. Test after each phase
5. Final review before merge
6. Update version to 0.2.0

---

**Document Version**: 1.0  
**Last Updated**: 2025-10-09  
**Author**: Assistant + User Collaboration
