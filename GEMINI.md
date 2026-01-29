# GEMINI.md: Your Guide to the Self-Developing Coding Agent

This document provides the essential context for understanding, running, and contributing to this self-developing coding agent. It is intended to be used as a primary reference for interacting with the Gemini CLI.

## Project Overview

This project is a sophisticated, self-developing coding agent built in Rust. It operates in a continuous improvement loop, leveraging Large Language Model (LLM) capabilities, Language Server Protocol (LSP) integration, and a comprehensive tool ecosystem to become more efficient, effective, and precise over time.

### Key Design Principles

*   **Provider Agnostic**: Supports multiple LLM providers (OpenAI, Anthropic, local models).
*   **Language Agnostic**: Multi-language LSP support for diverse codebases.
*   **Tool Extensible**: A plugin-based tool integration framework allows for easy extension.
*   **Safety First**: Strict guardrails are in place to protect the core system and user data.
*   **Documentation Driven**: The agent persists knowledge through structured documentation.
*   **Continuous Learning**: It is designed for self-improvement through feedback loops and re-prompting.

### Architecture

The agent is built with a modular, multi-crate workspace architecture. The key components are:

*   **`agent-core`**: The central orchestrator that manages the agent's operation loop.
*   **`intelligence`**: An abstraction layer for LLM provider interactions, prompt management, and the re-prompting engine.
*   **`analysis`**: Integrates with the Language Server Protocol (LSP) for deep code analysis, including AST parsing and semantic understanding.
*   **`knowledge`**: A knowledge management layer that uses a vector store (Qdrant), a knowledge graph (Neo4j), and a document store for long-term memory.
*   **`tools`**: A framework for integrating tools, with built-in support for file system operations, Git, HTTP requests, and search.
*   **`config`**: A hierarchical configuration system that manages settings for the agent and its various components.
*   **`common`**: Shared utilities and types used across the workspace.

For a deep dive into the system's design, please refer to the [**`docs/ARCHITECTURE.md`**](./docs/ARCHITECTURE.md) file.

## Building and Running

This project uses the standard Rust toolchain.

*   **Build the project:**
    ```bash
    cargo build --release
    ```

*   **Run the agent:**
    The main binary is `coding-agent`.
    ```bash
    cargo run --bin coding-agent -- [args]
    ```

*   **Run tests:**
    ```bash
    cargo test --workspace
    ```

*   **Check formatting and linting:**
    ```bash
    cargo fmt --all -- --check
    cargo clippy --workspace -- -D warnings
    ```

## Development Conventions

*   **Configuration**: The agent is configured via a set of `*.toml` files, typically located in a `.agent/` directory, and can be overridden by environment variables. See the "Configuration Architecture" section in `docs/ARCHITECTURE.md` for details.
*   **Tooling**: This project follows standard Rust conventions. Use `rustfmt` for formatting and `clippy` for linting.
*   **Modularity**: Functionality is organized into separate crates. When adding new features, consider which crate is the most appropriate home or if a new crate is necessary.
*   **Extensibility**: The agent is designed to be extensible. New tools can be added by implementing the `Tool` trait and registering them with the tool framework.
*   **Logging and Tracing**: The project uses the `tracing` crate for structured logging. Familiarize yourself with the different log levels and how to enable them for debugging.
*   **Security**: A comprehensive security audit has been completed. See [SECURITY_AUDIT.md](SECURITY_AUDIT.md) for details on vulnerabilities and recommended fixes.
