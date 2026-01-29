//! Hierarchical Task Network (HTN) Planning System
//!
//! This module implements HTN planning for the orchestrator, enabling
//! complex multi-step task decomposition and execution with replanning.
//!
//! Inspired by:
//! - HTN planning from AI research
//! - SWE-agent's planning approach
//! - Claude Code's task decomposition
//!
//! Key features:
//! - Goal decomposition into primitive tasks
//! - Method selection based on world state
//! - Dynamic replanning on failure
//! - Plan validation and optimization

use crate::orchestrator::Orchestrator;
use common::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, info, warn};

/// A goal that needs to be achieved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    /// Goal identifier
    pub id: String,
    /// Goal description
    pub description: String,
    /// Goal type
    pub goal_type: GoalType,
    /// Parameters for the goal
    pub parameters: HashMap<String, serde_json::Value>,
    /// Priority (higher = more important)
    pub priority: u32,
    /// Deadline if any
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

/// Types of goals
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalType {
    /// Generate code
    CodeGeneration,
    /// Modify existing code
    CodeModification,
    /// Analyze code
    Analysis,
    /// Run tests
    Testing,
    /// Generate documentation
    Documentation,
    /// Optimize performance
    Optimization,
    /// Fix bugs
    BugFix,
    /// Refactor code
    Refactoring,
    /// Custom goal type
    Custom(String),
}

/// A task in the HTN (can be primitive or compound)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Task {
    /// Primitive task - directly executable
    Primitive(PrimitiveTask),
    /// Compound task - needs decomposition
    Compound(CompoundTask),
}

/// A primitive (atomic) task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveTask {
    /// Task identifier
    pub id: String,
    /// Task name
    pub name: String,
    /// Task description
    pub description: String,
    /// Tool to use (if any)
    pub tool: Option<String>,
    /// Tool arguments
    pub tool_args: serde_json::Value,
    /// Preconditions that must be satisfied
    pub preconditions: Vec<Condition>,
    /// Expected effects
    pub effects: Vec<Effect>,
    /// Estimated cost (time, tokens, etc.)
    pub estimated_cost: Cost,
}

/// A compound (decomposable) task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundTask {
    /// Task identifier
    pub id: String,
    /// Task name
    pub name: String,
    /// Task description
    pub description: String,
    /// Available decomposition methods
    pub methods: Vec<Method>,
}

/// A method for decomposing a compound task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    /// Method name
    pub name: String,
    /// Preconditions for this method to be applicable
    pub preconditions: Vec<Condition>,
    /// Subtasks to execute
    pub subtasks: Vec<Task>,
    /// Constraints on subtask ordering
    pub ordering: OrderingConstraint,
    /// Method cost
    pub cost: Cost,
}

/// Ordering constraints for subtasks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderingConstraint {
    /// Sequential - execute in order
    Sequential,
    /// Parallel - can execute simultaneously
    Parallel,
    /// Partial order - some constraints specified
    Partial(Vec<(String, String)>), // (before, after) pairs
}

/// A condition that must be satisfied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Condition type
    pub condition_type: ConditionType,
    /// Condition expression
    pub expression: String,
}

/// Types of conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    /// File exists
    FileExists,
    /// Directory exists
    DirectoryExists,
    /// Tool available
    ToolAvailable,
    /// State predicate
    StatePredicate,
    /// Custom condition
    Custom,
}

/// An effect of executing a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    /// Effect type
    pub effect_type: EffectType,
    /// What is affected
    pub target: String,
    /// New value/state
    pub value: serde_json::Value,
}

/// Types of effects
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectType {
    /// Create file
    CreateFile,
    /// Modify file
    ModifyFile,
    /// Delete file
    DeleteFile,
    /// Update state
    UpdateState,
    /// Side effect
    SideEffect,
}

/// Cost of executing a task or method
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Cost {
    /// Time cost in milliseconds (estimated)
    pub time_ms: u64,
    /// Token cost (for LLM operations)
    pub tokens: u32,
    /// API calls
    pub api_calls: u32,
    /// Monetary cost
    pub monetary: f64,
}

impl Cost {
    /// Add two costs together
    pub fn add(&self, other: &Cost) -> Cost {
        Cost {
            time_ms: self.time_ms + other.time_ms,
            tokens: self.tokens + other.tokens,
            api_calls: self.api_calls + other.api_calls,
            monetary: self.monetary + other.monetary,
        }
    }

    /// Calculate total weighted cost
    pub fn weighted_score(&self) -> f64 {
        let time_weight = 0.3;
        let token_weight = 0.4;
        let api_weight = 0.2;
        let monetary_weight = 0.1;

        time_weight * self.time_ms as f64 / 1000.0
            + token_weight * self.tokens as f64 / 1000.0
            + api_weight * self.api_calls as f64
            + monetary_weight * self.monetary * 100.0
    }
}

/// A plan - ordered sequence of primitive tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// Plan identifier
    pub id: String,
    /// Tasks to execute (in order)
    pub tasks: Vec<PrimitiveTask>,
    /// Total estimated cost
    pub total_cost: Cost,
    /// Plan metadata
    pub metadata: PlanMetadata,
}

/// Plan metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMetadata {
    /// When the plan was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Goal this plan achieves
    pub goal_id: String,
    /// Planning algorithm used
    pub algorithm: String,
    /// Number of backtracks during planning
    pub backtracks: u32,
}

/// World state for planning
#[derive(Debug, Clone, Default)]
pub struct WorldState {
    /// File system state
    pub files: HashSet<String>,
    /// Directory state
    pub directories: HashSet<String>,
    /// Available tools
    pub available_tools: HashSet<String>,
    /// Custom state predicates
    pub predicates: HashMap<String, bool>,
}

impl WorldState {
    /// Check if a condition is satisfied
    pub fn satisfies(&self, condition: &Condition) -> bool {
        match condition.condition_type {
            ConditionType::FileExists => {
                self.files.contains(&condition.expression)
            }
            ConditionType::DirectoryExists => {
                self.directories.contains(&condition.expression)
            }
            ConditionType::ToolAvailable => {
                self.available_tools.contains(&condition.expression)
            }
            ConditionType::StatePredicate => {
                self.predicates.get(&condition.expression).copied().unwrap_or(false)
            }
            ConditionType::Custom => {
                // Custom conditions always pass in basic implementation
                true
            }
        }
    }

    /// Apply an effect to update the state
    pub fn apply_effect(&mut self, effect: &Effect) {
        match effect.effect_type {
            EffectType::CreateFile => {
                self.files.insert(effect.target.clone());
            }
            EffectType::ModifyFile => {
                self.files.insert(effect.target.clone());
            }
            EffectType::DeleteFile => {
                self.files.remove(&effect.target);
            }
            EffectType::UpdateState => {
                if let Ok(value) = serde_json::from_value::<bool>(effect.value.clone()) {
                    self.predicates.insert(effect.target.clone(), value);
                }
            }
            EffectType::SideEffect => {
                // Side effects don't change world state
            }
        }
    }
}

/// HTN Planner
pub struct HTNPlanner {
    /// Task library - known compound tasks and their methods
    task_library: HashMap<String, CompoundTask>,
    /// Maximum planning depth
    max_depth: usize,
    /// Maximum backtracks allowed
    max_backtracks: u32,
}

impl HTNPlanner {
    /// Create a new HTN planner
    pub fn new() -> Self {
        let mut planner = Self {
            task_library: HashMap::new(),
            max_depth: 10,
            max_backtracks: 100,
        };
        planner.initialize_default_tasks();
        planner
    }

    /// Initialize default compound tasks
    fn initialize_default_tasks(&mut self) {
        // Add default task decompositions here
        // For example, "RefactorCode" might decompose into:
        // 1. Analyze code
        // 2. Identify refactoring opportunities
        // 3. Apply refactoring
        // 4. Verify changes
    }

    /// Register a compound task
    pub fn register_task(&mut self, task: CompoundTask) {
        self.task_library.insert(task.name.clone(), task);
    }

    /// Create a plan to achieve a goal
    pub async fn plan(
        &self,
        goal: &Goal,
        initial_state: &WorldState,
        available_tools: &[String],
    ) -> Result<Plan> {
        info!("Planning for goal: {}", goal.description);

        let mut state = initial_state.clone();
        for tool in available_tools {
            state.available_tools.insert(tool.clone());
        }

        let root_task = self.goal_to_task(goal);
        let mut plan_builder = PlanBuilder::new(goal.id.clone());

        match self.decompose_task(&root_task, &state, &mut plan_builder, 0).await {
            Ok(()) => {
                let plan = plan_builder.build();
                info!("Plan created with {} tasks", plan.tasks.len());
                Ok(plan)
            }
            Err(e) => {
                warn!("Planning failed: {}", e);
                Err(e)
            }
        }
    }

    /// Convert a goal to a task
    fn goal_to_task(&self, goal: &Goal) -> Task {
        // Map goal types to compound tasks
        let task_name = match &goal.goal_type {
            GoalType::CodeGeneration => "generate_code",
            GoalType::CodeModification => "modify_code",
            GoalType::Analysis => "analyze_code",
            GoalType::Testing => "run_tests",
            GoalType::Documentation => "generate_docs",
            GoalType::Optimization => "optimize_code",
            GoalType::BugFix => "fix_bug",
            GoalType::Refactoring => "refactor_code",
            GoalType::Custom(name) => name.as_str(),
        };

        if let Some(compound) = self.task_library.get(task_name) {
            Task::Compound(compound.clone())
        } else {
            // Create a primitive task as fallback
            Task::Primitive(PrimitiveTask {
                id: format!("task_{}", goal.id),
                name: task_name.to_string(),
                description: goal.description.clone(),
                tool: None,
                tool_args: serde_json::json!({}),
                preconditions: vec![],
                effects: vec![],
                estimated_cost: Cost::default(),
            })
        }
    }

    /// Decompose a task into primitive tasks
    async fn decompose_task(
        &self,
        task: &Task,
        state: &WorldState,
        builder: &mut PlanBuilder,
        depth: usize,
    ) -> Result<()> {
        if depth > self.max_depth {
            return Err(common::Error::Internal(
                "Maximum planning depth exceeded".to_string()
            ));
        }

        match task {
            Task::Primitive(primitive) => {
                // Check preconditions
                for condition in &primitive.preconditions {
                    if !state.satisfies(condition) {
                        return Err(common::Error::Validation(
                            format!("Precondition not satisfied: {:?}", condition)
                        ));
                    }
                }
                builder.add_task(primitive.clone());
                Ok(())
            }
            Task::Compound(compound) => {
                // Find applicable method
                let method = self.select_method(compound, state).ok_or_else(|| {
                    common::Error::Internal(
                        format!("No applicable method for task: {}", compound.name)
                    )
                })?;

                // Decompose subtasks
                for subtask in &method.subtasks {
                    Box::pin(self.decompose_task(subtask, state, builder, depth + 1)).await?;
                }

                Ok(())
            }
        }
    }

    /// Select the best applicable method for a compound task
    fn select_method(&self, task: &CompoundTask, state: &WorldState) -> Option<Method> {
        let mut best_method: Option<Method> = None;
        let mut best_cost = f64::MAX;

        for method in &task.methods {
            // Check if preconditions are satisfied
            let applicable = method.preconditions.iter().all(|c| state.satisfies(c));

            if applicable {
                let cost = method.cost.weighted_score();
                if cost < best_cost {
                    best_cost = cost;
                    best_method = Some(method.clone());
                }
            }
        }

        best_method
    }
}

impl Default for HTNPlanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing plans
struct PlanBuilder {
    goal_id: String,
    tasks: Vec<PrimitiveTask>,
    total_cost: Cost,
    backtracks: u32,
}

impl PlanBuilder {
    fn new(goal_id: String) -> Self {
        Self {
            goal_id,
            tasks: vec![],
            total_cost: Cost::default(),
            backtracks: 0,
        }
    }

    fn add_task(&mut self, task: PrimitiveTask) {
        self.total_cost = self.total_cost.add(&task.estimated_cost);
        self.tasks.push(task);
    }

    fn build(self) -> Plan {
        Plan {
            id: format!("plan_{}", uuid::Uuid::new_v4()),
            tasks: self.tasks,
            total_cost: self.total_cost,
            metadata: PlanMetadata {
                created_at: chrono::Utc::now(),
                goal_id: self.goal_id,
                algorithm: "htn".to_string(),
                backtracks: self.backtracks,
            },
        }
    }
}

/// Plan executor with replanning capabilities
pub struct PlanExecutor {
    planner: HTNPlanner,
    max_retries: u32,
}

impl PlanExecutor {
    /// Create a new plan executor
    pub fn new() -> Self {
        Self {
            planner: HTNPlanner::new(),
            max_retries: 3,
        }
    }

    /// Execute a plan with replanning on failure
    pub async fn execute_with_replanning(
        &self,
        plan: &Plan,
        orchestrator: &Orchestrator,
        initial_state: &WorldState,
    ) -> Result<ExecutionResult> {
        info!("Executing plan with replanning: {}", plan.id);

        let mut current_plan = plan.clone();
        let mut state = initial_state.clone();
        let mut execution_history = vec![];
        let mut retries = 0;

        loop {
            match self.execute_plan(&current_plan, orchestrator, &mut state).await {
                Ok(result) => {
                    info!("Plan executed successfully");
                    return Ok(ExecutionResult {
                        success: true,
                        final_state: state,
                        execution_history,
                        retries,
                    });
                }
                Err(e) => {
                    warn!("Plan execution failed: {}", e);

                    if retries >= self.max_retries {
                        return Ok(ExecutionResult {
                            success: false,
                            final_state: state,
                            execution_history,
                            retries,
                        });
                    }

                    // Analyze failure and replan
                    let failure_analysis = self.analyze_failure(&e, &execution_history).await?;
                    current_plan = self.replan(&current_plan, &failure_analysis, &state).await?;
                    retries += 1;

                    info!("Replanning attempt {}/{}", retries, self.max_retries);
                }
            }
        }
    }

    /// Execute a plan
    async fn execute_plan(
        &self,
        plan: &Plan,
        _orchestrator: &Orchestrator,
        state: &mut WorldState,
    ) -> Result<()> {
        for task in &plan.tasks {
            debug!("Executing task: {}", task.name);

            // In full implementation, this would:
            // 1. Execute the task using the orchestrator
            // 2. Handle the result
            // 3. Update the world state with effects

            // Apply effects
            for effect in &task.effects {
                state.apply_effect(effect);
            }
        }

        Ok(())
    }

    /// Analyze a failure to determine replanning strategy
    async fn analyze_failure(
        &self,
        _error: &common::Error,
        history: &[ExecutionStep],
    ) -> Result<FailureAnalysis> {
        // In full implementation, this would use an LLM to analyze
        // what went wrong and how to fix it

        Ok(FailureAnalysis {
            error_type: FailureType::ExecutionError,
            failed_step: history.last().map(|s| s.task_id.clone()),
            suggested_fix: "Retry with modified parameters".to_string(),
        })
    }

    /// Create a new plan based on failure analysis
    async fn replan(
        &self,
        original_plan: &Plan,
        analysis: &FailureAnalysis,
        _current_state: &WorldState,
    ) -> Result<Plan> {
        info!("Replanning after failure: {:?}", analysis.error_type);

        // In full implementation, this would:
        // 1. Modify the original plan based on failure analysis
        // 2. Or create a completely new plan
        // 3. Ensure the new plan addresses the failure cause

        // For now, return the original plan (placeholder)
        Ok(original_plan.clone())
    }
}

impl Default for PlanExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of plan execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Whether execution was successful
    pub success: bool,
    /// Final world state
    pub final_state: WorldState,
    /// History of executed steps
    pub execution_history: Vec<ExecutionStep>,
    /// Number of replanning retries
    pub retries: u32,
}

/// A step in execution history
#[derive(Debug, Clone)]
pub struct ExecutionStep {
    /// Task identifier
    pub task_id: String,
    /// Task name
    pub task_name: String,
    /// Execution result
    pub result: StepResult,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Result of executing a single step
#[derive(Debug, Clone)]
pub enum StepResult {
    /// Step succeeded
    Success,
    /// Step failed with error
    Failure(String),
    /// Step was skipped
    Skipped,
}

/// Analysis of a plan failure
#[derive(Debug, Clone)]
pub struct FailureAnalysis {
    /// Type of failure
    pub error_type: FailureType,
    /// Which step failed (if any)
    pub failed_step: Option<String>,
    /// Suggested fix
    pub suggested_fix: String,
}

/// Types of failures
#[derive(Debug, Clone)]
pub enum FailureType {
    /// Precondition not met
    PreconditionFailure,
    /// Execution error
    ExecutionError,
    /// Tool not available
    ToolUnavailable,
    /// Timeout
    Timeout,
    /// Unknown error
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_state_satisfies() {
        let mut state = WorldState::default();
        state.files.insert("test.rs".to_string());

        let condition = Condition {
            condition_type: ConditionType::FileExists,
            expression: "test.rs".to_string(),
        };

        assert!(state.satisfies(&condition));

        let false_condition = Condition {
            condition_type: ConditionType::FileExists,
            expression: "nonexistent.rs".to_string(),
        };

        assert!(!state.satisfies(&false_condition));
    }

    #[test]
    fn test_cost_calculation() {
        let cost1 = Cost {
            time_ms: 1000,
            tokens: 500,
            api_calls: 2,
            monetary: 0.01,
        };

        let cost2 = Cost {
            time_ms: 2000,
            tokens: 1000,
            api_calls: 4,
            monetary: 0.02,
        };

        let total = cost1.add(&cost2);
        assert_eq!(total.time_ms, 3000);
        assert_eq!(total.tokens, 1500);
        assert_eq!(total.api_calls, 6);
    }

    #[test]
    fn test_plan_builder() {
        let mut builder = PlanBuilder::new("goal1".to_string());
        
        builder.add_task(PrimitiveTask {
            id: "task1".to_string(),
            name: "test_task".to_string(),
            description: "Test".to_string(),
            tool: None,
            tool_args: serde_json::json!({}),
            preconditions: vec![],
            effects: vec![],
            estimated_cost: Cost {
                time_ms: 1000,
                tokens: 500,
                api_calls: 1,
                monetary: 0.01,
            },
        });

        let plan = builder.build();
        assert_eq!(plan.tasks.len(), 1);
        assert_eq!(plan.total_cost.time_ms, 1000);
    }
}
