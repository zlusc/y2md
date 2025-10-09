# DeepSeek LLM Integration Plan

## Overview
Add DeepSeek as a new LLM provider option for y2md, allowing users to use DeepSeek's API for transcript formatting.

## DeepSeek API Information
- **API Endpoint**: `https://api.deepseek.com/v1`
- **API Format**: OpenAI-compatible
- **Authentication**: Bearer token (API key)
- **Default Model**: `deepseek-chat` or `deepseek-coder`
- **Pricing**: Competitive pricing model
- **Documentation**: https://platform.deepseek.com/docs

## Implementation Plan

### Phase 1: Core Configuration (src/lib.rs)

#### 1.1 Update LlmProviderType Enum
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProviderType {
    Local,
    OpenAI,
    Anthropic,
    DeepSeek,      // NEW
    Custom,
}
```

**Files to modify:**
- Line ~38: Add `DeepSeek` variant to enum
- Line ~54: Add display implementation for DeepSeek
- Line ~67: Add FromStr parsing for "deepseek"

#### 1.2 Create DeepSeekConfig Struct
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekConfig {
    pub endpoint: String,
    pub model: String,
}

impl Default for DeepSeekConfig {
    fn default() -> Self {
        DeepSeekConfig {
            endpoint: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
        }
    }
}
```

**Location:** After AnthropicConfig (~line 120)

#### 1.3 Update LlmSettings Struct
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSettings {
    pub enabled: bool,
    pub provider: LlmProviderType,
    pub local: LocalLlmConfig,
    pub openai: OpenAiConfig,
    pub anthropic: AnthropicConfig,
    pub deepseek: DeepSeekConfig,    // NEW
    pub custom: CustomLlmConfig,
}

impl Default for LlmSettings {
    fn default() -> Self {
        LlmSettings {
            enabled: false,
            provider: LlmProviderType::Local,
            local: LocalLlmConfig::default(),
            openai: OpenAiConfig::default(),
            anthropic: AnthropicConfig::default(),
            deepseek: DeepSeekConfig::default(),    // NEW
            custom: CustomLlmConfig::default(),
        }
    }
}
```

**Location:** ~Line 137-157

#### 1.4 Add DeepSeek API Key Support
Update `CredentialManager::get_api_key()` to support environment variable:
- Check `Y2MD_DEEPSEEK_API_KEY` environment variable
- Check `DEEPSEEK_API_KEY` environment variable (fallback)

**Location:** ~Line 265-270

#### 1.5 Create format_with_deepseek Function
```rust
async fn format_with_deepseek(
    transcript: &str,
    llm_config: &DeepSeekConfig,
    api_key: &str,
) -> Result<String, Y2mdError> {
    let client = reqwest::Client::new();

    let prompt = format!(
        "Please format the following transcript into well-structured markdown. 
        Keep the original content but improve readability by:
        - Organizing into logical paragraphs
        - Fixing any grammar or punctuation issues
        - Removing filler words if appropriate
        - Maintaining the original meaning and tone
        
        Transcript:\n\n{}",
        transcript
    );

    let request_body = serde_json::json!({
        "model": llm_config.model,
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant that formats transcripts into well-structured markdown."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": 0.1
    });

    let response = client
        .post(format!("{}/chat/completions", llm_config.endpoint))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                Y2mdError::Llm("LLM request timed out after 2 minutes".to_string())
            } else {
                Y2mdError::Llm(format!("Failed to connect to DeepSeek API: {}", e))
            }
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(Y2mdError::Llm(format!(
            "DeepSeek API returned error {}: {}",
            status, error_text
        )));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| Y2mdError::Llm(format!("Failed to parse DeepSeek response: {}", e)))?;

    let formatted_text = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| Y2mdError::Llm("Invalid response format from DeepSeek".to_string()))?
        .trim()
        .to_string();

    if formatted_text.is_empty() {
        return Err(Y2mdError::Llm(
            "DeepSeek returned empty response".to_string(),
        ));
    }

    Ok(formatted_text)
}
```

**Location:** After `format_with_anthropic()` (~line 1418)

#### 1.6 Update format_with_llm Function
Add DeepSeek case to the match statement:

```rust
match provider {
    LlmProviderType::Local => format_with_local(transcript, &config.llm.local).await,
    LlmProviderType::OpenAI => {
        let api_key = cred_manager.get_api_key(&LlmProviderType::OpenAI)?
            .ok_or_else(|| Y2mdError::Llm("OpenAI API key not set. Use: y2md llm set-key openai".to_string()))?;
        format_with_openai(transcript, &config.llm.openai, &api_key).await
    }
    LlmProviderType::Anthropic => {
        let api_key = cred_manager.get_api_key(&LlmProviderType::Anthropic)?
            .ok_or_else(|| Y2mdError::Llm("Anthropic API key not set. Use: y2md llm set-key anthropic".to_string()))?;
        format_with_anthropic(transcript, &config.llm.anthropic, &api_key).await
    }
    LlmProviderType::DeepSeek => {    // NEW
        let api_key = cred_manager.get_api_key(&LlmProviderType::DeepSeek)?
            .ok_or_else(|| Y2mdError::Llm("DeepSeek API key not set. Use: y2md llm set-key deepseek".to_string()))?;
        format_with_deepseek(transcript, &config.llm.deepseek, &api_key).await
    }
    LlmProviderType::Custom => {
        let api_key = cred_manager.get_api_key(&LlmProviderType::Custom)?;
        format_with_custom(transcript, &config.llm.custom, api_key.as_deref()).await
    }
}
```

**Location:** ~Line 1238-1264

#### 1.7 Update format_markdown Function
Add DeepSeek case when determining the model name:

```rust
actual_llm_model = Some(match provider {
    LlmProviderType::Local => cfg.llm.local.model.clone(),
    LlmProviderType::OpenAI => cfg.llm.openai.model.clone(),
    LlmProviderType::Anthropic => cfg.llm.anthropic.model.clone(),
    LlmProviderType::DeepSeek => cfg.llm.deepseek.model.clone(),    // NEW
    LlmProviderType::Custom => cfg.llm.custom.model.clone(),
});
```

**Location:** ~Line 960-970

### Phase 2: CLI Updates (src/main.rs)

#### 2.1 Update CLI Help Text
Add DeepSeek to the provider list in help messages.

**Locations:**
- Main command help text
- `--llm` flag documentation
- Examples section

### Phase 3: Configuration Updates

#### 3.1 Update config.example.toml
Add DeepSeek configuration section:

```toml
# DeepSeek - Fast, affordable (requires API key)
[llm.deepseek]
endpoint = "https://api.deepseek.com/v1"
model = "deepseek-chat"
```

**Location:** After `[llm.anthropic]` section

### Phase 4: Documentation Updates

#### 4.1 README.md
Add DeepSeek to:
- LLM providers list
- Setup instructions
- Examples
- Supported providers table

**New section:**
```markdown
### Option 4: DeepSeek

**Pros**: Fast, affordable, competitive quality  
**Cons**: Requires API key

```bash
# 1. Set API key
y2md llm set-key deepseek
# Enter your API key when prompted

# 2. Use it!
y2md <URL> --llm deepseek

# Alternative: Use environment variable
export Y2MD_DEEPSEEK_API_KEY="sk-..."
```
```

#### 4.2 AGENTS.md
Add DeepSeek to supported providers list:

```markdown
5. **DeepSeek**
   - Supports deepseek-chat, deepseek-coder models
   - Requires API key (set via `y2md llm set-key deepseek` or `DEEPSEEK_API_KEY`)
   - Configure endpoint and model in `[llm.deepseek]`
```

#### 4.3 CHANGELOG.md
Add to [Unreleased] section:

```markdown
### Added
- **DeepSeek LLM Provider**: New provider option for transcript formatting
  - Fast and affordable alternative to OpenAI/Anthropic
  - OpenAI-compatible API
  - Configure via `[llm.deepseek]` in config.toml
  - Set API key via `y2md llm set-key deepseek`
```

### Phase 5: Testing

#### 5.1 Unit Tests
Add tests for DeepSeek provider:
- LlmProviderType parsing
- Display formatting
- Config serialization/deserialization

#### 5.2 Integration Tests
Test with actual DeepSeek API (if API key available):
- Basic formatting request
- Error handling
- Timeout behavior
- API key validation

#### 5.3 Manual Testing
```bash
# Set up DeepSeek
y2md config edit  # Add deepseek config
y2md llm set-key deepseek

# Test with a real video
y2md <YOUTUBE_URL> --llm deepseek

# Verify output metadata includes:
# - formatted_by: "llm"
# - llm_provider: "deepseek"
# - llm_model: "deepseek-chat"
```

## Implementation Checklist

### Code Changes
- [ ] Add `DeepSeek` to `LlmProviderType` enum
- [ ] Add display implementation for DeepSeek
- [ ] Add FromStr parsing for "deepseek"
- [ ] Create `DeepSeekConfig` struct with Default
- [ ] Update `LlmSettings` to include `deepseek: DeepSeekConfig`
- [ ] Update `LlmSettings::default()` to include DeepSeek
- [ ] Update `CredentialManager` for DeepSeek API key env vars
- [ ] Create `format_with_deepseek()` function
- [ ] Update `format_with_llm()` match statement
- [ ] Update `format_markdown()` model name matching

### Configuration
- [ ] Update `config.example.toml` with DeepSeek section

### Documentation
- [ ] Update README.md with DeepSeek setup instructions
- [ ] Update AGENTS.md with DeepSeek provider info
- [ ] Update CHANGELOG.md with new feature
- [ ] Create/update examples showing DeepSeek usage

### Testing
- [ ] Run `cargo test` - ensure all tests pass
- [ ] Run `cargo clippy` - fix any warnings
- [ ] Run `cargo fmt` - format code
- [ ] Manual test with real YouTube URL
- [ ] Verify metadata in output markdown

### Final Steps
- [ ] Build successfully: `cargo build --release`
- [ ] Test all providers still work (local, openai, anthropic, custom)
- [ ] Update version in Cargo.toml if needed
- [ ] Commit changes with descriptive message

## Estimated Implementation Time

- **Phase 1 (Core)**: 30-45 minutes
- **Phase 2 (CLI)**: 5-10 minutes
- **Phase 3 (Config)**: 5 minutes
- **Phase 4 (Docs)**: 15-20 minutes
- **Phase 5 (Testing)**: 15-20 minutes

**Total**: ~70-100 minutes

## Benefits of Adding DeepSeek

1. **Cost-effective**: More affordable than OpenAI/Anthropic
2. **Fast**: Quick response times
3. **OpenAI-compatible**: Easy integration using existing patterns
4. **Quality**: Competitive output quality
5. **Options**: More choice for users based on budget/needs

## Potential Issues & Solutions

### Issue 1: API Format Differences
**Solution**: Since DeepSeek is OpenAI-compatible, we can use the same request/response format as OpenAI.

### Issue 2: Rate Limiting
**Solution**: Use the same 2-minute timeout and error handling as other providers.

### Issue 3: Model Availability
**Solution**: Default to `deepseek-chat` but allow users to configure other models in config.toml.

### Issue 4: API Key Management
**Solution**: Use existing keychain infrastructure, same as OpenAI/Anthropic.

## Notes

- DeepSeek API is OpenAI-compatible, so implementation is straightforward
- Can reuse most of the OpenAI implementation pattern
- Consider adding multiple DeepSeek models in documentation (chat vs coder)
- Environment variable support for both `Y2MD_DEEPSEEK_API_KEY` and `DEEPSEEK_API_KEY`

## Future Enhancements

After initial implementation:
1. Add model-specific configuration (e.g., temperature, max_tokens)
2. Add DeepSeek Coder model support with different prompts
3. Add usage/cost tracking for DeepSeek API calls
4. Compare quality benchmarks with other providers

---

**Status**: 📋 Planning Complete - Ready for Implementation  
**Priority**: Medium  
**Complexity**: Low (reuses existing patterns)
