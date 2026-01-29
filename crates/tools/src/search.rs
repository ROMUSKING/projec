//! Search tool implementation.
//!
//! This module provides search operations like grep, find, and symbol search.

use super::{Parameter, ParameterType, ReturnType, Tool};
use common::{async_trait, Error, Result};
use serde_json::Value;
use tokio::io::AsyncBufReadExt;
use tracing::warn;

/// Search tool
pub struct SearchTool;

#[async_trait]
impl Tool for SearchTool {
    fn name(&self) -> &str {
        "search"
    }

    fn description(&self) -> &str {
        "Search operations including grep, find, and symbol search"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter {
                name: "operation".to_string(),
                description: "Operation to perform: grep, find, symbol".to_string(),
                required: true,
                parameter_type: ParameterType::Enum(vec![
                    "grep".to_string(),
                    "find".to_string(),
                    "symbol".to_string(),
                ]),
                default: None,
            },
            Parameter {
                name: "path".to_string(),
                description: "Search path".to_string(),
                required: true,
                parameter_type: ParameterType::String,
                default: None,
            },
            Parameter {
                name: "pattern".to_string(),
                description: "Search pattern".to_string(),
                required: true,
                parameter_type: ParameterType::String,
                default: None,
            },
            Parameter {
                name: "file_pattern".to_string(),
                description: "File pattern to filter (e.g., *.rs)".to_string(),
                required: false,
                parameter_type: ParameterType::String,
                default: None,
            },
        ]
    }

    fn returns(&self) -> ReturnType {
        ReturnType {
            description: "Search results".to_string(),
            return_type: ParameterType::Object({
                let mut map = std::collections::HashMap::new();
                map.insert("success".to_string(), ParameterType::Boolean);
                map.insert("matches".to_string(), ParameterType::Array(Box::new(ParameterType::Object({
                    let mut match_map = std::collections::HashMap::new();
                    match_map.insert("path".to_string(), ParameterType::String);
                    match_map.insert("line".to_string(), ParameterType::Integer);
                    match_map.insert("content".to_string(), ParameterType::String);
                    match_map
                }))));
                map
            }),
        }
    }

    async fn execute(&self, args: &Value) -> Result<Value> {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Missing operation parameter".to_string()))?;

        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Missing path parameter".to_string()))?;

        let pattern = args
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Missing pattern parameter".to_string()))?;

        let file_pattern = args.get("file_pattern").and_then(|v| v.as_str());

        match operation {
            "grep" => self.grep(path, pattern, file_pattern).await,
            "find" => self.find(path, pattern).await,
            "symbol" => self.symbol_search(path, pattern).await,
            _ => Err(Error::Validation(format!("Unknown operation: {}", operation))),
        }
    }

    fn validate(&self, args: &Value) -> Result<()> {
        if args.get("operation").is_none() {
            return Err(Error::Validation("Missing operation parameter".to_string()));
        }
        if args.get("path").is_none() {
            return Err(Error::Validation("Missing path parameter".to_string()));
        }
        if args.get("pattern").is_none() {
            return Err(Error::Validation("Missing pattern parameter".to_string()));
        }
        Ok(())
    }

    fn is_safe(&self, _args: &Value) -> bool {
        true
    }
}

impl SearchTool {
    async fn grep(&self, path: &str, pattern: &str, file_pattern: Option<&str>) -> Result<Value> {
        let regex = regex::Regex::new(pattern)
            .map_err(|e| Error::Validation(format!("Invalid regex: {}", e)))?;

        let mut matches = Vec::new();

        for entry in walkdir::WalkDir::new(path) {
            let entry = entry.map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            if !entry.file_type().is_file() {
                continue;
            }

            // Check file pattern if specified
            if let Some(fp) = file_pattern {
                if let Some(name) = entry.file_name().to_str() {
                    let glob_pattern = glob::Pattern::new(fp)
                        .map_err(|e| Error::Validation(format!("Invalid glob pattern: {}", e)))?;
                    if !glob_pattern.matches(name) {
                        continue;
                    }
                }
            }

            // Skip binary files and read line by line
            if let Ok(file) = tokio::fs::File::open(entry.path()).await {
                let reader = tokio::io::BufReader::new(file);
                let mut lines = reader.lines();
                let mut line_num = 0;

                loop {
                    match lines.next_line().await {
                        Ok(Some(line)) => {
                            line_num += 1;
                            if regex.is_match(&line) {
                                matches.push(serde_json::json!({
                                    "path": entry.path().to_string_lossy().to_string(),
                                    "line": line_num,
                                    "content": line.trim(),
                                }));
                            }
                        }
                        Ok(None) => break, // EOF
                        Err(e) => {
                            warn!("Error reading file {}: {}", entry.path().display(), e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(serde_json::json!({
            "success": true,
            "pattern": pattern,
            "matches": matches,
            "total": matches.len(),
        }))
    }

    async fn find(&self, path: &str, pattern: &str) -> Result<Value> {
        let glob_pattern = glob::Pattern::new(pattern)
            .map_err(|e| Error::Validation(format!("Invalid glob pattern: {}", e)))?;

        let mut matches = Vec::new();

        for entry in walkdir::WalkDir::new(path) {
            let entry = entry.map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            if let Some(name) = entry.file_name().to_str() {
                if glob_pattern.matches(name) {
                    matches.push(serde_json::json!({
                        "path": entry.path().to_string_lossy().to_string(),
                        "name": name,
                        "is_directory": entry.file_type().is_dir(),
                    }));
                }
            }
        }

        Ok(serde_json::json!({
            "success": true,
            "pattern": pattern,
            "matches": matches,
            "total": matches.len(),
        }))
    }

    async fn symbol_search(&self, _path: &str, _symbol: &str) -> Result<Value> {
        // TODO: Implement symbol search using LSP
        Ok(serde_json::json!({
            "success": true,
            "pattern": _symbol,
            "matches": [],
            "total": 0,
            "note": "Symbol search requires LSP integration",
        }))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_search_tool_large_file() {
        // Create a temp dir to isolate the test
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large_file.txt");
        let mut temp_file = std::fs::File::create(&file_path).unwrap();

        // Write 10MB of data
        let chunk = "a".repeat(1024);
        for _ in 0..10240 {
             writeln!(temp_file, "{}", chunk).unwrap();
        }
        writeln!(temp_file, "needle").unwrap();

        let tool = SearchTool;
        let args = serde_json::json!({
            "operation": "grep",
            "path": temp_dir.path().to_str().unwrap(),
            "pattern": "needle",
            "file_pattern": "large_file.txt"
        });

        let result = tool.execute(&args).await.unwrap();
        let matches = result["matches"].as_array().unwrap();

        assert!(!matches.is_empty(), "Should find matches");
        let found = matches.iter().any(|m| {
             m["content"].as_str().unwrap() == "needle"
        });
        assert!(found, "Should find 'needle' in large file");
    }
}
