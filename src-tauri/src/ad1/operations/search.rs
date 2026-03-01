// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AD1 search operations — search by name, extension, or hash.

use super::super::parser::Session;
use super::super::types::{Item, SearchResult, AD1_FOLDER_SIGNATURE};
use super::super::utils::*;
use crate::containers::ContainerError;

// =============================================================================
// Search Functions
// =============================================================================

/// Find entries by name pattern (case-insensitive substring match)
#[must_use = "this returns the search results, which should be used"]
pub fn find_by_name(path: &str, pattern: &str) -> Result<Vec<SearchResult>, ContainerError> {
    let session = Session::open(path)?;
    let pattern_lower = pattern.to_lowercase();
    let mut results = Vec::new();

    fn search_items(
        items: &[Item],
        pattern: &str,
        parent_path: &str,
        depth: u32,
        results: &mut Vec<SearchResult>,
    ) {
        for item in items {
            let item_path = join_path(parent_path, &item.name);

            if item.name.to_lowercase().contains(pattern) {
                results.push(SearchResult {
                    entry: build_tree_entry(item, parent_path, true),
                    match_type: "name".to_string(),
                    depth,
                });
            }

            search_items(&item.children, pattern, &item_path, depth + 1, results);
        }
    }

    search_items(&session.root_items, &pattern_lower, "", 0, &mut results);
    Ok(results)
}

/// Find entries by file extension
#[must_use = "this returns the search results, which should be used"]
pub fn find_by_extension(path: &str, extension: &str) -> Result<Vec<SearchResult>, ContainerError> {
    let session = Session::open(path)?;
    let ext_lower = extension.to_lowercase().trim_start_matches('.').to_string();
    let mut results = Vec::new();

    fn search_items(
        items: &[Item],
        ext: &str,
        parent_path: &str,
        depth: u32,
        results: &mut Vec<SearchResult>,
    ) {
        for item in items {
            let item_path = join_path(parent_path, &item.name);

            // Check extension
            if item.item_type != AD1_FOLDER_SIGNATURE {
                if let Some(item_ext) = item.name.rsplit('.').next() {
                    if item_ext.to_lowercase() == ext {
                        results.push(SearchResult {
                            entry: build_tree_entry(item, parent_path, true),
                            match_type: "extension".to_string(),
                            depth,
                        });
                    }
                }
            }

            search_items(&item.children, ext, &item_path, depth + 1, results);
        }
    }

    search_items(&session.root_items, &ext_lower, "", 0, &mut results);
    Ok(results)
}

/// Find entry by hash value (searches MD5 and SHA1)
#[must_use = "this returns the search result, which should be used"]
pub fn find_by_hash(path: &str, hash: &str) -> Result<Option<SearchResult>, ContainerError> {
    let session = Session::open(path)?;
    let hash_lower = hash.to_lowercase();

    fn search_items(
        items: &[Item],
        hash: &str,
        parent_path: &str,
        depth: u32,
    ) -> Option<SearchResult> {
        for item in items {
            let item_path = join_path(parent_path, &item.name);

            // Check MD5 and SHA1 hashes
            let entry = build_tree_entry(item, parent_path, true);
            if let Some(ref md5) = entry.md5_hash {
                if md5.to_lowercase() == hash {
                    return Some(SearchResult {
                        entry,
                        match_type: "md5".to_string(),
                        depth,
                    });
                }
            }
            if let Some(ref sha1) = entry.sha1_hash {
                if sha1.to_lowercase() == hash {
                    return Some(SearchResult {
                        entry,
                        match_type: "sha1".to_string(),
                        depth,
                    });
                }
            }

            if let Some(found) = search_items(&item.children, hash, &item_path, depth + 1) {
                return Some(found);
            }
        }
        None
    }

    Ok(search_items(&session.root_items, &hash_lower, "", 0))
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_by_name_nonexistent() {
        let result = find_by_name("/nonexistent/path/file.ad1", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_by_extension_nonexistent() {
        let result = find_by_extension("/nonexistent/path/file.ad1", "txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_by_hash_nonexistent() {
        let result = find_by_hash("/nonexistent/path/file.ad1", "abc123");
        assert!(result.is_err());
    }
}
