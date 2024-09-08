use super::LLM;
use super::{calculate_tokens, log_performance};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::Instant;

pub struct Ollama {
    model: String,
    client: Client,
}

impl Clone for Ollama {
    fn clone(&self) -> Self {
        Ollama {
            model: self.model.clone(),
            client: self.client.clone(),
        }
    }
}

impl Ollama {
    pub fn new(model: &str) -> Self {
        Ollama {
            model: model.to_string(),
            client: Client::new(),
        }
    }
    pub fn model_name(&self) -> String {
        format!("Ollama ({})", self.model)
    }
}

#[async_trait]
impl LLM for Ollama {
    fn clone_box(&self) -> Box<dyn LLM> {
        Box::new(self.clone())
    }
    async fn summarize(&self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let input_tokens = calculate_tokens(text);

        let response = self
            .client
            .post("http://localhost:11434/api/generate")
            .json(&json!({
                "model": self.model,
                "prompt": format!("Summarize this function in one line: {}", text),
                "stream": false
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let output = response["response"].as_str().unwrap_or("").to_string();
        let output_tokens = calculate_tokens(&output);

        log_performance(&self.model_name(), start_time, input_tokens, output_tokens);

        Ok(output)
    }
    fn model_name(&self) -> String {
        self.model_name()
    }
}
