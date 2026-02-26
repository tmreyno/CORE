// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AD1 export (JSON/CSV) and extraction operations.

use std::path::Path;

use super::super::parser::Session;
use super::super::utils::count_files;
use crate::common::escape_csv;
use crate::containers::ContainerError;

// =============================================================================
// Extraction Functions
// =============================================================================

/// Extract container contents to output directory
#[must_use = "this returns a Result that should be checked for errors"]
pub fn extract(path: &str, output_dir: &str) -> Result<(), ContainerError> {
    extract_with_progress(path, output_dir, |_, _| {})
}

/// Extract with progress callback
#[must_use = "this returns a Result that should be checked for errors"]
pub fn extract_with_progress<F>(path: &str, output_dir: &str, mut progress_callback: F) -> Result<(), ContainerError>
where
    F: FnMut(u64, u64)
{
    let mut session = Session::open(path)?;
    let output_path = Path::new(output_dir);
    
    // Count total files for progress
    let total = count_files(&session.root_items);
    let mut current = 0u64;
    
    // Clone root_items to avoid borrow checker issues
    let root_items = session.root_items.clone();
    
    for item in &root_items {
        session.extract_item_with_progress(item, output_path, &mut current, total, &mut progress_callback)?;
    }
    
    Ok(())
}

// =============================================================================
// Export Functions
// =============================================================================

/// Export tree structure to JSON string
#[must_use = "this returns the JSON string, which should be used"]
pub fn export_tree_json(path: &str) -> Result<String, ContainerError> {
    let entries = super::get_tree(path)?;
    serde_json::to_string_pretty(&entries)
        .map_err(|e| ContainerError::SerializationError(format!("Failed to serialize tree to JSON: {e}")))
}

/// Export tree structure to CSV string
#[must_use = "this returns the CSV string, which should be used"]
pub fn export_tree_csv(path: &str) -> Result<String, ContainerError> {
    let entries = super::get_tree(path)?;
    let mut csv = String::from("path,is_dir,size,item_type,md5_hash,sha1_hash,created,accessed,modified\n");
    
    for entry in entries {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            escape_csv(&entry.path),
            entry.is_dir,
            entry.size,
            entry.item_type,
            entry.md5_hash.as_deref().unwrap_or(""),
            entry.sha1_hash.as_deref().unwrap_or(""),
            entry.created.as_deref().unwrap_or(""),
            entry.accessed.as_deref().unwrap_or(""),
            entry.modified.as_deref().unwrap_or(""),
        ));
    }
    
    Ok(csv)
}

/// Export container metadata as JSON (container-level info, not file tree)
#[must_use = "this returns the JSON string, which should be used"]
pub fn export_metadata_json(path: &str) -> Result<String, ContainerError> {
    let info = super::info(path, false)?; // Don't include tree for metadata export
    let stats = super::get_stats(path)?;
    
    #[derive(serde::Serialize)]
    struct Ad1Metadata {
        format: String,
        segment_info: SegmentInfo,
        logical_info: LogicalInfo,
        statistics: Statistics,
        companion_log: Option<CompanionLogSummary>,
    }
    
    #[derive(serde::Serialize)]
    struct SegmentInfo {
        segment_count: u32,
        fragments_size: u32,
        segment_files: Vec<String>,
        segment_sizes: Vec<u64>,
        total_size: u64,
        total_size_formatted: String,
        missing_segments: Option<Vec<String>>,
    }
    
    #[derive(serde::Serialize)]
    struct LogicalInfo {
        image_version: u32,
        zlib_chunk_size: u32,
        data_source_name: String,
    }
    
    #[derive(serde::Serialize)]
    struct Statistics {
        total_items: u64,
        total_files: u64,
        total_folders: u64,
        total_size: u64,
        compressed_size: u64,
        compression_ratio: f64,
        max_depth: u32,
        files_with_md5: u64,
        files_with_sha1: u64,
        largest_file_size: u64,
        largest_file_path: Option<String>,
    }
    
    #[derive(serde::Serialize)]
    struct CompanionLogSummary {
        case_number: Option<String>,
        evidence_number: Option<String>,
        examiner: Option<String>,
        md5_hash: Option<String>,
        sha1_hash: Option<String>,
        sha256_hash: Option<String>,
    }
    
    let total_size = info.total_size.unwrap_or(0);
    
    let metadata = Ad1Metadata {
        format: "AD1".to_string(),
        segment_info: SegmentInfo {
            segment_count: info.segment.segment_number,
            fragments_size: info.segment.fragments_size,
            segment_files: info.segment_files.unwrap_or_default(),
            segment_sizes: info.segment_sizes.unwrap_or_default(),
            total_size,
            total_size_formatted: crate::common::format_size(total_size),
            missing_segments: info.missing_segments,
        },
        logical_info: LogicalInfo {
            image_version: info.logical.image_version,
            zlib_chunk_size: info.logical.zlib_chunk_size,
            data_source_name: info.logical.data_source_name.clone(),
        },
        statistics: Statistics {
            total_items: stats.total_items,
            total_files: stats.total_files,
            total_folders: stats.total_folders,
            total_size: stats.total_size,
            compressed_size: stats.compressed_size,
            compression_ratio: stats.compression_ratio,
            max_depth: stats.max_depth,
            files_with_md5: stats.files_with_md5,
            files_with_sha1: stats.files_with_sha1,
            largest_file_size: stats.largest_file_size,
            largest_file_path: stats.largest_file_path.clone(),
        },
        companion_log: info.companion_log.as_ref().map(|log| CompanionLogSummary {
            case_number: log.case_number.clone(),
            evidence_number: log.evidence_number.clone(),
            examiner: log.examiner.clone(),
            md5_hash: log.md5_hash.clone(),
            sha1_hash: log.sha1_hash.clone(),
            sha256_hash: log.sha256_hash.clone(),
        }),
    };
    
    serde_json::to_string_pretty(&metadata)
        .map_err(|e| ContainerError::SerializationError(format!("Failed to serialize metadata to JSON: {e}")))
}

/// Export container metadata as CSV (container-level info, not file tree)
#[must_use = "this returns the CSV string, which should be used"]
pub fn export_metadata_csv(path: &str) -> Result<String, ContainerError> {
    let info = super::info(path, false)?;
    let stats = super::get_stats(path)?;
    
    let mut csv = String::new();
    let total_size = info.total_size.unwrap_or(0);
    
    // Header section
    csv.push_str("# AD1 Container Metadata\n");
    csv.push_str("Format,AD1\n");
    csv.push_str(&format!("Segment Count,{}\n", info.segment.segment_number));
    csv.push_str(&format!("Fragments Size,{}\n", info.segment.fragments_size));
    csv.push_str(&format!("Total Size,{}\n", total_size));
    csv.push_str(&format!("Total Size (Formatted),\"{}\"\n", crate::common::format_size(total_size)));
    csv.push('\n');
    
    // Logical header info
    csv.push_str("# Logical Header\n");
    csv.push_str(&format!("Image Version,{}\n", info.logical.image_version));
    csv.push_str(&format!("zlib Chunk Size,{}\n", info.logical.zlib_chunk_size));
    csv.push_str(&format!("Data Source Name,\"{}\"\n", escape_csv(&info.logical.data_source_name)));
    csv.push('\n');
    
    // Statistics
    csv.push_str("# Statistics\n");
    csv.push_str(&format!("Total Items,{}\n", stats.total_items));
    csv.push_str(&format!("Total Files,{}\n", stats.total_files));
    csv.push_str(&format!("Total Folders,{}\n", stats.total_folders));
    csv.push_str(&format!("Compressed Size,{}\n", stats.compressed_size));
    csv.push_str(&format!("Compression Ratio,{:.2}\n", stats.compression_ratio));
    csv.push_str(&format!("Max Depth,{}\n", stats.max_depth));
    csv.push_str(&format!("Files With MD5,{}\n", stats.files_with_md5));
    csv.push_str(&format!("Files With SHA1,{}\n", stats.files_with_sha1));
    if let Some(ref largest_path) = stats.largest_file_path {
        csv.push_str(&format!("Largest File,\"{}\"\n", escape_csv(largest_path)));
        csv.push_str(&format!("Largest File Size,{}\n", stats.largest_file_size));
    }
    csv.push('\n');
    
    // Companion log info if available
    if let Some(ref log) = info.companion_log {
        csv.push_str("# Companion Log\n");
        if let Some(ref case) = log.case_number {
            csv.push_str(&format!("Case Number,\"{}\"\n", escape_csv(case)));
        }
        if let Some(ref evidence) = log.evidence_number {
            csv.push_str(&format!("Evidence Number,\"{}\"\n", escape_csv(evidence)));
        }
        if let Some(ref examiner) = log.examiner {
            csv.push_str(&format!("Examiner,\"{}\"\n", escape_csv(examiner)));
        }
        if let Some(ref md5) = log.md5_hash {
            csv.push_str(&format!("MD5 Hash,{}\n", md5));
        }
        if let Some(ref sha1) = log.sha1_hash {
            csv.push_str(&format!("SHA1 Hash,{}\n", sha1));
        }
        if let Some(ref sha256) = log.sha256_hash {
            csv.push_str(&format!("SHA256 Hash,{}\n", sha256));
        }
        csv.push('\n');
    }
    
    // Segment details
    csv.push_str("# Segment Details\n");
    csv.push_str("Index,Filename,Size,Size (Formatted)\n");
    if let (Some(files), Some(sizes)) = (&info.segment_files, &info.segment_sizes) {
        for (i, (name, &size)) in files.iter().zip(sizes.iter()).enumerate() {
            csv.push_str(&format!(
                "{},\"{}\",{},\"{}\"\n",
                i + 1,
                escape_csv(name),
                size,
                crate::common::format_size(size)
            ));
        }
    }
    
    Ok(csv)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_nonexistent() {
        let result = extract("/nonexistent/path/file.ad1", "/tmp/output");
        assert!(result.is_err());
    }

    #[test]
    fn test_export_tree_json_nonexistent() {
        let result = export_tree_json("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_export_tree_csv_nonexistent() {
        let result = export_tree_csv("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_export_metadata_json_nonexistent() {
        let result = export_metadata_json("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_export_metadata_csv_nonexistent() {
        let result = export_metadata_csv("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv("with\"quote"), "\"with\"\"quote\"");
        assert_eq!(escape_csv("with\nnewline"), "\"with\nnewline\"");
    }

    #[test]
    fn test_escape_csv_empty_string() {
        assert_eq!(escape_csv(""), "");
    }

    #[test]
    fn test_escape_csv_unicode() {
        assert_eq!(escape_csv("文件.txt"), "文件.txt");
        assert_eq!(escape_csv("文件,名称"), "\"文件,名称\"");
    }

    #[test]
    fn test_escape_csv_long_string() {
        let long_string = "a".repeat(10000);
        assert_eq!(escape_csv(&long_string), long_string);
    }
}
