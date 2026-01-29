//! Documentation management module.
//!
//! This module handles document lifecycle, auto-generation,
//! and validation of documentation.

use common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Documentation manager
pub struct DocumentationManager {
    documents: HashMap<String, super::Documentation>,
    storage_path: Option<PathBuf>,
}

impl DocumentationManager {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            storage_path: None,
        }
    }

    /// Initialize the documentation manager
    pub async fn initialize(&mut self) -> Result<()> {
        // TODO: Load existing documents from storage
        Ok(())
    }

    /// Shutdown the documentation manager
    pub async fn shutdown(&mut self) -> Result<()> {
        // TODO: Persist documents to storage
        Ok(())
    }

    /// Store a document
    pub async fn store_document(&mut self, path: &Path, content: &str) -> Result<()> {
        let doc = super::Documentation {
            title: Self::extract_title(content),
            content: content.to_string(),
            path: path.to_path_buf(),
            metadata: super::DocumentMetadata {
                created_at: common::chrono::Utc::now(),
                updated_at: common::chrono::Utc::now(),
                author: None,
                tags: Self::extract_tags(content),
            },
        };

        let key = path.to_string_lossy().to_string();
        self.documents.insert(key, doc);

        Ok(())
    }

    /// Get documentation by topic
    pub async fn get_documentation(&self, topic: &str) -> Result<Option<super::Documentation>> {
        // Search by title or content
        for doc in self.documents.values() {
            if doc.title.contains(topic) || doc.content.contains(topic) {
                return Ok(Some(doc.clone()));
            }
        }
        Ok(None)
    }

    /// Get all documents
    pub fn get_all_documents(&self) -> Vec<&super::Documentation> {
        self.documents.values().collect()
    }

    /// Generate documentation from code
    pub async fn generate_from_code(&self, _code_path: &Path) -> Result<String> {
        // TODO: Implement code-to-documentation generation
        Ok(String::new())
    }

    /// Validate documentation
    pub fn validate_documentation(&self, doc: &super::Documentation) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check for empty content
        if doc.content.trim().is_empty() {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                message: "Documentation content is empty".to_string(),
                location: None,
            });
        }

        // Check for broken links
        issues.extend(self.check_broken_links(doc));

        // Check for outdated content
        issues.extend(self.check_outdated_content(doc));

        issues
    }

    fn check_broken_links(&self, _doc: &super::Documentation) -> Vec<ValidationIssue> {
        // TODO: Implement link checking
        vec![]
    }

    fn check_outdated_content(&self, doc: &super::Documentation) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let age = common::chrono::Utc::now() - doc.metadata.updated_at;

        // Flag documents older than 90 days
        if age.num_days() > 90 {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                message: format!("Documentation may be outdated (last updated {} days ago)", age.num_days()),
                location: None,
            });
        }

        issues
    }

    fn extract_title(content: &str) -> String {
        // Extract first heading as title
        content
            .lines()
            .find(|line| line.starts_with("# "))
            .map(|line| line.trim_start_matches("# ").to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    fn extract_tags(content: &str) -> Vec<String> {
        // Extract tags from content (e.g., #tag or @tag)
        let mut tags = Vec::new();
        let tag_regex = regex::Regex::new(r"#(\w+)").unwrap();

        for cap in tag_regex.captures_iter(content) {
            if let Some(tag) = cap.get(1) {
                let tag_str = tag.as_str();
                tags.push(tag_str.to_string());
            }
        }

        tags
    }
}

impl Default for DocumentationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub message: String,
    pub location: Option<Location>,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Error,
    Warning,
    Information,
}

/// Location in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub line: u32,
    pub column: u32,
}

/// Document types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    ApiDocumentation,
    ArchitectureDecisionRecord,
    DesignPattern,
    CodeExample,
    TaskTemplate,
    WorkflowGuide,
    BestPractice,
    Troubleshooting,
    Configuration,
}

/// Architecture Decision Record (ADR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adr {
    pub number: u32,
    pub title: String,
    pub status: AdrStatus,
    pub context: String,
    pub decision: String,
    pub consequences: String,
    pub date: common::chrono::DateTime<common::chrono::Utc>,
}

/// ADR status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdrStatus {
    Proposed,
    Accepted,
    Deprecated,
    Superseded,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title() {
        let content = "# My Title\n\nSome content";
        assert_eq!(DocumentationManager::extract_title(content), "My Title");
    }

    #[test]
    fn test_extract_tags() {
        let content = "This is about #rust and #programming";
        let tags = DocumentationManager::extract_tags(content);
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"programming".to_string()));
    }
}