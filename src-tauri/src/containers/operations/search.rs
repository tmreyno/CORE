// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Search operations for forensic containers
//!
//! Provides unified search across AD1, L01, Archive, and UFED containers.
//! Disk image containers (E01, Raw) do not have file tree structures and
//! are not searchable.

use tracing::debug;

use crate::ad1;
use crate::archive;
use crate::ufed;

use crate::containers::types::{ContainerKind, SearchQuery, ContainerSearchResult, SearchMatchType};

use super::detect_container;

/// Search for files within a container
/// 
/// This unified search API works across all container types that support
/// file trees (AD1, L01, Archive). For disk image containers (E01, Raw),
/// this function will return an error as they don't have file tree structures.
/// 
/// # Arguments
/// * `path` - Path to the container file
/// * `query` - Search query specifying what to find
/// 
/// # Returns
/// A vector of search results matching the query
/// 
/// # Example
/// ```rust,ignore
/// use crate::containers::{search, SearchQuery};
/// 
/// // Find all .exe files
/// let query = SearchQuery::new().with_extension("exe");
/// let results = search("/evidence/disk.ad1", query)?;
/// 
/// // Find files by name pattern
/// let query = SearchQuery::new().with_name("*.log");
/// let results = search("/evidence/disk.ad1", query)?;
/// ```
pub fn search(path: &str, query: SearchQuery) -> Result<Vec<ContainerSearchResult>, String> {
    debug!(path = %path, "Searching container");
    
    let kind = detect_container(path)?;
    
    match kind {
        ContainerKind::Ad1 => search_ad1(path, &query),
        ContainerKind::L01 => search_l01(path, &query),
        ContainerKind::Archive => search_archive(path, &query),
        ContainerKind::E01 => Err("E01 containers are disk images without file tree structure. Use filesystem tools after extraction.".to_string()),
        ContainerKind::Raw => Err("Raw containers are disk images without file tree structure. Use filesystem tools after extraction.".to_string()),
        ContainerKind::Ufed => search_ufed(path, &query),
    }
}

/// Search within an AD1 container
fn search_ad1(path: &str, query: &SearchQuery) -> Result<Vec<ContainerSearchResult>, String> {
    let mut results = Vec::new();
    
    // Search by name pattern
    if let Some(ref pattern) = query.name_pattern {
        let matches = ad1::find_by_name(path, pattern).map_err(|e| e.to_string())?;
        for m in matches {
            if should_include_ad1(&m.entry, query) {
                // Extract name from path
                let name = std::path::Path::new(&m.entry.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| m.entry.path.clone());
                
                results.push(ContainerSearchResult {
                    container_path: path.to_string(),
                    container_type: "AD1".to_string(),
                    entry_path: m.entry.path.clone(),
                    name,
                    is_directory: m.entry.is_dir,
                    size: m.entry.size,
                    match_type: SearchMatchType::Name,
                    depth: m.depth,
                    md5: m.entry.md5_hash.clone(),
                    sha1: m.entry.sha1_hash.clone(),
                    created: m.entry.created.clone(),
                    modified: m.entry.modified.clone(),
                });
            }
        }
    }
    
    // Search by extension
    if let Some(ref ext) = query.extension {
        let matches = ad1::find_by_extension(path, ext).map_err(|e| e.to_string())?;
        for m in matches {
            // Avoid duplicates if both name and extension are specified
            if query.name_pattern.is_some() && results.iter().any(|r| r.entry_path == m.entry.path) {
                continue;
            }
            if should_include_ad1(&m.entry, query) {
                let name = std::path::Path::new(&m.entry.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| m.entry.path.clone());
                
                results.push(ContainerSearchResult {
                    container_path: path.to_string(),
                    container_type: "AD1".to_string(),
                    entry_path: m.entry.path.clone(),
                    name,
                    is_directory: m.entry.is_dir,
                    size: m.entry.size,
                    match_type: SearchMatchType::Extension,
                    depth: m.depth,
                    md5: m.entry.md5_hash.clone(),
                    sha1: m.entry.sha1_hash.clone(),
                    created: m.entry.created.clone(),
                    modified: m.entry.modified.clone(),
                });
            }
        }
    }
    
    // Search by hash
    if let Some(ref hash) = query.hash {
        if let Some(found) = ad1::find_by_hash(path, hash).map_err(|e| e.to_string())? {
            if should_include_ad1(&found.entry, query) {
                let name = std::path::Path::new(&found.entry.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| found.entry.path.clone());
                
                results.push(ContainerSearchResult {
                    container_path: path.to_string(),
                    container_type: "AD1".to_string(),
                    entry_path: found.entry.path.clone(),
                    name,
                    is_directory: found.entry.is_dir,
                    size: found.entry.size,
                    match_type: SearchMatchType::Hash,
                    depth: found.depth,
                    md5: found.entry.md5_hash.clone(),
                    sha1: found.entry.sha1_hash.clone(),
                    created: found.entry.created.clone(),
                    modified: found.entry.modified.clone(),
                });
            }
        }
    }
    
    // Limit results if specified
    if let Some(max) = query.max_results {
        results.truncate(max);
    }
    
    Ok(results)
}

/// Search within an L01 container (logical EWF)
fn search_l01(path: &str, query: &SearchQuery) -> Result<Vec<ContainerSearchResult>, String> {
    // L01 containers store logical evidence as a single data stream.
    // Get EWF info to report what we know about the container.
    let info = crate::ewf::info(path).map_err(|e| e.to_string())?;
    
    let container_name = std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    
    // Extract stored hashes
    let stored_md5 = info.stored_hashes.iter()
        .find(|h| h.algorithm.to_uppercase().contains("MD5"))
        .map(|h| h.hash.clone());
    let stored_sha1 = info.stored_hashes.iter()
        .find(|h| h.algorithm.to_uppercase().contains("SHA"))
        .map(|h| h.hash.clone());
    
    let mut results = Vec::new();
    
    // Report the container itself as a searchable entry
    if let Some(ref pattern) = query.name_pattern {
        if matches_pattern(&container_name, pattern) {
            results.push(ContainerSearchResult {
                container_path: path.to_string(),
                container_type: "L01".to_string(),
                entry_path: "/".to_string(),
                name: container_name.clone(),
                is_directory: false,
                size: info.total_size,
                match_type: SearchMatchType::Name,
                depth: 0,
                md5: stored_md5.clone(),
                sha1: stored_sha1.clone(),
                created: None,
                modified: None,
            });
        }
    }
    
    // Search by hash if stored hashes are available
    if let Some(ref hash) = query.hash {
        let lower_hash = hash.to_lowercase();
        let md5_match = stored_md5.as_ref().map(|h| h.to_lowercase() == lower_hash).unwrap_or(false);
        let sha1_match = stored_sha1.as_ref().map(|h| h.to_lowercase() == lower_hash).unwrap_or(false);
        
        if md5_match || sha1_match {
            results.push(ContainerSearchResult {
                container_path: path.to_string(),
                container_type: "L01".to_string(),
                entry_path: "/".to_string(),
                name: container_name,
                is_directory: false,
                size: info.total_size,
                match_type: SearchMatchType::Hash,
                depth: 0,
                md5: stored_md5,
                sha1: stored_sha1,
                created: None,
                modified: None,
            });
        }
    }
    
    if let Some(max) = query.max_results {
        results.truncate(max);
    }
    
    Ok(results)
}

/// Search within an archive container
fn search_archive(path: &str, query: &SearchQuery) -> Result<Vec<ContainerSearchResult>, String> {
    let entries = archive::list_zip_entries(path)?;
    let mut results = Vec::new();
    
    for entry in entries {
        let name = std::path::Path::new(&entry.path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| entry.path.clone());
        
        let matches = check_entry_matches(&name, &entry.path, entry.size, entry.is_directory, query);
        
        if let Some(match_type) = matches {
            results.push(ContainerSearchResult {
                container_path: path.to_string(),
                container_type: "Archive".to_string(),
                entry_path: entry.path.clone(),
                name,
                is_directory: entry.is_directory,
                size: entry.size,
                match_type,
                depth: entry.path.matches('/').count() as u32,
                md5: None,
                sha1: None,
                created: None,
                modified: Some(entry.last_modified),
            });
            
            // Check result limit
            if let Some(max) = query.max_results {
                if results.len() >= max {
                    break;
                }
            }
        }
    }
    
    Ok(results)
}

/// Search within a UFED container
fn search_ufed(path: &str, query: &SearchQuery) -> Result<Vec<ContainerSearchResult>, String> {
    // Get root children and recursively search
    let root_children = ufed::get_root_children(path).map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    
    search_ufed_recursive(path, &root_children, query, &mut results, 0)?;
    
    if let Some(max) = query.max_results {
        results.truncate(max);
    }
    
    Ok(results)
}

/// Recursively search UFED entries
fn search_ufed_recursive(
    container_path: &str,
    entries: &[ufed::UfedTreeEntry],
    query: &SearchQuery,
    results: &mut Vec<ContainerSearchResult>,
    depth: u32,
) -> Result<(), String> {
    for entry in entries {
        // Check result limit
        if let Some(max) = query.max_results {
            if results.len() >= max {
                return Ok(());
            }
        }
        
        let matches = check_entry_matches(&entry.name, &entry.path, entry.size, entry.is_dir, query);
        
        if let Some(match_type) = matches {
            results.push(ContainerSearchResult {
                container_path: container_path.to_string(),
                container_type: "UFED".to_string(),
                entry_path: entry.path.clone(),
                name: entry.name.clone(),
                is_directory: entry.is_dir,
                size: entry.size,
                match_type,
                depth,
                md5: None,
                sha1: None,
                created: None,
                modified: None,
            });
        }
        
        // Recurse into directories
        if entry.is_dir {
            if let Ok(children) = ufed::get_children(container_path, &entry.path) {
                search_ufed_recursive(container_path, &children, query, results, depth + 1)?;
            }
        }
    }
    
    Ok(())
}

/// Check if an entry matches the query criteria
fn check_entry_matches(
    name: &str,
    path: &str,
    size: u64,
    is_directory: bool,
    query: &SearchQuery,
) -> Option<SearchMatchType> {
    // Check directory/file filters
    if query.directories_only && !is_directory {
        return None;
    }
    if query.files_only && is_directory {
        return None;
    }
    
    // Check size filters
    if let Some(min) = query.min_size {
        if size < min {
            return None;
        }
    }
    if let Some(max) = query.max_size {
        if size > max {
            return None;
        }
    }
    
    let mut matched = false;
    let mut match_type = SearchMatchType::Name;
    
    // Check name pattern
    if let Some(ref pattern) = query.name_pattern {
        if matches_pattern(name, pattern) || matches_pattern(path, pattern) {
            matched = true;
            match_type = SearchMatchType::Name;
        }
    }
    
    // Check extension
    if let Some(ref ext) = query.extension {
        let lower_name = name.to_lowercase();
        let lower_ext = ext.to_lowercase();
        if lower_name.ends_with(&format!(".{}", lower_ext)) {
            if matched {
                match_type = SearchMatchType::Multiple;
            } else {
                matched = true;
                match_type = SearchMatchType::Extension;
            }
        }
    }
    
    // If no search criteria specified, match all (filtered by dir/file/size above)
    if query.name_pattern.is_none() && query.extension.is_none() && query.hash.is_none() {
        return Some(SearchMatchType::Name);
    }
    
    if matched {
        Some(match_type)
    } else {
        None
    }
}

/// Simple wildcard pattern matching (* and ?)
fn matches_pattern(text: &str, pattern: &str) -> bool {
    let lower_text = text.to_lowercase();
    let lower_pattern = pattern.to_lowercase();
    
    // Convert glob pattern to simple matching
    if lower_pattern.contains('*') || lower_pattern.contains('?') {
        // Simple wildcard matching
        let parts: Vec<&str> = lower_pattern.split('*').collect();
        if parts.len() == 1 {
            // No wildcards, exact match
            lower_text == lower_pattern
        } else {
            // Check if text contains all parts in order
            let mut pos = 0;
            for (i, part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }
                if i == 0 {
                    // First part must match at start
                    if !lower_text.starts_with(part) {
                        return false;
                    }
                    pos = part.len();
                } else if i == parts.len() - 1 && !pattern.ends_with('*') {
                    // Last part must match at end
                    if !lower_text.ends_with(part) {
                        return false;
                    }
                } else {
                    // Middle part must exist somewhere after current position
                    if let Some(found_pos) = lower_text[pos..].find(part) {
                        pos += found_pos + part.len();
                    } else {
                        return false;
                    }
                }
            }
            true
        }
    } else {
        // No wildcards - check for substring
        lower_text.contains(&lower_pattern)
    }
}

/// Helper to check if an AD1 tree entry should be included based on query filters
fn should_include_ad1(entry: &ad1::TreeEntry, query: &SearchQuery) -> bool {
    // Check directory/file filters
    if query.directories_only && !entry.is_dir {
        return false;
    }
    if query.files_only && entry.is_dir {
        return false;
    }
    
    // Check size filters
    if let Some(min) = query.min_size {
        if entry.size < min {
            return false;
        }
    }
    if let Some(max) = query.max_size {
        if entry.size > max {
            return false;
        }
    }
    
    true
}
