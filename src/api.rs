use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::message::AgentMessage;

#[derive(Deserialize)]
pub struct TaskRequest {
    pub description: String,
    pub target_role: String,
}

#[derive(Serialize)]
pub struct TaskResponse {
    pub status: String,
    pub task_id: String,
    pub message: String,
}

#[derive(Clone)]
pub struct AppState {
    pub sender: broadcast::Sender<AgentMessage>,
}

pub async fn submit_task(
    State(state): State<AppState>,
    Json(payload): Json<TaskRequest>,
) -> Json<TaskResponse> {
    let task_id = Uuid::new_v4();

    let _ = state.sender.send(AgentMessage::NewTask {
        task_id,
        description: payload.description,
        target_role: payload.target_role,
    });

    Json(TaskResponse {
        status: "success".to_string(),
        task_id: task_id.to_string(),
        message: "Task queued for processing".to_string(),
    })
}