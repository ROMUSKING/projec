//! Telemetry and survey data collection.
//! 
//! This module handles the secure collection of anonymized performance metrics.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use common::{Result, Error};
use common::crypto::CryptoManager;
use tokio::sync::RwLock;
use std::sync::Arc;

/// Anonymized survey data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveyData {
    pub session_id: String, // Randomly generated per session
    pub timestamp: common::chrono::DateTime<common::chrono::Utc>,
    pub task_type: String, // e.g. "CodeGeneration"
    pub model: String,
    pub latency_ms: u64,
    pub success: bool,
    pub score: Option<f32>, // 0.0 to 10.0
    pub tokens_used: u32,
    pub cost_usd: f64,
    pub error_type: Option<String>,
}

/// Telemetry manager handles data collection and encryption
pub struct TelemetryManager {
    config: agent_config::TelemetryConfig,
    crypto: CryptoManager,
    buffer: Arc<RwLock<Vec<SurveyData>>>,
    session_id: String,
}

impl TelemetryManager {
    pub fn new(config: agent_config::TelemetryConfig) -> Self {
        let crypto = CryptoManager::new(); // In prod, load key
        let session_id = common::uuid::Uuid::new_v4().to_string();
        
        Self {
            config,
            crypto,
            buffer: Arc::new(RwLock::new(Vec::new())),
            session_id,
        }
    }

    /// Record a metrics event
    pub async fn record_event(&self, 
        task_type: &str, 
        model: &str, 
        latency_ms: u64, 
        success: bool,
        score: Option<f32>,
        tokens: u32
    ) {
        if !self.config.enabled {
            return;
        }

        let event = SurveyData {
            session_id: self.session_id.clone(),
            timestamp: common::chrono::Utc::now(),
            task_type: task_type.to_string(),
            model: model.to_string(),
            latency_ms,
            success,
            score,
            tokens_used: tokens,
            cost_usd: 0.0, // TODO: Implement cost calc
            error_type: None,
        };

        self.buffer.write().await.push(event);
        
        // Auto-save if buffer gets large
        if self.buffer.read().await.len() >= 10 {
            let _ = self.flush().await;
        }
    }

    /// Flush buffer to encrypted storage and optionally upload
    pub async fn flush(&self) -> Result<()> {
        if !self.config.enabled || self.buffer.read().await.is_empty() {
            return Ok(());
        }

        let mut buffer = self.buffer.write().await;
        let data = std::mem::take(&mut *buffer);
        
        let json = serde_json::to_string(&data)?;
        let encrypted = self.crypto.encrypt(json.as_bytes())?;
        
        // Save locally
        if !self.config.storage_path.exists() {
            tokio::fs::create_dir_all(&self.config.storage_path).await?;
        }
        
        let filename = format!("survey_{}.enc", common::chrono::Utc::now().timestamp_millis());
        let path = self.config.storage_path.join(filename);
        
        tokio::fs::write(path, &encrypted).await?;

        // Upload to central server if configured
        if let Some(url) = &self.config.central_server_url {
            self.upload(&encrypted, url).await?;
        }
        
        Ok(())
    }

    async fn upload(&self, encrypted_data: &str, url: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let mut request = client.post(format!("{}/api/v1/upload", url))
            .body(encrypted_data.to_string());

        if let Some(key) = &self.config.telemetry_api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        match request.send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    common::tracing::warn!("Failed to upload telemetry: {}", resp.status());
                }
            }
            Err(e) => {
                common::tracing::warn!("Error uploading telemetry: {}", e);
            }
        }
        
        Ok(())
    }
}
