use std::fmt;
use std::time::SystemTimeError;

#[derive(Debug)]
pub enum FolderSummaryError {
    IoError(std::io::Error),
    ConfigError(String),
    LlmError(String),
    AnalysisError(String),
    CacheError(String),
    TaskJoinError(String),
}

impl std::error::Error for FolderSummaryError {}

impl fmt::Display for FolderSummaryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FolderSummaryError::IoError(err) => write!(f, "IO error: {}", err),
            FolderSummaryError::ConfigError(err) => write!(f, "Configuration error: {}", err),
            FolderSummaryError::LlmError(err) => write!(f, "LLM error: {}", err),
            FolderSummaryError::AnalysisError(err) => write!(f, "Analysis error: {}", err),
            FolderSummaryError::CacheError(err) => write!(f, "Cache error: {}", err),
            FolderSummaryError::TaskJoinError(err) => write!(f, "TaskJoin error: {}", err),
        }
    }
}

impl From<serde_json::Error> for FolderSummaryError {
    fn from(err: serde_json::Error) -> Self {
        FolderSummaryError::TaskJoinError(err.to_string())
    }
}

impl From<std::io::Error> for FolderSummaryError {
    fn from(err: std::io::Error) -> Self {
        FolderSummaryError::IoError(err)
    }
}

impl From<SystemTimeError> for FolderSummaryError {
    fn from(error: SystemTimeError) -> Self {
        FolderSummaryError::IoError(std::io::Error::new(std::io::ErrorKind::Other, error))
    }
}
// Add more From implementations as needed
