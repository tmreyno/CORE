// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! UFED metadata export operations
//!
//! Provides JSON and CSV export of UFED container metadata including
//! case info, device info, extraction details, and stored hashes.

use tracing::{debug, instrument};

use crate::common::escape_csv;
use crate::containers::ContainerError;

/// Export UFED container metadata as JSON
#[instrument]
pub fn export_metadata_json(path: &str) -> Result<String, ContainerError> {
    debug!(path = %path, "Exporting UFED metadata as JSON");

    let info = super::info(path)?;
    let stats = super::get_stats(path)?;

    #[derive(serde::Serialize)]
    struct UfedMetadata {
        format: String,
        file_size: u64,
        file_size_formatted: String,
        device_hint: Option<String>,
        statistics: UfedStatistics,
        case_info: Option<CaseInfoSummary>,
        device_info: Option<DeviceInfoSummary>,
        extraction_info: Option<ExtractionInfoSummary>,
        associated_files: Vec<AssociatedFileSummary>,
        stored_hashes: Vec<StoredHashSummary>,
    }

    #[derive(serde::Serialize)]
    struct UfedStatistics {
        associated_file_count: usize,
        associated_files_size: u64,
        stored_hash_count: usize,
        has_case_info: bool,
        has_device_info: bool,
        has_extraction_info: bool,
        has_collection_info: bool,
    }

    #[derive(serde::Serialize)]
    struct CaseInfoSummary {
        case_identifier: Option<String>,
        examiner_name: Option<String>,
        department: Option<String>,
        crime_type: Option<String>,
    }

    #[derive(serde::Serialize)]
    struct DeviceInfoSummary {
        full_name: Option<String>,
        model: Option<String>,
        os_version: Option<String>,
        vendor: Option<String>,
    }

    #[derive(serde::Serialize)]
    struct ExtractionInfoSummary {
        extraction_type: Option<String>,
        acquisition_tool: Option<String>,
        start_time: Option<String>,
        end_time: Option<String>,
    }

    #[derive(serde::Serialize)]
    struct AssociatedFileSummary {
        filename: String,
        file_type: String,
        size: u64,
        size_formatted: String,
        stored_hash: Option<String>,
    }

    #[derive(serde::Serialize)]
    struct StoredHashSummary {
        filename: String,
        hash: String,
        algorithm: String,
    }

    let metadata = UfedMetadata {
        format: info.format.clone(),
        file_size: info.size,
        file_size_formatted: crate::common::format_size(info.size),
        device_hint: info.device_hint.clone(),
        statistics: UfedStatistics {
            associated_file_count: stats.associated_file_count,
            associated_files_size: stats.associated_files_size,
            stored_hash_count: stats.stored_hash_count,
            has_case_info: stats.has_case_info,
            has_device_info: stats.has_device_info,
            has_extraction_info: stats.has_extraction_info,
            has_collection_info: stats.has_collection_info,
        },
        case_info: info.case_info.as_ref().map(|c| CaseInfoSummary {
            case_identifier: c.case_identifier.clone(),
            examiner_name: c.examiner_name.clone(),
            department: c.department.clone(),
            crime_type: c.crime_type.clone(),
        }),
        device_info: info.device_info.as_ref().map(|d| DeviceInfoSummary {
            full_name: d.full_name.clone(),
            model: d.model.clone(),
            os_version: d.os_version.clone(),
            vendor: d.vendor.clone(),
        }),
        extraction_info: info
            .extraction_info
            .as_ref()
            .map(|e| ExtractionInfoSummary {
                extraction_type: e.extraction_type.clone(),
                acquisition_tool: e.acquisition_tool.clone(),
                start_time: e.start_time.clone(),
                end_time: e.end_time.clone(),
            }),
        associated_files: info
            .associated_files
            .iter()
            .map(|f| AssociatedFileSummary {
                filename: f.filename.clone(),
                file_type: f.file_type.clone(),
                size: f.size,
                size_formatted: crate::common::format_size(f.size),
                stored_hash: f.stored_hash.clone(),
            })
            .collect(),
        stored_hashes: info
            .stored_hashes
            .as_ref()
            .map(|hashes| {
                hashes
                    .iter()
                    .map(|h| StoredHashSummary {
                        filename: h.filename.clone(),
                        hash: h.hash.clone(),
                        algorithm: h.algorithm.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default(),
    };

    serde_json::to_string_pretty(&metadata).map_err(|e| {
        ContainerError::SerializationError(format!(
            "Failed to serialize UFED metadata to JSON: {e}"
        ))
    })
}

/// Export UFED container metadata as CSV
#[instrument]
pub fn export_metadata_csv(path: &str) -> Result<String, ContainerError> {
    debug!(path = %path, "Exporting UFED metadata as CSV");

    let info = super::info(path)?;
    let stats = super::get_stats(path)?;

    let mut csv = String::new();

    // Header section
    csv.push_str("# UFED Container Metadata\n");
    csv.push_str(&format!("Format,{}\n", info.format));
    csv.push_str(&format!("File Size,{}\n", info.size));
    csv.push_str(&format!(
        "File Size (Formatted),\"{}\"\n",
        crate::common::format_size(info.size)
    ));
    if let Some(ref hint) = info.device_hint {
        csv.push_str(&format!("Device Hint,\"{}\"\n", escape_csv(hint)));
    }
    csv.push('\n');

    // Statistics
    csv.push_str("# Statistics\n");
    csv.push_str(&format!(
        "Associated File Count,{}\n",
        stats.associated_file_count
    ));
    csv.push_str(&format!(
        "Associated Files Size,{}\n",
        stats.associated_files_size
    ));
    csv.push_str(&format!("Stored Hash Count,{}\n", stats.stored_hash_count));
    csv.push_str(&format!("Has Case Info,{}\n", stats.has_case_info));
    csv.push_str(&format!("Has Device Info,{}\n", stats.has_device_info));
    csv.push_str(&format!(
        "Has Extraction Info,{}\n",
        stats.has_extraction_info
    ));
    csv.push_str(&format!(
        "Has Collection Info,{}\n",
        stats.has_collection_info
    ));
    csv.push('\n');

    // Case info
    if let Some(ref case_info) = info.case_info {
        csv.push_str("# Case Information\n");
        if let Some(ref ci) = case_info.case_identifier {
            csv.push_str(&format!("Case Identifier,\"{}\"\n", escape_csv(ci)));
        }
        if let Some(ref examiner) = case_info.examiner_name {
            csv.push_str(&format!("Examiner Name,\"{}\"\n", escape_csv(examiner)));
        }
        if let Some(ref dept) = case_info.department {
            csv.push_str(&format!("Department,\"{}\"\n", escape_csv(dept)));
        }
        if let Some(ref crime) = case_info.crime_type {
            csv.push_str(&format!("Crime Type,\"{}\"\n", escape_csv(crime)));
        }
        csv.push('\n');
    }

    // Device info
    if let Some(ref device_info) = info.device_info {
        csv.push_str("# Device Information\n");
        if let Some(ref name) = device_info.full_name {
            csv.push_str(&format!("Full Name,\"{}\"\n", escape_csv(name)));
        }
        if let Some(ref model) = device_info.model {
            csv.push_str(&format!("Model,\"{}\"\n", escape_csv(model)));
        }
        if let Some(ref vendor) = device_info.vendor {
            csv.push_str(&format!("Vendor,\"{}\"\n", escape_csv(vendor)));
        }
        if let Some(ref os) = device_info.os_version {
            csv.push_str(&format!("OS Version,\"{}\"\n", escape_csv(os)));
        }
        csv.push('\n');
    }

    // Associated files
    csv.push_str("# Associated Files\n");
    csv.push_str("Filename,File Type,Size,Size (Formatted),Hash\n");
    for file in &info.associated_files {
        csv.push_str(&format!(
            "\"{}\",\"{}\",{},\"{}\",{}\n",
            escape_csv(&file.filename),
            escape_csv(&file.file_type),
            file.size,
            crate::common::format_size(file.size),
            file.stored_hash.as_deref().unwrap_or("")
        ));
    }
    csv.push('\n');

    // Stored hashes
    if let Some(ref hashes) = info.stored_hashes {
        csv.push_str("# Stored Hashes\n");
        csv.push_str("Filename,Hash,Algorithm\n");
        for hash in hashes {
            csv.push_str(&format!(
                "\"{}\",{},{}\n",
                escape_csv(&hash.filename),
                hash.hash,
                hash.algorithm
            ));
        }
    }

    Ok(csv)
}
