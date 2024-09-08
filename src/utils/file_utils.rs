use crate::config::Config;
use ignore::{WalkBuilder, WalkState};
use log::{debug, info};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use toml;
use walkdir::WalkDir;
use globset::{Glob, GlobSetBuilder};

use serde_json;

pub fn collect_documentation_files(dir: &Path) -> Vec<String> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.file_name().to_str().map_or(false, |s| {
                    s.ends_with(".md") || s.ends_with(".txt") || s.ends_with(".rst")
                })
        })
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect()
}

pub fn parse_package_files(dir: &Path) -> HashMap<String, String> {
    let mut package_info = HashMap::new();

    // Parse package.json for npm projects
    if let Ok(contents) = fs::read_to_string(dir.join("package.json")) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
            if let Some(name) = json["name"].as_str() {
                if let Some(version) = json["version"].as_str() {
                    package_info.insert(name.to_string(), version.to_string());
                }
            }
        }
    }

    // Parse Cargo.toml for Rust projects
    if let Ok(contents) = fs::read_to_string(dir.join("Cargo.toml")) {
        if let Ok(toml) = toml::from_str::<toml::Value>(&contents) {
            if let Some(package) = toml.get("package") {
                if let (Some(name), Some(version)) =
                    (package["name"].as_str(), package["version"].as_str())
                {
                    package_info.insert(name.to_string(), version.to_string());
                }
            }
        }
    }

    package_info
}

pub fn collect_code_files(dir: &Path, config: &Config) -> Vec<String> {
    let code_files = Arc::new(Mutex::new(Vec::new()));
    let ignore_patterns = create_ignore_set(config);
    let code_identifiers = config.get_code_identifiers();

    info!("Starting to collect code files from: {:?}", dir);
    debug!("Ignore patterns: {:?}", ignore_patterns);
    debug!("Code identifiers: {:?}", code_identifiers);

    let code_dirs = find_code_directories(dir, &code_identifiers, &ignore_patterns);
    debug!("Found code directories: {:?}", code_dirs);

    for code_dir in code_dirs {
        let ignore_patterns_clone = ignore_patterns.clone(); // Clone inside the loop
        WalkBuilder::new(&code_dir)
            .hidden(false)
            .add_custom_ignore_filename(".gitignore")
            .filter_entry(move |entry| {
                let path = entry.path();
                let should_include = !is_ignored(path, &ignore_patterns_clone); // Use the cloned ignore patterns
                debug!("Checking entry: {:?}, should include: {}", path, should_include);
                should_include
            })
            .build_parallel()
            .run(|| {
                let code_files = Arc::clone(&code_files);
                Box::new(move |entry| {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(e) => {
                            debug!("Error processing entry: {:?}", e);
                            return WalkState::Continue;
                        }
                    };

                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        if let Some(ext) = entry.path().extension() {
                            if ext == "rs" || ext == "js" || ext == "ts" || ext == "py" {
                                let mut code_files = code_files.lock().unwrap();
                                code_files.push(entry.path().to_string_lossy().into_owned());
                                debug!("Added code file: {:?}", entry.path());
                            } else {
                                debug!("Skipping non-code file: {:?}", entry.path());
                            }
                        }
                    }

                    WalkState::Continue
                })
            });
    }

    let collected_files = Arc::try_unwrap(code_files)
        .unwrap()
        .into_inner()
        .unwrap();
    
    info!("Collected {} code files", collected_files.len());
    collected_files
}

fn create_ignore_set(config: &Config) -> globset::GlobSet {
    let mut builder = GlobSetBuilder::new();
    for pattern in config.get_custom_ignore_paths() {
        builder.add(Glob::new(&pattern).expect("Invalid glob pattern"));
    }
    builder.build().expect("Failed to build GlobSet")
}

fn is_ignored(path: &Path, ignore_set: &globset::GlobSet) -> bool {
    ignore_set.is_match(path) || path.components().any(|c| ignore_set.is_match(c.as_os_str()))
}

fn find_code_directories(dir: &Path, code_identifiers: &[String], ignore_set: &globset::GlobSet) -> HashSet<PathBuf> {
    let mut code_dirs = HashSet::new();
    let walker = WalkDir::new(dir).into_iter();
    for entry in walker.filter_entry(|e| !is_ignored(e.path(), ignore_set)) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                debug!("Error accessing entry: {:?}", e);
                continue;
            }
        };
        let path = entry.path();
        if path.is_dir() {
            if code_identifiers.iter().any(|id| path.join(id).exists()) {
                code_dirs.insert(path.to_path_buf());
                debug!("Found code directory: {:?}", path);
            }
        }
    }
    code_dirs
}

pub fn get_project_name(dir: &Path) -> Option<String> {
    // Check for Cargo.toml
    if let Ok(content) = fs::read_to_string(dir.join("Cargo.toml")) {
        if let Ok(toml) = content.parse::<toml::Value>() {
            if let Some(package) = toml.get("package") {
                if let Some(name) = package.get("name") {
                    return Some(name.as_str().unwrap().to_string());
                }
            }
        }
    }

    // Check for package.json
    if let Ok(content) = fs::read_to_string(dir.join("package.json")) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(name) = json.get("name") {
                if let Some(name_str) = name.as_str() {
                    return Some(name_str.to_string());
                }
            }
        }
    }

    None
}
