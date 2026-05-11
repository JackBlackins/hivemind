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
    ai_engine: Arc<Mutex<InferenceEngine>>,
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

    pub async fn run(mut self) {
        println!("Agent [{}] online. Role: {}", self.id, self.role);

        // Dynamically build the backticks so we don't break the Markdown UI
        let ticks = "`".repeat(3);
        let start_marker = format!("{}wat\n", ticks);
        let end_marker = ticks.clone();

        loop {
            let msg = match self.receiver.recv().await {
                Ok(m) => m,
                Err(_) => continue,
            };

            match msg {
                AgentMessage::NewTask { task_id, description, target_role } => {
                    if target_role == self.role || target_role == "all" {
                        println!("Agent [{}] executing: {}", self.id, description);
                        
                        if self.role == "planner" {
                            println!("Agent [{}] is thinking...", self.id);
                            
                            let mut engine = self.ai_engine.lock().await;
                            
                            // Inject a Few-Shot example so the AI knows what WAT syntax actually looks like
                            let prompt = format!(
                                "<|system|>\nYou are a WebAssembly compiler. Output raw WAT code wrapped in {0}wat blocks. Do not explain the code.\n\nExample Output:\n{0}wat\n(module\n  (func $add (param $a i32) (param $b i32) (result i32)\n    local.get $a\n    local.get $b\n    i32.add)\n  (export \"add\" (func $add))\n)\n{0}</s>\n<|user|>\nWrite a WAT module with an exported function 'add' that adds two i32 numbers. Task: {1}</s>\n<|assistant|>\n", 
                                ticks, description
                            );
                            
                            let ai_thoughts = engine.generate_response(&prompt).unwrap_or_default();

                            let _ = self.sender.send(AgentMessage::TaskCompleted {
                                task_id,
                                agent_id: self.id.clone(),
                                result: ai_thoughts,
                            });
                        }
                    }
                }
                AgentMessage::TaskCompleted { task_id: _, agent_id, result } => {
                    if self.role == "executor" && agent_id == "agent-001" {
                        println!("Agent [{}] intercepted AI code. Initiating secure sandbox...", self.id);
                        
                        // Safely search for the dynamic markers
                        if let Some(start) = result.find(&start_marker) {
                            let code_start = start + start_marker.len();
                            if let Some(end) = result[code_start..].find(&end_marker) {
                                let wat_code = &result[code_start..code_start + end].trim();
                                
                                let sandbox = SecureSandbox::new();
                                match sandbox.execute_dynamic_wat(wat_code, 150, 50) {
                                    Ok(val) => println!("Agent [{}] Sandbox Success! 150 + 50 = {}", self.id, val),
                                    Err(e) => eprintln!("Agent [{}] Sandbox Execution Failed: {}", self.id, e),
                                }
                            }
                        } else {
                            println!("Agent [{}] failed to find valid WebAssembly code in the AI output.", self.id);
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