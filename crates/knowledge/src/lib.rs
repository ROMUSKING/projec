//! Knowledge management layer for the coding agent.
//!
//! This crate provides documentation management, knowledge graph,
//! and vector store integration for semantic search.

use common::{async_trait, Error, Module, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod consolidation;
pub mod documentation;
pub mod graph;
pub mod memory;
pub mod vector;

/// Main knowledge engine
pub struct KnowledgeEngine {
    doc_manager: documentation::DocumentationManager,
    knowledge_graph: graph::KnowledgeGraph,
    vector_store: vector::VectorStore,
    /// Hierarchical memory system for context management
    pub memory: memory::HierarchicalMemory,
}

impl KnowledgeEngine {
    pub fn new() -> Self {
        Self {
            doc_manager: documentation::DocumentationManager::new(),
            knowledge_graph: graph::KnowledgeGraph::new(),
            vector_store: vector::VectorStore::new(),
            memory: memory::HierarchicalMemory::new(),
        }
    }

    /// Index a document for search
    pub async fn index_document(&mut self, path: &PathBuf, content: &str) -> Result<()> {
        // Index in vector store for semantic search
        self.vector_store.index_document(path, content).await?;

        // Extract entities and relations for knowledge graph
        let entities = self.extract_entities(content);
        for entity in entities {
            self.knowledge_graph.add_entity(entity).await?;
        }

        // Store in documentation manager
        self.doc_manager.store_document(path, content).await?;

        Ok(())
    }

    /// Search for relevant knowledge
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // Semantic search via vector store
        let semantic_results = self.vector_store.search(query, limit).await?;
        results.extend(semantic_results);

        // Graph search for related concepts
        let graph_results = self.knowledge_graph.search(query, limit).await?;
        results.extend(graph_results);

        // Sort by relevance
        results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        results.truncate(limit);

        Ok(results)
    }

    /// Get documentation for a specific topic
    pub async fn get_documentation(&self, topic: &str) -> Result<Option<Documentation>> {
        self.doc_manager.get_documentation(topic).await
    }

    /// Query the knowledge graph
    pub async fn query_graph(&self, query: &GraphQuery) -> Result<Vec<GraphResult>> {
        self.knowledge_graph.query(query).await
    }

    fn extract_entities(&self, _content: &str) -> Vec<graph::Entity> {
        // TODO: Implement entity extraction
        vec![]
    }
}

#[async_trait]
impl Module for KnowledgeEngine {
    fn name(&self) -> &str {
        "knowledge"
    }

    async fn initialize(&mut self) -> Result<()> {
        self.vector_store.initialize().await?;
        self.knowledge_graph.initialize().await?;
        self.doc_manager.initialize().await?;
        self.memory.initialize().await
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.vector_store.shutdown().await?;
        self.knowledge_graph.shutdown().await?;
        self.doc_manager.shutdown().await?;
        self.memory.shutdown().await
    }
}

impl Default for KnowledgeEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Search result from knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub source: String,
    pub content: String,
    pub relevance: f32,
    pub result_type: ResultType,
}

/// Types of search results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultType {
    Documentation,
    Code,
    Historical,
    Pattern,
}

/// Documentation structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Documentation {
    pub title: String,
    pub content: String,
    pub path: PathBuf,
    pub metadata: DocumentMetadata,
}

/// Document metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub created_at: common::chrono::DateTime<common::chrono::Utc>,
    pub updated_at: common::chrono::DateTime<common::chrono::Utc>,
    pub author: Option<String>,
    pub tags: Vec<String>,
}

/// Knowledge graph query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQuery {
    pub query_type: QueryType,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Types of graph queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryType {
    FindEntity,
    FindRelations,
    FindPath,
    SimilarEntities,
}

/// Graph query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResult {
    pub entities: Vec<graph::Entity>,
    pub relations: Vec<graph::Relation>,
    pub score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_type_serialization() {
        let rt = ResultType::Documentation;
        let json = serde_json::to_string(&rt).unwrap();
        assert_eq!(json, "\"documentation\"");
    }
}