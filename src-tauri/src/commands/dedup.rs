// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for file deduplication analysis.
//!
//! Provides commands for:
//! - Running deduplication scans across all indexed containers
//! - Exporting dedup results as CSV/JSON reports

use std::collections::HashMap;

use tracing::info;

use crate::commands::project_db::with_project_db;
use crate::common::csv_row;
use crate::dedup;
use crate::dedup::types::{DedupOptions, DedupResults, DuplicateFile, DuplicateGroup};

const DEDUP_MAX_GROUPS: usize = 10_000;
const DEDUP_MAX_FILES_PER_GROUP: usize = 10_000;
const DEDUP_MAX_HASH_MAP_ENTRIES: usize = 100_000;
const DEDUP_TEXT_FIELD_MAX_CHARS: usize = 4096;

// =============================================================================
// Dedup Commands
// =============================================================================

/// Run deduplication analysis on all indexed containers.
///
/// Reads file metadata from the Tantivy search index, groups by size and name,
/// and optionally enriches with stored hashes from the project database.
#[tauri::command]
pub async fn dedup_analyze(
    window: tauri::Window,
    options: DedupOptions,
) -> Result<DedupResults, String> {
    let label = window.label().to_string();
    let project_db_label = label.clone();
    let hash_algorithm = options
        .hash_algorithm
        .clone()
        .unwrap_or_else(|| "SHA-256".to_string());

    let mut results =
        tauri::async_runtime::spawn_blocking(move || dedup::analyze_duplicates(&label, &options))
            .await
            .map_err(|e| format!("Dedup task failed: {}", e))??;
    results = bounded_dedup_results(results);

    if let Ok(hash_map) = with_project_db(&project_db_label, |db| {
        db.latest_source_hash_map(&hash_algorithm)
    }) {
        if !hash_map.is_empty() {
            let hash_map = bounded_hash_map_for_results(hash_map, &results);
            dedup::enrich_with_hashes(&mut results, &hash_map);
        }
    }

    Ok(bounded_dedup_results(results))
}

/// Enrich dedup results with stored hashes from .ffxdb.
///
/// Takes a hash map of (container_path:entry_path → hash_value) and upgrades
/// match types from SizeAndName to ExactHash where hashes confirm identity.
#[tauri::command]
pub async fn dedup_enrich_hashes(
    mut results: DedupResults,
    hash_map: HashMap<String, String>,
) -> Result<DedupResults, String> {
    results = bounded_dedup_results(results);
    let hash_map = bounded_hash_map_for_results(hash_map, &results);
    dedup::enrich_with_hashes(&mut results, &hash_map);
    Ok(bounded_dedup_results(results))
}

/// Export dedup results as a CSV string.
#[tauri::command]
pub async fn dedup_export_csv(results: DedupResults) -> Result<String, String> {
    let results = bounded_dedup_results(results);
    Ok(dedup_results_to_csv(&results))
}

fn bounded_hash_map_for_results(
    hash_map: HashMap<String, String>,
    results: &DedupResults,
) -> HashMap<String, String> {
    let mut bounded = HashMap::new();

    'groups: for group in &results.groups {
        for file in &group.files {
            for key in hash_lookup_keys(file) {
                if bounded.len() >= DEDUP_MAX_HASH_MAP_ENTRIES {
                    break 'groups;
                }
                if let Some(value) = hash_map.get(&key) {
                    bounded.insert(key, truncate_dedup_text(value.clone()));
                }
            }
        }
    }

    bounded
}

fn hash_lookup_keys(file: &DuplicateFile) -> [String; 3] {
    [
        format!("{}:{}", file.container_path, file.entry_path),
        format!(
            "{}:{}:{}",
            file.container_type, file.container_path, file.entry_path
        ),
        format!(
            "vfs:{}:{}:{}",
            file.container_type, file.container_path, file.entry_path
        ),
    ]
}

fn bounded_dedup_results(mut results: DedupResults) -> DedupResults {
    results.groups.truncate(DEDUP_MAX_GROUPS);
    results.groups = results
        .groups
        .into_iter()
        .map(bounded_duplicate_group)
        .collect();
    refresh_bounded_stats(&mut results);
    results
}

fn bounded_duplicate_group(mut group: DuplicateGroup) -> DuplicateGroup {
    group.id = truncate_dedup_text(group.id);
    group.representative_name = truncate_dedup_text(group.representative_name);
    group.extension = truncate_dedup_text(group.extension);
    group.file_category = truncate_dedup_text(group.file_category);
    group.files.truncate(DEDUP_MAX_FILES_PER_GROUP);
    group.files = group
        .files
        .into_iter()
        .map(bounded_duplicate_file)
        .collect();

    group.file_count = group.files.len() as u64;
    group.wasted_bytes = group
        .file_count
        .saturating_sub(1)
        .saturating_mul(group.file_size);
    group.cross_container = group
        .files
        .first()
        .map(|first| {
            group
                .files
                .iter()
                .any(|file| file.container_path != first.container_path)
        })
        .unwrap_or(false);
    group
}

fn bounded_duplicate_file(mut file: DuplicateFile) -> DuplicateFile {
    file.container_path = truncate_dedup_text(file.container_path);
    file.container_type = truncate_dedup_text(file.container_type);
    file.entry_path = truncate_dedup_text(file.entry_path);
    file.filename = truncate_dedup_text(file.filename);
    file.hash = file.hash.map(truncate_dedup_text);
    file.file_category = truncate_dedup_text(file.file_category);
    file
}

fn truncate_dedup_text(value: String) -> String {
    if value.chars().count() <= DEDUP_TEXT_FIELD_MAX_CHARS {
        value
    } else {
        value.chars().take(DEDUP_TEXT_FIELD_MAX_CHARS).collect()
    }
}

fn refresh_bounded_stats(results: &mut DedupResults) {
    let total_duplicate_files = results
        .groups
        .iter()
        .map(|group| group.file_count)
        .fold(0u64, u64::saturating_add);
    let total_wasted_bytes = results
        .groups
        .iter()
        .map(|group| group.wasted_bytes)
        .fold(0u64, u64::saturating_add);

    results.stats.total_duplicate_groups = results.groups.len() as u64;
    results.stats.total_duplicate_files = total_duplicate_files;
    results.stats.total_wasted_bytes = total_wasted_bytes;
    results.stats.unique_files = results
        .stats
        .total_files_scanned
        .saturating_sub(total_duplicate_files)
        .saturating_add(results.stats.total_duplicate_groups);
}

fn dedup_results_to_csv(results: &DedupResults) -> String {
    let mut csv = String::new();
    csv.push_str(&csv_row(&[
        "Group ID",
        "Representative Name",
        "Match Type",
        "File Size",
        "Wasted Bytes",
        "Cross Container",
        "Container Path",
        "Entry Path",
        "Filename",
        "Hash",
    ]));

    for group in &results.groups {
        let match_type = match group.match_type {
            dedup::types::DuplicateMatchType::ExactHash => "Exact Hash",
            dedup::types::DuplicateMatchType::SizeAndName => "Size + Name",
            dedup::types::DuplicateMatchType::SizeOnly => "Size Only",
        };

        for file in &group.files {
            let file_size = group.file_size.to_string();
            let wasted_bytes = group.wasted_bytes.to_string();
            let cross_container = group.cross_container.to_string();
            csv.push_str(&csv_row(&[
                group.id.as_str(),
                group.representative_name.as_str(),
                match_type,
                file_size.as_str(),
                wasted_bytes.as_str(),
                cross_container.as_str(),
                file.container_path.as_str(),
                file.entry_path.as_str(),
                file.filename.as_str(),
                file.hash.as_deref().unwrap_or(""),
            ]));
        }
    }

    info!(
        "Dedup CSV export: {} groups, {} total lines",
        results.groups.len(),
        csv.lines().count()
    );

    csv
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dedup::types::{DedupStats, DuplicateFile, DuplicateGroup, DuplicateMatchType};

    fn duplicate_file(filename: &str) -> DuplicateFile {
        DuplicateFile {
            container_path: "/case/disk.E01".to_string(),
            container_type: "e01".to_string(),
            entry_path: format!("/Users/Alice/{}", filename),
            filename: filename.to_string(),
            size: 42,
            modified: 0,
            hash: Some("hash".to_string()),
            file_category: "document".to_string(),
        }
    }

    fn duplicate_group(id: &str, files: Vec<DuplicateFile>) -> DuplicateGroup {
        DuplicateGroup {
            id: id.to_string(),
            representative_name: "report.pdf".to_string(),
            file_size: 42,
            file_count: files.len() as u64,
            wasted_bytes: files.len().saturating_sub(1) as u64 * 42,
            match_type: DuplicateMatchType::ExactHash,
            cross_container: false,
            extension: "pdf".to_string(),
            file_category: "document".to_string(),
            files,
        }
    }

    fn dedup_results(groups: Vec<DuplicateGroup>) -> DedupResults {
        DedupResults {
            groups,
            stats: DedupStats {
                total_files_scanned: 1_000_000,
                total_duplicate_groups: 1_000_000,
                total_duplicate_files: 1_000_000,
                total_wasted_bytes: 1_000_000,
                unique_files: 0,
                elapsed_ms: 0,
            },
        }
    }

    #[test]
    fn bounded_dedup_results_caps_group_and_file_counts() {
        let mut groups = Vec::with_capacity(DEDUP_MAX_GROUPS + 1);
        groups.push(duplicate_group(
            "oversized-group",
            (0..DEDUP_MAX_FILES_PER_GROUP + 1)
                .map(|file_index| duplicate_file(&format!("file-{}.pdf", file_index)))
                .collect(),
        ));
        for group_index in 0..DEDUP_MAX_GROUPS {
            groups.push(duplicate_group(
                &format!("group-{}", group_index),
                vec![duplicate_file("file.pdf")],
            ));
        }

        let results = bounded_dedup_results(dedup_results(groups));

        assert_eq!(results.groups.len(), DEDUP_MAX_GROUPS);
        assert_eq!(results.groups[0].files.len(), DEDUP_MAX_FILES_PER_GROUP);
        assert_eq!(
            results.groups[0].file_count,
            DEDUP_MAX_FILES_PER_GROUP as u64
        );
        assert_eq!(
            results.stats.total_duplicate_groups,
            DEDUP_MAX_GROUPS as u64
        );
    }

    #[test]
    fn bounded_dedup_results_caps_text_fields() {
        let long_text = "x".repeat(DEDUP_TEXT_FIELD_MAX_CHARS + 1);
        let group = DuplicateGroup {
            id: long_text.clone(),
            representative_name: long_text.clone(),
            extension: long_text.clone(),
            file_category: long_text.clone(),
            ..duplicate_group(
                "group",
                vec![DuplicateFile {
                    container_path: long_text.clone(),
                    container_type: long_text.clone(),
                    entry_path: long_text.clone(),
                    filename: long_text.clone(),
                    hash: Some(long_text.clone()),
                    file_category: long_text.clone(),
                    ..duplicate_file("long.pdf")
                }],
            )
        };

        let results = bounded_dedup_results(dedup_results(vec![group]));
        let group = &results.groups[0];
        let file = &group.files[0];

        assert_eq!(group.id.chars().count(), DEDUP_TEXT_FIELD_MAX_CHARS);
        assert_eq!(
            group.representative_name.chars().count(),
            DEDUP_TEXT_FIELD_MAX_CHARS
        );
        assert_eq!(
            file.container_path.chars().count(),
            DEDUP_TEXT_FIELD_MAX_CHARS
        );
        assert_eq!(
            file.hash.as_ref().unwrap().chars().count(),
            DEDUP_TEXT_FIELD_MAX_CHARS
        );
    }

    #[test]
    fn bounded_hash_map_caps_entries_and_hash_values_without_truncating_keys() {
        let long_key = "k".repeat(DEDUP_TEXT_FIELD_MAX_CHARS + 100);
        let long_value = "v".repeat(DEDUP_TEXT_FIELD_MAX_CHARS + 100);
        let results = dedup_results(vec![duplicate_group(
            "group",
            vec![DuplicateFile {
                container_path: long_key.clone(),
                entry_path: "entry".to_string(),
                ..duplicate_file("file.pdf")
            }],
        )]);
        let lookup_key = format!("{}:entry", long_key);
        let bounded = bounded_hash_map_for_results(
            HashMap::from([(lookup_key.clone(), long_value)]),
            &results,
        );
        assert!(bounded.contains_key(&lookup_key));
        assert_eq!(
            bounded.get(&lookup_key).unwrap().chars().count(),
            DEDUP_TEXT_FIELD_MAX_CHARS
        );

        let mut hash_map = HashMap::new();
        let groups = (0..DEDUP_MAX_HASH_MAP_ENTRIES + 10)
            .map(|index| {
                duplicate_group(
                    &format!("group-{}", index),
                    vec![DuplicateFile {
                        container_path: format!("/case/disk-{}.E01", index),
                        entry_path: format!("/file-{}.pdf", index),
                        ..duplicate_file("file.pdf")
                    }],
                )
            })
            .collect();
        let results = dedup_results(groups);
        for index in 0..DEDUP_MAX_HASH_MAP_ENTRIES + 10 {
            hash_map.insert(format!("key-{}", index), "value".to_string());
            hash_map.insert(
                format!("/case/disk-{}.E01:/file-{}.pdf", index, index),
                "value".to_string(),
            );
        }

        let bounded = bounded_hash_map_for_results(hash_map, &results);

        assert_eq!(bounded.len(), DEDUP_MAX_HASH_MAP_ENTRIES);
        assert!(bounded
            .values()
            .all(|value| value.chars().count() <= DEDUP_TEXT_FIELD_MAX_CHARS));
    }

    #[test]
    fn dedup_results_to_csv_escapes_all_text_fields() {
        let results = DedupResults {
            groups: vec![DuplicateGroup {
                id: "group,\"one\"".to_string(),
                representative_name: "report,\nfinal.pdf".to_string(),
                file_size: 42,
                file_count: 1,
                wasted_bytes: 0,
                match_type: DuplicateMatchType::ExactHash,
                cross_container: true,
                extension: "pdf".to_string(),
                file_category: "document".to_string(),
                files: vec![DuplicateFile {
                    container_path: "/case/a,\"disk\".E01".to_string(),
                    container_type: "e01".to_string(),
                    entry_path: "/Users/Alice/report,\nfinal.pdf".to_string(),
                    filename: "report,\nfinal.pdf".to_string(),
                    size: 42,
                    modified: 0,
                    hash: Some("sha,\"256\"".to_string()),
                    file_category: "document".to_string(),
                }],
            }],
            stats: DedupStats {
                total_files_scanned: 1,
                total_duplicate_groups: 1,
                total_duplicate_files: 1,
                total_wasted_bytes: 0,
                unique_files: 0,
                elapsed_ms: 0,
            },
        };

        let csv = dedup_results_to_csv(&results);

        assert!(csv.starts_with("Group ID,Representative Name,Match Type"));
        assert!(csv.contains("\"group,\"\"one\"\"\""));
        assert!(csv.contains("\"report,\nfinal.pdf\""));
        assert!(csv.contains("\"/case/a,\"\"disk\"\".E01\""));
        assert!(csv.contains("\"sha,\"\"256\"\"\""));
    }
}
