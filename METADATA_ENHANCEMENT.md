# Metadata Enhancement Feature

## Overview

Enhanced the y2md output to include comprehensive metadata about the LLM formatting process. This allows users to track which LLM provider and model processed each transcript for future reference, auditing, and reproducibility.

## What Changed

### New Metadata Fields in Output

All generated markdown files now include three new fields in the YAML front matter:

1. **`formatted_by`**: Indicates the formatting method used
   - `"llm"` - Transcript was formatted using an LLM
   - `"standard"` - Transcript was formatted using the built-in formatter (either by choice or as fallback)

2. **`llm_provider`**: The LLM provider used (only present if `formatted_by: "llm"`)
   - `"local"` - Local Ollama instance
   - `"openai"` - OpenAI API
   - `"anthropic"` - Anthropic Claude API
   - `"custom"` - Custom OpenAI-compatible API

3. **`llm_model`**: The specific model name used (only present if `formatted_by: "llm"`)
   - Examples: `"claude-3-sonnet-20240229"`, `"gpt-4-turbo-preview"`, `"mistral-nemo:12b-instruct-2407-q5_0"`

### Example Output

**With LLM formatting:**
```yaml
---
title: "Understanding Rust Ownership"
channel: "Rust Programming"
url: "https://www.youtube.com/watch?v=abc123"
video_id: "abc123"
duration: "15:30"
source: "captions"
language: "en"
extracted_at: "2025-10-09T14:23:45.123Z"
formatted_by: "llm"
llm_provider: "anthropic"
llm_model: "claude-3-sonnet-20240229"
---
```

**With standard formatting:**
```yaml
---
title: "Understanding Rust Ownership"
channel: "Rust Programming"
url: "https://www.youtube.com/watch?v=abc123"
video_id: "abc123"
duration: "15:30"
source: "captions"
language: "en"
extracted_at: "2025-10-09T14:23:45.123Z"
formatted_by: "standard"
---
```

## Benefits

### 1. **Reproducibility**
Users can reproduce results by using the same LLM provider and model shown in the metadata.

### 2. **Quality Auditing**
Compare transcript quality across different providers and models to find the best fit for specific use cases.

### 3. **Organization**
Filter and organize transcripts based on processing method, provider, or model.

### 4. **Transparency**
Complete visibility into which tools processed each transcript, useful for:
- Documentation and record-keeping
- Compliance and auditing requirements
- Research and analysis

### 5. **Debugging**
When issues arise, quickly identify which provider/model was used to process a specific transcript.

## Implementation Details

### Code Changes

**File**: `src/lib.rs`
**Function**: `format_markdown()`

The function now:
1. Tracks the formatting method used (LLM or standard)
2. Records the actual provider and model if LLM formatting succeeds
3. Includes this information in the YAML front matter before the transcript content

### Backward Compatibility

- **Old transcripts**: Will not have the new fields (this is fine, they're optional)
- **New transcripts**: Will always have `formatted_by` field
- **LLM transcripts**: Will have all three new fields (`formatted_by`, `llm_provider`, `llm_model`)
- **Standard transcripts**: Will only have `formatted_by: "standard"`

### Testing

All existing tests continue to pass:
- ✅ 14 unit tests passed
- ✅ No compilation errors
- ✅ No clippy warnings (related to this change)

## Documentation Updates

The following files have been updated to document this feature:

1. **README.md**: 
   - Added detailed "Output Format" section explaining all metadata fields
   - Included use cases for the metadata

2. **AGENTS.md**: 
   - Added "Output Metadata" section documenting all fields
   - Updated LLM Integration section to mention metadata tracking

3. **IMPLEMENTATION_STATUS.md**: 
   - Added metadata enhancement to completed features
   - Updated success metrics

4. **CHANGELOG.md**: 
   - New file documenting this and all other changes
   - Follows Keep a Changelog format

## Future Enhancements

Possible future additions to metadata:
- `config_version`: Track configuration schema version
- `processing_time`: How long the processing took
- `transcript_source_detail`: Distinguish between auto-generated and manual captions
- `model_parameters`: Temperature, max_tokens, etc. used for LLM calls

## Usage

No changes to CLI usage required. The metadata is automatically included in all new transcripts:

```bash
# Standard formatting
y2md https://youtube.com/watch?v=VIDEO_ID

# LLM formatting (will include provider/model info)
y2md https://youtube.com/watch?v=VIDEO_ID --llm
y2md https://youtube.com/watch?v=VIDEO_ID --llm anthropic
```

## Conclusion

This enhancement provides complete transparency about the processing pipeline, enabling better organization, reproducibility, and quality management of transcripts.

---

**Feature Status**: ✅ Complete  
**Date Implemented**: 2025-10-09  
**Version**: Unreleased (will be in next version)
