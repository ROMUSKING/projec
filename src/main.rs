use anyhow::Result;
use clap::Parser;
use tracing::{info, warn, error};

/// Self-developing coding agent
#[derive(Parser, Debug)]
#[command(name = "coding-agent")]
#[command(about = "A self-developing coding agent built in Rust")]
#[command(version)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    /// Working directory
    #[arg(short, long, value_name = "DIR")]
    workspace: Option<String>,

    /// Task to execute (if not provided, starts interactive mode)
    #[arg(value_name = "TASK")]
    task: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Start in daemon mode (continuous operation)
    #[arg(short, long)]
    daemon: bool,

    /// Trigger self-improvement cycle
    #[arg(long)]
    improve: bool,

    /// Show agent metrics
    #[arg(long)]
    metrics: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            if cli.verbose {
                "debug"
            } else {
                "info"
            }
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true);

    subscriber.init();

    info!("Starting coding agent v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = load_config(&cli).await?;
    info!("Configuration loaded successfully");

    // Show metrics if requested
    if cli.metrics {
        show_metrics(&config).await?;
        return Ok(());
    }

    // Initialize agent
    let mut agent = initialize_agent(config).await?;
    info!("Agent initialized successfully");

    // Handle different modes
    if cli.improve {
        info!("Triggering self-improvement cycle");
        agent.trigger_self_improvement().await?;
        
        // Run briefly to allow improvement to execute
        let run_future = agent.run();
        let timeout = tokio::time::Duration::from_secs(60);
        
        match tokio::time::timeout(timeout, run_future).await {
            Ok(result) => result?,
            Err(_) => {
                info!("Improvement cycle timeout reached, shutting down");
                agent.shutdown().await?;
            }
        }
    } else if cli.daemon {
        info!("Starting in daemon mode");
        run_daemon_mode(&mut agent).await?;
    } else if let Some(task_description) = cli.task {
        info!("Executing single task: {}", task_description);
        run_single_task(&mut agent, &task_description, cli.workspace).await?;
    } else {
        info!("Starting interactive mode");
        run_interactive_mode(agent).await?;
    }

    info!("Coding agent shutting down");
    Ok(())
}

/// Load configuration from file or use defaults
async fn load_config(cli: &Cli) -> Result<agent_config::AgentConfig> {
    let config_path = cli.config.as_ref().map(std::path::PathBuf::from);
    
    let overrides = agent_config::ConfigOverrides {
        log_level: if cli.verbose { Some("debug".to_string()) } else { None },
        workspace: cli.workspace.as_ref().map(std::path::PathBuf::from),
        llm_provider: None,
        llm_model: None,
        llm_api_key: None,
        llm_base_url: None,
        llm_temperature: None,
        llm_max_tokens: None,
        improvement_interval: None,
        max_concurrent_tasks: None,
        self_improvement_enabled: None,
        self_improvement_auto_apply: None,
        self_improvement_safety_checks: None,
        lsp_enabled: None,
        lsp_timeout: None,
        safety_max_file_size_mb: None,
        git_enabled: None,
        git_auto_commit: None,
        test_enabled: None,
        test_auto_run: None,
        test_framework: None,
        self_compile_enabled: None,
        self_compile_auto_restart: None,
    };

    let config = agent_config::AgentConfig::load(config_path, overrides).await?;
    Ok(config)
}

/// Initialize the agent with all modules
async fn initialize_agent(config: agent_config::AgentConfig) -> Result<agent_core::Agent> {
    use agent_core::*;
    use std::sync::Arc;

    // Create the agent
    let mut agent = Agent::new(config.clone());

    // Create and configure the intelligence engine
    let gateway_factory = intelligence::gateway::GatewayFactory::new();
    let gateway = gateway_factory.create(
        &config.llm.provider,
        Some(config.current_api_key().to_string()),
        config.llm.model.clone(),
    )?;
    let intelligence_engine = Arc::new(intelligence::IntelligenceEngine::new(gateway));

    // Create and configure the analysis engine
    let analysis_engine = Arc::new(analysis::AnalysisEngine::new());

    // Create and configure the knowledge engine
    let knowledge_engine = Arc::new(knowledge::KnowledgeEngine::new());

    // Create and configure the tool framework
    let tools_framework = Arc::new(tools::ToolFramework::new());

    // Create and configure orchestrator with all engines
    let orchestrator = orchestrator::Orchestrator::new()
        .with_config(config.clone())
        .with_intelligence(intelligence_engine)
        .with_analysis(analysis_engine)
        .with_knowledge(knowledge_engine)
        .with_tools(tools_framework);

    agent = agent.with_orchestrator(orchestrator);

    info!("Agent initialization complete");
    
    // Initialize the agent
    agent.initialize().await?;
    
    Ok(agent)
}

/// Run the agent in daemon mode (continuous operation)
async fn run_daemon_mode(agent: &mut agent_core::Agent) -> Result<()> {
    info!("Daemon mode started - Agent will run continuously");
    
    // Set up signal handlers for graceful shutdown
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;

    // Run the agent in a separate task
    let agent_future = agent.run();

    tokio::select! {
        result = agent_future => {
            result?;
        }
        _ = sigterm.recv() => {
            info!("SIGTERM received, initiating graceful shutdown");
            agent.shutdown().await?;
        }
        _ = sigint.recv() => {
            info!("SIGINT received, initiating graceful shutdown");
            agent.shutdown().await?;
        }
    }

    Ok(())
}

/// Run a single task and exit
async fn run_single_task(
    agent: &mut agent_core::Agent,
    task_description: &str,
    workspace: Option<String>,
) -> Result<()> {
    use agent_core::*;

    // Create the task
    let mut task = Task::new(task_description)
        .with_priority(TaskPriority::High);

    // Set workspace if provided
    if let Some(workspace_path) = workspace {
        let mut context = TaskContext::default();
        context.workspace_path = Some(workspace_path.into());
        task = task.with_context(context);
    }

    // Submit the task
    let task_id = agent.submit_task(task).await?;
    info!("Task submitted with ID: {}", task_id);

    // Run the agent briefly to process the task
    let run_future = agent.run();
    let timeout = tokio::time::Duration::from_secs(300); // 5 minute timeout

    match tokio::time::timeout(timeout, run_future).await {
        Ok(result) => result?,
        Err(_) => {
            warn!("Task execution timeout reached");
            agent.shutdown().await?;
        }
    }

    // Show final metrics
    let metrics = agent.get_metrics().await;
    info!("Task execution complete");
    info!("Success rate: {:.1}%", metrics.success_rate * 100.0);
    info!("Tasks completed: {}", metrics.tasks_completed);

    Ok(())
}

/// Run interactive mode with REPL
async fn run_interactive_mode(mut agent: agent_core::Agent) -> Result<()> {
    use std::io::{self, Write};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    info!("Interactive mode started - Type 'help' for commands, 'exit' to quit");

    // Wrap agent in Arc and Mutex to allow sharing between tasks
    let agent_arc = Arc::new(Mutex::new(agent));
    let agent_clone = Arc::clone(&agent_arc);
    
    // Run the agent in a background task
    let agent_handle = tokio::spawn(async move {
        let mut agent = agent_clone.lock().await;
        if let Err(e) = agent.run().await {
            error!("Agent error: {}", e);
        }
    });

    // REPL loop for user input
    loop {
        print!("agent> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "exit" | "quit" => {
                info!("Exiting interactive mode");
                break;
            }
            "help" => {
                print_help();
            }
            "status" => {
                let agent = agent_arc.lock().await;
                let state = agent.current_state().await;
                println!("Current state: {}", state);
            }
            "metrics" => {
                let agent = agent_arc.lock().await;
                let metrics = agent.get_metrics().await;
                print_metrics(&metrics);
            }
            "improve" => {
                let agent = agent_arc.lock().await;
                info!("Triggering self-improvement cycle");
                agent.trigger_self_improvement().await?;
            }
            "" => {
                // Empty line, do nothing
            }
            _ => {
                // Treat as a task
                use agent_core::*;
                let task = Task::new(input).with_priority(TaskPriority::Normal);
                let agent = agent_arc.lock().await;
                match agent.submit_task(task).await {
                    Ok(id) => println!("Task submitted: {}", id),
                    Err(e) => eprintln!("Failed to submit task: {}", e),
                }
            }
        }
    }

    // Stop the agent and wait for completion
    let mut agent = agent_arc.lock().await;
    agent.shutdown().await?;
    agent_handle.abort();

    Ok(())
}

/// Print help message
fn print_help() {
    println!("Available commands:");
    println!("  help     - Show this help message");
    println!("  status   - Show current agent state");
    println!("  metrics  - Show agent performance metrics");
    println!("  improve  - Trigger self-improvement cycle");
    println!("  exit     - Exit interactive mode");
    println!();
    println!("Any other input will be treated as a task description.");
}

/// Print metrics in a formatted way
fn print_metrics(metrics: &agent_core::AgentMetrics) {
    println!("\n=== Agent Metrics ===");
    println!("Tasks submitted:   {}", metrics.tasks_submitted);
    println!("Tasks completed:   {}", metrics.tasks_completed);
    println!("Tasks failed:      {}", metrics.tasks_failed);
    println!("Success rate:      {:.1}%", metrics.success_rate * 100.0);
    println!("Avg execution:     {}ms", metrics.average_execution_time_ms);
    println!("Total exec time:   {}ms", metrics.total_execution_time_ms);
    println!("Uptime:            {}s", metrics.uptime_seconds());
    println!("Improvements:      {} applied, {} rolled back",
        metrics.improvements_applied,
        metrics.improvements_rolled_back
    );
    println!("====================\n");
}

/// Show metrics and exit
async fn show_metrics(_config: &agent_config::AgentConfig) -> Result<()> {
    // In a real implementation, this would load persisted metrics
    println!("Agent Metrics");
    println!("=============");
    println!("No persisted metrics available.");
    println!("Run the agent to generate metrics.");
    Ok(())
}
