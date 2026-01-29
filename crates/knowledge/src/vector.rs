//! Vector store module.
//!
//! This module provides semantic search capabilities using
//! vector embeddings and similarity search.

use common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Vector store for semantic search
pub struct VectorStore {
    documents: HashMap<String, DocumentEmbedding>,
    // TODO: Add Qdrant or other vector database client
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    /// Initialize the vector store connection
    pub async fn initialize(&mut self) -> Result<()> {
        // TODO: Connect to vector database
        Ok(())
    }

    /// Shutdown the vector store connection
    pub async fn shutdown(&mut self) -> Result<()> {
        // TODO: Close vector database connection
        Ok(())
    }

    /// Index a document for semantic search
    pub async fn index_document(&mut self, path: &Path, content: &str) -> Result<()> {
        let embedding = self.generate_embedding(content).await?;

        let doc = DocumentEmbedding {
            id: path.to_string_lossy().to_string(),
            path: path.to_path_buf(),
            content: content.to_string(),
            embedding,
            metadata: DocumentMetadata {
                indexed_at: common::chrono::Utc::now(),
                chunk_index: 0,
            },
        };

        self.documents.insert(doc.id.clone(), doc);
        Ok(())
    }

    /// Search for similar documents
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<super::SearchResult>> {
        let query_embedding = self.generate_embedding(query).await?;

        let mut results: Vec<(String, f32)> = self
            .documents
            .values()
            .map(|doc| {
                let similarity = cosine_similarity(&query_embedding, &doc.embedding);
                (doc.id.clone(), similarity)
            })
            .filter(|(_, sim)| *sim > 0.7) // Threshold
            .collect();

        // Sort by similarity (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(limit);

        // Convert to SearchResult
        let search_results: Vec<super::SearchResult> = results
            .into_iter()
            .filter_map(|(id, score)| {
                self.documents.get(&id).map(|doc| super::SearchResult {
                    source: doc.path.to_string_lossy().to_string(),
                    content: doc.content.clone(),
                    relevance: score,
                    result_type: super::ResultType::Code,
                })
            })
            .collect();

        Ok(search_results)
    }

    /// Generate embedding for text
    async fn generate_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        // TODO: Integrate with embedding model (e.g., via API or local model)
        // For now, return a placeholder embedding
        Ok(vec![0.0; 384]) // 384-dimensional placeholder
    }

    /// Delete a document from the index
    pub async fn delete_document(&mut self, path: &Path) -> Result<()> {
        let id = path.to_string_lossy().to_string();
        self.documents.remove(&id);
        Ok(())
    }

    /// Get document count
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    /// Clear all documents
    pub async fn clear(&mut self) -> Result<()> {
        self.documents.clear();
        Ok(())
    }
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Document with embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEmbedding {
    pub id: String,
    pub path: std::path::PathBuf,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: DocumentMetadata,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub indexed_at: common::chrono::DateTime<common::chrono::Utc>,
    pub chunk_index: usize,
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Embedding model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model: String,
    pub dimensions: usize,
    pub batch_size: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model: "text-embedding-ada-002".to_string(),
            dimensions: 1536,
            batch_size: 100,
        }
    }
}

/// Text chunker for splitting long documents
pub struct TextChunker {
    chunk_size: usize,
    overlap: usize,
}

impl TextChunker {
    pub fn new(chunk_size: usize, overlap: usize) -> Self {
        Self {
            chunk_size,
            overlap,
        }
    }

    /// Split text into chunks
    pub fn chunk(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut start = 0;

        while start < text.len() {
            let end = (start + self.chunk_size).min(text.len());
            let chunk = &text[start..end];
            chunks.push(chunk.to_string());

            start += self.chunk_size - self.overlap;
        }

        chunks
    }
}

impl Default for TextChunker {
    fn default() -> Self {
        Self::new(1000, 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c)).abs() < 0.001);
    }

    #[test]
    fn test_text_chunker() {
        let chunker = TextChunker::new(10, 2);
        let text = "This is a long text that needs to be chunked into smaller pieces";
        let chunks = chunker.chunk(text);
        assert!(!chunks.is_empty());
        assert!(chunks[0].len() <= 10);
    }
}