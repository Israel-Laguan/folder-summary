mod javascript_analyzer;
mod python_analyzer;
mod rust_analyzer;
mod static_analysis;

pub use javascript_analyzer::JavaScriptAnalyzer;
pub use python_analyzer::PythonAnalyzer;
pub use rust_analyzer::RustAnalyzer;

use crate::cache::Cache;
use crate::error::FolderSummaryError;
use crate::llm::LLM;
use async_trait::async_trait;
use futures::future::join_all;
use indicatif::ProgressBar;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use tokio::task;

use crate::analyzer::static_analysis::FunctionAnalysis;

pub type ThreadSafeCache = Arc<Mutex<Cache>>;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeAnalysis {
    pub imports: Vec<String>,
    pub functions: Vec<FunctionAnalysis>,
    pub types: Vec<String>,
    pub exports: Vec<String>,
}

#[async_trait]
pub trait LanguageAnalyzer: Send + Sync {
    fn can_analyze(&self, file_path: &str) -> bool;
    fn analyze(&self, content: &str) -> Result<CodeAnalysis, FolderSummaryError>;
    async fn summarize(
        &self,
        analysis: &CodeAnalysis,
        llm: &Box<dyn LLM>,
    ) -> Result<CodeAnalysis, FolderSummaryError>;
}

pub fn get_analyzers() -> Vec<Box<dyn LanguageAnalyzer>> {
    vec![
        Box::new(RustAnalyzer),
        Box::new(JavaScriptAnalyzer),
        Box::new(PythonAnalyzer),
    ]
}

pub async fn analyze_code_files(
    files: &[String],
    llm: &Box<dyn LLM>,
    pb: &ProgressBar,
    cache: &ThreadSafeCache,
) -> Result<HashMap<String, CodeAnalysis>, FolderSummaryError> {
    let analysis_futures: Vec<_> = files
        .iter()
        .map(|file| {
            let file = file.clone();
            let llm = llm.clone();
            let pb = pb.clone();
            let cache = cache.clone();

            task::spawn(async move {
                let cached_analysis = {
                    let cache_lock = cache.lock().map_err(|_| {
                        FolderSummaryError::CacheError("Failed to acquire cache lock".to_string())
                    })?;
                    cache_lock.get(&file).cloned()
                };

                let analysis = if let Some(cached) = cached_analysis {
                    cached
                } else {
                    let new_analysis = analyze_file(&file, &llm).await?;
                    let mut cache_lock = cache.lock().map_err(|_| {
                        FolderSummaryError::CacheError("Failed to acquire cache lock".to_string())
                    })?;
                    cache_lock.set(file.clone(), new_analysis.clone())?;
                    new_analysis
                };

                pb.inc(1);
                Ok::<_, FolderSummaryError>((file, analysis))
            })
        })
        .collect();

    let results: Vec<Result<(String, CodeAnalysis), FolderSummaryError>> =
        join_all(analysis_futures)
            .await
            .into_iter()
            .map(|res| {
                res.map_err(|e| FolderSummaryError::TaskJoinError(e.to_string()))
                    .and_then(|inner| inner)
            })
            .collect();

    results.into_iter().collect()
}

pub async fn analyze_file(
    file_path: &str,
    llm: &Box<dyn LLM>,
) -> Result<CodeAnalysis, FolderSummaryError> {
    let analyzers = get_analyzers();
    for analyzer in analyzers {
        if analyzer.can_analyze(file_path) {
            let content =
                fs::read_to_string(file_path).map_err(|e| FolderSummaryError::IoError(e))?;
            let mut analysis = analyzer.analyze(&content)?;
            analysis = analyzer.summarize(&analysis, llm).await?;
            return Ok(analysis);
        }
    }
    Err(FolderSummaryError::AnalysisError(format!(
        "No suitable analyzer found for file: {}",
        file_path
    )))
}
