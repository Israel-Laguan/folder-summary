use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::analyzer::CodeAnalysis;
use crate::error::FolderSummaryError;

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    last_modified: u64,
    analysis: CodeAnalysis,
}

pub struct Cache {
    cache_file: String,
    cache: HashMap<String, CacheEntry>,
}

unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}

impl Cache {
    pub fn new(cache_file: &str) -> Result<Self, FolderSummaryError> {
        let cache = if Path::new(cache_file).exists() {
            let file = File::open(cache_file)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or_else(|_| HashMap::new())
        } else {
            HashMap::new()
        };

        Ok(Cache {
            cache_file: cache_file.to_string(),
            cache,
        })
    }

    pub fn get(&self, file_path: &str) -> Option<&CodeAnalysis> {
        let metadata = std::fs::metadata(file_path).ok()?;
        let last_modified = metadata
            .modified()
            .ok()?
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs();

        self.cache.get(file_path).and_then(|entry| {
            if entry.last_modified == last_modified {
                Some(&entry.analysis)
            } else {
                None
            }
        })
    }

    pub fn set(
        &mut self,
        file_path: String,
        analysis: CodeAnalysis,
    ) -> Result<(), FolderSummaryError> {
        let metadata = std::fs::metadata(&file_path)?;
        let last_modified = metadata
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        self.cache.insert(
            file_path,
            CacheEntry {
                last_modified,
                analysis,
            },
        );
        self.save()
    }

    fn save(&self) -> Result<(), FolderSummaryError> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.cache_file)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &self.cache)?;
        Ok(())
    }
}
