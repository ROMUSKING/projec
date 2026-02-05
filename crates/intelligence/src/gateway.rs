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
        let request = serde_json::json!({
            "model": self.model,
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "validate"}]
        });

        let url = format!("{}/v1/messages", self.base_url);
        let response = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::ExternalService(format!("Anthropic connection failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
                return Err(Error::Config(format!(
                    "Anthropic validation failed: Invalid API Key ({})",
                    status
                )));
            }
            return Err(Error::ExternalService(format!(
                "Anthropic validation failed: {}",
                status
            )));
        }

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

    async fn generate_stream(&self, _prompt: &str) -> Result<StreamResult> {
        // TODO: Implement Anthropic streaming
        Err(Error::Internal("Not implemented".to_string()))
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
        let response = self.client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await;

        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_arcee_health_check_success() {
        // Setup a mock server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let base_url = format!("http://127.0.0.1:{}", port);

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let response = "HTTP/1.1 200 OK\r\n\r\n";
            socket.write_all(response.as_bytes()).await.unwrap();
        });

        let gateway = ArceeGateway::new(
            "test-key".to_string(),
            "test-model".to_string(),
            reqwest::Client::new(),
        ).with_base_url(base_url);

        let result = gateway.health_check().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_arcee_health_check_failure() {
        // Setup a mock server that returns 500
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let base_url = format!("http://127.0.0.1:{}", port);

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let response = "HTTP/1.1 500 Internal Server Error\r\n\r\n";
            socket.write_all(response.as_bytes()).await.unwrap();
        });

        let gateway = ArceeGateway::new(
            "test-key".to_string(),
            "test-model".to_string(),
            reqwest::Client::new(),
        ).with_base_url(base_url);

        let result = gateway.health_check().await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_arcee_health_check_network_error() {
        // Bind to a port but don't accept connections (or close immediately)
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let base_url = format!("http://127.0.0.1:{}", port);
        drop(listener); // Port is now closed

        let gateway = ArceeGateway::new(
            "test-key".to_string(),
            "test-model".to_string(),
            reqwest::Client::new(),
        ).with_base_url(base_url);

        let result = gateway.health_check().await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
