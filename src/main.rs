mod message;
mod broker;
mod agent;
mod sandbox;

use broker::Broker;
use agent::Agent;
use message::AgentMessage;
use uuid::Uuid;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("Initializing HiveMind Orchestrator...");

    let broker = Broker::new(100);

    let planner_rx = broker.subscribe();
    let planner = Agent::new("agent-001", "planner", planner_rx, broker.sender.clone());

    let executor_rx = broker.subscribe();
    let executor = Agent::new("agent-002", "executor", executor_rx, broker.sender.clone());

    // Spawn agents into isolated threads
    let planner_handle = tokio::spawn(planner.run());
    let executor_handle = tokio::spawn(executor.run());

    sleep(Duration::from_millis(500)).await;

    // Dispatch a task to the planner
    let task_id = Uuid::new_v4();
    println!("Orchestrator dispatching task {}", task_id);
    
    let _ = broker.sender.send(AgentMessage::NewTask {
        task_id,
        description: String::from("Analyze system architecture"),
        target_role: String::from("planner"),
    });

    sleep(Duration::from_secs(4)).await;

    // Shut down the system
    let _ = broker.sender.send(AgentMessage::SystemHalt);

    let _ = tokio::join!(planner_handle, executor_handle);
    println!("HiveMind Orchestrator offline.");
}
