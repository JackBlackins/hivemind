use tokio::sync::broadcast;
use crate::message::AgentMessage;
use crate::sandbox::SecureSandbox;
use crate::inference::InferenceEngine;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Agent {
    id: String,
    role: String,
    receiver: broadcast::Receiver<AgentMessage>,
    sender: broadcast::Sender<AgentMessage>,
    ai_engine: Arc<Mutex<InferenceEngine>>, // Gives the agent a piece of the shared brain
}

impl Agent {
    pub fn new(
        id: &str,
        role: &str,
        receiver: broadcast::Receiver<AgentMessage>,
        sender: broadcast::Sender<AgentMessage>,
        ai_engine: Arc<Mutex<InferenceEngine>>,
    ) -> Self {
        Agent {
            id: id.to_string(),
            role: role.to_string(),
            receiver,
            sender,
            ai_engine,
        }
    }
// ... rest of the file continues below

    pub async fn run(mut self) {
        println!("Agent [{}] online. Role: {}", self.id, self.role);

        while let Ok(msg) = self.receiver.recv().await {
            match msg {
                AgentMessage::NewTask { task_id, description, target_role } => {
                    if target_role == self.role {
                        println!("Agent [{}] executing: {}", self.id, description);
                        
                        if self.role == "planner" {
                            println!("Agent [{}] is thinking...", self.id);
                            
                            // 1. Query the AI Engine
                            let mut engine = self.ai_engine.lock().await;
                            let prompt = format!("Write a WebAssembly Text (WAT) module that adds two numbers. Task: {}", description);
                            
                            // 2. We inject a valid WAT string into the AI's response to simulate 
                            // a successful LLM code generation phase.
                            let _ai_thoughts = engine.generate_response(&prompt).unwrap_or_default();
                            
                            let simulated_ai_output = "Here is the code to solve the problem:\n```wat\n(module\n  (func $add (param $a i32) (param $b i32) (result i32)\n    local.get $a\n    local.get $b\n    i32.add)\n  (export \"add\" (func $add))\n)\n```\nExecution ready.";

                            let _ = self.sender.send(AgentMessage::TaskCompleted {
                                task_id,
                                agent_id: self.id.clone(),
                                result: simulated_ai_output.to_string(),
                            });
                        }
                    }
                }
                AgentMessage::TaskCompleted { task_id, agent_id, result } => {
                    // If the Executor sees that the Planner finished a task, it goes to work!
                    if self.role == "executor" && agent_id == "agent-001" {
                        println!("Agent [{}] intercepted AI code. Initiating secure sandbox...", self.id);
                        
                        // 1. Extract the code between ```wat and ```
                        if let Some(start) = result.find("```wat\n") {
                            let code_start = start + 7;
                            if let Some(end) = result[code_start..].find("```") {
                                let wat_code = &result[code_start..code_start + end].trim();
                                
                                // 2. Spin up the sandbox and execute the AI's code!
                                let sandbox = SecureSandbox::new();
                                match sandbox.execute_dynamic_wat(wat_code, 150, 50) {
                                    Ok(val) => println!("Agent [{}] Sandbox Success! 150 + 50 = {}", self.id, val),
                                    Err(e) => eprintln!("Agent [{}] Sandbox Execution Failed: {}", self.id, e),
                                }
                            }
                        }
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