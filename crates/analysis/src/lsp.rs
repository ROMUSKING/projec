//! LSP (Language Server Protocol) client implementation.
//!
//! This module manages language server connections and provides
//! a unified interface for code analysis.

use common::{async_trait, Error, Result};
use dashmap::DashMap;
use lsp_types::*;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, oneshot, Mutex, RwLock as TokioRwLock};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

/// LSP Manager for handling multiple language servers
pub struct LspManager {
    servers: DashMap<String, Arc<LanguageServerInstance>>,
    registry: ServerRegistry,
    document_cache: Arc<DocumentCache>,
    workspace_roots: RwLock<Vec<PathBuf>>,
}

impl LspManager {
    pub fn new() -> Self {
        Self {
            servers: DashMap::new(),
            registry: ServerRegistry::new(),
            document_cache: Arc::new(DocumentCache::new()),
            workspace_roots: RwLock::new(Vec::new()),
        }
    }

    /// Add a workspace root for LSP servers
    pub fn add_workspace_root(&self, root: PathBuf) {
        self.workspace_roots.write().push(root);
    }

    /// Initialize all configured language servers
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing LSP manager");
        
        // Pre-configure common language servers
        let configs = vec![
            LanguageServerConfig {
                name: "rust-analyzer".to_string(),
                command: "rust-analyzer".to_string(),
                args: vec![],
                filetypes: vec!["rs".to_string()],
                root_patterns: vec!["Cargo.toml".to_string()],
                settings: None,
                connection_type: ConnectionType::Stdio,
            },
            LanguageServerConfig {
                name: "typescript-language-server".to_string(),
                command: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
                filetypes: vec!["ts".to_string(), "tsx".to_string(), "js".to_string(), "jsx".to_string()],
                root_patterns: vec!["package.json".to_string(), "tsconfig.json".to_string()],
                settings: None,
                connection_type: ConnectionType::Stdio,
            },
            LanguageServerConfig {
                name: "pylsp".to_string(),
                command: "pylsp".to_string(),
                args: vec![],
                filetypes: vec!["py".to_string()],
                root_patterns: vec!["setup.py".to_string(), "pyproject.toml".to_string(), "requirements.txt".to_string()],
                settings: None,
                connection_type: ConnectionType::Stdio,
            },
            LanguageServerConfig {
                name: "gopls".to_string(),
                command: "gopls".to_string(),
                args: vec![],
                filetypes: vec!["go".to_string()],
                root_patterns: vec!["go.mod".to_string()],
                settings: None,
                connection_type: ConnectionType::Stdio,
            },
        ];

        for config in configs {
            if self.is_server_available(&config.command).await {
                match self.start_server(config.clone()).await {
                    Ok(_) => info!("Started language server: {}", config.name),
                    Err(e) => warn!("Failed to start {}: {}", config.name, e),
                }
            } else {
                debug!("Language server not available: {}", config.command);
            }
        }

        Ok(())
    }

    /// Check if a language server command is available
    async fn is_server_available(&self, command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Start a language server with the given configuration
    pub async fn start_server(&self, config: LanguageServerConfig) -> Result<()> {
        let server = LanguageServerInstance::new(config.clone(), self.document_cache.clone()).await?;
        let server_arc = Arc::new(server);
        
        // Initialize the server
        server_arc.initialize().await?;
        
        self.servers.insert(config.name.clone(), server_arc);
        
        // Register filetype mappings
        for filetype in &config.filetypes {
            self.registry.register_mapping(filetype.clone(), config.name.clone());
        }
        
        Ok(())
    }

    /// Shutdown all language servers
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down all language servers");
        
        for entry in self.servers.iter() {
            let (name, server) = entry.pair();
            if let Err(e) = server.shutdown().await {
                warn!("Error shutting down {}: {}", name, e);
            }
        }
        
        self.servers.clear();
        Ok(())
    }

    /// Restart a specific language server
    pub async fn restart_server(&self, name: &str) -> Result<()> {
        if let Some((_, server)) = self.servers.remove(name) {
            server.shutdown().await?;
        }
        
        // TODO: Re-read config and restart
        info!("Restarted language server: {}", name);
        Ok(())
    }

    /// Analyze a file using the appropriate language server
    pub async fn analyze_file(&self, path: &PathBuf) -> Result<super::LspAnalysis> {
        let server = self.get_server_for_file(path).await?;
        server.analyze_document(path).await
    }

    /// Find references to a symbol
    pub async fn find_references(&self, path: &PathBuf, line: u32, column: u32) -> Result<Vec<super::Reference>> {
        let server = self.get_server_for_file(path).await?;
        server.find_references(path, line, column).await
    }

    /// Get diagnostics for a file
    pub async fn get_diagnostics(&self, path: &PathBuf) -> Result<Vec<super::Diagnostic>> {
        let server = self.get_server_for_file(path).await?;
        server.get_diagnostics(path).await
    }

    /// Go to definition
    pub async fn goto_definition(&self, path: &PathBuf, line: u32, column: u32) -> Result<Option<super::Location>> {
        let server = self.get_server_for_file(path).await?;
        server.goto_definition(path, line, column).await
    }

    /// Get hover information
    pub async fn hover(&self, path: &PathBuf, line: u32, column: u32) -> Result<Option<String>> {
        let server = self.get_server_for_file(path).await?;
        server.hover(path, line, column).await
    }

    /// Get completions at a position
    pub async fn complete(&self, path: &PathBuf, line: u32, column: u32) -> Result<Vec<super::CompletionItem>> {
        let server = self.get_server_for_file(path).await?;
        server.complete(path, line, column).await
    }

    /// Execute code action
    pub async fn execute_code_action(&self, path: &PathBuf, action: &str) -> Result<()> {
        let server = self.get_server_for_file(path).await?;
        server.execute_code_action(path, action).await
    }

    /// Get workspace symbols
    pub async fn workspace_symbol(&self, query: &str) -> Result<Vec<super::Symbol>> {
        let mut all_symbols = Vec::new();
        
        for entry in self.servers.iter() {
            let server = entry.value();
            match server.workspace_symbol(query).await {
                Ok(symbols) => all_symbols.extend(symbols),
                Err(e) => warn!("Error getting workspace symbols: {}", e),
            }
        }
        
        Ok(all_symbols)
    }

    /// Open a document for synchronization
    pub async fn open_document(&self, path: &PathBuf, content: String, version: i32) -> Result<()> {
        let server = self.get_server_for_file(path).await?;
        self.document_cache.insert(path.clone(), content.clone(), version);
        server.open_document(path, content, version).await
    }

    /// Update a document (incremental or full)
    pub async fn update_document(&self, path: &PathBuf, content: String, version: i32) -> Result<()> {
        let server = self.get_server_for_file(path).await?;
        self.document_cache.update(path.clone(), content.clone(), version);
        server.update_document(path, content, version).await
    }

    /// Close a document
    pub async fn close_document(&self, path: &PathBuf) -> Result<()> {
        let server = self.get_server_for_file(path).await?;
        self.document_cache.remove(path);
        server.close_document(path).await
    }

    /// Get the appropriate language server for a file
    async fn get_server_for_file(&self, path: &PathBuf) -> Result<Arc<LanguageServerInstance>> {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| Error::Validation("Could not determine file extension".to_string()))?;

        let server_name = self
            .registry
            .get_server_for_extension(extension)
            .ok_or_else(|| Error::NotFound(format!("No language server for extension: {}", extension)))?;

        self.servers
            .get(&server_name)
            .map(|s| s.clone())
            .ok_or_else(|| Error::NotFound(format!("Language server not initialized: {}", server_name)))
    }
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Language server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub filetypes: Vec<String>,
    pub root_patterns: Vec<String>,
    pub settings: Option<serde_json::Value>,
    pub connection_type: ConnectionType,
}

/// Connection type for LSP server
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionType {
    Stdio,
    Tcp,
    Socket,
}

/// Server registry for mapping file types to servers
pub struct ServerRegistry {
    mappings: RwLock<HashMap<String, String>>, // extension -> server name
}

impl ServerRegistry {
    pub fn new() -> Self {
        Self {
            mappings: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_server_for_extension(&self, extension: &str) -> Option<String> {
        self.mappings.read().get(extension).cloned()
    }

    pub fn register_mapping(&self, extension: String, server: String) {
        self.mappings.write().insert(extension, server);
    }

    pub fn unregister_mapping(&self, extension: &str) {
        self.mappings.write().remove(extension);
    }
}

impl Default for ServerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Document cache for tracking open documents
pub struct DocumentCache {
    documents: DashMap<PathBuf, DocumentState>,
}

#[derive(Debug, Clone)]
struct DocumentState {
    content: String,
    version: i32,
    language_id: String,
}

impl DocumentCache {
    pub fn new() -> Self {
        Self {
            documents: DashMap::new(),
        }
    }

    pub fn insert(&self, path: PathBuf, content: String, version: i32) {
        let language_id = Self::detect_language_id(&path);
        self.documents.insert(path, DocumentState {
            content,
            version,
            language_id,
        });
    }

    pub fn update(&self, path: PathBuf, content: String, version: i32) {
        if let Some(mut doc) = self.documents.get_mut(&path) {
            doc.content = content;
            doc.version = version;
        }
    }

    pub fn remove(&self, path: &PathBuf) {
        self.documents.remove(path);
    }

    pub fn get(&self, path: &PathBuf) -> Option<DocumentState> {
        self.documents.get(path).map(|d| d.clone())
    }

    fn detect_language_id(path: &Path) -> String {
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => "rust".to_string(),
            Some("ts") => "typescript".to_string(),
            Some("tsx") => "typescriptreact".to_string(),
            Some("js") => "javascript".to_string(),
            Some("jsx") => "javascriptreact".to_string(),
            Some("py") => "python".to_string(),
            Some("go") => "go".to_string(),
            Some("java") => "java".to_string(),
            Some("c") => "c".to_string(),
            Some("cpp") | Some("cc") | Some("cxx") => "cpp".to_string(),
            Some("h") | Some("hpp") => "c".to_string(),
            _ => "plaintext".to_string(),
        }
    }
}

impl Default for DocumentCache {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON-RPC message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum JsonRpcMessage {
    Request {
        jsonrpc: String,
        id: Option<Value>,
        method: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<Value>,
    },
    Response {
        jsonrpc: String,
        id: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<JsonRpcError>,
    },
    Notification {
        jsonrpc: String,
        method: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// Pending request tracking
type ResponseCallback = oneshot::Sender<Result<Value>>;

/// Language server instance managing a single LSP connection
pub struct LanguageServerInstance {
    config: LanguageServerConfig,
    request_id: AtomicI64,
    pending_requests: Arc<DashMap<i64, ResponseCallback>>,
    stdin: Arc<Mutex<ChildStdin>>,
    _stdout: Arc<Mutex<ChildStdout>>,
    _child: Arc<Mutex<Child>>,
    notification_tx: mpsc::UnboundedSender<JsonRpcMessage>,
    server_capabilities: RwLock<Option<ServerCapabilities>>,
    document_cache: Arc<DocumentCache>,
}

impl LanguageServerInstance {
    pub async fn new(config: LanguageServerConfig, document_cache: Arc<DocumentCache>) -> Result<Self> {
        let mut child = Command::new(&config.command)
            .args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::ExternalService(format!("Failed to start {}: {}", config.command, e)))?;

        let stdin = child.stdin.take().ok_or_else(|| {
            Error::ExternalService("Failed to get stdin".to_string())
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            Error::ExternalService("Failed to get stdout".to_string())
        })?;

        let (notification_tx, mut notification_rx) = mpsc::unbounded_channel();
        let pending_requests = Arc::new(DashMap::new());

        let instance = Self {
            config,
            request_id: AtomicI64::new(1),
            pending_requests: pending_requests.clone(),
            stdin: Arc::new(Mutex::new(stdin)),
            _stdout: Arc::new(Mutex::new(stdout)),
            _child: Arc::new(Mutex::new(child)),
            notification_tx,
            server_capabilities: RwLock::new(None),
            document_cache,
        };

        // Start the message reading loop
        let stdout = instance._stdout.clone();
        let pending_requests = instance.pending_requests.clone();
        tokio::spawn(async move {
            let mut stdout = stdout.lock().await;
            let mut reader = BufReader::new(&mut *stdout);
            
            loop {
                match Self::read_message(&mut reader).await {
                    Ok(message) => {
                        match message {
                            JsonRpcMessage::Response { id, result, error, .. } => {
                                if let Some(id_num) = id.as_i64() {
                                    if let Some((_, callback)) = pending_requests.remove(&id_num) {
                                        if let Some(err) = error {
                                            let _ = callback.send(Err(Error::ExternalService(err.message)));
                                        } else {
                                            let val: Result<Value> = Ok(result.unwrap_or(Value::Null));
                                            let _ = callback.send(val);
                                        }
                                    }
                                }
                            }
                            JsonRpcMessage::Notification { method, params, .. } => {
                                debug!("Received notification: {}", method);
                                // Handle notifications like textDocument/publishDiagnostics
                                if method == "textDocument/publishDiagnostics" {
                                    if let Some(params) = params {
                                        debug!("Diagnostics: {:?}", params);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("Error reading LSP message: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(instance)
    }

    /// Read a JSON-RPC message from the LSP server
    async fn read_message<R: AsyncBufReadExt + Unpin>(reader: &mut R) -> Result<JsonRpcMessage> {
        // Read headers
        let mut content_length: Option<usize> = None;
        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line).await?;
            
            if bytes_read == 0 {
                return Err(Error::ExternalService("Connection closed".to_string()));
            }
            
            let line = line.trim();
            if line.is_empty() {
                break;
            }
            
            if let Some(value) = line.strip_prefix("Content-Length: ") {
                content_length = value.parse().ok();
            }
        }
        
        let length = content_length.ok_or_else(|| {
            Error::ExternalService("Missing Content-Length header".to_string())
        })?;
        
        // Read body
        let mut buffer = vec![0u8; length];
        reader.read_exact(&mut buffer).await?;
        
        let message: JsonRpcMessage = serde_json::from_slice(&buffer)
            .map_err(|e| Error::Serialization(e))?;
        
        Ok(message)
    }

    /// Send a JSON-RPC message to the LSP server
    async fn send_message(&self, message: JsonRpcMessage) -> Result<()> {
        let body = serde_json::to_vec(&message)?;
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        
        let mut stdin = self.stdin.lock().await;
        stdin.write_all(header.as_bytes()).await?;
        stdin.write_all(&body).await?;
        stdin.flush().await?;
        
        Ok(())
    }

    /// Send a request and wait for response
    async fn request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        
        let message = JsonRpcMessage::Request {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::from(id)),
            method: method.to_string(),
            params,
        };
        
        let (tx, rx) = oneshot::channel();
        self.pending_requests.insert(id, tx);
        
        self.send_message(message).await?;
        
        match timeout(Duration::from_secs(30), rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(Error::ExternalService("Request cancelled".to_string())),
            Err(_) => {
                self.pending_requests.remove(&id);
                Err(Error::Timeout(format!("Request {} timed out", method)))
            }
        }
    }

    /// Send a notification (no response expected)
    async fn notify(&self, method: &str, params: Option<Value>) -> Result<()> {
        let message = JsonRpcMessage::Notification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        };
        
        self.send_message(message).await
    }

    /// Initialize the language server
    pub async fn initialize(&self) -> Result<()> {
        let workspace_roots = vec![WorkspaceFolder {
            uri: Url::from_file_path(std::env::current_dir()?).map_err(|_| {
                Error::Internal("Invalid workspace path".to_string())
            })?,
            name: "workspace".to_string(),
        }];

        let params = InitializeParams {
            process_id: Some(std::process::id() as u32),
            root_path: None,
            root_uri: workspace_roots.first().map(|w| w.uri.clone()),
            initialization_options: self.config.settings.clone(),
            capabilities: ClientCapabilities {
                workspace: Some(WorkspaceClientCapabilities {
                    workspace_folders: Some(true),
                    configuration: Some(true),
                    did_change_configuration: Some(DynamicRegistrationClientCapabilities {
                        dynamic_registration: Some(true),
                    }),
                    ..Default::default()
                }),
                text_document: Some(TextDocumentClientCapabilities {
                    synchronization: Some(TextDocumentSyncClientCapabilities {
                        dynamic_registration: Some(true),
                        will_save: Some(true),
                        will_save_wait_until: Some(true),
                        did_save: Some(true),
                    }),
                    completion: Some(CompletionClientCapabilities {
                        dynamic_registration: Some(true),
                        completion_item: Some(CompletionItemCapability {
                            snippet_support: Some(true),
                            commit_characters_support: Some(true),
                            documentation_format: Some(vec![MarkupKind::Markdown, MarkupKind::PlainText]),
                            deprecated_support: Some(true),
                            preselect_support: Some(true),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    hover: Some(HoverClientCapabilities {
                        dynamic_registration: Some(true),
                        content_format: Some(vec![MarkupKind::Markdown, MarkupKind::PlainText]),
                    }),
                    definition: Some(GotoCapability {
                        dynamic_registration: Some(true),
                        link_support: Some(true),
                    }),
                    document_symbol: Some(DocumentSymbolClientCapabilities {
                        dynamic_registration: Some(true),
                        hierarchical_document_symbol_support: Some(true),
                        ..Default::default()
                    }),
                    code_action: Some(CodeActionClientCapabilities {
                        dynamic_registration: Some(true),
                        code_action_literal_support: Some(CodeActionLiteralSupport {
                            code_action_kind: CodeActionKindLiteralSupport {
                                value_set: vec![
                                    CodeActionKind::QUICKFIX.as_str().to_string(),
                                    CodeActionKind::REFACTOR.as_str().to_string(),
                                    CodeActionKind::SOURCE.as_str().to_string(),
                                ],
                            },
                        }),
                        ..Default::default()
                    }),
                    formatting: Some(DynamicRegistrationClientCapabilities {
                        dynamic_registration: Some(true),
                    }),
                    rename: Some(RenameClientCapabilities {
                        dynamic_registration: Some(true),
                        prepare_support: Some(true),
                        prepare_support_default_behavior: None,
                        honors_change_annotations: None,
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            trace: None,
            workspace_folders: Some(workspace_roots),
            client_info: Some(ClientInfo {
                name: "coding-agent".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            locale: None,
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let result = self.request("initialize", Some(serde_json::to_value(params)?)).await?;
        
        let init_result: InitializeResult = serde_json::from_value(result)?;
        *self.server_capabilities.write() = Some(init_result.capabilities);
        
        // Send initialized notification
        self.notify("initialized", Some(serde_json::json!({}))).await?;
        
        info!("Language server initialized: {}", self.config.name);
        Ok(())
    }

    /// Shutdown the language server
    pub async fn shutdown(&self) -> Result<()> {
        // Send shutdown request
        let _ = self.request("shutdown", None).await;
        
        // Send exit notification
        let _ = self.notify("exit", None).await;
        
        info!("Language server shutdown: {}", self.config.name);
        Ok(())
    }

    /// Open a document
    pub async fn open_document(&self, path: &PathBuf, content: String, version: i32) -> Result<()> {
        let uri = Url::from_file_path(path).map_err(|_| {
            Error::Validation("Invalid file path".to_string())
        })?;

        let language_id = DocumentCache::detect_language_id(path);

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id,
                version,
                text: content,
            },
        };

        self.notify("textDocument/didOpen", Some(serde_json::to_value(params)?)).await
    }

    /// Update a document
    pub async fn update_document(&self, path: &PathBuf, content: String, version: i32) -> Result<()> {
        let uri = Url::from_file_path(path).map_err(|_| {
            Error::Validation("Invalid file path".to_string())
        })?;

        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri,
                version,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: content,
            }],
        };

        self.notify("textDocument/didChange", Some(serde_json::to_value(params)?)).await
    }

    /// Close a document
    pub async fn close_document(&self, path: &PathBuf) -> Result<()> {
        let uri = Url::from_file_path(path).map_err(|_| {
            Error::Validation("Invalid file path".to_string())
        })?;

        let params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri },
        };

        self.notify("textDocument/didClose", Some(serde_json::to_value(params)?)).await
    }

    /// Analyze a document
    pub async fn analyze_document(&self, path: &PathBuf) -> Result<super::LspAnalysis> {
        let uri = Url::from_file_path(path).map_err(|_| {
            Error::Validation("Invalid file path".to_string())
        })?;

        // Get document symbols
        let symbols = self.get_document_symbols(&uri).await?;
        
        // Get diagnostics (from cache or request)
        let diagnostics = self.get_diagnostics(path).await?;

        Ok(super::LspAnalysis {
            symbols,
            diagnostics,
            document_links: vec![],
        })
    }

    /// Get document symbols
    async fn get_document_symbols(&self, uri: &Url) -> Result<Vec<super::Symbol>> {
        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = self.request("textDocument/documentSymbol", Some(serde_json::to_value(params)?)).await?;
        
        // Parse document symbols (handle both flat and hierarchical)
        let mut symbols = Vec::new();
        
        if let Ok(document_symbols) = serde_json::from_value::<Vec<DocumentSymbol>>(result.clone()) {
            for ds in document_symbols {
                self.convert_document_symbol(&ds, &mut symbols);
            }
        } else if let Ok(symbol_informations) = serde_json::from_value::<Vec<SymbolInformation>>(result) {
            for si in symbol_informations {
                symbols.push(super::Symbol {
                    name: si.name,
                    kind: convert_symbol_kind(si.kind),
                    location: convert_location(&si.location),
                    documentation: None,
                });
            }
        }

        Ok(symbols)
    }

    fn convert_document_symbol(&self, ds: &DocumentSymbol, symbols: &mut Vec<super::Symbol>) {
        symbols.push(super::Symbol {
            name: ds.name.clone(),
            kind: convert_symbol_kind(ds.kind),
            location: convert_range(&ds.range, &ds.selection_range),
            documentation: ds.detail.clone(),
        });

        if let Some(children) = &ds.children {
            for child in children {
                self.convert_document_symbol(child, symbols);
            }
        }
    }

    /// Find references to a symbol
    pub async fn find_references(&self, path: &PathBuf, line: u32, column: u32) -> Result<Vec<super::Reference>> {
        let uri = Url::from_file_path(path).map_err(|_| {
            Error::Validation("Invalid file path".to_string())
        })?;

        let params = ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line, character: column },
            },
            context: ReferenceContext {
                include_declaration: true,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = self.request("textDocument/references", Some(serde_json::to_value(params)?)).await?;
        
        let locations: Vec<Location> = serde_json::from_value(result)?;
        
        Ok(locations.into_iter().map(|loc| super::Reference {
            symbol: String::new(), // Would need to resolve symbol name
            location: convert_lsp_location(&loc),
            is_definition: false,
        }).collect())
    }

    /// Get diagnostics for a file
    pub async fn get_diagnostics(&self, path: &PathBuf) -> Result<Vec<super::Diagnostic>> {
        // This would typically come from cached diagnostics published by the server
        // For now, return empty
        Ok(vec![])
    }

    /// Go to definition
    pub async fn goto_definition(&self, path: &PathBuf, line: u32, column: u32) -> Result<Option<super::Location>> {
        let uri = Url::from_file_path(path).map_err(|_| {
            Error::Validation("Invalid file path".to_string())
        })?;

        let params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line, character: column },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = self.request("textDocument/definition", Some(serde_json::to_value(params)?)).await?;
        
        if result.is_null() {
            return Ok(None);
        }

        // Handle both Location and Vec<Location>
        if let Ok(location) = serde_json::from_value::<Location>(result.clone()) {
            return Ok(Some(convert_lsp_location(&location)));
        }
        
        if let Ok(locations) = serde_json::from_value::<Vec<Location>>(result) {
            return Ok(locations.first().map(convert_lsp_location));
        }

        Ok(None)
    }

    /// Get hover information
    pub async fn hover(&self, path: &PathBuf, line: u32, column: u32) -> Result<Option<String>> {
        let uri = Url::from_file_path(path).map_err(|_| {
            Error::Validation("Invalid file path".to_string())
        })?;

        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line, character: column },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let result = self.request("textDocument/hover", Some(serde_json::to_value(params)?)).await?;
        
        if result.is_null() {
            return Ok(None);
        }

        let hover: Hover = serde_json::from_value(result)?;
        
        let content = match hover.contents {
            HoverContents::Scalar(MarkedString::String(s)) => s,
            HoverContents::Scalar(MarkedString::LanguageString(s)) => s.value,
            HoverContents::Array(arr) => arr.into_iter()
                .map(|ms| match ms {
                    MarkedString::String(s) => s,
                    MarkedString::LanguageString(s) => s.value,
                })
                .collect::<Vec<_>>()
                .join("\n"),
            HoverContents::Markup(markup) => markup.value,
        };

        Ok(Some(content))
    }

    /// Get completions at a position
    pub async fn complete(&self, path: &PathBuf, line: u32, column: u32) -> Result<Vec<super::CompletionItem>> {
        let uri = Url::from_file_path(path).map_err(|_| {
            Error::Validation("Invalid file path".to_string())
        })?;

        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line, character: column },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        };

        let result = self.request("textDocument/completion", Some(serde_json::to_value(params)?)).await?;
        
        if result.is_null() {
            return Ok(vec![]);
        }

        // Handle both CompletionList and Vec<CompletionItem>
        let items = if let Ok(list) = serde_json::from_value::<CompletionList>(result.clone()) {
            list.items
        } else if let Ok(items) = serde_json::from_value::<Vec<CompletionItem>>(result) {
            items
        } else {
            vec![]
        };

        Ok(items.into_iter().map(|item| super::CompletionItem {
            label: item.label,
            kind: item.kind.map(convert_completion_kind).unwrap_or(super::CompletionItemKind::Text),
            detail: item.detail,
            documentation: item.documentation.map(|d| match d {
                Documentation::String(s) => s,
                Documentation::MarkupContent(m) => m.value,
            }),
        }).collect())
    }

    /// Execute code action
    pub async fn execute_code_action(&self, path: &PathBuf, action_title: &str) -> Result<()> {
        let uri = Url::from_file_path(path).map_err(|_| {
            Error::Validation("Invalid file path".to_string())
        })?;

        let params = CodeActionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range: Range::default(),
            context: CodeActionContext {
                diagnostics: vec![],
                only: Some(vec![CodeActionKind::QUICKFIX]),
                trigger_kind: Some(CodeActionTriggerKind::INVOKED),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = self.request("textDocument/codeAction", Some(serde_json::to_value(params)?)).await?;
        
        // Execute the first matching action
        if let Ok(actions) = serde_json::from_value::<Vec<CodeActionOrCommand>>(result) {
            for action_or_cmd in actions {
                if let CodeActionOrCommand::CodeAction(action) = action_or_cmd {
                    if action.title == action_title {
                        if let Some(edit) = action.edit {
                            // Apply workspace edit
                            debug!("Applying workspace edit: {:?}", edit);
                        }
                        if let Some(command) = action.command {
                            // Execute command
                            debug!("Executing command: {:?}", command);
                        }
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Get workspace symbols
    pub async fn workspace_symbol(&self, query: &str) -> Result<Vec<super::Symbol>> {
        let params = WorkspaceSymbolParams {
            query: query.to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let result = self.request("workspace/symbol", Some(serde_json::to_value(params)?)).await?;
        
        if result.is_null() {
            return Ok(vec![]);
        }

        let symbol_informations: Vec<SymbolInformation> = serde_json::from_value(result)?;
        
        Ok(symbol_informations.into_iter().map(|si| super::Symbol {
            name: si.name,
            kind: convert_symbol_kind(si.kind),
            location: convert_location(&si.location),
            documentation: None,
        }).collect())
    }
}

/// Convert LSP SymbolKind to our SymbolKind
fn convert_symbol_kind(kind: lsp_types::SymbolKind) -> super::SymbolKind {
    match kind {
        lsp_types::SymbolKind::FILE => super::SymbolKind::Module,
        lsp_types::SymbolKind::MODULE => super::SymbolKind::Module,
        lsp_types::SymbolKind::NAMESPACE => super::SymbolKind::Namespace,
        lsp_types::SymbolKind::PACKAGE => super::SymbolKind::Package,
        lsp_types::SymbolKind::CLASS => super::SymbolKind::Class,
        lsp_types::SymbolKind::METHOD => super::SymbolKind::Method,
        lsp_types::SymbolKind::PROPERTY => super::SymbolKind::Variable,
        lsp_types::SymbolKind::FIELD => super::SymbolKind::Variable,
        lsp_types::SymbolKind::CONSTRUCTOR => super::SymbolKind::Function,
        lsp_types::SymbolKind::ENUM => super::SymbolKind::Enum,
        lsp_types::SymbolKind::INTERFACE => super::SymbolKind::Interface,
        lsp_types::SymbolKind::FUNCTION => super::SymbolKind::Function,
        lsp_types::SymbolKind::VARIABLE => super::SymbolKind::Variable,
        lsp_types::SymbolKind::CONSTANT => super::SymbolKind::Constant,
        lsp_types::SymbolKind::STRING => super::SymbolKind::Variable,
        lsp_types::SymbolKind::NUMBER => super::SymbolKind::Variable,
        lsp_types::SymbolKind::BOOLEAN => super::SymbolKind::Variable,
        lsp_types::SymbolKind::ARRAY => super::SymbolKind::Variable,
        lsp_types::SymbolKind::OBJECT => super::SymbolKind::Class,
        lsp_types::SymbolKind::KEY => super::SymbolKind::Variable,
        lsp_types::SymbolKind::NULL => super::SymbolKind::Variable,
        lsp_types::SymbolKind::ENUM_MEMBER => super::SymbolKind::Enum,
        lsp_types::SymbolKind::STRUCT => super::SymbolKind::Struct,
        lsp_types::SymbolKind::EVENT => super::SymbolKind::Variable,
        lsp_types::SymbolKind::OPERATOR => super::SymbolKind::Function,
        lsp_types::SymbolKind::TYPE_PARAMETER => super::SymbolKind::TypeParameter,
        _ => super::SymbolKind::Variable,
    }
}

/// Convert LSP CompletionItemKind to our CompletionItemKind
fn convert_completion_kind(kind: lsp_types::CompletionItemKind) -> super::CompletionItemKind {
    match kind {
        lsp_types::CompletionItemKind::TEXT => super::CompletionItemKind::Text,
        lsp_types::CompletionItemKind::METHOD => super::CompletionItemKind::Method,
        lsp_types::CompletionItemKind::FUNCTION => super::CompletionItemKind::Function,
        lsp_types::CompletionItemKind::CONSTRUCTOR => super::CompletionItemKind::Constructor,
        lsp_types::CompletionItemKind::FIELD => super::CompletionItemKind::Field,
        lsp_types::CompletionItemKind::VARIABLE => super::CompletionItemKind::Variable,
        lsp_types::CompletionItemKind::CLASS => super::CompletionItemKind::Class,
        lsp_types::CompletionItemKind::INTERFACE => super::CompletionItemKind::Interface,
        lsp_types::CompletionItemKind::MODULE => super::CompletionItemKind::Module,
        lsp_types::CompletionItemKind::PROPERTY => super::CompletionItemKind::Property,
        lsp_types::CompletionItemKind::UNIT => super::CompletionItemKind::Unit,
        lsp_types::CompletionItemKind::VALUE => super::CompletionItemKind::Value,
        lsp_types::CompletionItemKind::ENUM => super::CompletionItemKind::Enum,
        lsp_types::CompletionItemKind::KEYWORD => super::CompletionItemKind::Keyword,
        lsp_types::CompletionItemKind::SNIPPET => super::CompletionItemKind::Snippet,
        lsp_types::CompletionItemKind::COLOR => super::CompletionItemKind::Color,
        lsp_types::CompletionItemKind::FILE => super::CompletionItemKind::File,
        lsp_types::CompletionItemKind::REFERENCE => super::CompletionItemKind::Reference,
        _ => super::CompletionItemKind::Text,
    }
}

/// Convert LSP Location to our Location
fn convert_lsp_location(location: &Location) -> super::Location {
    super::Location {
        path: location.uri.to_file_path().unwrap_or_default(),
        line_start: location.range.start.line,
        line_end: location.range.end.line,
        column_start: location.range.start.character,
        column_end: location.range.end.character,
    }
}

/// Convert LSP Location to our Location
fn convert_location(location: &Location) -> super::Location {
    convert_lsp_location(location)
}

/// Convert LSP Range to our Location
fn convert_range(range: &Range, _selection_range: &Range) -> super::Location {
    super::Location {
        path: PathBuf::new(), // Would need to extract from context
        line_start: range.start.line,
        line_end: range.end.line,
        column_start: range.start.character,
        column_end: range.end.character,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_registry() {
        let registry = ServerRegistry::new();
        registry.register_mapping("rs".to_string(), "rust-analyzer".to_string());
        assert_eq!(registry.get_server_for_extension("rs"), Some("rust-analyzer".to_string()));
    }

    #[test]
    fn test_document_cache() {
        let cache = DocumentCache::new();
        let path = PathBuf::from("/test/file.rs");
        cache.insert(path.clone(), "fn main() {}".to_string(), 1);
        
        let doc = cache.get(&path).unwrap();
        assert_eq!(doc.version, 1);
        assert_eq!(doc.language_id, "rust");
    }

    #[test]
    fn test_detect_language_id() {
        assert_eq!(DocumentCache::detect_language_id(Path::new("test.rs")), "rust");
        assert_eq!(DocumentCache::detect_language_id(Path::new("test.ts")), "typescript");
        assert_eq!(DocumentCache::detect_language_id(Path::new("test.py")), "python");
        assert_eq!(DocumentCache::detect_language_id(Path::new("test.unknown")), "plaintext");
    }
}