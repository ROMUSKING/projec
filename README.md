# Self-Developing Coding Agent

A sophisticated, self-improving coding agent built in Rust that leverages Large Language Models (LLMs), Language Server Protocol (LSP) integration, and a comprehensive tool ecosystem to become more efficient, effective, and precise over time.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Configuration](#configuration)
  - [Basic Usage](#basic-usage)
- [Project Structure](#project-structure)
- [Development](#development)
  - [Building](#building)
  - [Testing](#testing)
  - [Running](#running)
- [Contributing](#contributing)
- [License](#license)

## Overview

The Self-Developing Coding Agent is an autonomous software development assistant designed to:

- **Understand and execute complex coding tasks** through natural language instructions
- **Continuously improve its own capabilities** through feedback loops and self-modification
- **Leverage multiple LLM providers** (OpenAI, Anthropic, local models) for optimal performance
- **Integrate with language servers** for deep code understanding and analysis
- **Maintain persistent knowledge** through documentation, knowledge graphs, and vector stores
- **Execute safely** with comprehensive guardrails and validation systems

### Key Design Principles

1. **Provider Agnostic**: Supports multiple LLM providers (OpenAI, Anthropic, Ollama, OpenRouter, Arcee)
2. **Language Agnostic**: Multi-language LSP support for diverse codebases
3. **Tool Extensible**: Plugin-based tool integration framework
4. **Safety First**: Strict guardrails for core system protection
5. **Documentation Driven**: Knowledge persistence through structured documentation
6. **Continuous Learning**: Self-improvement through feedback loops and re-prompting

## Features

### Core Capabilities

- **Task Orchestration**: Centralized management of complex, multi-step coding tasks
- **Intent Understanding**: Natural language parsing and goal decomposition
- **Context Management**: Intelligent gathering of code, knowledge, and execution context
- **Tool Execution**: Comprehensive toolset for file operations, Git, testing, and more
- **LSP Integration**: Deep code analysis through language server protocol
- **Knowledge Management**: Persistent storage of patterns, decisions, and best practices

### Self-Improvement

- **Re-prompting Engine**: Automatic optimization of prompts based on performance feedback
- **Pattern Detection**: Identification of recurring patterns and optimization opportunities
- **Safe Self-Modification**: Controlled modification of agent code with rollback capabilities
- **Performance Monitoring**: Continuous tracking of metrics and effectiveness

### Safety & Security

- **Protected Resources**: Core system files and safety rules are immutable without approval
- **Audit Logging**: Comprehensive logging of all actions and modifications
- **Validation Layers**: Static analysis, runtime guards, and output validation
- **Recovery System**: Automatic rollback on failure with checkpoint restoration
- **Security Audit**: A comprehensive security audit has been completed. See [SECURITY_AUDIT.md](SECURITY_AUDIT.md) for details.

## Architecture

The agent is organized into a modular workspace with seven core crates:

```
coding-agent/
├── crates/
│   ├── agent-core/      # Core orchestration and state management
│   ├── intelligence/    # LLM gateway and prompt management
│   ├── analysis/        # LSP integration and semantic analysis
│   ├── knowledge/       # Documentation, knowledge graph, vector store
│   ├── tools/           # Tool framework and built-in tools
│   ├── config/          # Configuration management
│   └── common/          # Shared types and utilities
├── src/main.rs          # CLI entry point
└── docs/                # Documentation
```

### Core Modules

- **Orchestrator**: Central coordination module managing the agent's operation loop
- **State Manager**: Manages internal state and persistence
- **Intent Manager**: Parses and interprets user requests and self-generated goals
- **Prompt Manager**: Manages prompt templates and optimization
- **LLM Gateway**: Abstracts LLM provider interactions
- **LSP Manager**: Manages language server lifecycle and requests
- **Documentation Manager**: Handles document lifecycle and knowledge extraction
- **Tool Framework**: Plugin-based tool execution system

For detailed architecture information, see [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## Getting Started

### Prerequisites

- **Rust**: Version 1.75 or later
- **Cargo**: Included with Rust
- **LLM API Key**: OpenAI, Anthropic, Ollama (local), OpenRouter, or Arcee
- **Optional**: Language servers for your target languages (rust-analyzer, typescript-language-server, etc.)

### Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/example/coding-agent.git
   cd coding-agent
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Install the binary** (optional):
   ```bash
   cargo install --path .
   ```

### Configuration

The agent uses a hierarchical configuration system. Create a configuration file at `~/.config/agent/config.toml` or use the `--config` flag:

```toml
[agent]
name = "coding-agent"
version = "0.1.0"
improvement_interval = 3600  # seconds
max_concurrent_tasks = 4
log_level = "info"

[agent.self_improvement]
enabled = true
auto_apply = false  # Require approval for changes
safety_checks = true
max_modifications_per_session = 10

[llm]
provider = "openrouter"  # or "anthropic", "openai", "ollama", "arcee"
model = "arcee-ai/trinity-large-preview:free"  # or "claude-3-5-sonnet-20241022", "gpt-4o"
temperature = 0.7
max_tokens = 4096

[llm.providers.openrouter]
api_key = "${OPENROUTER_API_KEY}"
base_url = "https://openrouter.ai/api/v1"

[lsp]
enabled = true
timeout = 30

[[lsp.servers]]
name = "rust-analyzer"
command = "rust-analyzer"
filetypes = ["rust"]
root_patterns = ["Cargo.toml"]

[safety]
protected_paths = [
    ".agent/core/**",
    ".agent/safety/**",
    ".agent/auth/**"
]
max_file_size_mb = 10
forbidden_commands = ["rm -rf /", "dd if=/dev/zero"]
require_approval_for = ["delete", "modify_protected", "git_push"]

[tools.git]
enabled = true
auto_commit = false
commit_prefix = "[agent]"

[tools.test]
enabled = true
framework = "cargo"
auto_run = true
fail_on_error = true
```

### Basic Usage

#### Interactive Mode

Start the agent in interactive mode:

```bash
coding-agent
```

Available commands:
- `help` - Show help message
- `status` - Show current agent state
- `metrics` - Show performance metrics
- `improve` - Trigger self-improvement cycle
- `exit` - Exit interactive mode

Any other input is treated as a task description.

#### Single Task Execution

Execute a single task and exit:

```bash
coding-agent "Add error handling to the user authentication module"
```

Specify a workspace directory:

```bash
coding-agent --workspace ./my-project "Refactor the database layer"
```

#### Daemon Mode

Run the agent continuously in the background:

```bash
coding-agent --daemon
```

#### Self-Improvement

Trigger a self-improvement cycle:

```bash
coding-agent --improve
```

#### Show Metrics

Display current metrics:

```bash
coding-agent --metrics
```

## Project Structure

### Workspace Crates

#### [`agent-core`](crates/agent-core/)
Core orchestration and state management.

- **[`orchestrator.rs`](crates/agent-core/src/orchestrator.rs)**: Central coordination module
- **[`state.rs`](crates/agent-core/src/state.rs)**: State machine and persistence
- **[`improvement.rs`](crates/agent-core/src/improvement.rs)**: Self-improvement logic
- **[`self_compile.rs`](crates/agent-core/src/self_compile.rs)**: Self-compilation capabilities

#### [`intelligence`](crates/intelligence/)
LLM integration and prompt management.

- **[`gateway.rs`](crates/intelligence/src/gateway.rs)**: LLM provider abstraction
- **[`prompt.rs`](crates/intelligence/src/prompt.rs)**: Prompt template management
- **[`intent.rs`](crates/intelligence/src/intent.rs)**: Intent parsing and classification

#### [`analysis`](crates/analysis/)
LSP integration and code analysis.

- **[`lsp.rs`](crates/analysis/src/lsp.rs)**: LSP client implementation
- **[`semantic.rs`](crates/analysis/src/semantic.rs)**: Semantic analysis pipeline

#### [`knowledge`](crates/knowledge/)
Documentation and knowledge management.

- **[`documentation.rs`](crates/knowledge/src/documentation.rs)**: Document lifecycle management
- **[`graph.rs`](crates/knowledge/src/graph.rs)**: Knowledge graph operations
- **[`vector.rs`](crates/knowledge/src/vector.rs)**: Vector store integration

#### [`tools`](crates/tools/)
Tool framework and built-in tools.

- **[`filesystem.rs`](crates/tools/src/filesystem.rs)**: File system operations
- **[`git.rs`](crates/tools/src/git.rs)**: Git integration
- **[`http.rs`](crates/tools/src/http.rs)**: HTTP client
- **[`search.rs`](crates/tools/src/search.rs)**: Search capabilities

#### [`config`](crates/config/)
Configuration management.

- **[`lib.rs`](crates/config/src/lib.rs)**: Configuration loading and validation

#### [`common`](crates/common/)
Shared types and utilities.

- **[`lib.rs`](crates/common/src/lib.rs)**: Common types and utilities

### Documentation

- **[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)**: Detailed system architecture
- **[`docs/ROADMAP.md`](docs/ROADMAP.md)**: Development roadmap
- **[`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md)**: Development guide

## Development

### Building

Build the entire workspace:

```bash
cargo build
```

Build with optimizations:

```bash
cargo build --release
```

Build a specific crate:

```bash
cargo build -p agent-core
```

### Testing

Run all tests:

```bash
cargo test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

Run tests for a specific crate:

```bash
cargo test -p agent-core
```

### Running

Run the agent in development mode:

```bash
cargo run
```

Run with verbose logging:

```bash
cargo run -- --verbose
```

Run a specific task:

```bash
cargo run -- "Add unit tests for the parser module"
```

### Development Guidelines

#### Code Style

- Follow Rust naming conventions
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Document public APIs with rustdoc comments

#### Testing

- Write unit tests for all public functions
- Write integration tests for complex workflows
- Use `mockall` for mocking dependencies
- Maintain test coverage above 80%

#### Documentation

- Update `ARCHITECTURE.md` for structural changes
- Update `ROADMAP.md` for progress tracking
- Document new features in `README.md`
- Add inline documentation for complex logic

#### Safety

- Never modify protected paths without approval
- Always validate user input
- Use sandboxed execution for untrusted code
- Maintain audit logs for all modifications

## Contributing

We welcome contributions! Please follow these guidelines:

### Getting Started

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Write tests for your changes
5. Ensure all tests pass: `cargo test`
6. Format your code: `cargo fmt`
7. Run clippy: `cargo clippy -- -D warnings`
8. Commit your changes: `git commit -m "Add my feature"`
9. Push to the branch: `git push origin feature/my-feature`
10. Open a Pull Request

### Contribution Guidelines

- **Code Quality**: All code must pass `cargo clippy` and `cargo fmt`
- **Testing**: New features must include tests
- **Documentation**: Update relevant documentation for API changes
- **Breaking Changes**: Discuss breaking changes in an issue first
- **Commits**: Use clear, descriptive commit messages

### Reporting Issues

When reporting issues, please include:

- A clear description of the problem
- Steps to reproduce the issue
- Expected behavior
- Actual behavior
- Environment information (OS, Rust version, etc.)
- Relevant logs or error messages

## License

This project is dual-licensed under:

- **MIT License** - See [LICENSE-MIT](LICENSE-MIT) for details
- **Apache License 2.0** - See [LICENSE-APACHE](LICENSE-APACHE) for details

You may choose either license for your use.

## Acknowledgments

This project is inspired by and builds upon ideas from:

- [Claude Code](https://github.com/anthropics/claude-code)
- [Aider](https://github.com/paul-gauthier/aider)
- [Continue](https://github.com/continuedev/continue)
- [Supermaven](https://supermaven.com/)

## Related Projects

- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
- [Rust Analyzer](https://rust-analyzer.github.io/book/)
- [Qdrant Vector Database](https://qdrant.tech/)
- [Neo4j Graph Database](https://neo4j.com/)

---

For more information, see the [documentation](docs/) directory.
