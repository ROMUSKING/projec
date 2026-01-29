//! Intelligence layer for the coding agent.
//!
//! This crate provides LLM gateway, prompt management, intent parsing,
//! and the re-prompting engine for continuous improvement.

use common::{async_trait, Error, Module, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod gateway;
pub mod gateway_vertex;
pub mod intent;
pub mod prompt;

/// Main intelligence engine
pub struct IntelligenceEngine {
    gateway: Box<dyn gateway::LlmGateway>,
    prompt_manager: prompt::PromptManager,
    intent_parser: intent::IntentParser,
}

impl IntelligenceEngine {
    pub fn new(gateway: Box<dyn gateway::LlmGateway>) -> Self {
        Self {
            gateway,
            prompt_manager: prompt::PromptManager::new(),
            intent_parser: intent::IntentParser::new(),
        }
    }

    /// Parse user intent from natural language
    pub async fn parse_intent(&self, input: &str) -> Result<Intent> {
        self.intent_parser.parse(input).await
    }

    /// Generate a response using the LLM
    pub async fn generate(&self, context: &Context, prompt: &str) -> Result<GenerationResult> {
        let formatted_prompt = self.prompt_manager.format(prompt, context);
        self.gateway.generate(&formatted_prompt).await
    }

    /// Stream a response from the LLM
    pub async fn generate_stream(
        &self,
        context: &Context,
        prompt: &str,
    ) -> Result<gateway::StreamResult> {
        let formatted_prompt = self.prompt_manager.format(prompt, context);
        self.gateway.generate_stream(&formatted_prompt).await
    }
}

#[async_trait]
impl Module for IntelligenceEngine {
    fn name(&self) -> &str {
        "intelligence"
    }

    async fn initialize(&mut self) -> Result<()> {
        self.gateway.initialize().await
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.gateway.shutdown().await
    }
}

/// Parsed intent from user input
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Intent {
    pub category: IntentCategory,
    pub confidence: f32,
    pub parameters: HashMap<String, serde_json::Value>,
    pub raw_input: String,
}

/// Intent categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentCategory {
    CodeGeneration,
    CodeModification,
    Analysis,
    Testing,
    Documentation,
    Optimization,
    SelfImprovement,
    Unknown,
}

impl std::fmt::Display for IntentCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentCategory::CodeGeneration => write!(f, "code_generation"),
            IntentCategory::CodeModification => write!(f, "code_modification"),
            IntentCategory::Analysis => write!(f, "analysis"),
            IntentCategory::Testing => write!(f, "testing"),
            IntentCategory::Documentation => write!(f, "documentation"),
            IntentCategory::Optimization => write!(f, "optimization"),
            IntentCategory::SelfImprovement => write!(f, "self_improvement"),
            IntentCategory::Unknown => write!(f, "unknown"),
        }
    }
}

/// Context for LLM generation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Context {
    pub code_context: CodeContext,
    pub knowledge_context: KnowledgeContext,
    pub execution_context: ExecutionContext,
    pub system_context: SystemContext,
}

/// Code-related context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeContext {
    pub current_file: Option<String>,
    pub related_files: Vec<String>,
    pub project_structure: Vec<String>,
    pub ast_info: Option<serde_json::Value>,
}

/// Knowledge-related context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeContext {
    pub documentation: Vec<String>,
    pub historical_decisions: Vec<String>,
    pub patterns: Vec<String>,
    /// Hierarchical memory context from memory system
    pub memory_context: Option<knowledge::memory::MemoryContext>,
}

/// Execution-related context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub previous_actions: Vec<Action>,
    pub tool_outputs: Vec<ToolOutput>,
    pub error_messages: Vec<String>,
}

/// System-related context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemContext {
    pub config: serde_json::Value,
    pub available_tools: Vec<String>,
    pub resource_limits: ResourceLimits,
}

/// Resource limits
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_tokens: u32,
    pub timeout_seconds: u64,
}

/// Action record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub tool: String,
    pub parameters: serde_json::Value,
    pub timestamp: common::chrono::DateTime<common::chrono::Utc>,
}

/// Tool output record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub tool: String,
    pub output: serde_json::Value,
    pub success: bool,
}

/// Generation result from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub content: String,
    pub tokens_used: u32,
    pub model: String,
    pub finish_reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_category_display() {
        assert_eq!(IntentCategory::CodeGeneration.to_string(), "code_generation");
        assert_eq!(IntentCategory::Analysis.to_string(), "analysis");
    }
}