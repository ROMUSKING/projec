//! File system tool implementation.
//!
//! This module provides file operations like read, write, edit, list, etc.

use super::{Parameter, ParameterType, ReturnType, Tool};
use common::{async_trait, Error, Result};
use serde_json::Value;
use std::path::Path;
use tokio::io::AsyncBufReadExt;
use tracing::warn;

/// File system tool
pub struct FileSystemTool {
    max_file_size: u64,
}

impl FileSystemTool {
    pub fn new(max_file_size: u64) -> Self {
        Self { max_file_size }
    }
}

#[async_trait]
impl Tool for FileSystemTool {
    fn name(&self) -> &str {
        "filesystem"
    }

    fn description(&self) -> &str {
        "File system operations including read, write, edit, list, and delete"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter {
                name: "operation".to_string(),
                description: "Operation to perform: read, write, edit, list, delete, search".to_string(),
                required: true,
                parameter_type: ParameterType::Enum(vec![
                    "read".to_string(),
                    "write".to_string(),
                    "edit".to_string(),
                    "list".to_string(),
                    "delete".to_string(),
                    "search".to_string(),
                ]),
                default: None,
            },
            Parameter {
                name: "path".to_string(),
                description: "File or directory path".to_string(),
                required: true,
                parameter_type: ParameterType::String,
                default: None,
            },
            Parameter {
                name: "content".to_string(),
                description: "Content for write operations".to_string(),
                required: false,
                parameter_type: ParameterType::String,
                default: None,
            },
            Parameter {
                name: "pattern".to_string(),
                description: "Search pattern for search operations".to_string(),
                required: false,
                parameter_type: ParameterType::String,
                default: None,
            },
        ]
    }

    fn returns(&self) -> ReturnType {
        ReturnType {
            description: "Operation result".to_string(),
            return_type: ParameterType::Object({
                let mut map = std::collections::HashMap::new();
                map.insert("success".to_string(), ParameterType::Boolean);
                map.insert("content".to_string(), ParameterType::String);
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

        match operation {
            "read" => self.read_file(path).await,
            "write" => {
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::Validation("Missing content parameter".to_string()))?;
                self.write_file(path, content).await
            }
            "list" => self.list_directory(path).await,
            "delete" => self.delete_file(path).await,
            "search" => {
                let pattern = args
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::Validation("Missing pattern parameter".to_string()))?;
                self.search_files(path, pattern).await
            }
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
        Ok(())
    }

    fn is_safe(&self, args: &Value) -> bool {
        // Check for forbidden paths and validate paths to prevent traversal
        if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
            let forbidden = ["/etc/passwd", "/etc/shadow", ".ssh/id_rsa"];
            for f in &forbidden {
                if path.contains(f) {
                    return false;
                }
            }

            // Prevent path traversal
            if path.contains("../") || path.starts_with("/") {
                warn!("Attempted path traversal or absolute path access: {}", path);
                return false;
            }

            // Check for null bytes
            if path.contains('\0') {
                warn!("Attempted path with null byte: {}", path);
                return false;
            }
        }
        true
    }
}

impl FileSystemTool {
    async fn read_file(&self, path: &str) -> Result<Value> {
        // Safety: Check file size before reading to prevent OOM
        let metadata = tokio::fs::metadata(path).await?;

        if metadata.len() > self.max_file_size {
            return Err(Error::Validation(format!(
                "File too large to read: {} bytes (max {} bytes)",
                metadata.len(),
                self.max_file_size
            )));
        }

        let content = tokio::fs::read_to_string(path).await?;
        Ok(serde_json::json!({
            "success": true,
            "content": content,
            "path": path,
        }))
    }

    async fn write_file(&self, path: &str, content: &str) -> Result<Value> {
        // Create parent directories if needed
        if let Some(parent) = Path::new(path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(path, content).await?;
        Ok(serde_json::json!({
            "success": true,
            "path": path,
            "bytes_written": content.len(),
        }))
    }

    async fn list_directory(&self, path: &str) -> Result<Value> {
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(path).await?;

        while let Some(entry) = dir.next_entry().await? {
            let metadata = entry.metadata().await?;
            let file_type = if metadata.is_dir() {
                "directory"
            } else if metadata.is_file() {
                "file"
            } else {
                "other"
            };

            entries.push(serde_json::json!({
                "name": entry.file_name().to_string_lossy().to_string(),
                "type": file_type,
                "size": metadata.len(),
            }));
        }

        Ok(serde_json::json!({
            "success": true,
            "path": path,
            "entries": entries,
        }))
    }

    async fn delete_file(&self, path: &str) -> Result<Value> {
        let path_obj = Path::new(path);

        if path_obj.is_dir() {
            tokio::fs::remove_dir_all(path).await?;
        } else {
            tokio::fs::remove_file(path).await?;
        }

        Ok(serde_json::json!({
            "success": true,
            "path": path,
        }))
    }

    async fn search_files(&self, path: &str, pattern: &str) -> Result<Value> {
        let regex = regex::Regex::new(pattern)
            .map_err(|e| Error::Validation(format!("Invalid regex: {}", e)))?;

        let mut matches = Vec::new();

        for entry in walkdir::WalkDir::new(path) {
            let entry = entry.map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            if entry.file_type().is_file() {
                // Optimize: Read line by line instead of loading whole file
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
        }

        Ok(serde_json::json!({
            "success": true,
            "pattern": pattern,
            "matches": matches,
        }))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_search_files_large() {
        // Create a temporary file
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_owned();
        let path_str = path.to_str().unwrap();

        // Write 10MB of data
        // We write lines to test line iteration
        let chunk = "a".repeat(1024);
        for _ in 0..10240 {
            writeln!(temp_file, "{}", chunk).unwrap();
        }
        writeln!(temp_file, "needle").unwrap();
        temp_file.flush().unwrap();

        let tool = FileSystemTool::new(10 * 1024 * 1024);

        // Search specifically in this file (WalkDir handles file paths too)
        let result = tool.search_files(path_str, "needle").await.unwrap();
        let matches = result["matches"].as_array().unwrap();

        assert!(!matches.is_empty(), "Should find matches");
        let found = matches.iter().any(|m| {
            m["content"].as_str().unwrap() == "needle"
        });
        assert!(found, "Should find 'needle' in large file");
    }

    #[tokio::test]
    async fn test_read_file_limit() {
        // Create a temporary file larger than 10MB
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_owned();
        let path_str = path.to_str().unwrap();

        // Write 10MB + 1 byte
        let chunk = "a".repeat(1024);
        for _ in 0..10241 {
            writeln!(temp_file, "{}", chunk).unwrap();
        }
        temp_file.flush().unwrap();

        let tool = FileSystemTool::new(10 * 1024 * 1024);
        let result = tool.read_file(path_str).await;

        assert!(result.is_err());
        match result {
            Err(Error::Validation(msg)) => {
                assert!(msg.contains("File too large"));
            }
            _ => panic!("Expected Validation error"),
        }
    }
}
