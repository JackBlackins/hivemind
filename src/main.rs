mod message;
mod broker;
mod agent;
mod sandbox;
mod inference;

use std::sync::Arc;
use tokio::sync::Mutex;
use broker::Broker;
use agent::Agent;
use message::AgentMessage;
use inference::InferenceEngine; // <-- 1. Import the new AI Engine
use uuid::Uuid;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("Initializing HiveMind Orchestrator...");

    println!("Spinning up the Neural Inference Engine...");
    let ai_engine = InferenceEngine::new().expect("Failed to load AI Engine");
    
    // Wrap the engine in a thread-safe, asynchronously lockable Mutex
    let shared_brain = Arc::new(Mutex::new(ai_engine));

    let broker = Broker::new(100);

    let planner_rx = broker.subscribe();
    // Pass shared_brain.clone() into the constructor
    let planner = Agent::new("agent-001", "planner", planner_rx, broker.sender.clone(), shared_brain.clone());

    let executor_rx = broker.subscribe();
    // Pass shared_brain.clone() into the constructor
    let executor = Agent::new("agent-002", "executor", executor_rx, broker.sender.clone(), shared_brain.clone());

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

    println!("Orchestrator is running. Press Ctrl+C to shut down gracefully.");
    
    // This will block the main thread forever until you press Ctrl+C
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            println!("\nShutdown signal received. Halting agents...");
        },
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        },
    }

    // Shut down the system
    let _ = broker.sender.send(AgentMessage::SystemHalt);

    let _ = tokio::join!(planner_handle, executor_handle);
    println!("HiveMind Orchestrator offline.");
}
