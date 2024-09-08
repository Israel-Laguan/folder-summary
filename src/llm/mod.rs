mod gemini;
mod ollama;
mod openai;

pub use gemini::Gemini;
pub use ollama::Ollama;
pub use openai::OpenAI;

use crate::config::Config;
use async_trait::async_trait;
use log::info;
use std::env;
use std::time::Instant;

#[async_trait]
pub trait LLM: Send + Sync {
    async fn summarize(&self, text: &str) -> Result<String, Box<dyn std::error::Error>>;
    fn model_name(&self) -> String;
    fn clone_box(&self) -> Box<dyn LLM>;
}

impl Clone for Box<dyn LLM> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub fn calculate_tokens(text: &str) -> usize {
    // This is a very simple approximation. For more accurate results,
    // you might want to use a proper tokenizer for each model.
    text.split_whitespace().count()
}

pub fn log_performance(
    model: &str,
    start_time: Instant,
    input_tokens: usize,
    output_tokens: usize,
) {
    let duration = start_time.elapsed();
    let total_tokens = input_tokens + output_tokens;
    let tokens_per_second = total_tokens as f64 / duration.as_secs_f64();

    info!(
        "{} - Total duration: {:?}, Input tokens: {}, Output tokens: {}, Total tokens: {}, Tokens per second: {:.2}",
        model, duration, input_tokens, output_tokens, total_tokens, tokens_per_second
    );
}

pub fn get_llm(config: &Config) -> Result<Box<dyn LLM>, Box<dyn std::error::Error>> {
    let llm_provider: Result<String, env::VarError> = env::var("LLM_PROVIDER").or_else(|_| {
        Ok(config
            .llm_provider
            .clone()
            .unwrap_or_else(|| "ollama".to_string()))
    });

    match llm_provider?.as_str() {
        "ollama" => {
            let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| {
                config
                    .ollama_model
                    .clone()
                    .unwrap_or_else(|| "mannix/gemma2-2b".to_string())
            });
            Ok(Box::new(Ollama::new(&model)))
        }
        "gemini" => {
            let api_key = env::var("GEMINI_API_KEY")?;
            let model = env::var("GEMINI_MODEL").unwrap_or_else(|_| {
                config
                    .gemini_model
                    .clone()
                    .unwrap_or_else(|| "gemini-1.5-flash".to_string())
            });
            Ok(Box::new(Gemini::new(&api_key, &model)))
        }
        "openai" => {
            let api_key = env::var("OPENAI_API_KEY")?;
            let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| {
                config
                    .openai_model
                    .clone()
                    .unwrap_or_else(|| "gpt-4o-mini".to_string())
            });
            let url = env::var("CUSTOM_OPENAI_URL").unwrap_or_else(|_| {
                config
                    .custom_openai_url
                    .clone()
                    .unwrap_or_else(|| "https://api.openai.com/v1".to_string())
            });
            Ok(Box::new(OpenAI::new(&api_key, &model, &url)))
        }
        _ => Err("Invalid LLM provider".into()),
    }
}
