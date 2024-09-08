use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::env;
use chrono::{Local, DateTime};
use std::time::SystemTime;
use toml;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub llm_provider: Option<String>,
    pub ollama_model: Option<String>,
    pub gemini_model: Option<String>,
    pub openai_model: Option<String>,
    pub custom_openai_url: Option<String>,
    pub custom_prompt: Option<String>,
    pub custom_gemini_config: Option<GeminiConfig>,
    pub custom_openai_config: Option<OpenAIConfig>,
    pub summary_output_path: Option<String>,
    pub summary_filename_format: Option<String>,
    pub custom_ignore_paths: Option<Vec<String>>,
    pub code_identifiers: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct GeminiConfig {
    // Add Gemini-specific configuration options here
}

#[derive(Deserialize, Debug)]
pub struct OpenAIConfig {
    // Add OpenAI-specific configuration options here
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
    pub fn get_summary_output_path(&self) -> PathBuf {
        self.summary_output_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                env::var("HOME")
                    .map(PathBuf::from)
                    .expect("Could not find home directory")
                    .join(".local")
                    .join("share")
                    .join("folder_summary")
            })
    }

    pub fn get_summary_filename(&self, folder_name: &str) -> String {
        let now: DateTime<Local> = SystemTime::now().into();
        let date_str = now.format("%Y-%m-%d").to_string();

        self.summary_filename_format
            .as_ref()
            .map(|format| format.replace("{folder}", folder_name).replace("{date}", &date_str))
            .unwrap_or_else(|| format!("summary-{}-{}.md", folder_name, date_str))
    }
    
    pub fn get_custom_ignore_paths(&self) -> Vec<String> {
        let mut ignore_paths = self.custom_ignore_paths.clone().unwrap_or_else(Vec::new);
        ignore_paths.extend(Self::default_ignore_patterns());
        ignore_paths
    }

    pub fn get_code_identifiers(&self) -> Vec<String> {
        self.code_identifiers.clone().unwrap_or_else(|| {
            vec![
                "Cargo.toml".to_string(),
                "package.json".to_string(),
                "setup.py".to_string(),
                "requirements.txt".to_string(),
            ]
        })
    }

    fn default_ignore_patterns() -> Vec<String> {
        vec![
            "node_modules".to_string(),
            "target".to_string(),
            "dist".to_string(),
            "build".to_string(),
            ".git".to_string(),
            ".*ignore".to_string(),
            "*.log".to_string(),
            "*.tmp".to_string(),
            "*.temp".to_string(),
            "*.swp".to_string(),
            "*.bak".to_string(),
        ]
    }
}
