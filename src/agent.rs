use tokio::sync::broadcast;
use crate::message::AgentMessage;
use crate::sandbox::SecureSandbox;

pub struct Agent {
    id: String,
    role: String,
    receiver: broadcast::Receiver<AgentMessage>,
    sender: broadcast::Sender<AgentMessage>,
}

impl Agent {
    pub fn new(
        id: &str,
        role: &str,
        receiver: broadcast::Receiver<AgentMessage>,
        sender: broadcast::Sender<AgentMessage>,
    ) -> Self {
        Agent {
            id: id.to_string(),
            role: role.to_string(),
            receiver,
            sender,
        }
    }

    pub async fn run(mut self) {
        println!("Agent [{}] online. Role: {}", self.id, self.role);

        while let Ok(msg) = self.receiver.recv().await {
            match msg {
                AgentMessage::NewTask { task_id, description, target_role } => {
                    if target_role == self.role {
                        println!("Agent [{}] executing: {}", self.id, description);
                        
                        // Idiomatic Rust: assign the result of the if/else directly
                        let final_result = if self.role == "executor" {
                            let sandbox = SecureSandbox::new();
                            match sandbox.execute_math_task(10, 42) {
                                Ok(val) => format!("Sandbox execution successful. Result: {}", val),
                                Err(e) => format!("Sandbox execution failed: {}", e),
                            }
                        } else {
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            format!("Task planned by {}", self.role)
                        }; // <-- Note the semicolon here ending the let statement

                        let _ = self.sender.send(AgentMessage::TaskCompleted {
                            task_id,
                            agent_id: self.id.clone(),
                            result: final_result,
                        });
                    }
                }
                AgentMessage::TaskCompleted { task_id, agent_id, result } => {
                    if agent_id != self.id {
                        println!("Agent [{}] observed completion of {} by {}: {}", 
                            self.id, task_id, agent_id, result);
                    }
                }
                AgentMessage::SystemHalt => {
                    println!("Agent [{}] shutting down.", self.id);
                    break;
                }
            }
        }
    }
}