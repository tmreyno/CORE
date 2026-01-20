// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Search commands for searching within containers
//!
//! Provides commands to search file names and paths within forensic containers
//! (AD1, ZIP, 7z, RAR, TAR, E01/VFS mounted).

use serde::{Deserialize, Serialize};
use tracing::debug;

/// Search result from within a container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerSearchResult {
    /// Container file path
    pub container_path: String,
    /// Container type (ad1, zip, e01, etc.)
    pub container_type: String,
    /// Path within the container
    pub entry_path: String,
    /// Entry name (filename)
    pub name: String,
    /// Whether entry is a directory
    pub is_dir: bool,
    /// File size in bytes
    pub size: u64,
    /// Match score (higher = better match)
    pub score: u32,
    /// Which part matched: "name", "path", or "both"
    pub match_type: String,
}

/// Search options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchOptions {
    /// Case sensitive search
    #[serde(default)]
    pub case_sensitive: bool,
    /// Use regex pattern
    #[serde(default)]
    pub use_regex: bool,
    /// Include directories in results
    #[serde(default = "default_true")]
    pub include_dirs: bool,
    /// Maximum results to return
    #[serde(default = "default_max_results")]
    pub max_results: usize,
}

fn default_true() -> bool { true }
fn default_max_results() -> usize { 500 }

/// Search within a single container for entries matching the query
#[tauri::command]
pub async fn search_container(
    #[allow(non_snake_case)]
    containerPath: String,
    #[allow(non_snake_case)]
    containerType: String,
    query: String,
    options: Option<SearchOptions>,
) -> Result<Vec<ContainerSearchResult>, String> {
    debug!("search_container: path={}, type={}, query={}", containerPath, containerType, query);
    
    let opts = options.unwrap_or_default();
    let query_lower = if opts.case_sensitive { query.clone() } else { query.to_lowercase() };
    
    tauri::async_runtime::spawn_blocking(move || {
        let mut results = Vec::new();
        
        match containerType.to_lowercase().as_str() {
            "ad1" => {
                // Search AD1 container
                match crate::ad1::get_tree(&containerPath) {
                    Ok(entries) => {
                        for entry in entries {
                            if let Some(result) = match_entry(
                                &containerPath, "ad1", &entry.path, &entry.name, 
                                entry.is_dir, entry.size, &query_lower, &opts
                            ) {
                                results.push(result);
                                if results.len() >= opts.max_results { break; }
                            }
                        }
                    }
                    Err(e) => debug!("Failed to search AD1: {}", e),
                }
            }
            "zip" => {
                // Search ZIP archive
                match crate::archive::list_zip_entries(&containerPath) {
                    Ok(entries) => {
                        for entry in entries {
                            let name = std::path::Path::new(&entry.path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(&entry.path)
                                .to_string();
                            if let Some(result) = match_entry(
                                &containerPath, "zip", &entry.path, &name,
                                entry.is_directory, entry.size, &query_lower, &opts
                            ) {
                                results.push(result);
                                if results.len() >= opts.max_results { break; }
                            }
                        }
                    }
                    Err(e) => debug!("Failed to search ZIP: {}", e),
                }
            }
            "7z" => {
                match crate::archive::sevenz::list_entries(&containerPath) {
                    Ok(entries) => {
                        for entry in entries {
                            let name = std::path::Path::new(&entry.path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(&entry.path)
                                .to_string();
                            if let Some(result) = match_entry(
                                &containerPath, "7z", &entry.path, &name,
                                entry.is_directory, entry.size, &query_lower, &opts
                            ) {
                                results.push(result);
                                if results.len() >= opts.max_results { break; }
                            }
                        }
                    }
                    Err(e) => debug!("Failed to search 7z: {}", e),
                }
            }
            "rar" => {
                match crate::archive::rar::list_entries(&containerPath) {
                    Ok(entries) => {
                        for entry in entries {
                            let name = std::path::Path::new(&entry.path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(&entry.path)
                                .to_string();
                            if let Some(result) = match_entry(
                                &containerPath, "rar", &entry.path, &name,
                                entry.is_directory, entry.size, &query_lower, &opts
                            ) {
                                results.push(result);
                                if results.len() >= opts.max_results { break; }
                            }
                        }
                    }
                    Err(e) => debug!("Failed to search RAR: {}", e),
                }
            }
            "tar" | "tgz" | "tar.gz" | "tar.bz2" | "tar.xz" => {
                match crate::archive::tar::list_entries(&containerPath) {
                    Ok(entries) => {
                        for entry in entries {
                            let name = std::path::Path::new(&entry.path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(&entry.path)
                                .to_string();
                            if let Some(result) = match_entry(
                                &containerPath, &containerType, &entry.path, &name,
                                entry.is_directory, entry.size, &query_lower, &opts
                            ) {
                                results.push(result);
                                if results.len() >= opts.max_results { break; }
                            }
                        }
                    }
                    Err(e) => debug!("Failed to search TAR: {}", e),
                }
            }
            _ => {
                debug!("Unsupported container type for search: {}", containerType);
            }
        }
        
        // Sort by score (highest first)
        results.sort_by(|a, b| b.score.cmp(&a.score));
        
        Ok(results)
    })
    .await
    .map_err(|e| format!("Search task failed: {}", e))?
}

/// Search within multiple containers simultaneously
#[tauri::command]
pub async fn search_all_containers(
    containers: Vec<(String, String)>, // Vec of (path, type)
    query: String,
    options: Option<SearchOptions>,
) -> Result<Vec<ContainerSearchResult>, String> {
    debug!("search_all_containers: {} containers, query={}", containers.len(), query);
    
    let opts = options.unwrap_or_default();
    let max_per_container = opts.max_results / containers.len().max(1);
    
    let mut all_results = Vec::new();
    
    for (path, container_type) in containers {
        let container_opts = SearchOptions {
            max_results: max_per_container.max(50), // At least 50 per container
            ..opts.clone()
        };
        
        match search_container(path, container_type, query.clone(), Some(container_opts)).await {
            Ok(results) => all_results.extend(results),
            Err(e) => debug!("Search failed for container: {}", e),
        }
        
        if all_results.len() >= opts.max_results {
            break;
        }
    }
    
    // Sort all results by score
    all_results.sort_by(|a, b| b.score.cmp(&a.score));
    all_results.truncate(opts.max_results);
    
    Ok(all_results)
}

/// Match an entry against the search query
fn match_entry(
    container_path: &str,
    container_type: &str,
    entry_path: &str,
    name: &str,
    is_dir: bool,
    size: u64,
    query: &str,
    opts: &SearchOptions,
) -> Option<ContainerSearchResult> {
    // Skip directories if not included
    if is_dir && !opts.include_dirs {
        return None;
    }
    
    let name_lower = if opts.case_sensitive { name.to_string() } else { name.to_lowercase() };
    let path_lower = if opts.case_sensitive { entry_path.to_string() } else { entry_path.to_lowercase() };
    
    let name_matches = name_lower.contains(query);
    let path_matches = path_lower.contains(query);
    
    if !name_matches && !path_matches {
        return None;
    }
    
    // Calculate score
    let mut score: u32 = 0;
    
    // Exact name match scores highest
    if name_lower == query {
        score += 1000;
    } else if name_lower.starts_with(query) {
        score += 500;
    } else if name_matches {
        score += 200;
    }
    
    // Path match scores lower
    if path_matches && !name_matches {
        score += 50;
    }
    
    // Files score higher than directories
    if !is_dir {
        score += 10;
    }
    
    let match_type = if name_matches && path_matches {
        "both"
    } else if name_matches {
        "name"
    } else {
        "path"
    };
    
    Some(ContainerSearchResult {
        container_path: container_path.to_string(),
        container_type: container_type.to_string(),
        entry_path: entry_path.to_string(),
        name: name.to_string(),
        is_dir,
        size,
        score,
        match_type: match_type.to_string(),
    })
}
