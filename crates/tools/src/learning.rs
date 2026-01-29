//! Tool Learning and Discovery Framework
//!
//! This module implements tool learning capabilities inspired by:
//! - Voyager's skill library approach
//! - AutoGPT's tool evolution
//! - MetaGPT's role-based tool use
//!
//! Key features:
//! - Tool embedding for semantic search
//! - API documentation parsing and tool synthesis
//! - Tool composition and chaining
//! - Tool effectiveness learning from execution traces

use crate::{Tool, ToolResult, ToolFramework};
use common::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Embedding vector for tool similarity search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEmbedding {
    /// Tool name
    pub tool_name: String,
    /// Embedding vector
    pub vector: Vec<f32>,
    /// Extracted capabilities
    pub capabilities: Vec<String>,
    /// Tool description
    pub description: String,
}

impl ToolEmbedding {
    /// Calculate cosine similarity with another embedding
    pub fn similarity(&self, other: &ToolEmbedding) -> f32 {
        if self.vector.is_empty() || other.vector.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = self
            .vector
            .iter()
            .zip(other.vector.iter())
            .map(|(a, b)| a * b)
            .sum();

        let self_norm: f32 = self.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        let other_norm: f32 = other.vector.iter().map(|x| x * x).sum::<f32>().sqrt();

        if self_norm == 0.0 || other_norm == 0.0 {
            0.0
        } else {
            dot_product / (self_norm * other_norm)
        }
    }
}

/// Tool library for storing and retrieving tool embeddings
pub struct ToolLibrary {
    embeddings: Arc<RwLock<HashMap<String, ToolEmbedding>>>,
    // In a full implementation, this would connect to a vector database
    // like Qdrant, Pinecone, or Weaviate
}

impl ToolLibrary {
    /// Create a new tool library
    pub fn new() -> Self {
        Self {
            embeddings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store a tool embedding
    pub async fn store(&self, embedding: ToolEmbedding) -> Result<()> {
        let mut embeddings = self.embeddings.write().await;
        let tool_name = embedding.tool_name.clone();
        embeddings.insert(tool_name.clone(), embedding);
        debug!("Stored embedding for tool: {}", tool_name);
        Ok(())
    }

    /// Find similar tools by embedding
    pub async fn find_similar(&self, query: &ToolEmbedding, limit: usize) -> Result<Vec<(String, f32)>> {
        let embeddings = self.embeddings.read().await;
        
        let mut similarities: Vec<(String, f32)> = embeddings
            .values()
            .map(|emb| (emb.tool_name.clone(), query.similarity(emb)))
            .collect();

        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similarities.truncate(limit);

        Ok(similarities)
    }

    /// Search tools by capability
    pub async fn search_by_capability(&self, capability: &str) -> Result<Vec<String>> {
        let embeddings = self.embeddings.read().await;
        
        let matching: Vec<String> = embeddings
            .values()
            .filter(|emb| emb.capabilities.iter().any(|c| c.contains(capability)))
            .map(|emb| emb.tool_name.clone())
            .collect();

        Ok(matching)
    }

    /// Get all stored tool names
    pub async fn list_tools(&self) -> Vec<String> {
        let embeddings = self.embeddings.read().await;
        embeddings.keys().cloned().collect()
    }
}

impl Default for ToolLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool embedder for creating embeddings from tools and tasks
pub struct ToolEmbedder;

impl ToolEmbedder {
    /// Create a new tool embedder
    pub fn new() -> Self {
        Self
    }

    /// Embed a tool for storage
    pub async fn embed_tool(&self, tool: &dyn Tool) -> Result<ToolEmbedding> {
        // In a full implementation, this would use an embedding model
        // (e.g., OpenAI's text-embedding-ada-002 or a local model)
        // to create a semantic embedding from the tool's description
        
        let description = format!(
            "{}: {}. Parameters: {:?}",
            tool.name(),
            tool.description(),
            tool.parameters()
        );

        // Placeholder: create a simple hash-based embedding
        let vector = Self::simple_embed(&description);
        let capabilities = Self::extract_capabilities(tool);

        Ok(ToolEmbedding {
            tool_name: tool.name().to_string(),
            vector,
            capabilities,
            description,
        })
    }

    /// Embed a task description for tool search
    pub async fn embed_task(&self, task_description: &str) -> Result<ToolEmbedding> {
        // Placeholder implementation
        let vector = Self::simple_embed(task_description);

        Ok(ToolEmbedding {
            tool_name: "query".to_string(),
            vector,
            capabilities: vec![],
            description: task_description.to_string(),
        })
    }

    /// Simple embedding for prototyping (replace with actual embedding model)
    fn simple_embed(text: &str) -> Vec<f32> {
        // This is a placeholder that creates a simple frequency-based vector
        // In production, use a proper embedding model
        let mut counts = HashMap::new();
        for word in text.to_lowercase().split_whitespace() {
            *counts.entry(word.to_string()).or_insert(0) += 1;
        }

        // Create a fixed-size vector (simplified)
        let mut vector = vec![0.0f32; 128];
        for (i, (_, count)) in counts.iter().enumerate().take(128) {
            vector[i] = *count as f32;
        }

        // Normalize
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut vector {
                *x /= norm;
            }
        }

        vector
    }

    /// Extract capabilities from a tool
    fn extract_capabilities(tool: &dyn Tool) -> Vec<String> {
        let mut capabilities = vec![];
        let name = tool.name().to_lowercase();
        let desc = tool.description().to_lowercase();

        // Simple keyword-based capability extraction
        if name.contains("file") || desc.contains("file") {
            capabilities.push("filesystem".to_string());
        }
        if name.contains("git") || desc.contains("git") {
            capabilities.push("version_control".to_string());
        }
        if name.contains("search") || desc.contains("search") {
            capabilities.push("search".to_string());
        }
        if name.contains("http") || desc.contains("http") || desc.contains("api") {
            capabilities.push("network".to_string());
        }

        capabilities
    }
}

impl Default for ToolEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool learner for discovering and synthesizing new tools
pub struct ToolLearner {
    embedder: ToolEmbedder,
}

impl ToolLearner {
    /// Create a new tool learner
    pub fn new() -> Self {
        Self {
            embedder: ToolEmbedder::new(),
        }
    }

    /// Discover tools from API documentation
    pub async fn discover_from_docs(&self, api_docs: &str) -> Result<Vec<DiscoveredTool>> {
        info!("Discovering tools from API documentation");

        // In a full implementation, this would:
        // 1. Parse OpenAPI/Swagger specs
        // 2. Extract endpoints, parameters, and descriptions
        // 3. Generate tool implementations

        let mut discovered = vec![];

        // Simple heuristic-based discovery (placeholder)
        if api_docs.contains("openapi") || api_docs.contains("swagger") {
            // Parse OpenAPI spec
            discovered.extend(self.parse_openapi(api_docs).await?);
        }

        info!("Discovered {} tools from documentation", discovered.len());
        Ok(discovered)
    }

    /// Parse OpenAPI specification
    async fn parse_openapi(&self, _spec: &str) -> Result<Vec<DiscoveredTool>> {
        // Placeholder: In full implementation, use openapi-parser crate
        debug!("Parsing OpenAPI specification");
        
        // Return empty for now
        Ok(vec![])
    }

    /// Learn a tool from demonstration examples
    pub async fn learn_from_demonstration(
        &self,
        name: &str,
        description: &str,
        examples: Vec<ToolDemonstration>,
    ) -> Result<LearnedTool> {
        info!("Learning tool '{}' from {} demonstrations", name, examples.len());

        // In a full implementation, this would:
        // 1. Analyze successful tool executions
        // 2. Extract patterns and parameters
        // 3. Generate a tool implementation
        // 4. Validate with test cases

        Ok(LearnedTool {
            name: name.to_string(),
            description: description.to_string(),
            parameters: HashMap::new(),
            examples,
            confidence: 0.5,
        })
    }

    /// Synthesize a composite tool from existing tools
    pub async fn synthesize_composite(
        &self,
        name: &str,
        description: &str,
        steps: Vec<CompositeStep>,
    ) -> Result<CompositeTool> {
        info!("Synthesizing composite tool '{}' with {} steps", name, steps.len());

        Ok(CompositeTool {
            name: name.to_string(),
            description: description.to_string(),
            steps,
            created_at: chrono::Utc::now(),
        })
    }
}

impl Default for ToolLearner {
    fn default() -> Self {
        Self::new()
    }
}

/// A tool discovered from API documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredTool {
    pub name: String,
    pub description: String,
    pub endpoint: String,
    pub method: String,
    pub parameters: Vec<ParameterSpec>,
    pub source: String,
}

/// Parameter specification for discovered tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

/// A tool learned from demonstrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedTool {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, String>,
    pub examples: Vec<ToolDemonstration>,
    pub confidence: f32,
}

/// Demonstration example for tool learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDemonstration {
    pub input: serde_json::Value,
    pub expected_output: ToolResult,
    pub context: String,
}

/// A composite tool made of multiple steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeTool {
    pub name: String,
    pub description: String,
    pub steps: Vec<CompositeStep>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A step in a composite tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeStep {
    pub tool_name: String,
    pub argument_mapping: HashMap<String, String>,
    pub condition: Option<String>,
}

/// Tool effectiveness tracker for learning from execution
pub struct ToolEffectivenessTracker {
    stats: Arc<RwLock<HashMap<String, ToolStats>>>,
}

impl ToolEffectivenessTracker {
    /// Create a new effectiveness tracker
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a tool execution result
    pub async fn record_execution(
        &self,
        tool_name: &str,
        success: bool,
        duration_ms: u64,
    ) -> Result<()> {
        let mut stats = self.stats.write().await;
        let entry = stats.entry(tool_name.to_string()).or_insert_with(ToolStats::default);

        entry.total_calls += 1;
        if success {
            entry.successful_calls += 1;
        }
        entry.total_duration_ms += duration_ms;
        entry.last_used = Some(chrono::Utc::now());

        debug!(
            "Recorded execution for {}: success={}, duration={}ms",
            tool_name, success, duration_ms
        );

        Ok(())
    }

    /// Get effectiveness statistics for a tool
    pub async fn get_stats(&self, tool_name: &str) -> Option<ToolStats> {
        let stats = self.stats.read().await;
        stats.get(tool_name).cloned()
    }

    /// Get all tool statistics
    pub async fn get_all_stats(&self) -> HashMap<String, ToolStats> {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Find underperforming tools
    pub async fn find_underperforming(&self, threshold: f32) -> Vec<(String, ToolStats)> {
        let stats = self.stats.read().await;
        
        stats
            .iter()
            .filter(|(_, s)| s.success_rate() < threshold)
            .map(|(name, stats)| (name.clone(), stats.clone()))
            .collect()
    }
}

impl Default for ToolEffectivenessTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for tool effectiveness
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolStats {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub total_duration_ms: u64,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

impl ToolStats {
    /// Calculate success rate
    pub fn success_rate(&self) -> f32 {
        if self.total_calls == 0 {
            0.0
        } else {
            self.successful_calls as f32 / self.total_calls as f32
        }
    }

    /// Calculate average duration
    pub fn average_duration_ms(&self) -> f64 {
        if self.total_calls == 0 {
            0.0
        } else {
            self.total_duration_ms as f64 / self.total_calls as f64
        }
    }
}

/// Enhanced ToolFramework with learning capabilities
pub struct LearningToolFramework {
    base_framework: ToolFramework,
    library: ToolLibrary,
    embedder: ToolEmbedder,
    learner: ToolLearner,
    tracker: ToolEffectivenessTracker,
}

impl LearningToolFramework {
    /// Create a new learning tool framework
    pub fn new() -> Self {
        Self {
            base_framework: ToolFramework::new(),
            library: ToolLibrary::new(),
            embedder: ToolEmbedder::new(),
            learner: ToolLearner::new(),
            tracker: ToolEffectivenessTracker::new(),
        }
    }

    /// Initialize and index all existing tools
    pub async fn initialize_tool_library(&self) -> Result<()> {
        info!("Initializing tool library with existing tools");

        for tool in self.base_framework.list_tools() {
            let embedding = self.embedder.embed_tool(tool).await?;
            self.library.store(embedding).await?;
        }

        info!("Tool library initialized");
        Ok(())
    }

    /// Find the best tool for a task
    pub async fn find_tool_for_task(&self, task_description: &str) -> Result<Vec<(String, f32)>> {
        let query = self.embedder.embed_task(task_description).await?;
        self.library.find_similar(&query, 5).await
    }

    /// Execute a tool with effectiveness tracking
    pub async fn execute_tracked(&self, tool_name: &str, args: serde_json::Value) -> Result<ToolResult> {
        let start = std::time::Instant::now();
        
        let result = self.base_framework.execute(tool_name, args).await;
        
        let duration = start.elapsed().as_millis() as u64;
        let success = result.is_ok();
        
        self.tracker.record_execution(tool_name, success, duration).await?;
        
        result
    }

    /// Discover new tools from API documentation
    pub async fn discover_tools(&self, api_docs: &str) -> Result<Vec<DiscoveredTool>> {
        self.learner.discover_from_docs(api_docs).await
    }

    /// Get tool effectiveness report
    pub async fn effectiveness_report(&self) -> HashMap<String, ToolStats> {
        self.tracker.get_all_stats().await
    }
}

impl Default for LearningToolFramework {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_embedding_similarity() {
        let emb1 = ToolEmbedding {
            tool_name: "tool1".to_string(),
            vector: vec![1.0, 0.0, 0.0],
            capabilities: vec!["test".to_string()],
            description: "test".to_string(),
        };

        let emb2 = ToolEmbedding {
            tool_name: "tool2".to_string(),
            vector: vec![1.0, 0.0, 0.0],
            capabilities: vec!["test".to_string()],
            description: "test".to_string(),
        };

        let similarity = emb1.similarity(&emb2);
        assert!(similarity > 0.99); // Should be nearly identical
    }

    #[test]
    fn test_tool_stats_success_rate() {
        let stats = ToolStats {
            total_calls: 10,
            successful_calls: 8,
            total_duration_ms: 1000,
            last_used: None,
        };

        assert_eq!(stats.success_rate(), 0.8);
        assert_eq!(stats.average_duration_ms(), 100.0);
    }
}
