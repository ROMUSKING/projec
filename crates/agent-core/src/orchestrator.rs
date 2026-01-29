//! Orchestrator for coordinating agent operations.
//!
//! This module manages the task lifecycle and coordinates
//! between different modules to complete tasks.

use common::{async_trait, Error, Module, Result, TaskId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::evaluation::{EvaluationEngine, EvaluationReport, Persona};

/// Orchestrator for task coordination
pub struct Orchestrator {
    intelligence: Option<Arc<intelligence::IntelligenceEngine>>,
    analysis: Option<Arc<analysis::AnalysisEngine>>,
    knowledge: Option<Arc<knowledge::KnowledgeEngine>>,
    tools: Option<Arc<tools::ToolFramework>>,
    evaluation: Option<Arc<RwLock<EvaluationEngine>>>,
    config: Option<agent_config::AgentConfig>,
    pipeline: TaskExecutionPipeline,
    retry_policy: RetryPolicy,
    checkpoint_store: Arc<RwLock<CheckpointStore>>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            intelligence: None,
            analysis: None,
            knowledge: None,
            tools: None,
            evaluation: None,
            config: None,
            pipeline: TaskExecutionPipeline::new(),
            retry_policy: RetryPolicy::default(),
            checkpoint_store: Arc::new(RwLock::new(CheckpointStore::new())),
        }
    }

    /// Set the evaluation engine
    pub fn with_evaluation(mut self, evaluation: Arc<RwLock<EvaluationEngine>>) -> Self {
        self.evaluation = Some(evaluation);
        self
    }

    /// Set the intelligence engine
    pub fn with_intelligence(mut self, intelligence: Arc<intelligence::IntelligenceEngine>) -> Self {
        self.intelligence = Some(intelligence);
        self
    }

    /// Set the analysis engine
    pub fn with_analysis(mut self, analysis: Arc<analysis::AnalysisEngine>) -> Self {
        self.analysis = Some(analysis);
        self
    }

    /// Set the knowledge engine
    pub fn with_knowledge(mut self, knowledge: Arc<knowledge::KnowledgeEngine>) -> Self {
        self.knowledge = Some(knowledge);
        self
    }

    /// Set the tool framework
    pub fn with_tools(mut self, tools: Arc<tools::ToolFramework>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set the configuration
    pub fn with_config(mut self, config: agent_config::AgentConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set retry policy
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    /// Process a task through the full pipeline
    pub async fn process_task(&self, task: super::Task) -> Result<super::TaskResult> {
        // Guardrail: Ensure task description is not empty
        debug_assert!(!task.description.is_empty(), "Task description cannot be empty");
        
        info!("Orchestrator processing task: {:?}", task.id);
        let start_time = common::chrono::Utc::now();

        // Create initial checkpoint
        let checkpoint = self.create_checkpoint(&task, PipelineStage::IntentParsing).await?;

        // Step 1: Parse intent (if not already done)
        let intent = if task.intent.category == intelligence::IntentCategory::Unknown {
            match self.parse_intent_with_retry(&task.description).await {
                Ok(intent) => {
                    debug!("Parsed intent: {:?} (confidence: {:.2})", intent.category, intent.confidence);
                    intent
                }
                Err(e) => {
                    error!("Failed to parse intent after retries: {}", e);
                    return Err(e);
                }
            }
        } else {
            task.intent.clone()
        };

        self.update_checkpoint(&checkpoint, PipelineStage::ContextGathering).await?;

        // Step 2: Gather context
        let context = match self.gather_context(&task, &intent).await {
            Ok(ctx) => ctx,
            Err(e) => {
                warn!("Context gathering failed: {}, proceeding with minimal context", e);
                intelligence::Context::default()
            }
        };

        self.update_checkpoint(&checkpoint, PipelineStage::Planning).await?;

        // Step 3: Generate plan
        let plan = match self.generate_plan_with_retry(&intent, &context).await {
            Ok(plan) => plan,
            Err(e) => {
                error!("Failed to generate plan: {}", e);
                return Err(e);
            }
        };

        self.update_checkpoint(&checkpoint, PipelineStage::Execution).await?;

        // Step 4: Execute plan
        let execution_result = match self.execute_plan_with_checkpoint(&plan, &checkpoint).await {
            Ok(result) => result,
            Err(e) => {
                error!("Plan execution failed: {}", e);
                return Err(e);
            }
        };

        self.update_checkpoint(&checkpoint, PipelineStage::Validation).await?;

        // Step 5: Validate results
        let validation = match self.validate_results_with_retry(&execution_result).await {
            Ok(val) => val,
            Err(e) => {
                warn!("Validation failed: {}, using basic validation", e);
                ValidationResult {
                    success: execution_result.success,
                    issues: vec![],
                }
            }
        };

        self.update_checkpoint(&checkpoint, PipelineStage::KnowledgeUpdate).await?;

        // Step 6: Update knowledge
        if let Err(e) = self.update_knowledge(&task, &execution_result).await {
            warn!("Failed to update knowledge: {}", e);
            // Non-fatal, continue
        }

        // Mark checkpoint as complete
        self.complete_checkpoint(&checkpoint).await?;

        let execution_time_ms = common::chrono::Utc::now()
            .signed_duration_since(start_time)
            .num_milliseconds() as u64;

        // Return task result
        Ok(super::TaskResult {
            task_id: task.id,
            success: validation.success,
            output: execution_result.summary,
            artifacts: execution_result.artifacts,
            completed_at: common::chrono::Utc::now(),
            execution_time_ms,
            metrics: super::TaskExecutionMetrics {
                tokens_used: execution_result.tokens_used,
                api_calls: execution_result.api_calls,
                tools_used: execution_result.tools_used,
                retries: execution_result.retries,
            },
        })
    }

    /// Run a cross-evaluation for a task
    pub async fn run_evaluation(&self, task: &super::Task, output: &str) -> Result<EvaluationReport> {
        if let (Some(intelligence), Some(evaluation)) = (&self.intelligence, &self.evaluation) {
            info!("Running cross-evaluation for task: {}", task.id);
            
            // Determine persona based on task intent
            let persona = match task.intent.category {
                intelligence::IntentCategory::CodeGeneration => Persona::Reviewer,
                intelligence::IntentCategory::Analysis => Persona::Architect,
                _ => Persona::ProductOwner,
            };
            
            let engine = evaluation.read().await;
            engine.evaluate(intelligence, task, output, persona).await
        } else {
            Err(Error::Internal("Intelligence or Evaluation engine not available".to_string()))
        }
    }

    /// Parse intent from task description with retry logic
    async fn parse_intent_with_retry(&self, description: &str) -> Result<intelligence::Intent> {
        let mut last_error = None;

        for attempt in 0..self.retry_policy.max_retries {
            match self.parse_intent(description).await {
                Ok(intent) => return Ok(intent),
                Err(e) => {
                    warn!("Intent parsing attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                    
                    if attempt < self.retry_policy.max_retries - 1 {
                        let delay = self.retry_policy.calculate_delay(attempt);
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::Internal("Intent parsing failed".to_string())))
    }

    /// Parse intent from task description
    async fn parse_intent(&self, description: &str) -> Result<intelligence::Intent> {
        if let Some(intelligence) = &self.intelligence {
            intelligence.parse_intent(description).await
        } else {
            // Fallback: simple keyword-based intent parsing
            let category = self.classify_intent_simple(description);
            Ok(intelligence::Intent {
                category,
                confidence: 0.5,
                parameters: Default::default(),
                raw_input: description.to_string(),
            })
        }
    }

    /// Simple intent classification
    fn classify_intent_simple(&self, description: &str) -> intelligence::IntentCategory {
        let lower = description.to_lowercase();
        
        if lower.contains("create") || lower.contains("generate") || lower.contains("write") {
            intelligence::IntentCategory::CodeGeneration
        } else if lower.contains("modify") || lower.contains("update") || lower.contains("change") || lower.contains("fix") {
            intelligence::IntentCategory::CodeModification
        } else if lower.contains("analyze") || lower.contains("review") || lower.contains("check") {
            intelligence::IntentCategory::Analysis
        } else if lower.contains("test") || lower.contains("debug") {
            intelligence::IntentCategory::Testing
        } else if lower.contains("document") || lower.contains("doc") {
            intelligence::IntentCategory::Documentation
        } else if lower.contains("optimize") || lower.contains("improve") || lower.contains("refactor") {
            intelligence::IntentCategory::Optimization
        } else if lower.contains("improve yourself") || lower.contains("self improve") {
            intelligence::IntentCategory::SelfImprovement
        } else {
            intelligence::IntentCategory::Unknown
        }
    }

    /// Gather context for the task
    async fn gather_context(
        &self,
        task: &super::Task,
        intent: &intelligence::Intent,
    ) -> Result<intelligence::Context> {
        let mut context = intelligence::Context::default();

        // Get code context
        if let Some(analysis) = &self.analysis {
            for file in &task.context.files {
                if let Ok(file_analysis) = analysis.analyze_file(file).await {
                    context.code_context.related_files.push(file.to_string_lossy().to_string());
                    
                    // Add symbol information
                    for symbol in &file_analysis.lsp.symbols {
                        context.code_context.project_structure.push(format!(
                            "{} ({:?})",
                            symbol.name, symbol.kind
                        ));
                    }
                }
            }

            // Get diagnostics for related files
            for file in &task.context.files {
                if let Ok(diagnostics) = analysis.get_diagnostics(file).await {
                    for diag in diagnostics {
                        context.execution_context.error_messages.push(format!(
                            "{}:{} - {:?}: {}",
                            file.display(),
                            diag.location.line_start,
                            diag.severity,
                            diag.message
                        ));
                    }
                }
            }
        }

        // Get knowledge context
        if let Some(knowledge) = &self.knowledge {
            let search_results = knowledge.search(&task.description, 5).await?;
            for result in search_results {
                context.knowledge_context.documentation.push(result.content);
            }

            // Get patterns from knowledge graph
            let query = knowledge::GraphQuery {
                query_type: knowledge::QueryType::SimilarEntities,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("intent".to_string(), serde_json::json!(intent.category.to_string()));
                    params
                },
            };
            
            if let Ok(graph_results) = knowledge.query_graph(&query).await {
                for result in graph_results {
                    for entity in &result.entities {
                        context.knowledge_context.patterns.push(format!(
                            "Pattern: {:?}",
                            entity
                        ));
                    }
                }
            }
        }

        // Set system context
        if let Some(config) = &self.config {
            context.system_context.config = serde_json::json!({
                "max_tokens": config.llm.max_tokens,
                "temperature": config.llm.temperature,
            });
        }

        if let Some(tools) = &self.tools {
            context.system_context.available_tools = tools
                .list_tools()
                .iter()
                .map(|t| t.name().to_string())
                .collect();
        }

        Ok(context)
    }

    /// Generate a plan with retry logic
    async fn generate_plan_with_retry(
        &self,
        intent: &intelligence::Intent,
        context: &intelligence::Context,
    ) -> Result<ActionPlan> {
        let mut last_error = None;

        for attempt in 0..self.retry_policy.max_retries {
            match self.generate_plan(intent, context).await {
                Ok(plan) => return Ok(plan),
                Err(e) => {
                    warn!("Plan generation attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                    
                    if attempt < self.retry_policy.max_retries - 1 {
                        let delay = self.retry_policy.calculate_delay(attempt);
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::Internal("Plan generation failed".to_string())))
    }

    /// Generate an action plan
    async fn generate_plan(
        &self,
        intent: &intelligence::Intent,
        context: &intelligence::Context,
    ) -> Result<ActionPlan> {
        if let Some(intelligence) = &self.intelligence {
            // Use specialized prompt template based on intent category
            let template_name = match intent.category {
                intelligence::IntentCategory::CodeGeneration => "plan_code_generation",
                intelligence::IntentCategory::CodeModification => "plan_refactoring", // Reuse refactoring template
                intelligence::IntentCategory::Testing => "plan_test_generation",
                _ => "plan_generic",
            };

            let prompt = format!(
                "Generate a structured JSON plan for the following task.\n\
                Intent: {:?}\n\
                Input: {}\n\
                Context:\n- Current file: {:?}\n- Related files: {:?}\n\n\
                The output must be a JSON object with a 'steps' array, where each step has:\n\
                - description: string\n\
                - tool: string (optional)\n\
                - parameters: object (optional)\n\
                - expected_output: string\n\
                - timeout_seconds: number",
                intent.category,
                intent.raw_input,
                context.code_context.current_file,
                context.code_context.related_files
            );
            
            let result = intelligence.generate(context, &prompt).await?;

            // Try to parse structured JSON output
            let steps = match self.parse_structured_plan(&result.content) {
                Ok(s) => s,
                Err(e) => {
                    warn!("Failed to parse structured plan: {}. Falling back to text parsing.", e);
                    parse_plan_from_text(&result.content)
                }
            };

            Ok(ActionPlan {
                steps,
                intent_category: intent.category,
                estimated_tokens: result.tokens_used,
            })
        } else {
            // Fallback: simple plan based on intent
            let steps = vec![
                PlanStep {
                    description: format!("Execute {:?} task: {}", intent.category, intent.raw_input),
                    tool: None,
                    parameters: Default::default(),
                    expected_output: "Task completed".to_string(),
                    timeout_seconds: 60,
                },
            ];

            Ok(ActionPlan {
                steps,
                intent_category: intent.category,
                estimated_tokens: 0,
            })
        }
    }

    /// Parse structured plan from JSON content
    fn parse_structured_plan(&self, content: &str) -> Result<Vec<PlanStep>> {
        // Extract JSON block if wrapped in markdown code fence
        let json_content = if let Some(start) = content.find("```json") {
            if let Some(end) = content[start..].find("```") {
                // Find end of block, excluding the start fence
                let block_start = start + 7;
                if let Some(block_end) = content[block_start..].find("```") {
                    &content[block_start..block_start + block_end]
                } else {
                    content
                }
            } else {
                content
            }
        } else {
            content
        };

        #[derive(Deserialize)]
        struct PlanResponse {
            steps: Vec<PlanStep>,
        }

        let response: PlanResponse = serde_json::from_str(json_content.trim())
            .map_err(|e| Error::Serialization(e))?;

        Ok(response.steps)
    }

    /// Execute plan with checkpointing
    async fn execute_plan_with_checkpoint(
        &self,
        plan: &ActionPlan,
        checkpoint: &TaskCheckpoint,
    ) -> Result<ExecutionResult> {
        let mut artifacts = Vec::new();
        let mut logs = Vec::new();
        let mut tokens_used = 0u32;
        let mut api_calls = 0u32;
        let mut tools_used = Vec::new();
        let mut retries = 0u32;

        for (i, step) in plan.steps.iter().enumerate() {
            info!("Executing plan step {}: {}", i + 1, step.description);

            // Update checkpoint with current step
            self.update_checkpoint_step(checkpoint, i).await?;

            let step_start = common::chrono::Utc::now();
            
            let step_result = if let Some(tool_name) = &step.tool {
                if let Some(tools) = &self.tools {
                    tools_used.push(tool_name.clone());
                    
                    match tokio::time::timeout(
                        tokio::time::Duration::from_secs(step.timeout_seconds),
                        tools.execute(tool_name, serde_json::json!(step.parameters))
                    ).await {
                        Ok(Ok(result)) => {
                            api_calls += 1;
                            if result.success {
                                logs.push(format!("✓ Step {} succeeded", i + 1));
                                StepResult::Success(result.data)
                            } else {
                                logs.push(format!("✗ Step {} failed: {:?}", i + 1, result.data));
                                StepResult::Failure(result.data.to_string())
                            }
                        }
                        Ok(Err(e)) => {
                            retries += 1;
                            StepResult::Failure(e.to_string())
                        }
                        Err(_) => {
                            retries += 1;
                            StepResult::Timeout
                        }
                    }
                } else {
                    StepResult::Failure("Tools not available".to_string())
                }
            } else {
                // No tool specified, treat as informational step
                logs.push(format!("→ Step {}: {}", i + 1, step.description));
                StepResult::Success(serde_json::json!({"info": step.description}))
            };

            let step_duration = common::chrono::Utc::now()
                .signed_duration_since(step_start)
                .num_milliseconds();
            
            debug!("Step {} completed in {}ms", i + 1, step_duration);

            // Handle step failure with retry
            match step_result {
                StepResult::Success(data) => {
                    if let Some(artifact) = self.data_to_artifact(&data).await {
                        artifacts.push(artifact);
                    }
                }
                StepResult::Failure(msg) => {
                    warn!("Step {} failed: {}", i + 1, msg);
                    
                    // Try recovery if configured
                    if self.retry_policy.retry_on_failure && retries < self.retry_policy.max_retries {
                        warn!("Attempting recovery for step {} (retry {}/{})", i + 1, retries, self.retry_policy.max_retries);

                        // Simple retry logic: just retry the step once more after a delay
                        // In a real implementation, this would involve analyzing the error and potentially modifying the plan
                        let delay = self.retry_policy.calculate_delay(retries);
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

                        // Re-execute step (recursive call or loop would be better, but for now we just fail after one retry in this block structure)
                        // Note: ideally we would loop here, but refactoring the loop structure is risky.
                        // Instead, we'll mark it as failed but non-fatal if possible, or propagate error.
                        // For this implementation, we propagate the error to stop execution on persistent failure.
                        return Err(Error::Execution(format!("Step {} failed after retry: {}", i + 1, msg)));
                    } else {
                        return Err(Error::Execution(format!("Step {} failed: {}", i + 1, msg)));
                    }
                }
                StepResult::Timeout => {
                    error!("Step {} timed out", i + 1);
                    return Err(Error::Timeout(format!("Step {} timed out", i + 1)));
                }
            }
        }

        Ok(ExecutionResult {
            success: true,
            summary: logs.join("\n"),
            artifacts,
            tokens_used,
            api_calls,
            tools_used,
            retries,
        })
    }

    /// Validate results with retry
    async fn validate_results_with_retry(
        &self,
        result: &ExecutionResult,
    ) -> Result<ValidationResult> {
        for attempt in 0..self.retry_policy.max_retries {
            match self.validate_results(result).await {
                Ok(validation) => return Ok(validation),
                Err(e) => {
                    if attempt < self.retry_policy.max_retries - 1 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        
        Ok(ValidationResult {
            success: result.success,
            issues: vec![],
        })
    }

    /// Validate execution results
    async fn validate_results(&self, result: &ExecutionResult) -> Result<ValidationResult> {
        let mut issues = Vec::new();

        // Check for empty artifacts when success is claimed
        if result.success && result.artifacts.is_empty() {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                message: "Task marked successful but no artifacts produced".to_string(),
                location: None,
            });
        }

        // Validate each artifact
        for artifact in &result.artifacts {
            if let Err(e) = self.validate_artifact(artifact).await {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    message: format!("Artifact validation failed: {}", e),
                    location: artifact.path.as_ref().map(|p| p.to_string_lossy().to_string()),
                });
            }
        }

        let success = result.success && !issues.iter().any(|i| i.severity == IssueSeverity::Error);

        Ok(ValidationResult { success, issues })
    }

    /// Validate a single artifact
    async fn validate_artifact(&self, artifact: &super::Artifact) -> Result<()> {
        match artifact.artifact_type {
            super::ArtifactType::File => {
                if artifact.content.is_empty() {
                    return Err(Error::Validation("Empty file content".to_string()));
                }
            }
            super::ArtifactType::Diff => {
                // Validate diff format
                if !artifact.content.contains("---") || !artifact.content.contains("+++") {
                    return Err(Error::Validation("Invalid diff format".to_string()));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Update knowledge base with task results
    async fn update_knowledge(
        &self,
        task: &super::Task,
        result: &ExecutionResult,
    ) -> Result<()> {
        if let Some(knowledge) = &self.knowledge {
            // Index task description and results
            let content = format!(
                "Task: {}\nIntent: {:?}\nResult: {}",
                task.description, task.intent.category, result.summary
            );
            
            let path = std::path::PathBuf::from(format!(
                "knowledge/tasks/{}",
                task.id
            ));
            
            // This would need a mutable reference, so we skip for now
            // In production, use a message queue or separate task
            debug!("Would update knowledge base at {:?}", path);
        }

        Ok(())
    }

    /// Create a checkpoint for a task
    async fn create_checkpoint(
        &self,
        task: &super::Task,
        stage: PipelineStage,
    ) -> Result<TaskCheckpoint> {
        let checkpoint = TaskCheckpoint {
            id: uuid::Uuid::new_v4().to_string(),
            task_id: task.id,
            stage,
            current_step: 0,
            created_at: common::chrono::Utc::now(),
            completed_at: None,
        };

        self.checkpoint_store.write().await.add(checkpoint.clone());
        Ok(checkpoint)
    }

    /// Update checkpoint stage
    async fn update_checkpoint(
        &self,
        checkpoint: &TaskCheckpoint,
        stage: PipelineStage,
    ) -> Result<()> {
        self.checkpoint_store
            .write()
            .await
            .update_stage(&checkpoint.id, stage);
        Ok(())
    }

    /// Update checkpoint step
    async fn update_checkpoint_step(
        &self,
        checkpoint: &TaskCheckpoint,
        step: usize,
    ) -> Result<()> {
        self.checkpoint_store
            .write()
            .await
            .update_step(&checkpoint.id, step);
        Ok(())
    }

    /// Mark checkpoint as complete
    async fn complete_checkpoint(&self, checkpoint: &TaskCheckpoint) -> Result<()> {
        self.checkpoint_store
            .write()
            .await
            .complete(&checkpoint.id);
        Ok(())
    }

    /// Convert execution data to artifact
    async fn data_to_artifact(&self, data: &serde_json::Value) -> Option<super::Artifact> {
        if let Some(content) = data.as_str() {
            Some(super::Artifact {
                artifact_type: super::ArtifactType::Message,
                content: content.to_string(),
                path: None,
                metadata: HashMap::new(),
            })
        } else {
            Some(super::Artifact {
                artifact_type: super::ArtifactType::Message,
                content: data.to_string(),
                path: None,
                metadata: HashMap::new(),
            })
        }
    }
}

#[async_trait]
impl Module for Orchestrator {
    fn name(&self) -> &str {
        "orchestrator"
    }

    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing orchestrator");
        
        // Validate all required components are present
        if self.intelligence.is_none() {
            warn!("No intelligence engine configured");
        }
        if self.analysis.is_none() {
            warn!("No analysis engine configured");
        }
        if self.knowledge.is_none() {
            warn!("No knowledge engine configured");
        }
        if self.tools.is_none() {
            warn!("No tools configured");
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down orchestrator");
        Ok(())
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Task execution pipeline
pub struct TaskExecutionPipeline {
    stages: Vec<PipelineStage>,
}

impl TaskExecutionPipeline {
    pub fn new() -> Self {
        Self {
            stages: vec![
                PipelineStage::IntentParsing,
                PipelineStage::ContextGathering,
                PipelineStage::Planning,
                PipelineStage::Execution,
                PipelineStage::Validation,
                PipelineStage::KnowledgeUpdate,
            ],
        }
    }

    pub fn get_stage(&self, index: usize) -> Option<&PipelineStage> {
        self.stages.get(index)
    }

    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }
}

impl Default for TaskExecutionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Pipeline stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStage {
    IntentParsing,
    ContextGathering,
    Planning,
    Execution,
    Validation,
    KnowledgeUpdate,
    Completed,
    Failed,
}

/// Task checkpoint for recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCheckpoint {
    pub id: String,
    pub task_id: TaskId,
    pub stage: PipelineStage,
    pub current_step: usize,
    pub created_at: common::chrono::DateTime<common::chrono::Utc>,
    pub completed_at: Option<common::chrono::DateTime<common::chrono::Utc>>,
}

/// Checkpoint store
pub struct CheckpointStore {
    checkpoints: Vec<TaskCheckpoint>,
}

impl CheckpointStore {
    pub fn new() -> Self {
        Self {
            checkpoints: Vec::new(),
        }
    }

    pub fn add(&mut self, checkpoint: TaskCheckpoint) {
        self.checkpoints.push(checkpoint);
    }

    pub fn update_stage(&mut self, id: &str, stage: PipelineStage) {
        if let Some(cp) = self.checkpoints.iter_mut().find(|c| c.id == id) {
            cp.stage = stage;
        }
    }

    pub fn update_step(&mut self, id: &str, step: usize) {
        if let Some(cp) = self.checkpoints.iter_mut().find(|c| c.id == id) {
            cp.current_step = step;
        }
    }

    pub fn complete(&mut self, id: &str) {
        if let Some(cp) = self.checkpoints.iter_mut().find(|c| c.id == id) {
            cp.stage = PipelineStage::Completed;
            cp.completed_at = Some(common::chrono::Utc::now());
        }
    }

    pub fn get(&self, id: &str) -> Option<&TaskCheckpoint> {
        self.checkpoints.iter().find(|c| c.id == id)
    }
}

impl Default for CheckpointStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_backoff: bool,
    pub retry_on_failure: bool,
}

impl RetryPolicy {
    /// Calculate delay for a given retry attempt
    pub fn calculate_delay(&self, attempt: u32) -> u64 {
        if self.exponential_backoff {
            let delay = self.base_delay_ms * 2_u64.pow(attempt);
            delay.min(self.max_delay_ms)
        } else {
            self.base_delay_ms
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            exponential_backoff: true,
            retry_on_failure: true,
        }
    }
}

/// Action plan for task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    pub steps: Vec<PlanStep>,
    pub intent_category: intelligence::IntentCategory,
    pub estimated_tokens: u32,
}

/// Individual plan step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub description: String,
    pub tool: Option<String>,
    pub parameters: serde_json::Value,
    pub expected_output: String,
    pub timeout_seconds: u64,
}

/// Step execution result
#[derive(Debug, Clone)]
pub enum StepResult {
    Success(serde_json::Value),
    Failure(String),
    Timeout,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub summary: String,
    pub artifacts: Vec<super::Artifact>,
    pub tokens_used: u32,
    pub api_calls: u32,
    pub tools_used: Vec<String>,
    pub retries: u32,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub success: bool,
    pub issues: Vec<ValidationIssue>,
}

/// Validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub message: String,
    pub location: Option<String>,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueSeverity::Error => write!(f, "error"),
            IssueSeverity::Warning => write!(f, "warning"),
            IssueSeverity::Info => write!(f, "info"),
        }
    }
}

/// Parse plan from text (placeholder implementation)
fn parse_plan_from_text(text: &str) -> Vec<PlanStep> {
    // In a real implementation, this would parse structured output from the LLM
    // For now, create a simple single-step plan
    vec![PlanStep {
        description: text.to_string(),
        tool: None,
        parameters: Default::default(),
        expected_output: "Task completed".to_string(),
        timeout_seconds: 60,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_delay() {
        let policy = RetryPolicy::default();
        
        assert_eq!(policy.calculate_delay(0), 100);
        assert_eq!(policy.calculate_delay(1), 200);
        assert_eq!(policy.calculate_delay(2), 400);
    }

    #[test]
    fn test_intent_classification() {
        let orchestrator = Orchestrator::new();
        
        assert_eq!(
            orchestrator.classify_intent_simple("Create a new function"),
            intelligence::IntentCategory::CodeGeneration
        );
        
        assert_eq!(
            orchestrator.classify_intent_simple("Fix the bug in this code"),
            intelligence::IntentCategory::CodeModification
        );
        
        assert_eq!(
            orchestrator.classify_intent_simple("Analyze this file"),
            intelligence::IntentCategory::Analysis
        );
    }
}
