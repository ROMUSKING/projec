//! LLM Gateway for provider abstraction.
//!
//! This module provides a unified interface for interacting with different
//! LLM providers (OpenAI, Anthropic, Ollama, etc.).

use common::{async_trait, Error, Result};
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Log a prompt being sent to the model
pub fn log_prompt(provider: &str, model: &str, prompt: &str) {
    eprintln!("\n=== LLM REQUEST [{} - {}] ===", provider, model);
    eprintln!("{}", prompt);
    eprintln!("=== END REQUEST ===\n");
}

/// Log a response received from the model
pub fn log_response(provider: &str, model: &str, response: &str) {
    eprintln!("\n=== LLM RESPONSE [{} - {}] ===", provider, model);
    eprintln!("{}", response);
    eprintln!("=== END RESPONSE ===\n");
}

/// LLM Gateway trait for provider abstraction
#[async_trait]
pub trait LlmGateway: Send + Sync {
    /// Initialize the gateway
    async fn initialize(&mut self) -> Result<()>;

    /// Shutdown the gateway
    async fn shutdown(&mut self) -> Result<()>;

    /// Generate a completion
    async fn generate(&self, prompt: &str) -> Result<super::GenerationResult>;

    /// Generate a streaming completion
    async fn generate_stream(&self, prompt: &str) -> Result<StreamResult>;

    /// Get available models
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;

    /// Validate the connection
    async fn health_check(&self) -> Result<bool>;
}

/// Streaming result type
pub type StreamResult = Pin<Box<dyn futures::Stream<Item = Result<StreamChunk>> + Send>>;

/// Stream chunk from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub content: String,
    pub is_finished: bool,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_window: u32,
    pub capabilities: Vec<ModelCapability>,
}

/// Model capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    Chat,
    Completion,
    FunctionCalling,
    Vision,
    Streaming,
}

/// OpenAI gateway implementation
pub struct OpenAiGateway {
    api_key: String,
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAiGateway {
    pub fn new(api_key: String, model: String, client: reqwest::Client) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com".to_string(),
            model,
            client,
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

#[async_trait]
impl LlmGateway for OpenAiGateway {
    async fn initialize(&mut self) -> Result<()> {
        // TODO: Validate API key and connection
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        // TODO: Cleanup resources
        Ok(())
    }

    async fn generate(&self, prompt: &str) -> Result<super::GenerationResult> {
        log_prompt("OpenAI", &self.model, prompt);

        let request = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.7
        });

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::ExternalService(format!("OpenAI request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::ExternalService(format!("OpenAI error: {}", response.status())));
        }

        let body: serde_json::Value = response.json().await
            .map_err(|e| Error::ExternalService(format!("Failed to parse OpenAI response: {}", e)))?;

        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::ExternalService("Invalid OpenAI response format".to_string()))?
            .to_string();

        let tokens_used = body["usage"]["total_tokens"]
            .as_u64()
            .unwrap_or(0) as u32;

        let finish_reason = body["choices"][0]["finish_reason"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        log_response("OpenAI", &self.model, &content);

        Ok(super::GenerationResult {
            content,
            tokens_used,
            model: self.model.clone(),
            finish_reason,
        })
    }

    async fn generate_stream(&self, _prompt: &str) -> Result<StreamResult> {
        // TODO: Implement OpenAI streaming (requires handling SSE)
        Err(Error::Internal("OpenAI streaming not yet implemented".to_string()))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let response = self.client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| Error::ExternalService(format!("OpenAI request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::ExternalService(format!("OpenAI error: {}", response.status())));
        }

        let body: serde_json::Value = response.json().await
            .map_err(|e| Error::ExternalService(format!("Failed to parse OpenAI models: {}", e)))?;

        let models = body["data"]
            .as_array()
            .ok_or_else(|| Error::ExternalService("Invalid OpenAI models response".to_string()))?
            .iter()
            .map(|m| ModelInfo {
                id: m["id"].as_str().unwrap_or("unknown").to_string(),
                name: m["id"].as_str().unwrap_or("unknown").to_string(),
                context_window: 4096, // Placeholder, as OpenAI API doesn't always return this
                capabilities: vec![ModelCapability::Chat],
            })
            .collect();

        Ok(models)
    }

    async fn health_check(&self) -> Result<bool> {
        // TODO: Check OpenAI API health
        Ok(true)
    }
}

/// Anthropic gateway implementation
pub struct AnthropicGateway {
    api_key: String,
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl AnthropicGateway {
    pub fn new(api_key: String, model: String, client: reqwest::Client) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
            model,
            client,
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

#[async_trait]
impl LlmGateway for AnthropicGateway {
    async fn initialize(&mut self) -> Result<()> {
        // TODO: Validate API key and connection
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        // TODO: Cleanup resources
        Ok(())
    }

    async fn generate(&self, _prompt: &str) -> Result<super::GenerationResult> {
        // TODO: Implement Anthropic completion
        Err(Error::Internal("Not implemented".to_string()))
    }

    async fn generate_stream(&self, prompt: &str) -> Result<StreamResult> {
        log_prompt("Anthropic", &self.model, prompt);

        let request = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "stream": true,
            "max_tokens": 4096
        });

        let response = self.client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::ExternalService(format!("Anthropic request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::ExternalService(format!("Anthropic error: {}", error_text)));
        }

        let stream = response.bytes_stream();
        Ok(process_anthropic_stream(stream))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // TODO: Fetch available models from Anthropic
        Ok(vec![])
    }

    async fn health_check(&self) -> Result<bool> {
        // TODO: Check Anthropic API health
        Ok(true)
    }
}

/// Ollama gateway for local models
pub struct OllamaGateway {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaGateway {
    pub fn new(model: String, client: reqwest::Client) -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model,
            client,
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

#[async_trait]
impl LlmGateway for OllamaGateway {
    async fn initialize(&mut self) -> Result<()> {
        // TODO: Validate connection to Ollama
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        // TODO: Cleanup resources
        Ok(())
    }

    async fn generate(&self, _prompt: &str) -> Result<super::GenerationResult> {
        // TODO: Implement Ollama completion
        Err(Error::Internal("Not implemented".to_string()))
    }

    async fn generate_stream(&self, _prompt: &str) -> Result<StreamResult> {
        // TODO: Implement Ollama streaming
        Err(Error::Internal("Not implemented".to_string()))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // TODO: Fetch available models from Ollama
        Ok(vec![])
    }

    async fn health_check(&self) -> Result<bool> {
        // TODO: Check Ollama health
        Ok(true)
    }
}

/// OpenRouter gateway implementation
pub struct OpenRouterGateway {
    api_key: String,
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl OpenRouterGateway {
    pub fn new(api_key: String, model: String, client: reqwest::Client) -> Self {
        Self {
            api_key,
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model,
            client,
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

#[async_trait]
impl LlmGateway for OpenRouterGateway {
    async fn initialize(&mut self) -> Result<()> {
        // TODO: Validate API key and connection
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        // TODO: Cleanup resources
        Ok(())
    }

    async fn generate(&self, prompt: &str) -> Result<super::GenerationResult> {
        log_prompt("OpenRouter", &self.model, prompt);

        let request = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.7
        });

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://your-app.com")  // Required by OpenRouter
            .header("X-Title", "Your App Name")  // Required by OpenRouter
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::ExternalService(format!("OpenRouter request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::ExternalService(format!("OpenRouter error: {}", response.status())));
        }

        let body: serde_json::Value = response.json().await
            .map_err(|e| Error::ExternalService(format!("Failed to parse OpenRouter response: {}", e)))?;

        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::ExternalService("Invalid OpenRouter response format".to_string()))?
            .to_string();

        let tokens_used = body["usage"]["total_tokens"]
            .as_u64()
            .unwrap_or(0) as u32;

        let finish_reason = body["choices"][0]["finish_reason"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        log_response("OpenRouter", &self.model, &content);

        Ok(super::GenerationResult {
            content,
            tokens_used,
            model: self.model.clone(),
            finish_reason,
        })
    }

    async fn generate_stream(&self, _prompt: &str) -> Result<StreamResult> {
        // TODO: Implement OpenRouter streaming (requires handling SSE)
        Err(Error::Internal("OpenRouter streaming not yet implemented".to_string()))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let response = self.client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| Error::ExternalService(format!("OpenRouter request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::ExternalService(format!("OpenRouter error: {}", response.status())));
        }

        let body: serde_json::Value = response.json().await
            .map_err(|e| Error::ExternalService(format!("Failed to parse OpenRouter models: {}", e)))?;

        let models = body["data"]
            .as_array()
            .ok_or_else(|| Error::ExternalService("Invalid OpenRouter models response".to_string()))?
            .iter()
            .map(|m| ModelInfo {
                id: m["id"].as_str().unwrap_or("unknown").to_string(),
                name: m["name"].as_str().unwrap_or("unknown").to_string(),
                context_window: 4096, // Placeholder
                capabilities: vec![ModelCapability::Chat],
            })
            .collect();

        Ok(models)
    }

    async fn health_check(&self) -> Result<bool> {
        // TODO: Check OpenRouter API health
        Ok(true)
    }
}

/// Arcee gateway implementation
pub struct ArceeGateway {
    api_key: String,
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl ArceeGateway {
    pub fn new(api_key: String, model: String, client: reqwest::Client) -> Self {
        Self {
            api_key,
            base_url: "https://api.arcee.ai/v1".to_string(),
            model,
            client,
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

#[async_trait]
impl LlmGateway for ArceeGateway {
    async fn initialize(&mut self) -> Result<()> {
        // TODO: Validate API key and connection
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        // TODO: Cleanup resources
        Ok(())
    }

    async fn generate(&self, prompt: &str) -> Result<super::GenerationResult> {
        log_prompt("Arcee", &self.model, prompt);

        let request = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.7
        });

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::ExternalService(format!("Arcee request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::ExternalService(format!("Arcee error: {}", response.status())));
        }

        let body: serde_json::Value = response.json().await
            .map_err(|e| Error::ExternalService(format!("Failed to parse Arcee response: {}", e)))?;

        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::ExternalService("Invalid Arcee response format".to_string()))?
            .to_string();

        let tokens_used = body["usage"]["total_tokens"]
            .as_u64()
            .unwrap_or(0) as u32;

        let finish_reason = body["choices"][0]["finish_reason"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        log_response("Arcee", &self.model, &content);

        Ok(super::GenerationResult {
            content,
            tokens_used,
            model: self.model.clone(),
            finish_reason,
        })
    }

    async fn generate_stream(&self, _prompt: &str) -> Result<StreamResult> {
        // TODO: Implement Arcee streaming (requires handling SSE)
        Err(Error::Internal("Arcee streaming not yet implemented".to_string()))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let response = self.client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| Error::ExternalService(format!("Arcee request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::ExternalService(format!("Arcee error: {}", response.status())));
        }

        let body: serde_json::Value = response.json().await
            .map_err(|e| Error::ExternalService(format!("Failed to parse Arcee models: {}", e)))?;

        let models = body["data"]
            .as_array()
            .ok_or_else(|| Error::ExternalService("Invalid Arcee models response".to_string()))?
            .iter()
            .map(|m| ModelInfo {
                id: m["id"].as_str().unwrap_or("unknown").to_string(),
                name: m["name"].as_str().unwrap_or("unknown").to_string(),
                context_window: 4096, // Placeholder
                capabilities: vec![ModelCapability::Chat],
            })
            .collect();

        Ok(models)
    }

    async fn health_check(&self) -> Result<bool> {
        // TODO: Check Arcee API health
        Ok(true)
    }
}

/// Mock gateway for testing and demonstration
pub struct MockGateway;

impl MockGateway {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LlmGateway for MockGateway {
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    async fn generate(&self, prompt: &str) -> Result<super::GenerationResult> {
        log_prompt("Mock", "mock-model", prompt);

        // Simple heuristic response generation for testing
        let content = if prompt.contains("plan") {
            "1. Analyze the codebase\n2. Identify issues\n3. Apply fixes".to_string()
        } else if prompt.contains("code") {
            "fn example() { println!(\"Hello from Mock AI\"); }".to_string()
        } else {
            format!("Mock response to: {}", prompt)
        };

        log_response("Mock", "mock-model", &content);

        Ok(super::GenerationResult {
            content,
            tokens_used: 10,
            model: "mock-model".to_string(),
            finish_reason: "stop".to_string(),
        })
    }

    async fn generate_stream(&self, prompt: &str) -> Result<StreamResult> {
        let content = format!("Mock stream response to: {}", prompt);
        let stream = futures::stream::iter(vec![
            Ok(StreamChunk { content: content, is_finished: true })
        ]);
        Ok(Box::pin(stream))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(vec![ModelInfo {
            id: "mock-model".to_string(),
            name: "Mock Model".to_string(),
            context_window: 4096,
            capabilities: vec![ModelCapability::Chat, ModelCapability::Completion],
        }])
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

/// Gateway factory for creating appropriate gateway instances
pub struct GatewayFactory {
    client: reqwest::Client,
}

impl GatewayFactory {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Create a gateway based on provider configuration
    pub fn create(
        &self,
        provider: &str,
        api_key: Option<String>,
        model: String,
    ) -> Result<Box<dyn LlmGateway>> {
        match provider {
            "openai" => {
                let key = api_key.ok_or_else(|| Error::Config("OpenAI API key required".to_string()))?;
                Ok(Box::new(OpenAiGateway::new(key, model, self.client.clone())))
            }
            "anthropic" => {
                let key = api_key.ok_or_else(|| Error::Config("Anthropic API key required".to_string()))?;
                Ok(Box::new(AnthropicGateway::new(key, model, self.client.clone())))
            }
            "ollama" => {
                Ok(Box::new(OllamaGateway::new(model, self.client.clone())))
            }
            "openrouter" => {
                let key = api_key.ok_or_else(|| Error::Config("OpenRouter API key required".to_string()))?;
                Ok(Box::new(OpenRouterGateway::new(key, model, self.client.clone())))
            }
            "arcee" => {
                let key = api_key.ok_or_else(|| Error::Config("Arcee API key required".to_string()))?;
                Ok(Box::new(ArceeGateway::new(key, model, self.client.clone())))
            }
            "mock" => {
                Ok(Box::new(MockGateway::new()))
            }
            _ => Err(Error::Config(format!("Unknown provider: {}", provider))),
        }
    }
}

impl Default for GatewayFactory {
    fn default() -> Self {
        Self::new()
    }
}

// --- Anthropic Streaming Helpers ---

use futures::StreamExt;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum AnthropicStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart,
    #[serde(rename = "content_block_start")]
    ContentBlockStart,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        delta: AnthropicDelta,
    },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop,
    #[serde(rename = "message_delta")]
    MessageDelta {
        usage: Option<AnthropicUsage>,
    },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "error")]
    Error {
        error: AnthropicError,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum AnthropicDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    #[allow(dead_code)]
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct AnthropicError {
    #[allow(dead_code)]
    message: String,
}

fn process_anthropic_stream<S>(stream: S) -> StreamResult
where
    S: futures::Stream<Item = reqwest::Result<bytes::Bytes>> + Send + 'static,
{
    // Map reqwest errors to common::Error
    let stream = stream.map(|res| res.map_err(|e| Error::ExternalService(format!("Stream error: {}", e))));

    // Box the stream to erase the type and make it easier to use in try_unfold closure
    let stream: Pin<Box<dyn futures::Stream<Item = Result<bytes::Bytes>> + Send>> = Box::pin(stream);

    let stream = futures::stream::try_unfold(
        (stream, Vec::new()),
        |(mut stream, mut buffer): (Pin<Box<dyn futures::Stream<Item = Result<bytes::Bytes>> + Send>>, Vec<u8>)| async move {
            loop {
                // Check if we have a full event in buffer (ended by \n\n)
                let mut found_event = None;
                for i in 0..buffer.len().saturating_sub(1) {
                    if buffer[i] == b'\n' && buffer[i+1] == b'\n' {
                        found_event = Some(i);
                        break;
                    }
                }

                if let Some(pos) = found_event {
                    let event_bytes: Vec<u8> = buffer.drain(..pos + 2).collect();
                    match parse_sse_event(&event_bytes) {
                        Ok(Some(chunk)) => {
                            return Ok(Some((chunk, (stream, buffer))));
                        },
                        Ok(None) => {
                            continue;
                        },
                        Err(e) => {
                            tracing::warn!("Failed to parse Anthropic SSE event: {}", e);
                            continue;
                        }
                    }
                }

                match stream.next().await {
                    Some(Ok(bytes)) => {
                        buffer.extend_from_slice(&bytes);
                    }
                    Some(Err(e)) => return Err(e),
                    None => {
                        return Ok(None);
                    }
                }
            }
        }
    );

    Box::pin(stream)
}

fn parse_sse_event(bytes: &[u8]) -> Result<Option<StreamChunk>> {
    let s = std::str::from_utf8(bytes)
        .map_err(|e| Error::Validation(format!("Invalid UTF-8 in SSE: {}", e)))?;

    let mut data_line = None;

    for line in s.lines() {
        if line.starts_with("data: ") {
            data_line = Some(line.trim_start_matches("data: "));
            break;
        }
    }

    if let Some(data) = data_line {
        // [DONE] is sometimes used in SSE but Anthropic uses JSON
        if data == "[DONE]" {
            return Ok(Some(StreamChunk {
                content: String::new(),
                is_finished: true,
            }));
        }

        let event: AnthropicStreamEvent = serde_json::from_str(data)
            .map_err(|e| Error::Serialization(e))?;

        match event {
            AnthropicStreamEvent::ContentBlockDelta { delta } => {
                if let AnthropicDelta::TextDelta { text } = delta {
                    return Ok(Some(StreamChunk {
                        content: text,
                        is_finished: false,
                    }));
                }
            }
            AnthropicStreamEvent::MessageStop => {
                return Ok(Some(StreamChunk {
                    content: String::new(),
                    is_finished: true,
                }));
            }
            AnthropicStreamEvent::Error { error } => {
                return Err(Error::ExternalService(format!("Anthropic stream error: {}", error.message)));
            }
            _ => {}
        }
    }

    Ok(None)
}
#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;
    use bytes::Bytes;

    #[tokio::test]
    async fn test_process_anthropic_stream_success() {
        let event1 = "event: message_start\ndata: {\"type\": \"message_start\", \"message\": {}}\n\n";
        let event2 = "event: content_block_delta\ndata: {\"type\": \"content_block_delta\", \"index\": 0, \"delta\": {\"type\": \"text_delta\", \"text\": \"Hello\"}}\n\n";
        let event3 = "event: content_block_delta\ndata: {\"type\": \"content_block_delta\", \"index\": 0, \"delta\": {\"type\": \"text_delta\", \"text\": \" World\"}}\n\n";
        let event4 = "event: message_stop\ndata: {\"type\": \"message_stop\"}\n\n";

        let stream_data: Vec<reqwest::Result<Bytes>> = vec![
            Ok(Bytes::from(event1)),
            Ok(Bytes::from(event2)),
            Ok(Bytes::from(event3)),
            Ok(Bytes::from(event4)),
        ];

        let stream = stream::iter(stream_data);

        let result_stream = process_anthropic_stream(stream);
        let chunks: Vec<_> = result_stream.collect().await;

        assert_eq!(chunks.len(), 3);

        let chunk0 = chunks[0].as_ref().expect("Chunk 0 error");
        assert_eq!(chunk0.content, "Hello");
        assert!(!chunk0.is_finished);

        let chunk1 = chunks[1].as_ref().expect("Chunk 1 error");
        assert_eq!(chunk1.content, " World");
        assert!(!chunk1.is_finished);

        let chunk2 = chunks[2].as_ref().expect("Chunk 2 error");
        assert!(chunk2.is_finished);
    }

    #[tokio::test]
    async fn test_process_anthropic_stream_split_events() {
        let part1 = "event: content_block_delta\ndata: {\"type\": \"content_block_delta\", \"index\": 0, \"delta\": {\"type\": \"text_d";
        let part2 = "elta\", \"text\": \"Split\"}}\n\n";

        let stream_data: Vec<reqwest::Result<Bytes>> = vec![
            Ok(Bytes::from(part1)),
            Ok(Bytes::from(part2)),
        ];

        let stream = stream::iter(stream_data);
        let result_stream = process_anthropic_stream(stream);
        let chunks: Vec<_> = result_stream.collect().await;

        assert_eq!(chunks.len(), 1);
        let chunk0 = chunks[0].as_ref().expect("Chunk 0 error");
        assert_eq!(chunk0.content, "Split");
    }
}
