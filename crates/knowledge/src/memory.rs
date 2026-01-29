//! Hierarchical Memory System for the Coding Agent
//!
//! This module implements a tiered memory architecture with four layers:
//! 1. Working Memory - Active context for current task
//! 2. Session Memory - Conversation history and recent actions
//! 3. Project Memory - Project-specific knowledge and patterns
//! 4. Persistent Memory - Cross-project documentation and learnings
//!
//! Based on state-of-the-art agentic coding practices from Claude Code,
//! Cursor, and GitHub Copilot.

use common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Memory tier enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryTier {
    /// Working memory - immediate task context (~100K tokens)
    Working,
    /// Session memory - current session history
    Session,
    /// Project memory - project-specific knowledge
    Project,
    /// Persistent memory - cross-project learnings
    Persistent,
}

impl MemoryTier {
    /// Get the priority level (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            MemoryTier::Working => 0,
            MemoryTier::Session => 1,
            MemoryTier::Project => 2,
            MemoryTier::Persistent => 3,
        }
    }

    /// Get the maximum token budget for this tier
    pub fn token_budget(&self) -> usize {
        match self {
            MemoryTier::Working => 100_000,      // ~100K tokens
            MemoryTier::Session => 500_000,      // ~500K tokens
            MemoryTier::Project => 2_000_000,    // ~2M tokens
            MemoryTier::Persistent => 10_000_000, // ~10M tokens
        }
    }
}

/// A memory chunk with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryChunk {
    /// Unique identifier
    pub id: String,
    /// Memory tier
    pub tier: MemoryTier,
    /// Content type
    pub content_type: ContentType,
    /// Actual content
    pub content: String,
    /// Token count estimate
    pub token_count: usize,
    /// Relevance score (0.0 - 1.0)
    pub relevance_score: f32,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last access timestamp
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    /// Access count
    pub access_count: u32,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Source reference (file, task, etc.)
    pub source: Option<String>,
}

impl MemoryChunk {
    /// Create a new memory chunk
    pub fn new(
        id: impl Into<String>,
        tier: MemoryTier,
        content_type: ContentType,
        content: impl Into<String>,
    ) -> Self {
        let content = content.into();
        let token_count = estimate_token_count(&content);
        let now = chrono::Utc::now();
        
        Self {
            id: id.into(),
            tier,
            content_type,
            content,
            token_count,
            relevance_score: 1.0,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            tags: Vec::new(),
            source: None,
        }
    }

    /// Mark as accessed
    pub fn mark_accessed(&mut self) {
        self.last_accessed = chrono::Utc::now();
        self.access_count += 1;
        // Boost relevance slightly on access
        self.relevance_score = (self.relevance_score * 1.05).min(1.0);
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

/// Content type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    /// Code snippet or file content
    Code,
    /// Documentation or explanation
    Documentation,
    /// Error message or stack trace
    Error,
    /// Task description or intent
    Task,
    /// System configuration
    SystemConfig,
    /// Project metadata
    ProjectInfo,
    /// Conversation or thought
    Conversation,
    /// Pattern or template
    Pattern,
    /// Decision or rationale
    Decision,
}

/// System configuration memory layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMemory {
    /// Operating system information
    pub os_info: OsInfo,
    /// Hardware specifications
    pub hardware: HardwareInfo,
    /// Development environment
    pub dev_environment: DevEnvironment,
    /// Installed tools and versions
    pub tools: HashMap<String, String>,
    /// Environment variables (sanitized)
    pub env_vars: HashMap<String, String>,
    /// Agent configuration snapshot
    pub agent_config: serde_json::Value,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl SystemMemory {
    /// Create new system memory
    pub async fn capture() -> Result<Self> {
        Ok(Self {
            os_info: OsInfo::detect(),
            hardware: HardwareInfo::detect(),
            dev_environment: DevEnvironment::detect().await?,
            tools: detect_installed_tools().await,
            env_vars: capture_relevant_env_vars(),
            agent_config: serde_json::Value::Null,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Convert to memory chunks
    pub fn to_chunks(&self) -> Vec<MemoryChunk> {
        let mut chunks = Vec::new();

        // OS info chunk
        chunks.push(
            MemoryChunk::new(
                "system:os",
                MemoryTier::Persistent,
                ContentType::SystemConfig,
                format!(
                    "Operating System: {} {} ({})",
                    self.os_info.name, self.os_info.version, self.os_info.architecture
                ),
            )
            .with_tag("system")
            .with_tag("os"),
        );

        // Hardware info chunk
        chunks.push(
            MemoryChunk::new(
                "system:hardware",
                MemoryTier::Persistent,
                ContentType::SystemConfig,
                format!(
                    "Hardware: {} CPUs, {} RAM, {} disk",
                    self.hardware.cpu_count,
                    format_bytes(self.hardware.memory_bytes),
                    format_bytes(self.hardware.disk_bytes)
                ),
            )
            .with_tag("system")
            .with_tag("hardware"),
        );

        // Dev environment chunk
        chunks.push(
            MemoryChunk::new(
                "system:dev_env",
                MemoryTier::Persistent,
                ContentType::SystemConfig,
                format!(
                    "Development Environment:\n- Shell: {}\n- Terminal: {}\n- Editor: {}\n- Languages: {}",
                    self.dev_environment.shell,
                    self.dev_environment.terminal,
                    self.dev_environment.editor,
                    self.dev_environment.languages.join(", ")
                ),
            )
            .with_tag("system")
            .with_tag("environment"),
        );

        // Tools chunk
        let tools_content = self
            .tools
            .iter()
            .map(|(name, version)| format!("- {}: {}", name, version))
            .collect::<Vec<_>>()
            .join("\n");
        
        chunks.push(
            MemoryChunk::new(
                "system:tools",
                MemoryTier::Persistent,
                ContentType::SystemConfig,
                format!("Installed Tools:\n{}", tools_content),
            )
            .with_tag("system")
            .with_tag("tools"),
        );

        chunks
    }
}

/// OS information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub family: String,
}

impl OsInfo {
    /// Detect current OS
    pub fn detect() -> Self {
        Self {
            name: std::env::consts::OS.to_string(),
            version: sys_info::os_release().unwrap_or_default(),
            architecture: std::env::consts::ARCH.to_string(),
            family: std::env::consts::FAMILY.to_string(),
        }
    }
}

/// Hardware information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu_count: usize,
    pub memory_bytes: u64,
    pub disk_bytes: u64,
}

impl HardwareInfo {
    /// Detect hardware
    pub fn detect() -> Self {
        Self {
            cpu_count: num_cpus::get(),
            memory_bytes: sys_info::mem_info()
                .map(|m| m.total * 1024)
                .unwrap_or(0),
            disk_bytes: 0, // Would need platform-specific implementation
        }
    }
}

/// Development environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevEnvironment {
    pub shell: String,
    pub terminal: String,
    pub editor: String,
    pub languages: Vec<String>,
}

impl DevEnvironment {
    /// Detect dev environment
    pub async fn detect() -> Result<Self> {
        Ok(Self {
            shell: std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string()),
            terminal: std::env::var("TERM").unwrap_or_else(|_| "unknown".to_string()),
            editor: std::env::var("EDITOR").unwrap_or_else(|_| "unknown".to_string()),
            languages: detect_installed_languages().await,
        })
    }
}

/// Project context memory layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMemory {
    /// Project root path
    pub root_path: PathBuf,
    /// Project name
    pub name: String,
    /// Project type (rust, python, etc.)
    pub project_type: String,
    /// Programming languages used
    pub languages: Vec<String>,
    /// Key files and their purposes
    pub key_files: HashMap<String, String>,
    /// Dependencies and versions
    pub dependencies: HashMap<String, String>,
    /// Architecture overview
    pub architecture_summary: String,
    /// Important patterns used
    pub patterns: Vec<String>,
    /// Recent changes summary
    pub recent_changes: Vec<ChangeSummary>,
    /// Active features being worked on
    pub active_features: Vec<FeatureContext>,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl ProjectMemory {
    /// Create new project memory
    pub async fn load(project_root: impl Into<PathBuf>) -> Result<Self> {
        let root_path = project_root.into();
        let name = root_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            root_path,
            name,
            project_type: "unknown".to_string(),
            languages: Vec::new(),
            key_files: HashMap::new(),
            dependencies: HashMap::new(),
            architecture_summary: String::new(),
            patterns: Vec::new(),
            recent_changes: Vec::new(),
            active_features: Vec::new(),
            last_updated: chrono::Utc::now(),
        })
    }

    /// Convert to memory chunks
    pub fn to_chunks(&self) -> Vec<MemoryChunk> {
        let mut chunks = Vec::new();

        // Project overview
        chunks.push(
            MemoryChunk::new(
                format!("project:{}:overview", self.name),
                MemoryTier::Project,
                ContentType::ProjectInfo,
                format!(
                    "Project: {}\nType: {}\nLanguages: {}\nPath: {}",
                    self.name,
                    self.project_type,
                    self.languages.join(", "),
                    self.root_path.display()
                ),
            )
            .with_tag("project")
            .with_tag("overview"),
        );

        // Architecture summary
        if !self.architecture_summary.is_empty() {
            chunks.push(
                MemoryChunk::new(
                    format!("project:{}:architecture", self.name),
                    MemoryTier::Project,
                    ContentType::Documentation,
                    format!("Architecture:\n{}", self.architecture_summary),
                )
                .with_tag("project")
                .with_tag("architecture"),
            );
        }

        // Key files
        for (file, purpose) in &self.key_files {
            chunks.push(
                MemoryChunk::new(
                    format!("project:{}:file:{}", self.name, file),
                    MemoryTier::Project,
                    ContentType::ProjectInfo,
                    format!("File: {}\nPurpose: {}", file, purpose),
                )
                .with_tag("project")
                .with_tag("file")
                .with_source(file),
            );
        }

        // Patterns
        for (i, pattern) in self.patterns.iter().enumerate() {
            chunks.push(
                MemoryChunk::new(
                    format!("project:{}:pattern:{}", self.name, i),
                    MemoryTier::Project,
                    ContentType::Pattern,
                    pattern.clone(),
                )
                .with_tag("project")
                .with_tag("pattern"),
            );
        }

        // Active features
        for feature in &self.active_features {
            chunks.push(
                MemoryChunk::new(
                    format!("project:{}:feature:{}", self.name, feature.name),
                    MemoryTier::Project,
                    ContentType::Task,
                    format!(
                        "Feature: {}\nStatus: {}\nDescription: {}",
                        feature.name, feature.status, feature.description
                    ),
                )
                .with_tag("project")
                .with_tag("feature")
                .with_tag(&feature.name),
            );
        }

        chunks
    }
}

/// Feature context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureContext {
    pub name: String,
    pub status: String,
    pub description: String,
    pub related_files: Vec<String>,
    pub decisions: Vec<String>,
}

/// Change summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSummary {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub description: String,
    pub files_changed: Vec<String>,
}

/// Task-specific memory layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMemory {
    /// Task ID
    pub task_id: String,
    /// Task description
    pub description: String,
    /// Task intent/purpose
    pub intent: String,
    /// Current status
    pub status: TaskStatus,
    /// Steps taken so far
    pub steps: Vec<TaskStep>,
    /// Decisions made
    pub decisions: Vec<Decision>,
    /// Errors encountered
    pub errors: Vec<ErrorRecord>,
    /// Files being worked on
    pub active_files: Vec<String>,
    /// Context gathered
    pub gathered_context: Vec<String>,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl TaskMemory {
    /// Create new task memory
    pub fn new(task_id: impl Into<String>, description: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            task_id: task_id.into(),
            description: description.into(),
            intent: String::new(),
            status: TaskStatus::InProgress,
            steps: Vec::new(),
            decisions: Vec::new(),
            errors: Vec::new(),
            active_files: Vec::new(),
            gathered_context: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Record a step
    pub fn record_step(&mut self, action: impl Into<String>, result: impl Into<String>) {
        self.steps.push(TaskStep {
            timestamp: chrono::Utc::now(),
            action: action.into(),
            result: result.into(),
        });
        self.updated_at = chrono::Utc::now();
    }

    /// Record a decision
    pub fn record_decision(&mut self, decision: impl Into<String>, rationale: impl Into<String>) {
        self.decisions.push(Decision {
            timestamp: chrono::Utc::now(),
            decision: decision.into(),
            rationale: rationale.into(),
        });
        self.updated_at = chrono::Utc::now();
    }

    /// Record an error
    pub fn record_error(&mut self, error: impl Into<String>, context: impl Into<String>) {
        self.errors.push(ErrorRecord {
            timestamp: chrono::Utc::now(),
            error: error.into(),
            context: context.into(),
        });
        self.updated_at = chrono::Utc::now();
    }

    /// Convert to memory chunks
    pub fn to_chunks(&self) -> Vec<MemoryChunk> {
        let mut chunks = Vec::new();

        // Task overview
        chunks.push(
            MemoryChunk::new(
                format!("task:{}:overview", self.task_id),
                MemoryTier::Working,
                ContentType::Task,
                format!(
                    "Task: {}\nDescription: {}\nStatus: {:?}",
                    self.task_id, self.description, self.status
                ),
            )
            .with_tag("task")
            .with_tag(&self.task_id),
        );

        // Intent
        if !self.intent.is_empty() {
            chunks.push(
                MemoryChunk::new(
                    format!("task:{}:intent", self.task_id),
                    MemoryTier::Working,
                    ContentType::Task,
                    format!("Intent: {}", self.intent),
                )
                .with_tag("task")
                .with_tag("intent"),
            );
        }

        // Recent steps (last 5)
        let recent_steps = self.steps.iter().rev().take(5).collect::<Vec<_>>();
        if !recent_steps.is_empty() {
            let steps_content = recent_steps
                .iter()
                .map(|s| format!("- {}: {}", s.action, s.result))
                .collect::<Vec<_>>()
                .join("\n");
            
            chunks.push(
                MemoryChunk::new(
                    format!("task:{}:steps", self.task_id),
                    MemoryTier::Session,
                    ContentType::Conversation,
                    format!("Recent Steps:\n{}", steps_content),
                )
                .with_tag("task")
                .with_tag("steps"),
            );
        }

        // Decisions
        for (i, decision) in self.decisions.iter().enumerate() {
            chunks.push(
                MemoryChunk::new(
                    format!("task:{}:decision:{}", self.task_id, i),
                    MemoryTier::Session,
                    ContentType::Decision,
                    format!(
                        "Decision: {}\nRationale: {}",
                        decision.decision, decision.rationale
                    ),
                )
                .with_tag("task")
                .with_tag("decision"),
            );
        }

        // Active files
        if !self.active_files.is_empty() {
            chunks.push(
                MemoryChunk::new(
                    format!("task:{}:files", self.task_id),
                    MemoryTier::Working,
                    ContentType::Code,
                    format!("Active Files:\n- {}", self.active_files.join("\n- ")),
                )
                .with_tag("task")
                .with_tag("files"),
            );
        }

        chunks
    }
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    InProgress,
    Completed,
    Failed,
    Blocked,
}

/// Task step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: String,
    pub result: String,
}

/// Decision record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub decision: String,
    pub rationale: String,
}

/// Error record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub error: String,
    pub context: String,
}

/// Hierarchical Memory Manager
pub struct HierarchicalMemory {
    /// All memory chunks indexed by ID
    chunks: Arc<RwLock<HashMap<String, MemoryChunk>>>,
    /// Chunks organized by tier
    tier_index: Arc<RwLock<HashMap<MemoryTier, Vec<String>>>>,
    /// System memory
    system_memory: Arc<RwLock<Option<SystemMemory>>>,
    /// Project memory
    project_memory: Arc<RwLock<Option<ProjectMemory>>>,
    /// Active task memories
    task_memories: Arc<RwLock<HashMap<String, TaskMemory>>>,
    /// Session history
    session_history: Arc<RwLock<VecDeque<String>>>,
}

impl HierarchicalMemory {
    /// Create new hierarchical memory
    pub fn new() -> Self {
        Self {
            chunks: Arc::new(RwLock::new(HashMap::new())),
            tier_index: Arc::new(RwLock::new(HashMap::new())),
            system_memory: Arc::new(RwLock::new(None)),
            project_memory: Arc::new(RwLock::new(None)),
            task_memories: Arc::new(RwLock::new(HashMap::new())),
            session_history: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Initialize system memory
    pub async fn initialize_system(&self) -> Result<()> {
        info!("Initializing system memory...");
        let sys_mem = SystemMemory::capture().await?;
        
        // Store chunks
        let chunks = sys_mem.to_chunks();
        for chunk in chunks {
            self.store_chunk(chunk).await?;
        }
        
        *self.system_memory.write().await = Some(sys_mem);
        info!("System memory initialized");
        Ok(())
    }

    /// Initialize project memory
    pub async fn initialize_project(&self, project_root: impl Into<PathBuf>) -> Result<()> {
        info!("Initializing project memory...");
        let proj_mem = ProjectMemory::load(project_root).await?;
        
        // Store chunks
        let chunks = proj_mem.to_chunks();
        for chunk in chunks {
            self.store_chunk(chunk).await?;
        }
        
        *self.project_memory.write().await = Some(proj_mem);
        info!("Project memory initialized");
        Ok(())
    }

    /// Create task memory
    pub async fn create_task(&self, task_id: impl Into<String>, description: impl Into<String>) -> Result<()> {
        let task_id = task_id.into();
        let task_mem = TaskMemory::new(&task_id, description);
        
        // Store chunks
        let chunks = task_mem.to_chunks();
        for chunk in chunks {
            self.store_chunk(chunk).await?;
        }
        
        self.task_memories.write().await.insert(task_id, task_mem);
        Ok(())
    }

    /// Store a memory chunk
    pub async fn store_chunk(&self, chunk: MemoryChunk) -> Result<()> {
        let mut chunks = self.chunks.write().await;
        let mut tier_index = self.tier_index.write().await;
        
        // Add to tier index
        tier_index
            .entry(chunk.tier)
            .or_insert_with(Vec::new)
            .push(chunk.id.clone());
        
        // Store chunk
        chunks.insert(chunk.id.clone(), chunk);
        
        Ok(())
    }

    /// Retrieve a chunk by ID
    pub async fn get_chunk(&self, id: &str) -> Result<Option<MemoryChunk>> {
        let mut chunks = self.chunks.write().await;
        
        if let Some(chunk) = chunks.get_mut(id) {
            chunk.mark_accessed();
            Ok(Some(chunk.clone()))
        } else {
            Ok(None)
        }
    }

    /// Query memory with context assembly
    pub async fn query(&self, query: &str, max_tokens: usize) -> Result<AssembledContext> {
        debug!("Querying memory: {}", query);
        
        let chunks = self.chunks.read().await;
        let mut scored_chunks: Vec<(f32, &MemoryChunk)> = Vec::new();
        
        // Score all chunks
        for chunk in chunks.values() {
            let score = score_relevance(query, chunk);
            if score > 0.3 { // Relevance threshold
                scored_chunks.push((score, chunk));
            }
        }
        
        // Sort by score descending
        scored_chunks.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        // Assemble context within token budget
        let mut assembled = AssembledContext::new();
        let mut remaining_tokens = max_tokens;
        
        for (score, chunk) in scored_chunks {
            if chunk.token_count <= remaining_tokens {
                assembled.add_chunk(chunk.clone(), score);
                remaining_tokens -= chunk.token_count;
            } else if remaining_tokens < 1000 {
                // Not enough room for anything meaningful
                break;
            }
        }
        
        Ok(assembled)
    }

    /// Get context for a specific task
    pub async fn get_task_context(&self, task_id: &str, max_tokens: usize) -> Result<AssembledContext> {
        let mut assembled = AssembledContext::new();
        let mut remaining_tokens = max_tokens;
        
        // 1. Working memory - task-specific (highest priority)
        {
            let chunks = self.chunks.read().await;
            let task_chunks: Vec<_> = chunks
                .values()
                .filter(|c| c.tier == MemoryTier::Working && c.id.contains(&format!("task:{}", task_id)))
                .cloned()
                .collect();
            
            for chunk in task_chunks {
                if chunk.token_count <= remaining_tokens {
                    assembled.add_chunk(chunk.clone(), 1.0);
                    remaining_tokens -= chunk.token_count;
                }
            }
        }
        
        // 2. Session memory - recent history
        if remaining_tokens > 5000 {
            let chunks = self.chunks.read().await;
            let session_chunks: Vec<_> = chunks
                .values()
                .filter(|c| c.tier == MemoryTier::Session)
                .cloned()
                .collect();
            
            for chunk in session_chunks {
                if chunk.token_count <= remaining_tokens {
                    assembled.add_chunk(chunk.clone(), 0.8);
                    remaining_tokens -= chunk.token_count;
                }
            }
        }
        
        // 3. Project memory - relevant project info
        if remaining_tokens > 5000 {
            let chunks = self.chunks.read().await;
            let project_chunks: Vec<_> = chunks
                .values()
                .filter(|c| c.tier == MemoryTier::Project)
                .take(10) // Limit project context
                .cloned()
                .collect();
            
            for chunk in project_chunks {
                if chunk.token_count <= remaining_tokens {
                    assembled.add_chunk(chunk.clone(), 0.6);
                    remaining_tokens -= chunk.token_count;
                }
            }
        }
        
        // 4. System memory - essential system info
        if remaining_tokens > 2000 {
            let chunks = self.chunks.read().await;
            let system_chunks: Vec<_> = chunks
                .values()
                .filter(|c| c.tier == MemoryTier::Persistent && c.id.starts_with("system:"))
                .take(3) // Essential system info only
                .cloned()
                .collect();
            
            for chunk in system_chunks {
                if chunk.token_count <= remaining_tokens {
                    assembled.add_chunk(chunk.clone(), 0.4);
                    remaining_tokens -= chunk.token_count;
                }
            }
        }
        
        Ok(assembled)
    }

    /// Summarize and compress memory tier
    pub async fn compress_tier(&self, tier: MemoryTier) -> Result<()> {
        info!("Compressing memory tier: {:?}", tier);
        
        let chunks = self.chunks.read().await;
        let tier_chunks: Vec<_> = chunks
            .values()
            .filter(|c| c.tier == tier)
            .cloned()
            .collect();
        
        // TODO: Implement actual compression with LLM
        // For now, just remove low-relevance chunks
        let mut to_remove = Vec::new();
        for chunk in &tier_chunks {
            if chunk.relevance_score < 0.2 && chunk.access_count < 2 {
                to_remove.push(chunk.id.clone());
            }
        }
        
        drop(chunks);
        
        let count = to_remove.len();
        let mut chunks = self.chunks.write().await;
        for id in to_remove {
            chunks.remove(&id);
        }
        
        info!("Compression complete, removed {} chunks", count);
        Ok(())
    }
}

impl Default for HierarchicalMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[common::async_trait]
impl common::Module for HierarchicalMemory {
    fn name(&self) -> &str {
        "hierarchical_memory"
    }

    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing hierarchical memory system...");
        // Initialize system memory
        self.initialize_system().await?;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down hierarchical memory system...");
        // Persist any unsaved memory
        Ok(())
    }
}

/// Memory context for external use (e.g., in prompts)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryContext {
    /// Working memory content
    pub working_memory: Vec<String>,
    /// Session history
    pub session_history: Vec<String>,
    /// Project context
    pub project_context: Vec<String>,
    /// Persistent knowledge
    pub persistent_knowledge: Vec<String>,
    /// Total tokens used
    pub total_tokens: usize,
}

impl MemoryContext {
    /// Create memory context from assembled context
    pub fn from_assembled(assembled: &AssembledContext) -> Self {
        let mut working = Vec::new();
        let mut session = Vec::new();
        let mut project = Vec::new();
        let mut persistent = Vec::new();

        for (chunk, _score) in &assembled.chunks {
            match chunk.tier {
                MemoryTier::Working => working.push(chunk.content.clone()),
                MemoryTier::Session => session.push(chunk.content.clone()),
                MemoryTier::Project => project.push(chunk.content.clone()),
                MemoryTier::Persistent => persistent.push(chunk.content.clone()),
            }
        }

        Self {
            working_memory: working,
            session_history: session,
            project_context: project,
            persistent_knowledge: persistent,
            total_tokens: assembled.total_tokens,
        }
    }

    /// Format as string for prompts
    pub fn format_for_prompt(&self) -> String {
        let mut sections = Vec::new();

        if !self.working_memory.is_empty() {
            sections.push(format!(
                "## Working Memory\n{}",
                self.working_memory.join("\n\n")
            ));
        }

        if !self.session_history.is_empty() {
            sections.push(format!(
                "## Session History\n{}",
                self.session_history.join("\n\n")
            ));
        }

        if !self.project_context.is_empty() {
            sections.push(format!(
                "## Project Context\n{}",
                self.project_context.join("\n\n")
            ));
        }

        if !self.persistent_knowledge.is_empty() {
            sections.push(format!(
                "## Persistent Knowledge\n{}",
                self.persistent_knowledge.join("\n\n")
            ));
        }

        sections.join("\n\n")
    }
}

/// Assembled context for prompts
#[derive(Debug, Clone)]
pub struct AssembledContext {
    pub chunks: Vec<(MemoryChunk, f32)>,
    pub total_tokens: usize,
}

impl AssembledContext {
    /// Create new empty context
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            total_tokens: 0,
        }
    }

    /// Add a chunk
    pub fn add_chunk(&mut self, chunk: MemoryChunk, score: f32) {
        self.total_tokens += chunk.token_count;
        self.chunks.push((chunk, score));
    }

    /// Format as string for prompts
    pub fn format_for_prompt(&self) -> String {
        let mut sections: Vec<String> = Vec::new();
        
        // Group by tier
        let mut working = Vec::new();
        let mut session = Vec::new();
        let mut project = Vec::new();
        let mut persistent = Vec::new();
        
        for (chunk, score) in &self.chunks {
            let formatted = format!("[{} - relevance: {:.2}]\n{}", 
                chunk.id, score, chunk.content);
            
            match chunk.tier {
                MemoryTier::Working => working.push(formatted),
                MemoryTier::Session => session.push(formatted),
                MemoryTier::Project => project.push(formatted),
                MemoryTier::Persistent => persistent.push(formatted),
            }
        }
        
        // Build sections in priority order
        if !working.is_empty() {
            sections.push(format!("## Current Task Context\n{}", working.join("\n\n")));
        }
        if !session.is_empty() {
            sections.push(format!("## Session History\n{}", session.join("\n\n")));
        }
        if !project.is_empty() {
            sections.push(format!("## Project Context\n{}", project.join("\n\n")));
        }
        if !persistent.is_empty() {
            sections.push(format!("## System Information\n{}", persistent.join("\n\n")));
        }
        
        sections.join("\n\n---\n\n")
    }
}

impl Default for AssembledContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Estimate token count for content
fn estimate_token_count(content: &str) -> usize {
    // Rough estimate: ~4 characters per token on average
    content.len() / 4
}

/// Score relevance of a chunk to a query
fn score_relevance(query: &str, chunk: &MemoryChunk) -> f32 {
    let query_lower = query.to_lowercase();
    let content_lower = chunk.content.to_lowercase();
    
    // Exact match bonus
    if content_lower.contains(&query_lower) {
        return 1.0 * chunk.relevance_score;
    }
    
    // Word overlap scoring
    let query_words: std::collections::HashSet<_> = query_lower.split_whitespace().collect();
    let content_words: std::collections::HashSet<_> = content_lower.split_whitespace().collect();
    
    let overlap: f32 = query_words.intersection(&content_words).count() as f32;
    let score = overlap / query_words.len() as f32;
    
    // Tag matching bonus
    let tag_bonus: f32 = chunk.tags.iter()
        .filter(|t| query_lower.contains(&t.to_lowercase()))
        .count() as f32 * 0.1;
    
    (score + tag_bonus).min(1.0) * chunk.relevance_score
}

/// Detect installed tools
async fn detect_installed_tools() -> HashMap<String, String> {
    let mut tools = HashMap::new();
    
    // Check for common tools
    let tool_commands = vec![
        ("cargo", vec!["--version"]),
        ("rustc", vec!["--version"]),
        ("node", vec!["--version"]),
        ("npm", vec!["--version"]),
        ("python", vec!["--version"]),
        ("python3", vec!["--version"]),
        ("git", vec!["--version"]),
        ("docker", vec!["--version"]),
        ("kubectl", vec!["version", "--client"]),
    ];
    
    for (name, args) in tool_commands {
        if let Ok(output) = tokio::process::Command::new(name)
            .args(&args)
            .output()
            .await
        {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                tools.insert(name.to_string(), version);
            }
        }
    }
    
    tools
}

/// Capture relevant environment variables
fn capture_relevant_env_vars() -> HashMap<String, String> {
    let relevant_vars = vec![
        "PATH",
        "HOME",
        "USER",
        "SHELL",
        "EDITOR",
        "LANG",
        "TERM",
        "RUST_VERSION",
        "CARGO_HOME",
        "RUSTUP_HOME",
    ];
    
    let mut vars = HashMap::new();
    for var in relevant_vars {
        if let Ok(value) = std::env::var(var) {
            vars.insert(var.to_string(), value);
        }
    }
    vars
}

/// Detect installed programming languages
async fn detect_installed_languages() -> Vec<String> {
    let mut languages = Vec::new();
    
    let checks = vec![
        ("Rust", "cargo", vec!["--version"]),
        ("Node.js", "node", vec!["--version"]),
        ("Python", "python3", vec!["--version"]),
        ("Go", "go", vec!["version"]),
        ("Java", "java", vec!["--version"]),
    ];
    
    for (name, cmd, args) in checks {
        if tokio::process::Command::new(cmd)
            .args(&args)
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            languages.push(name.to_string());
        }
    }
    
    languages
}

/// Format bytes to human-readable string
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.1} {}", size, UNITS[unit_index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_chunk_creation() {
        let chunk = MemoryChunk::new("test", MemoryTier::Working, ContentType::Code, "fn main() {}");
        assert_eq!(chunk.id, "test");
        assert_eq!(chunk.tier, MemoryTier::Working);
        assert!(chunk.token_count > 0);
    }

    #[test]
    fn test_memory_chunk_tags() {
        let chunk = MemoryChunk::new("test", MemoryTier::Working, ContentType::Code, "content")
            .with_tag("rust")
            .with_tag("main");
        
        assert!(chunk.tags.contains(&"rust".to_string()));
        assert!(chunk.tags.contains(&"main".to_string()));
    }

    #[test]
    fn test_token_estimation() {
        let content = "fn main() { println!(\"Hello\"); }";
        let tokens = estimate_token_count(content);
        assert!(tokens > 0);
        assert!(tokens < content.len());
    }

    #[test]
    fn test_relevance_scoring() {
        let chunk = MemoryChunk::new("test", MemoryTier::Working, ContentType::Code, "fn main() {}")
            .with_tag("rust");
        
        let score = score_relevance("rust function", &chunk);
        assert!(score > 0.0);
    }
}
