// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Metadata export operations for forensic containers
//!
//! Provides JSON and CSV export of container metadata across all supported formats.

use tracing::debug;

use crate::ad1;
use crate::archive;
use crate::ewf;
use crate::raw;
use crate::ufed;

use crate::containers::types::ContainerKind;

use super::detect_container;

/// Export container metadata as JSON
pub fn export_metadata_json(path: &str) -> Result<String, String> {
    debug!(path = %path, "Exporting container metadata as JSON");

    match detect_container(path)? {
        ContainerKind::Ad1 => ad1::export_tree_json(path).map_err(|e| e.to_string()),
        ContainerKind::E01 | ContainerKind::L01 => {
            ewf::export_metadata_json(path).map_err(|e| e.to_string())
        }
        ContainerKind::Raw => raw::export_metadata_json(path).map_err(|e| e.to_string()),
        ContainerKind::Archive => {
            // Serialize archive info as JSON
            let info = archive::info(path)?;
            serde_json::to_string_pretty(&info).map_err(|e| format!("Failed to serialize: {}", e))
        }
        ContainerKind::Ufed => {
            // Serialize UFED info as JSON
            let info = ufed::info(path)?;
            serde_json::to_string_pretty(&info).map_err(|e| format!("Failed to serialize: {}", e))
        }
    }
}

/// Export container metadata as CSV
pub fn export_metadata_csv(path: &str) -> Result<String, String> {
    debug!(path = %path, "Exporting container metadata as CSV");

    match detect_container(path)? {
        ContainerKind::Ad1 => ad1::export_tree_csv(path).map_err(|e| e.to_string()),
        ContainerKind::E01 | ContainerKind::L01 => {
            ewf::export_metadata_csv(path).map_err(|e| e.to_string())
        }
        ContainerKind::Raw => raw::export_metadata_csv(path).map_err(|e| e.to_string()),
        ContainerKind::Archive => {
            // Generate CSV from archive entries
            let entries = archive::list_zip_entries(path)?;
            let mut csv = String::new();
            csv.push_str("Index,Path,Is Directory,Size,Compressed Size,CRC32,Last Modified\n");
            for entry in entries {
                csv.push_str(&format!(
                    "{},\"{}\",{},{},{},{},\"{}\"\n",
                    entry.index,
                    entry.path,
                    entry.is_directory,
                    entry.size,
                    entry.compressed_size,
                    entry.crc32,
                    entry.last_modified
                ));
            }
            Ok(csv)
        }
        ContainerKind::Ufed => {
            // Generate CSV from UFED info
            let info = ufed::info(path)?;
            let mut csv = String::new();
            csv.push_str("# UFED Container Metadata\n");
            csv.push_str(&format!("Format,{}\n", info.format));
            csv.push_str(&format!("Size,{}\n", info.size));
            if let Some(ref hint) = info.device_hint {
                csv.push_str(&format!("Device Hint,\"{}\"\n", hint));
            }
            csv.push_str("\n# Associated Files\n");
            csv.push_str("Filename,Size,Type,Hash\n");
            for file in &info.associated_files {
                csv.push_str(&format!(
                    "\"{}\",{},\"{}\",\"{}\"\n",
                    file.filename,
                    file.size,
                    file.file_type,
                    file.stored_hash.as_deref().unwrap_or("")
                ));
            }
            Ok(csv)
        }
    }
}
