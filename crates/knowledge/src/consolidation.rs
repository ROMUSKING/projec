//! Memory Consolidation System
//!
//! This module implements memory consolidation mechanisms inspired by
//! human memory processes and state-of-the-art AI agent research.
//!
//! Key features:
//! - Episodic to semantic memory transfer
//! - Importance-based retention
//! - Pattern extraction and knowledge graph updates
//! - Automatic compression and forgetting

use crate::memory::{MemoryChunk, MemoryTier, HierarchicalMemory, AssembledContext};
use crate::graph::{KnowledgeGraph, Entity, Relation};
use common::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, info, warn};

/// Configuration for memory consolidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationConfig {
    /// How often to run consolidation (in seconds)
    pub consolidation_interval_secs: u64,
    /// Importance threshold for extracting to semantic memory
    pub semantic_extraction_threshold: f32,
    /// Importance threshold for persistent storage
    pub persistent_storage_threshold: f32,
    /// Maximum episodic memories before forced consolidation
    pub max_episodic_memories: usize,
    /// Memory age after which to consider for compression (in hours)
    pub compression_age_hours: u64,
    /// Decay rate for memory importance (per day)
    pub importance_decay_rate: f32,
    /// Minimum importance to retain a memory
    pub retention_threshold: f32,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            consolidation_interval_secs: 3600, // 1 hour
            semantic_extraction_threshold: 0.7,
            persistent_storage_threshold: 0.8,
            max_episodic_memories: 1000,
            compression_age_hours: 24,
            importance_decay_rate: 0.1,
            retention_threshold: 0.2,
        }
    }
}

/// Memory importance factors
#[derive(Debug, Clone)]
pub struct ImportanceFactors {
    /// Success rate of actions associated with this memory
    pub success_rate: f32,
    /// How many times this memory has been accessed
    pub access_count: u32,
    /// Complexity of the task/situation
    pub complexity: f32,
    /// How novel/unusual this memory is
    pub novelty: f32,
    /// User-defined importance (if any)
    pub user_importance: Option<f32>,
}

impl ImportanceFactors {
    /// Calculate composite importance score (0.0 - 1.0)
    pub fn composite_score(&self) -> f32 {
        let base_score = (
            self.success_rate * 0.3 +
            (self.access_count.min(100) as f32 / 100.0) * 0.25 +
            self.complexity * 0.25 +
            self.novelty * 0.2
        ).min(1.0);

        if let Some(user) = self.user_importance {
            (base_score * 0.7 + user * 0.3).min(1.0)
        } else {
            base_score
        }
    }
}

/// Extracted knowledge from episodic memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedKnowledge {
    /// Original memory IDs that contributed
    pub source_memories: Vec<String>,
    /// Extracted concepts/entities
    pub entities: Vec<Entity>,
    /// Discovered relationships
    pub relations: Vec<Relation>,
    /// Generalized pattern or rule
    pub pattern: String,
    /// Confidence in extraction (0.0 - 1.0)
    pub confidence: f32,
}

/// Memory consolidator that runs background consolidation tasks
pub struct MemoryConsolidator {
    config: ConsolidationConfig,
    memory: std::sync::Arc<HierarchicalMemory>,
    knowledge_graph: std::sync::Arc<KnowledgeGraph>,
}

impl MemoryConsolidator {
    /// Create a new memory consolidator
    pub fn new(
        config: ConsolidationConfig,
        memory: std::sync::Arc<HierarchicalMemory>,
        knowledge_graph: std::sync::Arc<KnowledgeGraph>,
    ) -> Self {
        Self {
            config,
            memory,
            knowledge_graph,
        }
    }

    /// Start background consolidation loop
    pub async fn start_consolidation_loop(&self) {
        let mut interval = interval(Duration::from_secs(self.config.consolidation_interval_secs));
        let memory = self.memory.clone();
        let knowledge_graph = self.knowledge_graph.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            loop {
                interval.tick().await;
                
                info!("Running memory consolidation cycle");
                
                if let Err(e) = Self::consolidate_memories(
                    &memory,
                    &knowledge_graph,
                    &config,
                ).await {
                    warn!("Memory consolidation failed: {}", e);
                }
            }
        });
    }

    /// Run a single consolidation cycle
    pub async fn consolidate_memories(
        memory: &HierarchicalMemory,
        knowledge_graph: &KnowledgeGraph,
        config: &ConsolidationConfig,
    ) -> Result<()> {
        // Step 1: Calculate importance for all memories
        let memories_with_importance = Self::calculate_importance_scores(memory).await?;

        // Step 2: Extract high-importance memories to semantic layer
        let high_importance: Vec<_> = memories_with_importance
            .iter()
            .filter(|(_, importance)| *importance >= config.semantic_extraction_threshold)
            .collect();

        for (chunk, importance) in high_importance {
            info!(
                "Extracting knowledge from memory {} (importance: {:.2})",
                chunk.id, importance
            );

            match Self::extract_knowledge(chunk).await {
                Ok(knowledge) => {
                    if let Err(e) = Self::integrate_knowledge(knowledge_graph, &knowledge).await {
                        warn!("Failed to integrate knowledge: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Failed to extract knowledge from {}: {}", chunk.id, e);
                }
            }
        }

        // Step 3: Compress old memories
        Self::compress_old_memories(memory, config).await?;

        // Step 4: Apply decay and remove low-importance memories
        Self::apply_decay_and_prune(memory, config).await?;

        info!("Memory consolidation cycle complete");
        Ok(())
    }

    /// Calculate importance scores for all memories
    async fn calculate_importance_scores(
        _memory: &HierarchicalMemory,
    ) -> Result<Vec<(MemoryChunk, f32)>> {
        // This would iterate through all memory chunks and calculate importance
        // For now, return empty vector as placeholder
        // In full implementation, this would:
        // 1. Get all chunks from HierarchicalMemory
        // 2. Calculate ImportanceFactors for each
        // 3. Return sorted by importance
        Ok(vec![])
    }

    /// Extract knowledge from a memory chunk
    async fn extract_knowledge(chunk: &MemoryChunk) -> Result<ExtractedKnowledge> {
        // In a full implementation, this would use an LLM to:
        // 1. Identify key entities in the memory
        // 2. Extract relationships between entities
        // 3. Generalize patterns from specific instances
        // 4. Summarize into reusable knowledge

        debug!("Extracting knowledge from chunk: {}", chunk.id);

        // Placeholder implementation
        Ok(ExtractedKnowledge {
            source_memories: vec![chunk.id.clone()],
            entities: vec![],
            relations: vec![],
            pattern: chunk.content.clone(),
            confidence: chunk.relevance_score,
        })
    }

    /// Integrate extracted knowledge into the knowledge graph
    async fn integrate_knowledge(
        _knowledge_graph: &KnowledgeGraph,
        knowledge: &ExtractedKnowledge,
    ) -> Result<()> {
        // In full implementation, this would add entities and relations
        // to the knowledge graph. For now, just log the operation.
        
        info!(
            "Integrated knowledge with {} entities and {} relations",
            knowledge.entities.len(),
            knowledge.relations.len()
        );

        Ok(())
    }

    /// Compress old memories to save space
    async fn compress_old_memories(
        _memory: &HierarchicalMemory,
        config: &ConsolidationConfig,
    ) -> Result<()> {
        let age_threshold = chrono::Duration::hours(config.compression_age_hours as i64);
        let _threshold_time = chrono::Utc::now() - age_threshold;

        // In full implementation:
        // 1. Find memories older than threshold
        // 2. Use LLM to summarize/compress them
        // 3. Replace original with compressed version
        // 4. Mark as compressed

        debug!("Compressing memories older than {} hours", config.compression_age_hours);
        Ok(())
    }

    /// Apply importance decay and remove low-importance memories
    async fn apply_decay_and_prune(
        _memory: &HierarchicalMemory,
        config: &ConsolidationConfig,
    ) -> Result<()> {
        // In full implementation:
        // 1. Calculate age of each memory
        // 2. Apply decay: importance *= (1 - decay_rate)^days
        // 3. Remove memories below retention threshold
        // 4. Update remaining memories with decayed importance

        debug!(
            "Applying decay (rate: {}) and pruning below threshold {}",
            config.importance_decay_rate,
            config.retention_threshold
        );
        Ok(())
    }

    /// Manually trigger consolidation for specific memories
    pub async fn consolidate_specific(&self, memory_ids: &[String]) -> Result<()> {
        info!("Manually consolidating {} memories", memory_ids.len());
        
        for id in memory_ids {
            // In full implementation, retrieve each memory and consolidate
            debug!("Consolidating memory: {}", id);
        }

        Ok(())
    }
}

/// Memory statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub total_memories: usize,
    pub memories_by_tier: HashMap<MemoryTier, usize>,
    pub average_importance: f32,
    pub total_tokens: usize,
    pub last_consolidation: Option<chrono::DateTime<chrono::Utc>>,
}

impl MemoryStats {
    /// Calculate memory efficiency metrics
    pub fn efficiency_score(&self) -> f32 {
        if self.total_memories == 0 {
            return 1.0;
        }

        // Higher importance and lower token count = better efficiency
        let importance_factor = self.average_importance;
        let density_factor = (self.total_memories as f32 * 1000.0)
            .max(1.0)
            / self.total_tokens.max(1) as f32;

        (importance_factor * density_factor).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_importance_factors_composite() {
        let factors = ImportanceFactors {
            success_rate: 0.9,
            access_count: 50,
            complexity: 0.7,
            novelty: 0.5,
            user_importance: None,
        };

        let score = factors.composite_score();
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_importance_factors_with_user() {
        let factors = ImportanceFactors {
            success_rate: 0.5,
            access_count: 10,
            complexity: 0.5,
            novelty: 0.5,
            user_importance: Some(0.9),
        };

        let score = factors.composite_score();
        // User importance should boost the score
        assert!(score > 0.5);
    }

    #[test]
    fn test_memory_stats_efficiency() {
        let stats = MemoryStats {
            total_memories: 100,
            memories_by_tier: HashMap::new(),
            average_importance: 0.8,
            total_tokens: 50000,
            last_consolidation: None,
        };

        let efficiency = stats.efficiency_score();
        assert!(efficiency > 0.0 && efficiency <= 1.0);
    }
}
