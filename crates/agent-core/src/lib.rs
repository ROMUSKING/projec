//! Core agent module for the coding agent.
//!
//! This crate provides the orchestrator, state management, and
//! the main agent loop that coordinates all other modules.

use common::{async_trait, Error, Module, Result, TaskId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

pub mod improvement;
pub mod orchestrator;
pub mod planning;
pub mod self_compile;
pub mod state;
pub mod evaluation;
pub mod telemetry;
pub mod reporting;
pub mod model_assignment;

use improvement::{ImprovementEngine, PerformanceMetrics, calculate_success_rate, calculate_throughput};
use orchestrator::{Orchestrator, TaskExecutionPipeline};
use self_compile::SelfCompiler;
use evaluation::EvaluationEngine;
use telemetry::TelemetryManager;
use state::{AgentState, Checkpoint, StateManager};

/// Main agent structure
pub struct Agent {
    orchestrator: Arc<RwLock<Orchestrator>>,
    state_manager: Arc<RwLock<StateManager>>,
    improvement_engine: Arc<RwLock<ImprovementEngine>>,
    evaluation_engine: Arc<RwLock<EvaluationEngine>>,
    telemetry_manager: Arc<RwLock<TelemetryManager>>,
    self_compiler: Option<Arc<RwLock<SelfCompiler>>>,
    task_queue: TaskQueue,
    task_relationships: TaskRelationshipTracker,
    metrics: Arc<RwLock<AgentMetrics>>,
    modules: Vec<Box<dyn Module>>,
    config: agent_config::AgentConfig,
    shutdown_tx: Option<mpsc::Sender<()>>,
    event_tx: mpsc::Sender<AgentEvent>,
    event_rx: Arc<RwLock<mpsc::Receiver<AgentEvent>>>,
}

impl Agent {
    /// Create a new agent with the given configuration
    pub fn new(config: agent_config::AgentConfig) -> Self {
        let (event_tx, event_rx) = mpsc::channel(1000);
        
        // Initialize self-compiler if enabled
        let self_compiler = if config.self_compile.enabled {
            let compiler = SelfCompiler::new(
                config.self_compile.clone(),
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            );
            Some(Arc::new(RwLock::new(compiler)))
        } else {
            None
        };
        
        Self {
            orchestrator: Arc::new(RwLock::new(Orchestrator::new())),
            state_manager: Arc::new(RwLock::new(StateManager::new())),
            improvement_engine: Arc::new(RwLock::new(ImprovementEngine::new())),
            evaluation_engine: Arc::new(RwLock::new(EvaluationEngine::new())),
            telemetry_manager: Arc::new(RwLock::new(TelemetryManager::new(config.telemetry.clone()))),
            self_compiler,
            task_queue: TaskQueue::new(),
            task_relationships: TaskRelationshipTracker::new(),
            metrics: Arc::new(RwLock::new(AgentMetrics::default())),
            modules: Vec::new(),
            config,
            shutdown_tx: None,
            event_tx,
            event_rx: Arc::new(RwLock::new(event_rx)),
        }
    }

    /// Register a module with the agent
    pub fn register_module(&mut self, module: Box<dyn Module>) {
        self.modules.push(module);
    }

    /// Set the orchestrator
    pub fn with_orchestrator(mut self, mut orchestrator: Orchestrator) -> Self {
        orchestrator = orchestrator.with_evaluation(self.evaluation_engine.clone());
        self.orchestrator = Arc::new(RwLock::new(orchestrator));
        self
    }

    /// Initialize the agent and all modules
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing agent");

        // Initialize state manager
        self.state_manager.write().await.initialize().await?;

        // Initialize improvement engine
        self.improvement_engine.write().await.initialize().await?;

        // Initialize self-compiler if enabled
        if let Some(compiler) = &self.self_compiler {
            compiler.write().await.initialize().await?;
            info!("Self-compiler initialized");
        }

        // Initialize all modules
        for module in &mut self.modules {
            module.initialize().await?;
            info!("Initialized module: {}", module.name());
        }

        // Initialize orchestrator
        self.orchestrator.write().await.initialize().await?;

        // Load any persisted state
        self.load_state().await?;

        info!("Agent initialization complete");
        Ok(())
    }

    /// Run the agent main loop
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting agent main loop");

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Start the event processor
        let event_rx = Arc::clone(&self.event_rx);
        let event_processor = tokio::spawn(async move {
            let mut rx = event_rx.write().await;
            while let Some(event) = rx.recv().await {
                debug!("Processing agent event: {:?}", event);
                // Events are processed through the main loop
            }
        });

        // Start self-improvement timer if enabled
        let improvement_handle = if self.config.agent.self_improvement.enabled {
            let interval = self.config.agent.improvement_interval;
            let event_tx = self.event_tx.clone();
            Some(tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval));
                loop {
                    interval.tick().await;
                    if event_tx.send(AgentEvent::SelfImprovementTrigger).await.is_err() {
                        break;
                    }
                }
            }))
        } else {
            None
        };

        // Main event loop
        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received");
                    break;
                }

                // Process any pending events
                event = async {
                    let mut rx = self.event_rx.write().await;
                    rx.recv().await
                } => {
                    if let Some(event) = event {
                        self.handle_event(event).await?;
                    }
                }

                // Process tasks based on current state
                _ = self.process_cycle() => {
                    // Small yield to prevent tight loop
                    tokio::task::yield_now().await;
                }
            }
        }

        // Cleanup
        if let Some(handle) = improvement_handle {
            handle.abort();
        }
        event_processor.abort();

        info!("Agent main loop ended");
        Ok(())
    }

    /// Process a single cycle of the agent loop
    async fn process_cycle(&self) -> Result<()> {
        let state = self.state_manager.read().await.current_state();

        match state {
            AgentState::Idle => {
                // Check for pending tasks
                if let Some(task) = self.task_queue.pop_highest_priority().await {
                    info!("Popped task from queue: {:?}", task.id);
                    self.state_manager
                        .write()
                        .await
                        .transition_to(AgentState::Running(task));
                } else {
                    // No tasks, sleep briefly
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }
            }
            AgentState::Running(task) => {
                if let Err(e) = self.process_task(task.clone()).await {
                    error!("Task processing error: {}", e);
                    self.metrics.write().await.record_failure(&task.id, task.parent_id.is_some());
                    self.state_manager
                        .write()
                        .await
                        .transition_to(AgentState::Error(e.to_string()));
                }
            }
            AgentState::Improving => {
                if let Err(e) = self.self_improve().await {
                    error!("Self-improvement error: {}", e);
                    self.state_manager
                        .write()
                        .await
                        .transition_to(AgentState::Error(e.to_string()));
                }
            }
            AgentState::Error(msg) => {
                error!("Agent in error state: {}", msg);
                // Attempt recovery
                if let Err(e) = self.recover_from_error(&msg).await {
                    error!("Recovery failed: {}", e);
                }
                self.state_manager.write().await.transition_to(AgentState::Idle);
            }
            AgentState::ShuttingDown => {
                // Will be handled by main loop
            }
            _ => {
                // Other states are transitional
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }

        Ok(())
    }

    /// Handle agent events
    async fn handle_event(&self, event: AgentEvent) -> Result<()> {
        match event {
            AgentEvent::SelfImprovementTrigger => {
                let state = self.state_manager.read().await.current_state();
                if matches!(state, AgentState::Idle) {
                    info!("Triggering self-improvement cycle");
                    self.state_manager
                        .write()
                        .await
                        .transition_to(AgentState::Improving);
                }
            }
            AgentEvent::TaskSubmitted(task) => {
                self.task_queue.push(task).await;
            }
            AgentEvent::StateChangeRequested(new_state) => {
                self.state_manager.write().await.transition_to(new_state);
            }
            AgentEvent::MetricsRequest(tx) => {
                let metrics = self.metrics.read().await.clone();
                let _ = tx.send(metrics).await;
            }
        }
        Ok(())
    }

    /// Process a single task through the full pipeline
    async fn process_task(&self, task: Task) -> Result<TaskResult> {
        info!("Processing task: {:?}", task.id);
        let start_time = common::chrono::Utc::now();

        // Create checkpoint before task execution
        let checkpoint_id = self
            .state_manager
            .write()
            .await
            .create_checkpoint(&AgentState::Running(task.clone()));
        debug!("Created checkpoint: {}", checkpoint_id);

        // Update metrics
        self.metrics.write().await.record_task_start(&task.id, task.parent_id.is_some());

        // Execute through orchestrator
        let result = self.orchestrator.read().await.process_task(task.clone()).await;

        // Record completion
        let duration = common::chrono::Utc::now()
            .signed_duration_since(start_time)
            .num_milliseconds() as u64;

        match &result {
            Ok(task_result) => {
                info!("Task {:?} completed successfully in {}ms", task.id, duration);
                self.metrics.write().await.record_success(&task.id, duration, task.parent_id.is_some());
                
                // If this is a subtask, mark it as completed in the task relationships tracker
                if let Some(parent_id) = task.parent_id {
                    self.task_relationships.mark_subtask_completed(parent_id, task.id).await;
                    info!("Subtask {:?} completed for parent: {:?}", task.id, parent_id);
                    
                    // Check if all subtasks for the parent are completed
                    if self.task_relationships.are_all_subtasks_completed(parent_id).await {
                        info!("All subtasks completed for parent task: {:?}", parent_id);
                    }
                }
                
                // Update knowledge base with results
                self.update_knowledge(&task, task_result).await?;
            }
            Err(e) => {
                error!("Task {:?} failed: {}", task.id, e);
                self.metrics.write().await.record_failure(&task.id, task.parent_id.is_some());
            }
        }

        // Transition back to idle
        self.state_manager.write().await.transition_to(AgentState::Idle);

        result
    }

    /// Self-improvement routine - the core of the self-developing agent
    async fn self_improve(&self) -> Result<Improvement> {
        info!("Starting self-improvement cycle");

        let mut improvement_engine = self.improvement_engine.write().await;

        // Step 1: Observe - Collect current performance metrics
        let metrics = self.metrics.read().await.clone();
        improvement_engine.record_metrics(metrics).await?;

        // Step 2: Analyze - Identify bottlenecks and improvement opportunities
        let analysis = improvement_engine.analyze_performance().await?;
        info!("Performance analysis: {} bottlenecks identified", analysis.bottlenecks.len());

        // Step 3: Plan - Generate improvement strategies
        let strategies = improvement_engine.generate_strategies(&analysis).await?;
        info!("Generated {} improvement strategies", strategies.len());

        // Step 4: Execute - Test improvements safely
        let mut applied_improvements = Vec::new();
        for strategy in strategies {
            if let Ok(improvement) = improvement_engine.apply_strategy(&strategy).await {
                applied_improvements.push(improvement);
            }
        }

        // Step 5: Validate - A/B test and measure impact
        let validation = improvement_engine.validate_improvements(&applied_improvements).await?;
        
        // Rollback any changes that degraded performance
        for rollback in &validation.rollbacks {
            warn!("Rolling back improvement: {}", rollback.description);
            improvement_engine.rollback(rollback).await?;
        }

        // Persist successful improvements
        improvement_engine.persist_improvements(&validation.successful).await?;

        let improvement_summary = Improvement {
            description: format!(
                "Self-improvement cycle completed. Applied: {}, Successful: {}, Rolled back: {}",
                applied_improvements.len(),
                validation.successful.len(),
                validation.rollbacks.len()
            ),
            changes: validation.successful,
            metrics_before: analysis.metrics_before.map(|m| PerformanceMetrics {
                timestamp: common::chrono::Utc::now(),
                success_rate: calculate_success_rate(&m),
                average_latency_ms: m.average_execution_time_ms,
                throughput: calculate_throughput(&m),
            }),
            metrics_after: analysis.metrics_after.map(|m| PerformanceMetrics {
                timestamp: common::chrono::Utc::now(),
                success_rate: calculate_success_rate(&m),
                average_latency_ms: m.average_execution_time_ms,
                throughput: calculate_throughput(&m),
            }),
        };

        // Transition back to idle
        self.state_manager.write().await.transition_to(AgentState::Idle);

        Ok(improvement_summary)
    }

    /// Recover from an error state
    async fn recover_from_error(&self, error_msg: &str) -> Result<()> {
        warn!("Attempting recovery from error: {}", error_msg);

        // Try to restore from the most recent checkpoint
        let state_manager = self.state_manager.read().await;
        if let Some(latest_checkpoint) = state_manager.get_checkpoints().back() {
            let checkpoint_id = latest_checkpoint.id.clone();
            info!("Restoring from checkpoint: {}", checkpoint_id);
            drop(state_manager);
            self.state_manager
                .write()
                .await
                .restore_checkpoint(&checkpoint_id)?;
        }

        Ok(())
    }

    /// Update knowledge base with task results
    async fn update_knowledge(&self, task: &Task, result: &TaskResult) -> Result<()> {
        // This would integrate with the knowledge module
        debug!("Updating knowledge base with task {:?} results", task.id);
        Ok(())
    }

    /// Load persisted state
    async fn load_state(&self) -> Result<()> {
        debug!("Loading persisted state");
        // TODO: Implement state loading from disk
        Ok(())
    }

    /// Shutdown the agent gracefully
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down agent");

        // Signal shutdown
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(()).await;
        }

        // Transition to shutting down state
        self.state_manager
            .write()
            .await
            .transition_to(AgentState::ShuttingDown);

        // Persist current state
        self.persist_state().await?;

        // Shutdown orchestrator
        self.orchestrator.write().await.shutdown().await?;

        // Shutdown improvement engine
        self.improvement_engine.write().await.shutdown().await?;

        // Shutdown self-compiler if enabled
        if let Some(compiler) = &self.self_compiler {
            compiler.write().await.shutdown().await?;
        }

        // Shutdown all modules in reverse order
        for module in self.modules.iter_mut().rev() {
            module.shutdown().await?;
            info!("Shutdown module: {}", module.name());
        }

        // Shutdown state manager
        self.state_manager.write().await.shutdown().await?;

        // Flush telemetry
        self.telemetry_manager.write().await.flush().await?;

        info!("Agent shutdown complete");
        Ok(())
    }

    /// Persist current state to disk
    async fn persist_state(&self) -> Result<()> {
        debug!("Persisting agent state");
        // TODO: Implement state persistence
        Ok(())
    }

    /// Submit a new task to the agent
    pub async fn submit_task(&self, task: Task) -> Result<TaskId> {
        info!("Task submitted: {:?}", task.id);
        
        // If task has a parent, add to task relationships tracker
        if let Some(parent_id) = task.parent_id {
            self.task_relationships.add_subtask(parent_id, task.id).await;
        }
        
        // If task has subtasks, add them to the tracker
        for &subtask_id in &task.subtasks {
            self.task_relationships.add_subtask(task.id, subtask_id).await;
        }
        
        self.task_queue.push(task.clone()).await;
        let task_id = task.id;
        self.event_tx.send(AgentEvent::TaskSubmitted(task)).await
            .map_err(|_| Error::Internal("Failed to send task event".to_string()))?;
        Ok(task_id)
    }

    /// Submit a subtask to a parent task
    pub async fn submit_subtask(&self, parent_id: TaskId, mut task: Task) -> Result<TaskId> {
        info!("Subtask submitted: {:?} for parent: {:?}", task.id, parent_id);
        
        // Set parent for the subtask
        task.parent_id = Some(parent_id);
        
        // Add to task relationships tracker
        self.task_relationships.add_subtask(parent_id, task.id).await;
        
        self.task_queue.push(task.clone()).await;
        let task_id = task.id;
        self.event_tx.send(AgentEvent::TaskSubmitted(task)).await
            .map_err(|_| Error::Internal("Failed to send task event".to_string()))?;
        Ok(task_id)
    }

    /// Get all subtasks for a parent task
    pub async fn get_subtasks(&self, parent_id: TaskId) -> Vec<TaskId> {
        self.task_relationships.get_subtasks(parent_id).await
    }

    /// Get the parent task for a subtask
    pub async fn get_parent_task(&self, task_id: TaskId) -> Option<TaskId> {
        self.task_relationships.get_parent(task_id).await
    }

    /// Check if all subtasks of a parent task are completed
    pub async fn are_all_subtasks_completed(&self, parent_id: TaskId) -> bool {
        self.task_relationships.are_all_subtasks_completed(parent_id).await
    }

    /// Get completion status of subtasks for a parent task
    pub async fn get_subtask_completion_status(&self, parent_id: TaskId) -> HashMap<TaskId, bool> {
        self.task_relationships.get_subtask_completion_status(parent_id).await
    }

    /// Get current agent metrics
    pub async fn get_metrics(&self) -> AgentMetrics {
        self.metrics.read().await.clone()
    }

    /// Request a metrics snapshot
    pub async fn request_metrics(&self) -> Result<AgentMetrics> {
        let (tx, mut rx) = mpsc::channel(1);
        self.event_tx.send(AgentEvent::MetricsRequest(tx)).await
            .map_err(|_| Error::Internal("Failed to request metrics".to_string()))?;
        
        rx.recv().await
            .ok_or_else(|| Error::Internal("Failed to receive metrics".to_string()))
    }

    /// Get current state
    pub async fn current_state(&self) -> AgentState {
        self.state_manager.read().await.current_state()
    }

    /// Trigger self-improvement manually
    pub async fn trigger_self_improvement(&self) -> Result<()> {
        self.event_tx.send(AgentEvent::SelfImprovementTrigger).await
            .map_err(|_| Error::Internal("Failed to trigger self-improvement".to_string()))
    }

    /// Trigger a cross-evaluation for a completed task
    pub async fn evaluate_task(&self, task: &Task, output: &str) -> Result<evaluation::EvaluationReport> {
        // Need to acquire intelligence engine from orchestrator or store it in Agent
        // For now, we assume we can get it via the orchestrator if it was public, 
        // but since it's private, we might need to expose it or refactor.
        // A better approach for this prototype is to pass the intelligence engine if available.
        // However, `Agent` struct doesn't own `IntelligenceEngine` directly, `Orchestrator` does.
        
        // Let's delegate this to the Orchestrator or just log a placeholder if we can't access it easily
        // without refactoring `Orchestrator`.
        
        // Actually, Orchestrator has `intelligence`. We can add a method there or expose it.
        // For this step, I will add a method to `Orchestrator` to run evaluation.
        
        self.orchestrator.read().await.run_evaluation(task, output).await
    }

    /// Get evaluation dashboard
    pub async fn get_evaluation_dashboard(&self) -> String {
        self.evaluation_engine.read().await.get_dashboard().await
    }

    /// Record a performance metric
    pub async fn record_metric(&self, task_type: &str, model: &str, latency_ms: u64, success: bool) {
        self.telemetry_manager.read().await.record_event(
            task_type,
            model,
            latency_ms,
            success,
            None,
            0
        ).await;
    }

    /// Compile the agent source code
    pub async fn compile(&self) -> Result<PathBuf> {
        if let Some(compiler) = &self.self_compiler {
            let compiler = compiler.read().await;
            compiler.compile().await
        } else {
            Err(Error::Config("Self-compilation is not enabled".to_string()))
        }
    }

    /// Restart the agent with a new binary
    pub async fn restart_with_new_binary(&self, binary_path: PathBuf) -> Result<()> {
        if let Some(compiler) = &self.self_compiler {
            let compiler = compiler.read().await;
            compiler.restart_with_new_binary(&binary_path).await
        } else {
            Err(Error::Config("Self-compilation is not enabled".to_string()))
        }
    }

    /// Rollback to the previous version
    pub async fn rollback(&self) -> Result<()> {
        if let Some(compiler) = &self.self_compiler {
            let compiler = compiler.read().await;
            compiler.rollback().await
        } else {
            Err(Error::Config("Self-compilation is not enabled".to_string()))
        }
    }

    /// List available backups
    pub async fn list_backups(&self) -> Result<Vec<(PathBuf, std::time::SystemTime)>> {
        if let Some(compiler) = &self.self_compiler {
            let compiler = compiler.read().await;
            compiler.list_backups().await
        } else {
            Err(Error::Config("Self-compilation is not enabled".to_string()))
        }
    }

    /// Check if self-compilation is enabled
    pub fn is_self_compile_enabled(&self) -> bool {
        self.self_compiler.is_some()
    }
}

impl Default for Agent {
    fn default() -> Self {
        Self::new(agent_config::AgentConfig::default())
    }
}

/// Agent events for internal communication
#[derive(Debug, Clone)]
pub enum AgentEvent {
    SelfImprovementTrigger,
    TaskSubmitted(Task),
    StateChangeRequested(AgentState),
    MetricsRequest(mpsc::Sender<AgentMetrics>),
}

/// Task definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: TaskId,
    pub description: String,
    pub intent: intelligence::Intent,
    pub context: TaskContext,
    pub priority: TaskPriority,
    pub created_at: common::chrono::DateTime<common::chrono::Utc>,
    pub deadline: Option<common::chrono::DateTime<common::chrono::Utc>>,
    pub dependencies: Vec<TaskId>,
    pub parent_id: Option<TaskId>,
    pub subtasks: Vec<TaskId>,
}

impl Task {
    /// Create a new task
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: TaskId::new(),
            description: description.into(),
            intent: intelligence::Intent {
                category: intelligence::IntentCategory::Unknown,
                confidence: 0.0,
                parameters: Default::default(),
                raw_input: String::new(),
            },
            context: TaskContext::default(),
            priority: TaskPriority::Normal,
            created_at: common::chrono::Utc::now(),
            deadline: None,
            dependencies: Vec::new(),
            parent_id: None,
            subtasks: Vec::new(),
        }
    }

    /// Set the parent task
    pub fn with_parent(mut self, parent_id: TaskId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Add a subtask
    pub fn with_subtask(mut self, subtask_id: TaskId) -> Self {
        self.subtasks.push(subtask_id);
        self
    }

    /// Set the intent
    pub fn with_intent(mut self, intent: intelligence::Intent) -> Self {
        self.intent = intent;
        self
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the context
    pub fn with_context(mut self, context: TaskContext) -> Self {
        self.context = context;
        self
    }

    /// Add a dependency
    pub fn with_dependency(mut self, task_id: TaskId) -> Self {
        self.dependencies.push(task_id);
        self
    }
}

/// Task context
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TaskContext {
    pub workspace_path: Option<std::path::PathBuf>,
    pub files: Vec<std::path::PathBuf>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

/// Task result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub success: bool,
    pub output: String,
    pub artifacts: Vec<Artifact>,
    pub completed_at: common::chrono::DateTime<common::chrono::Utc>,
    pub execution_time_ms: u64,
    pub metrics: TaskExecutionMetrics,
}

/// Task execution metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskExecutionMetrics {
    pub tokens_used: u32,
    pub api_calls: u32,
    pub tools_used: Vec<String>,
    pub retries: u32,
}

/// Task artifact (file, message, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub artifact_type: ArtifactType,
    pub content: String,
    pub path: Option<std::path::PathBuf>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Artifact types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    File,
    Message,
    Diff,
    Log,
    TestResult,
    Analysis,
}

/// Self-improvement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Improvement {
    pub description: String,
    pub changes: Vec<Change>,
    pub metrics_before: Option<PerformanceMetrics>,
    pub metrics_after: Option<PerformanceMetrics>,
}

/// Individual change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: String,
    pub file_path: std::path::PathBuf,
    pub change_type: ChangeType,
    pub description: String,
    pub timestamp: common::chrono::DateTime<common::chrono::Utc>,
    pub rollback_data: Option<serde_json::Value>,
}

/// Types of changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Configuration,
    Prompt,
    Strategy,
}

/// Task relationship tracker to manage parent-child relationships
#[derive(Debug, Default)]
pub struct TaskRelationshipTracker {
    /// Map of task IDs to their subtask IDs
    task_subtasks: Arc<RwLock<HashMap<TaskId, Vec<TaskId>>>>,
    /// Map of subtask IDs to their parent task ID
    subtask_parent: Arc<RwLock<HashMap<TaskId, TaskId>>>,
    /// Map of task IDs to their subtask completion status
    subtask_completion: Arc<RwLock<HashMap<TaskId, HashMap<TaskId, bool>>>>,
}

impl TaskRelationshipTracker {
    /// Create a new task relationship tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a subtask to a parent task
    pub async fn add_subtask(&self, parent_id: TaskId, subtask_id: TaskId) {
        let mut task_subtasks = self.task_subtasks.write().await;
        let mut subtask_parent = self.subtask_parent.write().await;
        let mut subtask_completion = self.subtask_completion.write().await;

        task_subtasks.entry(parent_id).or_insert_with(Vec::new).push(subtask_id);
        subtask_parent.insert(subtask_id, parent_id);
        subtask_completion.entry(parent_id).or_insert_with(HashMap::new).insert(subtask_id, false);
    }

    /// Get all subtasks for a parent task
    pub async fn get_subtasks(&self, parent_id: TaskId) -> Vec<TaskId> {
        self.task_subtasks.read().await.get(&parent_id).cloned().unwrap_or_else(Vec::new)
    }

    /// Get the parent task for a subtask
    pub async fn get_parent(&self, subtask_id: TaskId) -> Option<TaskId> {
        self.subtask_parent.read().await.get(&subtask_id).cloned()
    }

    /// Mark a subtask as completed
    pub async fn mark_subtask_completed(&self, parent_id: TaskId, subtask_id: TaskId) {
        if let Some(completion_map) = self.subtask_completion.write().await.get_mut(&parent_id) {
            completion_map.insert(subtask_id, true);
        }
    }

    /// Check if all subtasks of a parent task are completed
    pub async fn are_all_subtasks_completed(&self, parent_id: TaskId) -> bool {
        if let Some(completion_map) = self.subtask_completion.read().await.get(&parent_id) {
            completion_map.values().all(|&completed| completed)
        } else {
            // No subtasks means all are completed
            true
        }
    }

    /// Get completion status of subtasks for a parent task
    pub async fn get_subtask_completion_status(&self, parent_id: TaskId) -> HashMap<TaskId, bool> {
        self.subtask_completion.read().await.get(&parent_id).cloned().unwrap_or_else(HashMap::new)
    }

    /// Remove a task and all its subtasks from the tracker
    pub async fn remove_task(&self, task_id: TaskId) {
        let mut task_subtasks = self.task_subtasks.write().await;
        let mut subtask_parent = self.subtask_parent.write().await;
        let mut subtask_completion = self.subtask_completion.write().await;

        // Remove all subtasks of the task
        if let Some(subtasks) = task_subtasks.remove(&task_id) {
            for subtask_id in subtasks {
                subtask_parent.remove(&subtask_id);
            }
        }

        // If this task is a subtask, remove it from its parent's subtask list
        if let Some(parent_id) = subtask_parent.remove(&task_id) {
            if let Some(subtasks) = task_subtasks.get_mut(&parent_id) {
                subtasks.retain(|&id| id != task_id);
            }
            if let Some(completion_map) = subtask_completion.get_mut(&parent_id) {
                completion_map.remove(&task_id);
            }
        }

        subtask_completion.remove(&task_id);
    }
}

/// Task queue with prioritization
pub struct TaskQueue {
    inner: Arc<RwLock<VecDeque<Task>>>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Push a task to the queue
    pub async fn push(&self, task: Task) {
        let mut queue = self.inner.write().await;
        queue.push_back(task);
        // Sort by priority (highest first) and then by creation time
        let mut tasks: Vec<_> = queue.drain(..).collect();
        tasks.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });
        queue.extend(tasks);
    }

    /// Pop the highest priority task
    pub async fn pop_highest_priority(&self) -> Option<Task> {
        let mut queue = self.inner.write().await;
        queue.pop_front()
    }

    /// Get queue length
    pub async fn len(&self) -> usize {
        self.inner.read().await.len()
    }

    /// Check if queue is empty
    pub async fn is_empty(&self) -> bool {
        self.inner.read().await.is_empty()
    }

    /// Get all pending tasks
    pub async fn get_pending(&self) -> Vec<Task> {
        self.inner.read().await.iter().cloned().collect()
    }

    /// Remove a task by ID
    pub async fn remove(&self, task_id: &TaskId) -> Option<Task> {
        let mut queue = self.inner.write().await;
        if let Some(pos) = queue.iter().position(|t| t.id == *task_id) {
            queue.remove(pos)
        } else {
            None
        }
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent-wide metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub tasks_submitted: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub subtasks_submitted: u64,
    pub subtasks_completed: u64,
    pub subtasks_failed: u64,
    pub total_execution_time_ms: u64,
    pub average_execution_time_ms: u64,
    pub success_rate: f64,
    pub improvements_applied: u64,
    pub improvements_rolled_back: u64,
    pub start_time: common::chrono::DateTime<common::chrono::Utc>,
    task_latencies: Vec<u64>,
}

impl AgentMetrics {
    pub fn new() -> Self {
        Self {
            start_time: common::chrono::Utc::now(),
            ..Default::default()
        }
    }

    /// Record task start
    pub fn record_task_start(&mut self, _task_id: &TaskId, is_subtask: bool) {
        if is_subtask {
            self.subtasks_submitted += 1;
        } else {
            self.tasks_submitted += 1;
        }
    }

    /// Record successful task completion
    pub fn record_success(&mut self, _task_id: &TaskId, duration_ms: u64, is_subtask: bool) {
        if is_subtask {
            self.subtasks_completed += 1;
        } else {
            self.tasks_completed += 1;
        }
        
        self.total_execution_time_ms += duration_ms;
        self.task_latencies.push(duration_ms);
        
        // Keep only last 100 latencies for average calculation
        if self.task_latencies.len() > 100 {
            self.task_latencies.remove(0);
        }
        
        self.recalculate_stats();
    }

    /// Record task failure
    pub fn record_failure(&mut self, _task_id: &TaskId, is_subtask: bool) {
        if is_subtask {
            self.subtasks_failed += 1;
        } else {
            self.tasks_failed += 1;
        }
        self.recalculate_stats();
    }

    /// Record improvement applied
    pub fn record_improvement_applied(&mut self) {
        self.improvements_applied += 1;
    }

    /// Record improvement rollback
    pub fn record_improvement_rollback(&mut self) {
        self.improvements_rolled_back += 1;
    }

    fn recalculate_stats(&mut self) {
        let total = self.tasks_completed + self.tasks_failed;
        if total > 0 {
            self.success_rate = self.tasks_completed as f64 / total as f64;
        }
        
        if !self.task_latencies.is_empty() {
            let sum: u64 = self.task_latencies.iter().sum();
            self.average_execution_time_ms = sum / self.task_latencies.len() as u64;
        }
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> i64 {
        common::chrono::Utc::now()
            .signed_duration_since(self.start_time)
            .num_seconds()
    }
}

// Re-export dependencies for convenience
pub use intelligence;
pub use analysis;
pub use knowledge;
pub use tools;
pub use agent_config as config;
pub use common;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::Critical > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Normal);
        assert!(TaskPriority::Normal > TaskPriority::Low);
    }

    #[test]
    fn test_agent_metrics() {
        let mut metrics = AgentMetrics::new();
        
        metrics.record_task_start(&TaskId::new(), false);
        metrics.record_success(&TaskId::new(), 100, false);
        
        assert_eq!(metrics.tasks_submitted, 1);
        assert_eq!(metrics.tasks_completed, 1);
        assert_eq!(metrics.success_rate, 1.0);
        
        metrics.record_task_start(&TaskId::new(), false);
        metrics.record_failure(&TaskId::new(), false);
        assert_eq!(metrics.success_rate, 0.5);
    }
}
