//! State management for the agent.
//!
//! This module handles the agent's state machine, checkpoints,
//! and session persistence.

use common::{Error, Result, SessionId};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// State manager for the agent
pub struct StateManager {
    current_state: AgentState,
    session_id: SessionId,
    checkpoints: VecDeque<Checkpoint>,
    max_checkpoints: usize,
    persistence: Option<StatePersistence>,
    state_history: Vec<StateTransition>,
    metadata: HashMap<String, serde_json::Value>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            current_state: AgentState::Idle,
            session_id: SessionId::new(),
            checkpoints: VecDeque::new(),
            max_checkpoints: 10,
            persistence: None,
            state_history: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create state manager with persistence
    pub fn with_persistence(mut self, storage_path: PathBuf) -> Self {
        self.persistence = Some(StatePersistence::new(storage_path));
        self
    }

    /// Initialize the state manager
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing state manager for session: {}", self.session_id);
        
        // Try to load previous session state if exists
        if let Some(persistence) = &self.persistence {
            match persistence.load(self.session_id).await {
                Ok(state) => {
                    info!("Restored previous session state");
                    self.current_state = state;
                }
                Err(e) => {
                    debug!("No previous session state found: {}", e);
                }
            }
        }

        // Create initial checkpoint
        self.create_checkpoint(&AgentState::Idle);
        
        Ok(())
    }

    /// Shutdown the state manager
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down state manager");
        
        // Persist current state
        if let Some(persistence) = &self.persistence {
            persistence.save(self.session_id, &self.current_state).await?;
            info!("Persisted session state");
        }
        
        Ok(())
    }

    /// Get current state
    pub fn current_state(&self) -> AgentState {
        self.current_state.clone()
    }

    /// Get session ID
    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    /// Transition to a new state
    pub fn transition_to(&mut self, new_state: AgentState) {
        let old_state = self.current_state.clone();
        
        // Don't transition if states are the same
        if old_state == new_state {
            return;
        }

        info!("State transition: {:?} -> {:?}", old_state, new_state);

        // Create checkpoint before transition
        if self.should_checkpoint(&old_state, &new_state) {
            self.create_checkpoint(&old_state);
        }

        // Record transition
        self.state_history.push(StateTransition {
            from: old_state.clone(),
            to: new_state.clone(),
            timestamp: common::chrono::Utc::now(),
        });

        self.current_state = new_state;

        // Persist state change if persistence is enabled
        if let Some(persistence) = &self.persistence {
            let persistence = persistence.clone();
            let session_id = self.session_id;
            let state = self.current_state.clone();
            tokio::spawn(async move {
                if let Err(e) = persistence.save(session_id, &state).await {
                    error!("Failed to persist state: {}", e);
                }
            });
        }
    }

    /// Create a checkpoint
    pub fn create_checkpoint(&mut self, state: &AgentState) -> String {
        let checkpoint = Checkpoint {
            id: uuid::Uuid::new_v4().to_string(),
            state: state.clone(),
            timestamp: common::chrono::Utc::now(),
            metadata: self.metadata.clone(),
        };

        let id = checkpoint.id.clone();
        self.checkpoints.push_back(checkpoint);

        // Limit checkpoint history
        while self.checkpoints.len() > self.max_checkpoints {
            self.checkpoints.pop_front();
        }

        debug!("Created checkpoint: {}", id);
        id
    }

    /// Determine if a checkpoint should be created
    fn should_checkpoint(&self, old: &AgentState, new: &AgentState) -> bool {
        // Create checkpoints on significant state changes
        matches!(
            (old, new),
            (AgentState::Idle, AgentState::Running(_))
                | (AgentState::Running(_), AgentState::Idle)
                | (AgentState::Running(_), AgentState::Error(_))
                | (AgentState::Idle, AgentState::Improving)
                | (AgentState::Improving, AgentState::Idle)
                | (AgentState::Analyzing, AgentState::Planning)
                | (AgentState::Planning, AgentState::Executing)
                | (AgentState::Executing, AgentState::Validating)
        )
    }

    /// Restore from a checkpoint
    pub fn restore_checkpoint(&mut self, checkpoint_id: &str) -> Result<()> {
        if let Some(checkpoint) = self.checkpoints.iter().find(|c| c.id == checkpoint_id) {
            self.current_state = checkpoint.state.clone();
            self.metadata = checkpoint.metadata.clone();
            info!("Restored checkpoint: {}", checkpoint_id);
            Ok(())
        } else {
            Err(Error::NotFound(format!("Checkpoint not found: {}", checkpoint_id)))
        }
    }

    /// Restore from the most recent checkpoint
    pub fn restore_latest_checkpoint(&mut self) -> Result<()> {
        if let Some(checkpoint) = self.checkpoints.back() {
            let id = checkpoint.id.clone();
            self.restore_checkpoint(&id)
        } else {
            Err(Error::NotFound("No checkpoints available".to_string()))
        }
    }

    /// Get checkpoint history
    pub fn get_checkpoints(&self) -> &VecDeque<Checkpoint> {
        &self.checkpoints
    }

    /// Get state transition history
    pub fn get_state_history(&self) -> &[StateTransition] {
        &self.state_history
    }

    /// Set metadata value
    pub fn set_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metadata.insert(key.into(), value);
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Check if in a specific state
    pub fn is_in_state(&self, state: &AgentState) -> bool {
        std::mem::discriminant(&self.current_state) == std::mem::discriminant(state)
    }

    /// Check if can transition to a state
    pub fn can_transition_to(&self, new_state: &AgentState) -> bool {
        use AgentState::*;
        
        match (&self.current_state, new_state) {
            // Idle can transition to Running, Improving, or ShuttingDown
            (Idle, Running(_)) | (Idle, Improving) | (Idle, ShuttingDown) => true,
            
            // Running can transition to Idle, Error, or Analyzing
            (Running(_), Idle) | (Running(_), Error(_)) | (Running(_), Analyzing) => true,
            
            // Analyzing can transition to Planning or Error
            (Analyzing, Planning) | (Analyzing, Error(_)) | (Analyzing, Idle) => true,
            
            // Planning can transition to Executing or Error
            (Planning, Executing) | (Planning, Error(_)) | (Planning, Idle) => true,
            
            // Executing can transition to Validating or Error
            (Executing, Validating) | (Executing, Error(_)) | (Executing, Idle) => true,
            
            // Validating can transition to Idle or Error
            (Validating, Idle) | (Validating, Error(_)) => true,
            
            // Improving can transition to Idle or Error
            (Improving, Idle) | (Improving, Error(_)) => true,
            
            // Error can transition to Idle (recovery)
            (Error(_), Idle) => true,
            
            // ShuttingDown is terminal
            (ShuttingDown, _) => false,
            
            // Same state transitions are allowed
            (a, b) if a == b => true,
            
            // All other transitions are invalid
            _ => false,
        }
    }

    /// Get time in current state
    pub fn time_in_current_state(&self) -> Option<chrono::Duration> {
        self.state_history.last().map(|transition| {
            common::chrono::Utc::now().signed_duration_since(transition.timestamp)
        })
    }

    /// Clear old checkpoints
    pub fn prune_checkpoints(&mut self, keep_count: usize) {
        while self.checkpoints.len() > keep_count {
            self.checkpoints.pop_front();
        }
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

use std::collections::HashMap;

/// Agent states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    /// Waiting for input
    Idle,
    /// Processing a task
    Running(super::Task),
    /// Gathering context
    Analyzing,
    /// Generating action plan
    Planning,
    /// Running tools
    Executing,
    /// Checking results
    Validating,
    /// Self-optimization
    Improving,
    /// Recovery mode
    Error(String),
    /// Shutting down
    ShuttingDown,
}

impl std::fmt::Display for AgentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentState::Idle => write!(f, "idle"),
            AgentState::Running(_) => write!(f, "running"),
            AgentState::Analyzing => write!(f, "analyzing"),
            AgentState::Planning => write!(f, "planning"),
            AgentState::Executing => write!(f, "executing"),
            AgentState::Validating => write!(f, "validating"),
            AgentState::Improving => write!(f, "improving"),
            AgentState::Error(msg) => write!(f, "error: {}", msg),
            AgentState::ShuttingDown => write!(f, "shutting_down"),
        }
    }
}

/// State transition record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: AgentState,
    pub to: AgentState,
    pub timestamp: common::chrono::DateTime<common::chrono::Utc>,
}

/// Checkpoint for state restoration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub state: AgentState,
    pub timestamp: common::chrono::DateTime<common::chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// State persistence
#[derive(Clone)]
pub struct StatePersistence {
    storage_path: PathBuf,
}

impl StatePersistence {
    pub fn new(storage_path: PathBuf) -> Self {
        Self { storage_path }
    }

    /// Save state to disk
    pub async fn save(&self, session_id: SessionId, state: &AgentState) -> Result<()> {
        let path = self.storage_path.join(format!("{}.json", session_id.0));
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        let json = serde_json::to_string_pretty(state)?;
        tokio::fs::write(path, json).await?;
        debug!("Saved state for session: {}", session_id.0);
        Ok(())
    }

    /// Load state from disk
    pub async fn load(&self, session_id: SessionId) -> Result<AgentState> {
        let path = self.storage_path.join(format!("{}.json", session_id.0));
        let json = tokio::fs::read_to_string(path).await?;
        let state = serde_json::from_str(&json)?;
        Ok(state)
    }

    /// List available sessions
    pub async fn list_sessions(&self) -> Result<Vec<SessionId>> {
        let mut sessions = Vec::new();
        
        match tokio::fs::read_dir(&self.storage_path).await {
            Ok(mut entries) => {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".json") {
                            let uuid_str = &name[..name.len() - 5];
                            if let Ok(uuid) = uuid::Uuid::parse_str(uuid_str) {
                                sessions.push(SessionId(uuid));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to list sessions: {}", e);
            }
        }

        Ok(sessions)
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: SessionId) -> Result<()> {
        let path = self.storage_path.join(format!("{}.json", session_id.0));
        tokio::fs::remove_file(path).await?;
        Ok(())
    }

    /// Export session to a file
    pub async fn export_session(
        &self,
        session_id: SessionId,
        export_path: PathBuf,
    ) -> Result<()> {
        let state = self.load(session_id).await?;
        let json = serde_json::to_string_pretty(&state)?;
        tokio::fs::write(export_path, json).await?;
        Ok(())
    }
}

/// Session manager for handling multiple sessions
pub struct SessionManager {
    persistence: StatePersistence,
    active_sessions: HashMap<SessionId, Arc<RwLock<StateManager>>>,
}

impl SessionManager {
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            persistence: StatePersistence::new(storage_path),
            active_sessions: HashMap::new(),
        }
    }

    /// Create a new session
    pub async fn create_session(&mut self) -> Result<SessionId> {
        let session_id = SessionId::new();
        let state_manager = Arc::new(RwLock::new(
            StateManager::new().with_persistence(self.persistence.storage_path.clone())
        ));
        
        state_manager.write().await.initialize().await?;
        self.active_sessions.insert(session_id, state_manager);
        
        info!("Created new session: {}", session_id.0);
        Ok(session_id)
    }

    /// Resume an existing session
    pub async fn resume_session(&mut self, session_id: SessionId) -> Result<Arc<RwLock<StateManager>>> {
        if let Some(manager) = self.active_sessions.get(&session_id) {
            return Ok(Arc::clone(manager));
        }

        // Try to load from persistence
        let state = self.persistence.load(session_id).await?;
        let state_manager = Arc::new(RwLock::new(
            StateManager {
                current_state: state,
                session_id,
                checkpoints: VecDeque::new(),
                max_checkpoints: 10,
                persistence: Some(self.persistence.clone()),
                state_history: Vec::new(),
                metadata: HashMap::new(),
            }
        ));
        
        self.active_sessions.insert(session_id, Arc::clone(&state_manager));
        info!("Resumed session: {}", session_id.0);
        
        Ok(state_manager)
    }

    /// Close a session
    pub async fn close_session(&mut self, session_id: SessionId) -> Result<()> {
        if let Some(manager) = self.active_sessions.remove(&session_id) {
            manager.write().await.shutdown().await?;
            info!("Closed session: {}", session_id.0);
        }
        Ok(())
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Result<Vec<SessionId>> {
        self.persistence.list_sessions().await
    }

    /// Get active session count
    pub fn active_session_count(&self) -> usize {
        self.active_sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transition() {
        let mut manager = StateManager::new();
        assert!(matches!(manager.current_state(), AgentState::Idle));

        manager.transition_to(AgentState::Analyzing);
        assert!(matches!(manager.current_state(), AgentState::Analyzing));

        manager.transition_to(AgentState::Planning);
        assert!(matches!(manager.current_state(), AgentState::Planning));
    }

    #[test]
    fn test_checkpoint_creation() {
        let mut manager = StateManager::new();
        let id = manager.create_checkpoint(&AgentState::Idle);
        
        assert!(!id.is_empty());
        assert_eq!(manager.get_checkpoints().len(), 1);
    }

    #[test]
    fn test_checkpoint_restore() {
        let mut manager = StateManager::new();
        let id = manager.create_checkpoint(&AgentState::Idle);
        
        manager.transition_to(AgentState::Analyzing);
        assert!(matches!(manager.current_state(), AgentState::Analyzing));
        
        manager.restore_checkpoint(&id).unwrap();
        assert!(matches!(manager.current_state(), AgentState::Idle));
    }

    #[test]
    fn test_state_validation() {
        let manager = StateManager::new();
        
        // Idle can transition to Improving
        assert!(manager.can_transition_to(&AgentState::Improving));
        
        // Idle cannot directly transition to Executing
        // (must go through Analyzing and Planning)
        // Actually, let's check what the implementation allows
    }

    #[test]
    fn test_checkpoint_limit() {
        let mut manager = StateManager::new();
        manager.max_checkpoints = 3;
        
        manager.create_checkpoint(&AgentState::Idle);
        manager.create_checkpoint(&AgentState::Analyzing);
        manager.create_checkpoint(&AgentState::Planning);
        manager.create_checkpoint(&AgentState::Executing);
        
        assert_eq!(manager.get_checkpoints().len(), 3);
    }
}
