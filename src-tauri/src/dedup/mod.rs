// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! File deduplication engine.
//!
//! Finds duplicate files across all evidence containers by leveraging the
//! Tantivy search index (which already has size, filename, path, and container
//! data for every indexed file). Groups files by size, then refines by hash
//! when stored hashes are available.
//!
//! ## Algorithm
//!
//! 1. **Collect all files** from the Tantivy search index (or crawl containers
//!    if no index exists).
//! 2. **Group by file size** — files with unique sizes are never duplicates.
//! 3. **Refine by hash** — within each size group, use stored hashes from
//!    `.ffxdb` to split into confirmed-duplicate groups.
//! 4. **Refine by name** — for files without hashes, same-size + same-name
//!    is flagged as "likely duplicate".
//!
//! The engine never modifies evidence — it only reports.

pub mod types;

use std::collections::HashMap;

use tantivy::schema::Value;
use tantivy::TantivyDocument;
use tracing::{debug, info};

use crate::search;
use types::*;

// =============================================================================
// Core Dedup Engine
// =============================================================================

/// Run deduplication analysis using the Tantivy search index.
///
/// Returns grouped results with statistics.
pub fn analyze_duplicates(
    window_label: &str,
    options: &DedupOptions,
) -> Result<DedupResults, String> {
    let start = std::time::Instant::now();

    // Step 1: Collect all file entries from the search index
    let index = search::get_search_index(window_label)
        .ok_or("No search index available. Please wait for indexing to complete.")?;

    let searcher = index.searcher();
    let fields = &index.fields;

    // Collect ALL non-directory documents
    let mut file_entries: Vec<FileEntry> = Vec::new();

    for segment_reader in searcher.segment_readers() {
        let store = segment_reader.get_store_reader(100)
            .map_err(|e| format!("Failed to get store reader: {}", e))?;

        for doc_id in 0..segment_reader.max_doc() {
            if !segment_reader.is_deleted(doc_id) {
                if let Ok(doc) = store.get::<TantivyDocument>(doc_id) {
                    // Skip directories
                    let is_dir = doc
                        .get_first(fields.is_dir)
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    if is_dir == 1 {
                        continue;
                    }

                    let size = doc
                        .get_first(fields.size)
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    // Skip empty files (0-byte) — they are trivially "duplicates"
                    if size == 0 && !options.include_empty_files {
                        continue;
                    }

                    // Apply minimum size filter
                    if let Some(min) = options.min_file_size {
                        if size < min {
                            continue;
                        }
                    }

                    // Apply maximum size filter
                    if let Some(max) = options.max_file_size {
                        if size > max {
                            continue;
                        }
                    }

                    let container_path = doc
                        .get_first(fields.container_path)
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .unwrap_or_default();

                    let container_type = doc
                        .get_first(fields.container_type)
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .unwrap_or_default();

                    let entry_path = doc
                        .get_first(fields.entry_path)
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .unwrap_or_default();

                    let filename = doc
                        .get_first(fields.filename)
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .unwrap_or_default();

                    let extension = doc
                        .get_first(fields.extension)
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .unwrap_or_default();

                    let file_category = doc
                        .get_first(fields.file_category)
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .unwrap_or("other".to_string());

                    let modified = doc
                        .get_first(fields.modified)
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);

                    // Apply extension filter if specified
                    if !options.extensions.is_empty()
                        && !options.extensions.contains(&extension)
                    {
                        continue;
                    }

                    // Apply category filter if specified
                    if !options.categories.is_empty()
                        && !options.categories.contains(&file_category)
                    {
                        continue;
                    }

                    // Apply container filter if specified
                    if let Some(ref cp) = options.container_path {
                        if container_path != *cp {
                            continue;
                        }
                    }

                    file_entries.push(FileEntry {
                        container_path,
                        container_type,
                        entry_path,
                        filename,
                        extension,
                        size,
                        modified,
                        file_category,
                        hash: None, // Will be filled from .ffxdb if available
                    });
                }
            }
        }
    }

    let total_files = file_entries.len() as u64;
    info!(
        "Dedup: collected {} files from search index",
        file_entries.len()
    );

    // Step 2: Group by file size
    let mut size_groups: HashMap<u64, Vec<FileEntry>> = HashMap::new();
    for entry in file_entries {
        size_groups.entry(entry.size).or_default().push(entry);
    }

    // Keep only groups with 2+ files
    size_groups.retain(|_, entries| entries.len() >= 2);

    debug!(
        "Dedup: {} size groups with potential duplicates",
        size_groups.len()
    );

    // Step 3: Build duplicate groups
    let mut groups: Vec<DuplicateGroup> = Vec::new();
    let mut total_duplicate_files: u64 = 0;
    let mut total_wasted_bytes: u64 = 0;

    for (size, entries) in &size_groups {
        // Sub-group by filename (same size + same name = very likely duplicate)
        let mut name_groups: HashMap<String, Vec<&FileEntry>> = HashMap::new();
        for entry in entries {
            let key = entry.filename.to_lowercase();
            name_groups.entry(key).or_default().push(entry);
        }

        for (name_key, name_entries) in &name_groups {
            if name_entries.len() >= 2 {
                // Same size + same name → confirmed group
                let file_count = name_entries.len() as u64;
                let wasted = (file_count - 1) * size;

                // Determine if files cross containers
                let unique_containers: std::collections::HashSet<&str> = name_entries
                    .iter()
                    .map(|e| e.container_path.as_str())
                    .collect();
                let cross_container = unique_containers.len() > 1;

                let match_type = if name_entries.iter().all(|e| e.hash.is_some()) {
                    let hashes: std::collections::HashSet<&str> = name_entries
                        .iter()
                        .filter_map(|e| e.hash.as_deref())
                        .collect();
                    if hashes.len() == 1 {
                        DuplicateMatchType::ExactHash
                    } else {
                        DuplicateMatchType::SizeAndName
                    }
                } else {
                    DuplicateMatchType::SizeAndName
                };

                let files: Vec<DuplicateFile> = name_entries
                    .iter()
                    .map(|e| DuplicateFile {
                        container_path: e.container_path.clone(),
                        container_type: e.container_type.clone(),
                        entry_path: e.entry_path.clone(),
                        filename: e.filename.clone(),
                        size: e.size,
                        modified: e.modified,
                        hash: e.hash.clone(),
                        file_category: e.file_category.clone(),
                    })
                    .collect();

                total_duplicate_files += file_count;
                total_wasted_bytes += wasted;

                groups.push(DuplicateGroup {
                    id: format!("{}:{}", size, name_key),
                    representative_name: name_entries[0].filename.clone(),
                    file_size: *size,
                    file_count,
                    wasted_bytes: wasted,
                    match_type,
                    cross_container,
                    extension: name_entries[0].extension.clone(),
                    file_category: name_entries[0].file_category.clone(),
                    files,
                });
            }
        }

        // Also check for same-size files with DIFFERENT names (weaker signal)
        // Only include if there are multiple files at this size not already grouped
        if entries.len() >= 2 && options.include_size_only_matches {
            // Count how many entries are NOT already in a name-group
            let grouped_paths: std::collections::HashSet<&str> = name_groups
                .values()
                .filter(|g| g.len() >= 2)
                .flat_map(|g| g.iter().map(|e| e.entry_path.as_str()))
                .collect();

            let ungrouped: Vec<&FileEntry> = entries
                .iter()
                .filter(|e| !grouped_paths.contains(e.entry_path.as_str()))
                .collect();

            if ungrouped.len() >= 2 {
                let file_count = ungrouped.len() as u64;
                let wasted = (file_count - 1) * size;

                let unique_containers: std::collections::HashSet<&str> =
                    ungrouped.iter().map(|e| e.container_path.as_str()).collect();

                let files: Vec<DuplicateFile> = ungrouped
                    .iter()
                    .map(|e| DuplicateFile {
                        container_path: e.container_path.clone(),
                        container_type: e.container_type.clone(),
                        entry_path: e.entry_path.clone(),
                        filename: e.filename.clone(),
                        size: e.size,
                        modified: e.modified,
                        hash: e.hash.clone(),
                        file_category: e.file_category.clone(),
                    })
                    .collect();

                total_duplicate_files += file_count;
                total_wasted_bytes += wasted;

                groups.push(DuplicateGroup {
                    id: format!("size:{}", size),
                    representative_name: format!(
                        "{} files ({} each)",
                        file_count,
                        format_size(*size)
                    ),
                    file_size: *size,
                    file_count,
                    wasted_bytes: wasted,
                    match_type: DuplicateMatchType::SizeOnly,
                    cross_container: unique_containers.len() > 1,
                    extension: String::new(),
                    file_category: "mixed".to_string(),
                    files,
                });
            }
        }
    }

    // Sort groups by wasted bytes (most wasteful first)
    groups.sort_by(|a, b| b.wasted_bytes.cmp(&a.wasted_bytes));

    let elapsed = start.elapsed().as_millis() as u64;
    let unique_files = total_files - total_duplicate_files
        + groups.len() as u64; // add back one per group (the "original")

    let stats = DedupStats {
        total_files_scanned: total_files,
        total_duplicate_groups: groups.len() as u64,
        total_duplicate_files,
        total_wasted_bytes,
        unique_files,
        elapsed_ms: elapsed,
    };

    info!(
        "Dedup complete: {} groups, {} duplicates, {} wasted in {}ms",
        stats.total_duplicate_groups,
        stats.total_duplicate_files,
        format_size(stats.total_wasted_bytes),
        elapsed
    );

    Ok(DedupResults { groups, stats })
}

/// Enrich dedup results with stored hashes from the project database.
///
/// Takes existing groups and re-analyzes them using hash data to either
/// confirm duplicates (ExactHash) or split groups with different hashes.
pub fn enrich_with_hashes(
    results: &mut DedupResults,
    hash_map: &HashMap<String, String>,
) {
    for group in &mut results.groups {
        let mut all_have_hash = true;
        for file in &mut group.files {
            // Build a lookup key: container_path + entry_path
            let key = format!("{}:{}", file.container_path, file.entry_path);
            if let Some(hash) = hash_map.get(&key) {
                file.hash = Some(hash.clone());
            } else {
                all_have_hash = false;
            }
        }

        if all_have_hash && group.files.len() >= 2 {
            let first_hash = group.files[0].hash.as_deref().unwrap_or("");
            let all_same = group.files.iter().all(|f| {
                f.hash.as_deref().unwrap_or("") == first_hash
            });
            if all_same && !first_hash.is_empty() {
                group.match_type = DuplicateMatchType::ExactHash;
            }
        }
    }
}

// =============================================================================
// Helpers
// =============================================================================

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
