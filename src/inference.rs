use candle_core::{Device, Tensor};
use candle_transformers::models::quantized_llama::ModelWeights;
use candle_transformers::generation::LogitsProcessor;
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;
use std::path::PathBuf;

pub struct InferenceEngine {
    model: ModelWeights,
    tokenizer: Tokenizer,
    device: Device,
    model_path: PathBuf, 
}

impl InferenceEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("Initializing Candle Inference Engine...");
        let device = Device::Cpu;

        let api = Api::new()?;
        let weights_repo = api.repo(Repo::with_revision(
            "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF".to_string(),
            RepoType::Model,
            "main".to_string(),
        ));
        
        println!("Loading model weights from cache...");
        let model_path = weights_repo.get("tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf")?;

        println!("Loading local tokenizer...");
        let tokenizer = Tokenizer::from_file("tokenizer.json").map_err(|e| e.to_string())?;

        let mut file = std::fs::File::open(&model_path)?;
        let gguf_content = candle_core::quantized::gguf_file::Content::read(&mut file)?;
        let mut tensor_file = std::fs::File::open(&model_path)?;
        let model = ModelWeights::from_gguf(gguf_content, &mut tensor_file, &device)?;

        println!("AI Engine loaded successfully.");

        Ok(Self {
            model,
            tokenizer,
            device,
            model_path, 
        })
    }

    pub fn generate_response(&mut self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        
        let mut file = std::fs::File::open(&self.model_path)?;
        let gguf_content = candle_core::quantized::gguf_file::Content::read(&mut file)?;
        let mut tensor_file = std::fs::File::open(&self.model_path)?;
        self.model = ModelWeights::from_gguf(gguf_content, &mut tensor_file, &self.device)?;

        let tokens = self.tokenizer.encode(prompt, true).map_err(|e| e.to_string())?.get_ids().to_vec();
        
        let mut logits_processor = LogitsProcessor::new(1337, Some(0.1), None);
        let eos_token = 2; 
        let mut generated_tokens = Vec::new();
        let mut prev_text_len = 0;
        let mut generated_text = String::new();
        let mut current_pos = 0;

        println!("\n--- AI Neural Generation Stream ---");

        let mut next_token = 0;
        for (i, &token) in tokens.iter().enumerate() {
            let input_tensor = Tensor::new(&[token], &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input_tensor, current_pos)?;
            current_pos += 1;
            
            if i == tokens.len() - 1 {
                let final_logits = if logits.rank() == 3 {
                    logits.get(0)?.get(0)?
                } else if logits.rank() == 2 {
                    logits.get(0)?
                } else {
                    logits
                }.to_dtype(candle_core::DType::F32)?;
                
                next_token = logits_processor.sample(&final_logits)?;
            }
        }

        for _ in 0..250 {
            if next_token == eos_token {
                break;
            }

            generated_tokens.push(next_token);

            if let Some(full_text) = self.tokenizer.decode(&generated_tokens, true).ok() {
                let new_text = &full_text[prev_text_len..];
                print!("{}", new_text);
                use std::io::Write;
                std::io::stdout().flush().unwrap();
                prev_text_len = full_text.len();
                generated_text = full_text;
            }

            let input_tensor = Tensor::new(&[next_token], &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input_tensor, current_pos)?;
            current_pos += 1;

            let final_logits = if logits.rank() == 3 {
                logits.get(0)?.get(0)?
            } else if logits.rank() == 2 {
                logits.get(0)?
            } else {
                logits
            }.to_dtype(candle_core::DType::F32)?;

            next_token = logits_processor.sample(&final_logits)?;
        }
        
        println!("\n-----------------------------------\n");
        Ok(generated_text)
    }
}