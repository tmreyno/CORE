// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for normalized artifact persistence.

use super::with_project_db;
use crate::commands::artifacts::artifact_extract_source;
use crate::commands::hash::{open_hash_source, HashSourceInput};
use crate::common::{
    read_range_fully, ArtifactExtractionOptions, EvidenceByteSource, NormalizedArtifact,
};
use crate::project_db::{
    DbArtifactCategorySummary, DbArtifactEvidenceSummary, DbArtifactExtractorSummary,
    DbEvidenceFile, DbNormalizedArtifact,
};
use crate::viewer::document::database_viewer::get_database_info;
use notatin::cell_value::CellValue;
use notatin::parser_builder::ParserBuilder;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

#[derive(Clone, Copy)]
struct RegistryStringMapping {
    value_name: &'static str,
    metadata_key: &'static str,
}

const SQLITE_ARTIFACT_SOURCE_MAX_BYTES: u64 = 512 * 1024 * 1024;
const SQLITE_ARTIFACT_COPY_CHUNK_BYTES: usize = 1024 * 1024;
const SYSTEM_IDENTITY_SOURCE_MAX_BYTES: u64 = 256 * 1024;
const REGISTRY_IDENTITY_SOURCE_MAX_BYTES: u64 = 256 * 1024 * 1024;
const SQLITE_METADATA_NAME_LIMIT: usize = 12;
const MAX_SYSTEM_IDENTITY_LIST_ITEMS: usize = 32;
const DEFAULT_ARTIFACT_EXTRACTOR: &str = "core-artifact-extractor";
const MAX_ARTIFACT_EXTRACTOR_CHARS: usize = 128;

const WINDOWS_SYSTEM_INFORMATION_REGISTRY_VALUES: &[RegistryStringMapping] = &[
    RegistryStringMapping {
        value_name: "SystemManufacturer",
        metadata_key: "system.manufacturer",
    },
    RegistryStringMapping {
        value_name: "SystemProductName",
        metadata_key: "system.model",
    },
    RegistryStringMapping {
        value_name: "SystemProductVersion",
        metadata_key: "system.productVersion",
    },
    RegistryStringMapping {
        value_name: "SystemSKU",
        metadata_key: "system.sku",
    },
    RegistryStringMapping {
        value_name: "SystemFamily",
        metadata_key: "system.family",
    },
    RegistryStringMapping {
        value_name: "SystemSerialNumber",
        metadata_key: "system.serialNumber",
    },
    RegistryStringMapping {
        value_name: "BaseBoardManufacturer",
        metadata_key: "system.baseboardManufacturer",
    },
    RegistryStringMapping {
        value_name: "BaseBoardProduct",
        metadata_key: "system.baseboardProduct",
    },
    RegistryStringMapping {
        value_name: "BaseBoardVersion",
        metadata_key: "system.baseboardVersion",
    },
    RegistryStringMapping {
        value_name: "BIOSVendor",
        metadata_key: "system.biosVendor",
    },
    RegistryStringMapping {
        value_name: "BIOSVersion",
        metadata_key: "system.biosVersion",
    },
    RegistryStringMapping {
        value_name: "BIOSReleaseDate",
        metadata_key: "system.biosReleaseDate",
    },
    RegistryStringMapping {
        value_name: "SystemBiosVersion",
        metadata_key: "system.systemBiosVersion",
    },
    RegistryStringMapping {
        value_name: "VideoBiosVersion",
        metadata_key: "system.videoBiosVersion",
    },
    RegistryStringMapping {
        value_name: "ComputerHardwareId",
        metadata_key: "system.hardwareId",
    },
    RegistryStringMapping {
        value_name: "ComputerHardwareIds",
        metadata_key: "system.hardwareIds",
    },
];

const WINDOWS_SOFTWARE_CURRENT_VERSION_REGISTRY_VALUES: &[RegistryStringMapping] = &[
    RegistryStringMapping {
        value_name: "ProductName",
        metadata_key: "system.osName",
    },
    RegistryStringMapping {
        value_name: "DisplayVersion",
        metadata_key: "system.osDisplayVersion",
    },
    RegistryStringMapping {
        value_name: "CurrentBuild",
        metadata_key: "system.osBuild",
    },
    RegistryStringMapping {
        value_name: "CurrentBuildNumber",
        metadata_key: "system.osBuildNumber",
    },
    RegistryStringMapping {
        value_name: "EditionID",
        metadata_key: "system.osEdition",
    },
    RegistryStringMapping {
        value_name: "ProductId",
        metadata_key: "system.productId",
    },
    RegistryStringMapping {
        value_name: "RegisteredOwner",
        metadata_key: "system.registeredOwner",
    },
    RegistryStringMapping {
        value_name: "RegisteredOrganization",
        metadata_key: "system.registeredOrganization",
    },
];

const WINDOWS_SOFTWARE_OEM_INFORMATION_REGISTRY_VALUES: &[RegistryStringMapping] = &[
    RegistryStringMapping {
        value_name: "Manufacturer",
        metadata_key: "system.oemManufacturer",
    },
    RegistryStringMapping {
        value_name: "Model",
        metadata_key: "system.oemModel",
    },
    RegistryStringMapping {
        value_name: "SupportURL",
        metadata_key: "system.oemSupportUrl",
    },
];
const MAX_ARTIFACT_RESPONSE_ROWS: usize = 10_000;
const MAX_ARTIFACT_FIELD_CHARS: usize = 4096;
const MAX_ARTIFACT_PREVIEW_CHARS: usize = 16_384;
const MAX_ARTIFACT_JSON_CHARS: usize = 65_536;
const MAX_ARTIFACT_JSON_DEPTH: usize = 4;
const MAX_ARTIFACT_JSON_ITEMS: usize = 256;
const MAX_ARTIFACT_METADATA_ENTRIES: usize = 96;
const MAX_ARTIFACT_METADATA_VALUE_CHARS: usize = 384;
const ARTIFACT_TRUNCATED_SUFFIX: &str = "... [truncated]";

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDbExtractArtifactRequest {
    pub source: HashSourceInput,
    pub options: Option<ArtifactExtractionOptions>,
    pub evidence_file_id: Option<String>,
    pub evidence_file: Option<DbEvidenceFile>,
    pub extractor: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDbExtractArtifactResult {
    pub artifact: NormalizedArtifact,
    pub record: DbNormalizedArtifact,
}

/// Insert or replace a normalized artifact record.
#[tauri::command]
pub fn project_db_upsert_artifact(
    window: tauri::Window,
    artifact: DbNormalizedArtifact,
) -> Result<(), String> {
    with_project_db(window.label(), |db| db.upsert_artifact(&artifact))
}

/// Get a normalized artifact by ID.
#[tauri::command]
pub fn project_db_get_artifact(
    window: tauri::Window,
    id: String,
) -> Result<Option<DbNormalizedArtifact>, String> {
    with_project_db(window.label(), |db| db.get_artifact(&id))
        .map(|artifact| artifact.map(bounded_artifact_record_for_response))
}

/// List normalized artifacts across the active project.
#[tauri::command]
pub fn project_db_list_artifacts(
    window: tauri::Window,
    limit: Option<i64>,
) -> Result<Vec<DbNormalizedArtifact>, String> {
    with_project_db(window.label(), |db| db.list_artifacts(limit)).map(|artifacts| {
        artifacts
            .into_iter()
            .take(MAX_ARTIFACT_RESPONSE_ROWS)
            .map(bounded_artifact_record_for_response)
            .collect()
    })
}

/// List normalized artifacts for an evidence file.
#[tauri::command]
pub fn project_db_list_artifacts_for_evidence(
    window: tauri::Window,
    evidence_file_id: String,
) -> Result<Vec<DbNormalizedArtifact>, String> {
    with_project_db(window.label(), |db| {
        db.list_artifacts_for_evidence(&evidence_file_id)
    })
    .map(|artifacts| {
        artifacts
            .into_iter()
            .take(MAX_ARTIFACT_RESPONSE_ROWS)
            .map(bounded_artifact_record_for_response)
            .collect()
    })
}

/// List normalized artifacts by category.
#[tauri::command]
pub fn project_db_list_artifacts_by_category(
    window: tauri::Window,
    category: String,
    limit: Option<i64>,
) -> Result<Vec<DbNormalizedArtifact>, String> {
    with_project_db(window.label(), |db| {
        db.list_artifacts_by_category(&category, limit)
    })
    .map(|artifacts| {
        artifacts
            .into_iter()
            .take(MAX_ARTIFACT_RESPONSE_ROWS)
            .map(bounded_artifact_record_for_response)
            .collect()
    })
}

/// Summarize normalized artifacts by category.
#[tauri::command]
pub fn project_db_summarize_artifacts_by_category(
    window: tauri::Window,
) -> Result<Vec<DbArtifactCategorySummary>, String> {
    with_project_db(window.label(), |db| db.summarize_artifacts_by_category())
}

/// Summarize normalized artifacts by evidence file.
#[tauri::command]
pub fn project_db_summarize_artifacts_by_evidence(
    window: tauri::Window,
) -> Result<Vec<DbArtifactEvidenceSummary>, String> {
    with_project_db(window.label(), |db| db.summarize_artifacts_by_evidence())
}

/// Summarize normalized artifacts by extractor engine.
#[tauri::command]
pub fn project_db_summarize_artifacts_by_extractor(
    window: tauri::Window,
) -> Result<Vec<DbArtifactExtractorSummary>, String> {
    with_project_db(window.label(), |db| db.summarize_artifacts_by_extractor())
}

/// Extract a normalized artifact from a source and persist it to the active
/// project database.
#[tauri::command]
pub async fn project_db_extract_artifact_source(
    window: tauri::Window,
    request: ProjectDbExtractArtifactRequest,
) -> Result<ProjectDbExtractArtifactResult, String> {
    let ProjectDbExtractArtifactRequest {
        source,
        options,
        evidence_file_id,
        evidence_file,
        extractor,
    } = request;

    let source_for_enrichment = source.clone();
    let resolved_evidence_id = evidence_file
        .as_ref()
        .map(|file| file.id.clone())
        .or(evidence_file_id);
    let mut artifact = artifact_extract_source(source, options).await?;
    enrich_sqlite_artifact_metadata(&source_for_enrichment, &mut artifact).await?;
    enrich_system_identity_artifact_metadata(&source_for_enrichment, &mut artifact).await?;
    let record = normalized_to_db_artifact(
        &artifact,
        resolved_evidence_id,
        normalize_artifact_extractor(extractor),
    )?;

    with_project_db(window.label(), |db| {
        if let Some(file) = &evidence_file {
            db.upsert_evidence_file(file)?;
        }
        db.upsert_artifact(&record)?;
        Ok(())
    })?;

    Ok(ProjectDbExtractArtifactResult { artifact, record })
}

async fn enrich_system_identity_artifact_metadata(
    source: &HashSourceInput,
    artifact: &mut NormalizedArtifact,
) -> Result<(), String> {
    let source_id = source_identity_path(source, artifact);
    if !is_system_identity_source(&source_id) {
        return Ok(());
    }

    let source = source.clone();
    let metadata_result =
        tauri::async_runtime::spawn_blocking(move || system_identity_metadata_from_source(&source))
            .await
            .map_err(|e| format!("System identity artifact metadata task failed: {e}"))?;

    match metadata_result {
        Ok(metadata) if !metadata.is_empty() => {
            artifact.metadata.extend(metadata);
            artifact.category = "systeminfo".to_string();
            artifact.type_description = "System Identification Artifact".to_string();
            artifact.confidence = "high".to_string();
        }
        Ok(_) => {}
        Err(error) => {
            artifact.metadata.insert(
                "system.identityStatus".to_string(),
                "unavailable".to_string(),
            );
            artifact.metadata.insert(
                "system.identityError".to_string(),
                truncate_metadata_value(&error, 180),
            );
        }
    }

    Ok(())
}

fn source_identity_path(source: &HashSourceInput, artifact: &NormalizedArtifact) -> String {
    source
        .entry_path
        .as_deref()
        .or(source.path.as_deref())
        .unwrap_or(&artifact.source_id)
        .replace('\\', "/")
}

fn is_system_identity_source(source_id: &str) -> bool {
    let source_id = source_id.replace('\\', "/").to_ascii_lowercase();
    if source_id.ends_with("/library/preferences/systemconfiguration/preferences.plist")
        || source_id.ends_with("/system/library/coreservices/systemversion.plist")
        || source_id.ends_with("/library/preferences/systemconfiguration/com.apple.boot.plist")
        || source_id.ends_with("/library/preferences/systemconfiguration/networkinterfaces.plist")
        || source_id.ends_with("/windows/system32/config/system")
        || source_id.ends_with("/windows/system32/config/software")
        || source_id.ends_with("/config/system")
        || source_id.ends_with("/config/software")
    {
        return true;
    }

    matches!(
        source_id.rsplit('/').next(),
        Some(
            "os-release"
                | "lsb-release"
                | "redhat-release"
                | "debian_version"
                | "machine-id"
                | "hostname"
                | "product_uuid"
                | "product_serial"
                | "product_name"
                | "sys_vendor"
                | "board_serial"
                | "board_name"
                | "bios_version"
                | "bios_vendor"
        )
    )
}

fn system_identity_metadata_from_source(
    source: &HashSourceInput,
) -> Result<BTreeMap<String, String>, String> {
    let byte_source = open_hash_source(source)?;
    let source_ref = byte_source.source_ref();
    let source_id = source_ref.display_id();
    let size = byte_source.len().map_err(|e| e.to_string())?;
    if is_windows_registry_identity_source(&source_id) {
        return registry_identity_metadata_from_byte_source(byte_source.as_ref(), size, &source_id);
    }

    if size > SYSTEM_IDENTITY_SOURCE_MAX_BYTES {
        return Err(format!(
            "System identity source is too large for metadata extraction: {size} bytes > {SYSTEM_IDENTITY_SOURCE_MAX_BYTES} bytes"
        ));
    }

    let read_size = usize::try_from(size)
        .map_err(|_| format!("System identity source size does not fit in memory: {size}"))?;
    let data = if read_size > 0 {
        read_range_fully(byte_source.as_ref(), 0, read_size).map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };

    Ok(system_identity_metadata_from_bytes(&source_id, &data))
}

fn is_windows_registry_identity_source(source_id: &str) -> bool {
    let source_id = source_id.replace('\\', "/").to_ascii_lowercase();
    source_id.ends_with("/windows/system32/config/system")
        || source_id.ends_with("/windows/system32/config/software")
        || source_id.ends_with("/config/system")
        || source_id.ends_with("/config/software")
}

fn registry_identity_metadata_from_byte_source(
    byte_source: &dyn EvidenceByteSource,
    size: u64,
    source_id: &str,
) -> Result<BTreeMap<String, String>, String> {
    if size > REGISTRY_IDENTITY_SOURCE_MAX_BYTES {
        return Err(format!(
            "Registry identity source is too large for metadata extraction: {size} bytes > {REGISTRY_IDENTITY_SOURCE_MAX_BYTES} bytes"
        ));
    }

    let suffix = if source_id.to_ascii_lowercase().ends_with("software") {
        ".software.hive"
    } else {
        ".system.hive"
    };
    let mut temp = tempfile::Builder::new()
        .prefix("core-ffx-registry-identity-")
        .suffix(suffix)
        .tempfile()
        .map_err(|e| format!("Failed to create temporary registry identity copy: {e}"))?;
    copy_artifact_source(byte_source, size, source_id, &mut temp, "registry identity")?;
    temp.flush()
        .map_err(|e| format!("Failed to flush temporary registry identity copy: {e}"))?;
    temp.as_file()
        .sync_all()
        .map_err(|e| format!("Failed to sync temporary registry identity copy: {e}"))?;

    registry_identity_metadata_from_hive_path(temp.path(), source_id)
}

fn registry_identity_metadata_from_hive_path(
    hive_path: &Path,
    source_id: &str,
) -> Result<BTreeMap<String, String>, String> {
    let lower = source_id.replace('\\', "/").to_ascii_lowercase();
    if lower.ends_with("software") {
        registry_software_identity_metadata(hive_path)
    } else {
        registry_system_identity_metadata(hive_path)
    }
}

fn registry_system_identity_metadata(hive_path: &Path) -> Result<BTreeMap<String, String>, String> {
    let hive_path = hive_path.to_path_buf();
    let mut parser = ParserBuilder::from_path(hive_path)
        .build()
        .map_err(|e| format!("Failed to open SYSTEM registry hive: {e}"))?;
    let current_control_set = registry_current_control_set(&mut parser).unwrap_or(1);
    let control_set = format!("ControlSet{current_control_set:03}");
    let mut metadata = BTreeMap::new();

    insert_registry_string_value(
        &mut metadata,
        &mut parser,
        &format!("{control_set}\\Control\\ComputerName\\ComputerName"),
        "ComputerName",
        "system.computerName",
    );
    insert_registry_string_value(
        &mut metadata,
        &mut parser,
        &format!("{control_set}\\Services\\Tcpip\\Parameters"),
        "Hostname",
        "system.hostname",
    );
    insert_registry_string_value(
        &mut metadata,
        &mut parser,
        &format!("{control_set}\\Services\\Tcpip\\Parameters"),
        "Domain",
        "system.domain",
    );
    insert_registry_string_value(
        &mut metadata,
        &mut parser,
        &format!("{control_set}\\Services\\Tcpip\\Parameters"),
        "NV Hostname",
        "system.networkHostname",
    );
    insert_registry_string_value(
        &mut metadata,
        &mut parser,
        &format!("{control_set}\\Control\\TimeZoneInformation"),
        "TimeZoneKeyName",
        "system.timeZone",
    );
    insert_registry_string_value(
        &mut metadata,
        &mut parser,
        &format!("{control_set}\\Control\\TimeZoneInformation"),
        "StandardName",
        "system.timeZoneStandardName",
    );
    insert_registry_string_values(
        &mut metadata,
        &mut parser,
        &format!("{control_set}\\Control\\SystemInformation"),
        WINDOWS_SYSTEM_INFORMATION_REGISTRY_VALUES,
    );

    Ok(finalize_registry_identity_metadata(
        metadata,
        "windows.system",
    ))
}

fn registry_software_identity_metadata(
    hive_path: &Path,
) -> Result<BTreeMap<String, String>, String> {
    let hive_path = hive_path.to_path_buf();
    let mut parser = ParserBuilder::from_path(hive_path)
        .build()
        .map_err(|e| format!("Failed to open SOFTWARE registry hive: {e}"))?;
    let current_version = "Microsoft\\Windows NT\\CurrentVersion";
    let mut metadata = BTreeMap::new();

    insert_registry_string_values(
        &mut metadata,
        &mut parser,
        current_version,
        WINDOWS_SOFTWARE_CURRENT_VERSION_REGISTRY_VALUES,
    );
    insert_registry_string_value(
        &mut metadata,
        &mut parser,
        "Microsoft\\Cryptography",
        "MachineGuid",
        "system.machineGuid",
    );
    insert_registry_string_values(
        &mut metadata,
        &mut parser,
        "Microsoft\\Windows\\CurrentVersion\\OEMInformation",
        WINDOWS_SOFTWARE_OEM_INFORMATION_REGISTRY_VALUES,
    );

    Ok(finalize_registry_identity_metadata(
        metadata,
        "windows.software",
    ))
}

fn finalize_registry_identity_metadata(
    mut metadata: BTreeMap<String, String>,
    source: &str,
) -> BTreeMap<String, String> {
    if metadata.is_empty() {
        return metadata;
    }
    metadata.insert("system.identityStatus".to_string(), "parsed".to_string());
    metadata.insert("system.identityHive".to_string(), source.to_string());
    metadata
}

fn registry_current_control_set(parser: &mut notatin::parser::Parser) -> Option<u32> {
    registry_value(parser, "Select", "Current").and_then(|value| match value {
        CellValue::U32(value) => Some(value),
        CellValue::I32(value) => u32::try_from(value).ok(),
        CellValue::String(value) => value.trim().parse::<u32>().ok(),
        _ => None,
    })
}

fn insert_registry_string_value(
    metadata: &mut BTreeMap<String, String>,
    parser: &mut notatin::parser::Parser,
    key_path: &str,
    value_name: &str,
    metadata_key: &str,
) {
    let Some(value) = registry_value(parser, key_path, value_name).and_then(registry_value_text)
    else {
        return;
    };
    insert_trimmed_metadata(metadata, metadata_key, &value);
}

fn insert_registry_string_values(
    metadata: &mut BTreeMap<String, String>,
    parser: &mut notatin::parser::Parser,
    key_path: &str,
    mappings: &[RegistryStringMapping],
) {
    for mapping in mappings {
        insert_registry_string_value(
            metadata,
            parser,
            key_path,
            mapping.value_name,
            mapping.metadata_key,
        );
    }
}

fn registry_value(
    parser: &mut notatin::parser::Parser,
    key_path: &str,
    value_name: &str,
) -> Option<CellValue> {
    let key = parser.get_key(key_path, false).ok().flatten()?;
    key.value_iter().find_map(|value| {
        (value.detail.value_name().eq_ignore_ascii_case(value_name)).then(|| value.get_content().0)
    })
}

fn registry_value_text(value: CellValue) -> Option<String> {
    match value {
        CellValue::String(value) => Some(value),
        CellValue::MultiString(values) => Some(values.join("; ")),
        CellValue::U32(value) => Some(value.to_string()),
        CellValue::I32(value) => Some(value.to_string()),
        CellValue::U64(value) => Some(value.to_string()),
        CellValue::I64(value) => Some(value.to_string()),
        _ => None,
    }
}

fn system_identity_metadata_from_bytes(source_id: &str, data: &[u8]) -> BTreeMap<String, String> {
    let normalized = source_id.replace('\\', "/");
    let lower = normalized.to_ascii_lowercase();
    let file_name = lower.rsplit('/').next().unwrap_or("");
    let text = String::from_utf8_lossy(data);
    let mut metadata = BTreeMap::new();

    metadata.insert("system.identitySource".to_string(), normalized);

    match file_name {
        "os-release" | "lsb-release" => {
            metadata.extend(parse_linux_release_metadata(&text));
        }
        "redhat-release" | "debian_version" => {
            let value = text.trim();
            if !value.is_empty() {
                metadata.insert(
                    "system.osVersion".to_string(),
                    truncate_metadata_value(value, 180),
                );
            }
        }
        "machine-id" => {
            insert_trimmed_metadata(&mut metadata, "system.machineId", &text);
        }
        "hostname" => {
            insert_trimmed_metadata(&mut metadata, "system.hostname", &text);
        }
        "product_uuid" => {
            insert_trimmed_metadata(&mut metadata, "system.hardwareUuid", &text);
        }
        "product_serial" => {
            insert_trimmed_metadata(&mut metadata, "system.serialNumber", &text);
        }
        "product_name" => {
            insert_trimmed_metadata(&mut metadata, "system.model", &text);
        }
        "sys_vendor" => {
            insert_trimmed_metadata(&mut metadata, "system.manufacturer", &text);
        }
        "board_serial" => {
            insert_trimmed_metadata(&mut metadata, "system.boardSerial", &text);
        }
        "board_name" => {
            insert_trimmed_metadata(&mut metadata, "system.boardName", &text);
        }
        "bios_version" => {
            insert_trimmed_metadata(&mut metadata, "system.biosVersion", &text);
        }
        "bios_vendor" => {
            insert_trimmed_metadata(&mut metadata, "system.biosVendor", &text);
        }
        "networkinterfaces.plist" => {
            metadata.extend(parse_macos_network_interfaces_metadata(data));
        }
        "systemversion.plist" | "preferences.plist" | "com.apple.boot.plist" => {
            metadata.extend(parse_macos_plist_identity_metadata(data));
        }
        _ => {}
    }

    if metadata.len() == 1 {
        metadata.clear();
    } else {
        metadata.insert("system.identityStatus".to_string(), "parsed".to_string());
    }

    metadata
}

fn parse_linux_release_metadata(text: &str) -> BTreeMap<String, String> {
    let values = parse_key_value_lines(text);
    let mut metadata = BTreeMap::new();

    if let Some(value) = values.get("NAME") {
        metadata.insert("system.osName".to_string(), value.clone());
    }
    if let Some(value) = values
        .get("VERSION_ID")
        .or_else(|| values.get("DISTRIB_RELEASE"))
    {
        metadata.insert("system.osVersion".to_string(), value.clone());
    }
    if let Some(value) = values
        .get("VERSION")
        .or_else(|| values.get("DISTRIB_DESCRIPTION"))
    {
        metadata.insert("system.osVersionDetail".to_string(), value.clone());
    }
    if let Some(value) = values.get("ID") {
        metadata.insert("system.osId".to_string(), value.clone());
    }
    if let Some(value) = values.get("PRETTY_NAME") {
        metadata.insert("system.osPrettyName".to_string(), value.clone());
    }

    metadata
}

fn parse_key_value_lines(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            let value = value.trim().trim_matches('"').trim_matches('\'').trim();
            if key.trim().is_empty() || value.is_empty() {
                return None;
            }
            Some((
                key.trim().to_string(),
                truncate_metadata_value(value, MAX_ARTIFACT_METADATA_VALUE_CHARS),
            ))
        })
        .collect()
}

fn parse_macos_plist_identity_metadata(data: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let Ok(value) = plist::from_bytes::<plist::Value>(data) else {
        return metadata;
    };

    insert_plist_string(&mut metadata, &value, "ProductName", "system.osName");
    insert_plist_string(&mut metadata, &value, "ProductVersion", "system.osVersion");
    insert_plist_string(
        &mut metadata,
        &value,
        "ProductBuildVersion",
        "system.osBuild",
    );
    insert_plist_string(&mut metadata, &value, "ComputerName", "system.computerName");
    insert_plist_string(&mut metadata, &value, "HostName", "system.hostname");
    insert_plist_string(
        &mut metadata,
        &value,
        "LocalHostName",
        "system.localHostname",
    );
    insert_plist_string(&mut metadata, &value, "HardwareUUID", "system.hardwareUuid");
    insert_plist_string(
        &mut metadata,
        &value,
        "IOPlatformUUID",
        "system.hardwareUuid",
    );
    insert_plist_string(&mut metadata, &value, "SerialNumber", "system.serialNumber");

    metadata
}

fn parse_macos_network_interfaces_metadata(data: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let Ok(value) = plist::from_bytes::<plist::Value>(data) else {
        return metadata;
    };
    let Some(interfaces) = find_plist_array(&value, "Interfaces") else {
        return metadata;
    };

    let mut descriptions = Vec::new();
    let mut mac_addresses = Vec::new();
    for interface in interfaces.iter().take(MAX_SYSTEM_IDENTITY_LIST_ITEMS) {
        let plist::Value::Dictionary(interface) = interface else {
            continue;
        };
        let bsd_name = plist_dict_string(interface, "BSD Name");
        let interface_type = plist_dict_string(interface, "SCNetworkInterfaceType")
            .or_else(|| plist_dict_string(interface, "SCNetworkInterfaceSubType"));
        let display_name = interface
            .get("SCNetworkInterfaceInfo")
            .and_then(|value| match value {
                plist::Value::Dictionary(info) => plist_dict_string(info, "UserDefinedName"),
                _ => None,
            });
        let mac_address = interface
            .get("IOMACAddress")
            .and_then(plist_data_mac_address);

        if let Some(mac_address) = &mac_address {
            push_unique_limited(&mut mac_addresses, mac_address.clone());
        }
        let Some(description) = describe_macos_network_interface(
            bsd_name,
            display_name,
            interface_type,
            mac_address.as_deref(),
        ) else {
            continue;
        };
        push_unique_limited(&mut descriptions, description);
    }

    if !descriptions.is_empty() {
        metadata.insert(
            "system.networkInterfaceCount".to_string(),
            descriptions.len().to_string(),
        );
        metadata.insert(
            "system.networkInterfaces".to_string(),
            truncate_metadata_value(&descriptions.join("; "), MAX_ARTIFACT_METADATA_VALUE_CHARS),
        );
    }
    if let Some(primary) = mac_addresses.first() {
        metadata.insert("system.primaryMacAddress".to_string(), primary.clone());
    }
    if !mac_addresses.is_empty() {
        metadata.insert(
            "system.macAddresses".to_string(),
            truncate_metadata_value(&mac_addresses.join("; "), MAX_ARTIFACT_METADATA_VALUE_CHARS),
        );
    }

    metadata
}

fn describe_macos_network_interface(
    bsd_name: Option<&str>,
    display_name: Option<&str>,
    interface_type: Option<&str>,
    mac_address: Option<&str>,
) -> Option<String> {
    let name = bsd_name.or(display_name)?;
    let mut parts = Vec::new();
    if let Some(display_name) = display_name.filter(|value| Some(*value) != bsd_name) {
        parts.push(display_name.to_string());
    }
    if let Some(interface_type) = interface_type {
        parts.push(interface_type.to_string());
    }
    if let Some(mac_address) = mac_address {
        parts.push(mac_address.to_string());
    }

    if parts.is_empty() {
        Some(name.to_string())
    } else {
        Some(format!("{} ({})", name, parts.join(", ")))
    }
}

fn insert_plist_string(
    metadata: &mut BTreeMap<String, String>,
    value: &plist::Value,
    plist_key: &str,
    metadata_key: &str,
) {
    if metadata.contains_key(metadata_key) {
        return;
    }
    if let Some(found) = find_plist_string(value, plist_key) {
        metadata.insert(
            metadata_key.to_string(),
            truncate_metadata_value(found, MAX_ARTIFACT_METADATA_VALUE_CHARS),
        );
    }
}

fn find_plist_string<'a>(value: &'a plist::Value, key: &str) -> Option<&'a str> {
    match value {
        plist::Value::Dictionary(dict) => {
            if let Some(value) = dict.get(key).and_then(plist::Value::as_string) {
                return Some(value);
            }
            dict.values()
                .find_map(|value| find_plist_string(value, key))
        }
        plist::Value::Array(values) => values
            .iter()
            .find_map(|value| find_plist_string(value, key)),
        _ => None,
    }
}

fn find_plist_array<'a>(value: &'a plist::Value, key: &str) -> Option<&'a Vec<plist::Value>> {
    match value {
        plist::Value::Dictionary(dict) => {
            if let Some(plist::Value::Array(value)) = dict.get(key) {
                return Some(value);
            }
            dict.values().find_map(|value| find_plist_array(value, key))
        }
        plist::Value::Array(values) => values.iter().find_map(|value| find_plist_array(value, key)),
        _ => None,
    }
}

fn plist_dict_string<'a>(dict: &'a plist::Dictionary, key: &str) -> Option<&'a str> {
    dict.get(key).and_then(plist::Value::as_string)
}

fn plist_data_mac_address(value: &plist::Value) -> Option<String> {
    let plist::Value::Data(data) = value else {
        return None;
    };
    format_mac_address(data)
}

fn format_mac_address(data: &[u8]) -> Option<String> {
    if data.len() != 6 {
        return None;
    }
    Some(
        data.iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<_>>()
            .join(":"),
    )
}

fn push_unique_limited(values: &mut Vec<String>, value: String) {
    if values.len() >= MAX_SYSTEM_IDENTITY_LIST_ITEMS || values.contains(&value) {
        return;
    }
    values.push(value);
}

fn insert_trimmed_metadata(metadata: &mut BTreeMap<String, String>, key: &str, value: &str) {
    let value = value.trim();
    if value.is_empty() {
        return;
    }
    metadata.insert(key.to_string(), truncate_metadata_value(value, 180));
}

async fn enrich_sqlite_artifact_metadata(
    source: &HashSourceInput,
    artifact: &mut NormalizedArtifact,
) -> Result<(), String> {
    if !is_sqlite_artifact(artifact) {
        return Ok(());
    }

    let source = source.clone();
    let metadata_result =
        tauri::async_runtime::spawn_blocking(move || sqlite_artifact_metadata_from_source(&source))
            .await
            .map_err(|e| format!("SQLite artifact metadata task failed: {e}"))?;

    match metadata_result {
        Ok(metadata) => {
            artifact.metadata.extend(metadata);
        }
        Err(error) => {
            artifact
                .metadata
                .insert("sqlite.schemaStatus".to_string(), "unavailable".to_string());
            artifact.metadata.insert(
                "sqlite.schemaError".to_string(),
                truncate_metadata_value(&error, 180),
            );
        }
    }

    Ok(())
}

fn is_sqlite_artifact(artifact: &NormalizedArtifact) -> bool {
    artifact.category == "database"
        || artifact.mime_type.as_deref() == Some("application/x-sqlite3")
        || matches!(
            artifact.extension.as_deref(),
            Some("db" | "sqlite" | "sqlite3" | "sqlitedb")
        )
}

fn sqlite_artifact_metadata_from_source(
    source: &HashSourceInput,
) -> Result<BTreeMap<String, String>, String> {
    with_sqlite_artifact_source(source, |path, _source_id| {
        let info = get_database_info(path).map_err(|e| e.to_string())?;
        let mut metadata = BTreeMap::new();

        let user_tables: Vec<_> = info
            .tables
            .iter()
            .filter(|table| !table.is_system)
            .collect();
        let system_tables: Vec<_> = info.tables.iter().filter(|table| table.is_system).collect();
        let total_rows = sqlite_total_rows(&info.tables);

        metadata.insert(
            "sqlite.tableCount".to_string(),
            info.tables.len().to_string(),
        );
        metadata.insert("sqlite.viewCount".to_string(), info.views.len().to_string());
        metadata.insert(
            "sqlite.userTableCount".to_string(),
            user_tables.len().to_string(),
        );
        metadata.insert(
            "sqlite.systemTableCount".to_string(),
            system_tables.len().to_string(),
        );
        metadata.insert("sqlite.totalRows".to_string(), total_rows.to_string());
        metadata.insert("sqlite.journalMode".to_string(), info.journal_mode);
        metadata.insert("sqlite.sqliteVersion".to_string(), info.sqlite_version);

        if !info.tables.is_empty() {
            metadata.insert(
                "sqlite.tableNames".to_string(),
                limited_names(info.tables.iter().map(|table| table.name.as_str())),
            );
            metadata.insert("sqlite.tables".to_string(), table_summaries(&info.tables));
        }
        if !info.views.is_empty() {
            metadata.insert(
                "sqlite.viewNames".to_string(),
                limited_names(info.views.iter().map(String::as_str)),
            );
        }
        if let Some(largest) = info.tables.iter().max_by_key(|table| table.row_count) {
            metadata.insert(
                "sqlite.largestTable".to_string(),
                format!("{} ({} rows)", largest.name, largest.row_count),
            );
        }

        Ok(metadata)
    })
}

fn with_sqlite_artifact_source<T>(
    source: &HashSourceInput,
    operation: impl FnOnce(&Path, String) -> Result<T, String>,
) -> Result<T, String> {
    let byte_source = open_hash_source(source)?;
    let source_id = byte_source.source_ref().display_id();
    let size = byte_source.len().map_err(|e| e.to_string())?;
    if size > SQLITE_ARTIFACT_SOURCE_MAX_BYTES {
        return Err(format!(
            "SQLite artifact source is too large for schema extraction: {size} bytes > {SQLITE_ARTIFACT_SOURCE_MAX_BYTES} bytes"
        ));
    }

    let mut temp = tempfile::Builder::new()
        .prefix("core-ffx-artifact-db-")
        .suffix(".sqlite")
        .tempfile()
        .map_err(|e| format!("Failed to create temporary SQLite artifact copy: {e}"))?;
    copy_sqlite_artifact_source(byte_source.as_ref(), size, &mut temp)?;
    temp.flush()
        .map_err(|e| format!("Failed to flush temporary SQLite artifact copy: {e}"))?;
    temp.as_file()
        .sync_all()
        .map_err(|e| format!("Failed to sync temporary SQLite artifact copy: {e}"))?;

    operation(temp.path(), source_id)
}

fn copy_sqlite_artifact_source(
    byte_source: &dyn EvidenceByteSource,
    expected_size: u64,
    writer: &mut impl Write,
) -> Result<(), String> {
    let source_id = byte_source.source_ref().display_id();
    copy_artifact_source(
        byte_source,
        expected_size,
        &source_id,
        writer,
        "SQLite artifact",
    )
}

fn copy_artifact_source(
    byte_source: &dyn EvidenceByteSource,
    expected_size: u64,
    source_id: &str,
    writer: &mut impl Write,
    label: &str,
) -> Result<(), String> {
    let mut offset = 0u64;

    while offset < expected_size {
        let remaining = expected_size - offset;
        let read_size = remaining.min(SQLITE_ARTIFACT_COPY_CHUNK_BYTES as u64) as usize;
        let chunk = byte_source.read_range(offset, read_size).map_err(|e| {
            format!("Failed to read {label} source {source_id} at offset {offset}: {e}")
        })?;

        if chunk.is_empty() {
            return Err(format!(
                "Short read materializing {label} source {source_id}: expected {expected_size} bytes but read {offset} bytes"
            ));
        }
        if chunk.len() as u64 > remaining {
            return Err(format!(
                "Invalid oversized read materializing {label} source {source_id}: {} bytes returned with {remaining} bytes remaining",
                chunk.len()
            ));
        }

        writer.write_all(&chunk).map_err(|e| {
            format!("Failed to write {label} source {source_id} at offset {offset}: {e}")
        })?;
        offset = checked_sqlite_copy_offset_add(offset, chunk.len(), source_id)?;
    }

    Ok(())
}

fn checked_sqlite_copy_offset_add(
    offset: u64,
    bytes_read: usize,
    source_id: &str,
) -> Result<u64, String> {
    let bytes_read = u64::try_from(bytes_read).map_err(|_| {
        format!(
            "SQLite artifact source {source_id} returned a chunk length that does not fit in u64"
        )
    })?;
    offset.checked_add(bytes_read).ok_or_else(|| {
        format!(
            "SQLite artifact copy offset overflow for {source_id}: offset {offset} + {bytes_read} bytes"
        )
    })
}

fn table_summaries(tables: &[crate::viewer::document::database_viewer::TableSummary]) -> String {
    let mut values: Vec<String> = tables
        .iter()
        .take(SQLITE_METADATA_NAME_LIMIT)
        .map(|table| {
            format!(
                "{} ({} rows, {} cols)",
                table.name, table.row_count, table.column_count
            )
        })
        .collect();
    if tables.len() > SQLITE_METADATA_NAME_LIMIT {
        values.push(format!(
            "{} more table(s)",
            tables.len() - SQLITE_METADATA_NAME_LIMIT
        ));
    }
    values.join(", ")
}

fn sqlite_total_rows(tables: &[crate::viewer::document::database_viewer::TableSummary]) -> i64 {
    tables.iter().fold(0i64, |total, table| {
        total.saturating_add(table.row_count.max(0))
    })
}

fn limited_names<'a>(names: impl Iterator<Item = &'a str>) -> String {
    let values: Vec<&str> = names.take(SQLITE_METADATA_NAME_LIMIT).collect();
    values.join(", ")
}

fn truncate_metadata_value(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let mut truncated: String = value.chars().take(max_chars).collect();
    truncated.push_str("...");
    truncated
}

fn normalized_to_db_artifact(
    artifact: &NormalizedArtifact,
    evidence_file_id: Option<String>,
    extractor: String,
) -> Result<DbNormalizedArtifact, String> {
    let extractor = normalize_artifact_extractor(Some(extractor));
    let source_ref_json = serde_json::to_string(&artifact.source_ref)
        .map_err(|e| format!("Failed to serialize artifact source ref: {e}"))?;
    let metadata = bounded_artifact_metadata(&artifact.metadata);
    let metadata_json = serde_json::to_string(&metadata)
        .map_err(|e| format!("Failed to serialize artifact metadata: {e}"))?;

    let record = DbNormalizedArtifact {
        id: truncate_chars_with_suffix(&artifact.id, MAX_ARTIFACT_FIELD_CHARS),
        evidence_file_id: evidence_file_id
            .map(|value| truncate_chars_with_suffix(&value, MAX_ARTIFACT_FIELD_CHARS)),
        source_id: truncate_chars_with_suffix(&artifact.source_id, MAX_ARTIFACT_FIELD_CHARS),
        source_ref_json: bounded_artifact_json_text(&source_ref_json, MAX_ARTIFACT_JSON_CHARS),
        name: truncate_chars_with_suffix(&artifact.name, MAX_ARTIFACT_FIELD_CHARS),
        extension: artifact
            .extension
            .as_ref()
            .map(|value| truncate_chars_with_suffix(value, MAX_ARTIFACT_FIELD_CHARS)),
        size: artifact_size_to_i64(artifact.size)?,
        mime_type: artifact
            .mime_type
            .as_ref()
            .map(|value| truncate_chars_with_suffix(value, MAX_ARTIFACT_FIELD_CHARS)),
        type_description: truncate_chars_with_suffix(
            &artifact.type_description,
            MAX_ARTIFACT_FIELD_CHARS,
        ),
        category: truncate_chars_with_suffix(&artifact.category, MAX_ARTIFACT_FIELD_CHARS),
        confidence: truncate_chars_with_suffix(&artifact.confidence, MAX_ARTIFACT_FIELD_CHARS),
        is_text: artifact.is_text,
        content_preview: artifact
            .content_preview
            .as_ref()
            .map(|value| truncate_chars_with_suffix(value, MAX_ARTIFACT_PREVIEW_CHARS)),
        metadata_json: Some(bounded_artifact_json_text(
            &metadata_json,
            MAX_ARTIFACT_JSON_CHARS,
        )),
        extracted_at: chrono::Utc::now().to_rfc3339(),
        extractor,
    };

    Ok(record)
}

fn artifact_size_to_i64(size: u64) -> Result<i64, String> {
    i64::try_from(size).map_err(|_| format!("Artifact size exceeds project DB range: {size} bytes"))
}

fn normalize_artifact_extractor(extractor: Option<String>) -> String {
    let value = extractor
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_ARTIFACT_EXTRACTOR);

    truncate_chars(value, MAX_ARTIFACT_EXTRACTOR_CHARS)
}

fn bounded_artifact_record_for_response(
    mut artifact: DbNormalizedArtifact,
) -> DbNormalizedArtifact {
    artifact.id = truncate_chars_with_suffix(&artifact.id, MAX_ARTIFACT_FIELD_CHARS);
    artifact.evidence_file_id = artifact
        .evidence_file_id
        .map(|value| truncate_chars_with_suffix(&value, MAX_ARTIFACT_FIELD_CHARS));
    artifact.source_id = truncate_chars_with_suffix(&artifact.source_id, MAX_ARTIFACT_FIELD_CHARS);
    artifact.source_ref_json =
        bounded_artifact_json_text(&artifact.source_ref_json, MAX_ARTIFACT_JSON_CHARS);
    artifact.name = truncate_chars_with_suffix(&artifact.name, MAX_ARTIFACT_FIELD_CHARS);
    artifact.extension = artifact
        .extension
        .map(|value| truncate_chars_with_suffix(&value, MAX_ARTIFACT_FIELD_CHARS));
    artifact.mime_type = artifact
        .mime_type
        .map(|value| truncate_chars_with_suffix(&value, MAX_ARTIFACT_FIELD_CHARS));
    artifact.type_description =
        truncate_chars_with_suffix(&artifact.type_description, MAX_ARTIFACT_FIELD_CHARS);
    artifact.category = truncate_chars_with_suffix(&artifact.category, MAX_ARTIFACT_FIELD_CHARS);
    artifact.confidence =
        truncate_chars_with_suffix(&artifact.confidence, MAX_ARTIFACT_FIELD_CHARS);
    artifact.content_preview = artifact
        .content_preview
        .map(|value| truncate_chars_with_suffix(&value, MAX_ARTIFACT_PREVIEW_CHARS));
    artifact.metadata_json = artifact
        .metadata_json
        .map(|value| bounded_artifact_json_text(&value, MAX_ARTIFACT_JSON_CHARS));
    artifact.extracted_at =
        truncate_chars_with_suffix(&artifact.extracted_at, MAX_ARTIFACT_FIELD_CHARS);
    artifact.extractor = truncate_chars_with_suffix(&artifact.extractor, MAX_ARTIFACT_FIELD_CHARS);
    artifact
}

fn bounded_artifact_metadata(metadata: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    metadata
        .iter()
        .take(MAX_ARTIFACT_METADATA_ENTRIES)
        .map(|(key, value)| {
            (
                truncate_chars_with_suffix(key, MAX_ARTIFACT_FIELD_CHARS),
                truncate_chars_with_suffix(value, MAX_ARTIFACT_METADATA_VALUE_CHARS),
            )
        })
        .collect()
}

fn truncate_chars_with_suffix(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let suffix_chars = ARTIFACT_TRUNCATED_SUFFIX.chars().count();
    let keep_chars = max_chars.saturating_sub(suffix_chars);
    let end = value
        .char_indices()
        .nth(keep_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len());
    let mut truncated = String::with_capacity(end + ARTIFACT_TRUNCATED_SUFFIX.len());
    truncated.push_str(&value[..end]);
    truncated.push_str(ARTIFACT_TRUNCATED_SUFFIX);
    truncated
}

fn bounded_artifact_json_text(value: &str, max_chars: usize) -> String {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(value) else {
        return if value.chars().count() <= max_chars {
            value.to_string()
        } else {
            truncate_chars_with_suffix(value, max_chars)
        };
    };
    let bounded = bounded_artifact_json_value(json, 0);
    let Ok(serialized) = serde_json::to_string(&bounded) else {
        return "{\"truncated\":true}".to_string();
    };

    if serialized.chars().count() <= max_chars {
        serialized
    } else {
        "{\"truncated\":true}".to_string()
    }
}

fn bounded_artifact_json_value(value: serde_json::Value, depth: usize) -> serde_json::Value {
    if depth >= MAX_ARTIFACT_JSON_DEPTH {
        return serde_json::Value::String(ARTIFACT_TRUNCATED_SUFFIX.to_string());
    }

    match value {
        serde_json::Value::String(value) => {
            serde_json::Value::String(truncate_chars_with_suffix(&value, MAX_ARTIFACT_FIELD_CHARS))
        }
        serde_json::Value::Array(values) => serde_json::Value::Array(
            values
                .into_iter()
                .take(MAX_ARTIFACT_JSON_ITEMS)
                .map(|value| bounded_artifact_json_value(value, depth + 1))
                .collect(),
        ),
        serde_json::Value::Object(object) => {
            let mut bounded = serde_json::Map::new();
            for (key, value) in object.into_iter().take(MAX_ARTIFACT_JSON_ITEMS) {
                bounded.insert(
                    truncate_chars_with_suffix(&key, MAX_ARTIFACT_FIELD_CHARS),
                    bounded_artifact_json_value(value, depth + 1),
                );
            }
            serde_json::Value::Object(bounded)
        }
        value @ (serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)) => value,
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let end = value
        .char_indices()
        .nth(max_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len());
    value[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{EvidenceSourceError, EvidenceSourceRef, EvidenceSourceResult};
    use rusqlite::Connection;
    use std::collections::BTreeMap;

    struct TestByteSource {
        source_ref: EvidenceSourceRef,
        declared_len: u64,
        data: Vec<u8>,
        max_chunk: usize,
    }

    impl TestByteSource {
        fn new(declared_len: u64, data: &[u8], max_chunk: usize) -> Self {
            Self {
                source_ref: EvidenceSourceRef::LocalFile {
                    path: "test-source.sqlite".to_string(),
                },
                declared_len,
                data: data.to_vec(),
                max_chunk,
            }
        }
    }

    impl EvidenceByteSource for TestByteSource {
        fn source_ref(&self) -> EvidenceSourceRef {
            self.source_ref.clone()
        }

        fn len(&self) -> EvidenceSourceResult<u64> {
            Ok(self.declared_len)
        }

        fn read_range(&self, offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>> {
            if offset > self.declared_len {
                return Err(EvidenceSourceError::InvalidRange {
                    source_id: self.source_ref.display_id(),
                    offset,
                    size: self.declared_len,
                });
            }

            let start = offset as usize;
            if start >= self.data.len() {
                return Ok(Vec::new());
            }
            let read_size = size.min(self.max_chunk);
            let end = start.saturating_add(read_size).min(self.data.len());
            Ok(self.data[start..end].to_vec())
        }
    }

    #[test]
    fn normalized_to_db_artifact_preserves_core_fields() {
        let artifact = NormalizedArtifact {
            id: "artifact-1".to_string(),
            source_ref: EvidenceSourceRef::LocalFile {
                path: "/case/a.txt".to_string(),
            },
            source_id: "/case/a.txt".to_string(),
            name: "a.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 42,
            mime_type: Some("text/plain".to_string()),
            type_description: "Plain Text".to_string(),
            category: "text".to_string(),
            confidence: "high".to_string(),
            is_text: true,
            content_preview: Some("hello".to_string()),
            metadata: BTreeMap::from([("k".to_string(), "v".to_string())]),
        };

        let record = normalized_to_db_artifact(
            &artifact,
            Some("ev-1".to_string()),
            "test-extractor".to_string(),
        )
        .unwrap();

        assert_eq!(record.id, "artifact-1");
        assert_eq!(record.evidence_file_id.as_deref(), Some("ev-1"));
        assert_eq!(record.source_id, "/case/a.txt");
        assert_eq!(record.category, "text");
        assert_eq!(record.extractor, "test-extractor");
        assert!(record.source_ref_json.contains("localFile"));
        assert!(record.metadata_json.unwrap().contains("\"k\":\"v\""));
    }

    #[test]
    fn system_identity_metadata_extracts_linux_os_release() {
        let metadata = system_identity_metadata_from_bytes(
            "/mnt/image/etc/os-release",
            br#"NAME="Ubuntu"
VERSION_ID="24.04"
VERSION="24.04.2 LTS (Noble Numbat)"
ID=ubuntu
PRETTY_NAME="Ubuntu 24.04.2 LTS"
"#,
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("system.osName").map(String::as_str),
            Some("Ubuntu")
        );
        assert_eq!(
            metadata.get("system.osVersion").map(String::as_str),
            Some("24.04")
        );
        assert_eq!(
            metadata.get("system.osId").map(String::as_str),
            Some("ubuntu")
        );
        assert_eq!(
            metadata.get("system.osPrettyName").map(String::as_str),
            Some("Ubuntu 24.04.2 LTS")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_linux_dmi_values() {
        let serial = system_identity_metadata_from_bytes(
            "/image/sys/class/dmi/id/product_serial",
            b"ABC123\n",
        );
        let vendor = system_identity_metadata_from_bytes(
            "/image/sys/class/dmi/id/sys_vendor",
            b"Dell Inc.\n",
        );

        assert_eq!(
            serial.get("system.serialNumber").map(String::as_str),
            Some("ABC123")
        );
        assert_eq!(
            vendor.get("system.manufacturer").map(String::as_str),
            Some("Dell Inc.")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_macos_system_version_plist() {
        let metadata = system_identity_metadata_from_bytes(
            "/System/Library/CoreServices/SystemVersion.plist",
            br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>ProductName</key><string>macOS</string>
  <key>ProductVersion</key><string>15.5</string>
  <key>ProductBuildVersion</key><string>24F74</string>
</dict>
</plist>
"#,
        );

        assert_eq!(
            metadata.get("system.osName").map(String::as_str),
            Some("macOS")
        );
        assert_eq!(
            metadata.get("system.osVersion").map(String::as_str),
            Some("15.5")
        );
        assert_eq!(
            metadata.get("system.osBuild").map(String::as_str),
            Some("24F74")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_macos_network_interfaces() {
        let metadata = system_identity_metadata_from_bytes(
            "/Library/Preferences/SystemConfiguration/NetworkInterfaces.plist",
            br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Interfaces</key>
  <array>
    <dict>
      <key>BSD Name</key><string>en0</string>
      <key>IOMACAddress</key><data>ABEiM0RV</data>
      <key>SCNetworkInterfaceInfo</key>
      <dict>
        <key>UserDefinedName</key><string>Wi-Fi</string>
      </dict>
      <key>SCNetworkInterfaceType</key><string>IEEE80211</string>
    </dict>
    <dict>
      <key>BSD Name</key><string>en1</string>
      <key>IOMACAddress</key><data>ZneImaq7</data>
      <key>SCNetworkInterfaceType</key><string>Ethernet</string>
    </dict>
  </array>
</dict>
</plist>
"#,
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata
                .get("system.networkInterfaceCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata.get("system.primaryMacAddress").map(String::as_str),
            Some("00:11:22:33:44:55")
        );
        assert_eq!(
            metadata.get("system.macAddresses").map(String::as_str),
            Some("00:11:22:33:44:55; 66:77:88:99:aa:bb")
        );
        assert!(metadata
            .get("system.networkInterfaces")
            .is_some_and(|value| value.contains("en0 (Wi-Fi, IEEE80211, 00:11:22:33:44:55)")));
    }

    #[test]
    fn system_identity_source_classifier_matches_known_identity_files() {
        assert!(is_system_identity_source("/Windows/System32/config/SYSTEM"));
        assert!(is_system_identity_source(
            "/Windows/System32/config/SOFTWARE"
        ));
        assert!(is_system_identity_source("/etc/machine-id"));
        assert!(is_system_identity_source("/sys/class/dmi/id/product_uuid"));
        assert!(is_system_identity_source(
            "/System/Library/CoreServices/SystemVersion.plist"
        ));
        assert!(is_system_identity_source(
            "/Library/Preferences/SystemConfiguration/preferences.plist"
        ));
        assert!(is_system_identity_source(
            "/Library/Preferences/SystemConfiguration/NetworkInterfaces.plist"
        ));
        assert!(!is_system_identity_source(
            "/Users/test/Documents/notes.txt"
        ));
        assert!(!is_system_identity_source(
            "/Users/test/Library/Preferences/preferences.plist"
        ));
    }

    #[test]
    fn registry_identity_source_classifier_matches_windows_hives() {
        assert!(is_windows_registry_identity_source(
            "ad1:/Windows/System32/config/SYSTEM"
        ));
        assert!(is_windows_registry_identity_source(
            "e01:/Windows/System32/config/SOFTWARE"
        ));
        assert!(!is_windows_registry_identity_source(
            "ad1:/Windows/System32/config/SAM"
        ));
        assert!(!is_windows_registry_identity_source(
            "ad1:/Users/test/NTUSER.DAT"
        ));
    }

    #[test]
    fn registry_value_text_converts_scalar_identity_values() {
        assert_eq!(
            registry_value_text(CellValue::String("Windows 11 Pro".to_string())).as_deref(),
            Some("Windows 11 Pro")
        );
        assert_eq!(
            registry_value_text(CellValue::MultiString(vec![
                "example".to_string(),
                "local".to_string()
            ]))
            .as_deref(),
            Some("example; local")
        );
        assert_eq!(
            registry_value_text(CellValue::U32(22631)).as_deref(),
            Some("22631")
        );
        assert!(registry_value_text(CellValue::Binary(vec![1, 2, 3])).is_none());
    }

    #[test]
    fn windows_registry_identity_mappings_cover_hardware_and_oem_values() {
        let system_values: BTreeMap<&str, &str> = WINDOWS_SYSTEM_INFORMATION_REGISTRY_VALUES
            .iter()
            .map(|mapping| (mapping.value_name, mapping.metadata_key))
            .collect();
        assert_eq!(
            system_values.get("SystemManufacturer").copied(),
            Some("system.manufacturer")
        );
        assert_eq!(
            system_values.get("SystemProductName").copied(),
            Some("system.model")
        );
        assert_eq!(
            system_values.get("SystemSerialNumber").copied(),
            Some("system.serialNumber")
        );
        assert_eq!(
            system_values.get("ComputerHardwareIds").copied(),
            Some("system.hardwareIds")
        );

        let oem_values: BTreeMap<&str, &str> = WINDOWS_SOFTWARE_OEM_INFORMATION_REGISTRY_VALUES
            .iter()
            .map(|mapping| (mapping.value_name, mapping.metadata_key))
            .collect();
        assert_eq!(
            oem_values.get("Manufacturer").copied(),
            Some("system.oemManufacturer")
        );
        assert_eq!(oem_values.get("Model").copied(), Some("system.oemModel"));

        let current_version_values: BTreeMap<&str, &str> =
            WINDOWS_SOFTWARE_CURRENT_VERSION_REGISTRY_VALUES
                .iter()
                .map(|mapping| (mapping.value_name, mapping.metadata_key))
                .collect();
        assert_eq!(
            current_version_values.get("ProductName").copied(),
            Some("system.osName")
        );
        assert_eq!(
            current_version_values.get("CurrentBuildNumber").copied(),
            Some("system.osBuildNumber")
        );
    }

    #[test]
    fn normalized_to_db_artifact_rejects_oversized_artifact_size() {
        let artifact = NormalizedArtifact {
            id: "huge-artifact".to_string(),
            source_ref: EvidenceSourceRef::LocalFile {
                path: "/case/huge.bin".to_string(),
            },
            source_id: "/case/huge.bin".to_string(),
            name: "huge.bin".to_string(),
            extension: Some("bin".to_string()),
            size: u64::MAX,
            mime_type: Some("application/octet-stream".to_string()),
            type_description: "Binary Data".to_string(),
            category: "binary".to_string(),
            confidence: "medium".to_string(),
            is_text: false,
            content_preview: None,
            metadata: BTreeMap::new(),
        };

        let err = normalized_to_db_artifact(&artifact, None, "test-extractor".to_string())
            .expect_err("artifact conversion should reject unrepresentable size");

        assert!(err.contains("Artifact size exceeds project DB range"));
    }

    #[test]
    fn normalized_to_db_artifact_normalizes_extractor_label() {
        let artifact = NormalizedArtifact {
            id: "artifact-1".to_string(),
            source_ref: EvidenceSourceRef::LocalFile {
                path: "/case/a.txt".to_string(),
            },
            source_id: "/case/a.txt".to_string(),
            name: "a.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 42,
            mime_type: Some("text/plain".to_string()),
            type_description: "Plain Text".to_string(),
            category: "text".to_string(),
            confidence: "high".to_string(),
            is_text: true,
            content_preview: Some("hello".to_string()),
            metadata: BTreeMap::new(),
        };
        let oversized = format!("  {}é  ", "x".repeat(MAX_ARTIFACT_EXTRACTOR_CHARS + 64));

        let record = normalized_to_db_artifact(&artifact, None, oversized)
            .expect("artifact conversion should succeed");
        assert_eq!(
            record.extractor.chars().count(),
            MAX_ARTIFACT_EXTRACTOR_CHARS
        );
        assert!(record.extractor.starts_with('x'));

        let defaulted = normalized_to_db_artifact(&artifact, None, "   ".to_string())
            .expect("artifact conversion should succeed");
        assert_eq!(defaulted.extractor, DEFAULT_ARTIFACT_EXTRACTOR);
    }

    #[test]
    fn normalized_to_db_artifact_bounds_preview_and_metadata() {
        let mut metadata = BTreeMap::new();
        for index in 0..(MAX_ARTIFACT_METADATA_ENTRIES + 25) {
            metadata.insert(
                format!("key-{index}"),
                "é".repeat(MAX_ARTIFACT_FIELD_CHARS + 32),
            );
        }
        let artifact = NormalizedArtifact {
            id: "artifact-1".to_string(),
            source_ref: EvidenceSourceRef::LocalFile {
                path: "/case/a.txt".to_string(),
            },
            source_id: "/case/a.txt".to_string(),
            name: "a.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 42,
            mime_type: Some("text/plain".to_string()),
            type_description: "Plain Text".to_string(),
            category: "text".to_string(),
            confidence: "high".to_string(),
            is_text: true,
            content_preview: Some("p".repeat(MAX_ARTIFACT_PREVIEW_CHARS + 32)),
            metadata,
        };

        let record = normalized_to_db_artifact(&artifact, None, "test-extractor".to_string())
            .expect("artifact conversion should succeed");

        let preview = record.content_preview.as_deref().unwrap();
        assert_eq!(preview.chars().count(), MAX_ARTIFACT_PREVIEW_CHARS);
        assert!(preview.ends_with(ARTIFACT_TRUNCATED_SUFFIX));

        let metadata: BTreeMap<String, String> =
            serde_json::from_str(record.metadata_json.as_deref().unwrap()).unwrap();
        assert_eq!(metadata.len(), MAX_ARTIFACT_METADATA_ENTRIES);
        assert!(metadata
            .values()
            .all(|value| value.chars().count() == MAX_ARTIFACT_METADATA_VALUE_CHARS));
        assert!(metadata
            .values()
            .all(|value| value.ends_with(ARTIFACT_TRUNCATED_SUFFIX)));
    }

    #[test]
    fn bounded_artifact_record_for_response_caps_payloads_and_preserves_json() {
        let artifact = DbNormalizedArtifact {
            id: "artifact-1".to_string(),
            evidence_file_id: Some("ev-1".to_string()),
            source_id: "s".repeat(MAX_ARTIFACT_FIELD_CHARS + 32),
            source_ref_json: serde_json::json!({
                "kind": "localFile",
                "path": "x".repeat(MAX_ARTIFACT_JSON_CHARS + 32)
            })
            .to_string(),
            name: "n".repeat(MAX_ARTIFACT_FIELD_CHARS + 32),
            extension: Some("txt".to_string()),
            size: 42,
            mime_type: Some("text/plain".to_string()),
            type_description: "Plain Text".to_string(),
            category: "text".to_string(),
            confidence: "high".to_string(),
            is_text: true,
            content_preview: Some("p".repeat(MAX_ARTIFACT_PREVIEW_CHARS + 32)),
            metadata_json: Some(
                serde_json::json!({
                    "large": "m".repeat(MAX_ARTIFACT_JSON_CHARS + 32)
                })
                .to_string(),
            ),
            extracted_at: "2026-02-16T10:00:00Z".to_string(),
            extractor: "test-extractor".to_string(),
        };

        let bounded = bounded_artifact_record_for_response(artifact);

        assert_eq!(bounded.source_id.chars().count(), MAX_ARTIFACT_FIELD_CHARS);
        assert!(bounded.source_id.ends_with(ARTIFACT_TRUNCATED_SUFFIX));
        assert_eq!(bounded.name.chars().count(), MAX_ARTIFACT_FIELD_CHARS);
        assert_eq!(
            bounded.content_preview.as_deref().unwrap().chars().count(),
            MAX_ARTIFACT_PREVIEW_CHARS
        );
        assert!(serde_json::from_str::<serde_json::Value>(&bounded.source_ref_json).is_ok());
        assert!(serde_json::from_str::<serde_json::Value>(
            bounded.metadata_json.as_deref().unwrap()
        )
        .is_ok());
    }

    #[test]
    fn bounded_artifact_json_value_caps_arrays_and_depth() {
        let json = serde_json::json!({
            "items": (0..(MAX_ARTIFACT_JSON_ITEMS + 10)).collect::<Vec<_>>(),
            "nested": {"a": {"b": {"c": {"d": "too deep"}}}}
        })
        .to_string();

        let bounded = bounded_artifact_json_text(&json, MAX_ARTIFACT_JSON_CHARS);
        let value: serde_json::Value = serde_json::from_str(&bounded).unwrap();

        assert_eq!(
            value["items"].as_array().unwrap().len(),
            MAX_ARTIFACT_JSON_ITEMS
        );
        assert!(bounded.contains(ARTIFACT_TRUNCATED_SUFFIX));
    }

    #[test]
    fn copy_sqlite_artifact_source_accepts_chunked_reads() {
        let source = TestByteSource::new(10, b"0123456789", 3);
        let mut output = Vec::new();

        copy_sqlite_artifact_source(&source, source.len().unwrap(), &mut output).unwrap();

        assert_eq!(output, b"0123456789");
    }

    #[test]
    fn copy_sqlite_artifact_source_rejects_short_reads() {
        let source = TestByteSource::new(8, b"abc", 8);
        let mut output = Vec::new();

        let err =
            copy_sqlite_artifact_source(&source, source.len().unwrap(), &mut output).unwrap_err();

        assert!(err.contains("Short read materializing SQLite artifact source"));
        assert!(err.contains("expected 8 bytes but read 3 bytes"));
        assert_eq!(output, b"abc");
    }

    #[test]
    fn checked_sqlite_copy_offset_add_rejects_overflow() {
        let err = checked_sqlite_copy_offset_add(u64::MAX, 1, "test-source.sqlite").unwrap_err();

        assert!(err.contains("SQLite artifact copy offset overflow"));
        assert!(err.contains("test-source.sqlite"));
    }

    #[test]
    fn sqlite_artifact_metadata_from_source_extracts_schema_summary() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let conn = Connection::open(tmp.path()).unwrap();
        conn.execute_batch(
            "CREATE TABLE contacts (id INTEGER PRIMARY KEY, name TEXT, email TEXT);
             CREATE TABLE logs (id INTEGER PRIMARY KEY, level TEXT, message TEXT);
             INSERT INTO contacts VALUES (1, 'Alice', 'alice@example.com');
             INSERT INTO contacts VALUES (2, 'Bob', 'bob@example.com');
             INSERT INTO logs VALUES (1, 'info', 'started');
             CREATE VIEW contact_names AS SELECT name FROM contacts;",
        )
        .unwrap();
        drop(conn);

        let source = HashSourceInput {
            path: Some(tmp.path().to_string_lossy().to_string()),
            container_path: None,
            entry_path: None,
            nested_archive_path: None,
            container_type: Some("disk".to_string()),
            size: Some(std::fs::metadata(tmp.path()).unwrap().len()),
            data_addr: None,
            item_addr: None,
        };

        let metadata = sqlite_artifact_metadata_from_source(&source).unwrap();

        assert_eq!(
            metadata.get("sqlite.tableCount").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata.get("sqlite.viewCount").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            metadata.get("sqlite.totalRows").map(String::as_str),
            Some("3")
        );
        assert_eq!(
            metadata.get("sqlite.tableNames").map(String::as_str),
            Some("contacts, logs")
        );
        assert!(metadata
            .get("sqlite.tables")
            .is_some_and(|value| value.contains("contacts (2 rows, 3 cols)")));
        assert_eq!(
            metadata.get("sqlite.largestTable").map(String::as_str),
            Some("contacts (2 rows)")
        );
    }

    #[test]
    fn sqlite_total_rows_clamps_negative_counts_and_saturates() {
        let tables = vec![
            crate::viewer::document::database_viewer::TableSummary {
                name: "negative".to_string(),
                row_count: -25,
                column_count: 1,
                is_system: false,
            },
            crate::viewer::document::database_viewer::TableSummary {
                name: "huge".to_string(),
                row_count: i64::MAX,
                column_count: 1,
                is_system: false,
            },
            crate::viewer::document::database_viewer::TableSummary {
                name: "extra".to_string(),
                row_count: 42,
                column_count: 1,
                is_system: false,
            },
        ];

        assert_eq!(sqlite_total_rows(&tables), i64::MAX);
    }
}
