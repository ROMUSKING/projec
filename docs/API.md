# API Documentation

This document provides API documentation for the Self-Developing Coding Agent project.

## Table of Contents

- [Agent Core API](#agent-core-api)
- [Intelligence API](#intelligence-api)
- [Analysis API](#analysis-api)
- [Knowledge API](#knowledge-api)
- [Tools API](#tools-api)
- [Configuration API](#configuration-api)
- [Common Types](#common-types)

---

## Agent Core API

### Agent

The main agent struct that orchestrates all operations.

```rust
pub struct Agent {
    orchestrator: Arc<RwLock<Orchestrator>>,
    state_manager: Arc<RwLock<StateManager>>,
    improvement_engine: Arc<RwLock<ImprovementEngine>>,
    evaluation_engine: Arc<RwLock<EvaluationEngine>>,
    telemetry_manager: Arc<RwLock<TelemetryManager>>,
    self_compiler: Option<Arc<RwLock<SelfCompiler>>>,
    task_queue: TaskQueue,
    metrics: Arc<RwLock<AgentMetrics>>,
    modules: Vec<Box<dyn Module>>,
    config: agent_config::AgentConfig,
    shutdown_tx: Option<mpsc::Sender<()>>,
    event_tx: mpsc::Sender<AgentEvent>,
    event_rx: Arc<RwLock<mpsc::Receiver<AgentEvent>>>,
}
```

#### Methods

##### `new`

Creates a new agent with the given configuration.

```rust
pub fn new(config: agent_config::AgentConfig) -> Self
```

**Parameters**:
- `config`: Agent configuration (from `agent_config` module)

**Returns**: A new `Agent` instance

**Example**:
```rust
let config = agent_config::AgentConfig::default();
let agent = Agent::new(config);
```

---

##### `register_module`

Registers a module with the agent.

```rust
pub fn register_module(&mut self, module: Box<dyn Module>)
```

**Parameters**:
- `module`: Module to register (must implement `Module` trait)

**Example**:
```rust
let mut agent = Agent::new(config);
let module = Box::new(MyCustomModule::new());
agent.register_module(module);
```

---

##### `with_orchestrator`

Sets a custom orchestrator for the agent (fluent API).

```rust
pub fn with_orchestrator(mut self, mut orchestrator: Orchestrator) -> Self
```

**Parameters**:
- `orchestrator`: Custom orchestrator instance

**Returns**: Self for method chaining

**Example**:
```rust
let orchestrator = Orchestrator::new();
let agent = Agent::new(config)
    .with_orchestrator(orchestrator);
```

---

##### `initialize`

Initializes the agent and all its modules.

```rust
pub async fn initialize(&mut self) -> Result<()>
```

**Returns**: `Result<()>` - Success or error

**Errors**:
- `AgentError::InitializationError`: If initialization fails

**Example**:
```rust
let mut agent = Agent::new(config);
agent.initialize().await?;
```

---

##### `run`

Runs the agent's main loop.

```rust
pub async fn run(&mut self) -> Result<()>
```

**Returns**: `Result<()>` - Success or error

**Errors**:
- `AgentError::RuntimeError`: If runtime error occurs

**Example**:
```rust
agent.run().await?;
```

---

##### `submit_task`

Submits a task for execution.

```rust
pub async fn submit_task(&mut self, task: Task) -> Result<TaskId>
```

**Parameters**:
- `task`: The task to execute

**Returns**: `Result<TaskId>` - The task ID

**Errors**:
- `AgentError::TaskError`: If task submission fails

**Example**:
```rust
let task = Task::new("Add error handling");
let task_id = agent.submit_task(task).await?;
```

---

##### `trigger_self_improvement`

Triggers a self-improvement cycle.

```rust
pub async fn trigger_self_improvement(&mut self) -> Result<()>
```

**Returns**: `Result<()>` - Success or error

**Errors**:
- `AgentError::ImprovementError`: If improvement fails

**Example**:
```rust
agent.trigger_self_improvement().await?;
```

---

##### `shutdown`

Shuts down the agent gracefully.

```rust
pub async fn shutdown(&mut self) -> Result<()>
```

**Returns**: `Result<()>` - Success or error

**Example**:
```rust
agent.shutdown().await?;
```

---

##### `get_metrics`

Returns the current agent metrics.

```rust
pub async fn get_metrics(&self) -> AgentMetrics
```

**Returns**: `AgentMetrics` - Current metrics

**Example**:
```rust
let metrics = agent.get_metrics().await;
println!("Success rate: {:.1}%", metrics.success_rate * 100.0);
```

---

##### `current_state`

Returns the current agent state.

```rust
pub async fn current_state(&self) -> AgentState
```

**Returns**: `AgentState` - Current state

**Example**:
```rust
let state = agent.current_state().await;
println!("Current state: {:?}", state);
```

---

### Task

Represents a task to be executed.

```rust
pub struct Task {
    pub id: TaskId,
    pub description: String,
    pub priority: TaskPriority,
    pub context: TaskContext,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### Methods

##### `new`

Creates a new task with the given description.

```rust
pub fn new(description: impl Into<String>) -> Self
```

**Parameters**:
- `description`: Task description

**Returns**: A new `Task` instance

**Example**:
```rust
let task = Task::new("Add unit tests for parser");
```

---

##### `with_priority`

Sets the task priority.

```rust
pub fn with_priority(mut self, priority: TaskPriority) -> Self
```

**Parameters**:
- `priority`: Task priority

**Returns**: Self for chaining

**Example**:
```rust
let task = Task::new("Fix critical bug")
    .with_priority(TaskPriority::High);
```

---

##### `with_context`

Sets the task context.

```rust
pub fn with_context(mut self, context: TaskContext) -> Self
```

**Parameters**:
- `context`: Task context

**Returns**: Self for chaining

**Example**:
```rust
let mut context = TaskContext::default();
context.workspace_path = Some(PathBuf::from("./my-project"));
let task = Task::new("Refactor code").with_context(context);
```

---

### TaskPriority

Task priority levels.

```rust
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}
```

### TaskStatus

Task execution status.

```rust
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}
```

### TaskContext

Additional context for task execution.

```rust
pub struct TaskContext {
    pub workspace_path: Option<PathBuf>,
    pub files: Vec<PathBuf>,
    pub environment: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
}
```

### AgentState

Agent execution state.

```rust
pub enum AgentState {
    Idle,
    Analyzing,
    Planning,
    Executing,
    Validating,
    Improving,
    Error(String),
}
```

### AgentMetrics

Agent performance metrics.

```rust
pub struct AgentMetrics {
    pub tasks_submitted: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub success_rate: f64,
    pub average_execution_time_ms: u64,
    pub total_execution_time_ms: u64,
    pub start_time: DateTime<Utc>,
    pub improvements_applied: u64,
    pub improvements_rolled_back: u64,
}
```

---

## Intelligence API

### LlmGateway

Gateway for LLM provider interactions.

```rust
pub trait LlmGateway: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<String>;
    async fn generate_stream(&self, prompt: &str) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>>;
    async fn count_tokens(&self, text: &str) -> Result<usize>;
}
```

#### Methods

##### `generate`

Generates a response from the LLM.

```rust
async fn generate(&self, prompt: &str) -> Result<String>
```

**Parameters**:
- `prompt`: The prompt to send to the LLM

**Returns**: `Result<String>` - The LLM response

**Errors**:
- `IntelligenceError::LlmError`: If LLM request fails

**Example**:
```rust
let response = llm_gateway.generate("Explain Rust ownership").await?;
```

---

##### `generate_stream`

Generates a streaming response from the LLM.

```rust
async fn generate_stream(&self, prompt: &str) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>>
```

**Parameters**:
- `prompt`: The prompt to send to the LLM

**Returns**: A stream of response chunks

**Example**:
```rust
let mut stream = llm_gateway.generate_stream("Write a function").await?;
while let Some(chunk) = stream.next().await {
    print!("{}", chunk?);
}
```

---

##### `count_tokens`

Counts the number of tokens in text.

```rust
async fn count_tokens(&self, text: &str) -> Result<usize>
```

**Parameters**:
- `text`: The text to count tokens for

**Returns**: `Result<usize>` - Token count

**Example**:
```rust
let count = llm_gateway.count_tokens("Hello, world!").await?;
```

---

### PromptManager

Manages prompt templates and optimization.

```rust
pub struct PromptManager {
    templates: HashMap<String, PromptTemplate>,
    versions: HashMap<String, Vec<PromptVersion>>,
}
```

#### Methods

##### `new`

Creates a new prompt manager.

```rust
pub fn new() -> Self
```

**Returns**: A new `PromptManager` instance

**Example**:
```rust
let manager = PromptManager::new();
```

---

##### `register_template`

Registers a prompt template.

```rust
pub fn register_template(&mut self, name: impl Into<String>, template: PromptTemplate)
```

**Parameters**:
- `name`: Template name
- `template`: The template to register

**Example**:
```rust
let template = PromptTemplate::new("Generate {{language}} code for {{task}}");
manager.register_template("code_generation", template);
```

---

##### `render`

Renders a template with the given context.

```rust
pub fn render(&self, name: &str, context: &HashMap<String, String>) -> Result<String>
```

**Parameters**:
- `name`: Template name
- `context`: Template context variables

**Returns**: `Result<String>` - Rendered prompt

**Errors**:
- `IntelligenceError::TemplateError`: If template not found or rendering fails

**Example**:
```rust
let mut context = HashMap::new();
context.insert("language".to_string(), "Rust".to_string());
context.insert("task".to_string(), "sorting".to_string());
let prompt = manager.render("code_generation", &context)?;
```

---

### IntentManager

Parses and interprets user requests.

```rust
pub struct IntentManager {
    llm_gateway: Arc<dyn LlmGateway>,
}
```

#### Methods

##### `new`

Creates a new intent manager.

```rust
pub fn new(llm_gateway: Arc<dyn LlmGateway>) -> Self
```

**Parameters**:
- `llm_gateway`: LLM gateway for intent parsing

**Returns**: A new `IntentManager` instance

**Example**:
```rust
let manager = IntentManager::new(llm_gateway);
```

---

##### `parse_intent`

Parses the intent from a user request.

```rust
pub async fn parse_intent(&self, request: &str) -> Result<Intent>
```

**Parameters**:
- `request`: The user request

**Returns**: `Result<Intent>` - Parsed intent

**Errors**:
- `IntelligenceError::ParseError`: If parsing fails

**Example**:
```rust
let intent = intent_manager.parse_intent("Add error handling to the auth module").await?;
```

---

### Intent

Represents a parsed user intent.

```rust
pub struct Intent {
    pub category: IntentCategory,
    pub description: String,
    pub parameters: HashMap<String, String>,
    pub confidence: f64,
}
```

### IntentCategory

Categories of intents.

```rust
pub enum IntentCategory {
    CodeGeneration,
    CodeModification,
    Analysis,
    Testing,
    Documentation,
    Optimization,
    SelfImprovement,
}
```

---

## Analysis API

### LspManager

Manages LSP server connections and requests.

```rust
pub struct LspManager {
    servers: HashMap<String, LspClient>,
    config: LspConfig,
}
```

#### Methods

##### `new`

Creates a new LSP manager.

```rust
pub fn new(config: LspConfig) -> Self
```

**Parameters**:
- `config`: LSP configuration

**Returns**: A new `LspManager` instance

**Example**:
```rust
let config = LspConfig::default();
let manager = LspManager::new(config);
```

---

##### `start_server`

Starts an LSP server.

```rust
pub async fn start_server(&mut self, name: impl Into<String>, server_config: LspServerConfig) -> Result<()>
```

**Parameters**:
- `name`: Server name
- `server_config`: Server configuration

**Returns**: `Result<()>` - Success or error

**Errors**:
- `AnalysisError::LspError`: If server fails to start

**Example**:
```rust
let config = LspServerConfig {
    command: "rust-analyzer".to_string(),
    args: vec![],
    filetypes: vec!["rust".to_string()],
};
manager.start_server("rust-analyzer", config).await?;
```

---

##### `get_definition`

Gets the definition location for a symbol.

```rust
pub async fn get_definition(&self, server: &str, uri: &str, position: Position) -> Result<Option<Location>>
```

**Parameters**:
- `server`: Server name
- `uri`: Document URI
- `position`: Symbol position

**Returns**: `Result<Option<Location>>` - Definition location

**Example**:
```rust
let location = manager.get_definition("rust-analyzer", "file:///path/to/file.rs", Position::new(10, 5)).await?;
```

---

##### `get_references`

Gets all references to a symbol.

```rust
pub async fn get_references(&self, server: &str, uri: &str, position: Position) -> Result<Vec<Location>>
```

**Parameters**:
- `server`: Server name
- `uri`: Document URI
- `position`: Symbol position

**Returns**: `Result<Vec<Location>>` - Reference locations

**Example**:
```rust
let references = manager.get_references("rust-analyzer", "file:///path/to/file.rs", Position::new(10, 5)).await?;
```

---

##### `get_diagnostics`

Gets diagnostics for a document.

```rust
pub async fn get_diagnostics(&self, server: &str, uri: &str) -> Result<Vec<Diagnostic>>
```

**Parameters**:
- `server`: Server name
- `uri`: Document URI

**Returns**: `Result<Vec<Diagnostic>>` - Diagnostics

**Example**:
```rust
let diagnostics = manager.get_diagnostics("rust-analyzer", "file:///path/to/file.rs").await?;
```

---

### SemanticAnalyzer

Performs semantic analysis on code.

```rust
pub struct SemanticAnalyzer {
    lsp_manager: Arc<LspManager>,
}
```

#### Methods

##### `new`

Creates a new semantic analyzer.

```rust
pub fn new(lsp_manager: Arc<LspManager>) -> Self
```

**Parameters**:
- `lsp_manager`: LSP manager for analysis

**Returns**: A new `SemanticAnalyzer` instance

**Example**:
```rust
let analyzer = SemanticAnalyzer::new(lsp_manager);
```

---

##### `analyze_file`

Analyzes a file semantically.

```rust
pub async fn analyze_file(&self, path: &Path) -> Result<SemanticAnalysis>
```

**Parameters**:
- `path`: File path

**Returns**: `Result<SemanticAnalysis>` - Analysis results

**Errors**:
- `AnalysisError::AnalysisError`: If analysis fails

**Example**:
```rust
let analysis = analyzer.analyze_file(Path::new("src/main.rs")).await?;
```

---

##### `extract_symbols`

Extracts symbols from a file.

```rust
pub async fn extract_symbols(&self, path: &Path) -> Result<Vec<Symbol>>
```

**Parameters**:
- `path`: File path

**Returns**: `Result<Vec<Symbol>>` - Extracted symbols

**Example**:
```rust
let symbols = analyzer.extract_symbols(Path::new("src/main.rs")).await?;
```

---

### SemanticAnalysis

Results of semantic analysis.

```rust
pub struct SemanticAnalysis {
    pub symbols: Vec<Symbol>,
    pub dependencies: Vec<Dependency>,
    pub metrics: CodeMetrics,
    pub diagnostics: Vec<Diagnostic>,
}
```

### Symbol

Represents a code symbol.

```rust
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
    pub documentation: Option<String>,
}
```

### SymbolKind

Types of symbols.

```rust
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Module,
    Variable,
    Constant,
    TypeAlias,
}
```

---

## Knowledge API

### DocumentationManager

Manages documentation lifecycle.

```rust
pub struct DocumentationManager {
    store: DocumentStore,
    vector_store: Arc<VectorStore>,
}
```

#### Methods

##### `new`

Creates a new documentation manager.

```rust
pub fn new(store: DocumentStore, vector_store: Arc<VectorStore>) -> Self
```

**Parameters**:
- `store`: Document store
- `vector_store`: Vector store for semantic search

**Returns**: A new `DocumentationManager` instance

**Example**:
```rust
let manager = DocumentationManager::new(store, vector_store);
```

---

##### `add_document`

Adds a document to the store.

```rust
pub async fn add_document(&mut self, document: Document) -> Result<DocumentId>
```

**Parameters**:
- `document`: Document to add

**Returns**: `Result<DocumentId>` - Document ID

**Errors**:
- `KnowledgeError::StorageError`: If storage fails

**Example**:
```rust
let doc = Document::new("API Reference", "# API Reference\n...");
let doc_id = manager.add_document(doc).await?;
```

---

##### `search`

Searches for documents.

```rust
pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>>
```

**Parameters**:
- `query`: Search query

**Returns**: `Result<Vec<SearchResult>>` - Search results

**Example**:
```rust
let results = manager.search("authentication").await?;
```

---

### KnowledgeGraph

Manages the knowledge graph.

```rust
pub struct KnowledgeGraph {
    graph: Graph,
}
```

#### Methods

##### `new`

Creates a new knowledge graph.

```rust
pub fn new() -> Self
```

**Returns**: A new `KnowledgeGraph` instance

**Example**:
```rust
let graph = KnowledgeGraph::new();
```

---

##### `add_node`

Adds a node to the graph.

```rust
pub fn add_node(&mut self, node: Node) -> NodeId
```

**Parameters**:
- `node`: Node to add

**Returns**: `NodeId` - Node ID

**Example**:
```rust
let node = Node::new("User", NodeType::Struct);
let node_id = graph.add_node(node);
```

---

##### `add_edge`

Adds an edge between nodes.

```rust
pub fn add_edge(&mut self, from: NodeId, to: NodeId, edge_type: EdgeType)
```

**Parameters**:
- `from`: Source node ID
- `to`: Target node ID
- `edge_type`: Edge type

**Example**:
```rust
graph.add_edge(user_id, auth_id, EdgeType::Uses);
```

---

##### `query`

Queries the graph.

```rust
pub fn query(&self, query: GraphQuery) -> Result<Vec<Node>>
```

**Parameters**:
- `query`: Graph query

**Returns**: `Result<Vec<Node>>` - Query results

**Example**:
```rust
let query = GraphQuery::related_to(user_id);
let results = graph.query(query)?;
```

---

### VectorStore

Manages vector embeddings.

```rust
pub trait VectorStore: Send + Sync {
    async fn add_embedding(&mut self, id: String, embedding: Vec<f32>) -> Result<()>;
    async fn search(&self, query: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>>;
    async fn delete(&mut self, id: &str) -> Result<()>;
}
```

#### Methods

##### `add_embedding`

Adds an embedding to the store.

```rust
async fn add_embedding(&mut self, id: String, embedding: Vec<f32>) -> Result<()>
```

**Parameters**:
- `id`: Embedding ID
- `embedding`: Embedding vector

**Returns**: `Result<()>` - Success or error

**Example**:
```rust
let embedding = vec![0.1, 0.2, 0.3, ...];
vector_store.add_embedding("doc1".to_string(), embedding).await?;
```

---

##### `search`

Searches for similar embeddings.

```rust
async fn search(&self, query: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>>
```

**Parameters**:
- `query`: Query embedding
- `limit`: Maximum results

**Returns**: `Result<Vec<SearchResult>>` - Search results

**Example**:
```rust
let query = vec![0.1, 0.2, 0.3, ...];
let results = vector_store.search(query, 10).await?;
```

---

## Tools API

### Tool

Trait for tool implementations.

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Vec<Parameter>;
    fn returns(&self) -> ReturnType;
    
    async fn execute(&self, ctx: &Context, args: Value) -> Result<Value>;
    
    fn validate(&self, args: &Value) -> Result<()>;
    fn is_safe(&self, args: &Value, ctx: &Context) -> bool;
}
```

#### Methods

##### `name`

Returns the tool name.

```rust
fn name(&self) -> &str
```

**Returns**: Tool name

---

##### `description`

Returns the tool description.

```rust
fn description(&self) -> &str
```

**Returns**: Tool description

---

##### `parameters`

Returns the tool parameters.

```rust
fn parameters(&self) -> Vec<Parameter>
```

**Returns**: Tool parameters

---

##### `execute`

Executes the tool.

```rust
async fn execute(&self, ctx: &Context, args: Value) -> Result<Value>
```

**Parameters**:
- `ctx`: Execution context
- `args`: Tool arguments

**Returns**: `Result<Value>` - Execution result

**Errors**: Tool-specific errors

**Example**:
```rust
let result = tool.execute(&context, json!({"path": "file.txt"})).await?;
```

---

### ToolFramework

Manages tool registration and execution.

```rust
pub struct ToolFramework {
    tools: HashMap<String, Box<dyn Tool>>,
    sandbox: Sandbox,
}
```

#### Methods

##### `new`

Creates a new tool framework.

```rust
pub fn new() -> Self
```

**Returns**: A new `ToolFramework` instance

**Example**:
```rust
let framework = ToolFramework::new();
```

---

##### `register_tool`

Registers a tool.

```rust
pub fn register_tool(&mut self, tool: Box<dyn Tool>)
```

**Parameters**:
- `tool`: Tool to register

**Example**:
```rust
let file_tool = Box::new(FileSystemTool::new());
framework.register_tool(file_tool);
```

---

##### `execute_tool`

Executes a tool by name.

```rust
pub async fn execute_tool(&self, name: &str, args: Value, ctx: &Context) -> Result<Value>
```

**Parameters**:
- `name`: Tool name
- `args`: Tool arguments
- `ctx`: Execution context

**Returns**: `Result<Value>` - Execution result

**Errors**:
- `ToolsError::ToolNotFound`: If tool not found
- `ToolsError::ExecutionError`: If execution fails

**Example**:
```rust
let result = framework.execute_tool("read_file", json!({"path": "file.txt"}), &context).await?;
```

---

### Built-in Tools

#### FileSystemTool

File system operations.

```rust
pub struct FileSystemTool {
    // ...
}
```

**Operations**:
- `read_file`: Read file contents
- `write_file`: Write file contents
- `edit_file`: Edit file
- `list_files`: List directory contents
- `search_files`: Search files
- `delete_file`: Delete file

#### GitTool

Git operations.

```rust
pub struct GitTool {
    // ...
}
```

**Operations**:
- `git_status`: Get repository status
- `git_diff`: Show changes
- `git_commit`: Create commit
- `git_branch`: Branch operations
- `git_log`: Commit history
- `git_push`: Push to remote

#### HttpTool

HTTP operations.

```rust
pub struct HttpTool {
    // ...
}
```

**Operations**:
- `http_get`: GET request
- `http_post`: POST request
- `http_request`: Custom request

---

## Configuration API

### AgentConfig

Main agent configuration.

```rust
pub struct AgentConfig {
    pub name: String,
    pub version: String,
    pub improvement_interval: Duration,
    pub max_concurrent_tasks: usize,
    pub log_level: String,
    pub self_improvement: SelfImprovementConfig,
    pub llm: LlmConfig,
    pub lsp: LspConfig,
    pub safety: SafetyConfig,
    pub tools: ToolsConfig,
}
```

#### Methods

##### `default`

Creates default configuration.

```rust
impl Default for AgentConfig {
    fn default() -> Self
}
```

**Returns**: Default configuration

**Example**:
```rust
let config = AgentConfig::default();
```

---

##### `load`

Loads configuration from file.

```rust
pub async fn load(path: Option<PathBuf>, overrides: ConfigOverrides) -> Result<Self>
```

**Parameters**:
- `path`: Configuration file path
- `overrides`: Configuration overrides

**Returns**: `Result<Self>` - Loaded configuration

**Errors**:
- `ConfigError::LoadError`: If loading fails

**Example**:
```rust
let config = AgentConfig::load(Some(PathBuf::from("config.toml")), overrides).await?;
```

---

### ConfigOverrides

Configuration overrides.

```rust
pub struct ConfigOverrides {
    pub log_level: Option<String>,
    pub workspace: Option<PathBuf>,
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
    pub llm_api_key: Option<String>,
    pub llm_base_url: Option<String>,
    pub llm_temperature: Option<f64>,
    pub llm_max_tokens: Option<usize>,
    // ... other fields
}
```

---

## Common Types

### Result

Standard result type.

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

### Error

Error types.

```rust
pub enum Error {
    AgentError(AgentError),
    IntelligenceError(IntelligenceError),
    AnalysisError(AnalysisError),
    KnowledgeError(KnowledgeError),
    ToolsError(ToolsError),
    ConfigError(ConfigError),
}
```

### Position

Position in a document.

```rust
pub struct Position {
    pub line: u32,
    pub character: u32,
}
```

### Location

Location in a document.

```rust
pub struct Location {
    pub uri: String,
    pub range: Range,
}
```

### Range

Range in a document.

```rust
pub struct Range {
    pub start: Position,
    pub end: Position,
}
```

### Diagnostic

Diagnostic information.

```rust
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: Option<String>,
}
```

---

## Error Handling

All API methods return `Result<T>` for error handling.

### Error Types

- `AgentError`: Agent-related errors
- `IntelligenceError`: LLM and intelligence errors
- `AnalysisError`: LSP and analysis errors
- `KnowledgeError`: Knowledge store errors
- `ToolsError`: Tool execution errors
- `ConfigError`: Configuration errors

### Example Error Handling

```rust
use agent_core::Agent;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AgentConfig::default();
    let mut agent = Agent::new(config);
    
    agent.initialize().await?;
    
    let task = Task::new("Add error handling");
    match agent.submit_task(task).await {
        Ok(task_id) => println!("Task submitted: {}", task_id),
        Err(e) => eprintln!("Failed to submit task: {}", e),
    }
    
    Ok(())
}
```

---

## Examples

### Basic Usage

```rust
use agent_core::{Agent, Task, TaskPriority};

#[tokio::main]
async fn main() -> Result<()> {
    // Create agent
    let config = AgentConfig::default();
    let mut agent = Agent::new(config);
    agent.initialize().await?;
    
    // Submit task
    let task = Task::new("Add unit tests")
        .with_priority(TaskPriority::High);
    let task_id = agent.submit_task(task).await?;
    
    // Run agent
    agent.run().await?;
    
    Ok(())
}
```

### Using LLM Gateway

```rust
use intelligence::{LlmGateway, AnthropicGateway};

#[tokio::main]
async fn main() -> Result<()> {
    let gateway = AnthropicGateway::new("api_key")?;
    let response = gateway.generate("Explain Rust ownership").await?;
    println!("{}", response);
    Ok(())
}
```

### Using LSP Manager

```rust
use analysis::{LspManager, LspConfig, LspServerConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let config = LspConfig::default();
    let mut manager = LspManager::new(config);
    
    let server_config = LspServerConfig {
        command: "rust-analyzer".to_string(),
        args: vec![],
        filetypes: vec!["rust".to_string()],
    };
    
    manager.start_server("rust-analyzer", server_config).await?;
    
    let diagnostics = manager.get_diagnostics("rust-analyzer", "file:///path/to/file.rs").await?;
    for diag in diagnostics {
        println!("{}: {}", diag.range.start, diag.message);
    }
    
    Ok(())
}
```

---

For more information, see:
- [Architecture Documentation](ARCHITECTURE.md)
- [Development Guide](DEVELOPMENT.md)
- [Roadmap](ROADMAP.md)
- [Project README](../README.md)
