use agent_core::{Agent, Task, TaskPriority};
use agent_config::AgentConfig;

#[tokio::test]
async fn test_task_delegation() {
    // Load configuration
    let config = AgentConfig::load(None, Default::default()).await.unwrap();

    // Initialize agent (directly using Agent::new since we're in the same crate)
    let mut agent = Agent::new(config);
    
    // Initialize agent (this would normally be done through the initialize_agent function)
    agent.initialize().await.unwrap();

    println!("Agent initialized successfully");

    // Create a parent task
    let parent_task = Task::new("Implement user authentication system")
        .with_priority(TaskPriority::High);
    
    let parent_id = agent.submit_task(parent_task).await.unwrap();
    println!("Created parent task: {}", parent_id);

    // Create subtasks
    let subtask1 = Task::new("Design database schema for users")
        .with_priority(TaskPriority::High);
    let subtask1_id = agent.submit_subtask(parent_id, subtask1).await.unwrap();
    println!("Created subtask 1: {}", subtask1_id);

    let subtask2 = Task::new("Implement user registration API endpoint")
        .with_priority(TaskPriority::High);
    let subtask2_id = agent.submit_subtask(parent_id, subtask2).await.unwrap();
    println!("Created subtask 2: {}", subtask2_id);

    // Check subtasks
    let subtasks = agent.get_subtasks(parent_id).await;
    assert_eq!(subtasks.len(), 2);
    println!("\nParent task has {} subtasks", subtasks.len());
    for subtask_id in &subtasks {
        println!("  - Subtask: {}", subtask_id);
        if let Some(parent) = agent.get_parent_task(*subtask_id).await {
            println!("    Parent task: {}", parent);
            assert_eq!(parent, parent_id);
        }
    }

    // Check completion status
    let completion_status = agent.get_subtask_completion_status(parent_id).await;
    println!("\nSubtask completion status:");
    for (subtask_id, completed) in &completion_status {
        println!("  - Subtask {}: {}", subtask_id, if *completed { "Completed" } else { "In Progress" });
        assert!(!completed); // Should not be completed yet
    }

    assert!(!agent.are_all_subtasks_completed(parent_id).await);

    // Check metrics
    println!("\n=== Current Metrics ===");
    let metrics = agent.get_metrics().await;
    println!("Tasks submitted: {}", metrics.tasks_submitted);
    println!("Tasks completed: {}", metrics.tasks_completed);
    println!("Subtasks submitted: {}", metrics.subtasks_submitted);
    println!("Subtasks completed: {}", metrics.subtasks_completed);
    println!("Success rate: {:.1}%", metrics.success_rate * 100.0);

    assert_eq!(metrics.tasks_submitted, 1);
    assert_eq!(metrics.tasks_completed, 0);
    assert_eq!(metrics.subtasks_submitted, 2);
    assert_eq!(metrics.subtasks_completed, 0);
}
