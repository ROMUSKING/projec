//! Vertex AI Gateway implementation
//!
//! This module provides integration with Google Cloud Vertex AI,
//! supporting multiple models including Gemini, Claude, and Llama.

use crate::gateway::{LlmGateway, ModelCapability, ModelInfo, StreamChunk, StreamResult};
use common::{async_trait, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vertex AI Gateway
pub struct VertexAiGateway {
    project_id: String,
    location: String,
    api_key: String,
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl VertexAiGateway {
    /// Create a new Vertex AI gateway
    pub fn new(project_id: String, location: String, api_key: String, model: String) -> Self {
        let base_url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers",
            location, project_id, location
        );

        Self {
            project_id,
            location,
            api_key,
            base_url,
            model,
            client: reqwest::Client::new(),
        }
    }

    /// Get the publisher for the current model
    fn get_publisher(&self) -> &str {
        // Determine publisher based on model name
        if self.model.starts_with("gemini") {
            "google"
        } else if self.model.starts_with("claude") {
            "anthropic"
        } else if self.model.contains("llama") {
            "meta"
        } else {
            "google" // default
        }
    }

    /// Build the API URL for the current model
    fn build_api_url(&self) -> String {
        let publisher = self.get_publisher();
        format!(
            "{}/{}/models/{}:generateContent",
            self.base_url, publisher, self.model
        )
    }

    /// Build the streaming API URL
    fn build_streaming_url(&self) -> String {
        let publisher = self.get_publisher();
        format!(
            "{}/{}/models/{}:streamGenerateContent",
            self.base_url, publisher, self.model
        )
    }

    /// Map model name to capabilities
    fn get_model_capabilities(&self) -> Vec<ModelCapability> {
        let mut caps = vec![
            ModelCapability::Chat,
            ModelCapability::Streaming,
        ];

        if self.model.starts_with("gemini") {
            caps.push(ModelCapability::Vision);
            if self.model.contains("pro") {
                caps.push(ModelCapability::FunctionCalling);
            }
        } else if self.model.starts_with("claude") {
            caps.push(ModelCapability::Vision);
            caps.push(ModelCapability::FunctionCalling);
        }

        caps
    }
}

#[async_trait]
impl LlmGateway for VertexAiGateway {
    async fn initialize(&mut self) -> Result<()> {
        // Validate configuration by making a health check
        self.health_check().await?;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        // No cleanup needed for Vertex AI
        Ok(())
    }

    async fn generate(&self, prompt: &str) -> Result<crate::GenerationResult> {
        let url = self.build_api_url();

        let request_body = serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [{"text": prompt}]
            }],
            "generationConfig": {
                "temperature": 0.7,
                "maxOutputTokens": 4096,
                "topP": 0.95,
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::ExternalService(format!("Vertex AI request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::ExternalService(format!(
                "Vertex AI error: {}",
                error_text
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::ExternalService(format!("Failed to parse JSON: {}", e)))?;

        // Extract content from response
        let content = response_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let tokens_used = response_json["usageMetadata"]["totalTokenCount"]
            .as_u64()
            .unwrap_or(0) as u32;

        Ok(crate::GenerationResult {
            content,
            tokens_used,
            model: self.model.clone(),
            finish_reason: response_json["candidates"][0]["finishReason"]
                .as_str()
                .unwrap_or("STOP")
                .to_string(),
        })
    }

    async fn generate_stream(&self, _prompt: &str) -> Result<StreamResult> {
        // For now, return a simple error. Full streaming implementation would require
        // setting up a stream that yields StreamChunk items
        Err(Error::Internal("Streaming not yet implemented for Vertex AI".to_string()))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // Return supported Vertex AI models
        let models = vec![
            ModelInfo {
                id: "gemini-1.5-pro-002".to_string(),
                name: "Gemini 1.5 Pro".to_string(),
                context_window: 2_000_000,
                capabilities: vec![
                    ModelCapability::Chat,
                    ModelCapability::Vision,
                    ModelCapability::FunctionCalling,
                    ModelCapability::Streaming,
                ],
            },
            ModelInfo {
                id: "gemini-1.5-flash-002".to_string(),
                name: "Gemini 1.5 Flash".to_string(),
                context_window: 1_000_000,
                capabilities: vec![
                    ModelCapability::Chat,
                    ModelCapability::Vision,
                    ModelCapability::Streaming,
                ],
            },
            ModelInfo {
                id: "gemini-1.0-pro-002".to_string(),
                name: "Gemini 1.0 Pro".to_string(),
                context_window: 32_768,
                capabilities: vec![
                    ModelCapability::Chat,
                    ModelCapability::FunctionCalling,
                    ModelCapability::Streaming,
                ],
            },
            ModelInfo {
                id: "claude-3-5-sonnet@20241022".to_string(),
                name: "Claude 3.5 Sonnet".to_string(),
                context_window: 200_000,
                capabilities: vec![
                    ModelCapability::Chat,
                    ModelCapability::Vision,
                    ModelCapability::FunctionCalling,
                    ModelCapability::Streaming,
                ],
            },
            ModelInfo {
                id: "claude-3-opus@20240229".to_string(),
                name: "Claude 3 Opus".to_string(),
                context_window: 200_000,
                capabilities: vec![
                    ModelCapability::Chat,
                    ModelCapability::Vision,
                    ModelCapability::FunctionCalling,
                    ModelCapability::Streaming,
                ],
            },
            ModelInfo {
                id: "claude-3-haiku@20240307".to_string(),
                name: "Claude 3 Haiku".to_string(),
                context_window: 200_000,
                capabilities: vec![
                    ModelCapability::Chat,
                    ModelCapability::Vision,
                    ModelCapability::Streaming,
                ],
            },
            ModelInfo {
                id: "llama-3.1-405b-instruct-maas".to_string(),
                name: "Llama 3.1 405B".to_string(),
                context_window: 128_000,
                capabilities: vec![
                    ModelCapability::Chat,
                    ModelCapability::Streaming,
                ],
            },
            ModelInfo {
                id: "llama-3.1-70b-instruct-maas".to_string(),
                name: "Llama 3.1 70B".to_string(),
                context_window: 128_000,
                capabilities: vec![
                    ModelCapability::Chat,
                    ModelCapability::Streaming,
                ],
            },
        ];

        Ok(models)
    }

    async fn health_check(&self) -> Result<bool> {
        // Simple health check by listing models
        match self.list_models().await {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::warn!("Vertex AI health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

/// Request body for Vertex AI generateContent
#[derive(Debug, Serialize)]
struct GenerateContentRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safety_settings: Option<Vec<SafetySetting>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f32,
    max_output_tokens: i32,
    top_p: f32,
}

#[derive(Debug, Serialize)]
struct SafetySetting {
    category: String,
    threshold: String,
}

/// Response from Vertex AI generateContent
#[derive(Debug, Deserialize)]
struct GenerateContentResponse {
    candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Content,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<i32>,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<i32>,
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_ai_gateway_creation() {
        let gateway = VertexAiGateway::new(
            "test-project".to_string(),
            "us-central1".to_string(),
            "test-api-key".to_string(),
            "gemini-1.5-pro-002".to_string(),
        );

        assert_eq!(gateway.project_id, "test-project");
        assert_eq!(gateway.location, "us-central1");
        assert_eq!(gateway.model, "gemini-1.5-pro-002");
        assert!(gateway.base_url.contains("us-central1-aiplatform"));
    }

    #[test]
    fn test_get_publisher() {
        let gemini_gateway = VertexAiGateway::new(
            "test".to_string(),
            "us-central1".to_string(),
            "key".to_string(),
            "gemini-1.5-pro".to_string(),
        );
        assert_eq!(gemini_gateway.get_publisher(), "google");

        let claude_gateway = VertexAiGateway::new(
            "test".to_string(),
            "us-central1".to_string(),
            "key".to_string(),
            "claude-3-5-sonnet".to_string(),
        );
        assert_eq!(claude_gateway.get_publisher(), "anthropic");

        let llama_gateway = VertexAiGateway::new(
            "test".to_string(),
            "us-central1".to_string(),
            "key".to_string(),
            "llama-3.1-405b".to_string(),
        );
        assert_eq!(llama_gateway.get_publisher(), "meta");
    }

    #[test]
    fn test_build_api_url() {
        let gateway = VertexAiGateway::new(
            "my-project".to_string(),
            "us-central1".to_string(),
            "key".to_string(),
            "gemini-1.5-pro-002".to_string(),
        );

        let url = gateway.build_api_url();
        assert!(url.contains("my-project"));
        assert!(url.contains("us-central1"));
        assert!(url.contains("gemini-1.5-pro-002"));
        assert!(url.contains("google"));
    }
}
