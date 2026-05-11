use uuid::Uuid;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum AgentMessage {
    NewTask {
        task_id: Uuid,
        description: String,
        target_role: String,
    },
    TaskCompleted {
        task_id: Uuid,
        agent_id: String,
        result: String,
    },
    SystemHalt,
}