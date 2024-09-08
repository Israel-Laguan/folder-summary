use super::LLM;
use super::{calculate_tokens, log_performance};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::Instant;

pub struct OpenAI {
    api_key: String,
    model: String,
    client: Client,
    url: String,
}

impl Clone for OpenAI {
    fn clone(&self) -> Self {
        OpenAI {
            api_key: self.api_key.clone(),
            model: self.model.clone(),
            client: self.client.clone(),
            url: self.url.clone(),
        }
    }
}

impl OpenAI {
    pub fn new(api_key: &str, model: &str, url: &str) -> Self {
        OpenAI {
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: Client::new(),
            url: url.to_string(),
        }
    }
    pub fn model_name(&self) -> String {
        format!("OpenAI ({})", self.model)
    }
}

#[async_trait]
impl LLM for OpenAI {
    fn clone_box(&self) -> Box<dyn LLM> {
        Box::new(self.clone())
    }
    async fn summarize(&self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let input_tokens = calculate_tokens(text);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", self.api_key).parse()?);

        let response = self.client
            .post(format!("{}/chat/completions", self.url))
            .headers(headers)
            .json(&json!({
                "model": self.model,
                "messages": [
                    {"role": "system", "content": "You are a helpful assistant that summarizes functions in one line."},
                    {"role": "user", "content": format!("Summarize this function in one line: {}", text)}
                ]
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let output = response["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let output_tokens = calculate_tokens(&output);

        log_performance(&self.model_name(), start_time, input_tokens, output_tokens);

        Ok(output)
    }
    fn model_name(&self) -> String {
        self.model_name()
    }
}
