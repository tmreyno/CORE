// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Types for the file deduplication engine.

use serde::{Deserialize, Serialize};

// =============================================================================
// Input / Options
// =============================================================================

/// Options for controlling deduplication analysis.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedupOptions {
    /// Include 0-byte files (default: false — they are trivially identical)
    #[serde(default)]
    pub include_empty_files: bool,

    /// Include size-only matches (files with same size but different names)
    #[serde(default)]
    pub include_size_only_matches: bool,

    /// Minimum file size to consider (bytes, None = no minimum)
    pub min_file_size: Option<u64>,

    /// Maximum file size to consider (bytes, None = no maximum)
    pub max_file_size: Option<u64>,

    /// Filter to specific file extensions (empty = all)
    #[serde(default)]
    pub extensions: Vec<String>,

    /// Filter to specific file categories (empty = all)
    #[serde(default)]
    pub categories: Vec<String>,

    /// Filter to a specific container path (None = all containers)
    pub container_path: Option<String>,
}

impl Default for DedupOptions {
    fn default() -> Self {
        Self {
            include_empty_files: false,
            include_size_only_matches: false,
            min_file_size: None,
            max_file_size: None,
            extensions: Vec::new(),
            categories: Vec::new(),
            container_path: None,
        }
    }
}

// =============================================================================
// Internal: File entry (used during collection)
// =============================================================================

/// A file entry collected from the search index for analysis.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub container_path: String,
    pub container_type: String,
    pub entry_path: String,
    pub filename: String,
    pub extension: String,
    pub size: u64,
    pub modified: i64,
    pub file_category: String,
    pub hash: Option<String>,
}

// =============================================================================
// Output: Dedup Results
// =============================================================================

/// Complete deduplication analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedupResults {
    /// All groups of duplicate files, sorted by wasted bytes (descending)
    pub groups: Vec<DuplicateGroup>,
    /// Summary statistics
    pub stats: DedupStats,
}

/// A group of files believed to be duplicates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    /// Unique group identifier
    pub id: String,
    /// Representative display name (filename of first entry, or summary)
    pub representative_name: String,
    /// File size in bytes (all files in the group have this size)
    pub file_size: u64,
    /// Number of files in this group
    pub file_count: u64,
    /// Bytes wasted by duplicates: (file_count - 1) * file_size
    pub wasted_bytes: u64,
    /// How the duplicates were identified
    pub match_type: DuplicateMatchType,
    /// Whether files in this group come from different containers
    pub cross_container: bool,
    /// File extension (if all files share the same extension)
    pub extension: String,
    /// File category (document, image, email, etc.)
    pub file_category: String,
    /// All files in this duplicate group
    pub files: Vec<DuplicateFile>,
}

/// How duplicates were matched.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DuplicateMatchType {
    /// Confirmed identical by hash comparison
    ExactHash,
    /// Same file size AND same filename (high confidence)
    SizeAndName,
    /// Same file size only (lower confidence)
    SizeOnly,
}

/// A single file within a duplicate group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateFile {
    /// Container file path
    pub container_path: String,
    /// Container type (ad1, e01, zip, etc.)
    pub container_type: String,
    /// Path within the container
    pub entry_path: String,
    /// Filename
    pub filename: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified timestamp (unix)
    pub modified: i64,
    /// Hash value (if available from stored hashes)
    pub hash: Option<String>,
    /// File category
    pub file_category: String,
}

/// Summary statistics for a deduplication analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedupStats {
    /// Total files scanned
    pub total_files_scanned: u64,
    /// Number of duplicate groups found
    pub total_duplicate_groups: u64,
    /// Total number of duplicate files (across all groups)
    pub total_duplicate_files: u64,
    /// Total bytes wasted by duplicates
    pub total_wasted_bytes: u64,
    /// Number of unique files
    pub unique_files: u64,
    /// Analysis time in milliseconds
    pub elapsed_ms: u64,
}
