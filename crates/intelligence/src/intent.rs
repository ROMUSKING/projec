//! Intent parsing module.
//!
//! This module handles natural language understanding and intent classification.

use common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Intent parser for natural language understanding
pub struct IntentParser {
    // TODO: Add NLP model or rule-based classifier
}

impl IntentParser {
    pub fn new() -> Self {
        Self {}
    }

    /// Parse user input into structured intent
    pub async fn parse(&self, input: &str) -> Result<super::Intent> {
        // TODO: Implement actual NLP parsing
        // For now, use simple keyword matching
        let category = self.classify_intent(input);
        let parameters = self.extract_parameters(input);
        let confidence = 0.8; // Placeholder

        Ok(super::Intent {
            category,
            confidence,
            parameters,
            raw_input: input.to_string(),
        })
    }

    fn classify_intent(&self, input: &str) -> super::IntentCategory {
        let input_lower = input.to_lowercase();

        if input_lower.contains("create") || input_lower.contains("generate") || input_lower.contains("new") {
            super::IntentCategory::CodeGeneration
        } else if input_lower.contains("modify") || input_lower.contains("change") || input_lower.contains("update") || input_lower.contains("refactor") {
            super::IntentCategory::CodeModification
        } else if input_lower.contains("analyze") || input_lower.contains("understand") || input_lower.contains("explain") {
            super::IntentCategory::Analysis
        } else if input_lower.contains("test") || input_lower.contains("testing") {
            super::IntentCategory::Testing
        } else if input_lower.contains("document") || input_lower.contains("doc") {
            super::IntentCategory::Documentation
        } else if input_lower.contains("optimize") || input_lower.contains("performance") || input_lower.contains("improve") {
            super::IntentCategory::Optimization
        } else if input_lower.contains("improve yourself") || input_lower.contains("learn") || input_lower.contains("self") {
            super::IntentCategory::SelfImprovement
        } else {
            super::IntentCategory::Unknown
        }
    }

    fn extract_parameters(&self, input: &str) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();

        // Extract file paths
        let file_pattern = regex::Regex::new(r"`([^`]+\.(rs|toml|json|md))`").unwrap();
        let files: Vec<String> = file_pattern
            .captures_iter(input)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        if !files.is_empty() {
            params.insert("files".to_string(), serde_json::json!(files));
        }

        // Extract code blocks
        let code_pattern = regex::Regex::new(r"```(\w+)?\n(.*?)```").unwrap();
        let code_blocks: Vec<String> = code_pattern
            .captures_iter(input)
            .filter_map(|cap| cap.get(2).map(|m| m.as_str().to_string()))
            .collect();

        if !code_blocks.is_empty() {
            params.insert("code_blocks".to_string(), serde_json::json!(code_blocks));
        }

        params
    }
}

impl Default for IntentParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Intent classification model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentClassifier {
    pub categories: Vec<IntentCategoryDefinition>,
}

/// Definition of an intent category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentCategoryDefinition {
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub examples: Vec<String>,
}

/// Extracted entity from user input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_type: EntityType,
    pub value: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

/// Types of entities that can be extracted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    FilePath,
    FunctionName,
    VariableName,
    CodeBlock,
    Language,
    Number,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_intent_classification() {
        let parser = IntentParser::new();

        let intent = parser.parse("Create a new function").await.unwrap();
        assert_eq!(intent.category, super::super::IntentCategory::CodeGeneration);

        let intent = parser.parse("Refactor this code").await.unwrap();
        assert_eq!(intent.category, super::super::IntentCategory::CodeModification);

        let intent = parser.parse("Analyze the codebase").await.unwrap();
        assert_eq!(intent.category, super::super::IntentCategory::Analysis);
    }
}