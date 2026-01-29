//! HTTP tool implementation.
//!
//! This module provides HTTP operations like GET, POST, etc.

use super::{Parameter, ParameterType, ReturnType, Tool};
use common::{async_trait, Error, Result};
use serde_json::Value;
use std::collections::HashMap;
use tracing::warn;

/// HTTP tool
pub struct HttpTool;

#[async_trait]
impl Tool for HttpTool {
    fn name(&self) -> &str {
        "http"
    }

    fn description(&self) -> &str {
        "HTTP operations including GET, POST, PUT, DELETE"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter {
                name: "method".to_string(),
                description: "HTTP method: GET, POST, PUT, DELETE".to_string(),
                required: true,
                parameter_type: ParameterType::Enum(vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                ]),
                default: None,
            },
            Parameter {
                name: "url".to_string(),
                description: "Request URL".to_string(),
                required: true,
                parameter_type: ParameterType::String,
                default: None,
            },
            Parameter {
                name: "headers".to_string(),
                description: "Request headers".to_string(),
                required: false,
                parameter_type: ParameterType::Object(HashMap::new()),
                default: None,
            },
            Parameter {
                name: "body".to_string(),
                description: "Request body".to_string(),
                required: false,
                parameter_type: ParameterType::String,
                default: None,
            },
            Parameter {
                name: "timeout".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                parameter_type: ParameterType::Integer,
                default: Some(Value::Number(30.into())),
            },
        ]
    }

    fn returns(&self) -> ReturnType {
        ReturnType {
            description: "HTTP response".to_string(),
            return_type: ParameterType::Object({
                let mut map = std::collections::HashMap::new();
                map.insert("success".to_string(), ParameterType::Boolean);
                map.insert("status".to_string(), ParameterType::Integer);
                map.insert("headers".to_string(), ParameterType::Object(HashMap::new()));
                map.insert("body".to_string(), ParameterType::String);
                map
            }),
        }
    }

    async fn execute(&self, args: &Value) -> Result<Value> {
        let method = args
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Missing method parameter".to_string()))?;

        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Missing url parameter".to_string()))?;

        let headers = args.get("headers").and_then(|v| v.as_object());
        let body = args.get("body").and_then(|v| v.as_str());
        let timeout = args
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);

        match method {
            "GET" => self.get(url, headers, timeout).await,
            "POST" => self.post(url, headers, body, timeout).await,
            "PUT" => self.put(url, headers, body, timeout).await,
            "DELETE" => self.delete(url, headers, timeout).await,
            _ => Err(Error::Validation(format!("Unknown method: {}", method))),
        }
    }

    fn validate(&self, args: &Value) -> Result<()> {
        if args.get("method").is_none() {
            return Err(Error::Validation("Missing method parameter".to_string()));
        }
        if args.get("url").is_none() {
            return Err(Error::Validation("Missing url parameter".to_string()));
        }
        Ok(())
    }

    fn is_safe(&self, args: &Value) -> bool {
        // Strict URL validation to prevent SSRF
        if let Some(url) = args.get("url").and_then(|v| v.as_str()) {
            // Only allow HTTPS for external APIs (prevents MITM attacks)
            if !url.starts_with("https://") && !url.starts_with("http://localhost") && !url.starts_with("http://127.0.0.1") {
                warn!("Attempted to use insecure HTTP protocol: {}", url);
                return false;
            }

            // Allow only specific domains
            let allowed_domains = [
                "api.openai.com",
                "api.anthropic.com",
                "localhost",
                "127.0.0.1",
            ];
            
            let mut is_allowed = false;
            for domain in &allowed_domains {
                if url.contains(domain) {
                    is_allowed = true;
                    break;
                }
            }
            
            if !is_allowed {
                warn!("Attempted to access disallowed domain: {}", url);
                return false;
            }

            // Block internal/private IP ranges (RFC1918)
            if url.contains("192.168.") || url.contains("10.") || url.contains("172.16.") 
                || url.contains("172.17.") || url.contains("172.18.") || url.contains("172.19.")
                || url.contains("172.20.") || url.contains("172.21.") || url.contains("172.22.")
                || url.contains("172.23.") || url.contains("172.24.") || url.contains("172.25.")
                || url.contains("172.26.") || url.contains("172.27.") || url.contains("172.28.")
                || url.contains("172.29.") || url.contains("172.30.") || url.contains("172.31.") {
                warn!("Attempted to access internal/private IP range: {}", url);
                return false;
            }

            // Block loopback addresses (other than 127.0.0.1)
            if url.contains("::1") {
                warn!("Attempted to access IPv6 loopback address: {}", url);
                return false;
            }
        }

        true
    }
}

impl HttpTool {
    fn build_client(&self, timeout_secs: u64) -> Result<reqwest::Client> {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| Error::ExternalService(format!("Failed to build HTTP client: {}", e)))
    }

    fn build_headers(&self, headers: Option<&serde_json::Map<String, Value>>) -> reqwest::header::HeaderMap {
        let mut header_map = reqwest::header::HeaderMap::new();

        if let Some(h) = headers {
            for (key, value) in h {
                if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                    if let Some(val_str) = value.as_str() {
                        if let Ok(header_value) = reqwest::header::HeaderValue::from_str(val_str) {
                            header_map.insert(header_name, header_value);
                        }
                    }
                }
            }
        }

        header_map
    }

    async fn get(&self, url: &str, headers: Option<&serde_json::Map<String, Value>>, timeout: u64) -> Result<Value> {
        let client = self.build_client(timeout)?;
        let header_map = self.build_headers(headers);

        let response = client
            .get(url)
            .headers(header_map)
            .send()
            .await
            .map_err(|e| Error::ExternalService(format!("HTTP request failed: {}", e)))?;

        let status = response.status().as_u16() as i64;
        let headers = response.headers().clone();
        let body = response
            .text()
            .await
            .map_err(|e| Error::ExternalService(format!("Failed to read response body: {}", e)))?;

        let header_json: HashMap<String, String> = headers
            .iter()
            .filter_map(|(k, v)| {
                v.to_str().ok().map(|s| (k.to_string(), s.to_string()))
            })
            .collect();

        Ok(serde_json::json!({
            "success": status >= 200 && status < 300,
            "status": status,
            "headers": header_json,
            "body": body,
        }))
    }

    async fn post(&self, url: &str, headers: Option<&serde_json::Map<String, Value>>, body: Option<&str>, timeout: u64) -> Result<Value> {
        let client = self.build_client(timeout)?;
        let header_map = self.build_headers(headers);

        let mut request = client.post(url).headers(header_map);

        if let Some(b) = body {
            request = request.body(b.to_string());
        }

        let response = request
            .send()
            .await
            .map_err(|e| Error::ExternalService(format!("HTTP request failed: {}", e)))?;

        let status = response.status().as_u16() as i64;
        let response_body = response
            .text()
            .await
            .map_err(|e| Error::ExternalService(format!("Failed to read response body: {}", e)))?;

        Ok(serde_json::json!({
            "success": status >= 200 && status < 300,
            "status": status,
            "body": response_body,
        }))
    }

    async fn put(&self, url: &str, headers: Option<&serde_json::Map<String, Value>>, body: Option<&str>, timeout: u64) -> Result<Value> {
        let client = self.build_client(timeout)?;
        let header_map = self.build_headers(headers);

        let mut request = client.put(url).headers(header_map);

        if let Some(b) = body {
            request = request.body(b.to_string());
        }

        let response = request
            .send()
            .await
            .map_err(|e| Error::ExternalService(format!("HTTP request failed: {}", e)))?;

        let status = response.status().as_u16() as i64;
        let response_body = response
            .text()
            .await
            .map_err(|e| Error::ExternalService(format!("Failed to read response body: {}", e)))?;

        Ok(serde_json::json!({
            "success": status >= 200 && status < 300,
            "status": status,
            "body": response_body,
        }))
    }

    async fn delete(&self, url: &str, headers: Option<&serde_json::Map<String, Value>>, timeout: u64) -> Result<Value> {
        let client = self.build_client(timeout)?;
        let header_map = self.build_headers(headers);

        let response = client
            .delete(url)
            .headers(header_map)
            .send()
            .await
            .map_err(|e| Error::ExternalService(format!("HTTP request failed: {}", e)))?;

        let status = response.status().as_u16() as i64;
        let response_body = response
            .text()
            .await
            .map_err(|e| Error::ExternalService(format!("Failed to read response body: {}", e)))?;

        Ok(serde_json::json!({
            "success": status >= 200 && status < 300,
            "status": status,
            "body": response_body,
        }))
    }
}