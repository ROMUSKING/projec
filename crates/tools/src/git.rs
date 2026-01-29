//! Git tool implementation.
//!
//! This module provides Git operations like status, diff, commit, etc.

use super::{Parameter, ParameterType, ReturnType, Tool};
use common::{async_trait, Error, Result};
use serde_json::Value;

/// Git tool
pub struct GitTool;

#[async_trait]
impl Tool for GitTool {
    fn name(&self) -> &str {
        "git"
    }

    fn description(&self) -> &str {
        "Git operations including status, diff, commit, branch, and push"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter {
                name: "operation".to_string(),
                description: "Operation to perform: status, diff, commit, branch, log, push".to_string(),
                required: true,
                parameter_type: ParameterType::Enum(vec![
                    "status".to_string(),
                    "diff".to_string(),
                    "commit".to_string(),
                    "branch".to_string(),
                    "log".to_string(),
                    "push".to_string(),
                ]),
                default: None,
            },
            Parameter {
                name: "path".to_string(),
                description: "Repository path".to_string(),
                required: true,
                parameter_type: ParameterType::String,
                default: None,
            },
            Parameter {
                name: "message".to_string(),
                description: "Commit message".to_string(),
                required: false,
                parameter_type: ParameterType::String,
                default: None,
            },
            Parameter {
                name: "files".to_string(),
                description: "Files to commit".to_string(),
                required: false,
                parameter_type: ParameterType::Array(Box::new(ParameterType::String)),
                default: None,
            },
        ]
    }

    fn returns(&self) -> ReturnType {
        ReturnType {
            description: "Git operation result".to_string(),
            return_type: ParameterType::Object({
                let mut map = std::collections::HashMap::new();
                map.insert("success".to_string(), ParameterType::Boolean);
                map.insert("output".to_string(), ParameterType::String);
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
            "status" => self.status(path).await,
            "diff" => self.diff(path).await,
            "commit" => {
                let message = args
                    .get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::Validation("Missing message parameter".to_string()))?;
                let files = args
                    .get("files")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                self.commit(path, message, &files).await
            }
            "branch" => self.branch(path).await,
            "log" => self.log(path).await,
            "push" => self.push(path).await,
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
        // Push operations require explicit approval
        if let Some(op) = args.get("operation").and_then(|v| v.as_str()) {
            if op == "push" {
                // TODO: Check if approval is granted
                return false;
            }
        }
        true
    }
}

impl GitTool {
    async fn run_git(&self, path: &str, args: &[&str]) -> Result<std::process::Output> {
        let output = tokio::process::Command::new("git")
            .current_dir(path)
            .args(args)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::ExternalService(format!("Git error: {}", stderr)));
        }

        Ok(output)
    }

    async fn status(&self, path: &str) -> Result<Value> {
        let output = self.run_git(path, &["status", "--porcelain"]).await?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();

        for line in stdout.lines() {
            if line.len() < 3 {
                continue;
            }
            let status = &line[0..2];
            let file = &line[3..];

            if status.starts_with('?') {
                untracked.push(file.to_string());
            } else if status.starts_with(' ') {
                unstaged.push(file.to_string());
            } else {
                staged.push(file.to_string());
            }
        }

        Ok(serde_json::json!({
            "success": true,
            "staged": staged,
            "unstaged": unstaged,
            "untracked": untracked,
        }))
    }

    async fn diff(&self, path: &str) -> Result<Value> {
        let output = self.run_git(path, &["diff"]).await?;
        let diff = String::from_utf8_lossy(&output.stdout);

        Ok(serde_json::json!({
            "success": true,
            "diff": diff.to_string(),
        }))
    }

    async fn commit(&self, path: &str, message: &str, files: &[String]) -> Result<Value> {
        // Add files if specified
        if !files.is_empty() {
            let mut args = vec!["add"];
            for file in files {
                args.push(file);
            }
            self.run_git(path, &args).await?;
        } else {
            // Add all changes
            self.run_git(path, &["add", "-A"]).await?;
        }

        // Commit
        let output = self.run_git(path, &["commit", "-m", message]).await?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        Ok(serde_json::json!({
            "success": true,
            "output": stdout.to_string(),
        }))
    }

    async fn branch(&self, path: &str) -> Result<Value> {
        let output = self.run_git(path, &["branch", "-v"]).await?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        let branches: Vec<Value> = stdout
            .lines()
            .map(|line| {
                let current = line.starts_with('*');
                let name = line.trim_start_matches('*').trim().split_whitespace().next().unwrap_or("");
                serde_json::json!({
                    "name": name,
                    "current": current,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "success": true,
            "branches": branches,
        }))
    }

    async fn log(&self, path: &str) -> Result<Value> {
        let output = self
            .run_git(path, &["log", "--oneline", "-n", "10"])
            .await?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        let commits: Vec<Value> = stdout
            .lines()
            .map(|line| {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                serde_json::json!({
                    "hash": parts.get(0).unwrap_or(&""),
                    "message": parts.get(1).unwrap_or(&""),
                })
            })
            .collect();

        Ok(serde_json::json!({
            "success": true,
            "commits": commits,
        }))
    }

    async fn push(&self, path: &str) -> Result<Value> {
        let output = self.run_git(path, &["push"]).await?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        Ok(serde_json::json!({
            "success": true,
            "output": stdout.to_string(),
        }))
    }
}