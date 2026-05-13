mod message;
mod broker;
mod agent;
mod sandbox;
mod inference;
mod api;

use std::sync::Arc;
use tokio::sync::Mutex;
use broker::Broker;
use agent::Agent;
use inference::InferenceEngine;
use axum::{Router, routing::post};
use api::{submit_task, AppState};

#[tokio::main]
async fn main() {
    println!("Initializing HiveMind Orchestrator");

    println!("the Neural Inference Engine");
    let ai_engine = InferenceEngine::new().expect("Failed to load AI Engine");
    
    let shared_brain = Arc::new(Mutex::new(ai_engine));

    let broker = Broker::new(100);

    let planner_rx = broker.subscribe();
    let planner = Agent::new("agent-001", "planner", planner_rx, broker.sender.clone(), shared_brain.clone());

    let executor_rx = broker.subscribe();
    let executor = Agent::new("agent-002", "executor", executor_rx, broker.sender.clone(), shared_brain.clone());

    tokio::spawn(planner.run());
    tokio::spawn(executor.run());
    
    let app_state = AppState {
        sender: broker.sender.clone(),
    };

    let app = Router::new()
        .route("/api/v1/task", post(submit_task))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("\nHiveMind API Gateway online at http://localhost:3000/api/v1/task");
    println!("Waiting for incoming web requests...");

    axum::serve(listener, app).await.unwrap();
}


