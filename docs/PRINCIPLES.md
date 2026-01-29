# Principles of Safe and Performant Software Development

This document synthesizes core principles from systems programming best practices, critical systems development, and performance optimization. These principles guide the development of the Self-Developing Coding Agent and should be incorporated into all code generation and modification decisions.

## Table of Contents

- [Core Principles for Safe Software Development](#core-principles-for-safe-software-development)
- [Core Principles for Performant Software Development](#core-principles-for-performant-software-development)
- [Rust-Specific Systems Programming Guidelines](#rust-specific-systems-programming-guidelines)
- [Best Practices for Critical Systems](#best-practices-for-critical-systems)
- [Performance Optimization Strategies](#performance-optimization-strategies)
- [Code Quality Standards](#code-quality-standards)
- [Testing and Verification Requirements](#testing-and-verification-requirements)
- [Documentation Standards](#documentation-standards)
- [References](#references)

---

## Core Principles for Safe Software Development

### 1. Simplicity and Clarity

**Principle**: Simple code is safer code. Complexity breeds bugs and makes systems harder to understand, test, and maintain.

**Guidelines**:
- Prefer straightforward implementations over clever ones
- Avoid premature optimization that sacrifices clarity
- Write code that reads like prose
- Use descriptive names for variables, functions, and types
- Keep functions small and focused on a single responsibility
- Limit nesting depth to 3-4 levels maximum

**Rationale**: From NASA's Power of 10 - "Avoid complex flow constructs like goto and setjmp/longjmp." Complex control flow is a major source of bugs in critical systems.

### 2. Explicit Over Implicit

**Principle**: Make behavior explicit and visible. Hidden behavior leads to unexpected side effects and bugs.

**Guidelines**:
- Avoid implicit type conversions
- Make all dependencies explicit in function signatures
- Use explicit error handling rather than silent failures
- Document all side effects
- Avoid global mutable state
- Make ownership and borrowing patterns clear

**Rationale**: TigerBeetle's tiger_style emphasizes explicit behavior to prevent subtle bugs that can occur with implicit operations.

### 3. Fail Fast and Fail Loudly

**Principle**: Detect errors as early as possible and make them impossible to ignore.

**Guidelines**:
- Use Rust's `Result` and `Option` types instead of panics where possible
- Validate all inputs at system boundaries
- Use `unwrap()` and `expect()` only when you can prove safety
- Log errors with sufficient context for debugging
- Never silently ignore errors
- Use assertions for invariants that must always hold

**Rationale**: Early error detection prevents bugs from propagating and causing harder-to-diagnose failures later.

### 4. Memory Safety by Construction

**Principle**: Design systems that are memory-safe by default, not by convention.

**Guidelines**:
- Leverage Rust's ownership system to prevent data races
- Use `Arc` and `Mutex` only when absolutely necessary
- Prefer stack allocation over heap allocation
- Avoid raw pointers and unsafe code
- Use `#[deny(unsafe_code)]` in modules that don't need it
- Audit all `unsafe` blocks with extra scrutiny

**Rationale**: Memory safety bugs are among the most dangerous and difficult to debug. Rust's type system provides compile-time guarantees.

### 5. Defensive Programming

**Principle**: Assume inputs can be invalid and design accordingly.

**Guidelines**:
- Validate all external inputs
- Use bounded loops and recursion
- Check for integer overflow in critical paths
- Use saturating or wrapping arithmetic explicitly when needed
- Implement rate limiting and resource quotas
- Design for graceful degradation

**Rationale**: From NASA's Power of 10 - "Restrict all code to very simple control flow constructs." Defensive programming prevents cascading failures.

### 6. Isolation and Containment

**Principle**: Limit the blast radius of failures through isolation.

**Guidelines**:
- Use process isolation for untrusted code
- Implement circuit breakers for external dependencies
- Use sandboxing for tool execution
- Separate critical and non-critical code paths
- Implement resource limits (memory, CPU, file handles)
- Use separate threads or tasks for independent operations

**Rationale**: Isolation prevents failures in one component from bringing down the entire system.

### 7. Deterministic Behavior

**Principle**: Systems should behave predictably and reproducibly.

**Guidelines**:
- Avoid non-deterministic data structures (e.g., `HashMap` without ordering)
- Use fixed-size data structures where possible
- Avoid relying on timing for correctness
- Make random number generation explicit and seedable
- Document all sources of non-determinism
- Use deterministic algorithms for critical operations

**Rationale**: Deterministic behavior is essential for testing, debugging, and reasoning about system correctness.

### 8. Resource Management

**Principle**: Explicitly manage all resources to prevent leaks and exhaustion.

**Guidelines**:
- Use RAII (Resource Acquisition Is Initialization) patterns
- Implement `Drop` for custom resource types
- Use scopes to limit resource lifetime
- Set timeouts on all I/O operations
- Implement backpressure for streaming operations
- Monitor resource usage and alert on anomalies

**Rationale**: Resource leaks can cause gradual degradation and eventual system failure.

---

## Core Principles for Performant Software Development

### 1. Measure Before Optimizing

**Principle**: Never optimize without measurements. Premature optimization is the root of much evil.

**Guidelines**:
- Profile before optimizing
- Use benchmarks to measure performance impact
- Focus on hot paths identified by profiling
- Establish performance baselines
- Track performance metrics over time
- Optimize for the actual workload, not theoretical cases

**Rationale**: Casey Muratori's Performance Aware Programming emphasizes that optimization without measurement often wastes time and can make code worse.

### 2. Algorithmic Efficiency First

**Principle**: Choose the right algorithm before micro-optimizing.

**Guidelines**:
- Understand time and space complexity of algorithms
- Choose appropriate data structures for the problem
- Prefer O(n log n) over O(n²) when possible
- Consider cache locality in data structure design
- Use appropriate search and sort algorithms
- Avoid unnecessary work in loops

**Rationale**: Algorithmic improvements often provide orders-of-magnitude speedups compared to micro-optimizations.

### 3. Minimize Allocations

**Principle**: Memory allocation is expensive. Minimize it, especially in hot paths.

**Guidelines**:
- Reuse buffers and allocations where possible
- Use stack allocation for small, short-lived data
- Implement object pooling for frequently allocated types
- Use `Cow` (Copy on Write) for conditional ownership
- Prefer iterators over collecting into intermediate vectors
- Use `String::with_capacity` when size is known

**Rationale**: Allocation and deallocation are among the most expensive operations in many programs.

### 4. Cache-Friendly Design

**Principle**: Design for CPU cache efficiency. Cache misses are expensive.

**Guidelines**:
- Use contiguous memory layouts (arrays, Vec)
- Structure data for sequential access patterns
- Avoid pointer chasing in hot paths
- Use struct-of-arrays instead of array-of-structs when beneficial
- Align data structures to cache line boundaries
- Consider cache line size (typically 64 bytes) in design

**Rationale**: Cache misses can cost hundreds of CPU cycles. Cache-friendly designs can dramatically improve performance.

### 5. Zero-Cost Abstractions

**Principle**: Use abstractions that compile down to efficient code.

**Guidelines**:
- Leverage Rust's zero-cost abstractions (iterators, closures)
- Use generics instead of runtime polymorphism when possible
- Prefer compile-time computation over runtime
- Use `const` and `const fn` for compile-time evaluation
- Avoid virtual dispatch in performance-critical code
- Use `#[inline]` judiciously for small, hot functions

**Rationale**: Zero-cost abstractions allow writing clean, expressive code without runtime overhead.

### 6. Parallelism and Concurrency

**Principle**: Use parallelism to leverage multiple cores, but be careful about overhead.

**Guidelines**:
- Use async/await for I/O-bound operations
- Use threads for CPU-bound parallel work
- Consider work-stealing schedulers for task parallelism
- Minimize synchronization and contention
- Use lock-free data structures when appropriate
- Profile to ensure parallelism provides benefit

**Rationale**: Proper use of parallelism can provide near-linear speedups, but incorrect use can degrade performance.

### 7. I/O Efficiency

**Principle**: I/O is orders of magnitude slower than memory. Minimize and batch I/O operations.

**Guidelines**:
- Use buffered I/O
- Batch small operations into larger ones
- Use asynchronous I/O to avoid blocking
- Implement read-ahead and write-behind caching
- Use efficient serialization formats
- Consider memory-mapped files for large data

**Rationale**: I/O operations are typically the bottleneck in many applications. Efficient I/O can dramatically improve performance.

### 8. Lazy Evaluation

**Principle**: Defer work until it's actually needed.

**Guidelines**:
- Use iterators for lazy processing
- Implement lazy loading for large data structures
- Use `Option` and `Result` to defer error handling
- Avoid computing values that might not be used
- Use memoization for expensive, repeated computations
- Consider on-demand computation for complex operations

**Rationale**: Lazy evaluation avoids unnecessary work and can improve both performance and memory usage.

---

## Rust-Specific Systems Programming Guidelines

### 1. Ownership and Borrowing

**Principle**: Leverage Rust's ownership system for memory safety without garbage collection.

**Guidelines**:
- Understand and respect ownership rules
- Use borrowing instead of cloning when possible
- Use `Cow` for conditional ownership
- Prefer references over owned values in function signatures
- Use lifetime annotations to make borrowing relationships explicit
- Consider using `Rc`/`Arc` for shared ownership when needed

**Example**:
```rust
// Good: Borrow instead of clone
fn process_data(data: &[u8]) -> Result<()> {
    // Process data without taking ownership
}

// Avoid: Unnecessary cloning
fn process_data_bad(data: Vec<u8>) -> Result<()> {
    // Takes ownership, may require clone at call site
}
```

### 2. Error Handling

**Principle**: Use Rust's error handling features for explicit, composable error management.

**Guidelines**:
- Use `Result<T, E>` for fallible operations
- Use `thiserror` for defining error types
- Use `anyhow` for application-level error handling
- Implement `From` traits for error conversion
- Use `?` operator for error propagation
- Provide context with errors using `.context()` from anyhow

**Example**:
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

pub async fn execute_task(task: Task) -> Result<TaskResult, AgentError> {
    let result = perform_operation(task)?;
    Ok(result)
}
```

### 3. Concurrency

**Principle**: Use Rust's concurrency primitives for safe parallel programming.

**Guidelines**:
- Use `async`/`await` for I/O-bound concurrency
- Use `tokio::spawn` for independent concurrent tasks
- Use `tokio::sync` primitives (Mutex, RwLock, Semaphore) for synchronization
- Use channels (`mpsc`, `broadcast`, `watch`) for message passing
- Avoid blocking operations in async contexts
- Use `Send` and `Sync` bounds to ensure thread safety

**Example**:
```rust
use tokio::sync::mpsc;

pub async fn process_tasks(tasks: Vec<Task>) -> Vec<Result> {
    let (tx, mut rx) = mpsc::channel(100);
    
    // Spawn worker tasks
    for _ in 0..num_cpus::get() {
        let tx = tx.clone();
        tokio::spawn(async move {
            while let Some(task) = rx.recv().await {
                let result = process_task(task).await;
                // Send result back
            }
        });
    }
    
    // Send tasks
    for task in tasks {
        tx.send(task).await?;
    }
    
    // Collect results
    // ...
}
```

### 4. Unsafe Code

**Principle**: Minimize unsafe code and audit it thoroughly.

**Guidelines**:
- Avoid `unsafe` unless absolutely necessary
- Document all `unsafe` blocks with safety invariants
- Keep `unsafe` blocks small and well-contained
- Use `#[deny(unsafe_code)]` in safe modules
- Review all `unsafe` code in code review
- Consider using safe alternatives (e.g., `MaybeUninit` instead of raw pointers)

**Example**:
```rust
/// # Safety
/// 
/// The caller must ensure that:
/// - `ptr` is valid for reads of `len` bytes
/// - `ptr` is properly aligned for `T`
/// - The memory referenced by `ptr` is not mutated for the lifetime of the reference
unsafe fn read_slice<T>(ptr: *const T, len: usize) -> &[T] {
    std::slice::from_raw_parts(ptr, len)
}
```

### 5. Generic Programming

**Principle**: Use generics for code reuse without runtime overhead.

**Guidelines**:
- Use generics for algorithms that work on multiple types
- Use trait bounds to specify required functionality
- Prefer trait objects (`dyn Trait`) only when runtime polymorphism is needed
- Use associated types for trait relationships
- Use `where` clauses for complex bounds
- Consider using `impl Trait` for simpler function signatures

**Example**:
```rust
// Generic function with trait bounds
pub fn process_items<T, E>(items: Vec<T>) -> Result<Vec<Processed>, E>
where
    T: Processable<Error = E>,
{
    items.into_iter()
        .map(|item| item.process())
        .collect()
}

// Using impl Trait for simpler signatures
pub fn find_item(items: &[Item], predicate: impl Fn(&Item) -> bool) -> Option<&Item> {
    items.iter().find(predicate)
}
```

### 6. Zero-Cost Abstractions

**Principle**: Use Rust's abstractions that compile to efficient code.

**Guidelines**:
- Use iterators instead of loops with intermediate collections
- Use closures for short, simple operations
- Use `Option` and `Result` instead of null pointers and exceptions
- Use pattern matching for exhaustive handling
- Use `#[inline]` for small, frequently called functions
- Use `const` generics for compile-time parameters

**Example**:
```rust
// Iterator chain - compiles to efficient loop
pub fn process_numbers(numbers: &[i32]) -> Vec<i32> {
    numbers.iter()
        .filter(|&&n| n > 0)
        .map(|&n| n * 2)
        .collect()
}

// Equivalent to manual loop but more expressive
pub fn process_numbers_manual(numbers: &[i32]) -> Vec<i32> {
    let mut result = Vec::with_capacity(numbers.len());
    for &n in numbers {
        if n > 0 {
            result.push(n * 2);
        }
    }
    result
}
```

### 7. Memory Layout

**Principle**: Understand and optimize memory layout for performance.

**Guidelines**:
- Use `#[repr(C)]` for FFI compatibility
- Use `#[repr(transparent)]` for newtype wrappers
- Order struct fields by size (largest first) to minimize padding
- Use `Box` for large data to avoid stack overflow
- Use `Vec` for dynamic arrays with contiguous memory
- Consider `SmallVec` for small, stack-allocated vectors

**Example**:
```rust
// Good: Fields ordered by size to minimize padding
#[repr(C)]
pub struct OptimizedStruct {
    data: [u8; 64],  // 64 bytes
    count: u64,      // 8 bytes
    flags: u32,      // 4 bytes
    id: u16,         // 2 bytes
    tag: u8,         // 1 byte
}

// Bad: Fields in random order, more padding
pub struct PaddedStruct {
    tag: u8,         // 1 byte + 7 padding
    id: u16,         // 2 bytes + 6 padding
    flags: u32,      // 4 bytes + 4 padding
    count: u64,      // 8 bytes
    data: [u8; 64],  // 64 bytes
}
```

---

## Best Practices for Critical Systems

### 1. NASA's Power of 10 Rules

**Rule 1**: Restrict all code to very simple control flow constructs.

- Do not use `goto` statements
- Do not use `setjmp` or `longjmp`
- Avoid deep nesting
- Use early returns to reduce nesting

**Rule 2**: Give all loops a fixed upper-bound.

- Avoid infinite loops
- Use `for` loops with known bounds when possible
- Add timeout logic to `while` loops
- Implement watchdog timers for critical operations

**Rule 3**: Do not use dynamic memory allocation after initialization.

- Allocate all memory at startup
- Use fixed-size buffers
- Implement memory pools if dynamic allocation is needed
- Avoid heap allocation in real-time code paths

**Rule 4**: No function should be longer than what can be printed on a single sheet of paper.

- Keep functions under 60 lines
- Split large functions into smaller, focused ones
- Use helper functions for complex logic
- Maintain single responsibility principle

**Rule 5**: The assertion density of the code should average to a minimum of two assertions per function.

- Use `assert!` for invariants
- Use `debug_assert!` for debug-time checks
- Add assertions at function boundaries
- Document assumptions with assertions

**Rule 6**: Declare all data objects at the smallest possible level of scope.

- Minimize variable lifetime
- Use block scope for temporary variables
- Avoid global variables
- Use function parameters instead of shared state

**Rule 7**: Check the return value of all non-void functions.

- Never ignore `Result` or `Option`
- Use `?` operator for error propagation
- Handle all error cases explicitly
- Log errors with context

**Rule 8**: Use the preprocessor sparingly.

- Avoid complex macros
- Prefer `const` and `const fn` over macros
- Use macros only when necessary
- Document macro behavior clearly

**Rule 9**: Limit pointer use to a single dereferencing level.

- Avoid pointer chains
- Use references instead of raw pointers
- Minimize indirection
- Prefer value semantics

**Rule 10**: Compile with all possible warnings active.

- Use `-Wall -Wextra` (or Rust equivalent)
- Treat warnings as errors in CI
- Use `cargo clippy` for additional lints
- Enable `#[deny(warnings)]` in critical modules

### 2. TigerBeetle's Systems Programming Principles

**Principle 1**: Simplicity is the ultimate sophistication.

- Write code that is easy to understand
- Avoid clever tricks that obscure intent
- Prefer explicit over implicit
- Document non-obvious decisions

**Principle 2**: Performance is a feature.

- Measure before optimizing
- Optimize for the actual workload
- Consider performance in design decisions
- Profile regularly

**Principle 3**: Correctness is non-negotiable.

- Use formal methods where appropriate
- Write comprehensive tests
- Use property-based testing
- Verify critical algorithms

**Principle 4**: Reliability through determinism.

- Avoid non-deterministic behavior
- Make random generation explicit
- Use deterministic data structures
- Document all sources of non-determinism

**Principle 5**: Testability is a design goal.

- Write testable code
- Use dependency injection
- Avoid global state
- Mock external dependencies

### 3. Critical System Design Patterns

**Pattern 1: Watchdog Timer**

Implement watchdog timers to detect and recover from hangs:

```rust
pub struct Watchdog {
    duration: Duration,
    last_heartbeat: Instant,
}

impl Watchdog {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            last_heartbeat: Instant::now(),
        }
    }
    
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
    }
    
    pub fn check(&self) -> bool {
        self.last_heartbeat.elapsed() < self.duration
    }
}
```

**Pattern 2: Circuit Breaker**

Implement circuit breakers to prevent cascading failures:

```rust
pub struct CircuitBreaker {
    failure_count: usize,
    threshold: usize,
    last_failure: Option<Instant>,
    timeout: Duration,
    state: CircuitState,
}

enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn call<F, R, E>(&mut self, f: F) -> Result<R, E>
    where
        F: FnOnce() -> Result<R, E>,
    {
        match self.state {
            CircuitState::Open => {
                if self.should_attempt_reset() {
                    self.state = CircuitState::HalfOpen;
                } else {
                    return Err(/* circuit open error */);
                }
            }
            _ => {}
        }
        
        match f() {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(error) => {
                self.on_failure();
                Err(error)
            }
        }
    }
}
```

**Pattern 3: Idempotent Operations**

Design operations to be idempotent for safe retries:

```rust
pub trait IdempotentOperation {
    type Output;
    type Error;
    
    async fn execute(&self) -> Result<Self::Output, Self::Error>;
}

// Example: Idempotent file write
pub struct IdempotentFileWrite {
    path: PathBuf,
    content: Vec<u8>,
    expected_hash: Option<[u8; 32]>,
}

impl IdempotentOperation for IdempotentFileWrite {
    type Output = ();
    type Error = std::io::Error;
    
    async fn execute(&self) -> Result<Self::Output, Self::Error> {
        // Check if file already exists with expected content
        if let Ok(existing) = tokio::fs::read(&self.path).await {
            if let Some(expected) = self.expected_hash {
                let hash = compute_hash(&existing);
                if hash == expected {
                    return Ok(()); // Already done
                }
            }
        }
        
        // Write file atomically
        atomic_write(&self.path, &self.content).await
    }
}
```

---

## Performance Optimization Strategies

### 1. Profiling and Measurement

**Strategy**: Use profiling tools to identify bottlenecks before optimizing.

**Tools**:
- `cargo flamegraph` for flame graphs
- `perf` for Linux profiling
- `tokio-console` for async runtime profiling
- `criterion` for benchmarking
- `heaptrack` for memory profiling

**Guidelines**:
- Profile realistic workloads
- Measure before and after optimizations
- Focus on hot paths identified by profiling
- Track performance metrics over time
- Use continuous performance monitoring

### 2. Memory Optimization

**Strategy**: Minimize memory usage and allocations for better performance.

**Techniques**:
- Use stack allocation for small, short-lived data
- Implement object pooling for frequently allocated types
- Use `String::with_capacity` when size is known
- Reuse buffers instead of allocating new ones
- Use `Cow` for conditional ownership
- Consider `SmallVec` for small, stack-allocated vectors

**Example**:
```rust
// Object pooling for frequently allocated types
pub struct Pool<T> {
    objects: Vec<T>,
    create: Box<dyn Fn() -> T>,
}

impl<T> Pool<T> {
    pub fn acquire(&mut self) -> T {
        self.objects.pop().unwrap_or_else(|| (self.create)())
    }
    
    pub fn release(&mut self, object: T) {
        self.objects.push(object);
    }
}
```

### 3. CPU Optimization

**Strategy**: Optimize CPU usage through efficient algorithms and data structures.

**Techniques**:
- Choose appropriate algorithms for the problem
- Use efficient data structures (e.g., `HashMap` vs `BTreeMap`)
- Minimize branching in hot paths
- Use SIMD instructions when beneficial
- Consider cache locality in data layout
- Use branch prediction hints (`likely`/`unlikely`)

**Example**:
```rust
// Branch prediction hints
#[inline]
fn process_value(value: u32) -> u32 {
    if likely!(value > 100) {
        value * 2
    } else {
        value + 10
    }
}
```

### 4. I/O Optimization

**Strategy**: Minimize I/O operations and batch them when possible.

**Techniques**:
- Use buffered I/O
- Batch small operations into larger ones
- Use asynchronous I/O to avoid blocking
- Implement read-ahead and write-behind caching
- Use efficient serialization formats
- Consider memory-mapped files for large data

**Example**:
```rust
// Batched I/O operations
pub struct BatchWriter<W> {
    writer: W,
    buffer: Vec<u8>,
    batch_size: usize,
}

impl<W: AsyncWrite + Unpin> BatchWriter<W> {
    pub async fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.buffer.extend_from_slice(data);
        if self.buffer.len() >= self.batch_size {
            self.flush().await?;
        }
        Ok(())
    }
    
    pub async fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            self.writer.write_all(&self.buffer).await?;
            self.buffer.clear();
        }
        Ok(())
    }
}
```

### 5. Concurrency Optimization

**Strategy**: Use concurrency effectively to leverage multiple cores.

**Techniques**:
- Use async/await for I/O-bound operations
- Use threads for CPU-bound parallel work
- Minimize synchronization and contention
- Use lock-free data structures when appropriate
- Consider work-stealing schedulers
- Profile to ensure parallelism provides benefit

**Example**:
```rust
// Parallel processing with rayon
use rayon::prelude::*;

pub fn process_parallel<T, R>(items: Vec<T>) -> Vec<R>
where
    T: Send + Sync,
    R: Send,
{
    items.into_par_iter()
        .map(|item| process_item(item))
        .collect()
}
```

### 6. Caching Strategies

**Strategy**: Use caching to avoid redundant computations and I/O.

**Techniques**:
- Cache expensive computations
- Use LRU caches for frequently accessed data
- Implement cache invalidation strategies
- Consider cache warming for critical paths
- Use memoization for pure functions
- Monitor cache hit rates

**Example**:
```rust
// LRU cache implementation
use lru::LruCache;

pub struct CachedComputation<K, V, F>
where
    F: Fn(&K) -> V,
{
    cache: LruCache<K, V>,
    compute: F,
}

impl<K, V, F> CachedComputation<K, V, F>
where
    K: Hash + Eq,
    F: Fn(&K) -> V,
{
    pub fn get(&mut self, key: K) -> V {
        if let Some(value) = self.cache.get(&key) {
            return value.clone();
        }
        
        let value = (self.compute)(&key);
        self.cache.put(key, value.clone());
        value
    }
}
```

---

## Code Quality Standards

### 1. Code Style and Formatting

**Standard**: Use consistent code style and formatting across the codebase.

**Guidelines**:
- Use `cargo fmt` for automatic formatting
- Follow Rust naming conventions
- Use meaningful names for variables, functions, and types
- Keep lines under 100 characters
- Use consistent indentation (4 spaces)
- Use trailing commas in multi-line structures

**Example**:
```rust
// Good: Consistent formatting
pub fn process_data(
    input: &Input,
    config: &Config,
) -> Result<Output, Error> {
    let result = input
        .items
        .iter()
        .map(|item| process_item(item, config))
        .collect::<Result<Vec<_>, _>>()?;
    
    Ok(Output { result })
}
```

### 2. Documentation

**Standard**: Document all public APIs and complex implementations.

**Guidelines**:
- Document all public items with `///` or `//!`
- Include examples in documentation
- Document all parameters and return values
- Document errors that can be returned
- Include performance characteristics where relevant
- Use `#[doc]` attributes for special documentation

**Example**:
```rust
/// Processes a task and returns the result.
///
/// # Arguments
///
/// * `task` - The task to process
/// * `config` - Configuration for processing
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
/// - Configuration is invalid
///
/// # Examples
///
/// ```
/// use agent::Task;
///
/// let task = Task::new("example");
/// let result = process_task(&task, &config)?;
/// ```
pub fn process_task(task: &Task, config: &Config) -> Result<Output, Error> {
    // Implementation
}
```

### 3. Error Handling

**Standard**: Use consistent and informative error handling.

**Guidelines**:
- Use `thiserror` for defining error types
- Provide context with errors
- Use `anyhow` for application-level errors
- Document all error variants
- Handle errors appropriately at each level
- Never silently ignore errors

**Example**:
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
    
    #[error("Timeout after {duration:?}")]
    Timeout { duration: Duration },
}
```

### 4. Testing

**Standard**: Write comprehensive tests for all code.

**Guidelines**:
- Write unit tests for all public functions
- Write integration tests for workflows
- Use property-based testing for complex logic
- Test error paths
- Use descriptive test names
- Keep tests fast and independent

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_task_with_valid_input() {
        let task = Task::new("test task");
        let config = Config::default();
        let result = process_task(&task, &config);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, Status::Completed);
    }
    
    #[test]
    fn test_process_task_with_invalid_input() {
        let task = Task::new("");
        let config = Config::default();
        let result = process_task(&task, &config);
        
        assert!(matches!(result, Err(AgentError::TaskError(_))));
    }
}
```

### 5. Code Review

**Standard**: All code must be reviewed before merging.

**Guidelines**:
- Review for correctness and safety
- Review for performance implications
- Review for code quality and style
- Review for documentation completeness
- Review for test coverage
- Provide constructive feedback

### 6. Static Analysis

**Standard**: Use static analysis tools to catch issues early.

**Tools**:
- `cargo clippy` for additional lints
- `cargo audit` for security vulnerabilities
- `cargo outdated` for outdated dependencies
- `cargo deny` for license and dependency checks
- Custom lints for project-specific patterns

**Guidelines**:
- Fix all clippy warnings
- Address security vulnerabilities promptly
- Keep dependencies up to date
- Review license compatibility
- Use `#[deny(warnings)]` in critical modules

---

## Testing and Verification Requirements

### 1. Unit Testing

**Requirement**: All public functions must have unit tests.

**Guidelines**:
- Test all public APIs
- Test edge cases and boundary conditions
- Test error paths
- Use descriptive test names
- Keep tests fast and independent
- Use test fixtures for common setup

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn setup_test_config() -> Config {
        Config {
            timeout: Duration::from_secs(10),
            max_retries: 3,
        }
    }
    
    #[test]
    fn test_process_task_success() {
        let config = setup_test_config();
        let task = Task::new("test task");
        let result = process_task(&task, &config);
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_process_task_timeout() {
        let config = Config {
            timeout: Duration::from_millis(1),
            ..Default::default()
        };
        let task = Task::new("slow task");
        let result = process_task(&task, &config);
        
        assert!(matches!(result, Err(AgentError::Timeout { .. })));
    }
}
```

### 2. Integration Testing

**Requirement**: All major workflows must have integration tests.

**Guidelines**:
- Test end-to-end workflows
- Test interactions between modules
- Use mock services for external dependencies
- Test error scenarios
- Test performance characteristics
- Use realistic test data

**Example**:
```rust
#[tokio::test]
async fn test_full_task_execution() {
    let mut agent = Agent::new(test_config()).await;
    agent.initialize().await.unwrap();
    
    let task = Task::new("integration test task");
    let task_id = agent.submit_task(task).await.unwrap();
    
    // Wait for completion
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    let metrics = agent.get_metrics().await;
    assert_eq!(metrics.tasks_completed, 1);
}
```

### 3. Property-Based Testing

**Requirement**: Complex algorithms must have property-based tests.

**Guidelines**:
- Use `proptest` for property-based testing
- Test invariants and properties
- Test with random inputs
- Shrink failing cases to minimal examples
- Document properties being tested

**Example**:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_roundtrip_encoding(input in any::<Vec<u8>>) {
        let encoded = encode(&input);
        let decoded = decode(&encoded).unwrap();
        prop_assert_eq!(input, decoded);
    }
    
    #[test]
    fn test_sorting_preserves_elements(mut vec in any::<Vec<i32>>()) {
        let original = vec.clone();
        vec.sort();
        prop_assert!(original.iter().all(|x| vec.contains(x)));
    }
}
```

### 4. Performance Testing

**Requirement**: Performance-critical code must have benchmarks.

**Guidelines**:
- Use `criterion` for benchmarking
- Benchmark realistic workloads
- Compare against baselines
- Track performance over time
- Document performance characteristics

**Example**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_process_task(c: &mut Criterion) {
    let config = Config::default();
    let task = Task::new("benchmark task");
    
    c.bench_function("process_task", |b| {
        b.iter(|| process_task(black_box(&task), black_box(&config)))
    });
}

criterion_group!(benches, bench_process_task);
criterion_main!(benches);
```

### 5. Fuzz Testing

**Requirement**: Input parsing and validation must have fuzz tests.

**Guidelines**:
- Use `cargo-fuzz` for fuzz testing
- Fuzz all public APIs that accept external input
- Run fuzz tests in CI
- Fix all crashes found by fuzzing
- Document fuzzing coverage

**Example**:
```rust
// fuzz/fuzz_targets/parse_input.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        let _ = parse_input(input);
    }
});
```

### 6. Verification

**Requirement**: Critical algorithms must be verified.

**Guidelines**:
- Use formal verification tools when possible
- Document invariants and proofs
- Use model checking for state machines
- Review critical code with extra scrutiny
- Consider using verified libraries

---

## Documentation Standards

### 1. Code Documentation

**Standard**: All public APIs must be documented.

**Guidelines**:
- Use `///` for item documentation
- Use `//!` for module documentation
- Include examples in documentation
- Document all parameters and return values
- Document errors that can be returned
- Include performance characteristics

**Example**:
```rust
/// Represents a task to be executed by the agent.
///
/// Tasks are the primary unit of work in the agent system. Each task
/// has a description, priority, and optional context.
///
/// # Examples
///
/// ```
/// use agent::Task;
///
/// let task = Task::new("Add error handling")
///     .with_priority(TaskPriority::High);
/// ```
#[derive(Debug, Clone)]
pub struct Task {
    /// Unique identifier for the task
    pub id: TaskId,
    
    /// Human-readable description of the task
    pub description: String,
    
    /// Priority level for scheduling
    pub priority: TaskPriority,
}
```

### 2. Architecture Documentation

**Standard**: All architectural decisions must be documented.

**Guidelines**:
- Use Architecture Decision Records (ADRs)
- Document the context and problem
- Document the decision and rationale
- Document alternatives considered
- Document consequences and implications
- Keep ADRs up to date

**Example**:
```markdown
# ADR-001: Use Tokio for Async Runtime

## Context
The agent needs to handle many concurrent I/O operations efficiently.

## Decision
Use Tokio as the async runtime for the project.

## Rationale
- Tokio is the de facto standard for async Rust
- Excellent performance and scalability
- Rich ecosystem of compatible crates
- Active maintenance and community support

## Alternatives Considered
- async-std: Less mature ecosystem
- smol: Smaller community, fewer features

## Consequences
- All async code must use Tokio types
- Dependencies must be Tokio-compatible
- Team must learn Tokio-specific patterns
```

### 3. API Documentation

**Standard**: All public APIs must have comprehensive documentation.

**Guidelines**:
- Document all public types, traits, and functions
- Include usage examples
- Document thread safety guarantees
- Document performance characteristics
- Document panicking conditions
- Keep documentation in sync with code

### 4. User Documentation

**Standard**: Provide clear documentation for end users.

**Guidelines**:
- Write clear, concise documentation
- Include getting started guides
- Include tutorials and examples
- Document configuration options
- Document common workflows
- Keep documentation up to date

### 5. Developer Documentation

**Standard**: Provide comprehensive documentation for developers.

**Guidelines**:
- Document development setup
- Document testing procedures
- Document contribution guidelines
- Document code organization
- Document debugging procedures
- Document deployment procedures

---

## References

### Primary Sources

1. **TigerBeetle tiger_style.md**
   - Systems programming best practices
   - Focus on simplicity, performance, and correctness
   - https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/tiger_style.md

2. **NASA's Power of 10**
   - Rules for developing safety-critical code
   - Emphasis on simplicity and verification
   - https://lars-lab.jpl.nasa.gov/JPL_Coding_Standard_C.pdf

3. **Casey Muratori - Performance Aware Programming**
   - Performance optimization principles
   - Emphasis on measurement and understanding
   - https://www.computerenhance.com/

### Additional Resources

4. **Rust API Guidelines**
   - Guidelines for designing Rust APIs
   - https://rust-lang.github.io/api-guidelines/

5. **The Rustonomicon**
   - Unsafe Rust and advanced topics
   - https://doc.rust-lang.org/nomicon/

6. **Rust Performance Book**
   - Performance optimization in Rust
   - https://nnethercote.github.io/perf-book/

7. **Google C++ Style Guide**
   - Additional systems programming guidelines
   - https://google.github.io/styleguide/cppguide.html

8. **Linux Kernel Coding Style**
   - Systems programming conventions
   - https://www.kernel.org/doc/html/latest/process/coding-style.html

---

## Summary

These principles provide a comprehensive foundation for developing safe, performant, and maintainable software. They draw from decades of experience in systems programming, critical systems development, and performance optimization.

**Key Takeaways**:

1. **Safety First**: Design systems that are safe by construction, not by convention
2. **Measure Before Optimizing**: Never optimize without measurements
3. **Simplicity Matters**: Simple code is safer, faster, and easier to maintain
4. **Explicit Over Implicit**: Make behavior visible and predictable
5. **Test Everything**: Comprehensive testing is essential for critical systems
6. **Document Decisions**: Document the why, not just the what
7. **Learn from the Best**: Study and apply principles from proven systems

These principles should guide all development decisions for the Self-Developing Coding Agent, from initial design to implementation and maintenance.
