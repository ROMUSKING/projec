//! Semantic analysis module.
//!
//! This module provides code analysis beyond what LSP offers,
//! including dependency analysis, complexity metrics, etc.

use common::{Error, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use tokio::fs;
use tracing::{debug, warn};

/// Semantic analyzer for code analysis
pub struct SemanticAnalyzer {
    cache: dashmap::DashMap<PathBuf, super::SemanticAnalysis>,
    parsers: HashMap<String, Arc<dyn LanguageParser>>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut parsers: HashMap<String, Arc<dyn LanguageParser>> = HashMap::new();
        
        // Register built-in parsers
        parsers.insert("rs".to_string(), Arc::new(RustParser));
        parsers.insert("ts".to_string(), Arc::new(TypeScriptParser));
        parsers.insert("tsx".to_string(), Arc::new(TypeScriptParser));
        parsers.insert("js".to_string(), Arc::new(TypeScriptParser));
        parsers.insert("jsx".to_string(), Arc::new(TypeScriptParser));
        parsers.insert("py".to_string(), Arc::new(PythonParser));
        parsers.insert("go".to_string(), Arc::new(GoParser));
        
        Self {
            cache: dashmap::DashMap::new(),
            parsers,
        }
    }

    /// Analyze a file semantically
    pub async fn analyze(&self, path: &PathBuf) -> Result<super::SemanticAnalysis> {
        // Check cache first
        if let Some(cached) = self.cache.get(path) {
            return Ok(cached.clone());
        }

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| Error::Validation("Could not determine file extension".to_string()))?;

        let content = fs::read_to_string(path).await?;
        
        let analysis = if let Some(parser) = self.parsers.get(extension) {
            parser.analyze(path, &content).await?
        } else {
            // Fallback to generic analysis
            self.generic_analyze(path, &content).await?
        };

        // Cache the result
        self.cache.insert(path.clone(), analysis.clone());
        
        Ok(analysis)
    }

    /// Generic analysis for unsupported languages
    async fn generic_analyze(&self, path: &PathBuf, content: &str) -> Result<super::SemanticAnalysis> {
        let mut complexity = CodeComplexity::default();
        let mut dependencies = Vec::new();
        let mut exports = Vec::new();
        let mut imports = Vec::new();

        // Basic line counting
        let lines: Vec<&str> = content.lines().collect();
        complexity.lines_of_code = lines.len() as u32;
        complexity.lines_of_comments = lines
            .iter()
            .filter(|l| l.trim().starts_with("//") || l.trim().starts_with("#") || l.trim().starts_with("/*"))
            .count() as u32;

        Ok(super::SemanticAnalysis {
            complexity: complexity.into(),
            dependencies,
            exports,
            imports,
        })
    }

    /// Calculate cyclomatic complexity
    pub fn calculate_complexity(&self, code: &str, language: &str) -> CodeComplexity {
        let mut complexity = CodeComplexity::default();
        let lines: Vec<&str> = code.lines().collect();
        
        complexity.lines_of_code = lines.len() as u32;
        complexity.lines_of_comments = self.count_comment_lines(&lines, language);
        complexity.blank_lines = lines.iter().filter(|l| l.trim().is_empty()).count() as u32;
        
        // Calculate cyclomatic complexity based on language
        complexity.cyclomatic_complexity = match language {
            "rs" => self.calculate_rust_complexity(code),
            "ts" | "js" => self.calculate_js_complexity(code),
            "py" => self.calculate_python_complexity(code),
            "go" => self.calculate_go_complexity(code),
            _ => 1,
        };

        complexity.cognitive_complexity = complexity.cyclomatic_complexity;
        
        complexity
    }

    fn count_comment_lines(&self, lines: &[&str], language: &str) -> u32 {
        let mut count = 0;
        let mut in_block_comment = false;

        for line in lines {
            let trimmed = line.trim();
            
            match language {
                "rs" | "ts" | "js" | "go" | "java" | "cpp" | "c" => {
                    if in_block_comment {
                        count += 1;
                        if trimmed.contains("*/") {
                            in_block_comment = false;
                        }
                    } else if trimmed.starts_with("//") {
                        count += 1;
                    } else if trimmed.starts_with("/*") {
                        count += 1;
                        if !trimmed.contains("*/") {
                            in_block_comment = true;
                        }
                    }
                }
                "py" => {
                    if trimmed.starts_with("#") || trimmed.starts_with("\"\"\"") || trimmed.starts_with("'") {
                        count += 1;
                    }
                }
                _ => {}
            }
        }

        count
    }

    fn calculate_rust_complexity(&self, code: &str) -> u32 {
        let mut complexity = 1u32;
        
        // Count decision points
        let patterns = [
            "if ", "match ", "for ", "while ", "loop {",
            "&&", "||", "?",
        ];
        
        for pattern in &patterns {
            complexity += code.matches(pattern).count() as u32;
        }

        complexity
    }

    fn calculate_js_complexity(&self, code: &str) -> u32 {
        let mut complexity = 1u32;
        
        let patterns = [
            "if ", "else if ", "switch ", "for ", "while ", "do ",
            "&&", "||", "?",
        ];
        
        for pattern in &patterns {
            complexity += code.matches(pattern).count() as u32;
        }

        complexity
    }

    fn calculate_python_complexity(&self, code: &str) -> u32 {
        let mut complexity = 1u32;
        
        let patterns = [
            "if ", "elif ", "for ", "while ", "except", "with ",
            "and", "or",
        ];
        
        for pattern in &patterns {
            complexity += code.matches(pattern).count() as u32;
        }

        complexity
    }

    fn calculate_go_complexity(&self, code: &str) -> u32 {
        let mut complexity = 1u32;
        
        let patterns = [
            "if ", "for ", "switch ", "select ",
            "&&", "||",
        ];
        
        for pattern in &patterns {
            complexity += code.matches(pattern).count() as u32;
        }

        complexity
    }

    /// Extract dependencies from code
    pub fn extract_dependencies(&self, path: &PathBuf, content: &str) -> Vec<super::Dependency> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        
        match extension {
            "rs" => self.extract_rust_dependencies(content),
            "ts" | "tsx" | "js" | "jsx" => self.extract_js_dependencies(content),
            "py" => self.extract_python_dependencies(content),
            "go" => self.extract_go_dependencies(content),
            _ => vec![],
        }
    }

    fn extract_rust_dependencies(&self, content: &str) -> Vec<super::Dependency> {
        let mut deps = Vec::new();
        
        // Extract use statements
        static USE_REGEX: OnceLock<Regex> = OnceLock::new();
        let use_regex = USE_REGEX.get_or_init(|| Regex::new(r"use\s+([\w:]+)").unwrap());
        for cap in use_regex.captures_iter(content) {
            let name = cap[1].to_string();
            deps.push(super::Dependency {
                name,
                version: None,
                path: None,
                kind: super::DependencyKind::Direct,
            });
        }

        // Extract extern crate
        static EXTERN_REGEX: OnceLock<Regex> = OnceLock::new();
        let extern_regex = EXTERN_REGEX.get_or_init(|| Regex::new(r"extern\s+crate\s+(\w+)").unwrap());
        for cap in extern_regex.captures_iter(content) {
            deps.push(super::Dependency {
                name: cap[1].to_string(),
                version: None,
                path: None,
                kind: super::DependencyKind::Direct,
            });
        }

        deps
    }

    fn extract_js_dependencies(&self, content: &str) -> Vec<super::Dependency> {
        let mut deps = Vec::new();
        
        // ES6 imports
        static IMPORT_REGEX: OnceLock<Regex> = OnceLock::new();
        let import_regex = IMPORT_REGEX.get_or_init(|| Regex::new(r#"import\s+.*?\s+from\s+['"]([^'"]+)['"]|import\s+['"]([^'"]+)['"]"#).unwrap());
        for cap in import_regex.captures_iter(content) {
            let name = cap.get(1).or(cap.get(2)).map(|m| m.as_str().to_string()).unwrap_or_default();
            if !name.is_empty() {
                deps.push(super::Dependency {
                    name,
                    version: None,
                    path: None,
                    kind: super::DependencyKind::Direct,
                });
            }
        }

        // CommonJS requires
        static REQUIRE_REGEX: OnceLock<Regex> = OnceLock::new();
        let require_regex = REQUIRE_REGEX.get_or_init(|| Regex::new(r#"require\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap());
        for cap in require_regex.captures_iter(content) {
            deps.push(super::Dependency {
                name: cap[1].to_string(),
                version: None,
                path: None,
                kind: super::DependencyKind::Direct,
            });
        }

        deps
    }

    fn extract_python_dependencies(&self, content: &str) -> Vec<super::Dependency> {
        let mut deps = Vec::new();
        
        // Import statements
        static IMPORT_REGEX: OnceLock<Regex> = OnceLock::new();
        let import_regex = IMPORT_REGEX.get_or_init(|| Regex::new(r"(?:from\s+(\S+)\s+)?import\s+(.+)").unwrap());
        for cap in import_regex.captures_iter(content) {
            let module = cap.get(1).map(|m| m.as_str().to_string())
                .or_else(|| cap.get(2).map(|m| m.as_str().split(',').next().unwrap().trim().to_string()))
                .unwrap_or_default();
            
            if !module.is_empty() && !module.starts_with('.') {
                deps.push(super::Dependency {
                    name: module,
                    version: None,
                    path: None,
                    kind: super::DependencyKind::Direct,
                });
            }
        }

        deps
    }

    fn extract_go_dependencies(&self, content: &str) -> Vec<super::Dependency> {
        let mut deps = Vec::new();
        
        // Import statements
        static IMPORT_REGEX: OnceLock<Regex> = OnceLock::new();
        let import_regex = IMPORT_REGEX.get_or_init(|| Regex::new(r#"import\s+(?:\(\s*)?["`]([^"`]+)["`]"#).unwrap());
        for cap in import_regex.captures_iter(content) {
            let import_path = cap[1].to_string();
            deps.push(super::Dependency {
                name: import_path.clone(),
                version: None,
                path: Some(PathBuf::from(&import_path)),
                kind: super::DependencyKind::Direct,
            });
        }

        deps
    }

    /// Find dead code
    pub fn find_dead_code(&self, path: &PathBuf, content: &str) -> Vec<DeadCodeResult> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        
        match extension {
            "rs" => self.find_rust_dead_code(content),
            "ts" | "tsx" | "js" | "jsx" => self.find_js_dead_code(content),
            "py" => self.find_python_dead_code(content),
            _ => vec![],
        }
    }

    fn find_rust_dead_code(&self, content: &str) -> Vec<DeadCodeResult> {
        let mut results = Vec::new();
        
        // Look for functions/structs that are private and not used
        static FN_REGEX: OnceLock<Regex> = OnceLock::new();
        let fn_regex = FN_REGEX.get_or_init(|| Regex::new(r"fn\s+(\w+)").unwrap());
        static STRUCT_REGEX: OnceLock<Regex> = OnceLock::new();
        let _struct_regex = STRUCT_REGEX.get_or_init(|| Regex::new(r"struct\s+(\w+)").unwrap());
        
        // This is a simplified check - real dead code detection would need full project analysis
        for cap in fn_regex.captures_iter(content) {
            let name = cap[1].to_string();
            // Check if function is unused (simplified)
            if !content.contains(&format!("{}(", name)) || content.contains(&format!("pub fn {}", name)) {
                continue;
            }
        }

        results
    }

    fn find_js_dead_code(&self, _content: &str) -> Vec<DeadCodeResult> {
        // Simplified - would need full AST analysis
        vec![]
    }

    fn find_python_dead_code(&self, _content: &str) -> Vec<DeadCodeResult> {
        // Simplified - would need full AST analysis
        vec![]
    }

    /// Build dependency graph for a set of files
    pub async fn build_dependency_graph(&self, files: &[PathBuf]) -> Result<DependencyGraph> {
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();
        
        for file in files {
            let node_id = file.to_string_lossy().to_string();
            let kind = if file.extension().map(|e| e == "rs").unwrap_or(false) {
                NodeKind::File
            } else {
                NodeKind::File
            };
            
            nodes.insert(node_id.clone(), GraphNode {
                id: node_id.clone(),
                path: file.clone(),
                kind,
            });

            // Extract imports/dependencies
            if let Ok(content) = fs::read_to_string(file).await {
                let deps = self.extract_dependencies(file, &content);
                for dep in deps {
                    edges.push(GraphEdge {
                        from: node_id.clone(),
                        to: dep.name,
                        kind: EdgeKind::Imports,
                    });
                }
            }
        }

        Ok(DependencyGraph {
            nodes: nodes.into_values().collect(),
            edges,
        })
    }

    /// Clear the analysis cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Invalidate cache entry for a specific file
    pub fn invalidate(&self, path: &PathBuf) {
        self.cache.remove(path);
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Language parser trait
#[async_trait::async_trait]
pub trait LanguageParser: Send + Sync {
    async fn analyze(&self, path: &PathBuf, content: &str) -> Result<super::SemanticAnalysis>;
    fn parse_symbols(&self, content: &str) -> Vec<ParsedSymbol>;
    fn parse_imports(&self, content: &str) -> Vec<super::Import>;
    fn parse_exports(&self, content: &str) -> Vec<super::Export>;
}

/// Parsed symbol from source code
#[derive(Debug, Clone)]
pub struct ParsedSymbol {
    pub name: String,
    pub kind: super::SymbolKind,
    pub line: u32,
    pub is_public: bool,
    pub documentation: Option<String>,
}

/// Rust language parser
pub struct RustParser;

#[async_trait::async_trait]
impl LanguageParser for RustParser {
    async fn analyze(&self, path: &PathBuf, content: &str) -> Result<super::SemanticAnalysis> {
        let analyzer = SemanticAnalyzer::new();
        
        let complexity = analyzer.calculate_complexity(content, "rs");
        let dependencies = analyzer.extract_dependencies(path, content);
        let imports = self.parse_imports(content);
        let exports = self.parse_exports(content);

        Ok(super::SemanticAnalysis {
            complexity: complexity.into(),
            dependencies,
            exports,
            imports,
        })
    }

    fn parse_symbols(&self, content: &str) -> Vec<ParsedSymbol> {
        let mut symbols = Vec::new();
        
        // Parse functions
        let fn_regex = Regex::new(r"(?:(pub)\s+)?fn\s+(\w+)").unwrap();
        for (i, line) in content.lines().enumerate() {
            if let Some(cap) = fn_regex.captures(line) {
                symbols.push(ParsedSymbol {
                    name: cap[2].to_string(),
                    kind: super::SymbolKind::Function,
                    line: i as u32,
                    is_public: cap.get(1).is_some(),
                    documentation: None,
                });
            }
        }

        // Parse structs
        let struct_regex = Regex::new(r"(?:(pub)\s+)?struct\s+(\w+)").unwrap();
        for (i, line) in content.lines().enumerate() {
            if let Some(cap) = struct_regex.captures(line) {
                symbols.push(ParsedSymbol {
                    name: cap[2].to_string(),
                    kind: super::SymbolKind::Struct,
                    line: i as u32,
                    is_public: cap.get(1).is_some(),
                    documentation: None,
                });
            }
        }

        // Parse enums
        let enum_regex = Regex::new(r"(?:(pub)\s+)?enum\s+(\w+)").unwrap();
        for (i, line) in content.lines().enumerate() {
            if let Some(cap) = enum_regex.captures(line) {
                symbols.push(ParsedSymbol {
                    name: cap[2].to_string(),
                    kind: super::SymbolKind::Enum,
                    line: i as u32,
                    is_public: cap.get(1).is_some(),
                    documentation: None,
                });
            }
        }

        // Parse traits
        let trait_regex = Regex::new(r"(?:(pub)\s+)?trait\s+(\w+)").unwrap();
        for (i, line) in content.lines().enumerate() {
            if let Some(cap) = trait_regex.captures(line) {
                symbols.push(ParsedSymbol {
                    name: cap[2].to_string(),
                    kind: super::SymbolKind::Interface,
                    line: i as u32,
                    is_public: cap.get(1).is_some(),
                    documentation: None,
                });
            }
        }

        symbols
    }

    fn parse_imports(&self, content: &str) -> Vec<super::Import> {
        let mut imports = Vec::new();
        
        let use_regex = Regex::new(r"use\s+([\w:]+)(?:::\{([^}]+)\})?;").unwrap();
        for cap in use_regex.captures_iter(content) {
            let source = cap[1].to_string();
            let items = cap.get(2)
                .map(|m| m.as_str().split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();
            
            imports.push(super::Import {
                source,
                items,
                is_default: false,
            });
        }

        imports
    }

    fn parse_exports(&self, content: &str) -> Vec<super::Export> {
        let mut exports = Vec::new();
        
        // Find all pub items
        let pub_regex = Regex::new(r"pub\s+(?:\([^)]+\)\s+)?(\w+)\s+(\w+)").unwrap();
        for cap in pub_regex.captures_iter(content) {
            let kind = match &cap[1] {
                "fn" => super::SymbolKind::Function,
                "struct" => super::SymbolKind::Struct,
                "enum" => super::SymbolKind::Enum,
                "trait" => super::SymbolKind::Interface,
                "mod" => super::SymbolKind::Module,
                "const" => super::SymbolKind::Constant,
                "static" => super::SymbolKind::Constant,
                "type" => super::SymbolKind::TypeParameter,
                _ => super::SymbolKind::Variable,
            };
            
            exports.push(super::Export {
                name: cap[2].to_string(),
                kind,
                is_public: true,
            });
        }

        exports
    }
}

/// TypeScript/JavaScript parser
pub struct TypeScriptParser;

#[async_trait::async_trait]
impl LanguageParser for TypeScriptParser {
    async fn analyze(&self, path: &PathBuf, content: &str) -> Result<super::SemanticAnalysis> {
        let analyzer = SemanticAnalyzer::new();
        
        let complexity = analyzer.calculate_complexity(content, "ts");
        let dependencies = analyzer.extract_dependencies(path, content);
        let imports = self.parse_imports(content);
        let exports = self.parse_exports(content);

        Ok(super::SemanticAnalysis {
            complexity: complexity.into(),
            dependencies,
            exports,
            imports,
        })
    }

    fn parse_symbols(&self, content: &str) -> Vec<ParsedSymbol> {
        let mut symbols = Vec::new();
        
        // Parse functions
        let fn_regex = Regex::new(r"(?:export\s+)?(?:async\s+)?function\s+(\w+)").unwrap();
        for (i, line) in content.lines().enumerate() {
            if let Some(cap) = fn_regex.captures(line) {
                symbols.push(ParsedSymbol {
                    name: cap[1].to_string(),
                    kind: super::SymbolKind::Function,
                    line: i as u32,
                    is_public: line.contains("export"),
                    documentation: None,
                });
            }
        }

        // Parse classes
        let class_regex = Regex::new(r"(?:export\s+)?class\s+(\w+)").unwrap();
        for (i, line) in content.lines().enumerate() {
            if let Some(cap) = class_regex.captures(line) {
                symbols.push(ParsedSymbol {
                    name: cap[1].to_string(),
                    kind: super::SymbolKind::Class,
                    line: i as u32,
                    is_public: line.contains("export"),
                    documentation: None,
                });
            }
        }

        // Parse interfaces
        let interface_regex = Regex::new(r"(?:export\s+)?interface\s+(\w+)").unwrap();
        for (i, line) in content.lines().enumerate() {
            if let Some(cap) = interface_regex.captures(line) {
                symbols.push(ParsedSymbol {
                    name: cap[1].to_string(),
                    kind: super::SymbolKind::Interface,
                    line: i as u32,
                    is_public: line.contains("export"),
                    documentation: None,
                });
            }
        }

        symbols
    }

    fn parse_imports(&self, content: &str) -> Vec<super::Import> {
        let mut imports = Vec::new();
        
        // ES6 imports
        let import_regex = Regex::new(r#"import\s+(?:(\w+)\s*,?\s*)?(?:\{\s*([^}]*)\s*\})?\s*from\s+['"]([^'"]+)['"];?"#).unwrap();
        for cap in import_regex.captures_iter(content) {
            let source = cap[3].to_string();
            let default = cap.get(1).map(|m| m.as_str().to_string());
            let named: Vec<String> = cap.get(2)
                .map(|m| m.as_str().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
                .unwrap_or_default();
            
            let mut items = named;
            if let Some(def) = default {
                items.insert(0, def);
            }
            
            imports.push(super::Import {
                source,
                items,
                is_default: cap.get(1).is_some() && cap.get(2).is_none(),
            });
        }

        // Namespace imports
        let namespace_import_regex = Regex::new(r#"import\s+\*\s+as\s+(\w+)\s+from\s+['"]([^'"]+)['"];?"#).unwrap();
        for cap in namespace_import_regex.captures_iter(content) {
            imports.push(super::Import {
                source: cap[2].to_string(),
                items: vec![cap[1].to_string()],
                is_default: false,
            });
        }

        imports
    }

    fn parse_exports(&self, content: &str) -> Vec<super::Export> {
        let mut exports = Vec::new();
        
        // Named exports
        let export_regex = Regex::new(r"export\s+(?:const|let|var|function|class|interface|type|enum)\s+(\w+)").unwrap();
        for cap in export_regex.captures_iter(content) {
            exports.push(super::Export {
                name: cap[1].to_string(),
                kind: super::SymbolKind::Variable,
                is_public: true,
            });
        }

        // Export { ... } from '...'
        let reexport_regex = Regex::new(r#"export\s+\{([^}]+)\}\s+from\s+['"]([^'"]+)['"];?"#).unwrap();
        for cap in reexport_regex.captures_iter(content) {
            for name in cap[1].split(',').map(|s| s.trim()) {
                exports.push(super::Export {
                    name: name.to_string(),
                    kind: super::SymbolKind::Variable,
                    is_public: true,
                });
            }
        }

        exports
    }
}

/// Python parser
pub struct PythonParser;

#[async_trait::async_trait]
impl LanguageParser for PythonParser {
    async fn analyze(&self, path: &PathBuf, content: &str) -> Result<super::SemanticAnalysis> {
        let analyzer = SemanticAnalyzer::new();
        
        let complexity = analyzer.calculate_complexity(content, "py");
        let dependencies = analyzer.extract_dependencies(path, content);
        let imports = self.parse_imports(content);
        let exports = self.parse_exports(content);

        Ok(super::SemanticAnalysis {
            complexity: complexity.into(),
            dependencies,
            exports,
            imports,
        })
    }

    fn parse_symbols(&self, content: &str) -> Vec<ParsedSymbol> {
        let mut symbols = Vec::new();
        
        // Parse functions
        let fn_regex = Regex::new(r"(?:async\s+)?def\s+(\w+)").unwrap();
        for (i, line) in content.lines().enumerate() {
            if let Some(cap) = fn_regex.captures(line) {
                // Check if it's a method (indented)
                let is_public = !cap[1].starts_with('_');
                symbols.push(ParsedSymbol {
                    name: cap[1].to_string(),
                    kind: super::SymbolKind::Function,
                    line: i as u32,
                    is_public,
                    documentation: None,
                });
            }
        }

        // Parse classes
        let class_regex = Regex::new(r"class\s+(\w+)").unwrap();
        for (i, line) in content.lines().enumerate() {
            if let Some(cap) = class_regex.captures(line) {
                let is_public = !cap[1].starts_with('_');
                symbols.push(ParsedSymbol {
                    name: cap[1].to_string(),
                    kind: super::SymbolKind::Class,
                    line: i as u32,
                    is_public,
                    documentation: None,
                });
            }
        }

        symbols
    }

    fn parse_imports(&self, content: &str) -> Vec<super::Import> {
        let mut imports = Vec::new();
        
        // from X import Y
        let from_import_regex = Regex::new(r"from\s+(\S+)\s+import\s+(.+)").unwrap();
        for cap in from_import_regex.captures_iter(content) {
            let source = cap[1].to_string();
            let items: Vec<String> = cap[2].split(',').map(|s| s.trim().to_string()).collect();
            
            imports.push(super::Import {
                source,
                items,
                is_default: false,
            });
        }

        // import X
        let import_regex = Regex::new(r"^import\s+(.+)").unwrap();
        for cap in import_regex.captures_iter(content) {
            let items: Vec<String> = cap[1].split(',').map(|s| s.trim().to_string()).collect();
            
            imports.push(super::Import {
                source: items[0].clone(),
                items,
                is_default: true,
            });
        }

        imports
    }

    fn parse_exports(&self, content: &str) -> Vec<super::Export> {
        let mut exports = Vec::new();
        
        // In Python, exports are typically defined in __all__
        let all_regex = Regex::new(r"__all__\s*=\s*\[([^\]]+)\]").unwrap();
        if let Some(cap) = all_regex.captures(content) {
            for name in cap[1].split(',').map(|s| s.trim().trim_matches('"').trim_matches('\'')) {
                if !name.is_empty() {
                    exports.push(super::Export {
                        name: name.to_string(),
                        kind: super::SymbolKind::Variable,
                        is_public: true,
                    });
                }
            }
        }

        exports
    }
}

/// Go parser
pub struct GoParser;

#[async_trait::async_trait]
impl LanguageParser for GoParser {
    async fn analyze(&self, path: &PathBuf, content: &str) -> Result<super::SemanticAnalysis> {
        let analyzer = SemanticAnalyzer::new();
        
        let complexity = analyzer.calculate_complexity(content, "go");
        let dependencies = analyzer.extract_dependencies(path, content);
        let imports = self.parse_imports(content);
        let exports = self.parse_exports(content);

        Ok(super::SemanticAnalysis {
            complexity: complexity.into(),
            dependencies,
            exports,
            imports,
        })
    }

    fn parse_symbols(&self, content: &str) -> Vec<ParsedSymbol> {
        let mut symbols = Vec::new();
        
        // Parse functions
        let fn_regex = Regex::new(r"func\s+(?:\([^)]+\)\s+)?(\w+)").unwrap();
        for (i, line) in content.lines().enumerate() {
            if let Some(cap) = fn_regex.captures(line) {
                let is_public = cap[1].chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                symbols.push(ParsedSymbol {
                    name: cap[1].to_string(),
                    kind: super::SymbolKind::Function,
                    line: i as u32,
                    is_public,
                    documentation: None,
                });
            }
        }

        // Parse types (structs, interfaces)
        let type_regex = Regex::new(r"type\s+(\w+)").unwrap();
        for (i, line) in content.lines().enumerate() {
            if let Some(cap) = type_regex.captures(line) {
                let is_public = cap[1].chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                let kind = if line.contains("interface") {
                    super::SymbolKind::Interface
                } else {
                    super::SymbolKind::Struct
                };
                
                symbols.push(ParsedSymbol {
                    name: cap[1].to_string(),
                    kind,
                    line: i as u32,
                    is_public,
                    documentation: None,
                });
            }
        }

        symbols
    }

    fn parse_imports(&self, content: &str) -> Vec<super::Import> {
        let mut imports = Vec::new();
        
        // Single import
        let import_regex = Regex::new(r#"import\s+["`]([^"`]+)["`]"#).unwrap();
        for cap in import_regex.captures_iter(content) {
            imports.push(super::Import {
                source: cap[1].to_string(),
                items: vec![],
                is_default: true,
            });
        }

        // Block imports
        let block_import_regex = Regex::new(r"import\s+\(([^)]+)\)").unwrap();
        for cap in block_import_regex.captures_iter(content) {
            for line in cap[1].lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('"') || trimmed.starts_with('`') {
                    let path = trimmed.trim_matches('"').trim_matches('`');
                    imports.push(super::Import {
                        source: path.to_string(),
                        items: vec![],
                        is_default: true,
                    });
                }
            }
        }

        imports
    }

    fn parse_exports(&self, content: &str) -> Vec<super::Export> {
        let mut exports = Vec::new();
        
        // In Go, exports are identifiers starting with uppercase
        // Already captured in parse_symbols
        
        exports
    }
}

/// Code complexity metrics
#[derive(Debug, Clone, Default)]
pub struct CodeComplexity {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub lines_of_code: u32,
    pub lines_of_comments: u32,
    pub blank_lines: u32,
}

impl From<CodeComplexity> for super::CodeComplexity {
    fn from(c: CodeComplexity) -> Self {
        super::CodeComplexity {
            cyclomatic_complexity: c.cyclomatic_complexity,
            cognitive_complexity: c.cognitive_complexity,
            lines_of_code: c.lines_of_code,
            lines_of_comments: c.lines_of_comments,
        }
    }
}

/// Dead code detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeResult {
    pub symbol: String,
    pub location: super::Location,
    pub reason: DeadCodeReason,
}

/// Reasons for code being considered dead
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeadCodeReason {
    Unused,
    Unreachable,
    Redundant,
}

/// Dependency graph for the codebase
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

/// Node in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub path: PathBuf,
    pub kind: NodeKind,
}

/// Types of nodes in the graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    File,
    Module,
    Function,
    Type,
}

/// Edge in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
}

/// Types of edges in the graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Imports,
    Uses,
    Implements,
    Extends,
    DependsOn,
}

/// Code metrics collector
pub struct MetricsCollector;

impl MetricsCollector {
    /// Collect all metrics for a file
    pub fn collect_metrics(&self, path: &PathBuf, content: &str) -> Result<CodeMetrics> {
        let analyzer = SemanticAnalyzer::new();
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        
        let complexity = analyzer.calculate_complexity(content, extension);
        
        // Calculate Halstead metrics (simplified)
        let halstead = self.calculate_halstead(content);
        
        // Calculate maintainability index (simplified)
        let maintainability = self.calculate_maintainability(&complexity, &halstead);

        Ok(CodeMetrics {
            lines_of_code: complexity.lines_of_code,
            lines_of_comments: complexity.lines_of_comments,
            blank_lines: complexity.blank_lines,
            cyclomatic_complexity: complexity.cyclomatic_complexity,
            cognitive_complexity: complexity.cognitive_complexity,
            halstead_metrics: halstead,
            maintainability_index: maintainability,
        })
    }

    fn calculate_halstead(&self, content: &str) -> HalsteadMetrics {
        // Simplified Halstead metrics calculation
        let operators_regex = Regex::new(r"[+\-*/%=<>!&|^~]+|&&|\|\||<<|>>|->|=>|\+\+|--").unwrap();
        let operands_regex = Regex::new(r"\b[a-zA-Z_]\w*\b|\b\d+\b").unwrap();
        
        let operators: HashSet<_> = operators_regex.find_iter(content).map(|m| m.as_str()).collect();
        let operands: HashSet<_> = operands_regex.find_iter(content).map(|m| m.as_str()).collect();
        
        let n1 = operators.len() as u32;
        let n2 = operands.len() as u32;
        let N1 = operators_regex.find_iter(content).count() as u32;
        let N2 = operands_regex.find_iter(content).count() as u32;
        
        let vocabulary = (n1 + n2) as f32;
        let length = (N1 + N2) as f32;
        let volume = if vocabulary > 0.0 {
            length * vocabulary.log2()
        } else {
            0.0
        };
        let difficulty = if n2 > 0 {
            (n1 as f32 * N2 as f32) / (2.0 * n2 as f32)
        } else {
            0.0
        };
        let effort = difficulty * volume;

        HalsteadMetrics {
            operators: N1,
            operands: N2,
            unique_operators: n1,
            unique_operands: n2,
            program_length: length,
            vocabulary_size: vocabulary,
            volume,
            difficulty,
            effort,
        }
    }

    fn calculate_maintainability(&self, complexity: &CodeComplexity, halstead: &HalsteadMetrics) -> f32 {
        // Simplified maintainability index
        // MI = 171 - 5.2 * ln(Halstead Volume) - 0.23 * (Cyclomatic Complexity) - 16.2 * ln(Lines of Code)
        let loc = complexity.lines_of_code.max(1) as f32;
        let vol = halstead.volume.max(1.0);
        let cc = complexity.cyclomatic_complexity as f32;
        
        let mi = 171.0 - 5.2 * vol.ln() - 0.23 * cc - 16.2 * loc.ln();
        mi.max(0.0).min(100.0)
    }

    /// Calculate code coverage (if available)
    pub fn calculate_coverage(&self, _path: &PathBuf) -> Result<CoverageMetrics> {
        // TODO: Implement coverage calculation from coverage reports
        Ok(CoverageMetrics::default())
    }
}

/// Comprehensive code metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub lines_of_code: u32,
    pub lines_of_comments: u32,
    pub blank_lines: u32,
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub halstead_metrics: HalsteadMetrics,
    pub maintainability_index: f32,
}

/// Halstead complexity metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HalsteadMetrics {
    pub operators: u32,
    pub operands: u32,
    pub unique_operators: u32,
    pub unique_operands: u32,
    pub program_length: f32,
    pub vocabulary_size: f32,
    pub volume: f32,
    pub difficulty: f32,
    pub effort: f32,
}

/// Code coverage metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoverageMetrics {
    pub line_coverage: f32,
    pub branch_coverage: f32,
    pub function_coverage: f32,
    pub uncovered_lines: Vec<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dead_code_reason_serialization() {
        let reason = DeadCodeReason::Unused;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, "\"unused\"");
    }

    #[test]
    fn test_rust_complexity() {
        let analyzer = SemanticAnalyzer::new();
        let code = r#"
fn test() {
    if x > 0 {
        if y > 0 {
            do_something();
        }
    }
    match value {
        Some(x) => x,
        None => 0,
    }
}
"#;
        let complexity = analyzer.calculate_complexity(code, "rs");
        assert!(complexity.cyclomatic_complexity > 1);
    }

    #[test]
    fn test_rust_imports() {
        let parser = RustParser;
        let code = r#"
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::utils::helpers;
"#;
        let imports = parser.parse_imports(code);
        assert_eq!(imports.len(), 3);
    }

    #[test]
    fn test_js_imports() {
        let parser = TypeScriptParser;
        let code = r#"
import React from 'react';
import { useState, useEffect } from 'react';
import * as utils from './utils';
"#;
        let imports = parser.parse_imports(code);
        assert_eq!(imports.len(), 3);
    }

    #[test]
    fn test_halstead_metrics() {
        let collector = MetricsCollector;
        let code = "fn main() { let x = 1 + 2; let y = x * 3; }";
        let metrics = collector.calculate_halstead(code);
        assert!(metrics.vocabulary_size > 0.0);
        assert!(metrics.volume > 0.0);
    }

}