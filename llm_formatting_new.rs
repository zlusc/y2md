pub async fn format_with_llm(transcript: &str, provider_override: Option<LlmProviderType>) -> Result<String, Y2mdError> {
    let config = AppConfig::load()?;
    let cred_manager = CredentialManager::new();

    let provider = provider_override.unwrap_or(config.llm.provider.clone());

    match provider {
        LlmProviderType::Local => {
            format_with_local(transcript, &config.llm.local).await
        }
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
        LlmProviderType::Custom => {
            let api_key = cred_manager.get_api_key(&LlmProviderType::Custom)?;
            format_with_custom(transcript, &config.llm.custom, api_key.as_deref()).await
        }
    }
}

async fn format_with_local(
    transcript: &str,
    llm_config: &LocalLlmConfig,
) -> Result<String, Y2mdError> {
    let client = reqwest::Client::new();
    
    let health_check = client.get(format!("{}/api/tags", llm_config.endpoint)).send().await;

    if health_check.is_err() {
        return Err(Y2mdError::Llm(format!(
            "Ollama service not available at {}. Make sure Ollama is running",
            llm_config.endpoint
        )));
    }

    let prompt = format!(
        "Transform this raw transcript into a polished, well-structured markdown document. 

**Formatting Guidelines:**
- **Structure**: Create logical sections with appropriate headings (## for main sections, ### for subsections)
- **Paragraphs**: Group related thoughts into coherent paragraphs (3-5 sentences each)
- **Readability**: Fix grammar, punctuation, and sentence structure while preserving meaning
- **Speaker Handling**: If multiple speakers are present, identify them clearly
- **Content Enhancement**: 
  - Remove excessive filler words (um, uh, like, you know)
  - Improve flow between sentences and paragraphs
  - Add emphasis with **bold** or *italic* where appropriate
  - Use bullet points for lists and key takeaways
  - Maintain the original speaker's tone and style

**Transcript:**

{}

**Formatted Markdown:**",
        transcript
    );

    let request_body = serde_json::json!({
        "model": llm_config.model,
        "prompt": prompt,
        "stream": false
    });

    let response = client
        .post(format!("{}/api/generate", llm_config.endpoint))
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                Y2mdError::Llm("LLM request timed out after 2 minutes".to_string())
            } else {
                Y2mdError::Llm(format!("Failed to connect to Ollama: {}", e))
            }
        })?;

    if !response.status().is_success() {
        return Err(Y2mdError::Llm(format!(
            "Ollama API returned error: {}",
            response.status()
        )));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| Y2mdError::Llm(format!("Failed to parse Ollama response: {}", e)))?;

    let formatted_text = response_json["response"]
        .as_str()
        .ok_or_else(|| Y2mdError::Llm("Invalid response format from Ollama".to_string()))?
        .trim()
        .to_string();

    if formatted_text.is_empty() {
        return Err(Y2mdError::Llm(
            "Ollama returned empty response".to_string(),
        ));
    }

    Ok(formatted_text)
}

async fn format_with_openai(
    transcript: &str,
    llm_config: &OpenAiConfig,
    api_key: &str,
) -> Result<String, Y2mdError> {
    let client = reqwest::Client::new();

    let prompt = format!(
        "Transform this raw transcript into a polished, well-structured markdown document. 

**Formatting Guidelines:**
- **Structure**: Create logical sections with appropriate headings (## for main sections, ### for subsections)
- **Paragraphs**: Group related thoughts into coherent paragraphs (3-5 sentences each)
- **Readability**: Fix grammar, punctuation, and sentence structure while preserving meaning
- **Speaker Handling**: If multiple speakers are present, identify them clearly
- **Content Enhancement**: 
  - Remove excessive filler words (um, uh, like, you know)
  - Improve flow between sentences and paragraphs
  - Add emphasis with **bold** or *italic* where appropriate
  - Use bullet points for lists and key takeaways
  - Maintain the original speaker's tone and style

**Transcript:**

{}",
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
                Y2mdError::Llm(format!("Failed to connect to OpenAI API: {}", e))
            }
        })?;

    if !response.status().is_success() {
        return Err(Y2mdError::Llm(format!(
            "OpenAI API returned error: {}",
            response.status()
        )));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| Y2mdError::Llm(format!("Failed to parse OpenAI response: {}", e)))?;

    let formatted_text = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| Y2mdError::Llm("Invalid response format from OpenAI".to_string()))?
        .trim()
        .to_string();

    if formatted_text.is_empty() {
        return Err(Y2mdError::Llm(
            "OpenAI returned empty response".to_string(),
        ));
    }

    Ok(formatted_text)
}

async fn format_with_anthropic(
    transcript: &str,
    llm_config: &AnthropicConfig,
    api_key: &str,
) -> Result<String, Y2mdError> {
    let client = reqwest::Client::new();

    let prompt = format!(
        "Transform this raw transcript into a polished, well-structured markdown document. 

**Formatting Guidelines:**
- **Structure**: Create logical sections with appropriate headings (## for main sections, ### for subsections)
- **Paragraphs**: Group related thoughts into coherent paragraphs (3-5 sentences each)
- **Readability**: Fix grammar, punctuation, and sentence structure while preserving meaning
- **Speaker Handling**: If multiple speakers are present, identify them clearly
- **Content Enhancement**: 
  - Remove excessive filler words (um, uh, like, you know)
  - Improve flow between sentences and paragraphs
  - Add emphasis with **bold** or *italic* where appropriate
  - Use bullet points for lists and key takeaways
  - Maintain the original speaker's tone and style

**Transcript:**

{}",
        transcript
    );

    let request_body = serde_json::json!({
        "model": llm_config.model,
        "max_tokens": 4096,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let response = client
        .post(format!("{}/messages", llm_config.endpoint))
        .header("anthropic-version", "2023-06-01")
        .header("x-api-key", api_key)
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                Y2mdError::Llm("LLM request timed out after 2 minutes".to_string())
            } else {
                Y2mdError::Llm(format!("Failed to connect to Anthropic API: {}", e))
            }
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(Y2mdError::Llm(format!(
            "Anthropic API returned error {}: {}",
            status, error_text
        )));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| Y2mdError::Llm(format!("Failed to parse Anthropic response: {}", e)))?;

    let formatted_text = response_json["content"][0]["text"]
        .as_str()
        .ok_or_else(|| Y2mdError::Llm("Invalid response format from Anthropic".to_string()))?
        .trim()
        .to_string();

    if formatted_text.is_empty() {
        return Err(Y2mdError::Llm(
            "Anthropic returned empty response".to_string(),
        ));
    }

    Ok(formatted_text)
}

async fn format_with_custom(
    transcript: &str,
    llm_config: &CustomLlmConfig,
    api_key: Option<&str>,
) -> Result<String, Y2mdError> {
    if llm_config.endpoint.is_empty() {
        return Err(Y2mdError::Llm(
            "Custom LLM endpoint not configured. Please set it in your config file.".to_string(),
        ));
    }

    let client = reqwest::Client::new();

    let prompt = format!(
        "Transform this raw transcript into a polished, well-structured markdown document. 

**Formatting Guidelines:**
- **Structure**: Create logical sections with appropriate headings (## for main sections, ### for subsections)
- **Paragraphs**: Group related thoughts into coherent paragraphs (3-5 sentences each)
- **Readability**: Fix grammar, punctuation, and sentence structure while preserving meaning
- **Speaker Handling**: If multiple speakers are present, identify them clearly
- **Content Enhancement**: 
  - Remove excessive filler words (um, uh, like, you know)
  - Improve flow between sentences and paragraphs
  - Add emphasis with **bold** or *italic* where appropriate
  - Use bullet points for lists and key takeaways
  - Maintain the original speaker's tone and style

**Transcript:**

{}",
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

    let mut request_builder = client
        .post(format!("{}/chat/completions", llm_config.endpoint))
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(120));

    if let Some(key) = api_key {
        request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
    }

    let response = request_builder.send().await.map_err(|e| {
        if e.is_timeout() {
            Y2mdError::Llm("LLM request timed out after 2 minutes".to_string())
        } else {
            Y2mdError::Llm(format!("Failed to connect to custom LLM API: {}", e))
        }
    })?;

    if !response.status().is_success() {
        return Err(Y2mdError::Llm(format!(
            "Custom LLM API returned error: {}",
            response.status()
        )));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| Y2mdError::Llm(format!("Failed to parse custom LLM response: {}", e)))?;

    let formatted_text = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| Y2mdError::Llm("Invalid response format from custom LLM".to_string()))?
        .trim()
        .to_string();

    if formatted_text.is_empty() {
        return Err(Y2mdError::Llm(
            "Custom LLM returned empty response".to_string(),
        ));
    }

    Ok(formatted_text)
}
