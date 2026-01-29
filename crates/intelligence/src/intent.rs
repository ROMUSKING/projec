//! Intent parsing module.
//!
//! This module handles natural language understanding and intent classification.

use common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rule for intent classification
struct IntentRule {
    category: super::IntentCategory,
    keywords: Vec<(&'static str, f32)>,
}

/// Intent parser for natural language understanding
pub struct IntentParser {
    rules: Vec<IntentRule>,
    // Compiled regexes
    file_pattern: regex::Regex,
    code_pattern: regex::Regex,
    fn_pattern: regex::Regex,
    var_pattern: regex::Regex,
    decompose_pattern: regex::Regex,
}

impl IntentParser {
    pub fn new() -> Self {
        Self {
            rules: vec![
                IntentRule {
                    category: super::IntentCategory::CodeGeneration,
                    keywords: vec![
                        ("create", 1.0), ("generate", 1.0), ("new", 0.8), ("write", 0.9),
                        ("implement", 1.0), ("build", 0.8), ("add", 0.6)
                    ],
                },
                IntentRule {
                    category: super::IntentCategory::CodeModification,
                    keywords: vec![
                        ("modify", 1.0), ("change", 1.0), ("update", 1.0), ("refactor", 1.0),
                        ("fix", 1.0), ("edit", 0.8), ("improve", 0.5), ("optimize", 0.5)
                    ],
                },
                IntentRule {
                    category: super::IntentCategory::Analysis,
                    keywords: vec![
                        ("analyze", 1.0), ("understand", 1.0), ("explain", 1.0), ("check", 0.8),
                        ("review", 0.9), ("scan", 0.7), ("audit", 0.9)
                    ],
                },
                IntentRule {
                    category: super::IntentCategory::Testing,
                    keywords: vec![
                        ("test", 1.0), ("testing", 1.0), ("verify", 0.9), ("benchmark", 0.9),
                        ("coverage", 0.8)
                    ],
                },
                IntentRule {
                    category: super::IntentCategory::Documentation,
                    keywords: vec![
                        ("document", 1.0), ("documentation", 1.0), ("doc", 0.9), ("comment", 0.8),
                        ("describe", 0.7)
                    ],
                },
                IntentRule {
                    category: super::IntentCategory::Optimization,
                    keywords: vec![
                        ("optimize", 1.0), ("performance", 1.0), ("improve", 0.8), ("speed", 0.7),
                        ("fast", 0.6), ("efficient", 0.8)
                    ],
                },
                IntentRule {
                    category: super::IntentCategory::SelfImprovement,
                    keywords: vec![
                        ("self improve", 1.0), ("improve yourself", 1.0), ("learn", 0.8), ("self", 0.5)
                    ],
                },
            ],
            file_pattern: regex::Regex::new(r"`([^`]+\.(rs|toml|json|md|txt|yml|yaml))`").unwrap(),
            code_pattern: regex::Regex::new(r"```(\w+)?\n(.*?)```").unwrap(),
            fn_pattern: regex::Regex::new(r"fn\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap(),
            var_pattern: regex::Regex::new(r"(?:let|const)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap(),
            decompose_pattern: regex::Regex::new(r"(?i)\s+(?:and|then)\s+|;\s+|\.\s+").unwrap(),
        }
    }

    /// Parse user input into structured intent
    pub async fn parse(&self, input: &str) -> Result<super::Intent> {
        // TODO: Implement actual NLP parsing (e.g. LLM based)
        // For now, use weighted keyword matching
        let (category, confidence) = self.classify_intent(input);
        let parameters = self.extract_parameters(input);

        Ok(super::Intent {
            category,
            confidence,
            parameters,
            raw_input: input.to_string(),
        })
    }

    /// Decompose a complex task into sub-tasks
    pub async fn decompose(&self, input: &str) -> Result<Vec<super::Intent>> {
        let mut intents = Vec::new();

        let parts: Vec<&str> = self.decompose_pattern.split(input).filter(|s| !s.trim().is_empty()).collect();

        for part in parts {
            intents.push(self.parse(part).await?);
        }

        Ok(intents)
    }

    fn classify_intent(&self, input: &str) -> (super::IntentCategory, f32) {
        let input_lower = input.to_lowercase();
        let mut best_category = super::IntentCategory::Unknown;
        let mut max_score = 0.0;

        for rule in &self.rules {
            let mut score = 0.0;
            for (keyword, weight) in &rule.keywords {
                if input_lower.contains(keyword) {
                    score += weight;
                }
            }

            if score > max_score {
                max_score = score;
                best_category = rule.category;
            }
        }

        // Normalize score to 0.0 - 1.0 range based on a heuristic max score (e.g., 3.0)
        let confidence = (max_score / 3.0).min(1.0);

        if max_score > 0.0 {
            (best_category, confidence)
        } else {
            (super::IntentCategory::Unknown, 0.0)
        }
    }

    fn extract_parameters(&self, input: &str) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();

        // Extract file paths
        let files: Vec<String> = self.file_pattern
            .captures_iter(input)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        if !files.is_empty() {
            params.insert("files".to_string(), serde_json::json!(files));
        }

        // Extract code blocks
        let code_blocks: Vec<String> = self.code_pattern
            .captures_iter(input)
            .filter_map(|cap| cap.get(2).map(|m| m.as_str().to_string()))
            .collect();

        if !code_blocks.is_empty() {
            params.insert("code_blocks".to_string(), serde_json::json!(code_blocks));
        }

        // Extract function names: fn <name>
        let functions: Vec<String> = self.fn_pattern
            .captures_iter(input)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        if !functions.is_empty() {
            params.insert("functions".to_string(), serde_json::json!(functions));
        }

        // Extract variable names: let <name> or const <name>
        let variables: Vec<String> = self.var_pattern
            .captures_iter(input)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        if !variables.is_empty() {
            params.insert("variables".to_string(), serde_json::json!(variables));
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

    #[tokio::test]
    async fn test_weighted_scoring() {
        let parser = IntentParser::new();

        // "create" (1.0) + "new" (0.8) = 1.8 score
        let intent = parser.parse("create new file").await.unwrap();
        assert_eq!(intent.category, super::super::IntentCategory::CodeGeneration);
        assert!(intent.confidence > 0.5);
    }

    #[tokio::test]
    async fn test_parameter_extraction() {
        let parser = IntentParser::new();

        let input = "Check function fn process_data and variable let result";
        let intent = parser.parse(input).await.unwrap();

        let functions = intent.parameters.get("functions").unwrap().as_array().unwrap();
        assert_eq!(functions[0].as_str().unwrap(), "process_data");

        let variables = intent.parameters.get("variables").unwrap().as_array().unwrap();
        assert_eq!(variables[0].as_str().unwrap(), "result");
    }

    #[tokio::test]
    async fn test_decompose() {
        let parser = IntentParser::new();

        let input = "Create a new function and then test it";
        let intents = parser.decompose(input).await.unwrap();

        assert_eq!(intents.len(), 2);
        assert_eq!(intents[0].category, super::super::IntentCategory::CodeGeneration);
        assert_eq!(intents[1].category, super::super::IntentCategory::Testing);
    }

    #[tokio::test]
    async fn test_empty_input() {
        let parser = IntentParser::new();
        let intent = parser.parse("").await.unwrap();
        assert_eq!(intent.category, super::super::IntentCategory::Unknown);
        assert_eq!(intent.confidence, 0.0);
    }

    #[tokio::test]
    async fn test_unknown_intent() {
        let parser = IntentParser::new();
        let intent = parser.parse("blabla 1234").await.unwrap();
        assert_eq!(intent.category, super::super::IntentCategory::Unknown);
    }
}
