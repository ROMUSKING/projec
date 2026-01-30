use std::time::Duration;
use tokio::time::sleep;
use coding_agent::agent_core::{Agent, Task, TaskPriority, TaskContext};
use coding_agent::agent_config::AgentConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = AgentConfig::load(None, Default::default()).await?;

    // Initialize agent
    let mut agent = coding_agent::initialize_agent(config).await?;

    println!("Agent initialized successfully");

    // Example 1: Create a parent task with subtasks
    println!("\n=== Example 1: Parent task with subtasks ===");
    
    let parent_task = Task::new("Implement user authentication system")
        .with_priority(TaskPriority::High);
    
    let parent_id = agent.submit_task(parent_task).await?;
    println!("Created parent task: {}", parent_id);

    // Create subtasks
    let subtask1 = Task::new("Design database schema for users")
        .with_priority(TaskPriority::High);
    let subtask1_id = agent.submit_subtask(parent_id, subtask1).await?;
    println!("Created subtask 1: {}", subtask1_id);

    let subtask2 = Task::new("Implement user registration API endpoint")
        .with_priority(TaskPriority::High);
    let subtask2_id = agent.submit_subtask(parent_id, subtask2).await?;
    println!("Created subtask 2: {}", subtask2_id);

    let subtask3 = Task::new("Implement login API endpoint")
        .with_priority(TaskPriority::High);
    let subtask3_id = agent.submit_subtask(parent_id, subtask3).await?;
    println!("Created subtask 3: {}", subtask3_id);

    // Check subtasks
    let subtasks = agent.get_subtasks(parent_id).await;
    println!("\nParent task has {} subtasks", subtasks.len());
    for subtask_id in &subtasks {
        println!("  - Subtask: {}", subtask_id);
        if let Some(parent) = agent.get_parent_task(*subtask_id).await {
            println!("    Parent task: {}", parent);
        }
    }

    // Check completion status
    let completion_status = agent.get_subtask_completion_status(parent_id).await;
    println!("\nSubtask completion status:");
    for (subtask_id, completed) in &completion_status {
        println!("  - Subtask {}: {}", subtask_id, if *completed { "Completed" } else { "In Progress" });
    }

    println!("\nAll subtasks completed: {}", agent.are_all_subtasks_completed(parent_id).await);

    // Example 2: Create a task with nested subtasks
    println!("\n=== Example 2: Nested subtasks ===");
    
    let grandparent_task = Task::new("Build e-commerce platform")
        .with_priority(TaskPriority::Critical);
    
    let grandparent_id = agent.submit_task(grandparent_task).await?;
    println!("Created grandparent task: {}", grandparent_id);

    let parent_task2 = Task::new("Implement product catalog system")
        .with_priority(TaskPriority::High);
    let parent2_id = agent.submit_subtask(grandparent_id, parent_task2).await?;
    println!("Created parent task: {}", parent2_id);

    let subtask4 = Task::new("Create product database tables")
        .with_priority(TaskPriority::High);
    let subtask4_id = agent.submit_subtask(parent2_id, subtask4).await?;
    println!("Created nested subtask: {}", subtask4_id);

    let subtask5 = Task::new("Implement product search API")
        .with_priority(TaskPriority::Normal);
    let subtask5_id = agent.submit_subtask(parent2_id, subtask5).await?;
    println!("Created nested subtask: {}", subtask5_id);

    println!("\nParent task {} has {} subtasks", parent2_id, agent.get_subtasks(parent2_id).await.len());
    println!("Grandparent task {} has {} subtasks", grandparent_id, agent.get_subtasks(grandparent_id).await.len());

    // Check metrics
    println!("\n=== Current Metrics ===");
    let metrics = agent.get_metrics().await;
    println!("Tasks submitted: {}", metrics.tasks_submitted);
    println!("Tasks completed: {}", metrics.tasks_completed);
    println!("Subtasks submitted: {}", metrics.subtasks_submitted);
    println!("Subtasks completed: {}", metrics.subtasks_completed);
    println!("Success rate: {:.1}%", metrics.success_rate * 100.0);

    Ok(())
}
