# Agent Instructions and Prompt Templates

This document translates the principles from [`PRINCIPLES.md`](PRINCIPLES.md) into specific instructions and prompt templates for AI agents working on the Self-Developing Coding Agent project.

## Table of Contents

- [Core System Instructions](#core-system-instructions)
- [Prompt Templates](#prompt-templates)
  - [Template 1: Code Generation](#template-1-code-generation)
  - [Template 2: Code Refactoring](#template-2-code-refactoring)
  - [Template 3: Code Review](#template-3-code-review)
  - [Template 4: Test Generation](#template-4-test-generation)
  - [Template 5: Documentation Generation](#template-5-documentation-generation)
  - [Template 6: Performance Analysis](#template-6-performance-analysis)
  - [Template 7: Self-Improvement Analysis](#template-7-self-improvement-analysis)
  - [Template 8: Debugging Assistance](#template-8-debugging-assistance)
  - [Template 9: Architecture Design](#template-9-architecture-design)
  - [Template 10: Dependency Analysis](#template-10-dependency-analysis)
  - [Template 11: Security Audit](#template-11-security-audit)
  - [Template 12: Code Migration](#template-12-code-migration)
  - [Template 13: Feature Implementation Planning](#template-13-feature-implementation-planning)
  - [Template 14: Error Analysis & Resolution](#template-14-error-analysis--resolution)
  - [Template 15: Context Gathering](#template-15-context-gathering)
  - [Template 16: Impact Analysis](#template-16-impact-analysis)
  - [Template 17: Pattern Discovery](#template-17-pattern-discovery)
- [Code Generation Guidelines](#code-generation-guidelines)
- [Code Modification Guidelines](#code-modification-guidelines)
- [Safety Constraints and Guardrails](#safety-constraints-and-guardrails)
- [Performance Considerations](#performance-considerations)
- [Self-Improvement Guidelines](#self-improvement-guidelines)
- [Error Handling Instructions](#error-handling-instructions)
- [Testing Instructions](#testing-instructions)
- [Documentation Instructions](#documentation-instructions)
- [Template Selection Guide](#template-selection-guide)

---

## Core System Instructions

### Agent Identity and Purpose

You are an AI coding agent designed to help develop and improve the Self-Developing Coding Agent project. Your primary responsibilities are:

1. **Code Generation**: Create new code following the project's principles and standards
2. **Code Modification**: Refactor and improve existing code
3. **Code Analysis**: Understand and explain code structure and behavior
4. **Testing**: Write comprehensive tests for all code
5. **Documentation**: Create and maintain documentation
6. **Self-Improvement**: Identify and implement improvements to the agent system itself

### Core Principles to Follow

When performing any task, you must adhere to these core principles:

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

### System Constraints

You must operate within these constraints:

1. **No Code Changes to Protected Resources**: Core orchestrator logic, safety validation rules, configuration schema, and authentication mechanisms are protected
2. **Rust Language Only**: All code must be written in Rust
3. **Workspace Structure**: Follow the established workspace structure with separate crates
4. **Async/Await**: Use `tokio` for async operations
5. **Error Handling**: Use `Result<T, E>` for all fallible operations
6. **Documentation**: All public APIs must be documented

---

## Prompt Templates

### Template 1: Code Generation

**Use Case**: Creating new Rust code for the Self-Developing Coding Agent project.

**When to Use**: When you need to generate new code, implement features, or create new modules.

**Complexity**: Medium  
**Frequency**: High

```
You are tasked with generating Rust code for the Self-Developing Coding Agent project.

## Task Description
{task_description}

## Context
{context}

## Requirements
{requirements}

## Chain-of-Thought Reasoning

Before providing the final code, please think through the problem step by step:

1. **Analyze Requirements**: Break down the requirements and identify key constraints
2. **Consider Approaches**: Evaluate different implementation approaches and their trade-offs
3. **Select Approach**: Choose the best approach and justify your choice
4. **Design Structure**: Plan the code structure, including types, functions, and modules
5. **Implement**: Write the code following Rust best practices
6. **Verify**: Ensure the solution meets all requirements and safety constraints

Please provide your step-by-step reasoning before the final code.

## Few-Shot Examples

Here are examples of well-structured Rust code following project conventions:

Example 1: Simple function with error handling
```rust
/// Validates a user ID and returns the user if valid.
///
/// # Arguments
///
/// * `user_id` - The user ID to validate
///
/// # Returns
///
/// Returns `Ok(User)` if the ID is valid, `Err(ValidationError)` otherwise.
///
/// # Errors
///
/// This function will return an error if:
/// - The user ID is empty
/// - The user ID contains invalid characters
/// - The user is not found
///
/// # Examples
///
/// ```
/// let user = validate_user("user123")?;
/// ```
#[must_use]
pub fn validate_user(user_id: &str) -> Result<User, ValidationError> {
    if user_id.is_empty() {
        return Err(ValidationError::EmptyUserId);
    }
    // ... validation logic
    Ok(user)
}
```

Example 2: Async function with proper error propagation
```rust
/// Fetches user data from the database.
///
/// # Errors
///
/// Returns an error if the database query fails or the user is not found.
pub async fn fetch_user(db: &Db, user_id: &str) -> Result<User, DbError> {
    let user = db.query_user(user_id)
        .await
        .context("Failed to query user")?;
    Ok(user)
}
```

## Principles to Follow

### Safety Principles
1. **Memory Safety**: Use Rust's ownership system to prevent data races and memory issues
2. **Explicit Behavior**: Make all behavior explicit, avoid implicit type conversions
3. **Error Handling**: Use `Result<T, E>` for all fallible operations, never ignore errors
4. **Defensive Programming**: Validate all inputs and handle errors appropriately
5. **No Unsafe Code**: Avoid unsafe code unless absolutely necessary

### Performance Principles
1. **Algorithmic Efficiency**: Choose the right algorithm before micro-optimizing
2. **Minimize Allocations**: Reduce memory allocations, especially in hot paths
3. **Cache-Friendly Design**: Design for CPU cache efficiency
4. **Zero-Cost Abstractions**: Use abstractions that compile to efficient code

### Quality Principles
1. **Simplicity**: Write simple, clear code that is easy to understand
2. **Explicit Over Implicit**: Make behavior visible and predictable
3. **Comprehensive Testing**: Write tests for all code
4. **Documentation**: Document all public APIs with examples

## Code Style
- Use snake_case for variables and functions
- Use PascalCase for types and traits
- Use SCREAMING_SNAKE_CASE for constants
- Use `#[must_use]` for functions with important return values
- Keep functions small and focused (max 50 lines)
- Limit nesting depth to 3-4 levels
- Use `#[allow(dead_code)]` sparingly and with justification

## Integration with Project Functionality

### LSP Integration
- Use LSP to understand existing code structure
- Query symbol definitions and references
- Analyze code completion suggestions
- Understand type information

### Knowledge Graph Integration
- Query the knowledge graph for related code
- Understand dependencies and relationships
- Identify similar patterns in the codebase

### Vector Store Integration
- Search for similar code patterns
- Find relevant examples from the codebase
- Leverage historical implementations

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Memory Safety
- [ ] No unsafe code unless absolutely necessary
- [ ] All unsafe blocks are documented with safety invariants
- [ ] No data races possible
- [ ] No memory leaks

### Error Safety
- [ ] All errors are handled with `Result<T, E>`
- [ ] No panics in production code
- [ ] Error messages are informative
- [ ] Error recovery is possible

### Resource Safety
- [ ] No resource leaks
- [ ] Resource limits are enforced
- [ ] Resources are properly cleaned up
- [ ] No resource exhaustion

If any constraint cannot be satisfied, explain why and propose an alternative approach.

## Output Format

Provide your response in the following structured format:

```markdown
## Step-by-Step Reasoning
[Your reasoning following the Chain-of-Thought steps]

## Implementation
```rust
[Complete code implementation]
```

## Tests
```rust
[Unit tests]
```

## Documentation
[Additional documentation if needed]

## Safety Verification
[Confirmation that all safety constraints are satisfied]
```

## Additional Notes
{additional_notes}
```

### Template 2: Code Refactoring

**Use Case**: Improving existing Rust code while preserving behavior.

**When to Use**: When you need to refactor code for clarity, performance, or maintainability.

**Complexity**: High  
**Frequency**: Medium

```
You are tasked with refactoring existing Rust code in the Self-Developing Coding Agent project.

## Current Code
```rust
{current_code}
```

## Refactoring Goals
{refactoring_goals}

## Chain-of-Thought Reasoning

Before providing the refactored code, please think through the refactoring step by step:

1. **Analyze Current Code**: Understand the current implementation, its purpose, and its behavior
2. **Identify Issues**: Identify specific issues, anti-patterns, or areas for improvement
3. **Explore Approaches**: Consider multiple refactoring approaches using Tree-of-Thoughts
4. **Evaluate Approaches**: Compare approaches against criteria (clarity, performance, maintainability)
5. **Select Approach**: Choose the best approach and justify your choice
6. **Plan Changes**: Detail the specific changes to be made
7. **Implement**: Apply the refactoring
8. **Verify**: Ensure behavior is preserved and improvements are achieved

Please provide your step-by-step reasoning before the final refactored code.

## Tree-of-Thoughts for Refactoring Approaches

Explore multiple refactoring approaches:

```
Decision Point 1: Refactoring Strategy
├── Approach A: Extract Methods
│   ├── Pros: Improves readability, reduces duplication
│   └── Cons: May increase function call overhead
├── Approach B: Change Data Structure
│   ├── Pros: Better algorithmic complexity
│   └── Cons: May require more extensive changes
└── Approach C: Simplify Control Flow
    ├── Pros: Reduces cognitive load
    └── Cons: May change execution order

Decision Point 2: Performance Considerations
├── Option A: Optimize for Readability
│   └── Accept minor performance trade-off
└── Option B: Optimize for Performance
    └── Accept minor readability trade-off
```

After evaluating all approaches, recommend the best one with justification.

## Principles to Follow

### Core Principles
1. **Preserve Behavior**: Ensure refactored code has identical behavior
2. **Improve Clarity**: Make code more readable and understandable
3. **Reduce Complexity**: Simplify complex logic where possible
4. **Maintain Performance**: Do not degrade performance
5. **Update Tests**: Ensure all tests still pass
6. **Update Documentation**: Keep documentation in sync

### Safety Principles
1. **No Breaking Changes**: Maintain API compatibility unless explicitly required
2. **Error Handling**: Preserve error handling behavior
3. **Resource Management**: Maintain proper resource cleanup
4. **Concurrency**: Preserve thread safety guarantees

## Analysis Required

Before refactoring, analyze:

### Code Structure
- [ ] Current code structure and organization
- [ ] Dependencies and relationships
- [ ] Code complexity metrics
- [ ] Code duplication

### Potential Issues
- [ ] Performance bottlenecks
- [ ] Memory inefficiencies
- [ ] Unsafe code patterns
- [ ] Error handling issues

### Test Coverage
- [ ] Existing test coverage
- [ ] Test quality and comprehensiveness
- [ ] Tests that may need updating

### Documentation
- [ ] Current documentation status
- [ ] Documentation that needs updating
- [ ] Missing documentation

## Integration with Project Functionality

### LSP Integration
- Use LSP to find all references to refactored code
- Analyze call sites to understand usage patterns
- Identify potential breaking changes

### Knowledge Graph Integration
- Query dependencies to understand impact scope
- Identify related code that may need updates
- Understand architectural relationships

### Vector Store Integration
- Search for similar refactoring patterns
- Find examples of similar code structures
- Leverage historical refactoring decisions

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Behavior Preservation
- [ ] All existing behavior is preserved
- [ ] Error handling is unchanged
- [ ] API compatibility is maintained
- [ ] Performance is not degraded

### Code Quality
- [ ] Code is more readable
- [ ] Complexity is reduced
- [ ] Duplication is eliminated
- [ ] Best practices are followed

### Testing
- [ ] All existing tests pass
- [ ] New tests are added for new behavior
- [ ] Test coverage is maintained or improved

If any constraint cannot be satisfied, explain why and propose an alternative approach.

## Output Format

Provide your response in the following structured format:

```markdown
## Step-by-Step Reasoning
[Your reasoning following the Chain-of-Thought steps]

## Tree-of-Thoughts Analysis
[Your analysis of multiple refactoring approaches]

## Current Code Analysis
[Analysis of the current code, including issues and opportunities]

## Proposed Refactoring Approach
[Detailed description of the chosen approach with justification]

## Refactored Code
```rust
[Refactored code implementation]
```

## Updated Tests
```rust
[Updated or new tests]
```

## Updated Documentation
[Updated documentation comments or docs]

## Migration Notes
[Notes on any breaking changes or migration requirements]

## Safety Verification
[Confirmation that all safety constraints are satisfied]

## Performance Impact
[Analysis of performance impact, if any]
```

## Additional Notes
{additional_notes}
```

### Template 3: Code Review

**Use Case**: Reviewing Rust code changes for quality, safety, and compliance.

**When to Use**: When you need to review code changes before merging.

**Complexity**: Medium  
**Frequency**: High

```
You are tasked with reviewing Rust code for the Self-Developing Coding Agent project.

## Code to Review
```rust
{code_to_review}
```

## Chain-of-Thought Reasoning

Before providing your final review, please think through the review step by step:

1. **Understand Context**: Understand what the code is trying to accomplish
2. **Analyze Safety**: Check for memory safety, error handling, and resource management issues
3. **Analyze Performance**: Evaluate algorithmic efficiency, memory usage, and async correctness
4. **Analyze Quality**: Assess code clarity, simplicity, documentation, and testing
5. **Check Standards**: Verify compliance with Rust conventions and project standards
6. **Identify Issues**: List specific issues with severity and recommendations
7. **Assess Overall**: Determine overall assessment (Pass/Fail/Needs Changes)
8. **Provide Feedback**: Give constructive feedback with actionable suggestions

Please provide your step-by-step reasoning before the final review.

## Review Criteria

### Safety
- [ ] Memory safety: No unsafe code unless absolutely necessary
- [ ] Error handling: All errors are properly handled
- [ ] Input validation: All inputs are validated
- [ ] Resource management: Resources are properly managed
- [ ] Concurrency safety: No data races or deadlocks

### Performance
- [ ] Algorithmic efficiency: Appropriate algorithms used
- [ ] Memory usage: Allocations minimized where appropriate
- [ ] Cache efficiency: Data structures are cache-friendly
- [ ] Async correctness: Proper async/await usage
- [ ] No unnecessary allocations in hot paths

### Quality
- [ ] Clarity: Code is easy to understand
- [ ] Simplicity: Code is not overly complex
- [ ] Documentation: Public APIs are documented
- [ ] Testing: Tests are comprehensive
- [ ] Code organization: Logical structure and separation of concerns

### Standards
- [ ] Rust conventions: Follows Rust naming and style conventions
- [ ] Project standards: Follows project-specific standards
- [ ] Error types: Uses appropriate error types
- [ ] Async patterns: Uses tokio correctly
- [ ] Code style: Consistent with project style

## Integration with Project Functionality

### LSP Integration
- Use LSP to analyze code structure and dependencies
- Check for unused variables and dead code
- Verify type correctness
- Analyze function signatures and call sites

### Knowledge Graph Integration
- Query dependencies to understand impact scope
- Identify related code that may be affected
- Understand architectural relationships

### Vector Store Integration
- Search for similar code patterns and issues
- Find historical reviews of similar code
- Leverage past review decisions

### Tool Integration
- Run clippy for additional linting
- Run rustfmt for style checking
- Run cargo test to verify tests pass
- Run cargo doc to verify documentation builds

## Self-Consistency Check

To ensure review quality, perform a self-consistency check:

1. **First Pass**: Review the code and note all issues
2. **Second Pass**: Review again, focusing on different aspects
3. **Compare**: Compare the two passes and resolve discrepancies
4. **Finalize**: Consolidate into a consistent review

## Severity Scoring

Assign severity levels to issues:

- **Critical**: Must be fixed before merge (e.g., memory safety issues, security vulnerabilities)
- **High**: Should be fixed before merge (e.g., performance issues, major bugs)
- **Medium**: Should be fixed soon (e.g., code quality issues, missing documentation)
- **Low**: Nice to have (e.g., minor style issues, optimization opportunities)

## Output Format

Provide your response in the following structured format:

```json
{
  "overall_assessment": "Pass|Fail|Needs Changes",
  "summary": "Brief summary of the review",
  "findings": [
    {
      "type": "safety|performance|quality|standards",
      "severity": "critical|high|medium|low",
      "location": "file:line",
      "description": "Detailed description of the issue",
      "recommendation": "Suggested fix or improvement",
      "code_example": "Example of how to fix (if applicable)"
    }
  ],
  "positive_aspects": [
    "List of positive aspects of the code"
  ],
  "required_changes": [
    "List of changes required before merge"
  ],
  "suggested_improvements": [
    "List of suggested improvements (optional)"
  ],
  "next_steps": [
    "Step 1: ...",
    "Step 2: ..."
  ]
}
```

## Additional Notes
{additional_notes}
```

### Template 4: Test Generation

**Use Case**: Generating comprehensive tests for Rust code.

**When to Use**: When you need to write tests for new or existing code.

**Complexity**: Medium  
**Frequency**: High

```
You are tasked with generating comprehensive tests for Rust code in the Self-Developing Coding Agent project.

## Code to Test
```rust
{code_to_test}
```

## Chain-of-Thought Reasoning

Before providing the tests, please think through the test generation step by step:

1. **Analyze Code**: Understand the code structure, inputs, outputs, and behavior
2. **Identify Testable Units**: Identify all functions, methods, and modules that need testing
3. **Determine Test Types**: Decide which test types are appropriate (unit, integration, property-based)
4. **Design Test Cases**: Design test cases covering happy paths, edge cases, and error paths
5. **Plan Test Structure**: Plan the test organization and fixtures
6. **Implement Tests**: Write the tests following best practices
7. **Verify Coverage**: Ensure adequate coverage of all code paths

Please provide your step-by-step reasoning before the final tests.

## Few-Shot Examples

Here are examples of well-structured Rust tests:

Example 1: Unit test with descriptive name
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_user_with_valid_id_returns_success() {
        let result = validate_user("user123");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "user123");
    }

    #[test]
    fn test_validate_user_with_empty_id_returns_error() {
        let result = validate_user("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ValidationError::EmptyUserId);
    }

    #[test]
    fn test_validate_user_with_invalid_characters_returns_error() {
        let result = validate_user("user@123");
        assert!(result.is_err());
    }
}
```

Example 2: Integration test with setup
```rust
#[tokio::test]
async fn test_fetch_user_workflow() {
    // Setup
    let db = setup_test_db().await;
    let user_id = "test_user";
    insert_test_user(&db, user_id).await;

    // Execute
    let result = fetch_user(&db, user_id).await;

    // Verify
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.id, user_id);
}
```

Example 3: Property-based test
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parse_id_always_returns_valid_id(input in "[a-zA-Z0-9_]{1,50}") {
        let result = parse_id(&input);
        assert!(result.is_ok());
        let id = result.unwrap();
        assert_eq!(id.value, input);
    }
}
```

## Testing Requirements

Generate tests that cover:

### Unit Tests
- [ ] All public functions
- [ ] Edge cases and boundary conditions
- [ ] Error paths
- [ ] Typical use cases
- [ ] Performance characteristics (if applicable)

### Integration Tests
- [ ] End-to-end workflows
- [ ] Interactions with dependencies
- [ ] Error scenarios
- [ ] Realistic test data

### Property-Based Tests
- [ ] Invariants and properties
- [ ] Random inputs
- [ ] Shrinking of failing cases

## Test Principles

1. **Independence**: Tests should not depend on each other
2. **Speed**: Tests should be fast
3. **Clarity**: Test names should describe what they test
4. **Maintainability**: Tests should be easy to maintain
5. **Coverage**: Aim for high code coverage (target: 80%+)

## Integration with Project Functionality

### LSP Integration
- Use LSP to identify all testable code paths
- Analyze function signatures to understand inputs and outputs
- Identify public APIs that require testing

### Knowledge Graph Integration
- Query dependencies to understand integration points
- Identify related code that should be tested together
- Understand architectural relationships

### Vector Store Integration
- Search for similar test patterns
- Find examples of testing similar code
- Leverage historical test implementations

### Tool Integration
- Use cargo test to run tests
- Use tarpaulin or grcov for coverage analysis
- Use proptest for property-based testing
- Use mockall for mocking dependencies

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Test Quality
- [ ] Tests are independent and don't depend on each other
- [ ] Tests are deterministic and produce consistent results
- [ ] Tests are fast and don't have unnecessary delays
- [ ] Test names are descriptive and explain what is being tested

### Coverage
- [ ] All public functions are tested
- [ ] Error paths are tested
- [ ] Edge cases are covered
- [ ] Coverage meets target (80%+)

### Test Safety
- [ ] Tests don't modify production data
- [ ] Tests use appropriate fixtures and setup
- [ ] Tests clean up after themselves
- [ ] Tests don't have race conditions

## Output Format

Provide your response in the following structured format:

```markdown
## Step-by-Step Reasoning
[Your reasoning following the Chain-of-Thought steps]

## Test Analysis
[Analysis of the code and test requirements]

## Unit Tests
```rust
[Unit tests]
```

## Integration Tests
```rust
[Integration tests]
```

## Property-Based Tests
```rust
[Property-based tests]
```

## Test Fixtures and Helpers
```rust
[Test fixtures and helper functions]
```

## Coverage Report
[Documentation of test coverage and any gaps]

## Safety Verification
[Confirmation that all safety constraints are satisfied]
```

## Additional Notes
{additional_notes}
```

### Template 5: Documentation Generation

**Use Case**: Generating comprehensive documentation for Rust code.

**When to Use**: When you need to document new or existing code.

**Complexity**: Low  
**Frequency**: Medium

```
You are tasked with generating documentation for Rust code in the Self-Developing Coding Agent project.

## Code to Document
```rust
{code_to_document}
```

## Chain-of-Thought Reasoning

Before providing the documentation, please think through the documentation generation step by step:

1. **Analyze Code**: Understand the code structure, purpose, and behavior
2. **Identify Documentation Needs**: Identify what needs to be documented (APIs, modules, architecture)
3. **Plan Documentation Structure**: Plan the documentation organization and hierarchy
4. **Draft Documentation**: Write the documentation following best practices
5. **Add Examples**: Provide clear, runnable examples
6. **Verify Completeness**: Ensure all public APIs are documented
7. **Check Quality**: Verify documentation is clear, accurate, and complete

Please provide your step-by-step reasoning before the final documentation.

## Few-Shot Examples

Here are examples of well-structured Rust documentation:

Example 1: Function documentation
```rust
/// Processes a task and returns the result.
///
/// This function validates the task, executes it, and returns the result.
/// It handles errors gracefully and provides informative error messages.
///
/// # Arguments
///
/// * `task` - The task to process
///
/// # Returns
///
/// Returns a `Result` containing the processed output or an error.
///
/// # Errors
///
/// This function will return an error if:
/// - The task is invalid
/// - Processing fails
/// - A timeout occurs
///
/// # Examples
///
/// ```
/// use agent::Task;
///
/// let task = Task::new("example task");
/// let result = process_task(task)?;
/// println!("Result: {:?}", result);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Performance
///
/// This function has O(n) time complexity where n is the task size.
/// For large tasks, consider using `process_task_async` instead.
#[must_use]
pub fn process_task(task: Task) -> Result<Output, Error> {
    // Implementation
}
```

Example 2: Module documentation
```rust
//! Task orchestration and management.
//!
//! This module provides the core functionality for managing tasks,
//! including task creation, execution, and result handling.
//!
//! # Overview
//!
//! The task management system consists of three main components:
//!
//! 1. **Task Creation**: Creating and validating tasks
//! 2. **Task Execution**: Executing tasks asynchronously
//! 3. **Result Handling**: Processing and storing results
//!
//! # Examples
//!
//! ```no_run
//! use agent::{Task, TaskManager};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let manager = TaskManager::new().await?;
//! let task = Task::new("example task");
//! let result = manager.execute(task).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! Tasks are processed through a pipeline of stages:
//! 1. Validation
//! 2. Planning
//! 3. Execution
//! 4. Verification
```

Example 3: Type documentation
```rust
/// Represents a task that can be executed by the agent.
///
/// A task contains all the information needed to execute a specific
/// operation, including the task description, context, and requirements.
///
/// # Examples
///
/// ```
/// use agent::Task;
///
/// let task = Task::builder()
///     .description("Process data")
///     .context("data processing pipeline")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct Task {
    /// The task description
    pub description: String,
    /// Additional context for the task
    pub context: Option<String>,
    /// Task requirements
    pub requirements: Vec<Requirement>,
}
```

## Documentation Requirements

Generate documentation that includes:

### Code Documentation
- [ ] All public items have documentation comments
- [ ] Parameters are documented
- [ ] Return values are documented
- [ ] Errors are documented
- [ ] Examples are provided
- [ ] Performance characteristics are noted

### Module Documentation
- [ ] Module-level documentation
- [ ] Overview of module purpose
- [ ] Key types and functions
- [ ] Usage examples

### Architecture Documentation
- [ ] Design decisions
- [ ] Trade-offs considered
- [ ] Future improvements

## Documentation Principles

1. **Clarity**: Documentation should be clear and concise
2. **Completeness**: Document all public APIs
3. **Examples**: Provide usage examples
4. **Accuracy**: Keep documentation in sync with code
5. **Context**: Explain why, not just what

## Integration with Project Functionality

### LSP Integration
- Use LSP to extract API information
- Analyze function signatures and types
- Identify public APIs that need documentation

### Knowledge Graph Integration
- Query dependencies to document relationships
- Identify related modules and their interactions
- Understand architectural context

### Vector Store Integration
- Search for similar documentation patterns
- Find examples of documenting similar code
- Leverage historical documentation

### Tool Integration
- Use cargo doc to build documentation
- Use rustdoc for documentation generation
- Verify documentation builds without warnings

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Documentation Quality
- [ ] All public APIs are documented
- [ ] Documentation is clear and understandable
- [ ] Examples are accurate and runnable
- [ ] Error conditions are documented

### Documentation Completeness
- [ ] Parameters are documented
- [ ] Return values are documented
- [ ] Errors are documented
- [ ] Performance characteristics are noted

### Documentation Accuracy
- [ ] Documentation matches code behavior
- [ ] Examples compile and run correctly
- [ ] No broken links or references

## Output Format

Provide your response in the following structured format:

```markdown
## Step-by-Step Reasoning
[Your reasoning following the Chain-of-Thought steps]

## Documentation Analysis
[Analysis of the code and documentation requirements]

## Code Documentation
```rust
[Inline documentation comments]
```

## Module Documentation
```rust
[Module-level documentation]
```

## Usage Examples
```rust
[Usage examples]
```

## Architecture Notes
[Architecture documentation if applicable]

## Documentation Checklist
[Verification that all documentation requirements are met]

## Safety Verification
[Confirmation that all safety constraints are satisfied]
```

## Additional Notes
{additional_notes}
```

### Template 6: Performance Analysis

**Use Case**: Analyzing the performance of Rust code.

**When to Use**: When you need to analyze performance, identify bottlenecks, or optimize code.

**Complexity**: High  
**Frequency**: Low

```
You are tasked with analyzing the performance of Rust code in the Self-Developing Coding Agent project.

## Code to Analyze
```rust
{code_to_analyze}
```

## Chain-of-Thought Reasoning

Before providing the performance analysis, please think through the analysis step by step:

1. **Understand Code**: Understand the code structure, purpose, and expected workload
2. **Analyze Complexity**: Analyze time and space complexity
3. **Identify Hot Paths**: Identify code paths that execute frequently
4. **Analyze Memory**: Analyze allocation patterns and memory usage
5. **Evaluate Cache Efficiency**: Evaluate data layout and access patterns
6. **Assess Concurrency**: Assess async correctness and parallelism opportunities
7. **Identify Bottlenecks**: Identify specific performance bottlenecks
8. **Recommend Optimizations**: Recommend specific optimizations with justification

Please provide your step-by-step reasoning before the final analysis.

## Performance Analysis Requirements

Analyze the code for:

### Algorithmic Complexity
- [ ] Time complexity analysis (Big O notation)
- [ ] Space complexity analysis
- [ ] Bottleneck identification
- [ ] Optimization opportunities

### Memory Usage
- [ ] Allocation patterns
- [ ] Memory leaks potential
- [ ] Stack vs heap usage
- [ ] Buffer sizes and capacity planning

### Cache Efficiency
- [ ] Data layout
- [ ] Access patterns
- [ ] Cache misses potential
- [ ] Optimization opportunities

### Concurrency
- [ ] Async correctness
- [ ] Lock contention
- [ ] Parallelism opportunities
- [ ] Deadlock potential

## Performance Principles

1. **Measure First**: Profile before optimizing
2. **Focus on Hot Paths**: Optimize code that runs frequently
3. **Consider Trade-offs**: Balance performance vs. maintainability
4. **Document Decisions**: Document performance-related decisions

## Integration with Project Functionality

### LSP Integration
- Use LSP to identify hot paths and call sites
- Analyze function signatures for performance implications
- Identify frequently called functions

### Knowledge Graph Integration
- Query dependencies to understand performance impact
- Identify related code that may be affected
- Understand architectural relationships

### Vector Store Integration
- Search for similar performance patterns
- Find examples of optimized similar code
- Leverage historical performance decisions

### Tool Integration
- Use criterion for benchmarking
- Use flamegraph for profiling
- Use perf for CPU profiling
- Use valgrind for memory profiling

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Analysis Quality
- [ ] Analysis is based on actual measurements or sound reasoning
- [ ] Complexity analysis is correct
- [ ] Bottlenecks are accurately identified
- [ ] Recommendations are justified

### Optimization Safety
- [ ] Optimizations don't break correctness
- [ ] Optimizations don't introduce security vulnerabilities
- [ ] Optimizations don't degrade maintainability excessively
- [ ] Optimizations are tested and validated

### Measurement Validity
- [ ] Benchmarks are representative of real workloads
- [ ] Measurements are accurate and reproducible
- [ ] Baseline comparisons are fair
- [ ] Performance targets are realistic

## Output Format

Provide your response in the following structured format:

```json
{
  "analysis_summary": "Brief summary of the performance analysis",
  "complexity_analysis": {
    "time_complexity": "O(n) or similar",
    "space_complexity": "O(n) or similar",
    "explanation": "Explanation of complexity analysis"
  },
  "bottlenecks": [
    {
      "location": "file:line",
      "type": "algorithmic|memory|cache|concurrency",
      "description": "Description of the bottleneck",
      "severity": "critical|high|medium|low",
      "impact": "Estimated performance impact"
    }
  ],
  "optimization_recommendations": [
    {
      "priority": "high|medium|low",
      "description": "Description of the optimization",
      "location": "file:line (if applicable)",
      "expected_improvement": "Expected performance improvement",
      "trade_offs": "Any trade-offs or risks",
      "implementation": "Brief implementation guidance"
    }
  ],
  "benchmark_suggestions": [
    "Suggestion 1: ...",
    "Suggestion 2: ..."
  ],
  "monitoring_recommendations": [
    "Recommendation 1: ...",
    "Recommendation 2: ..."
  ],
  "next_steps": [
    "Step 1: ...",
    "Step 2: ..."
  ]
}
```

## Additional Notes
{additional_notes}
```

### Template 7: Self-Improvement Analysis

**Use Case**: Analyzing the Self-Developing Coding Agent for potential improvements.

**When to Use**: When you need to identify and plan improvements to the agent system.

**Complexity**: High  
**Frequency**: Low

```
You are tasked with analyzing the Self-Developing Coding Agent for potential improvements.

## Current State
{current_state}

## Chain-of-Thought Reasoning

Before providing the improvement analysis, please think through the analysis step by step:

1. **Analyze Current State**: Understand the current state of the agent system
2. **Identify Improvement Areas**: Identify areas where improvements can be made
3. **Evaluate Opportunities**: Evaluate each improvement opportunity
4. **Prioritize Improvements**: Prioritize improvements based on impact and effort
5. **Plan Implementation**: Plan the implementation of top improvements
6. **Define Success Criteria**: Define measurable success criteria for each improvement
7. **Assess Risks**: Assess risks and mitigation strategies

Please provide your step-by-step reasoning before the final analysis.

## Analysis Areas

Analyze the agent for improvements in:

### Code Quality
- [ ] Code complexity
- [ ] Code duplication
- [ ] Code organization
- [ ] Code maintainability

### Performance
- [ ] Response time
- [ ] Resource usage
- [ ] Scalability
- [ ] Bottlenecks

### Capabilities
- [ ] Missing features
- [ ] Feature gaps
- [ ] Integration opportunities
- [ ] Automation opportunities

### Safety
- [ ] Error handling
- [ ] Input validation
- [ ] Resource management
- [ ] Security considerations

## Improvement Principles

1. **Incremental**: Make small, incremental improvements
2. **Tested**: Test all improvements thoroughly
3. **Measured**: Measure impact of improvements
4. **Documented**: Document all improvements
5. **Reversible**: Ensure improvements can be rolled back

## Integration with Project Functionality

### Telemetry Integration
- Analyze telemetry data to identify improvement opportunities
- Use metrics to prioritize improvements
- Measure impact of implemented improvements

### Knowledge Graph Integration
- Query system relationships to understand dependencies
- Identify areas where improvements can have cascading benefits
- Understand architectural context

### Vector Store Integration
- Search for similar improvement patterns
- Find examples of similar improvements in other systems
- Leverage historical improvement decisions

### Tool Integration
- Use profiling tools to identify performance issues
- Use static analysis tools to identify code quality issues
- Use testing tools to identify test coverage gaps

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Improvement Safety
- [ ] Improvements don't break existing functionality
- [ ] Improvements don't introduce security vulnerabilities
- [ ] Improvements don't degrade performance
- [ ] Improvements are tested and validated

### Implementation Safety
- [ ] Rollback plan exists for each improvement
- [ ] Improvements are incremental and reversible
- [ ] Impact is measured and monitored
- [ ] Documentation is updated

### Risk Management
- [ ] Risks are identified and assessed
- [ ] Mitigation strategies are defined
- [ ] Success criteria are measurable
- [ ] Improvements are prioritized appropriately

## Output Format

Provide your response in the following structured format:

```json
{
  "current_state_analysis": {
    "code_quality": "Assessment of current code quality",
    "performance": "Assessment of current performance",
    "capabilities": "Assessment of current capabilities",
    "safety": "Assessment of current safety"
  },
  "improvement_opportunities": [
    {
      "area": "code_quality|performance|capabilities|safety",
      "description": "Description of the improvement opportunity",
      "current_state": "Current state",
      "desired_state": "Desired state",
      "impact": "high|medium|low",
      "effort": "high|medium|low",
      "priority": "high|medium|low",
      "dependencies": ["List of dependencies"]
    }
  ],
  "prioritized_improvements": [
    {
      "rank": 1,
      "improvement": "Description of the improvement",
      "rationale": "Why this improvement is prioritized",
      "expected_benefit": "Expected benefit"
    }
  ],
  "implementation_plan": [
    {
      "improvement": "Description of the improvement",
      "steps": [
        "Step 1: ...",
        "Step 2: ..."
      ],
      "success_criteria": [
        "Criterion 1: ...",
        "Criterion 2: ..."
      ],
      "rollback_plan": "Rollback strategy",
      "estimated_effort": "Estimated effort",
      "risks": ["List of risks"]
    }
  ],
  "next_steps": [
    "Step 1: ...",
    "Step 2: ..."
  ]
}
```

## Additional Notes
{additional_notes}
```

---

### Template 8: Debugging Assistance

**Use Case**: Debugging issues and bugs in the codebase.

**When to Use**: When you need to investigate bug reports, analyze error logs, or identify root causes.

**Complexity**: High  
**Frequency**: High

```
You are tasked with debugging an issue in the Self-Developing Coding Agent project.

## Issue Description
{issue_description}

## Error Logs
{error_logs}

## Chain-of-Thought Reasoning

Before providing the debugging analysis, please think through the debugging process step by step:

1. **Understand the Issue**: Understand what the issue is and what the expected behavior should be
2. **Analyze Error Logs**: Analyze error logs and stack traces to identify the failure point
3. **Examine Code**: Examine the relevant code to understand the implementation
4. **Identify Root Cause**: Identify the root cause of the issue
5. **Propose Solutions**: Propose one or more solutions to fix the issue
6. **Validate Solutions**: Validate that the proposed solutions address the root cause
7. **Plan Testing**: Plan how to test the fix

Please provide your step-by-step reasoning before the final analysis.

## Integration with Project Functionality

### LSP Integration
- Use LSP to understand code structure and relationships
- Analyze function signatures and call sites
- Identify related code that may be affected

### Knowledge Graph Integration
- Query dependencies to understand the impact scope
- Trace error propagation through the system
- Identify related components that may be involved

### Vector Store Integration
- Search for similar bugs and their fixes
- Find historical debugging sessions
- Leverage past solutions

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Fix Safety
- [ ] The fix doesn't introduce new bugs
- [ ] The fix doesn't break existing functionality
- [ ] The fix doesn't introduce security vulnerabilities
- [ ] The fix is tested and validated

### Root Cause Verification
- [ ] The root cause is accurately identified
- [ ] The fix addresses the root cause, not just symptoms
- [ ] The fix is minimal and focused
- [ ] The fix is maintainable

## Output Format

Provide your response in the following structured format:

```json
{
  "issue_summary": "Brief summary of the issue",
  "root_cause": {
    "description": "Description of the root cause",
    "location": "file:line",
    "evidence": ["Evidence supporting the root cause analysis"]
  },
  "proposed_solutions": [
    {
      "description": "Description of the solution",
      "code": "Code snippet showing the fix",
      "pros": ["List of advantages"],
      "cons": ["List of disadvantages"],
      "risk": "low|medium|high"
    }
  ],
  "recommended_solution": {
    "description": "Description of the recommended solution",
    "justification": "Why this solution is recommended",
    "implementation": "Implementation guidance"
  },
  "testing_plan": [
    "Test 1: ...",
    "Test 2: ..."
  ],
  "prevention_measures": [
    "Measure 1: ...",
    "Measure 2: ..."
  ],
  "next_steps": [
    "Step 1: ...",
    "Step 2: ..."
  ]
}
```

## Additional Notes
{additional_notes}
```

---

### Template 9: Architecture Design

**Use Case**: Designing system architecture and making architectural decisions.

**When to Use**: When you need to design new systems, evaluate architectural alternatives, or document architectural decisions.

**Complexity**: High  
**Frequency**: Medium

```
You are tasked with designing architecture for the Self-Developing Coding Agent project.

## Design Requirements
{design_requirements}

## Context
{context}

## Chain-of-Thought Reasoning

Before providing the architecture design, please think through the design process step by step:

1. **Understand Requirements**: Understand the functional and non-functional requirements
2. **Identify Constraints**: Identify technical, business, and operational constraints
3. **Explore Alternatives**: Explore multiple architectural alternatives using Tree-of-Thoughts
4. **Evaluate Alternatives**: Evaluate each alternative against criteria
5. **Select Design**: Select the best design and justify your choice
6. **Document Decisions**: Document architectural decisions with rationale
7. **Plan Implementation**: Plan the implementation approach

Please provide your step-by-step reasoning before the final design.

## Tree-of-Thoughts for Architectural Alternatives

Explore multiple architectural approaches:

```
Decision Point 1: System Architecture
├── Approach A: Monolithic
│   ├── Pros: Simpler deployment, easier debugging
│   └── Cons: Harder to scale, single point of failure
├── Approach B: Microservices
│   ├── Pros: Better scalability, independent deployment
│   └── Cons: More complex, distributed system challenges
└── Approach C: Modular Monolith
    ├── Pros: Balance of simplicity and modularity
    └── Cons: Still single deployment unit

Decision Point 2: Data Storage
├── Option A: Relational Database
│   └── ACID transactions, structured queries
└── Option B: NoSQL Database
    └── Flexible schema, horizontal scaling
```

After evaluating all approaches, recommend the best one with justification.

## Integration with Project Functionality

### Knowledge Graph Integration
- Query existing architecture to understand current design
- Identify dependencies and relationships
- Understand architectural patterns in use

### Vector Store Integration
- Search for similar architectural patterns
- Find examples of similar systems
- Leverage historical architectural decisions

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Design Safety
- [ ] The design is secure by default
- [ ] The design is fault-tolerant
- [ ] The design is scalable
- [ ] The design is maintainable

### Decision Quality
- [ ] Decisions are well-justified
- [ ] Trade-offs are explicitly documented
- [ ] Alternatives were considered
- [ ] Risks are identified and mitigated

## Output Format

Provide your response in the following structured format:

```markdown
## Step-by-Step Reasoning
[Your reasoning following the Chain-of-Thought steps]

## Tree-of-Thoughts Analysis
[Your analysis of multiple architectural alternatives]

## Architecture Overview
[High-level overview of the architecture]

## Architectural Decisions

### Decision 1: [Decision Title]
- **Context**: [Context for the decision]
- **Alternatives Considered**: [List of alternatives]
- **Decision**: [The decision made]
- **Rationale**: [Justification for the decision]
- **Trade-offs**: [Trade-offs accepted]
- **Consequences**: [Consequences of the decision]

### Decision 2: [Decision Title]
[Same structure as Decision 1]

## Component Design
[Description of key components and their interactions]

## Data Flow
[Description of data flow through the system]

## Security Considerations
[Security considerations and mitigations]

## Scalability Considerations
[Scalability considerations and strategies]

## Implementation Plan
[Plan for implementing the architecture]

## Risks and Mitigations
[List of risks and mitigation strategies]

## Next Steps
[List of next steps]
```

## Additional Notes
{additional_notes}
```

---

### Template 10: Dependency Analysis

**Use Case**: Analyzing and managing dependencies in the project.

**When to Use**: When you need to add, update, or remove dependencies, or assess security vulnerabilities.

**Complexity**: Medium  
**Frequency**: Low

```
You are tasked with analyzing dependencies for the Self-Developing Coding Agent project.

## Dependency Change
{dependency_change}

## Context
{context}

## Chain-of-Thought Reasoning

Before providing the dependency analysis, please think through the analysis step by step:

1. **Understand the Change**: Understand what dependency change is being proposed
2. **Analyze the Dependency**: Analyze the dependency itself (purpose, maintenance, license)
3. **Assess Impact**: Assess the impact of the change on the project
4. **Check Security**: Check for security vulnerabilities
5. **Check Compatibility**: Check compatibility with existing dependencies
6. **Evaluate Alternatives**: Evaluate alternative dependencies if applicable
7. **Recommend Decision**: Recommend whether to proceed with the change

Please provide your step-by-step reasoning before the final analysis.

## Integration with Project Functionality

### Knowledge Graph Integration
- Query dependency graph to understand impact scope
- Identify transitive dependencies
- Understand dependency relationships

### Vector Store Integration
- Search for similar dependency decisions
- Find historical dependency changes
- Leverage past dependency evaluations

### Tool Integration
- Use cargo-audit for security vulnerability scanning
- Use cargo-outdated for checking outdated dependencies
- Use cargo-license for license compliance checking

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Dependency Safety
- [ ] The dependency is actively maintained
- [ ] The dependency has no known security vulnerabilities
- [ ] The dependency's license is compatible
- [ ] The dependency is compatible with existing dependencies

### Change Safety
- [ ] The change doesn't break existing functionality
- [ ] The change doesn't introduce security vulnerabilities
- [ ] The change is tested and validated
- [ ] Rollback plan exists

## Output Format

Provide your response in the following structured format:

```json
{
  "dependency_analysis": {
    "name": "Dependency name",
    "version": "Version",
    "purpose": "Purpose of the dependency",
    "maintenance": "Active|Deprecated|Unknown",
    "license": "License type",
    "license_compatible": true|false
  },
  "security_assessment": {
    "vulnerabilities": [
      {
        "severity": "critical|high|medium|low",
        "description": "Description of the vulnerability",
        "cve": "CVE identifier if applicable"
      }
    ],
    "overall_risk": "critical|high|medium|low|none"
  },
  "compatibility_assessment": {
    "compatible": true|false,
    "conflicts": ["List of conflicting dependencies"],
    "breaking_changes": ["List of breaking changes"]
  },
  "impact_analysis": {
    "affected_components": ["List of affected components"],
    "migration_effort": "high|medium|low",
    "testing_required": ["List of testing requirements"]
  },
  "alternatives": [
    {
      "name": "Alternative dependency name",
      "pros": ["List of advantages"],
      "cons": ["List of disadvantages"]
    }
  ],
  "recommendation": {
    "decision": "proceed|reject|defer",
    "justification": "Justification for the recommendation",
    "conditions": ["Conditions if any"]
  },
  "next_steps": [
    "Step 1: ...",
    "Step 2: ..."
  ]
}
```

## Additional Notes
{additional_notes}
```

---

### Template 11: Security Audit

**Use Case**: Conducting security audits and vulnerability assessments.

**When to Use**: When you need to review code for security issues, assess vulnerabilities, or verify compliance.

**Complexity**: High  
**Frequency**: Medium

```
You are tasked with conducting a security audit for the Self-Developing Coding Agent project.

## Scope
{scope}

## Context
{context}

## Chain-of-Thought Reasoning

Before providing the security audit, please think through the audit process step by step:

1. **Understand the Scope**: Understand what is being audited and why
2. **Identify Threats**: Identify potential threats and attack vectors
3. **Analyze Code**: Analyze the code for security vulnerabilities
4. **Check Compliance**: Check compliance with security best practices
5. **Assess Risk**: Assess the risk level of identified issues
6. **Recommend Mitigations**: Recommend mitigations for identified issues
7. **Prioritize Actions**: Prioritize actions based on risk and impact

Please provide your step-by-step reasoning before the final audit.

## Security Checklist

### Input Validation
- [ ] All inputs are validated
- [ ] Input sanitization is performed
- [ ] Length limits are enforced
- [ ] Type checking is performed

### Output Encoding
- [ ] Output is properly encoded
- [ ] XSS prevention is in place
- [ ] SQL injection prevention is in place
- [ ] Command injection prevention is in place

### Authentication & Authorization
- [ ] Authentication is properly implemented
- [ ] Authorization checks are in place
- [ ] Session management is secure
- [ ] Password handling is secure

### Cryptography
- [ ] Cryptographic algorithms are appropriate
- [ ] Key management is secure
- [ ] Random number generation is secure
- [ ] Certificate validation is performed

### Error Handling
- [ ] Error messages don't leak sensitive information
- [ ] Error handling doesn't bypass security checks
- [ ] Logging doesn't expose sensitive data

### Resource Management
- [ ] Resource limits are enforced
- [ ] Denial of service prevention is in place
- [ ] Memory safety is maintained

## Integration with Project Functionality

### LSP Integration
- Use LSP to analyze code structure
- Identify potential security issues
- Analyze data flow

### Knowledge Graph Integration
- Query dependencies to understand attack surface
- Identify related security-sensitive code
- Understand data flow through the system

### Vector Store Integration
- Search for similar security issues
- Find historical security audits
- Leverage past security decisions

### Tool Integration
- Use cargo-audit for dependency vulnerability scanning
- Use clippy for additional security checks
- Use static analysis tools for security analysis

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Audit Quality
- [ ] All security issues are identified
- [ ] Risk assessments are accurate
- [ ] Recommendations are actionable
- [ ] Prioritization is appropriate

### Mitigation Safety
- [ ] Mitigations don't introduce new vulnerabilities
- [ ] Mitigations are tested and validated
- [ ] Mitigations are maintainable
- [ ] Mitigations don't break functionality

## Output Format

Provide your response in the following structured format:

```json
{
  "audit_summary": "Brief summary of the security audit",
  "threat_model": {
    "threats": [
      {
        "type": "threat type",
        "description": "Description of the threat",
        "likelihood": "high|medium|low",
        "impact": "high|medium|low"
      }
    ]
  },
  "findings": [
    {
      "severity": "critical|high|medium|low",
      "type": "vulnerability type",
      "location": "file:line",
      "description": "Description of the issue",
      "cve": "CVE identifier if applicable",
      "recommendation": "Recommended mitigation",
      "priority": "immediate|high|medium|low"
    }
  ],
  "compliance_assessment": {
    "owasp_top_10": "compliant|partial|non-compliant",
    "rust_security_guidelines": "compliant|partial|non-compliant",
    "project_security_policies": "compliant|partial|non-compliant"
  },
  "risk_assessment": {
    "overall_risk": "critical|high|medium|low",
    "critical_issues": 0,
    "high_issues": 0,
    "medium_issues": 0,
    "low_issues": 0
  },
  "recommendations": [
    {
      "priority": "immediate|high|medium|low",
      "action": "Description of the recommended action",
      "justification": "Why this action is recommended",
      "effort": "high|medium|low"
    }
  ],
  "next_steps": [
    "Step 1: ...",
    "Step 2: ..."
  ]
}
```

## Additional Notes
{additional_notes}
```

---

### Template 12: Code Migration

**Use Case**: Migrating code between versions, APIs, or platforms.

**When to Use**: When you need to upgrade dependencies, migrate between APIs, or refactor to new patterns.

**Complexity**: High  
**Frequency**: Low

```
You are tasked with migrating code for the Self-Developing Coding Agent project.

## Migration Details
{migration_details}

## Context
{context}

## Chain-of-Thought Reasoning

Before providing the migration plan, please think through the migration process step by step:

1. **Understand the Migration**: Understand what needs to be migrated and why
2. **Analyze Current Code**: Analyze the current code to understand dependencies
3. **Identify Changes**: Identify all changes required for the migration
4. **Assess Risk**: Assess the risk and impact of the migration
5. **Plan Migration**: Plan the migration approach and steps
6. **Plan Testing**: Plan comprehensive testing for the migration
7. **Plan Rollback**: Plan rollback strategy in case of issues

Please provide your step-by-step reasoning before the final migration plan.

## Integration with Project Functionality

### LSP Integration
- Use LSP to identify all code affected by the migration
- Analyze function signatures and call sites
- Identify breaking changes

### Knowledge Graph Integration
- Query dependencies to understand impact scope
- Identify related code that needs migration
- Understand architectural relationships

### Vector Store Integration
- Search for similar migrations
- Find examples of successful migrations
- Leverage historical migration decisions

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Migration Safety
- [ ] The migration doesn't break existing functionality
- [ ] The migration doesn't introduce security vulnerabilities
- [ ] The migration is tested and validated
- [ ] Rollback plan exists and is tested

### Testing Safety
- [ ] All affected code is tested
- [ ] Edge cases are covered
- [ ] Performance is not degraded
- [ ] Integration points are tested

## Output Format

Provide your response in the following structured format:

```json
{
  "migration_summary": "Brief summary of the migration",
  "risk_assessment": {
    "overall_risk": "high|medium|low",
    "complexity": "high|medium|low",
    "impact_scope": "Description of impact scope",
    "potential_issues": ["List of potential issues"]
  },
  "migration_plan": {
    "approach": "Description of the migration approach",
    "phases": [
      {
        "phase": 1,
        "description": "Description of the phase",
        "tasks": [
          "Task 1: ...",
          "Task 2: ..."
        ],
        "deliverables": ["List of deliverables"]
      }
    ]
  },
  "changes_required": [
    {
      "file": "file path",
      "changes": "Description of changes",
      "breaking_change": true|false,
      "migration_notes": "Notes for the migration"
    }
  ],
  "testing_plan": {
    "unit_tests": ["List of unit tests"],
    "integration_tests": ["List of integration tests"],
    "regression_tests": ["List of regression tests"],
    "performance_tests": ["List of performance tests"]
  },
  "rollback_plan": {
    "strategy": "Description of rollback strategy",
    "triggers": ["Conditions that trigger rollback"],
    "steps": [
      "Step 1: ...",
      "Step 2: ..."
    ]
  },
  "next_steps": [
    "Step 1: ...",
    "Step 2: ..."
  ]
}
```

## Additional Notes
{additional_notes}
```

---

### Template 13: Feature Implementation Planning

**Use Case**: Planning the implementation of new features.

**When to Use**: When you need to plan new features, break down complex features, or estimate effort.

**Complexity**: Medium  
**Frequency**: High

```
You are tasked with planning the implementation of a new feature for the Self-Developing Coding Agent project.

## Feature Requirements
{feature_requirements}

## Context
{context}

## Chain-of-Thought Reasoning

Before providing the implementation plan, please think through the planning process step by step:

1. **Understand Requirements**: Understand the feature requirements and goals
2. **Analyze Dependencies**: Analyze dependencies and integration points
3. **Break Down Feature**: Break down the feature into smaller, manageable tasks
4. **Identify Risks**: Identify potential risks and mitigation strategies
5. **Estimate Effort**: Estimate effort for each task
6. **Plan Testing**: Plan the testing strategy
7. **Plan Documentation**: Plan documentation requirements

Please provide your step-by-step reasoning before the final plan.

## Tree-of-Thoughts for Implementation Approaches

Explore multiple implementation approaches:

```
Decision Point 1: Implementation Strategy
├── Approach A: Incremental
│   ├── Pros: Lower risk, faster feedback
│   └── Cons: May take longer overall
├── Approach B: Big Bang
│   ├── Pros: Faster completion
│   └── Cons: Higher risk, harder to debug
└── Approach C: Feature Flags
    ├── Pros: Can deploy incrementally
    └── Cons: Additional complexity

Decision Point 2: Architecture Changes
├── Option A: Minimal Changes
│   └── Least disruptive, but may not be optimal
└── Option B: Refactor First
    └── Better long-term, but more upfront work
```

After evaluating all approaches, recommend the best one with justification.

## Integration with Project Functionality

### Knowledge Graph Integration
- Query dependencies to understand integration points
- Identify related components that may be affected
- Understand architectural context

### Vector Store Integration
- Search for similar features
- Find examples of similar implementations
- Leverage past implementation decisions

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Plan Quality
- [ ] The plan is comprehensive and complete
- [ ] The plan is realistic and achievable
- [ ] The plan considers all dependencies
- [ ] The plan includes adequate testing

### Risk Management
- [ ] Risks are identified and assessed
- [ ] Mitigation strategies are defined
- [ ] Contingency plans exist
- [ ] Effort estimates are realistic

## Output Format

Provide your response in the following structured format:

```json
{
  "feature_summary": "Brief summary of the feature",
  "requirements_analysis": {
    "functional_requirements": ["List of functional requirements"],
    "non_functional_requirements": ["List of non-functional requirements"],
    "acceptance_criteria": ["List of acceptance criteria"]
  },
  "implementation_approach": {
    "strategy": "Description of the chosen strategy",
    "justification": "Justification for the strategy",
    "alternatives_considered": ["List of alternatives considered"]
  },
  "task_breakdown": [
    {
      "task": "Description of the task",
      "subtasks": [
        "Subtask 1: ...",
        "Subtask 2: ..."
      ],
      "dependencies": ["List of dependencies"],
      "estimated_effort": "Estimated effort",
      "priority": "high|medium|low"
    }
  ],
  "risk_assessment": [
    {
      "risk": "Description of the risk",
      "likelihood": "high|medium|low",
      "impact": "high|medium|low",
      "mitigation": "Mitigation strategy"
    }
  ],
  "testing_plan": {
    "unit_tests": ["List of unit tests"],
    "integration_tests": ["List of integration tests"],
    "e2e_tests": ["List of end-to-end tests"],
    "performance_tests": ["List of performance tests"]
  },
  "documentation_requirements": [
    "Documentation requirement 1: ...",
    "Documentation requirement 2: ..."
  ],
  "next_steps": [
    "Step 1: ...",
    "Step 2: ..."
  ]
}
```

## Additional Notes
{additional_notes}
```

---

### Template 14: Error Analysis & Resolution

**Use Case**: Analyzing error patterns and designing error handling strategies.

**When to Use**: When you need to analyze error patterns, improve error messages, or design error handling strategies.

**Complexity**: Medium  
**Frequency**: Medium

```
You are tasked with analyzing errors for the Self-Developing Coding Agent project.

## Error Context
{error_context}

## Chain-of-Thought Reasoning

Before providing the error analysis, please think through the analysis process step by step:

1. **Understand the Error**: Understand the error and its context
2. **Classify the Error**: Classify the error type and category
3. **Analyze Root Cause**: Analyze the root cause of the error
4. **Identify Patterns**: Identify patterns in similar errors
5. **Design Error Handling**: Design appropriate error handling strategies
6. **Improve Error Messages**: Improve error messages for clarity
7. **Plan Prevention**: Plan how to prevent similar errors in the future

Please provide your step-by-step reasoning before the final analysis.

## Error Classification Framework

### Error Types
- **Validation Errors**: Input validation failures
- **Resource Errors**: Resource exhaustion or unavailability
- **Network Errors**: Network-related failures
- **Logic Errors**: Logic or algorithm errors
- **Concurrency Errors**: Race conditions, deadlocks
- **Configuration Errors**: Configuration issues

### Error Severity
- **Critical**: System failure, data loss
- **High**: Major functionality broken
- **Medium**: Partial functionality affected
- **Low**: Minor issues, workarounds available

## Integration with Project Functionality

### LSP Integration
- Use LSP to analyze error propagation
- Identify error handling patterns
- Analyze error types and their usage

### Knowledge Graph Integration
- Query dependencies to understand error flow
- Identify related error handling code
- Understand error propagation paths

### Vector Store Integration
- Search for similar error patterns
- Find historical error resolutions
- Leverage past error handling decisions

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Error Handling Safety
- [ ] Error handling doesn't mask critical errors
- [ ] Error handling doesn't introduce new errors
- [ ] Error messages are informative but not exposing
- [ ] Error recovery is safe and tested

### Resolution Safety
- [ ] The resolution doesn't break existing functionality
- [ ] The resolution doesn't introduce security vulnerabilities
- [ ] The resolution is tested and validated
- [ ] The resolution is maintainable

## Output Format

Provide your response in the following structured format:

```json
{
  "error_analysis": {
    "error_type": "Type of the error",
    "severity": "critical|high|medium|low",
    "frequency": "Description of error frequency",
    "impact": "Description of error impact"
  },
  "root_cause": {
    "description": "Description of the root cause",
    "location": "file:line",
    "contributing_factors": ["List of contributing factors"]
  },
  "pattern_analysis": {
    "similar_errors": ["List of similar errors"],
    "common_causes": ["List of common causes"],
    "trends": "Description of any trends"
  },
  "error_handling_recommendations": [
    {
      "location": "file:line",
      "current_handling": "Description of current handling",
      "recommended_handling": "Description of recommended handling",
      "justification": "Justification for the recommendation"
    }
  ],
  "error_message_improvements": [
    {
      "current_message": "Current error message",
      "improved_message": "Improved error message",
      "justification": "Justification for the improvement"
    }
  ],
  "prevention_measures": [
    {
      "measure": "Description of the prevention measure",
      "implementation": "Implementation guidance",
      "effectiveness": "Expected effectiveness"
    }
  ],
  "monitoring_recommendations": [
    "Recommendation 1: ...",
    "Recommendation 2: ..."
  ],
  "next_steps": [
    "Step 1: ...",
    "Step 2: ..."
  ]
}
```

## Additional Notes
{additional_notes}
```

---

### Template 15: Context Gathering

**Use Case**: Gathering relevant context for a task.

**When to Use**: When you need to understand codebase structure, gather relevant code, or identify dependencies.

**Complexity**: Low  
**Frequency**: High

```
You are tasked with gathering context for a task in the Self-Developing Coding Agent project.

## Task Description
{task_description}

## Chain-of-Thought Reasoning

Before providing the context, please think through the context gathering process step by step:

1. **Understand the Task**: Understand what the task is and what context is needed
2. **Identify Relevant Code**: Identify code that is relevant to the task
3. **Analyze Dependencies**: Analyze dependencies and relationships
4. **Prioritize Context**: Prioritize context based on relevance and importance
5. **Organize Context**: Organize the context in a clear and structured way
6. **Validate Completeness**: Validate that the context is complete and sufficient

Please provide your step-by-step reasoning before the final context.

## Context Priority

Prioritize context from most to least important:

1. **Direct Task Requirements**: Requirements and constraints for the task
2. **Directly Related Code**: Code directly related to the task
3. **Dependencies and Relationships**: Dependencies and relationships from the knowledge graph
4. **Similar Code Patterns**: Similar code patterns from the vector store
5. **Project Conventions**: Project-wide conventions and standards
6. **Historical Context**: Historical context (summarized)

## Integration with Project Functionality

### LSP Integration
- Use LSP to understand code structure
- Query symbol definitions and references
- Analyze code completion suggestions
- Understand type information

### Knowledge Graph Integration
- Query dependencies to understand relationships
- Identify related code and components
- Understand architectural context

### Vector Store Integration
- Search for similar code patterns
- Find relevant examples from the codebase
- Leverage historical implementations

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Context Quality
- [ ] The context is relevant to the task
- [ ] The context is accurate and up-to-date
- [ ] The context is complete and sufficient
- [ ] The context is well-organized

### Context Safety
- [ ] The context doesn't expose sensitive information
- [ ] The context is appropriate for the task
- [ ] The context respects access controls

## Output Format

Provide your response in the following structured format:

```json
{
  "task_summary": "Brief summary of the task",
  "context_summary": "Brief summary of the gathered context",
  "relevant_code": [
    {
      "file": "file path",
      "description": "Description of the code",
      "relevance": "high|medium|low",
      "excerpt": "Brief excerpt if applicable"
    }
  ],
  "dependencies": [
    {
      "name": "Dependency name",
      "type": "internal|external",
      "relationship": "Description of the relationship"
    }
  ],
  "similar_patterns": [
    {
      "pattern": "Description of the pattern",
      "location": "file:line",
      "similarity": "high|medium|low"
    }
  ],
  "project_conventions": [
    "Convention 1: ...",
    "Convention 2: ..."
  ],
  "additional_context": {
    "architecture_notes": "Architecture notes if applicable",
    "historical_context": "Historical context if applicable",
    "known_issues": ["List of known issues"]
  },
  "context_completeness": {
    "sufficient": true|false,
    "missing_information": ["List of missing information if any"]
  }
}
```

## Additional Notes
{additional_notes}
```

---

### Template 16: Impact Analysis

**Use Case**: Analyzing the impact of proposed changes.

**When to Use**: When you need to analyze the impact of proposed changes, assess risk, or plan testing scope.

**Complexity**: Medium  
**Frequency**: High

```
You are tasked with analyzing the impact of proposed changes for the Self-Developing Coding Agent project.

## Proposed Changes
{proposed_changes}

## Context
{context}

## Chain-of-Thought Reasoning

Before providing the impact analysis, please think through the analysis process step by step:

1. **Understand the Changes**: Understand what changes are being proposed
2. **Identify Affected Components**: Identify all components affected by the changes
3. **Analyze Dependencies**: Analyze dependencies and relationships
4. **Assess Risk**: Assess the risk and impact of the changes
5. **Identify Breaking Changes**: Identify any breaking changes
6. **Plan Testing**: Plan the testing scope based on impact
7. **Plan Rollback**: Plan rollback strategy if needed

Please provide your step-by-step reasoning before the final analysis.

## Integration with Project Functionality

### LSP Integration
- Use LSP to identify all references to changed code
- Analyze call graphs to understand impact
- Identify affected APIs and their consumers

### Knowledge Graph Integration
- Query dependencies to understand impact scope
- Identify related components that may be affected
- Understand architectural relationships

### Vector Store Integration
- Search for similar changes and their impact
- Find historical impact analyses
- Leverage past impact assessments

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Analysis Quality
- [ ] The analysis is comprehensive and complete
- [ ] All affected components are identified
- [ ] Risk assessment is accurate
- [ ] Breaking changes are identified

### Risk Management
- [ ] Risks are identified and assessed
- [ ] Mitigation strategies are defined
- [ ] Testing scope is appropriate
- [ ] Rollback plan exists

## Output Format

Provide your response in the following structured format:

```json
{
  "change_summary": "Brief summary of the proposed changes",
  "impact_assessment": {
    "overall_impact": "critical|high|medium|low",
    "affected_components": [
      {
        "component": "Component name",
        "impact_level": "critical|high|medium|low",
        "impact_description": "Description of the impact"
      }
    ],
    "affected_apis": [
      {
        "api": "API name",
        "breaking_change": true|false,
        "migration_required": true|false
      }
    ]
  },
  "dependency_impact": [
    {
      "dependency": "Dependency name",
      "impact": "Description of the impact",
      "action_required": "Description of required action"
    }
  ],
  "risk_assessment": [
    {
      "risk": "Description of the risk",
      "likelihood": "high|medium|low",
      "impact": "high|medium|low",
      "mitigation": "Mitigation strategy"
    }
  ],
  "testing_scope": {
    "unit_tests": ["List of unit tests required"],
    "integration_tests": ["List of integration tests required"],
    "regression_tests": ["List of regression tests required"],
    "e2e_tests": ["List of end-to-end tests required"]
  },
  "rollback_plan": {
    "feasible": true|false,
    "strategy": "Description of rollback strategy",
    "complexity": "high|medium|low"
  },
  "recommendations": [
    {
      "priority": "high|medium|low",
      "recommendation": "Description of the recommendation",
      "justification": "Justification for the recommendation"
    }
  ],
  "next_steps": [
    "Step 1: ...",
    "Step 2: ..."
  ]
}
```

## Additional Notes
{additional_notes}
```

---

### Template 17: Pattern Discovery

**Use Case**: Discovering patterns in code for refactoring, documentation, and understanding.

**When to Use**: When you need to identify code patterns, find anti-patterns, or discover refactoring opportunities.

**Complexity**: Medium  
**Frequency**: Low

```
You are tasked with discovering patterns in the Self-Developing Coding Agent project.

## Scope
{scope}

## Context
{context}

## Chain-of-Thought Reasoning

Before providing the pattern discovery, please think through the discovery process step by step:

1. **Understand the Scope**: Understand what code or area to analyze
2. **Analyze Code Structure**: Analyze the code structure and organization
3. **Identify Patterns**: Identify recurring patterns and structures
4. **Classify Patterns**: Classify patterns as design patterns, anti-patterns, or idioms
5. **Assess Quality**: Assess the quality and appropriateness of patterns
6. **Recommend Improvements**: Recommend improvements for anti-patterns
7. **Document Patterns**: Document discovered patterns for future reference

Please provide your step-by-step reasoning before the final discovery.

## Pattern Classification

### Design Patterns
- **Creational**: Factory, Builder, Singleton, etc.
- **Structural**: Adapter, Decorator, Facade, etc.
- **Behavioral**: Strategy, Observer, Command, etc.

### Anti-Patterns
- **Code Smells**: Long methods, large classes, duplicate code
- **Design Smells**: Feature envy, inappropriate intimacy, divergent change
- **Architecture Smells**: Cyclic dependencies, god classes, shotgun surgery

### Idioms
- **Rust Idioms**: Result handling, Option handling, iterator patterns
- **Project Idioms**: Project-specific patterns and conventions

## Integration with Project Functionality

### LSP Integration
- Use LSP to analyze code structure
- Identify symbol usage patterns
- Analyze call graphs and data flow

### Knowledge Graph Integration
- Query dependencies to understand pattern relationships
- Identify pattern usage across the codebase
- Understand architectural patterns

### Vector Store Integration
- Search for similar patterns
- Find examples of pattern implementations
- Leverage historical pattern discoveries

## Safety Guardrails

Before providing your final output, verify that all safety constraints are satisfied:

### Discovery Quality
- [ ] Patterns are accurately identified
- [ ] Pattern classification is correct
- [ ] Quality assessment is objective
- [ ] Recommendations are actionable

### Recommendation Safety
- [ ] Recommendations don't break existing functionality
- [ ] Recommendations are tested and validated
- [ ] Recommendations are maintainable
- [ ] Recommendations align with project goals

## Output Format

Provide your response in the following structured format:

```json
{
  "discovery_summary": "Brief summary of the pattern discovery",
  "design_patterns": [
    {
      "pattern": "Pattern name",
      "type": "creational|structural|behavioral",
      "locations": ["file:line"],
      "description": "Description of the pattern usage",
      "quality": "good|fair|poor",
      "notes": "Additional notes"
    }
  ],
  "anti_patterns": [
    {
      "pattern": "Anti-pattern name",
      "type": "code_smell|design_smell|architecture_smell",
      "locations": ["file:line"],
      "description": "Description of the anti-pattern",
      "severity": "critical|high|medium|low",
      "recommendation": "Recommended improvement",
      "refactoring_opportunity": true|false
    }
  ],
  "idioms": [
    {
      "idiom": "Idiom name",
      "type": "rust|project",
      "locations": ["file:line"],
      "description": "Description of the idiom usage"
    }
  ],
  "refactoring_opportunities": [
    {
      "opportunity": "Description of the refactoring opportunity",
      "location": "file:line",
      "current_pattern": "Current pattern",
      "suggested_pattern": "Suggested pattern",
      "benefit": "Expected benefit",
      "effort": "high|medium|low"
    }
  ],
  "recommendations": [
    {
      "priority": "high|medium|low",
      "recommendation": "Description of the recommendation",
      "justification": "Justification for the recommendation"
    }
  ],
  "next_steps": [
    "Step 1: ...",
    "Step 2: ..."
  ]
}
```

## Additional Notes
{additional_notes}
```

---

## Code Generation Guidelines

### Before Generating Code

1. **Understand the Context**
   - Read the existing codebase
   - Understand the architecture
   - Identify dependencies
   - Consider integration points

2. **Analyze Requirements**
   - Clarify ambiguous requirements
   - Identify edge cases
   - Consider error scenarios
   - Plan for testing

3. **Design the Solution**
   - Choose appropriate data structures
   - Select efficient algorithms
   - Plan the API surface
   - Consider performance implications

### While Generating Code

1. **Follow Rust Conventions**
   - Use snake_case for variables and functions
   - Use PascalCase for types and traits
   - Use SCREAMING_SNAKE_CASE for constants
   - Use `#[must_use]` for important return values

2. **Prioritize Safety**
   - Use `Result<T, E>` for fallible operations
   - Avoid `unsafe` unless absolutely necessary
   - Validate all inputs
   - Handle all errors appropriately

3. **Write Clear Code**
   - Use descriptive names
   - Keep functions small and focused
   - Limit nesting depth
   - Add comments for complex logic

4. **Consider Performance**
   - Choose appropriate algorithms
   - Minimize allocations
   - Consider cache efficiency
   - Use async/await for I/O

### After Generating Code

1. **Add Documentation**
   - Document all public APIs
   - Include usage examples
   - Document errors
   - Note performance characteristics

2. **Write Tests**
   - Write unit tests for all functions
   - Test edge cases
   - Test error paths
   - Add integration tests if needed

3. **Review Code**
   - Check for safety issues
   - Check for performance issues
   - Check for quality issues
   - Ensure standards compliance

### Code Generation Checklist

- [ ] Code follows Rust conventions
- [ ] All public APIs are documented
- [ ] Error handling is comprehensive
- [ ] Tests are comprehensive
- [ ] Performance is considered
- [ ] Safety is prioritized
- [ ] Code is clear and maintainable
- [ ] Dependencies are appropriate
- [ ] Async/await is used correctly
- [ ] Resource management is correct

---

## Code Modification Guidelines

### Before Modifying Code

1. **Understand the Existing Code**
   - Read the code thoroughly
   - Understand the purpose
   - Identify dependencies
   - Consider impact on other code

2. **Analyze the Change**
   - Understand the requirements
   - Identify affected areas
   - Consider breaking changes
   - Plan migration path

3. **Plan the Modification**
   - Design the approach
   - Identify test updates needed
   - Plan documentation updates
   - Consider rollback strategy

### While Modifying Code

1. **Preserve Behavior**
   - Maintain existing behavior
   - Preserve error handling
   - Keep API compatibility
   - Maintain performance

2. **Improve Quality**
   - Simplify complex logic
   - Improve clarity
   - Reduce duplication
   - Enhance documentation

3. **Update Tests**
   - Update existing tests
   - Add new tests for changes
   - Ensure all tests pass
   - Add integration tests if needed

### After Modifying Code

1. **Verify Changes**
   - Run all tests
   - Check for regressions
   - Verify performance
   - Validate documentation

2. **Update Documentation**
   - Update code comments
   - Update API documentation
   - Update architecture docs
   - Update changelog

3. **Review Changes**
   - Self-review the changes
   - Check for safety issues
   - Check for quality issues
   - Ensure standards compliance

### Code Modification Checklist

- [ ] Existing behavior is preserved
- [ ] Changes are well-tested
- [ ] Documentation is updated
- [ ] No regressions introduced
- [ ] Performance is maintained or improved
- [ ] Error handling is preserved
- [ ] API compatibility is maintained
- [ ] Code quality is improved
- [ ] Tests are comprehensive
- [ ] Changes are reviewed

---

## Safety Constraints and Guardrails

### Protected Resources

The following resources are protected and cannot be modified without explicit approval:

1. **Core Orchestrator Logic**
   - Task lifecycle management
   - Module coordination
   - State machine transitions

2. **Safety Validation Rules**
   - Input validation logic
   - Resource limit enforcement
   - Security checks

3. **Configuration Schema**
   - Configuration structure
   - Validation rules
   - Default values

4. **Authentication Mechanisms**
   - API key handling
   - Secret management
   - Access control

### Safety Checks

Before making any code changes, perform these safety checks:

1. **Memory Safety**
   - No unsafe code unless absolutely necessary
   - All unsafe blocks are documented and reviewed
   - No data races possible
   - No memory leaks

2. **Error Safety**
   - All errors are handled
   - No panics in production code
   - Error messages are informative
   - Error recovery is possible

3. **Resource Safety**
   - No resource leaks
   - Resource limits are enforced
   - Resources are properly cleaned up
   - No resource exhaustion

4. **Concurrency Safety**
   - No deadlocks
   - No race conditions
   - Proper synchronization
   - No lock contention

### Guardrails

1. **Code Review Required**
   - All code changes must be reviewed
   - Critical changes require multiple reviewers
   - Unsafe code requires extra scrutiny

2. **Testing Required**
   - All code must have tests
   - Tests must pass before merge
   - Coverage must meet minimum thresholds

3. **Documentation Required**
   - All public APIs must be documented
   - Complex logic must be explained
   - Changes must be documented

4. **Performance Monitoring**
   - Performance must be measured
   - Regressions must be caught
   - Benchmarks must be updated

### Safety Checklist

- [ ] No unsafe code unless necessary
- [ ] All errors are handled
- [ ] No resource leaks
- [ ] No concurrency issues
- [ ] All inputs are validated
- [ ] All tests pass
- [ ] Documentation is complete
- [ ] Performance is acceptable
- [ ] Code is reviewed
- [ ] Changes are tested

---

## Performance Considerations

### Performance Analysis Before Changes

1. **Profile the Code**
   - Identify hot paths
   - Measure current performance
   - Identify bottlenecks
   - Establish baseline

2. **Analyze Complexity**
   - Time complexity analysis
   - Space complexity analysis
   - Algorithmic efficiency
   - Data structure efficiency

3. **Consider Trade-offs**
   - Performance vs. maintainability
   - Performance vs. safety
   - Performance vs. features
   - Performance vs. development time

### Performance Optimization Guidelines

1. **Algorithmic Optimizations**
   - Choose appropriate algorithms
   - Use efficient data structures
   - Avoid unnecessary work
   - Consider caching

2. **Memory Optimizations**
   - Minimize allocations
   - Reuse buffers
   - Use stack allocation
   - Implement pooling

3. **Cache Optimizations**
   - Use contiguous memory
   - Optimize access patterns
   - Consider cache line size
   - Minimize pointer chasing

4. **Concurrency Optimizations**
   - Use async/await for I/O
   - Use threads for CPU work
   - Minimize synchronization
   - Consider lock-free structures

### Performance Monitoring

1. **Establish Metrics**
   - Response time
   - Throughput
   - Resource usage
   - Error rates

2. **Set Thresholds**
   - Performance targets
   - Alert thresholds
   - SLOs (Service Level Objectives)
   - SLIs (Service Level Indicators)

3. **Monitor Continuously**
   - Track metrics over time
   - Detect regressions
   - Identify trends
   - Alert on issues

### Performance Checklist

- [ ] Performance is measured
- [ ] Baseline is established
- [ ] Optimizations are justified
- [ ] Trade-offs are considered
- [ ] Metrics are monitored
- [ ] Regressions are caught
- [ ] Benchmarks are updated
- [ ] Documentation is updated
- [ ] Code is reviewed
- [ ] Changes are tested

---

## Self-Improvement Guidelines

### Self-Improvement Process

1. **Observation**
   - Monitor performance metrics
   - Collect feedback
   - Analyze patterns
   - Identify opportunities

2. **Analysis**
   - Analyze metrics
   - Identify bottlenecks
   - Prioritize improvements
   - Plan changes

3. **Planning**
   - Design improvements
   - Plan implementation
   - Plan testing
   - Plan rollback

4. **Execution**
   - Implement changes
   - Test thoroughly
   - Monitor impact
   - Document changes

5. **Validation**
   - Verify improvements
   - Check for regressions
   - Update documentation
   - Commit or rollback

### Self-Improvement Principles

1. **Incremental Changes**
   - Make small changes
   - Test each change
   - Monitor impact
   - Iterate quickly

2. **Data-Driven**
   - Base decisions on data
   - Measure everything
   - Analyze results
   - Adjust based on feedback

3. **Safe Improvements**
   - Test thoroughly
   - Have rollback plan
   - Monitor closely
   - Document everything

4. **Continuous Learning**
   - Learn from mistakes
   - Share knowledge
   - Improve processes
   - Update guidelines

### Self-Improvement Areas

1. **Code Quality**
   - Reduce complexity
   - Improve clarity
   - Reduce duplication
   - Enhance maintainability

2. **Performance**
   - Improve response time
   - Reduce resource usage
   - Increase throughput
   - Optimize hot paths

3. **Capabilities**
   - Add new features
   - Improve existing features
   - Fix bugs
   - Enhance UX

4. **Safety**
   - Improve error handling
   - Enhance validation
   - Strengthen security
   - Improve reliability

### Self-Improvement Checklist

- [ ] Improvement is justified
- [ ] Baseline is measured
- [ ] Plan is documented
- [ ] Tests are comprehensive
- [ ] Rollback plan exists
- [ ] Impact is monitored
- [ ] Results are analyzed
- [ ] Documentation is updated
- [ ] Knowledge is shared
- [ ] Process is improved

---

## Error Handling Instructions

### Error Handling Principles

1. **Explicit Error Handling**
   - Use `Result<T, E>` for all fallible operations
   - Never ignore errors
   - Handle errors appropriately
   - Provide context with errors

2. **Informative Errors**
   - Include relevant context
   - Provide actionable information
   - Suggest solutions
   - Log errors appropriately

3. **Error Recovery**
   - Design for error recovery
   - Implement retry logic
   - Provide fallbacks
   - Graceful degradation

### Error Type Design

1. **Use `thiserror` for Error Types**
   ```rust
   use thiserror::Error;
   
   #[derive(Error, Debug)]
   pub enum AgentError {
       #[error("LLM request failed: {0}")]
       LlmError(#[from] reqwest::Error),
       
       #[error("Invalid configuration: {field} - {message}")]
       ConfigError { field: String, message: String },
       
       #[error("Task execution failed: {0}")]
       TaskError(String),
   }
   ```

2. **Use `anyhow` for Application Errors**
   ```rust
   use anyhow::{Context, Result};
   
   pub async fn process_task(task: Task) -> Result<Output> {
       let result = perform_operation(task)
           .await
           .context("Failed to perform operation")?;
       Ok(result)
   }
   ```

### Error Handling Patterns

1. **Propagate Errors with `?`**
   ```rust
   pub async fn process_task(task: Task) -> Result<TaskResult, AgentError> {
       let result = execute_task(task)?;
       Ok(result)
   }
   ```

2. **Handle Specific Errors**
   ```rust
   match execute_task(task).await {
       Ok(result) => println!("Success: {:?}", result),
       Err(AgentError::LlmError(e)) => eprintln!("LLM error: {}", e),
       Err(e) => eprintln!("Error: {}", e),
   }
   ```

3. **Provide Context**
   ```rust
   let result = operation()
       .await
       .context("Failed to process task")?;
   ```

### Error Handling Checklist

- [ ] All errors are handled
- [ ] Errors are informative
- [ ] Error types are appropriate
- [ ] Error recovery is considered
- [ ] Errors are logged
- [ ] Error context is provided
- [ ] No panics in production
- [ ] Error handling is tested
- [ ] Error documentation is complete
- [ ] Error patterns are consistent

---

## Testing Instructions

### Testing Principles

1. **Test Everything**
   - Test all public functions
   - Test all error paths
   - Test edge cases
   - Test integration points

2. **Test Independently**
   - Tests should not depend on each other
   - Tests should be fast
   - Tests should be deterministic
   - Tests should be maintainable

3. **Test Realistically**
   - Test with realistic data
   - Test realistic scenarios
   - Test error conditions
   - Test performance characteristics

### Test Types

1. **Unit Tests**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_function_success() {
           let result = function(input);
           assert_eq!(result, expected);
       }
       
       #[test]
       fn test_function_error() {
           let result = function(invalid_input);
           assert!(result.is_err());
       }
   }
   ```

2. **Integration Tests**
   ```rust
   #[tokio::test]
   async fn test_workflow() {
       let mut agent = Agent::new(config).await;
       agent.initialize().await.unwrap();
       
       let task = Task::new("test task");
       let task_id = agent.submit_task(task).await.unwrap();
       
       // Verify result
   }
   ```

3. **Property-Based Tests**
   ```rust
   use proptest::prelude::*;
   
   proptest! {
       #[test]
       fn test_property(input in any::<Vec<u8>>) {
           let result = process(&input);
           prop_assert!(verify_property(&result));
       }
   }
   ```

### Test Guidelines

1. **Descriptive Test Names**
   ```rust
   #[test]
   fn test_process_task_with_valid_input_returns_success() {
       // Test implementation
   }
   ```

2. **Test Fixtures**
   ```rust
   fn setup_test_config() -> Config {
       Config {
           timeout: Duration::from_secs(10),
           max_retries: 3,
       }
   }
   ```

3. **Test Helpers**
   ```rust
   async fn create_test_agent() -> Agent {
       let config = setup_test_config();
       Agent::new(config).await
   }
   ```

### Testing Checklist

- [ ] All public functions are tested
- [ ] Error paths are tested
- [ ] Edge cases are tested
- [ ] Integration tests exist
- [ ] Tests are independent
- [ ] Tests are fast
- [ ] Tests are descriptive
- [ ] Test coverage is high
- [ ] Tests are maintained
- [ ] Tests pass consistently

---

## Documentation Instructions

### Documentation Principles

1. **Document Everything**
   - Document all public APIs
   - Document complex logic
   - Document design decisions
   - Document trade-offs

2. **Document Clearly**
   - Use clear language
   - Provide examples
   - Explain why, not just what
   - Keep documentation up to date

3. **Document Completely**
   - Document parameters
   - Document return values
   - Document errors
   - Document performance

### Documentation Types

1. **Code Documentation**
   ```rust
   /// Processes a task and returns the result.
   ///
   /// # Arguments
   ///
   /// * `task` - The task to process
   ///
   /// # Returns
   ///
   /// Returns a `Result` containing the processed output or an error.
   ///
   /// # Errors
   ///
   /// This function will return an error if:
   /// - The task is invalid
   /// - Processing fails
   ///
   /// # Examples
   ///
   /// ```
   /// let result = process_task(task)?;
   /// ```
   pub fn process_task(task: Task) -> Result<Output, Error> {
       // Implementation
   }
   ```

2. **Module Documentation**
   ```rust
   //! Task orchestration and management.
   //!
   //! This module provides the core functionality for managing tasks,
   //! including task creation, execution, and result handling.
   ```

3. **Architecture Documentation**
   ```markdown
   # Task Processing Architecture
   
   ## Overview
   Tasks are processed through a pipeline of stages:
   1. Validation
   2. Planning
   3. Execution
   4. Verification
   
   ## Design Decisions
   - Async processing for I/O efficiency
   - Result caching for performance
   - Error recovery for reliability
   ```

### Documentation Guidelines

1. **Use Examples**
   ```rust
   /// # Examples
   ///
   /// ```
   /// use agent::Task;
   ///
   /// let task = Task::new("example task");
   /// let result = process_task(task)?;
   /// ```
   ```

2. **Document Errors**
   ```rust
   /// # Errors
   ///
   /// This function will return an error if:
   /// - The task is invalid
   /// - Processing fails
   /// - Timeout occurs
   ```

3. **Document Performance**
   ```rust
   /// # Performance
   ///
   /// This function has O(n) time complexity and O(1) space complexity.
   /// For large tasks, consider using `process_task_async` instead.
   ```

### Documentation Checklist

- [ ] All public APIs are documented
- [ ] Documentation includes examples
- [ ] Errors are documented
- [ ] Performance is documented
- [ ] Design decisions are documented
- [ ] Trade-offs are documented
- [ ] Documentation is clear
- [ ] Documentation is complete
- [ ] Documentation is up to date
- [ ] Documentation builds without errors

---

## Summary

These instructions and templates provide a comprehensive framework for AI agents working on the Self-Developing Coding Agent project. They translate the principles from [`PRINCIPLES.md`](PRINCIPLES.md) into actionable guidelines and templates.

**Key Points**:

1. **Safety First**: Always prioritize safety in code generation and modification
2. **Performance Awareness**: Consider performance implications but measure before optimizing
3. **Quality Standards**: Follow established code quality and documentation standards
4. **Testing**: Write comprehensive tests for all code
5. **Documentation**: Document all public APIs and complex implementations
6. **Self-Improvement**: Continuously identify and implement improvements

These instructions should be incorporated into all agent prompts and followed for all code generation and modification tasks.

---

## Template Selection Guide

This guide helps you select the appropriate template for your task.

### Quick Reference Table

| Template | Use Case | Complexity | Frequency | Priority |
|-----------|-----------|-------------|-------------|----------|
| Template 1: Code Generation | Creating new code | Medium | High | High |
| Template 2: Code Refactoring | Improving existing code | High | Medium | High |
| Template 3: Code Review | Reviewing code changes | Medium | High | High |
| Template 4: Test Generation | Writing tests | Medium | High | High |
| Template 5: Documentation Generation | Writing documentation | Low | Medium | Medium |
| Template 6: Performance Analysis | Analyzing performance | High | Low | Medium |
| Template 7: Self-Improvement Analysis | Improving the agent | High | Low | High |
| Template 8: Debugging Assistance | Debugging issues | High | High | High |
| Template 9: Architecture Design | Designing systems | High | Medium | High |
| Template 10: Dependency Analysis | Managing dependencies | Medium | Low | Medium |
| Template 11: Security Audit | Security reviews | High | Medium | High |
| Template 12: Code Migration | Migrating code | High | Low | Medium |
| Template 13: Feature Implementation Planning | Planning features | Medium | High | High |
| Template 14: Error Analysis & Resolution | Analyzing errors | Medium | Medium | Medium |
| Template 15: Context Gathering | Gathering context | Low | High | High |
| Template 16: Impact Analysis | Analyzing impact | Medium | High | High |
| Template 17: Pattern Discovery | Finding patterns | Medium | Low | Low |

### Selection Flowchart

```
Start
  │
  ├─→ Is this a new feature?
  │     └─→ Use Template 13: Feature Implementation Planning
  │           └─→ Then use Template 1: Code Generation
  │
  ├─→ Is this a bug fix?
  │     └─→ Use Template 8: Debugging Assistance
  │           └─→ Then use Template 1: Code Generation
  │
  ├─→ Is this a code review?
  │     └─→ Use Template 3: Code Review
  │
  ├─→ Is this a refactoring?
  │     └─→ Use Template 2: Code Refactoring
  │
  ├─→ Is this a performance issue?
  │     └─→ Use Template 6: Performance Analysis
  │
  ├─→ Is this a security concern?
  │     └─→ Use Template 11: Security Audit
  │
  ├─→ Is this an architecture decision?
  │     └─→ Use Template 9: Architecture Design
  │
  ├─→ Is this a dependency change?
  │     └─→ Use Template 10: Dependency Analysis
  │
  ├─→ Is this a code migration?
  │     └─→ Use Template 12: Code Migration
  │
  ├─→ Is this an error analysis?
  │     └─→ Use Template 14: Error Analysis & Resolution
  │
  ├─→ Do you need to understand impact?
  │     └─→ Use Template 16: Impact Analysis
  │
  ├─→ Do you need to gather context?
  │     └─→ Use Template 15: Context Gathering
  │
  └─→ Are you looking for patterns?
        └─→ Use Template 17: Pattern Discovery
```

### Template Combinations

Some tasks require using multiple templates in sequence:

**New Feature Development**:
1. Template 13: Feature Implementation Planning
2. Template 15: Context Gathering
3. Template 1: Code Generation
4. Template 4: Test Generation
5. Template 5: Documentation Generation
6. Template 3: Code Review

**Bug Fix**:
1. Template 8: Debugging Assistance
2. Template 16: Impact Analysis
3. Template 1: Code Generation
4. Template 4: Test Generation
5. Template 3: Code Review

**Refactoring**:
1. Template 2: Code Refactoring
2. Template 16: Impact Analysis
3. Template 4: Test Generation
4. Template 3: Code Review

**Performance Optimization**:
1. Template 6: Performance Analysis
2. Template 2: Code Refactoring
3. Template 4: Test Generation
4. Template 3: Code Review

**Security Review**:
1. Template 11: Security Audit
2. Template 16: Impact Analysis
3. Template 1: Code Generation (if fixes needed)
4. Template 4: Test Generation
5. Template 3: Code Review

### Template Usage Guidelines

**When to Use Multiple Templates**:
- Complex tasks that span multiple phases
- Tasks requiring different types of analysis
- Tasks with significant impact or risk

**When to Use a Single Template**:
- Focused, well-defined tasks
- Tasks with clear scope
- Low-risk changes

**Template Selection Best Practices**:
1. Start with the template that best matches the primary task
2. Use additional templates as needed for comprehensive coverage
3. Consider the complexity and risk of the task
4. Ensure all relevant aspects are covered (testing, documentation, review)

### Template Integration with Project Functionality

All templates integrate with the following project functionality:

**LSP (Language Server Protocol)**:
- Code structure analysis
- Symbol definitions and references
- Type information
- Call graph analysis

**Knowledge Graph**:
- Dependency mapping
- Relationship understanding
- Architectural context
- Impact analysis

**Vector Store**:
- Similar code patterns
- Historical implementations
- Pattern matching
- Context retrieval

**Tools**:
- clippy: Additional linting
- rustfmt: Style checking
- cargo test: Test execution
- cargo doc: Documentation generation
- cargo-audit: Security vulnerability scanning
- criterion: Benchmarking

### Prompt Engineering Techniques Used

All templates incorporate the following prompt engineering best practices:

1. **Chain-of-Thought (CoT)**: Step-by-step reasoning guidance
2. **Few-Shot Learning**: Examples to guide output format
3. **Self-Consistency**: Multiple approaches and validation
4. **Tree-of-Thoughts (ToT)**: Exploring multiple solution paths
5. **Context Window Optimization**: Prioritized context management
6. **Output Format Specification**: Structured, parseable output
7. **Safety Guardrails**: Explicit safety constraints and validation

### Template Maintenance

Templates should be reviewed and updated regularly to:
- Incorporate new prompt engineering techniques
- Improve effectiveness based on usage data
- Add new templates as needed
- Refine existing templates based on feedback

For more detailed information on prompt template research and best practices, see [`PROMPT_RESEARCH.md`](PROMPT_RESEARCH.md).
