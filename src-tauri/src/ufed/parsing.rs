// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! UFED file parsing
//!
//! Parsers for UFD (INI format) and UFDX (XML format) metadata files.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use super::types::{CaseInfo, CollectionInfo, DeviceInfo, ExtractionInfo, StoredHash};
use crate::containers::ContainerError;

/// Parsed UFD metadata tuple: (case_info, device_info, extraction_info, hashes)
type UfdParsedData = (
    Option<CaseInfo>,
    Option<DeviceInfo>,
    Option<ExtractionInfo>,
    Option<Vec<StoredHash>>,
);

/// Parse UFD file (INI-style format) and extract metadata
///
/// UFD files contain sections like:
/// - `\[Crime Case\]`: Case identifier, examiner, evidence number
/// - `\[DeviceInfo\]`: IMEI, model, OS version, vendor
/// - `\[General\]`: Acquisition tool, extraction type, timestamps
/// - `\[SHA256\]`, `\[SHA1\]`, `\[MD5\]`: Hash values for extraction files
pub fn parse_ufd_file(path: &str) -> Result<UfdParsedData, ContainerError> {
    let file = File::open(path).map_err(|e| format!("Failed to open UFD file: {e}"))?;

    let reader = BufReader::new(file);
    let mut sections: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current_section = String::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read UFD file: {e}"))?;
        let line = line.trim();

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Check for section header [SectionName]
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_string();
            sections.entry(current_section.clone()).or_default();
            continue;
        }

        // Parse key=value pairs
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_string();
            let value = line[eq_pos + 1..].trim().to_string();

            if !current_section.is_empty() {
                sections
                    .get_mut(&current_section)
                    .map(|s| s.insert(key, value));
            }
        }
    }

    // Extract Case Info from [Crime Case] section
    let case_info = sections.get("Crime Case").map(|s| CaseInfo {
        case_identifier: s.get("Case Identifier").cloned().filter(|v| !v.is_empty()),
        crime_type: s.get("Crime Type").cloned().filter(|v| !v.is_empty()),
        department: s.get("Department").cloned().filter(|v| !v.is_empty()),
        device_name: s
            .get("Device Name / Evidence Number")
            .cloned()
            .filter(|v| !v.is_empty()),
        examiner_name: s.get("Examiner Name").cloned().filter(|v| !v.is_empty()),
        location: s.get("Location").cloned().filter(|v| !v.is_empty()),
    });

    // Extract Device Info from [DeviceInfo] and [General] sections
    let device_section = sections.get("DeviceInfo");
    let general_section = sections.get("General");

    let device_info = if device_section.is_some() || general_section.is_some() {
        Some(DeviceInfo {
            vendor: general_section
                .and_then(|s| s.get("Vendor").cloned())
                .or_else(|| device_section.and_then(|s| s.get("Vendor").cloned()))
                .filter(|v| !v.is_empty()),
            model: device_section
                .and_then(|s| s.get("Model").cloned())
                .filter(|v| !v.is_empty()),
            full_name: general_section
                .and_then(|s| s.get("FullName").cloned())
                .or_else(|| general_section.and_then(|s| s.get("Model").cloned()))
                .filter(|v| !v.is_empty()),
            imei: device_section
                .and_then(|s| s.get("IMEI1").cloned())
                .or_else(|| device_section.and_then(|s| s.get("IMEI").cloned()))
                .filter(|v| !v.is_empty()),
            imei2: device_section
                .and_then(|s| s.get("IMEI2").cloned())
                .filter(|v| !v.is_empty()),
            iccid: device_section
                .and_then(|s| s.get("ICCID").cloned())
                .filter(|v| !v.is_empty()),
            os_version: device_section
                .and_then(|s| s.get("OS").cloned())
                .filter(|v| !v.is_empty()),
            serial_number: device_section
                .and_then(|s| s.get("SerialNumber").cloned())
                .filter(|v| !v.is_empty()),
        })
    } else {
        None
    };

    // Extract Extraction Info from [General] section
    let extraction_info = general_section.map(|s| ExtractionInfo {
        acquisition_tool: s.get("AcquisitionTool").cloned().filter(|v| !v.is_empty()),
        tool_version: s.get("Version").cloned().filter(|v| !v.is_empty()),
        unit_id: s.get("UnitId").cloned().filter(|v| !v.is_empty()),
        extraction_type: s.get("ExtractionType").cloned().filter(|v| !v.is_empty()),
        connection_type: s.get("ConnectionType").cloned().filter(|v| !v.is_empty()),
        start_time: s.get("Date").cloned().filter(|v| !v.is_empty()),
        end_time: s.get("EndTime").cloned().filter(|v| !v.is_empty()),
        guid: s.get("GUID").cloned().filter(|v| !v.is_empty()),
        machine_name: s.get("MachineName").cloned().filter(|v| !v.is_empty()),
    });

    // Get the extraction timestamp for hash records (use end time or start time)
    let hash_timestamp = extraction_info
        .as_ref()
        .and_then(|e| e.end_time.as_ref().or(e.start_time.as_ref()))
        .cloned();

    // Extract stored hashes from [SHA256], [SHA1], [MD5] sections
    let mut stored_hashes = Vec::new();

    for (algo, section_name) in [("SHA256", "SHA256"), ("SHA1", "SHA1"), ("MD5", "MD5")] {
        if let Some(section) = sections.get(section_name) {
            for (filename, hash) in section.iter() {
                stored_hashes.push(StoredHash {
                    filename: filename.clone(),
                    algorithm: algo.to_string(),
                    hash: hash.clone(),
                    timestamp: hash_timestamp.clone(),
                });
            }
        }
    }

    let stored_hashes = if stored_hashes.is_empty() {
        None
    } else {
        Some(stored_hashes)
    };

    Ok((case_info, device_info, extraction_info, stored_hashes))
}

/// Parse EvidenceCollection.ufdx (XML format)
///
/// UFDX files contain collection-level metadata about evidence and extractions.
pub fn parse_ufdx_file(path: &Path) -> Option<CollectionInfo> {
    let content = std::fs::read_to_string(path).ok()?;

    // Simple XML parsing - extract key attributes
    let evidence_id = extract_xml_attr(&content, "EvidenceID");
    let vendor = extract_xml_attr(&content, "Vendor");
    let model = extract_xml_attr(&content, "Model");
    let device_guid = extract_xml_attr(&content, "Guid");

    // Extract extraction paths
    let mut extractions = Vec::new();
    for line in content.lines() {
        if line.contains("<Extraction") && line.contains("Path=") {
            if let Some(path_val) = extract_xml_attr(line, "Path") {
                extractions.push(path_val);
            }
        }
    }

    Some(CollectionInfo {
        evidence_id,
        vendor,
        model,
        device_guid,
        extractions,
        ufdx_path: path.to_string_lossy().to_string(),
    })
}

/// Extract an XML attribute value from a string
pub fn extract_xml_attr(content: &str, attr_name: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr_name);
    let start = content.find(&pattern)?;
    let value_start = start + pattern.len();
    let value_end = content[value_start..].find('"')? + value_start;
    Some(content[value_start..value_end].to_string())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_xml_attr_simple() {
        let xml = r#"<Device Model="iPhone 12" Vendor="Apple"/>"#;
        assert_eq!(
            extract_xml_attr(xml, "Model"),
            Some("iPhone 12".to_string())
        );
        assert_eq!(extract_xml_attr(xml, "Vendor"), Some("Apple".to_string()));
    }

    #[test]
    fn test_extract_xml_attr_not_found() {
        let xml = r#"<Device Model="iPhone 12"/>"#;
        assert_eq!(extract_xml_attr(xml, "Missing"), None);
    }

    #[test]
    fn test_extract_xml_attr_multiline() {
        let xml = r#"
            <Evidence 
                EvidenceID="EVD-001"
                Guid="abc-123-def"
            />
        "#;
        assert_eq!(
            extract_xml_attr(xml, "EvidenceID"),
            Some("EVD-001".to_string())
        );
        assert_eq!(
            extract_xml_attr(xml, "Guid"),
            Some("abc-123-def".to_string())
        );
    }

    #[test]
    fn test_extract_xml_attr_empty_value() {
        let xml = r#"<Device Model="" Vendor="Apple"/>"#;
        assert_eq!(extract_xml_attr(xml, "Model"), Some("".to_string()));
    }

    #[test]
    fn test_extract_xml_attr_special_chars() {
        let xml = r#"<Device Path="/Users/test/file.txt" Name="Test &amp; File"/>"#;
        assert_eq!(
            extract_xml_attr(xml, "Path"),
            Some("/Users/test/file.txt".to_string())
        );
        // Note: This simple parser doesn't decode entities
        assert_eq!(
            extract_xml_attr(xml, "Name"),
            Some("Test &amp; File".to_string())
        );
    }

    #[test]
    fn test_extract_xml_attr_first_match() {
        // Should return first match when attribute appears multiple times
        let xml = r#"<Root><A Type="first"/><B Type="second"/></Root>"#;
        assert_eq!(extract_xml_attr(xml, "Type"), Some("first".to_string()));
    }
}
