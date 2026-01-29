//! Common types and utilities shared across all crates.
//!
//! This crate provides foundational types, error definitions, and utility
//! functions that are used throughout the coding agent system.

use std::fmt;

/// Re-export commonly used external crates
pub use async_trait::async_trait;
pub use chrono;
pub use serde;
pub use serde_json;
pub use tracing;
pub use uuid;

/// Common result type used across the codebase
pub type Result<T> = std::result::Result<T, Error>;

/// Common error type for the coding agent
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Cancelled")]
    Cancelled,
}

/// Unique identifier for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TaskId(pub uuid::Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for sessions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SessionId(pub uuid::Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Timestamp type alias for consistency
pub type Timestamp = chrono::DateTime<chrono::Utc>;

/// Get current timestamp
pub fn now() -> Timestamp {
    chrono::Utc::now()
}

/// Module trait for all agent modules
#[async_trait]
pub trait Module: Send + Sync {
    /// Module name
    fn name(&self) -> &str;

    /// Initialize the module
    async fn initialize(&mut self) -> Result<()>;

    /// Shutdown the module
    async fn shutdown(&mut self) -> Result<()>;
}

/// Version information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Utility functions
pub mod utils {
    use super::*;

    /// Sanitize a string for safe display/logging
    pub fn sanitize(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
            .collect()
    }

    /// Truncate a string to a maximum length
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }
}

pub mod crypto;

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_task_id_generation() {
        let id1 = TaskId::new();
        let id2 = TaskId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_version_display() {
        let version = Version {
            major: 1,
            minor: 2,
            patch: 3,
        };
        assert_eq!(version.to_string(), "1.2.3");
    }

    proptest! {
        #[test]
        fn test_version_roundtrip(major in 0u32..100, minor in 0u32..100, patch in 0u32..100) {
            let version = Version { major, minor, patch };
            let serialized = serde_json::to_string(&version).unwrap();
            let deserialized: Version = serde_json::from_str(&serialized).unwrap();
            
            assert_eq!(version.major, deserialized.major);
            assert_eq!(version.minor, deserialized.minor);
            assert_eq!(version.patch, deserialized.patch);
        }
        
        #[test]
        fn test_sanitize_does_not_crash(s in "\\PC*") {
            let _ = utils::sanitize(&s);
        }
    }
}