#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AgentMessage {
    NewTask {
        task_id: uuid::Uuid,
        description: String,
        target_role: String,
    },
    TaskCompleted {
        task_id: uuid::Uuid,
        agent_id: String,
        task_description: String,
        result: String,
    },
    SystemHalt,
}