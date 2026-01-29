# Development Guide

This guide provides detailed information for developers working on the Self-Developing Coding Agent project.

## Table of Contents

- [Core Principles](#core-principles)
- [Development Environment Setup](#development-environment-setup)
- [Project Structure](#project-structure)
- [Building and Testing](#building-and-testing)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Debugging](#debugging)
- [Contributing](#contributing)
- [Release Process](#release-process)

---

## Core Principles

Before diving into development, it's essential to understand the core principles that guide this project:

### Principles of Safe and Performant Software Development

The project follows comprehensive principles for safe and performant software development, synthesized from industry best practices including:

- **TigerBeetle's tiger_style.md**: Systems programming best practices
- **NASA's Power of 10**: Critical systems development rules
- **Casey Muratori's Performance Aware Programming**: Performance optimization principles
- **Rust-specific practices**: Ownership model, zero-cost abstractions, memory safety

For detailed information on these principles, see [`PRINCIPLES.md`](PRINCIPLES.md).

### Agent Instructions

For AI agents working on this project, specific instructions and prompt templates are provided in [`AGENT_INSTRUCTIONS.md`](AGENT_INSTRUCTIONS.md). This document includes:

- Core system instructions for AI agents
- Prompt templates for various tasks
- Code generation and modification guidelines
- Safety constraints and guardrails
- Performance considerations
- Self-improvement guidelines

### Key Principles Summary

When working on this project, always keep these core principles in mind:

#### Safety Principles
- **Memory Safety**: Use Rust's ownership system to prevent data races and memory issues
- **Explicit Behavior**: Make all behavior explicit and visible
- **Fail Fast**: Detect errors early and make them impossible to ignore
- **Defensive Programming**: Validate all inputs and handle errors appropriately
- **Isolation**: Limit the blast radius of failures through proper isolation

#### Performance Principles
- **Measure Before Optimizing**: Never optimize without measurements
- **Algorithmic Efficiency**: Choose the right algorithm before micro-optimizing
- **Minimize Allocations**: Reduce memory allocations, especially in hot paths
- **Cache-Friendly Design**: Design for CPU cache efficiency
- **Zero-Cost Abstractions**: Use abstractions that compile to efficient code

#### Quality Principles
- **Simplicity**: Write simple, clear code that is easy to understand
- **Explicit Over Implicit**: Make behavior visible and predictable
- **Comprehensive Testing**: Write tests for all code
- **Documentation**: Document all public APIs and complex implementations
- **Code Review**: Ensure all code meets quality standards

---

## Development Environment Setup

### Prerequisites

- **Rust**: Version 1.75 or later
- **Cargo**: Included with Rust
- **Git**: For version control
- **Optional**: Docker for containerized development

### Installation

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Clone the repository**:
   ```bash
   git clone https://github.com/example/coding-agent.git
   cd coding-agent
   ```

3. **Install development tools**:
   ```bash
   cargo install cargo-watch
   cargo install cargo-edit
   cargo install cargo-audit
   ```

4. **Set up pre-commit hooks** (optional but recommended):
   ```bash
   cargo install cargo-husky
   cargo husky install
   ```

### IDE Setup

#### VS Code

Install the following extensions:
- **rust-analyzer**: Rust language server
- **CodeLLDB**: Debugging support
- **Even Better TOML**: TOML syntax highlighting
- **Markdown All in One**: Markdown support

#### IntelliJ IDEA / CLion

Install the **Rust** plugin from JetBrains Marketplace.

### Environment Variables

Create a `.env` file in the project root (add to `.gitignore`):

```bash
# LLM API Keys
ANTHROPIC_API_KEY=your_anthropic_api_key
OPENAI_API_KEY=your_openai_api_key

# Development Settings
RUST_LOG=debug
RUST_BACKTRACE=1

# Test Settings
TEST_LLM_PROVIDER=mock
```

---

## Project Structure

### Workspace Overview

```
coding-agent/
├── crates/                    # Workspace crates
│   ├── agent-core/           # Core orchestration
│   ├── intelligence/         # LLM integration
│   ├── analysis/             # LSP integration
│   ├── knowledge/            # Knowledge management
│   ├── tools/                # Tool framework
│   ├── config/               # Configuration
│   └── common/               # Shared utilities
├── src/                      # Binary entry point
│   └── main.rs
├── docs/                     # Documentation
│   ├── ARCHITECTURE.md
│   ├── ROADMAP.md
│   └── DEVELOPMENT.md
├── tests/                    # Integration tests
├── examples/                 # Example code
├── scripts/                  # Utility scripts
├── .github/                  # GitHub workflows
├── Cargo.toml                # Workspace manifest
└── README.md
```

### Crate Responsibilities

#### [`agent-core`](../crates/agent-core/)

**Purpose**: Core orchestration and state management

**Key Modules**:
- [`orchestrator.rs`](../crates/agent-core/src/orchestrator.rs): Central coordination
- [`state.rs`](../crates/agent-core/src/state.rs): State machine
- [`improvement.rs`](../crates/agent-core/src/improvement.rs): Self-improvement
- [`self_compile.rs`](../crates/agent-core/src/self_compile.rs): Self-compilation

**Dependencies**: `common`, `config`, `intelligence`, `analysis`, `knowledge`, `tools`

#### [`intelligence`](../crates/intelligence/)

**Purpose**: LLM integration and prompt management

**Key Modules**:
- [`gateway.rs`](../crates/intelligence/src/gateway.rs): LLM provider abstraction
- [`prompt.rs`](../crates/intelligence/src/prompt.rs): Prompt templates
- [`intent.rs`](../crates/intelligence/src/intent.rs): Intent parsing

**Dependencies**: `common`, `config`

#### [`analysis`](../crates/analysis/)

**Purpose**: LSP integration and code analysis

**Key Modules**:
- [`lsp.rs`](../crates/analysis/src/lsp.rs): LSP client
- [`semantic.rs`](../crates/analysis/src/semantic.rs): Semantic analysis

**Dependencies**: `common`, `config`

#### [`knowledge`](../crates/knowledge/)

**Purpose**: Documentation and knowledge management

**Key Modules**:
- [`documentation.rs`](../crates/knowledge/src/documentation.rs): Document management
- [`graph.rs`](../crates/knowledge/src/graph.rs): Knowledge graph
- [`vector.rs`](../crates/knowledge/src/vector.rs): Vector store

**Dependencies**: `common`, `config`

#### [`tools`](../crates/tools/)

**Purpose**: Tool framework and built-in tools

**Key Modules**:
- [`filesystem.rs`](../crates/tools/src/filesystem.rs): File operations
- [`git.rs`](../crates/tools/src/git.rs): Git integration
- [`http.rs`](../crates/tools/src/http.rs): HTTP client
- [`search.rs`](../crates/tools/src/search.rs): Search capabilities

**Dependencies**: `common`, `config`

#### [`config`](../crates/config/)

**Purpose**: Configuration management

**Key Modules**:
- [`lib.rs`](../crates/config/src/lib.rs): Configuration loading

**Dependencies**: `common`

#### [`common`](../crates/common/)

**Purpose**: Shared types and utilities

**Key Modules**:
- [`lib.rs`](../crates/common/src/lib.rs): Common types

**Dependencies**: None

---

## Building and Testing

### Building

#### Debug Build

```bash
cargo build
```

#### Release Build

```bash
cargo build --release
```

#### Build Specific Crate

```bash
cargo build -p agent-core
```

#### Build with Features

```bash
cargo build --features "dev-tools"
```

### Testing

#### Run All Tests

```bash
cargo test
```

#### Run Tests with Output

```bash
cargo test -- --nocapture
```

#### Run Tests for Specific Crate

```bash
cargo test -p agent-core
```

#### Run Specific Test

```bash
cargo test test_name
```

#### Run Tests in Parallel

```bash
cargo test --test-threads=4
```

#### Run Integration Tests

```bash
cargo test --test '*'
```

### Code Quality

#### Format Code

```bash
cargo fmt
```

#### Check Formatting

```bash
cargo fmt -- --check
```

#### Run Clippy

```bash
cargo clippy
```

#### Run Clippy with Warnings as Errors

```bash
cargo clippy -- -D warnings
```

#### Check for Security Vulnerabilities

```bash
cargo audit
```

#### Check for Outdated Dependencies

```bash
cargo outdated
```

### Development Workflow

#### Watch for Changes and Rebuild

```bash
cargo watch -x build
```

#### Watch for Changes and Run Tests

```bash
cargo watch -x test
```

#### Watch for Changes and Run Clippy

```bash
cargo watch -x clippy
```

---

## Coding Standards

### Rust Conventions

Follow the official [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

- Use `snake_case` for variables and functions
- Use `PascalCase` for types and traits
- Use `SCREAMING_SNAKE_CASE` for constants
- Use `#[must_use]` for functions with important return values
- Prefer `Result<T, E>` over `Option<T>` for errors
- Use `thiserror` for error types
- Use `anyhow` for application errors

### Documentation

#### Public API Documentation

All public items must have documentation comments:

```rust
/// Executes a task and returns the result.
///
/// # Arguments
///
/// * `task` - The task to execute
///
/// # Returns
///
/// Returns a `Result` containing the task result or an error.
///
/// # Errors
///
/// Returns an error if the task execution fails.
///
/// # Examples
///
/// ```
/// use agent_core::Task;
///
/// let task = Task::new("test task");
/// let result = execute_task(task)?;
/// ```
pub async fn execute_task(task: Task) -> Result<TaskResult> {
    // Implementation
}
```

#### Module Documentation

Each module should have a module-level comment:

```rust
//! Task orchestration and management.
//!
//! This module provides the core functionality for managing tasks,
//! including task creation, execution, and result handling.
```

#### Inline Comments

Use inline comments to explain complex logic:

```rust
// Calculate the optimal batch size based on available memory
// and the average item size to avoid OOM errors
let batch_size = (available_memory / avg_item_size).min(MAX_BATCH_SIZE);
```

### Error Handling

#### Define Error Types

Use `thiserror` for error types:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("LLM request failed: {0}")]
    LlmError(#[from] reqwest::Error),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Task execution failed: {0}")]
    TaskError(String),
}
```

#### Handle Errors Appropriately

```rust
// Use ? for propagating errors
pub async fn process_task(task: Task) -> Result<TaskResult> {
    let result = execute_task(task)?;
    Ok(result)
}

// Use match for specific error handling
match execute_task(task).await {
    Ok(result) => println!("Success: {:?}", result),
    Err(AgentError::LlmError(e)) => eprintln!("LLM error: {}", e),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Async/Await

#### Use Async for I/O Operations

```rust
use tokio::fs;

pub async fn read_file(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path).await?;
    Ok(content)
}
```

#### Use `tokio::spawn` for Concurrent Tasks

```rust
let task1 = tokio::spawn(async move {
    process_file("file1.txt").await
});

let task2 = tokio::spawn(async move {
    process_file("file2.txt").await
});

let (result1, result2) = tokio::join!(task1, task2);
```

### Logging

#### Use Structured Logging

```rust
use tracing::{info, warn, error, debug};

info!("Starting task execution");
debug!("Task details: {:?}", task);
warn!("Task took longer than expected");
error!("Task execution failed: {}", error);
```

#### Use Instrumentation for Spans

```rust
use tracing::instrument;

#[instrument(skip(self))]
pub async fn execute_task(&self, task: Task) -> Result<TaskResult> {
    info!("Executing task: {}", task.id);
    // Implementation
}
```

---

## Testing Guidelines

### Unit Tests

#### Write Tests for All Public Functions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("test task");
        assert_eq!(task.description, "test task");
    }

    #[test]
    fn test_task_with_priority() {
        let task = Task::new("test task")
            .with_priority(TaskPriority::High);
        assert_eq!(task.priority, TaskPriority::High);
    }
}
```

#### Use Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_task_id_generation(desc in "[a-z]+") {
        let task = Task::new(&desc);
        assert!(!task.id.is_empty());
    }
}
```

### Integration Tests

#### Create Integration Tests in `tests/` Directory

```rust
// tests/integration_test.rs
use coding_agent::Agent;

#[tokio::test]
async fn test_end_to_end_task_execution() {
    let agent = Agent::new(test_config()).await.unwrap();
    let task = Task::new("test task");
    let result = agent.submit_task(task).await.unwrap();
    assert!(result.success);
}
```

### Mocking

#### Use `mockall` for Mocking Dependencies

```rust
use mockall::mock;

mock! {
    LlmGateway {}

    #[async_trait]
    impl LlmGateway for LlmGateway {
        async fn generate(&self, prompt: &str) -> Result<String>;
    }
}

#[tokio::test]
async fn test_with_mock_llm() {
    let mut mock_llm = MockLlmGateway::new();
    mock_llm
        .expect_generate()
        .returning(|_| Ok("mock response".to_string()));

    let result = process_with_llm(&mock_llm).await;
    assert!(result.is_ok());
}
```

### Test Organization

#### Organize Tests by Module

```
crates/agent-core/src/
├── orchestrator.rs
├── orchestrator_tests.rs  # Unit tests for orchestrator
├── state.rs
├── state_tests.rs        # Unit tests for state
└── lib.rs
```

#### Use Test Fixtures

```rust
#[cfg(test)]
mod fixtures {
    use super::*;

    pub fn create_test_task() -> Task {
        Task::new("test task")
            .with_priority(TaskPriority::Normal)
    }

    pub fn create_test_config() -> AgentConfig {
        AgentConfig::default()
    }
}
```

---

## Debugging

### Using `println!` and `dbg!`

#### Quick Debugging

```rust
// Use dbg! for quick debugging
let result = dbg!(calculate_something());

// Use println! for formatted output
println!("Result: {:?}", result);
```

### Using `tracing` for Structured Debugging

#### Enable Debug Logging

```bash
RUST_LOG=debug cargo run
```

#### Use Span Instrumentation

```rust
use tracing::{instrument, Level};

#[instrument(level = Level::DEBUG, skip(self))]
pub async fn execute_task(&self, task: Task) -> Result<TaskResult> {
    debug!("Executing task: {}", task.id);
    // Implementation
}
```

### Using a Debugger

#### VS Code Debugging

Create `.vscode/launch.json`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug coding-agent",
            "cargo": {
                "args": [
                    "build",
                    "--bin=coding-agent"
                ],
                "filter": {
                    "name": "coding-agent",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

#### GDB Debugging

```bash
cargo build
gdb target/debug/coding-agent
```

### Common Debugging Scenarios

#### Debugging Async Code

```rust
// Use tokio-console for async debugging
cargo install tokio-console
cargo run --features tokio-console
```

#### Debugging Memory Issues

```bash
# Use valgrind for memory debugging
cargo build
valgrind --leak-check=full target/debug/coding-agent
```

#### Debugging Performance Issues

```bash
# Use flamegraph for performance profiling
cargo install flamegraph
cargo flamegraph --bin coding-agent
```

---

## Contributing

### Workflow

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/my-feature`
3. **Make your changes**
4. **Write tests**: Ensure all tests pass
5. **Format code**: `cargo fmt`
6. **Run clippy**: `cargo clippy -- -D warnings`
7. **Commit changes**: `git commit -m "Add my feature"`
8. **Push to branch**: `git push origin feature/my-feature`
9. **Open a Pull Request**

### Commit Message Format

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Test changes
- `chore`: Build process or auxiliary tool changes

**Examples**:
```
feat(orchestrator): add task priority support

Implement task priority levels (High, Normal, Low) and
update the orchestrator to prioritize tasks accordingly.

Closes #123
```

```
fix(llm): handle rate limit errors

Add retry logic with exponential backoff when LLM API
returns rate limit errors.

Fixes #456
```

### Pull Request Guidelines

#### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] All tests pass

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Code is formatted (`cargo fmt`)
- [ ] Clippy passes (`cargo clippy -- -D warnings`)
- [ ] Documentation updated
- [ ] No new warnings generated
```

### Code Review Process

1. **Self-Review**: Review your own changes before submitting
2. **Automated Checks**: Ensure all CI checks pass
3. **Peer Review**: At least one approval required
4. **Address Feedback**: Respond to all review comments
5. **Final Approval**: Maintainer approval required for merge

---

## Release Process

### Versioning

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible functionality additions
- **PATCH**: Backwards-compatible bug fixes

### Release Checklist

1. **Update Version**: Update `Cargo.toml` version
2. **Update Changelog**: Add release notes to `CHANGELOG.md`
3. **Run Tests**: Ensure all tests pass
4. **Build Release**: `cargo build --release`
5. **Tag Release**: `git tag -a v0.1.0 -m "Release v0.1.0"`
6. **Push Tag**: `git push origin v0.1.0`
7. **Create Release**: Create GitHub release with notes
8. **Publish Crates**: `cargo publish` for each crate

### Changelog Format

```markdown
## [0.1.0] - 2024-01-15

### Added
- Initial release
- Core orchestrator implementation
- LLM gateway with multi-provider support
- LSP integration for Rust and TypeScript

### Changed
- N/A

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- N/A
```

---

## Additional Resources

### Documentation

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tokio Documentation](https://tokio.rs/)
- [Tracing Documentation](https://docs.rs/tracing/)

### Tools

- [cargo-watch](https://github.com/passcod/cargo-watch)
- [cargo-edit](https://github.com/killercup/cargo-edit)
- [cargo-audit](https://github.com/RustSec/cargo-audit)
- [cargo-outdated](https://github.com/kbknapp/cargo-outdated)

### Community

- [Rust Users Forum](https://users.rust-lang.org/)
- [Rust Discord](https://discord.gg/rust-lang)
- [Stack Overflow - Rust](https://stackoverflow.com/questions/tagged/rust)

---

For more information, see:
- [Architecture Documentation](ARCHITECTURE.md)
- [Roadmap](ROADMAP.md)
- [Project README](../README.md)
