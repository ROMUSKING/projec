//! Analysis layer for the coding agent.
//!
//! This crate provides LSP client functionality, AST analysis,
//! and semantic code analysis capabilities.

use common::{async_trait, Error, Module, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub mod lsp;
pub mod semantic;

/// Main analysis engine
pub struct AnalysisEngine {
    lsp_manager: Arc<lsp::LspManager>,
    semantic_analyzer: Arc<semantic::SemanticAnalyzer>,
    cache: Arc<AnalysisCache>,
    config: AnalysisConfig,
}

impl AnalysisEngine {
    pub fn new() -> Self {
        Self::with_config(AnalysisConfig::default())
    }

    pub fn with_config(config: AnalysisConfig) -> Self {
        Self {
            lsp_manager: Arc::new(lsp::LspManager::new()),
            semantic_analyzer: Arc::new(semantic::SemanticAnalyzer::new()),
            cache: Arc::new(AnalysisCache::new()),
            config,
        }
    }

    /// Initialize the analysis engine
    pub async fn initialize(&self) -> Result<() > {
        info!("Initializing analysis engine");
        
        // Initialize LSP manager
        self.lsp_manager.initialize().await?;
        
        info!("Analysis engine initialized");
        Ok(())
    }

    /// Shutdown the analysis engine
    pub async fn shutdown(&self) -> Result<() > {
        info!("Shutting down analysis engine");
        
        // Shutdown LSP manager
        self.lsp_manager.shutdown().await?;
        
        // Clear caches
        self.cache.clear();
        
        info!("Analysis engine shutdown complete");
        Ok(())
    }

    /// Analyze a file using available language servers and semantic analysis
    pub async fn analyze_file(&self, path: &PathBuf) -> Result<FileAnalysis> {
        debug!("Analyzing file: {:?}", path);
        
        // Check cache first
        if let Some(cached) = self.cache.get_file_analysis(path).await {
            if !self.config.disable_cache {
                return Ok(cached);
            }
        }

        // Perform LSP analysis
        let lsp_analysis = if self.config.enable_lsp {
            match self.lsp_manager.analyze_file(path).await {
                Ok(analysis) => analysis,
                Err(e) => {
                    warn!("LSP analysis failed for {:?}: {}", path, e);
                    LspAnalysis::default()
                }
            }
        } else {
            LspAnalysis::default()
        };

        // Perform semantic analysis
        let semantic_analysis = if self.config.enable_semantic {
            match self.semantic_analyzer.analyze(path).await {
                Ok(analysis) => analysis,
                Err(e) => {
                    warn!("Semantic analysis failed for {:?}: {}", path, e);
                    SemanticAnalysis::default()
                }
            }
        } else {
            SemanticAnalysis::default()
        };

        let analysis = FileAnalysis {
            path: path.clone(),
            lsp: lsp_analysis,
            semantic: semantic_analysis,
            timestamp: common::now(),
        };

        // Cache the result
        if self.config.enable_cache {
            self.cache.insert_file_analysis(path.clone(), analysis.clone()).await;
        }

        Ok(analysis)
    }

    /// Analyze multiple files
    pub async fn analyze_files(&self, paths: &[PathBuf]) -> Result<Vec<FileAnalysis>> {
        let mut results = Vec::with_capacity(paths.len());
        
        for path in paths {
            match self.analyze_file(path).await {
                Ok(analysis) => results.push(analysis),
                Err(e) => warn!("Failed to analyze {:?}: {}", path, e),
            }
        }

        Ok(results)
    }

    /// Analyze a directory recursively
    pub async fn analyze_directory(&self, dir: &PathBuf, file_patterns: &[String]) -> Result<Vec<FileAnalysis>> {
        let mut files = Vec::new();
        
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path().to_path_buf();
            
            // Check if file matches any pattern
            if file_patterns.is_empty() || 
               file_patterns.iter().any(|p| {
                   if let Some(ext) = path.extension() {
                       ext.to_string_lossy() == p.trim_start_matches("*.")
                   } else {
                       false
                   }
               }) {
                files.push(path);
            }
        }

        self.analyze_files(&files).await
    }

    /// Find symbol references across the codebase
    pub async fn find_references(&self, path: &PathBuf, line: u32, column: u32) -> Result<Vec<Reference>> {
        self.lsp_manager.find_references(path, line, column).await
    }

    /// Find all references to a symbol by name across the workspace
    pub async fn find_symbol_references(&self, symbol_name: &str) -> Result<Vec<Reference>> {
        let mut all_refs = Vec::new();
        
        // Search in cached analyses
        for entry in self.cache.file_analyses.iter() {
            let analysis = entry.value();
            
            // Check LSP symbols
            for symbol in &analysis.lsp.symbols {
                if symbol.name == symbol_name {
                    all_refs.push(Reference {
                        symbol: symbol_name.to_string(),
                        location: symbol.location.clone(),
                        is_definition: true,
                    });
                }
            }
        }

        Ok(all_refs)
    }

    /// Get code diagnostics for a file
    pub async fn get_diagnostics(&self, path: &PathBuf) -> Result<Vec<Diagnostic>> {
        self.lsp_manager.get_diagnostics(path).await
    }

    /// Go to definition
    pub async fn goto_definition(&self, path: &PathBuf, line: u32, column: u32) -> Result<Option<Location>> {
        self.lsp_manager.goto_definition(path, line, column).await
    }

    /// Get hover information at a position
    pub async fn hover(&self, path: &PathBuf, line: u32, column: u32) -> Result<Option<String>> {
        self.lsp_manager.hover(path, line, column).await
    }

    /// Get completions at a position
    pub async fn complete(&self, path: &PathBuf, line: u32, column: u32) -> Result<Vec<CompletionItem>> {
        self.lsp_manager.complete(path, line, column).await
    }

    /// Execute a code action
    pub async fn execute_code_action(&self, path: &PathBuf, action: &str) -> Result<()> {
        self.lsp_manager.execute_code_action(path, action).await
    }

    /// Search for symbols across the workspace
    pub async fn workspace_symbol(&self, query: &str) -> Result<Vec<Symbol>> {
        self.lsp_manager.workspace_symbol(query).await
    }

    /// Get symbols in a file
    pub async fn document_symbols(&self, path: &PathBuf) -> Result<Vec<Symbol>> {
        let analysis = self.analyze_file(path).await?;
        Ok(analysis.lsp.symbols)
    }

    /// Open a document for tracking
    pub async fn open_document(&self, path: &PathBuf, content: String) -> Result<()> {
        let version = 1;
        self.lsp_manager.open_document(path, content, version).await
    }

    /// Update a tracked document
    pub async fn update_document(&self, path: &PathBuf, content: String, version: i32) -> Result<()> {
        // Invalidate cache
        self.cache.invalidate(path).await;
        self.semantic_analyzer.invalidate(path);
        
        self.lsp_manager.update_document(path, content, version).await
    }

    /// Close a tracked document
    pub async fn close_document(&self, path: &PathBuf) -> Result<()> {
        self.lsp_manager.close_document(path).await
    }

    /// Get code complexity metrics
    pub async fn get_complexity(&self, path: &PathBuf) -> Result<CodeComplexity> {
        let analysis = self.analyze_file(path).await?;
        Ok(analysis.semantic.complexity)
    }

    /// Get dependencies for a file
    pub async fn get_dependencies(&self, path: &PathBuf) -> Result<Vec<Dependency>> {
        let analysis = self.analyze_file(path).await?;
        Ok(analysis.semantic.dependencies)
    }

    /// Get imports for a file
    pub async fn get_imports(&self, path: &PathBuf) -> Result<Vec<Import>> {
        let analysis = self.analyze_file(path).await?;
        Ok(analysis.semantic.imports)
    }

    /// Get exports for a file
    pub async fn get_exports(&self, path: &PathBuf) -> Result<Vec<Export>> {
        let analysis = self.analyze_file(path).await?;
        Ok(analysis.semantic.exports)
    }

    /// Build dependency graph for a set of files
    pub async fn build_dependency_graph(&self, files: &[PathBuf]) -> Result<semantic::DependencyGraph> {
        self.semantic_analyzer.build_dependency_graph(files).await
    }

    /// Find dead code in a file
    pub async fn find_dead_code(&self, path: &PathBuf) -> Result<Vec<semantic::DeadCodeResult>> {
        let content = tokio::fs::read_to_string(path).await?;
        Ok(self.semantic_analyzer.find_dead_code(path, &content))
    }

    /// Calculate code metrics
    pub async fn calculate_metrics(&self, path: &PathBuf) -> Result<semantic::CodeMetrics> {
        let content = tokio::fs::read_to_string(path).await?;
        let collector = semantic::MetricsCollector;
        collector.collect_metrics(path, &content)
    }

    /// Get analysis summary for a file
    pub async fn get_summary(&self, path: &PathBuf) -> Result<AnalysisSummary> {
        let analysis = self.analyze_file(path).await?;
        
        let symbol_count = analysis.lsp.symbols.len();
        let diagnostic_count = analysis.lsp.diagnostics.len();
        let error_count = analysis.lsp.diagnostics.iter()
            .filter(|d| matches!(d.severity, DiagnosticSeverity::Error))
            .count();
        let warning_count = analysis.lsp.diagnostics.iter()
            .filter(|d| matches!(d.severity, DiagnosticSeverity::Warning))
            .count();

        Ok(AnalysisSummary {
            path: path.clone(),
            symbol_count,
            diagnostic_count,
            error_count,
            warning_count,
            complexity: analysis.semantic.complexity.cyclomatic_complexity,
            lines_of_code: analysis.semantic.complexity.lines_of_code,
            dependency_count: analysis.semantic.dependencies.len(),
        })
    }

    /// Clear all caches
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
        self.semantic_analyzer.clear_cache();
    }

    /// Add a workspace root
    pub fn add_workspace_root(&self, root: PathBuf) {
        self.lsp_manager.add_workspace_root(root);
    }

    /// Get the LSP manager
    pub fn lsp_manager(&self) -> &Arc<lsp::LspManager> {
        &self.lsp_manager
    }

    /// Get the semantic analyzer
    pub fn semantic_analyzer(&self) -> &Arc<semantic::SemanticAnalyzer> {
        &self.semantic_analyzer
    }
}

impl Default for AnalysisEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Module for AnalysisEngine {
    fn name(&self) -> &str {
        "analysis"
    }

    async fn initialize(&mut self) -> Result<()> {
        self.initialize().await
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.shutdown().await
    }
}

/// Analysis configuration
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub enable_lsp: bool,
    pub enable_semantic: bool,
    pub enable_cache: bool,
    pub disable_cache: bool,
    pub cache_ttl_seconds: u64,
    pub max_cache_size: usize,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            enable_lsp: true,
            enable_semantic: true,
            enable_cache: true,
            disable_cache: false,
            cache_ttl_seconds: 3600,
            max_cache_size: 1000,
        }
    }
}

/// Analysis cache for storing results
pub struct AnalysisCache {
    file_analyses: DashMap<PathBuf, FileAnalysis>,
    symbol_index: DashMap<String, Vec<PathBuf>>,
}

impl AnalysisCache {
    pub fn new() -> Self {
        Self {
            file_analyses: DashMap::new(),
            symbol_index: DashMap::new(),
        }
    }

    pub async fn get_file_analysis(&self, path: &PathBuf) -> Option<FileAnalysis> {
        self.file_analyses.get(path).map(|a| a.clone())
    }

    pub async fn insert_file_analysis(&self, path: PathBuf, analysis: FileAnalysis) {
        // Index symbols
        for symbol in &analysis.lsp.symbols {
            self.symbol_index
                .entry(symbol.name.clone())
                .or_insert_with(Vec::new)
                .push(path.clone());
        }

        self.file_analyses.insert(path, analysis);
    }

    pub async fn invalidate(&self, path: &PathBuf) {
        if let Some((_, analysis)) = self.file_analyses.remove(path) {
            // Remove from symbol index
            for symbol in &analysis.lsp.symbols {
                if let Some(mut files) = self.symbol_index.get_mut(&symbol.name) {
                    files.retain(|p| p != path);
                }
            }
        }
    }

    pub async fn clear(&self) {
        self.file_analyses.clear();
        self.symbol_index.clear();
    }

    pub fn find_files_with_symbol(&self, symbol_name: &str) -> Vec<PathBuf> {
        self.symbol_index
            .get(symbol_name)
            .map(|v| v.clone())
            .unwrap_or_default()
    }
}

impl Default for AnalysisCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete file analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysis {
    pub path: PathBuf,
    pub lsp: LspAnalysis,
    pub semantic: SemanticAnalysis,
    pub timestamp: common::Timestamp,
}

/// Analysis summary for quick overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub path: PathBuf,
    pub symbol_count: usize,
    pub diagnostic_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub complexity: u32,
    pub lines_of_code: u32,
    pub dependency_count: usize,
}

/// LSP-based analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LspAnalysis {
    pub symbols: Vec<Symbol>,
    pub diagnostics: Vec<Diagnostic>,
    pub document_links: Vec<DocumentLink>,
}

/// Semantic analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SemanticAnalysis {
    pub complexity: CodeComplexity,
    pub dependencies: Vec<Dependency>,
    pub exports: Vec<Export>,
    pub imports: Vec<Import>,
}

/// Code symbol (function, struct, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
    pub documentation: Option<String>,
}

/// Types of symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Interface,
    Struct,
    Enum,
    Variable,
    Constant,
    Module,
    Namespace,
    Package,
    TypeParameter,
}

/// Source code location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub path: PathBuf,
    pub line_start: u32,
    pub line_end: u32,
    pub column_start: u32,
    pub column_end: u32,
}

/// Code diagnostic (error, warning, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub location: Location,
    pub code: Option<String>,
    pub source: String,
}

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// Document link (import, reference, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLink {
    pub target: PathBuf,
    pub location: Location,
}

/// Symbol reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub symbol: String,
    pub location: Location,
    pub is_definition: bool,
}

/// Code complexity metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeComplexity {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub lines_of_code: u32,
    pub lines_of_comments: u32,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub kind: DependencyKind,
}

/// Dependency kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
    Direct,
    Dev,
    Peer,
    Optional,
}

/// Export information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Export {
    pub name: String,
    pub kind: SymbolKind,
    pub is_public: bool,
}

/// Import information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub source: String,
    pub items: Vec<String>,
    pub is_default: bool,
}

/// Completion item for IDE-like features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
}

/// Completion item kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionItemKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_kind_serialization() {
        let kind = SymbolKind::Function;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"function\"");
    }

    #[test]
    fn test_analysis_config_default() {
        let config = AnalysisConfig::default();
        assert!(config.enable_lsp);
        assert!(config.enable_semantic);
        assert!(config.enable_cache);
    }

    #[test]
    fn test_analysis_cache() {
        let cache = AnalysisCache::new();
        let path = PathBuf::from("/test/file.rs");
        let analysis = FileAnalysis {
            path: path.clone(),
            lsp: LspAnalysis::default(),
            semantic: SemanticAnalysis::default(),
            timestamp: common::chrono::Utc::now(),
        };
        
        // Basic check to ensure struct can be created
        assert_eq!(analysis.path, path);
    }
}
