//! Self-compilation and restart capability for the coding agent.
//!
//! This module enables the agent to compile its own source code and restart
//! with the new binary, enabling true self-modification and improvement.

use common::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Stdio;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

/// Self-compilation manager
pub struct SelfCompiler {
    config: agent_config::SelfCompileConfig,
    project_root: PathBuf,
    backup_dir: PathBuf,
    current_binary: PathBuf,
}

impl SelfCompiler {
    /// Create a new self-compiler
    pub fn new(config: agent_config::SelfCompileConfig, project_root: impl AsRef<Path>) -> Self {
        let project_root = project_root.as_ref().to_path_buf();
        let backup_dir = project_root.join(".agent/backups");
        let current_binary = std::env::current_exe().unwrap_or_else(|_| project_root.join("coding-agent"));
        
        Self {
            config,
            project_root,
            backup_dir,
            current_binary,
        }
    }

    /// Initialize the self-compiler
    pub async fn initialize(&self) -> Result<()> {
        // Ensure backup directory exists
        if !self.backup_dir.exists() {
            tokio::fs::create_dir_all(&self.backup_dir).await?;
        }
        
        info!("Self-compiler initialized");
        info!("Project root: {:?}", self.project_root);
        info!("Backup directory: {:?}", self.backup_dir);
        info!("Current binary: {:?}", self.current_binary);
        
        Ok(())
    }

    /// Check if self-compilation is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Compile the project and return the path to the new binary
    pub async fn compile(&self) -> Result<PathBuf> {
        if !self.config.enabled {
            return Err(Error::Config("Self-compilation is disabled".to_string()));
        }

        info!("Starting self-compilation...");

        // Create backup of current binary
        self.backup_current_binary().await?;

        // Build the project
        let new_binary = self.build_project().await?;

        // Verify the new binary if configured
        if self.config.verify_before_restart {
            self.verify_binary(&new_binary).await?;
        }

        info!("Self-compilation completed successfully: {:?}", new_binary);
        Ok(new_binary)
    }

    /// Build the project using cargo
    async fn build_project(&self) -> Result<PathBuf> {
        let mut cmd = Command::new("cargo");
        cmd.arg("build")
            .arg("--bin")
            .arg("coding-agent");

        // Add profile
        match self.config.build_profile.as_str() {
            "release" => {
                cmd.arg("--release");
            }
            "dev" => {
                // Default, no flag needed
            }
            profile => {
                cmd.arg("--profile").arg(profile);
            }
        }

        // Add additional build arguments
        for arg in &self.config.build_args {
            cmd.arg(arg);
        }

        cmd.current_dir(&self.project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        debug!("Running: {:?}", cmd);

        let compile_timeout = Duration::from_secs(self.config.compile_timeout_seconds);
        let result = timeout(compile_timeout, cmd.output()).await;

        match result {
            Ok(Ok(output)) => {
                if output.status.success() {
                    // Determine the output binary path
                    let target_dir = self.project_root.join("target");
                    let binary_path = match self.config.build_profile.as_str() {
                        "release" => target_dir.join("release/coding-agent"),
                        "dev" => target_dir.join("debug/coding-agent"),
                        profile => target_dir.join(format!("{}/coding-agent", profile)),
                    };

                    if binary_path.exists() {
                        Ok(binary_path)
                    } else {
                        Err(Error::Internal(format!(
                            "Binary not found at expected path: {:?}",
                            binary_path
                        )))
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("Compilation failed:\n{}", stderr);
                    Err(Error::Internal(format!(
                        "Compilation failed with exit code: {:?}",
                        output.status.code()
                    )))
                }
            }
            Ok(Err(e)) => {
                error!("Failed to execute cargo build: {}", e);
                Err(Error::Io(e))
            }
            Err(_) => {
                error!("Compilation timed out after {} seconds", self.config.compile_timeout_seconds);
                Err(Error::Timeout("Compilation timed out".to_string()))
            }
        }
    }

    /// Create a backup of the current binary
    async fn backup_current_binary(&self) -> Result<PathBuf> {
        if !self.current_binary.exists() {
            warn!("Current binary not found at {:?}", self.current_binary);
            return Ok(PathBuf::new());
        }

        // Create backup with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("coding-agent_{}", timestamp);
        let backup_path = self.backup_dir.join(&backup_name);

        // Copy current binary to backup
        tokio::fs::copy(&self.current_binary, &backup_path).await?;
        
        info!("Created backup: {:?}", backup_path);

        // Clean up old backups
        self.cleanup_old_backups().await?;

        Ok(backup_path)
    }

    /// Clean up old backups, keeping only the most recent N
    async fn cleanup_old_backups(&self) -> Result<()> {
        let mut entries = tokio::fs::read_dir(&self.backup_dir).await?;
        let mut backups: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.file_name().unwrap_or_default().to_string_lossy().starts_with("coding-agent_") {
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        backups.push((path, modified));
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backups.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove excess backups
        if backups.len() > self.config.backup_count {
            for (path, _) in &backups[self.config.backup_count..] {
                if let Err(e) = tokio::fs::remove_file(path).await {
                    warn!("Failed to remove old backup {:?}: {}", path, e);
                } else {
                    debug!("Removed old backup: {:?}", path);
                }
            }
        }

        Ok(())
    }

    /// Verify that a binary is valid and can be executed
    async fn verify_binary(&self, binary_path: &Path) -> Result<()> {
        info!("Verifying binary: {:?}", binary_path);

        // Check file exists and is executable
        if !binary_path.exists() {
            return Err(Error::Validation(format!(
                "Binary does not exist: {:?}",
                binary_path
            )));
        }

        // Try to get version from binary
        let output = Command::new(binary_path)
            .arg("--version")
            .output()
            .await
            .map_err(|e| Error::Io(e))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            info!("Binary verification successful. Version: {}", version.trim());
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(Error::Validation(format!(
                "Binary verification failed: {}",
                stderr
            )))
        }
    }

    /// Restart the agent with a new binary
    /// 
    /// This function will replace the current binary and restart the process.
    /// Note: This will not return if successful - the process will be replaced.
    #[allow(unreachable_code)]
    pub async fn restart_with_new_binary(&self, new_binary: &Path) -> Result<()> {
        if !self.config.auto_restart {
            info!("Auto-restart is disabled. New binary available at: {:?}", new_binary);
            return Err(Error::Cancelled);
        }

        info!("Restarting with new binary: {:?}", new_binary);

        // Verify the new binary
        if self.config.verify_before_restart {
            self.verify_binary(new_binary).await?;
        }

        // Get current executable path
        let current_exe = std::env::current_exe()
            .map_err(|e| Error::Io(e))?;

        // Copy new binary over current binary
        // On Unix, we can replace a running binary
        #[cfg(unix)]
        {
            tokio::fs::copy(new_binary, &current_exe).await?;
            info!("Replaced current binary with new version");
        }

        #[cfg(windows)]
        {
            // On Windows, we need to rename the old binary first
            let old_binary = current_exe.with_extension("old");
            tokio::fs::rename(&current_exe, &old_binary).await?;
            tokio::fs::copy(new_binary, &current_exe).await?;
            tokio::fs::remove_file(&old_binary).await?;
        }

        // Prepare to restart
        info!("Executing restart...");

        // Get current process arguments
        let args: Vec<String> = std::env::args().collect();

        // Spawn new process
        let mut cmd = std::process::Command::new(&current_exe);
        cmd.args(&args[1..])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        // On Unix, use exec to replace current process
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let error = cmd.exec();
            return Err(Error::Io(error));
        }

        // On Windows, spawn and exit
        #[cfg(windows)]
        {
            let _ = cmd.spawn()?;
            std::process::exit(0);
        }
    }

    /// Rollback to the previous version
    pub async fn rollback(&self) -> Result<()> {
        if !self.config.rollback_on_failure {
            warn!("Rollback is disabled");
            return Err(Error::Config("Rollback is disabled".to_string()));
        }

        info!("Initiating rollback...");

        // Find the most recent backup
        let mut entries = tokio::fs::read_dir(&self.backup_dir).await?;
        let mut backups: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.file_name().unwrap_or_default().to_string_lossy().starts_with("coding-agent_") {
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        backups.push((path, modified));
                    }
                }
            }
        }

        if backups.is_empty() {
            return Err(Error::NotFound("No backups available for rollback".to_string()));
        }

        // Sort by modification time (newest first)
        backups.sort_by(|a, b| b.1.cmp(&a.1));
        let latest_backup = &backups[0].0;

        info!("Rolling back to: {:?}", latest_backup);

        // Replace current binary with backup
        let current_exe = std::env::current_exe()
            .map_err(|e| Error::Io(e))?;

        tokio::fs::copy(latest_backup, &current_exe).await?;

        info!("Rollback completed. Restart to use previous version.");

        Ok(())
    }

    /// Get the list of available backups
    pub async fn list_backups(&self) -> Result<Vec<(PathBuf, std::time::SystemTime)>> {
        let mut entries = tokio::fs::read_dir(&self.backup_dir).await?;
        let mut backups: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.file_name().unwrap_or_default().to_string_lossy().starts_with("coding-agent_") {
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        backups.push((path, modified));
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backups.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(backups)
    }

    /// Shutdown the self-compiler
    pub async fn shutdown(&self) -> Result<()> {
        info!("Self-compiler shutting down");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_self_compiler_creation() {
        let config = agent_config::SelfCompileConfig::default();
        let compiler = SelfCompiler::new(config, "/tmp/test");
        assert!(compiler.is_enabled());
    }
}
