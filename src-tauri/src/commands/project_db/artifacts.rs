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
use crate::viewer::document::binary::{analyze_binary_bytes, BinaryFormat, BinaryInfo};
use crate::viewer::document::database_viewer::get_database_info;
use crate::viewer::document::exif::{
    ensure_exif_size_allowed, extract_exif_from_reader, ExifMetadata,
};
use notatin::cell_value::CellValue;
use notatin::parser_builder::ParserBuilder;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::BTreeMap;
use std::io::Cursor;
use std::io::Write;
use std::path::Path;

#[derive(Clone, Copy)]
struct RegistryStringMapping {
    value_name: &'static str,
    metadata_key: &'static str,
}

const SQLITE_ARTIFACT_SOURCE_MAX_BYTES: u64 = 512 * 1024 * 1024;
const BINARY_ARTIFACT_SOURCE_MAX_BYTES: u64 = 512 * 1024 * 1024;
const IMAGE_ARTIFACT_SOURCE_MAX_BYTES: u64 = 100 * 1024 * 1024;
const SQLITE_ARTIFACT_COPY_CHUNK_BYTES: usize = 1024 * 1024;
const SYSTEM_IDENTITY_SOURCE_MAX_BYTES: u64 = 2 * 1024 * 1024;
const REGISTRY_IDENTITY_SOURCE_MAX_BYTES: u64 = 256 * 1024 * 1024;
const SQLITE_METADATA_NAME_LIMIT: usize = 12;
const MAX_SYSTEM_IDENTITY_LIST_ITEMS: usize = 32;
const MAX_SYSTEM_MOUNT_DESCRIPTION_CHARS: usize = 160;
const UNIX_REGULAR_USER_MIN_UID: u32 = 1000;
const UNIX_REGULAR_USER_MAX_UID: u32 = 60000;
const MACOS_REGULAR_USER_MIN_UID: u32 = 500;
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
        value_name: "UBR",
        metadata_key: "system.osUpdateBuildRevision",
    },
    RegistryStringMapping {
        value_name: "BuildLab",
        metadata_key: "system.osBuildLab",
    },
    RegistryStringMapping {
        value_name: "BuildLabEx",
        metadata_key: "system.osBuildLabExtended",
    },
    RegistryStringMapping {
        value_name: "EditionID",
        metadata_key: "system.osEdition",
    },
    RegistryStringMapping {
        value_name: "CompositionEditionID",
        metadata_key: "system.osCompositionEdition",
    },
    RegistryStringMapping {
        value_name: "InstallationType",
        metadata_key: "system.osInstallationType",
    },
    RegistryStringMapping {
        value_name: "InstallDate",
        metadata_key: "system.osInstallDateEpoch",
    },
    RegistryStringMapping {
        value_name: "PathName",
        metadata_key: "system.osPath",
    },
    RegistryStringMapping {
        value_name: "SystemRoot",
        metadata_key: "system.systemRoot",
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
    enrich_image_artifact_metadata(&source_for_enrichment, &mut artifact).await?;
    enrich_binary_artifact_metadata(&source_for_enrichment, &mut artifact).await?;
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
        || source_id.ends_with(
            "/library/preferences/systemconfiguration/com.apple.airport.preferences.plist",
        )
        || source_id.ends_with("/library/preferences/com.apple.wifi.known-networks.plist")
        || source_id.ends_with("/library/preferences/com.apple.alf.plist")
        || source_id.ends_with("/library/preferences/.globalpreferences.plist")
        || source_id.ends_with("/library/receipts/installhistory.plist")
        || source_id.ends_with("/var/db/diskmanagement.plist")
        || is_macos_hardware_identity_source(&source_id)
        || source_id.ends_with("/windows/system32/config/system")
        || source_id.ends_with("/windows/system32/config/software")
        || source_id.ends_with("/windows/system32/config/sam")
        || source_id.ends_with("/config/system")
        || source_id.ends_with("/config/software")
        || source_id.ends_with("/config/sam")
    {
        return true;
    }

    if is_macos_local_user_source(&source_id) || is_macos_local_group_source(&source_id) {
        return true;
    }

    if is_macos_kernel_extension_source(&source_id) {
        return true;
    }

    if is_linux_network_identity_source(&source_id) {
        return true;
    }

    if is_linux_hardware_inventory_source(&source_id) {
        return true;
    }

    if is_unix_account_identity_source(&source_id) {
        return true;
    }

    if is_windows_wifi_profile_source(&source_id) {
        return true;
    }

    if is_command_history_source(&source_id) {
        return true;
    }

    if is_firewall_identity_source(&source_id) {
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
                | "machine-info"
                | "hostname"
                | "timezone"
                | "localtime"
                | "locale"
                | "fstab"
                | "mtab"
                | "product_uuid"
                | "product_serial"
                | "product_name"
                | "product_version"
                | "product_family"
                | "product_sku"
                | "sys_vendor"
                | "board_asset_tag"
                | "board_serial"
                | "board_name"
                | "board_vendor"
                | "board_version"
                | "bios_version"
                | "bios_vendor"
                | "bios_date"
                | "chassis_asset_tag"
                | "chassis_vendor"
                | "chassis_type"
                | "chassis_serial"
                | "chassis_version"
        )
    )
}

fn is_linux_network_identity_source(source_id: &str) -> bool {
    let source_id = source_id.replace('\\', "/").to_ascii_lowercase();
    source_id.ends_with("/etc/network/interfaces")
        || source_id.ends_with("/etc/resolv.conf")
        || source_id.ends_with("/private/etc/resolv.conf")
        || source_id.ends_with("/etc/hosts")
        || source_id.ends_with("/private/etc/hosts")
        || source_id.ends_with("/windows/system32/drivers/etc/hosts")
        || source_id.contains("/etc/networkmanager/system-connections/")
        || source_id.contains("/etc/sysconfig/network-scripts/ifcfg-")
        || (source_id.contains("/etc/netplan/")
            && (source_id.ends_with(".yaml") || source_id.ends_with(".yml")))
}

fn is_linux_machine_identity_source(source_id: &str) -> bool {
    let source_id = source_id.replace('\\', "/").to_ascii_lowercase();
    source_id.ends_with("/var/lib/dbus/machine-id")
        || source_id.ends_with("/etc/machine-info")
        || source_id.ends_with("/etc/default/locale")
}

fn is_linux_hardware_inventory_source(source_id: &str) -> bool {
    let source_id = source_id.replace('\\', "/").to_ascii_lowercase();
    let file_name = source_id.rsplit('/').next().unwrap_or("");
    matches!(
        file_name,
        "dmidecode" | "dmidecode.txt" | "lshw" | "lshw.txt" | "lshw-short.txt"
    ) || source_id.ends_with("/var/log/installer/dmidecode")
}

fn is_unix_account_identity_source(source_id: &str) -> bool {
    let source_id = source_id.replace('\\', "/").to_ascii_lowercase();
    source_id.ends_with("/etc/passwd")
        || source_id.ends_with("/private/etc/passwd")
        || source_id.ends_with("/etc/group")
        || source_id.ends_with("/private/etc/group")
        || source_id.ends_with("/etc/shadow")
        || source_id.ends_with("/private/etc/shadow")
        || source_id.ends_with("/etc/gshadow")
        || source_id.ends_with("/private/etc/gshadow")
}

fn is_macos_local_user_source(source_id: &str) -> bool {
    let source_id = source_id.replace('\\', "/").to_ascii_lowercase();
    (source_id.contains("/private/var/db/dslocal/nodes/default/users/")
        || source_id.contains("/var/db/dslocal/nodes/default/users/"))
        && source_id.ends_with(".plist")
}

fn is_macos_local_group_source(source_id: &str) -> bool {
    let source_id = source_id.replace('\\', "/").to_ascii_lowercase();
    (source_id.contains("/private/var/db/dslocal/nodes/default/groups/")
        || source_id.contains("/var/db/dslocal/nodes/default/groups/"))
        && source_id.ends_with(".plist")
}

fn is_macos_hardware_identity_source(source_id: &str) -> bool {
    let source_id = source_id.replace('\\', "/").to_ascii_lowercase();
    source_id.ends_with("/ioplatformexpertdevice.plist")
        || source_id.ends_with("/ioregistry.plist")
        || source_id.ends_with("/sphardwaredatatype.plist")
        || source_id.ends_with("/system_profiler.spx")
}

fn is_macos_kernel_extension_source(source_id: &str) -> bool {
    let source_id = source_id.replace('\\', "/").to_ascii_lowercase();
    source_id.contains(".kext/contents/info.plist")
}

fn is_windows_wifi_profile_source(source_id: &str) -> bool {
    let source_id = source_id.replace('\\', "/").to_ascii_lowercase();
    source_id.contains("/programdata/microsoft/wlansvc/profiles/interfaces/")
        && source_id.ends_with(".xml")
}

fn is_command_history_source(source_id: &str) -> bool {
    let source_id = source_id.replace('\\', "/").to_ascii_lowercase();
    source_id.ends_with("/.bash_history")
        || source_id.ends_with("/.zsh_history")
        || source_id.ends_with("/consolehost_history.txt")
}

fn is_firewall_identity_source(source_id: &str) -> bool {
    let source_id = source_id.replace('\\', "/").to_ascii_lowercase();
    source_id.ends_with("/library/preferences/com.apple.alf.plist")
        || source_id.ends_with("/etc/sysconfig/iptables")
        || source_id.contains("/etc/iptables/")
        || source_id.ends_with("/windows/system32/logfiles/firewall/pfirewall.log")
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
        || source_id.ends_with("/windows/system32/config/sam")
        || source_id.ends_with("/config/system")
        || source_id.ends_with("/config/software")
        || source_id.ends_with("/config/sam")
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

    let lower = source_id.to_ascii_lowercase();
    let suffix = if lower.ends_with("software") {
        ".software.hive"
    } else if lower.ends_with("sam") {
        ".sam.hive"
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
    } else if lower.ends_with("sam") {
        registry_sam_identity_metadata(hive_path)
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
        &format!("{control_set}\\Control\\ComputerName\\ActiveComputerName"),
        "ComputerName",
        "system.activeComputerName",
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
    metadata.extend(registry_system_tcpip_interface_metadata(
        &mut parser,
        &control_set,
    ));
    metadata.extend(registry_system_driver_service_metadata(
        &mut parser,
        &control_set,
    ));
    metadata.extend(registry_system_mounted_devices_metadata(&mut parser));

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
    metadata.extend(registry_software_profile_list_metadata(&mut parser));
    metadata.extend(registry_software_network_list_metadata(&mut parser));

    Ok(finalize_registry_identity_metadata(
        metadata,
        "windows.software",
    ))
}

fn registry_sam_identity_metadata(hive_path: &Path) -> Result<BTreeMap<String, String>, String> {
    let hive_path = hive_path.to_path_buf();
    let mut parser = ParserBuilder::from_path(hive_path)
        .build()
        .map_err(|e| format!("Failed to open SAM registry hive: {e}"))?;
    let account_names = registry_subkey_names(&mut parser, "SAM\\Domains\\Account\\Users\\Names");
    let group_names = registry_subkey_names(&mut parser, "SAM\\Domains\\Builtin\\Aliases\\Names");

    Ok(finalize_registry_identity_metadata(
        registry_sam_account_metadata_from_names(account_names, group_names),
        "windows.sam",
    ))
}

fn registry_sam_account_metadata_from_names(
    account_names: Vec<String>,
    group_names: Vec<String>,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();

    if !account_names.is_empty() {
        metadata.insert(
            "system.accountConfigType".to_string(),
            "windows-sam".to_string(),
        );
        metadata.insert(
            "system.localUserCount".to_string(),
            account_names.len().to_string(),
        );
        insert_joined_metadata(&mut metadata, "system.localUsers", &account_names);
        if account_names
            .iter()
            .any(|name| name.eq_ignore_ascii_case("administrator"))
        {
            metadata.insert(
                "system.administratorAccountPresent".to_string(),
                "true".to_string(),
            );
        }
        if account_names
            .iter()
            .any(|name| name.eq_ignore_ascii_case("guest"))
        {
            metadata.insert("system.guestAccountPresent".to_string(), "true".to_string());
        }
    }

    if !group_names.is_empty() {
        metadata
            .entry("system.accountConfigType".to_string())
            .or_insert_with(|| "windows-sam".to_string());
        metadata.insert(
            "system.localGroupCount".to_string(),
            group_names.len().to_string(),
        );
        insert_joined_metadata(&mut metadata, "system.localGroups", &group_names);
        let admin_groups: Vec<String> = group_names
            .iter()
            .filter(|name| is_windows_admin_group(name))
            .cloned()
            .collect();
        insert_joined_metadata(&mut metadata, "system.adminGroups", &admin_groups);
    }

    metadata
}

fn is_windows_admin_group(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "administrators"
            | "domain admins"
            | "enterprise admins"
            | "account operators"
            | "backup operators"
    )
}

#[derive(Default)]
struct WindowsProfileListMetadata {
    sids: Vec<String>,
    names: Vec<String>,
    paths: Vec<String>,
    descriptions: Vec<String>,
}

fn registry_software_profile_list_metadata(
    parser: &mut notatin::parser::Parser,
) -> BTreeMap<String, String> {
    let profile_list_path = "Microsoft\\Windows NT\\CurrentVersion\\ProfileList";
    let Some(mut profile_list_key) = parser.get_key(profile_list_path, false).ok().flatten() else {
        return BTreeMap::new();
    };

    let mut values = WindowsProfileListMetadata::default();
    for profile_key in profile_list_key
        .read_sub_keys(parser)
        .iter()
        .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
    {
        let profile_key_path = format!("{profile_list_path}\\{}", profile_key.key_name);
        let profile_path = registry_value(parser, &profile_key_path, "ProfileImagePath")
            .and_then(registry_value_text);
        collect_windows_profile_list_metadata(&mut values, &profile_key.key_name, profile_path);
    }

    windows_profile_list_metadata_to_map(values)
}

fn collect_windows_profile_list_metadata(
    values: &mut WindowsProfileListMetadata,
    sid: &str,
    profile_path: Option<String>,
) {
    let sid = truncate_metadata_value(sid.trim(), 120);
    if sid.is_empty() {
        return;
    }
    push_unique_limited(&mut values.sids, sid.clone());

    let profile_path = profile_path
        .map(|value| truncate_metadata_value(value.trim(), 240))
        .filter(|value| !value.is_empty());
    if let Some(profile_path) = profile_path {
        push_unique_limited(&mut values.paths, profile_path.clone());
        if let Some(name) = windows_profile_name_from_path(&profile_path) {
            push_unique_limited(&mut values.names, name.clone());
            push_unique_limited(
                &mut values.descriptions,
                truncate_metadata_value(
                    &format!("{name} ({sid}, {profile_path})"),
                    MAX_SYSTEM_MOUNT_DESCRIPTION_CHARS,
                ),
            );
        } else {
            push_unique_limited(
                &mut values.descriptions,
                truncate_metadata_value(
                    &format!("{sid} ({profile_path})"),
                    MAX_SYSTEM_MOUNT_DESCRIPTION_CHARS,
                ),
            );
        }
    } else {
        push_unique_limited(&mut values.descriptions, sid);
    }
}

fn windows_profile_name_from_path(path: &str) -> Option<String> {
    let name = path
        .trim()
        .trim_end_matches(['\\', '/'])
        .rsplit(['\\', '/'])
        .next()?
        .trim();
    (!name.is_empty()).then(|| truncate_metadata_value(name, 120))
}

fn windows_profile_list_metadata_to_map(
    values: WindowsProfileListMetadata,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if values.sids.is_empty() {
        return metadata;
    }

    metadata.insert(
        "system.profileCount".to_string(),
        values.sids.len().to_string(),
    );
    insert_joined_metadata(&mut metadata, "system.profileSids", &values.sids);
    insert_joined_metadata(&mut metadata, "system.profileNames", &values.names);
    insert_joined_metadata(&mut metadata, "system.profilePaths", &values.paths);
    insert_joined_metadata(&mut metadata, "system.profiles", &values.descriptions);
    metadata
}

#[derive(Default)]
struct WindowsNetworkListMetadata {
    names: Vec<String>,
    categories: Vec<String>,
    descriptions: Vec<String>,
}

fn registry_software_network_list_metadata(
    parser: &mut notatin::parser::Parser,
) -> BTreeMap<String, String> {
    let profiles_path = "Microsoft\\Windows NT\\CurrentVersion\\NetworkList\\Profiles";
    let Some(mut profiles_key) = parser.get_key(profiles_path, false).ok().flatten() else {
        return BTreeMap::new();
    };

    let mut values = WindowsNetworkListMetadata::default();
    for profile_key in profiles_key
        .read_sub_keys(parser)
        .iter()
        .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
    {
        let profile_path = format!("{profiles_path}\\{}", profile_key.key_name);
        let name =
            registry_value(parser, &profile_path, "ProfileName").and_then(registry_value_text);
        let description =
            registry_value(parser, &profile_path, "Description").and_then(registry_value_text);
        let category = registry_value(parser, &profile_path, "Category")
            .and_then(windows_network_category_name);
        collect_windows_network_list_metadata(&mut values, name, description, category);
    }

    windows_network_list_metadata_to_map(values)
}

fn collect_windows_network_list_metadata(
    values: &mut WindowsNetworkListMetadata,
    name: Option<String>,
    description: Option<String>,
    category: Option<String>,
) {
    let name = name
        .map(|value| truncate_metadata_value(value.trim(), 120))
        .filter(|value| !value.is_empty());
    let description = description
        .map(|value| truncate_metadata_value(value.trim(), 160))
        .filter(|value| !value.is_empty());
    let category = category
        .map(|value| truncate_metadata_value(value.trim(), 80))
        .filter(|value| !value.is_empty());

    if let Some(name) = &name {
        push_unique_limited(&mut values.names, name.clone());
    }
    if let Some(category) = &category {
        push_unique_limited(&mut values.categories, category.clone());
    }

    let label = name.or(description);
    if let Some(label) = label {
        let detail = category
            .map(|category| format!("{label} ({category})"))
            .unwrap_or(label);
        push_unique_limited(
            &mut values.descriptions,
            truncate_metadata_value(&detail, MAX_SYSTEM_MOUNT_DESCRIPTION_CHARS),
        );
    }
}

fn windows_network_category_name(value: CellValue) -> Option<String> {
    match value {
        CellValue::U32(0) | CellValue::I32(0) => Some("public".to_string()),
        CellValue::U32(1) | CellValue::I32(1) => Some("private".to_string()),
        CellValue::U32(2) | CellValue::I32(2) => Some("domain-authenticated".to_string()),
        CellValue::U32(value) => Some(format!("category-{value}")),
        CellValue::I32(value) => Some(format!("category-{value}")),
        CellValue::String(value) => {
            let value = value.trim();
            (!value.is_empty()).then(|| value.to_string())
        }
        _ => None,
    }
}

fn windows_network_list_metadata_to_map(
    values: WindowsNetworkListMetadata,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if values.descriptions.is_empty() {
        return metadata;
    }

    metadata.insert(
        "system.networkProfileCount".to_string(),
        values.descriptions.len().to_string(),
    );
    insert_joined_metadata(&mut metadata, "system.networkProfileNames", &values.names);
    insert_joined_metadata(
        &mut metadata,
        "system.networkProfileCategories",
        &values.categories,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.networkProfiles",
        &values.descriptions,
    );
    metadata
}

#[derive(Default)]
struct WindowsMountedDevicesMetadata {
    drive_letters: Vec<String>,
    volume_guids: Vec<String>,
    device_names: Vec<String>,
}

fn registry_system_mounted_devices_metadata(
    parser: &mut notatin::parser::Parser,
) -> BTreeMap<String, String> {
    let Some(key) = parser.get_key("MountedDevices", false).ok().flatten() else {
        return BTreeMap::new();
    };

    let mut values = WindowsMountedDevicesMetadata::default();
    for value in key.value_iter().take(MAX_SYSTEM_IDENTITY_LIST_ITEMS) {
        collect_windows_mounted_device_name(&mut values, &value.detail.value_name());
    }

    windows_mounted_devices_metadata_to_map(values)
}

fn collect_windows_mounted_device_name(values: &mut WindowsMountedDevicesMetadata, name: &str) {
    let normalized = name.replace('/', "\\");
    let lower = normalized.to_ascii_lowercase();
    if let Some(drive) = lower
        .strip_prefix("\\dosdevices\\")
        .and_then(|_| normalized.rsplit('\\').next())
        .filter(|value| is_windows_drive_letter(value))
    {
        let drive = drive.to_ascii_uppercase();
        push_unique_limited(&mut values.drive_letters, drive.clone());
        push_unique_limited(&mut values.device_names, format!("drive {drive}"));
        return;
    }

    if let Some(volume) = extract_windows_volume_guid_name(&normalized) {
        push_unique_limited(&mut values.volume_guids, volume.clone());
        push_unique_limited(&mut values.device_names, volume);
    }
}

fn is_windows_drive_letter(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic()
}

fn extract_windows_volume_guid_name(value: &str) -> Option<String> {
    let lower = value.to_ascii_lowercase();
    let start = lower.find("volume{")?;
    let remainder = &value[start..];
    let end = remainder.find('}')?;
    let volume = &remainder[..=end];
    (volume.len() > "Volume{}".len()).then(|| truncate_metadata_value(volume, 120))
}

fn windows_mounted_devices_metadata_to_map(
    values: WindowsMountedDevicesMetadata,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if values.drive_letters.is_empty() && values.volume_guids.is_empty() {
        return metadata;
    }

    metadata.insert(
        "system.mountedDeviceCount".to_string(),
        values.device_names.len().to_string(),
    );
    insert_joined_metadata(&mut metadata, "system.driveLetters", &values.drive_letters);
    insert_joined_metadata(&mut metadata, "system.volumeGuids", &values.volume_guids);
    insert_joined_metadata(&mut metadata, "system.mountedDevices", &values.device_names);
    metadata
}

#[derive(Default)]
struct WindowsDriverServiceMetadata {
    names: Vec<String>,
    image_paths: Vec<String>,
    groups: Vec<String>,
    start_types: Vec<String>,
    descriptions: Vec<String>,
}

fn registry_system_driver_service_metadata(
    parser: &mut notatin::parser::Parser,
    control_set: &str,
) -> BTreeMap<String, String> {
    let services_path = format!("{control_set}\\Services");
    let Some(mut services_key) = parser.get_key(&services_path, false).ok().flatten() else {
        return BTreeMap::new();
    };

    let mut values = WindowsDriverServiceMetadata::default();
    for service_key in services_key
        .read_sub_keys(parser)
        .iter()
        .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
    {
        let service_path = format!("{services_path}\\{}", service_key.key_name);
        let driver_type = registry_value(parser, &service_path, "Type")
            .and_then(windows_driver_service_type_name);
        let image_path =
            registry_value(parser, &service_path, "ImagePath").and_then(registry_value_text);
        if driver_type.is_none()
            && !image_path
                .as_deref()
                .is_some_and(is_windows_driver_image_path)
        {
            continue;
        }

        let display_name =
            registry_value(parser, &service_path, "DisplayName").and_then(registry_value_text);
        let group = registry_value(parser, &service_path, "Group").and_then(registry_value_text);
        let start_type =
            registry_value(parser, &service_path, "Start").and_then(windows_service_start_name);
        collect_windows_driver_service_metadata(
            &mut values,
            &service_key.key_name,
            display_name,
            image_path,
            group,
            start_type,
            driver_type,
        );
    }

    windows_driver_service_metadata_to_map(values)
}

fn collect_windows_driver_service_metadata(
    values: &mut WindowsDriverServiceMetadata,
    service_name: &str,
    display_name: Option<String>,
    image_path: Option<String>,
    group: Option<String>,
    start_type: Option<String>,
    driver_type: Option<String>,
) {
    let service_name = truncate_metadata_value(service_name.trim(), 120);
    if service_name.is_empty() {
        return;
    }
    push_unique_limited(&mut values.names, service_name.clone());

    let image_path = image_path
        .map(|value| normalize_windows_driver_image_path(&value))
        .filter(|value| !value.is_empty());
    if let Some(image_path) = &image_path {
        push_unique_limited(&mut values.image_paths, image_path.clone());
    }

    let group = group
        .map(|value| truncate_metadata_value(value.trim(), 120))
        .filter(|value| !value.is_empty());
    if let Some(group) = &group {
        push_unique_limited(&mut values.groups, group.clone());
    }

    let start_type = start_type
        .map(|value| truncate_metadata_value(value.trim(), 80))
        .filter(|value| !value.is_empty());
    if let Some(start_type) = &start_type {
        push_unique_limited(&mut values.start_types, start_type.clone());
    }

    let label = display_name
        .map(|value| truncate_metadata_value(value.trim(), 120))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| service_name.clone());
    let mut details = Vec::new();
    if let Some(driver_type) = driver_type {
        details.push(driver_type);
    }
    if let Some(start_type) = start_type {
        details.push(format!("start={start_type}"));
    }
    if let Some(group) = group {
        details.push(format!("group={group}"));
    }
    if let Some(image_path) = image_path {
        details.push(format!("image={image_path}"));
    }

    let description = if details.is_empty() {
        label
    } else {
        format!("{label} ({})", details.join("; "))
    };
    push_unique_limited(
        &mut values.descriptions,
        truncate_metadata_value(&description, MAX_SYSTEM_MOUNT_DESCRIPTION_CHARS),
    );
}

fn windows_driver_service_type_name(value: CellValue) -> Option<String> {
    let value = registry_value_u32(value)?;
    match value {
        0x1 => Some("kernel-driver".to_string()),
        0x2 => Some("file-system-driver".to_string()),
        _ => None,
    }
}

fn windows_service_start_name(value: CellValue) -> Option<String> {
    let value = registry_value_u32(value)?;
    match value {
        0 => Some("boot".to_string()),
        1 => Some("system".to_string()),
        2 => Some("auto".to_string()),
        3 => Some("demand".to_string()),
        4 => Some("disabled".to_string()),
        other => Some(format!("start-{other}")),
    }
}

fn registry_value_u32(value: CellValue) -> Option<u32> {
    match value {
        CellValue::U32(value) => Some(value),
        CellValue::I32(value) => u32::try_from(value).ok(),
        CellValue::U64(value) => u32::try_from(value).ok(),
        CellValue::I64(value) => u32::try_from(value).ok(),
        CellValue::String(value) => {
            let value = value.trim();
            value
                .strip_prefix("0x")
                .or_else(|| value.strip_prefix("0X"))
                .map_or_else(
                    || value.parse::<u32>().ok(),
                    |hex| u32::from_str_radix(hex, 16).ok(),
                )
        }
        _ => None,
    }
}

fn normalize_windows_driver_image_path(value: &str) -> String {
    let normalized = value.trim().replace('/', "\\");
    truncate_metadata_value(&normalized, 180)
}

fn is_windows_driver_image_path(value: &str) -> bool {
    value
        .rsplit(['\\', '/'])
        .next()
        .is_some_and(|name| name.to_ascii_lowercase().ends_with(".sys"))
}

fn windows_driver_service_metadata_to_map(
    values: WindowsDriverServiceMetadata,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if values.names.is_empty() {
        return metadata;
    }

    metadata.insert(
        "system.driverServiceCount".to_string(),
        values.names.len().to_string(),
    );
    insert_joined_metadata(&mut metadata, "system.driverServices", &values.names);
    insert_joined_metadata(
        &mut metadata,
        "system.driverImagePaths",
        &values.image_paths,
    );
    insert_joined_metadata(&mut metadata, "system.driverGroups", &values.groups);
    insert_joined_metadata(
        &mut metadata,
        "system.driverStartTypes",
        &values.start_types,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.driverServiceDetails",
        &values.descriptions,
    );
    metadata
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

fn registry_subkey_names(parser: &mut notatin::parser::Parser, key_path: &str) -> Vec<String> {
    let Some(mut key) = parser.get_key(key_path, false).ok().flatten() else {
        return Vec::new();
    };
    key.read_sub_keys(parser)
        .iter()
        .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
        .map(|subkey| truncate_metadata_value(&subkey.key_name, 120))
        .collect()
}

fn registry_value_text(value: CellValue) -> Option<String> {
    let values = registry_value_texts(value);
    (!values.is_empty()).then(|| values.join("; "))
}

fn registry_value_texts(value: CellValue) -> Vec<String> {
    match value {
        CellValue::String(value) => vec![value],
        CellValue::MultiString(values) => values,
        CellValue::U32(value) => vec![value.to_string()],
        CellValue::I32(value) => vec![value.to_string()],
        CellValue::U64(value) => vec![value.to_string()],
        CellValue::I64(value) => vec![value.to_string()],
        _ => Vec::new(),
    }
}

fn registry_value_text_list(
    parser: &mut notatin::parser::Parser,
    key_path: &str,
    value_name: &str,
) -> Vec<String> {
    registry_value(parser, key_path, value_name)
        .map(registry_value_texts)
        .unwrap_or_default()
}

#[derive(Default)]
struct WindowsTcpipInterfaceMetadata {
    interfaces: Vec<String>,
    addresses: Vec<String>,
    gateways: Vec<String>,
    dns_servers: Vec<String>,
    domains: Vec<String>,
    dhcp_servers: Vec<String>,
    descriptions: Vec<String>,
}

fn registry_system_tcpip_interface_metadata(
    parser: &mut notatin::parser::Parser,
    control_set: &str,
) -> BTreeMap<String, String> {
    let interfaces_path = format!("{control_set}\\Services\\Tcpip\\Parameters\\Interfaces");
    let Some(mut interfaces_key) = parser.get_key(&interfaces_path, false).ok().flatten() else {
        return BTreeMap::new();
    };

    let mut values = WindowsTcpipInterfaceMetadata::default();
    for interface_key in interfaces_key
        .read_sub_keys(parser)
        .iter()
        .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
    {
        let interface_path = format!("{interfaces_path}\\{}", interface_key.key_name);
        collect_windows_tcpip_interface_metadata(
            parser,
            &interface_path,
            &interface_key.key_name,
            &mut values,
        );
    }

    windows_tcpip_interface_metadata_to_map(values)
}

fn collect_windows_tcpip_interface_metadata(
    parser: &mut notatin::parser::Parser,
    key_path: &str,
    interface_id: &str,
    values: &mut WindowsTcpipInterfaceMetadata,
) {
    push_unique_limited(&mut values.interfaces, interface_id.to_string());

    let mut interface_addresses = Vec::new();
    for value_name in ["IPAddress", "DhcpIPAddress"] {
        for value in registry_value_text_list(parser, key_path, value_name) {
            for address in split_windows_registry_network_list(&value) {
                if is_useful_windows_network_value(&address) {
                    push_unique_limited(&mut values.addresses, address.clone());
                    push_unique_limited(&mut interface_addresses, address);
                }
            }
        }
    }

    let mut interface_gateways = Vec::new();
    for value_name in ["DefaultGateway", "DhcpDefaultGateway"] {
        for value in registry_value_text_list(parser, key_path, value_name) {
            for gateway in split_windows_registry_network_list(&value) {
                if is_useful_windows_network_value(&gateway) {
                    push_unique_limited(&mut values.gateways, gateway.clone());
                    push_unique_limited(&mut interface_gateways, gateway);
                }
            }
        }
    }

    let mut interface_dns = Vec::new();
    for value_name in ["NameServer", "DhcpNameServer"] {
        for value in registry_value_text_list(parser, key_path, value_name) {
            for server in split_windows_registry_network_list(&value) {
                if is_useful_windows_network_value(&server) {
                    push_unique_limited(&mut values.dns_servers, server.clone());
                    push_unique_limited(&mut interface_dns, server);
                }
            }
        }
    }

    for value_name in ["Domain", "DhcpDomain", "SearchList"] {
        for value in registry_value_text_list(parser, key_path, value_name) {
            for domain in split_windows_registry_network_list(&value) {
                if is_useful_windows_network_value(&domain) {
                    push_unique_limited(&mut values.domains, domain);
                }
            }
        }
    }

    for value in registry_value_text_list(parser, key_path, "DhcpServer") {
        for server in split_windows_registry_network_list(&value) {
            if is_useful_windows_network_value(&server) {
                push_unique_limited(&mut values.dhcp_servers, server);
            }
        }
    }

    if !interface_addresses.is_empty()
        || !interface_gateways.is_empty()
        || !interface_dns.is_empty()
    {
        let mut parts = Vec::new();
        if !interface_addresses.is_empty() {
            parts.push(format!("ip={}", interface_addresses.join(",")));
        }
        if !interface_gateways.is_empty() {
            parts.push(format!("gateway={}", interface_gateways.join(",")));
        }
        if !interface_dns.is_empty() {
            parts.push(format!("dns={}", interface_dns.join(",")));
        }
        push_unique_limited(
            &mut values.descriptions,
            truncate_metadata_value(
                &format!("{interface_id} ({})", parts.join("; ")),
                MAX_SYSTEM_MOUNT_DESCRIPTION_CHARS,
            ),
        );
    }
}

fn split_windows_registry_network_list(value: &str) -> Vec<String> {
    value
        .split([';', ',', ' ', '\t', '\r', '\n'])
        .filter_map(|part| {
            let value = part.trim().trim_matches('"').trim_matches('\'');
            (!value.is_empty()).then(|| truncate_metadata_value(value, 120))
        })
        .collect()
}

fn is_useful_windows_network_value(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value != "0.0.0.0"
        && value != "::"
        && value != "255.255.255.255"
        && !value.eq_ignore_ascii_case("none")
}

fn windows_tcpip_interface_metadata_to_map(
    values: WindowsTcpipInterfaceMetadata,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    insert_joined_metadata(
        &mut metadata,
        "system.networkInterfaces",
        &values.interfaces,
    );
    insert_joined_metadata(&mut metadata, "system.ipv4Addresses", &values.addresses);
    insert_joined_metadata(&mut metadata, "system.gateways", &values.gateways);
    insert_joined_metadata(&mut metadata, "system.dnsServers", &values.dns_servers);
    insert_joined_metadata(&mut metadata, "system.networkDomains", &values.domains);
    insert_joined_metadata(&mut metadata, "system.dhcpServers", &values.dhcp_servers);
    insert_joined_metadata(
        &mut metadata,
        "system.networkInterfaceDescriptions",
        &values.descriptions,
    );
    metadata
}

fn system_identity_metadata_from_bytes(source_id: &str, data: &[u8]) -> BTreeMap<String, String> {
    let normalized = source_id.replace('\\', "/");
    let lower = normalized.to_ascii_lowercase();
    let file_name = lower.rsplit('/').next().unwrap_or("");
    let text = String::from_utf8_lossy(data);
    let mut metadata = BTreeMap::new();

    metadata.insert("system.identitySource".to_string(), normalized);
    if is_linux_machine_identity_source(&lower) {
        metadata.extend(parse_linux_machine_identity_metadata(&lower, &text));
    }
    if is_linux_network_identity_source(&lower) {
        metadata.extend(parse_linux_network_config_metadata(&lower, &text));
    }
    if is_linux_hardware_inventory_source(&lower) {
        metadata.extend(parse_linux_hardware_inventory_metadata(&lower, &text));
    }
    if is_unix_account_identity_source(&lower) {
        metadata.extend(parse_unix_account_metadata(&lower, &text));
    }
    if is_macos_local_user_source(&lower) {
        metadata.extend(parse_macos_local_user_metadata(data));
    }
    if is_macos_local_group_source(&lower) {
        metadata.extend(parse_macos_local_group_metadata(data));
    }
    if is_macos_hardware_identity_source(&lower) {
        metadata.extend(parse_macos_hardware_identity_metadata(data));
    }
    if is_macos_kernel_extension_source(&lower) {
        metadata.extend(parse_macos_kernel_extension_metadata(data));
    }
    if is_windows_wifi_profile_source(&lower) {
        metadata.extend(parse_windows_wifi_profile_metadata(&text));
    }
    if is_command_history_source(&lower) {
        metadata.extend(parse_command_history_metadata(&lower, &text));
    }
    if is_firewall_identity_source(&lower) {
        metadata.extend(parse_firewall_metadata(&lower, data, &text));
    }

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
        "machine-info" => {
            metadata.extend(parse_linux_machine_info_metadata(&text));
        }
        "hostname" => {
            insert_trimmed_metadata(&mut metadata, "system.hostname", &text);
        }
        "timezone" => {
            insert_trimmed_metadata(&mut metadata, "system.timeZone", &text);
        }
        "localtime" => {
            metadata.extend(parse_localtime_metadata(data, &text));
        }
        "locale" => {
            metadata.extend(parse_linux_locale_metadata(&text));
        }
        "fstab" | "mtab" => {
            metadata.extend(parse_mount_table_metadata(&text));
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
        "product_version" => {
            insert_trimmed_metadata(&mut metadata, "system.productVersion", &text);
        }
        "product_family" => {
            insert_trimmed_metadata(&mut metadata, "system.family", &text);
        }
        "product_sku" => {
            insert_trimmed_metadata(&mut metadata, "system.sku", &text);
        }
        "sys_vendor" => {
            insert_trimmed_metadata(&mut metadata, "system.manufacturer", &text);
        }
        "board_asset_tag" => {
            insert_trimmed_metadata(&mut metadata, "system.boardAssetTag", &text);
        }
        "board_serial" => {
            insert_trimmed_metadata(&mut metadata, "system.boardSerial", &text);
        }
        "board_name" => {
            insert_trimmed_metadata(&mut metadata, "system.boardName", &text);
        }
        "board_vendor" => {
            insert_trimmed_metadata(&mut metadata, "system.boardVendor", &text);
        }
        "board_version" => {
            insert_trimmed_metadata(&mut metadata, "system.boardVersion", &text);
        }
        "bios_version" => {
            insert_trimmed_metadata(&mut metadata, "system.biosVersion", &text);
        }
        "bios_vendor" => {
            insert_trimmed_metadata(&mut metadata, "system.biosVendor", &text);
        }
        "bios_date" => {
            insert_trimmed_metadata(&mut metadata, "system.biosDate", &text);
        }
        "chassis_asset_tag" => {
            insert_trimmed_metadata(&mut metadata, "system.chassisAssetTag", &text);
        }
        "chassis_vendor" => {
            insert_trimmed_metadata(&mut metadata, "system.chassisVendor", &text);
        }
        "chassis_type" => {
            insert_trimmed_metadata(&mut metadata, "system.chassisType", &text);
        }
        "chassis_serial" => {
            insert_trimmed_metadata(&mut metadata, "system.chassisSerial", &text);
        }
        "chassis_version" => {
            insert_trimmed_metadata(&mut metadata, "system.chassisVersion", &text);
        }
        "networkinterfaces.plist" => {
            metadata.extend(parse_macos_network_interfaces_metadata(data));
        }
        "com.apple.airport.preferences.plist" | "com.apple.wifi.known-networks.plist" => {
            metadata.extend(parse_macos_wifi_metadata(data));
        }
        "com.apple.alf.plist" => {
            metadata.extend(parse_macos_firewall_metadata(data));
        }
        ".globalpreferences.plist" => {
            metadata.extend(parse_macos_global_preferences_metadata(data));
        }
        "installhistory.plist" => {
            metadata.extend(parse_macos_install_history_metadata(data));
        }
        "diskmanagement.plist" => {
            metadata.extend(parse_macos_disk_management_metadata(data));
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

fn parse_linux_machine_identity_metadata(source_id: &str, text: &str) -> BTreeMap<String, String> {
    if source_id.ends_with("/var/lib/dbus/machine-id") {
        let mut metadata = BTreeMap::new();
        insert_trimmed_metadata(&mut metadata, "system.machineId", text);
        metadata.insert(
            "system.machineIdSource".to_string(),
            "dbus-machine-id".to_string(),
        );
        return metadata;
    }
    if source_id.ends_with("/etc/machine-info") {
        return parse_linux_machine_info_metadata(text);
    }
    if source_id.ends_with("/etc/default/locale") {
        return parse_linux_locale_metadata(text);
    }
    BTreeMap::new()
}

fn parse_linux_machine_info_metadata(text: &str) -> BTreeMap<String, String> {
    let values = parse_key_value_lines(text);
    let mut metadata = BTreeMap::new();

    if let Some(value) = values.get("PRETTY_HOSTNAME") {
        metadata.insert("system.prettyHostname".to_string(), value.clone());
    }
    if let Some(value) = values.get("ICON_NAME") {
        metadata.insert("system.iconName".to_string(), value.clone());
    }
    if let Some(value) = values.get("CHASSIS") {
        metadata.insert("system.chassis".to_string(), value.clone());
    }
    if let Some(value) = values.get("DEPLOYMENT") {
        metadata.insert("system.deployment".to_string(), value.clone());
    }
    if let Some(value) = values.get("LOCATION") {
        metadata.insert("system.location".to_string(), value.clone());
    }

    metadata
}

fn parse_linux_locale_metadata(text: &str) -> BTreeMap<String, String> {
    let values = parse_key_value_lines(text);
    let mut metadata = BTreeMap::new();

    if let Some(value) = values.get("LANG") {
        metadata.insert("system.locale".to_string(), value.clone());
    }
    if let Some(value) = values.get("LANGUAGE") {
        metadata.insert("system.language".to_string(), value.clone());
    }
    if let Some(value) = values.get("LC_TIME") {
        metadata.insert("system.localeTime".to_string(), value.clone());
    }
    if let Some(value) = values.get("LC_NUMERIC") {
        metadata.insert("system.localeNumeric".to_string(), value.clone());
    }

    metadata
}

fn parse_linux_hardware_inventory_metadata(
    source_id: &str,
    text: &str,
) -> BTreeMap<String, String> {
    if source_id
        .rsplit('/')
        .next()
        .is_some_and(|name| matches!(name, "lshw" | "lshw.txt" | "lshw-short.txt"))
    {
        return parse_lshw_text_metadata(text);
    }

    parse_dmidecode_metadata(text)
}

fn parse_dmidecode_metadata(text: &str) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let mut section = "";

    for raw_line in text.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !raw_line.starts_with(char::is_whitespace) && trimmed.ends_with("Information") {
            section = trimmed;
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let value = normalize_linux_hardware_value(value);
        if value.is_empty() {
            continue;
        }

        match (section, key.trim()) {
            ("System Information", "Manufacturer") => {
                metadata.insert("system.manufacturer".to_string(), value);
            }
            ("System Information", "Product Name") => {
                metadata.insert("system.model".to_string(), value);
            }
            ("System Information", "Version") => {
                metadata.insert("system.productVersion".to_string(), value);
            }
            ("System Information", "Serial Number") => {
                metadata.insert("system.serialNumber".to_string(), value);
            }
            ("System Information", "UUID") => {
                metadata.insert("system.hardwareUuid".to_string(), value);
            }
            ("System Information", "SKU Number") => {
                metadata.insert("system.sku".to_string(), value);
            }
            ("System Information", "Family") => {
                metadata.insert("system.family".to_string(), value);
            }
            ("Base Board Information", "Manufacturer") => {
                metadata.insert("system.boardVendor".to_string(), value);
            }
            ("Base Board Information", "Product Name") => {
                metadata.insert("system.boardName".to_string(), value);
            }
            ("Base Board Information", "Version") => {
                metadata.insert("system.boardVersion".to_string(), value);
            }
            ("Base Board Information", "Serial Number") => {
                metadata.insert("system.boardSerial".to_string(), value);
            }
            ("Base Board Information", "Asset Tag") => {
                metadata.insert("system.boardAssetTag".to_string(), value);
            }
            ("BIOS Information", "Vendor") => {
                metadata.insert("system.biosVendor".to_string(), value);
            }
            ("BIOS Information", "Version") => {
                metadata.insert("system.biosVersion".to_string(), value);
            }
            ("BIOS Information", "Release Date") => {
                metadata.insert("system.biosDate".to_string(), value);
            }
            ("Chassis Information", "Manufacturer") => {
                metadata.insert("system.chassisVendor".to_string(), value);
            }
            ("Chassis Information", "Type") => {
                metadata.insert("system.chassisType".to_string(), value);
            }
            ("Chassis Information", "Serial Number") => {
                metadata.insert("system.chassisSerial".to_string(), value);
            }
            ("Chassis Information", "Asset Tag") => {
                metadata.insert("system.chassisAssetTag".to_string(), value);
            }
            ("Chassis Information", "Version") => {
                metadata.insert("system.chassisVersion".to_string(), value);
            }
            _ => {}
        }
    }

    if !metadata.is_empty() {
        metadata.insert(
            "system.hardwareInventorySource".to_string(),
            "dmidecode".to_string(),
        );
    }
    metadata
}

fn parse_lshw_text_metadata(text: &str) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let mut section = "system";

    for raw_line in text.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("*-") {
            section = trimmed
                .trim_start_matches("*-")
                .split_whitespace()
                .next()
                .unwrap_or("");
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let value = normalize_linux_hardware_value(value);
        if value.is_empty() {
            continue;
        }

        match (section, key.trim()) {
            ("system", "vendor") => {
                metadata.insert("system.manufacturer".to_string(), value);
            }
            ("system", "product") => {
                metadata.insert("system.model".to_string(), value);
            }
            ("system", "version") => {
                metadata.insert("system.productVersion".to_string(), value);
            }
            ("system", "serial") => {
                metadata.insert("system.serialNumber".to_string(), value);
            }
            ("core", "vendor") => {
                metadata.insert("system.boardVendor".to_string(), value);
            }
            ("core", "product") => {
                metadata.insert("system.boardName".to_string(), value);
            }
            ("core", "version") => {
                metadata.insert("system.boardVersion".to_string(), value);
            }
            ("core", "serial") => {
                metadata.insert("system.boardSerial".to_string(), value);
            }
            ("firmware", "vendor") => {
                metadata.insert("system.biosVendor".to_string(), value);
            }
            ("firmware", "version") => {
                metadata.insert("system.biosVersion".to_string(), value);
            }
            ("firmware", "date") => {
                metadata.insert("system.biosDate".to_string(), value);
            }
            _ => {}
        }
    }

    if !metadata.is_empty() {
        metadata.insert(
            "system.hardwareInventorySource".to_string(),
            "lshw".to_string(),
        );
    }
    metadata
}

fn normalize_linux_hardware_value(value: &str) -> String {
    let value = value.trim().trim_matches('"').trim_matches('\'').trim();
    if value.is_empty()
        || matches!(
            value.to_ascii_lowercase().as_str(),
            "not specified"
                | "not available"
                | "to be filled by o.e.m."
                | "to be filled by oem"
                | "none"
                | "unknown"
        )
    {
        return String::new();
    }
    truncate_metadata_value(value, 180)
}

fn parse_localtime_metadata(data: &[u8], text: &str) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let trimmed = text.trim();
    if !data.starts_with(b"TZif") {
        if let Some(zone) = trimmed
            .split("/zoneinfo/")
            .nth(1)
            .or_else(|| trimmed.strip_prefix("zoneinfo/"))
            .filter(|value| !value.is_empty())
        {
            metadata.insert(
                "system.timeZone".to_string(),
                truncate_metadata_value(zone.trim_matches('/'), 180),
            );
        }
        return metadata;
    }

    metadata.insert("system.timeZoneFormat".to_string(), "TZif".to_string());
    if let Some(version) = data.get(4).copied().filter(|byte| *byte != 0) {
        metadata.insert(
            "system.timeZoneFileVersion".to_string(),
            (version as char).to_string(),
        );
    }
    if let Some(posix_rule) = tzif_posix_rule(data) {
        metadata.insert("system.timeZoneRule".to_string(), posix_rule);
    }
    let abbreviations = tzif_abbreviations(data);
    insert_joined_metadata(
        &mut metadata,
        "system.timeZoneAbbreviations",
        &abbreviations,
    );

    metadata
}

fn tzif_posix_rule(data: &[u8]) -> Option<String> {
    data.get(4)
        .copied()
        .filter(|byte| *byte == b'2' || *byte == b'3' || *byte == b'4')?;
    let last_newline = data.iter().rposition(|byte| *byte == b'\n')?;
    let previous_newline = data[..last_newline]
        .iter()
        .rposition(|byte| *byte == b'\n')?;
    let value = std::str::from_utf8(&data[previous_newline + 1..last_newline])
        .ok()?
        .trim();
    (!value.is_empty() && value.chars().all(|ch| ch.is_ascii_graphic()))
        .then(|| truncate_metadata_value(value, MAX_ARTIFACT_METADATA_VALUE_CHARS))
}

#[derive(Clone, Copy)]
struct TzifCounts {
    ttisgmt_count: usize,
    ttisstd_count: usize,
    leap_count: usize,
    time_count: usize,
    type_count: usize,
    char_count: usize,
}

fn tzif_abbreviations(data: &[u8]) -> Vec<String> {
    let Some(counts) = tzif_counts(data, 20) else {
        return Vec::new();
    };
    let mut offset = 44usize;
    let first_block = tzif_block_len(counts, 4);
    let mut char_offset = tzif_char_offset(offset, counts, 4);
    let mut char_len = counts.char_count;

    if matches!(data.get(4), Some(b'2' | b'3' | b'4')) {
        offset = offset.saturating_add(first_block);
        if offset + 44 <= data.len() && data.get(offset..offset + 4) == Some(b"TZif") {
            if let Some(counts) = tzif_counts(data, offset + 20) {
                char_offset = tzif_char_offset(offset + 44, counts, 8);
                char_len = counts.char_count;
            }
        }
    }

    let Some(char_offset) = char_offset else {
        return Vec::new();
    };
    let Some(end) = char_offset.checked_add(char_len) else {
        return Vec::new();
    };
    let Some(bytes) = data.get(char_offset..end) else {
        return Vec::new();
    };

    let mut abbreviations = Vec::new();
    for part in bytes.split(|byte| *byte == 0) {
        let Ok(value) = std::str::from_utf8(part) else {
            continue;
        };
        let value = value.trim();
        if value.len() >= 2
            && value.len() <= 12
            && value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-'))
        {
            push_unique_limited(&mut abbreviations, value.to_string());
        }
    }
    abbreviations
}

fn tzif_counts(data: &[u8], offset: usize) -> Option<TzifCounts> {
    Some(TzifCounts {
        ttisgmt_count: read_be_u32_usize(data, offset)?,
        ttisstd_count: read_be_u32_usize(data, offset + 4)?,
        leap_count: read_be_u32_usize(data, offset + 8)?,
        time_count: read_be_u32_usize(data, offset + 12)?,
        type_count: read_be_u32_usize(data, offset + 16)?,
        char_count: read_be_u32_usize(data, offset + 20)?,
    })
}

fn read_be_u32_usize(data: &[u8], offset: usize) -> Option<usize> {
    let bytes: [u8; 4] = data.get(offset..offset + 4)?.try_into().ok()?;
    usize::try_from(u32::from_be_bytes(bytes)).ok()
}

fn tzif_transition_block_len(time_count: usize, time_size: usize) -> usize {
    time_count.saturating_mul(time_size)
}

fn tzif_char_offset(offset: usize, counts: TzifCounts, time_size: usize) -> Option<usize> {
    offset
        .checked_add(tzif_transition_block_len(counts.time_count, time_size))
        .and_then(|value| value.checked_add(counts.time_count))
        .and_then(|value| value.checked_add(counts.type_count.checked_mul(6)?))
}

fn tzif_block_len(counts: TzifCounts, time_size: usize) -> usize {
    tzif_char_offset(0, counts, time_size)
        .unwrap_or(usize::MAX)
        .saturating_add(counts.char_count)
        .saturating_add(
            counts
                .leap_count
                .saturating_mul(time_size.saturating_add(4)),
        )
        .saturating_add(counts.ttisstd_count)
        .saturating_add(counts.ttisgmt_count)
}

fn parse_mount_table_metadata(text: &str) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let mut descriptions = Vec::new();
    let mut root_device = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 3 {
            continue;
        }
        let device = decode_mount_field(fields[0]);
        let mount_point = decode_mount_field(fields[1]);
        let fs_type = fields[2].to_string();
        if mount_point == "/" && root_device.is_none() {
            root_device = Some(device.clone());
        }
        let options = fields.get(3).copied().unwrap_or("-");
        push_unique_limited(
            &mut descriptions,
            truncate_metadata_value(
                &format!("{device} on {mount_point} ({fs_type}, {options})"),
                MAX_SYSTEM_MOUNT_DESCRIPTION_CHARS,
            ),
        );
    }

    if let Some(root_device) = root_device {
        metadata.insert("system.rootDevice".to_string(), root_device);
    }
    if !descriptions.is_empty() {
        metadata.insert(
            "system.mountCount".to_string(),
            descriptions.len().to_string(),
        );
        metadata.insert(
            "system.mounts".to_string(),
            truncate_metadata_value(&descriptions.join("; "), MAX_ARTIFACT_METADATA_VALUE_CHARS),
        );
    }

    metadata
}

fn decode_mount_field(value: &str) -> String {
    value
        .replace("\\040", " ")
        .replace("\\011", "\t")
        .replace("\\012", "\n")
        .replace("\\134", "\\")
}

#[derive(Default)]
struct LinuxNetworkMetadata {
    interfaces: Vec<String>,
    addresses: Vec<String>,
    gateways: Vec<String>,
    dns_servers: Vec<String>,
    methods: Vec<String>,
    search_domains: Vec<String>,
    host_aliases: Vec<String>,
    connection_ids: Vec<String>,
    connection_uuids: Vec<String>,
    mac_addresses: Vec<String>,
    wifi_ssids: Vec<String>,
}

fn parse_linux_network_config_metadata(source_id: &str, text: &str) -> BTreeMap<String, String> {
    let mut values = LinuxNetworkMetadata::default();
    let config_type = if source_id.ends_with("/etc/network/interfaces") {
        parse_debian_interfaces_metadata(text, &mut values);
        "debian-interfaces"
    } else if source_id.ends_with("/etc/resolv.conf") {
        parse_resolv_conf_metadata(text, &mut values);
        "resolver"
    } else if source_id.ends_with("/etc/hosts")
        || source_id.ends_with("/windows/system32/drivers/etc/hosts")
    {
        parse_hosts_metadata(text, &mut values);
        "hosts"
    } else if source_id.contains("/etc/sysconfig/network-scripts/ifcfg-") {
        parse_ifcfg_metadata(source_id, text, &mut values);
        "ifcfg"
    } else if source_id.contains("/etc/networkmanager/system-connections/") {
        parse_network_manager_metadata(text, &mut values);
        "networkmanager"
    } else {
        parse_netplan_metadata(text, &mut values);
        "netplan"
    };

    let mut metadata = BTreeMap::new();
    metadata.insert(
        "system.networkConfigType".to_string(),
        config_type.to_string(),
    );
    insert_joined_metadata(
        &mut metadata,
        "system.networkInterfaces",
        &values.interfaces,
    );
    insert_joined_metadata(&mut metadata, "system.ipv4Addresses", &values.addresses);
    insert_joined_metadata(&mut metadata, "system.gateways", &values.gateways);
    insert_joined_metadata(&mut metadata, "system.dnsServers", &values.dns_servers);
    insert_joined_metadata(&mut metadata, "system.networkMethods", &values.methods);
    insert_joined_metadata(
        &mut metadata,
        "system.dnsSearchDomains",
        &values.search_domains,
    );
    insert_joined_metadata(&mut metadata, "system.hostAliases", &values.host_aliases);
    insert_joined_metadata(
        &mut metadata,
        "system.connectionIds",
        &values.connection_ids,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.connectionUuids",
        &values.connection_uuids,
    );
    insert_joined_metadata(&mut metadata, "system.macAddresses", &values.mac_addresses);
    insert_joined_metadata(&mut metadata, "system.wifiSsids", &values.wifi_ssids);
    metadata
}

fn parse_debian_interfaces_metadata(text: &str, values: &mut LinuxNetworkMetadata) {
    let mut current_interface: Option<String> = None;
    for raw_line in text.lines() {
        let line = trim_network_config_line(raw_line);
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.as_slice() {
            ["auto" | "allow-hotplug", interfaces @ ..] => {
                for interface in interfaces {
                    push_unique_limited(&mut values.interfaces, (*interface).to_string());
                }
            }
            ["iface", interface, family, method, ..] => {
                current_interface = Some((*interface).to_string());
                push_unique_limited(&mut values.interfaces, (*interface).to_string());
                if *family == "inet" || *family == "inet6" {
                    push_unique_limited(
                        &mut values.methods,
                        format!("{interface}:{family}:{method}"),
                    );
                }
            }
            ["address", address, ..] => {
                push_unique_limited(&mut values.addresses, (*address).to_string());
            }
            ["gateway", gateway, ..] => {
                push_unique_limited(&mut values.gateways, (*gateway).to_string());
            }
            ["dns-nameservers", servers @ ..] => {
                for server in servers {
                    push_unique_limited(&mut values.dns_servers, (*server).to_string());
                }
            }
            ["hwaddress", "ether", mac, ..] => {
                if let Some(interface) = &current_interface {
                    push_unique_limited(&mut values.interfaces, format!("{interface} ({mac})"));
                }
            }
            _ => {}
        }
    }
}

fn parse_ifcfg_metadata(source_id: &str, text: &str, values: &mut LinuxNetworkMetadata) {
    let pairs = parse_key_value_lines(text);
    let interface = pairs
        .get("DEVICE")
        .or_else(|| pairs.get("NAME"))
        .cloned()
        .or_else(|| {
            source_id
                .rsplit('/')
                .next()
                .and_then(|name| name.strip_prefix("ifcfg-"))
                .map(ToString::to_string)
        });

    if let Some(interface) = &interface {
        push_unique_limited(&mut values.interfaces, interface.clone());
    }
    if let Some(address) = pairs.get("IPADDR") {
        let address = pairs
            .get("PREFIX")
            .map(|prefix| format!("{address}/{prefix}"))
            .or_else(|| {
                pairs
                    .get("NETMASK")
                    .map(|netmask| format!("{address}/{netmask}"))
            })
            .unwrap_or_else(|| address.clone());
        push_unique_limited(&mut values.addresses, address);
    }
    if let Some(gateway) = pairs.get("GATEWAY") {
        push_unique_limited(&mut values.gateways, gateway.clone());
    }
    for key in ["DNS1", "DNS2", "DNS3"] {
        if let Some(server) = pairs.get(key) {
            push_unique_limited(&mut values.dns_servers, server.clone());
        }
    }
    if let (Some(interface), Some(method)) = (interface.as_deref(), pairs.get("BOOTPROTO")) {
        push_unique_limited(&mut values.methods, format!("{interface}:inet:{method}"));
    }
}

fn parse_resolv_conf_metadata(text: &str, values: &mut LinuxNetworkMetadata) {
    for raw_line in text.lines() {
        let line = trim_network_config_line(raw_line);
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.as_slice() {
            ["nameserver", server, ..] => {
                push_unique_limited(&mut values.dns_servers, (*server).to_string());
            }
            ["domain", domain, ..] => {
                push_unique_limited(&mut values.search_domains, (*domain).to_string());
            }
            ["search", domains @ ..] => {
                for domain in domains {
                    push_unique_limited(&mut values.search_domains, (*domain).to_string());
                }
            }
            _ => {}
        }
    }
}

fn parse_hosts_metadata(text: &str, values: &mut LinuxNetworkMetadata) {
    for raw_line in text.lines() {
        let line = trim_network_config_line(raw_line);
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        let Some((address, aliases)) = parts.split_first() else {
            continue;
        };
        if aliases.is_empty() {
            continue;
        }
        let alias_list = aliases.join(",");
        push_unique_limited(
            &mut values.host_aliases,
            truncate_metadata_value(
                &format!("{address}={alias_list}"),
                MAX_SYSTEM_MOUNT_DESCRIPTION_CHARS,
            ),
        );
    }
}

fn parse_network_manager_metadata(text: &str, values: &mut LinuxNetworkMetadata) {
    let mut section = "";
    let mut connection_type: Option<String> = None;
    let mut interface: Option<String> = None;

    for raw_line in text.lines() {
        let line = trim_network_config_line(raw_line);
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line.trim_start_matches('[').trim_end_matches(']');
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim().trim_matches('"').trim_matches('\'');
        if value.is_empty() {
            continue;
        }

        match (section, key) {
            ("connection", "id") => {
                push_unique_limited(&mut values.connection_ids, value.to_string());
            }
            ("connection", "uuid") => {
                push_unique_limited(&mut values.connection_uuids, value.to_string());
            }
            ("connection", "type") => {
                connection_type = Some(value.to_string());
            }
            ("connection", "interface-name") => {
                interface = Some(value.to_string());
                push_unique_limited(&mut values.interfaces, value.to_string());
            }
            ("wifi", "ssid") => {
                push_unique_limited(&mut values.wifi_ssids, value.to_string());
            }
            ("wifi", "mac-address") | ("ethernet", "mac-address") => {
                push_unique_limited(&mut values.mac_addresses, value.to_string());
            }
            ("ipv4" | "ipv6", "method") => {
                let interface = interface.as_deref().unwrap_or("unknown");
                let family = if section == "ipv6" { "inet6" } else { "inet" };
                push_unique_limited(&mut values.methods, format!("{interface}:{family}:{value}"));
            }
            ("ipv4" | "ipv6", "addresses") | ("ipv4" | "ipv6", "address1") => {
                collect_network_manager_addresses(value, values);
            }
            ("ipv4" | "ipv6", "gateway") => {
                push_unique_limited(&mut values.gateways, value.to_string());
            }
            ("ipv4" | "ipv6", "dns") => {
                for server in split_network_manager_list(value) {
                    push_unique_limited(&mut values.dns_servers, server);
                }
            }
            ("ipv4" | "ipv6", "dns-search") => {
                for domain in split_network_manager_list(value) {
                    push_unique_limited(&mut values.search_domains, domain);
                }
            }
            _ => {}
        }
    }

    if values.interfaces.is_empty() {
        if let Some(connection_type) = connection_type {
            push_unique_limited(&mut values.interfaces, connection_type);
        }
    }
}

fn collect_network_manager_addresses(value: &str, values: &mut LinuxNetworkMetadata) {
    for address in value.split(';') {
        let address = address.trim();
        if address.is_empty() {
            continue;
        }
        let mut parts = address
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty());
        if let Some(ip) = parts.next() {
            push_unique_limited(&mut values.addresses, ip.to_string());
        }
        if let Some(gateway) = parts.next() {
            push_unique_limited(&mut values.gateways, gateway.to_string());
        }
    }
}

fn split_network_manager_list(value: &str) -> Vec<String> {
    value
        .split([';', ',', ' '])
        .filter_map(|part| {
            let part = part.trim();
            (!part.is_empty()).then(|| truncate_metadata_value(part, 120))
        })
        .collect()
}

fn parse_netplan_metadata(text: &str, values: &mut LinuxNetworkMetadata) {
    let mut current_interface: Option<String> = None;
    let mut in_nameservers = false;
    let mut pending_address_list = false;
    let mut pending_dns_list = false;

    for raw_line in text.lines() {
        let line = trim_network_config_line(raw_line);
        if line.is_empty() {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.ends_with(':') {
            let key = trimmed.trim_end_matches(':').trim().trim_matches('"');
            if key == "nameservers" {
                in_nameservers = true;
            } else {
                in_nameservers = false;
                pending_dns_list = false;
            }
            if is_netplan_interface_key(key) {
                current_interface = Some(key.to_string());
                push_unique_limited(&mut values.interfaces, key.to_string());
            }
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("- ") {
            if pending_dns_list || in_nameservers {
                for server in split_config_values(value) {
                    push_unique_limited(&mut values.dns_servers, server);
                }
            } else if pending_address_list {
                for address in split_config_values(value) {
                    push_unique_limited(&mut values.addresses, address);
                }
            }
            continue;
        }

        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        pending_address_list = false;
        pending_dns_list = false;

        match key {
            "addresses" if in_nameservers => {
                if value.is_empty() {
                    pending_dns_list = true;
                } else {
                    for server in split_config_values(value) {
                        push_unique_limited(&mut values.dns_servers, server);
                    }
                }
            }
            "addresses" => {
                if value.is_empty() {
                    pending_address_list = true;
                } else {
                    for address in split_config_values(value) {
                        push_unique_limited(&mut values.addresses, address);
                    }
                }
            }
            "gateway4" | "gateway6" => {
                for gateway in split_config_values(value) {
                    push_unique_limited(&mut values.gateways, gateway);
                }
            }
            "dhcp4" | "dhcp6" if value.eq_ignore_ascii_case("true") => {
                let interface = current_interface.as_deref().unwrap_or("unknown");
                let family = if key == "dhcp6" { "inet6" } else { "inet" };
                push_unique_limited(&mut values.methods, format!("{interface}:{family}:dhcp"));
            }
            _ => {}
        }
    }
}

fn trim_network_config_line(line: &str) -> &str {
    line.split_once('#')
        .map_or(line, |(before, _)| before)
        .trim()
}

fn is_netplan_interface_key(key: &str) -> bool {
    !matches!(
        key,
        "network"
            | "version"
            | "renderer"
            | "ethernets"
            | "wifis"
            | "bridges"
            | "bonds"
            | "vlans"
            | "addresses"
            | "nameservers"
            | "routes"
            | "gateway4"
            | "gateway6"
            | "dhcp4"
            | "dhcp6"
            | "optional"
            | "match"
            | "set-name"
    )
}

fn split_config_values(value: &str) -> Vec<String> {
    value
        .trim()
        .trim_matches('[')
        .trim_matches(']')
        .split([',', ' '])
        .filter_map(|part| {
            let part = part.trim().trim_matches('"').trim_matches('\'');
            (!part.is_empty()).then(|| truncate_metadata_value(part, 120))
        })
        .collect()
}

#[derive(Default)]
struct UnixAccountMetadata {
    user_count: usize,
    regular_user_count: usize,
    login_user_count: usize,
    group_count: usize,
    shadow_entry_count: usize,
    password_hash_user_count: usize,
    password_locked_user_count: usize,
    password_disabled_user_count: usize,
    password_empty_user_count: usize,
    users: Vec<String>,
    regular_users: Vec<String>,
    login_users: Vec<String>,
    password_hash_users: Vec<String>,
    password_locked_users: Vec<String>,
    password_disabled_users: Vec<String>,
    password_empty_users: Vec<String>,
    password_hash_algorithms: Vec<String>,
    home_directories: Vec<String>,
    login_shells: Vec<String>,
    groups: Vec<String>,
    admin_groups: Vec<String>,
    group_members: Vec<String>,
    min_uid: Option<u32>,
    max_uid: Option<u32>,
    root_present: bool,
}

fn parse_unix_account_metadata(source_id: &str, text: &str) -> BTreeMap<String, String> {
    let mut values = UnixAccountMetadata::default();
    let config_type = if source_id.ends_with("/etc/group")
        || source_id.ends_with("/private/etc/group")
    {
        parse_unix_group_metadata(text, &mut values);
        "unix-group"
    } else if source_id.ends_with("/etc/shadow") || source_id.ends_with("/private/etc/shadow") {
        parse_unix_shadow_metadata(text, &mut values);
        "unix-shadow"
    } else if source_id.ends_with("/etc/gshadow") || source_id.ends_with("/private/etc/gshadow") {
        parse_unix_gshadow_metadata(text, &mut values);
        "unix-gshadow"
    } else {
        parse_unix_passwd_metadata(text, &mut values);
        "unix-passwd"
    };

    unix_account_metadata_to_map(values, config_type)
}

fn parse_unix_passwd_metadata(text: &str, values: &mut UnixAccountMetadata) {
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 7 {
            continue;
        }
        let name = fields[0].trim();
        let uid = fields[2].trim().parse::<u32>().ok();
        let gid = fields[3].trim();
        let gecos = fields[4].trim();
        let home = fields[5].trim();
        let shell = fields[6].trim();
        if name.is_empty() {
            continue;
        }

        values.user_count = values.user_count.saturating_add(1);
        if name == "root" {
            values.root_present = true;
        }
        if let Some(uid) = uid {
            values.min_uid = Some(values.min_uid.map_or(uid, |current| current.min(uid)));
            values.max_uid = Some(values.max_uid.map_or(uid, |current| current.max(uid)));
        }
        push_unique_limited(
            &mut values.users,
            truncate_metadata_value(&format!("{name}:uid={}", uid.unwrap_or(0)), 120),
        );

        if is_unix_regular_user(uid) {
            values.regular_user_count = values.regular_user_count.saturating_add(1);
            let display_name = if gecos.is_empty() {
                name.to_string()
            } else {
                format!("{name} ({gecos})")
            };
            push_unique_limited(
                &mut values.regular_users,
                truncate_metadata_value(&display_name, 120),
            );
        }
        if is_unix_login_shell(shell) {
            values.login_user_count = values.login_user_count.saturating_add(1);
            push_unique_limited(
                &mut values.login_users,
                truncate_metadata_value(&format!("{name}:uid={}:gid={gid}", uid.unwrap_or(0)), 120),
            );
            push_unique_limited(
                &mut values.login_shells,
                truncate_metadata_value(shell, 120),
            );
        }
        if !home.is_empty() && home != "/" && home != "/nonexistent" {
            push_unique_limited(
                &mut values.home_directories,
                truncate_metadata_value(home, 160),
            );
        }
    }
}

fn parse_unix_group_metadata(text: &str, values: &mut UnixAccountMetadata) {
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 4 {
            continue;
        }
        let name = fields[0].trim();
        let gid = fields[2].trim();
        let members = fields[3].trim();
        if name.is_empty() {
            continue;
        }

        values.group_count = values.group_count.saturating_add(1);
        push_unique_limited(
            &mut values.groups,
            truncate_metadata_value(&format!("{name}:gid={gid}"), 120),
        );
        if is_unix_admin_group(name) {
            push_unique_limited(
                &mut values.admin_groups,
                truncate_metadata_value(&format!("{name}:members={members}"), 160),
            );
        }
        if !members.is_empty() {
            push_unique_limited(
                &mut values.group_members,
                truncate_metadata_value(&format!("{name}={members}"), 160),
            );
        }
    }
}

fn parse_unix_shadow_metadata(text: &str, values: &mut UnixAccountMetadata) {
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 2 {
            continue;
        }
        let name = fields[0].trim();
        let credential = fields[1].trim();
        if name.is_empty() {
            continue;
        }

        values.shadow_entry_count = values.shadow_entry_count.saturating_add(1);
        match unix_shadow_password_status(credential) {
            UnixShadowPasswordStatus::HasHash(algorithm) => {
                values.password_hash_user_count = values.password_hash_user_count.saturating_add(1);
                push_unique_limited(
                    &mut values.password_hash_users,
                    truncate_metadata_value(name, 120),
                );
                push_unique_limited(&mut values.password_hash_algorithms, algorithm.to_string());
            }
            UnixShadowPasswordStatus::Locked => {
                values.password_locked_user_count =
                    values.password_locked_user_count.saturating_add(1);
                push_unique_limited(
                    &mut values.password_locked_users,
                    truncate_metadata_value(name, 120),
                );
            }
            UnixShadowPasswordStatus::Disabled => {
                values.password_disabled_user_count =
                    values.password_disabled_user_count.saturating_add(1);
                push_unique_limited(
                    &mut values.password_disabled_users,
                    truncate_metadata_value(name, 120),
                );
            }
            UnixShadowPasswordStatus::Empty => {
                values.password_empty_user_count =
                    values.password_empty_user_count.saturating_add(1);
                push_unique_limited(
                    &mut values.password_empty_users,
                    truncate_metadata_value(name, 120),
                );
            }
        }
    }
}

fn parse_unix_gshadow_metadata(text: &str, values: &mut UnixAccountMetadata) {
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 4 {
            continue;
        }
        let name = fields[0].trim();
        let admins = fields[2].trim();
        let members = fields[3].trim();
        if name.is_empty() {
            continue;
        }

        values.group_count = values.group_count.saturating_add(1);
        push_unique_limited(&mut values.groups, truncate_metadata_value(name, 120));
        if !admins.is_empty() {
            push_unique_limited(
                &mut values.admin_groups,
                truncate_metadata_value(&format!("{name}:admins={admins}"), 160),
            );
        }
        if !members.is_empty() {
            push_unique_limited(
                &mut values.group_members,
                truncate_metadata_value(&format!("{name}={members}"), 160),
            );
        }
    }
}

enum UnixShadowPasswordStatus {
    HasHash(&'static str),
    Locked,
    Disabled,
    Empty,
}

fn unix_shadow_password_status(credential: &str) -> UnixShadowPasswordStatus {
    let credential = credential.trim();
    if credential.is_empty() {
        return UnixShadowPasswordStatus::Empty;
    }
    if credential.starts_with('!') {
        return UnixShadowPasswordStatus::Locked;
    }
    if credential.starts_with('*') {
        return UnixShadowPasswordStatus::Disabled;
    }
    UnixShadowPasswordStatus::HasHash(unix_shadow_hash_algorithm(credential))
}

fn unix_shadow_hash_algorithm(credential: &str) -> &'static str {
    if credential.starts_with("$y$") {
        "yescrypt"
    } else if credential.starts_with("$6$") {
        "sha512-crypt"
    } else if credential.starts_with("$5$") {
        "sha256-crypt"
    } else if credential.starts_with("$2a$")
        || credential.starts_with("$2b$")
        || credential.starts_with("$2y$")
    {
        "bcrypt"
    } else if credential.starts_with("$1$") {
        "md5-crypt"
    } else {
        "traditional-crypt"
    }
}

fn is_unix_regular_user(uid: Option<u32>) -> bool {
    uid.is_some_and(|uid| (UNIX_REGULAR_USER_MIN_UID..UNIX_REGULAR_USER_MAX_UID).contains(&uid))
}

fn is_unix_login_shell(shell: &str) -> bool {
    let shell = shell.trim();
    !shell.is_empty()
        && !shell.ends_with("/nologin")
        && !shell.ends_with("/false")
        && shell != "nologin"
        && shell != "false"
}

fn is_unix_admin_group(name: &str) -> bool {
    matches!(name, "admin" | "sudo" | "wheel" | "staff")
}

fn unix_account_metadata_to_map(
    values: UnixAccountMetadata,
    config_type: &str,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "system.accountConfigType".to_string(),
        config_type.to_string(),
    );
    if values.user_count > 0 {
        metadata.insert(
            "system.localUserCount".to_string(),
            values.user_count.to_string(),
        );
    }
    if values.regular_user_count > 0 {
        metadata.insert(
            "system.regularUserCount".to_string(),
            values.regular_user_count.to_string(),
        );
    }
    if values.login_user_count > 0 {
        metadata.insert(
            "system.loginUserCount".to_string(),
            values.login_user_count.to_string(),
        );
    }
    if values.group_count > 0 {
        metadata.insert(
            "system.localGroupCount".to_string(),
            values.group_count.to_string(),
        );
    }
    if values.shadow_entry_count > 0 {
        metadata.insert(
            "system.shadowEntryCount".to_string(),
            values.shadow_entry_count.to_string(),
        );
    }
    if values.password_hash_user_count > 0 {
        metadata.insert(
            "system.passwordHashUserCount".to_string(),
            values.password_hash_user_count.to_string(),
        );
    }
    if values.password_locked_user_count > 0 {
        metadata.insert(
            "system.passwordLockedUserCount".to_string(),
            values.password_locked_user_count.to_string(),
        );
    }
    if values.password_disabled_user_count > 0 {
        metadata.insert(
            "system.passwordDisabledUserCount".to_string(),
            values.password_disabled_user_count.to_string(),
        );
    }
    if values.password_empty_user_count > 0 {
        metadata.insert(
            "system.passwordEmptyUserCount".to_string(),
            values.password_empty_user_count.to_string(),
        );
    }
    if values.root_present {
        metadata.insert("system.rootAccountPresent".to_string(), "true".to_string());
    }
    if let (Some(min_uid), Some(max_uid)) = (values.min_uid, values.max_uid) {
        metadata.insert(
            "system.userUidRange".to_string(),
            format!("{min_uid}-{max_uid}"),
        );
    }
    insert_joined_metadata(&mut metadata, "system.localUsers", &values.users);
    insert_joined_metadata(&mut metadata, "system.regularUsers", &values.regular_users);
    insert_joined_metadata(&mut metadata, "system.loginUsers", &values.login_users);
    insert_joined_metadata(
        &mut metadata,
        "system.passwordHashUsers",
        &values.password_hash_users,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.passwordLockedUsers",
        &values.password_locked_users,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.passwordDisabledUsers",
        &values.password_disabled_users,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.passwordEmptyUsers",
        &values.password_empty_users,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.passwordHashAlgorithms",
        &values.password_hash_algorithms,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.homeDirectories",
        &values.home_directories,
    );
    insert_joined_metadata(&mut metadata, "system.loginShells", &values.login_shells);
    insert_joined_metadata(&mut metadata, "system.localGroups", &values.groups);
    insert_joined_metadata(&mut metadata, "system.adminGroups", &values.admin_groups);
    insert_joined_metadata(&mut metadata, "system.groupMembers", &values.group_members);
    metadata
}

fn parse_macos_local_user_metadata(data: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let Ok(value) = plist::from_bytes::<plist::Value>(data) else {
        return metadata;
    };
    let plist::Value::Dictionary(dict) = value else {
        return metadata;
    };

    let record_names = plist_dict_string_array(&dict, "name").unwrap_or_default();
    let Some(account_name) = record_names.first().cloned() else {
        return metadata;
    };

    metadata.insert(
        "system.accountConfigType".to_string(),
        "macos-dslocal-user".to_string(),
    );
    metadata.insert("system.localUserCount".to_string(), "1".to_string());
    metadata.insert(
        "system.localUsers".to_string(),
        truncate_metadata_value(&account_name, 120),
    );

    if let Some(uid) = plist_dict_first_string(&dict, "uid") {
        metadata.insert("system.userUidRange".to_string(), uid.clone());
        if macos_uid_is_regular(&uid) {
            metadata.insert("system.regularUserCount".to_string(), "1".to_string());
            metadata.insert(
                "system.regularUsers".to_string(),
                describe_macos_local_user(
                    &account_name,
                    plist_dict_first_string(&dict, "realname"),
                ),
            );
        }
    }
    if let Some(gid) = plist_dict_first_string(&dict, "gid") {
        metadata.insert("system.primaryGroupId".to_string(), gid);
    }
    if let Some(home) = plist_dict_first_string(&dict, "home") {
        metadata.insert("system.homeDirectories".to_string(), home);
    }
    if let Some(shell) = plist_dict_first_string(&dict, "shell") {
        if !shell.ends_with("/false") && !shell.ends_with("/nologin") {
            metadata.insert("system.loginUserCount".to_string(), "1".to_string());
            metadata.insert(
                "system.loginUsers".to_string(),
                describe_macos_local_user(
                    &account_name,
                    plist_dict_first_string(&dict, "realname"),
                ),
            );
        }
        metadata.insert("system.loginShells".to_string(), shell);
    }
    if let Some(real_name) = plist_dict_first_string(&dict, "realname") {
        metadata.insert(
            "system.userDisplayNames".to_string(),
            truncate_metadata_value(&real_name, 120),
        );
    }
    if let Some(uuid) = plist_dict_first_string(&dict, "generateduid") {
        metadata.insert("system.userGeneratedUids".to_string(), uuid);
    }
    if account_name == "root" {
        metadata.insert("system.rootAccountPresent".to_string(), "true".to_string());
    }

    metadata
}

fn parse_macos_local_group_metadata(data: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let Ok(value) = plist::from_bytes::<plist::Value>(data) else {
        return metadata;
    };
    let plist::Value::Dictionary(dict) = value else {
        return metadata;
    };

    let names = plist_dict_string_array(&dict, "name").unwrap_or_default();
    let Some(group_name) = names.first().cloned() else {
        return metadata;
    };

    metadata.insert(
        "system.accountConfigType".to_string(),
        "macos-dslocal-group".to_string(),
    );
    metadata.insert("system.localGroupCount".to_string(), "1".to_string());

    let group_description = if let Some(gid) = plist_dict_first_string(&dict, "gid") {
        format!(
            "{}:gid={}",
            truncate_metadata_value(&group_name, 120),
            truncate_metadata_value(&gid, 40)
        )
    } else {
        truncate_metadata_value(&group_name, 120)
    };
    metadata.insert("system.localGroups".to_string(), group_description);

    let users = plist_dict_string_array(&dict, "users").unwrap_or_default();
    if !users.is_empty() {
        let users = users
            .into_iter()
            .map(|user| truncate_metadata_value(&user, 120))
            .collect::<Vec<_>>()
            .join(",");
        metadata.insert(
            "system.groupMembers".to_string(),
            format!("{}={}", truncate_metadata_value(&group_name, 120), users),
        );
        if is_unix_admin_group(&group_name) {
            metadata.insert(
                "system.adminGroups".to_string(),
                format!(
                    "{}:members={}",
                    truncate_metadata_value(&group_name, 120),
                    metadata
                        .get("system.groupMembers")
                        .and_then(|value| value.split_once('=').map(|(_, members)| members))
                        .unwrap_or("")
                ),
            );
        }
    } else if is_unix_admin_group(&group_name) {
        metadata.insert(
            "system.adminGroups".to_string(),
            truncate_metadata_value(&group_name, 120),
        );
    }

    if let Some(generated_uids) = plist_dict_string_array(&dict, "groupmembers") {
        metadata.insert(
            "system.groupGeneratedUids".to_string(),
            truncate_metadata_value(
                &generated_uids
                    .into_iter()
                    .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
                    .collect::<Vec<_>>()
                    .join("; "),
                MAX_ARTIFACT_METADATA_VALUE_CHARS,
            ),
        );
    }

    metadata
}

fn insert_joined_metadata(metadata: &mut BTreeMap<String, String>, key: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    metadata.insert(
        key.to_string(),
        truncate_metadata_value(&values.join("; "), MAX_ARTIFACT_METADATA_VALUE_CHARS),
    );
}

fn parse_windows_wifi_profile_metadata(text: &str) -> BTreeMap<String, String> {
    let mut reader = Reader::from_str(text);
    reader.config_mut().trim_text(true);

    let mut metadata = BTreeMap::new();
    let mut element_stack: Vec<String> = Vec::new();
    let mut profile_names = Vec::new();
    let mut ssids = Vec::new();
    let mut auth_types = Vec::new();
    let mut encryption_types = Vec::new();
    let mut connection_modes = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) => {
                element_stack
                    .push(String::from_utf8_lossy(element.local_name().as_ref()).to_string());
            }
            Ok(Event::End(_)) => {
                element_stack.pop();
            }
            Ok(Event::Text(text_event)) => {
                let Ok(text) = text_event.unescape() else {
                    continue;
                };
                let text = text.trim();
                if text.is_empty() {
                    continue;
                }
                let current = element_stack.last().map(String::as_str).unwrap_or("");
                match current {
                    "name" if element_stack.iter().any(|element| element == "SSID") => {
                        push_unique_limited(&mut ssids, truncate_metadata_value(text, 120));
                    }
                    "name" if element_stack.iter().any(|element| element == "WLANProfile") => {
                        push_unique_limited(&mut profile_names, truncate_metadata_value(text, 120));
                    }
                    "authentication" => {
                        push_unique_limited(&mut auth_types, truncate_metadata_value(text, 120));
                    }
                    "encryption" => {
                        push_unique_limited(
                            &mut encryption_types,
                            truncate_metadata_value(text, 120),
                        );
                    }
                    "connectionMode" => {
                        push_unique_limited(
                            &mut connection_modes,
                            truncate_metadata_value(text, 120),
                        );
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    if profile_names.is_empty()
        && ssids.is_empty()
        && auth_types.is_empty()
        && encryption_types.is_empty()
        && connection_modes.is_empty()
    {
        return metadata;
    }

    metadata.insert(
        "system.networkConfigType".to_string(),
        "windows-wlan-profile".to_string(),
    );
    insert_joined_metadata(&mut metadata, "system.connectionIds", &profile_names);
    insert_joined_metadata(&mut metadata, "system.wifiSsids", &ssids);
    insert_joined_metadata(&mut metadata, "system.wifiAuthTypes", &auth_types);
    insert_joined_metadata(
        &mut metadata,
        "system.wifiEncryptionTypes",
        &encryption_types,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.networkConnectionModes",
        &connection_modes,
    );
    metadata
}

#[derive(Default)]
struct CommandHistoryMetadata {
    command_count: usize,
    command_names: Vec<String>,
    network_command_count: usize,
    privileged_command_count: usize,
    file_transfer_command_count: usize,
}

fn parse_command_history_metadata(source_id: &str, text: &str) -> BTreeMap<String, String> {
    let history_type = if source_id.ends_with("/consolehost_history.txt") {
        "powershell"
    } else if source_id.ends_with("/.zsh_history") {
        "zsh"
    } else {
        "bash"
    };
    let mut values = CommandHistoryMetadata::default();

    for line in text.lines() {
        let Some(command) = normalized_history_command(line, history_type) else {
            continue;
        };
        values.command_count = values.command_count.saturating_add(1);
        collect_command_history_summary(command, &mut values);
    }

    let mut metadata = BTreeMap::new();
    metadata.insert(
        "activity.commandHistoryType".to_string(),
        history_type.to_string(),
    );
    if values.command_count > 0 {
        metadata.insert(
            "activity.commandCount".to_string(),
            values.command_count.to_string(),
        );
    }
    if values.network_command_count > 0 {
        metadata.insert(
            "activity.networkCommandCount".to_string(),
            values.network_command_count.to_string(),
        );
    }
    if values.privileged_command_count > 0 {
        metadata.insert(
            "activity.privilegedCommandCount".to_string(),
            values.privileged_command_count.to_string(),
        );
    }
    if values.file_transfer_command_count > 0 {
        metadata.insert(
            "activity.fileTransferCommandCount".to_string(),
            values.file_transfer_command_count.to_string(),
        );
    }
    insert_joined_metadata(
        &mut metadata,
        "activity.commandNames",
        &values.command_names,
    );
    metadata
}

fn normalized_history_command<'a>(line: &'a str, history_type: &str) -> Option<&'a str> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    if history_type == "zsh" && line.starts_with(": ") {
        return line.split_once(';').map(|(_, command)| command.trim());
    }
    Some(line)
}

fn collect_command_history_summary(command: &str, values: &mut CommandHistoryMetadata) {
    let Some(command_name) = command_history_command_name(command) else {
        return;
    };
    let command_name_lower = command_name.to_ascii_lowercase();
    push_unique_limited(
        &mut values.command_names,
        truncate_metadata_value(&command_name, 80),
    );

    if matches!(command_name_lower.as_str(), "sudo" | "su" | "runas") {
        values.privileged_command_count = values.privileged_command_count.saturating_add(1);
    }
    if is_network_command(&command_name_lower) {
        values.network_command_count = values.network_command_count.saturating_add(1);
    }
    if is_file_transfer_command(&command_name_lower) {
        values.file_transfer_command_count = values.file_transfer_command_count.saturating_add(1);
    }
}

fn command_history_command_name(command: &str) -> Option<String> {
    let mut first = command
        .split_whitespace()
        .next()?
        .trim_matches(|c| matches!(c, '"' | '\'' | '&' | ';' | '(' | ')'));
    if first.is_empty() {
        return None;
    }
    if let Some(last) = first.rsplit(['/', '\\']).next() {
        first = last;
    }
    (!first.is_empty()).then(|| first.to_string())
}

fn is_network_command(command_name: &str) -> bool {
    matches!(
        command_name,
        "ssh"
            | "scp"
            | "sftp"
            | "curl"
            | "wget"
            | "nc"
            | "ncat"
            | "netcat"
            | "ftp"
            | "rsync"
            | "invoke-webrequest"
            | "iwr"
            | "invoke-restmethod"
            | "irm"
    )
}

fn is_file_transfer_command(command_name: &str) -> bool {
    matches!(
        command_name,
        "scp"
            | "sftp"
            | "curl"
            | "wget"
            | "ftp"
            | "rsync"
            | "invoke-webrequest"
            | "iwr"
            | "invoke-restmethod"
            | "irm"
    )
}

fn parse_firewall_metadata(source_id: &str, _data: &[u8], text: &str) -> BTreeMap<String, String> {
    if source_id.ends_with("/etc/sysconfig/iptables") || source_id.contains("/etc/iptables/") {
        return parse_iptables_metadata(text);
    }
    if source_id.ends_with("/windows/system32/logfiles/firewall/pfirewall.log") {
        return parse_windows_firewall_log_metadata(text);
    }
    BTreeMap::new()
}

fn parse_iptables_metadata(text: &str) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let mut tables = Vec::new();
    let mut chains = Vec::new();
    let mut policies = Vec::new();
    let mut rule_count = 0usize;

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(table) = line.strip_prefix('*') {
            push_unique_limited(&mut tables, table.to_string());
            continue;
        }
        if let Some(chain) = line.strip_prefix(':') {
            let mut parts = chain.split_whitespace();
            let Some(name) = parts.next() else {
                continue;
            };
            push_unique_limited(&mut chains, name.to_string());
            if let Some(policy) = parts.next().filter(|policy| *policy != "-") {
                push_unique_limited(&mut policies, format!("{name}:{policy}"));
            }
            continue;
        }
        if line.starts_with("-A ") || line.starts_with("-I ") {
            rule_count = rule_count.saturating_add(1);
        }
    }

    metadata.insert(
        "system.firewallConfigType".to_string(),
        "iptables".to_string(),
    );
    if rule_count > 0 {
        metadata.insert(
            "system.firewallRuleCount".to_string(),
            rule_count.to_string(),
        );
    }
    insert_joined_metadata(&mut metadata, "system.firewallTables", &tables);
    insert_joined_metadata(&mut metadata, "system.firewallChains", &chains);
    insert_joined_metadata(&mut metadata, "system.firewallPolicies", &policies);
    metadata
}

fn parse_windows_firewall_log_metadata(text: &str) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let mut entries = 0usize;
    let mut allowed = 0usize;
    let mut dropped = 0usize;
    let mut protocols = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 5 {
            continue;
        }
        entries = entries.saturating_add(1);
        match fields.get(2).copied().unwrap_or_default() {
            "ALLOW" => allowed = allowed.saturating_add(1),
            "DROP" => dropped = dropped.saturating_add(1),
            _ => {}
        }
        if let Some(protocol) = fields.get(3) {
            push_unique_limited(&mut protocols, (*protocol).to_string());
        }
    }

    metadata.insert(
        "system.firewallConfigType".to_string(),
        "windows-firewall-log".to_string(),
    );
    if entries > 0 {
        metadata.insert(
            "system.firewallLogEntryCount".to_string(),
            entries.to_string(),
        );
    }
    if allowed > 0 {
        metadata.insert(
            "system.firewallAllowedCount".to_string(),
            allowed.to_string(),
        );
    }
    if dropped > 0 {
        metadata.insert(
            "system.firewallDroppedCount".to_string(),
            dropped.to_string(),
        );
    }
    insert_joined_metadata(&mut metadata, "system.firewallProtocols", &protocols);
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

fn parse_macos_hardware_identity_metadata(data: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = parse_macos_plist_identity_metadata(data);
    let Ok(value) = plist::from_bytes::<plist::Value>(data) else {
        return metadata;
    };

    insert_plist_string(
        &mut metadata,
        &value,
        "IOPlatformSerialNumber",
        "system.serialNumber",
    );
    insert_plist_string(
        &mut metadata,
        &value,
        "serial_number",
        "system.serialNumber",
    );
    insert_plist_string(
        &mut metadata,
        &value,
        "machine_model",
        "system.modelIdentifier",
    );
    insert_plist_string(&mut metadata, &value, "machine_name", "system.model");
    insert_plist_string(
        &mut metadata,
        &value,
        "platform_UUID",
        "system.hardwareUuid",
    );
    insert_plist_string(
        &mut metadata,
        &value,
        "boot_rom_version",
        "system.bootRomVersion",
    );
    insert_plist_string(
        &mut metadata,
        &value,
        "smc_version_system",
        "system.smcVersion",
    );
    insert_plist_string(&mut metadata, &value, "cpu_type", "system.cpuType");
    insert_plist_string(
        &mut metadata,
        &value,
        "current_processor_speed",
        "system.cpuSpeed",
    );

    metadata
}

#[derive(Default)]
struct MacosKernelExtensionMetadata {
    identifiers: Vec<String>,
    names: Vec<String>,
    versions: Vec<String>,
    personality_names: Vec<String>,
    classes: Vec<String>,
    provider_classes: Vec<String>,
    matches: Vec<String>,
    libraries: Vec<String>,
}

fn parse_macos_kernel_extension_metadata(data: &[u8]) -> BTreeMap<String, String> {
    let Ok(value) = plist::from_bytes::<plist::Value>(data) else {
        return BTreeMap::new();
    };
    let plist::Value::Dictionary(dict) = value else {
        return BTreeMap::new();
    };

    let mut values = MacosKernelExtensionMetadata::default();
    collect_macos_kernel_extension_bundle_metadata(&dict, &mut values);
    if let Some(plist::Value::Dictionary(personalities)) = dict.get("IOKitPersonalities") {
        collect_macos_kernel_extension_personalities(personalities, &mut values);
    }
    if let Some(plist::Value::Dictionary(libraries)) = dict.get("OSBundleLibraries") {
        collect_macos_kernel_extension_libraries(libraries, &mut values);
    }

    macos_kernel_extension_metadata_to_map(values)
}

fn collect_macos_kernel_extension_bundle_metadata(
    dict: &plist::Dictionary,
    values: &mut MacosKernelExtensionMetadata,
) {
    for key in ["CFBundleIdentifier", "OSBundleIdentifier"] {
        if let Some(value) = plist_dict_scalar_string(dict, key) {
            push_unique_limited(
                &mut values.identifiers,
                truncate_metadata_value(&value, 180),
            );
        }
    }
    for key in ["CFBundleName", "CFBundleDisplayName", "IOClass"] {
        if let Some(value) = plist_dict_scalar_string(dict, key) {
            push_unique_limited(&mut values.names, truncate_metadata_value(&value, 120));
        }
    }
    for key in ["CFBundleVersion", "CFBundleShortVersionString"] {
        if let Some(value) = plist_dict_scalar_string(dict, key) {
            push_unique_limited(&mut values.versions, truncate_metadata_value(&value, 80));
        }
    }
}

fn collect_macos_kernel_extension_personalities(
    personalities: &plist::Dictionary,
    values: &mut MacosKernelExtensionMetadata,
) {
    for (personality_name, personality) in personalities.iter().take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
    {
        let plist::Value::Dictionary(personality) = personality else {
            continue;
        };
        let personality_name = truncate_metadata_value(personality_name, 120);
        push_unique_limited(&mut values.personality_names, personality_name.clone());

        let class = first_plist_dict_scalar_string(personality, &["IOClass", "CFBundleIdentifier"]);
        let provider = first_plist_dict_scalar_string(personality, &["IOProviderClass"]);
        let match_value = first_plist_dict_scalar_string(
            personality,
            &["IONameMatch", "IOProviderMergeProperties"],
        );
        if let Some(class) = &class {
            push_unique_limited(&mut values.classes, class.clone());
        }
        if let Some(provider) = &provider {
            push_unique_limited(&mut values.provider_classes, provider.clone());
        }
        if let Some(match_value) = &match_value {
            push_unique_limited(&mut values.matches, match_value.clone());
        }

        let mut parts = Vec::new();
        if let Some(class) = class {
            parts.push(format!("class={class}"));
        }
        if let Some(provider) = provider {
            parts.push(format!("provider={provider}"));
        }
        if let Some(match_value) = match_value {
            parts.push(format!("match={match_value}"));
        }
        if !parts.is_empty() {
            push_unique_limited(
                &mut values.names,
                truncate_metadata_value(
                    &format!("{personality_name} ({})", parts.join("; ")),
                    MAX_SYSTEM_MOUNT_DESCRIPTION_CHARS,
                ),
            );
        }
    }
}

fn collect_macos_kernel_extension_libraries(
    libraries: &plist::Dictionary,
    values: &mut MacosKernelExtensionMetadata,
) {
    for (library, version) in libraries.iter().take(MAX_SYSTEM_IDENTITY_LIST_ITEMS) {
        let version = match version {
            plist::Value::String(value) => Some(value.as_str()),
            _ => None,
        };
        let description = version.map_or_else(
            || library.to_string(),
            |version| format!("{library}>={version}"),
        );
        push_unique_limited(
            &mut values.libraries,
            truncate_metadata_value(&description, 180),
        );
    }
}

fn macos_kernel_extension_metadata_to_map(
    values: MacosKernelExtensionMetadata,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if values.identifiers.is_empty()
        && values.personality_names.is_empty()
        && values.libraries.is_empty()
    {
        return metadata;
    }

    metadata.insert(
        "system.kernelExtensionConfigType".to_string(),
        "macos-kext".to_string(),
    );
    insert_joined_metadata(
        &mut metadata,
        "system.kernelExtensionIdentifiers",
        &values.identifiers,
    );
    insert_joined_metadata(&mut metadata, "system.kernelExtensionNames", &values.names);
    insert_joined_metadata(
        &mut metadata,
        "system.kernelExtensionVersions",
        &values.versions,
    );
    if !values.personality_names.is_empty() {
        metadata.insert(
            "system.kernelExtensionPersonalityCount".to_string(),
            values.personality_names.len().to_string(),
        );
    }
    insert_joined_metadata(
        &mut metadata,
        "system.kernelExtensionPersonalities",
        &values.personality_names,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.kernelExtensionClasses",
        &values.classes,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.kernelExtensionProviderClasses",
        &values.provider_classes,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.kernelExtensionMatches",
        &values.matches,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.kernelExtensionLibraries",
        &values.libraries,
    );
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

#[derive(Default)]
struct MacosWifiMetadata {
    ssids: Vec<String>,
    security_types: Vec<String>,
    auto_join_ssids: Vec<String>,
    last_connected: Vec<String>,
}

fn parse_macos_wifi_metadata(data: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let Ok(value) = plist::from_bytes::<plist::Value>(data) else {
        return metadata;
    };
    let mut values = MacosWifiMetadata::default();

    collect_macos_wifi_metadata(&value, &mut values);

    if !values.ssids.is_empty() {
        metadata.insert(
            "system.wifiKnownNetworkCount".to_string(),
            values.ssids.len().to_string(),
        );
    }
    insert_joined_metadata(&mut metadata, "system.wifiSsids", &values.ssids);
    insert_joined_metadata(
        &mut metadata,
        "system.wifiSecurityTypes",
        &values.security_types,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.wifiAutoJoinSsids",
        &values.auto_join_ssids,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.wifiLastConnected",
        &values.last_connected,
    );
    metadata
}

fn collect_macos_wifi_metadata(value: &plist::Value, values: &mut MacosWifiMetadata) {
    match value {
        plist::Value::Dictionary(dict) => {
            collect_macos_wifi_network_dict(dict, values);
            for child in dict.values() {
                collect_macos_wifi_metadata(child, values);
            }
        }
        plist::Value::Array(items) => {
            for child in items {
                collect_macos_wifi_metadata(child, values);
            }
        }
        _ => {}
    }
}

fn collect_macos_wifi_network_dict(dict: &plist::Dictionary, values: &mut MacosWifiMetadata) {
    let Some(ssid) = macos_wifi_ssid(dict) else {
        return;
    };
    push_unique_limited(&mut values.ssids, ssid.clone());

    for key in [
        "SecurityType",
        "Security",
        "AuthType",
        "EncryptionType",
        "SupportedSecurityTypes",
    ] {
        if let Some(value) = plist_dict_string(dict, key) {
            push_unique_limited(&mut values.security_types, value.to_string());
        } else if let Some(list) = plist_dict_string_array(dict, key) {
            for value in list {
                push_unique_limited(&mut values.security_types, value);
            }
        }
    }

    if plist_dict_bool(dict, "AutoJoin").or_else(|| plist_dict_bool(dict, "AutoLogin"))
        == Some(true)
    {
        push_unique_limited(&mut values.auto_join_ssids, ssid.clone());
    }

    for key in ["LastConnected", "LastAutoJoined", "LastJoined"] {
        if let Some(date) = plist_dict_date(dict, key) {
            push_unique_limited(&mut values.last_connected, format!("{ssid}={date}"));
            break;
        }
    }
}

fn macos_wifi_ssid(dict: &plist::Dictionary) -> Option<String> {
    for key in ["SSIDString", "SSID_STR", "SSID", "name"] {
        if let Some(value) = plist_dict_string(dict, key) {
            let value = value.trim();
            if !value.is_empty() {
                return Some(truncate_metadata_value(value, 120));
            }
        }
    }
    for key in ["SSID", "SSIDData"] {
        if let Some(value) = dict.get(key).and_then(plist_data_utf8_string) {
            return Some(truncate_metadata_value(&value, 120));
        }
    }
    None
}

fn plist_dict_bool(dict: &plist::Dictionary, key: &str) -> Option<bool> {
    match dict.get(key)? {
        plist::Value::Boolean(value) => Some(*value),
        _ => None,
    }
}

fn plist_data_utf8_string(value: &plist::Value) -> Option<String> {
    let plist::Value::Data(data) = value else {
        return None;
    };
    let value = std::str::from_utf8(data).ok()?.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn parse_macos_firewall_metadata(data: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let Ok(value) = plist::from_bytes::<plist::Value>(data) else {
        return metadata;
    };
    let plist::Value::Dictionary(dict) = value else {
        return metadata;
    };

    metadata.insert(
        "system.firewallConfigType".to_string(),
        "macos-alf".to_string(),
    );
    insert_plist_integer_metadata(
        &mut metadata,
        &dict,
        "globalstate",
        "system.firewallGlobalState",
    );
    insert_plist_bool_metadata(
        &mut metadata,
        &dict,
        "stealthenabled",
        "system.firewallStealthEnabled",
    );
    insert_plist_bool_metadata(
        &mut metadata,
        &dict,
        "allowsignedenabled",
        "system.firewallAllowSignedEnabled",
    );
    insert_plist_bool_metadata(
        &mut metadata,
        &dict,
        "loggingenabled",
        "system.firewallLoggingEnabled",
    );
    if let Some(apps) = plist_dict_array_len(&dict, "applications") {
        metadata.insert(
            "system.firewallApplicationRuleCount".to_string(),
            apps.to_string(),
        );
    }
    metadata
}

fn insert_plist_bool_metadata(
    metadata: &mut BTreeMap<String, String>,
    dict: &plist::Dictionary,
    plist_key: &str,
    metadata_key: &str,
) {
    if let Some(value) = plist_dict_bool(dict, plist_key) {
        metadata.insert(metadata_key.to_string(), value.to_string());
    }
}

fn insert_plist_integer_metadata(
    metadata: &mut BTreeMap<String, String>,
    dict: &plist::Dictionary,
    plist_key: &str,
    metadata_key: &str,
) {
    if let Some(value) = plist_dict_integer(dict, plist_key) {
        metadata.insert(metadata_key.to_string(), value.to_string());
    }
}

fn plist_dict_integer(dict: &plist::Dictionary, key: &str) -> Option<i64> {
    let plist::Value::Integer(value) = dict.get(key)? else {
        return None;
    };
    value.as_signed()
}

fn plist_dict_array_len(dict: &plist::Dictionary, key: &str) -> Option<usize> {
    let plist::Value::Array(values) = dict.get(key)? else {
        return None;
    };
    Some(values.len())
}

fn parse_macos_global_preferences_metadata(data: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let Ok(value) = plist::from_bytes::<plist::Value>(data) else {
        return metadata;
    };

    insert_plist_string(&mut metadata, &value, "AppleLocale", "system.locale");
    insert_plist_string(&mut metadata, &value, "AppleCountry", "system.country");
    insert_plist_string(
        &mut metadata,
        &value,
        "AppleMeasurementUnits",
        "system.measurementUnits",
    );
    insert_plist_string(
        &mut metadata,
        &value,
        "AppleTemperatureUnit",
        "system.temperatureUnit",
    );
    if let Some(languages) = find_plist_string_array(&value, "AppleLanguages") {
        metadata.insert(
            "system.languages".to_string(),
            truncate_metadata_value(&languages.join(", "), MAX_ARTIFACT_METADATA_VALUE_CHARS),
        );
    }

    metadata
}

fn parse_macos_install_history_metadata(data: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let Ok(value) = plist::from_bytes::<plist::Value>(data) else {
        return metadata;
    };
    let plist::Value::Array(entries) = value else {
        return metadata;
    };

    let mut install_count = 0usize;
    let mut latest_install = None;
    for entry in entries.iter().take(MAX_SYSTEM_IDENTITY_LIST_ITEMS) {
        let plist::Value::Dictionary(dict) = entry else {
            continue;
        };
        install_count = install_count.saturating_add(1);
        latest_install = Some(dict);
    }

    if install_count > 0 {
        metadata.insert(
            "system.installHistoryCount".to_string(),
            install_count.to_string(),
        );
    }
    let Some(latest_install) = latest_install else {
        return metadata;
    };
    if let Some(value) = plist_dict_string(latest_install, "displayName") {
        metadata.insert(
            "system.latestInstallName".to_string(),
            truncate_metadata_value(value, MAX_ARTIFACT_METADATA_VALUE_CHARS),
        );
    }
    if let Some(value) = plist_dict_string(latest_install, "displayVersion") {
        metadata.insert(
            "system.latestInstallVersion".to_string(),
            truncate_metadata_value(value, MAX_ARTIFACT_METADATA_VALUE_CHARS),
        );
    }
    if let Some(value) = plist_dict_date(latest_install, "date") {
        metadata.insert("system.latestInstallDate".to_string(), value);
    }
    if let Some(values) = plist_dict_string_array(latest_install, "packageIdentifiers") {
        metadata.insert(
            "system.latestInstallPackages".to_string(),
            truncate_metadata_value(&values.join(", "), MAX_ARTIFACT_METADATA_VALUE_CHARS),
        );
    }

    metadata
}

#[derive(Default)]
struct MacosDiskManagementMetadata {
    volume_names: Vec<String>,
    volume_uuids: Vec<String>,
    disk_identifiers: Vec<String>,
    filesystems: Vec<String>,
    mount_points: Vec<String>,
    descriptions: Vec<String>,
    total_size_bytes: u64,
}

fn parse_macos_disk_management_metadata(data: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let Ok(value) = plist::from_bytes::<plist::Value>(data) else {
        return metadata;
    };
    let mut values = MacosDiskManagementMetadata::default();

    collect_macos_disk_management_metadata(&value, &mut values);

    if !values.descriptions.is_empty() {
        metadata.insert(
            "system.volumeCount".to_string(),
            values.descriptions.len().to_string(),
        );
    }
    if values.total_size_bytes > 0 {
        metadata.insert(
            "system.totalVolumeBytes".to_string(),
            values.total_size_bytes.to_string(),
        );
    }
    insert_joined_metadata(&mut metadata, "system.volumeNames", &values.volume_names);
    insert_joined_metadata(&mut metadata, "system.volumeUuids", &values.volume_uuids);
    insert_joined_metadata(
        &mut metadata,
        "system.diskIdentifiers",
        &values.disk_identifiers,
    );
    insert_joined_metadata(
        &mut metadata,
        "system.volumeFilesystems",
        &values.filesystems,
    );
    insert_joined_metadata(&mut metadata, "system.volumeMounts", &values.mount_points);
    insert_joined_metadata(&mut metadata, "system.volumes", &values.descriptions);

    metadata
}

fn collect_macos_disk_management_metadata(
    value: &plist::Value,
    values: &mut MacosDiskManagementMetadata,
) {
    match value {
        plist::Value::Dictionary(dict) => {
            collect_macos_disk_management_dict(dict, values);
            for child in dict.values() {
                collect_macos_disk_management_metadata(child, values);
            }
        }
        plist::Value::Array(items) => {
            for child in items {
                collect_macos_disk_management_metadata(child, values);
            }
        }
        _ => {}
    }
}

fn collect_macos_disk_management_dict(
    dict: &plist::Dictionary,
    values: &mut MacosDiskManagementMetadata,
) {
    let bsd_name = first_plist_dict_scalar_string(
        dict,
        &["BSD Name", "BSDName", "DeviceIdentifier", "DAMediaBSDName"],
    );
    let uuid = first_plist_dict_scalar_string(dict, &["VolumeUUID", "DAVolumeUUID"]);
    let filesystem = first_plist_dict_scalar_string(
        dict,
        &[
            "FilesystemName",
            "FilesystemType",
            "DAVolumeKind",
            "Content",
            "DAMediaContent",
        ],
    );
    let mount_point = first_plist_dict_scalar_string(dict, &["MountPoint", "DAVolumePath", "Path"]);
    let size = first_plist_dict_scalar_string(dict, &["Size", "VolumeSize", "DAMediaSize"]);

    if bsd_name.is_none()
        && uuid.is_none()
        && filesystem.is_none()
        && mount_point.is_none()
        && size.is_none()
    {
        return;
    }

    let name = first_plist_dict_scalar_string(dict, &["VolumeName", "DAVolumeName", "Name"]);
    if let Some(name) = &name {
        push_unique_limited(&mut values.volume_names, name.clone());
    }
    if let Some(uuid) = &uuid {
        push_unique_limited(&mut values.volume_uuids, uuid.clone());
    }
    if let Some(bsd_name) = &bsd_name {
        push_unique_limited(&mut values.disk_identifiers, bsd_name.clone());
    }
    if let Some(filesystem) = &filesystem {
        push_unique_limited(&mut values.filesystems, filesystem.clone());
    }
    if let Some(mount_point) = &mount_point {
        push_unique_limited(&mut values.mount_points, mount_point.clone());
    }
    if let Some(size) = &size {
        if let Ok(size) = size.parse::<u64>() {
            values.total_size_bytes = values.total_size_bytes.saturating_add(size);
        }
    }

    if let Some(description) =
        macos_disk_description(name, bsd_name, uuid, filesystem, mount_point, size)
    {
        push_unique_limited(&mut values.descriptions, description);
    }
}

fn macos_disk_description(
    name: Option<String>,
    bsd_name: Option<String>,
    uuid: Option<String>,
    filesystem: Option<String>,
    mount_point: Option<String>,
    size: Option<String>,
) -> Option<String> {
    let label = name.or_else(|| bsd_name.clone()).or_else(|| uuid.clone())?;
    let mut parts = Vec::new();
    if let Some(bsd_name) = bsd_name {
        if bsd_name != label {
            parts.push(bsd_name);
        }
    }
    if let Some(filesystem) = filesystem {
        parts.push(filesystem);
    }
    if let Some(mount_point) = mount_point {
        parts.push(format!("mounted={mount_point}"));
    }
    if let Some(uuid) = uuid {
        if uuid != label {
            parts.push(format!("uuid={uuid}"));
        }
    }
    if let Some(size) = size {
        parts.push(format!("size={size}"));
    }

    let value = if parts.is_empty() {
        label
    } else {
        format!("{label} ({})", parts.join(", "))
    };
    Some(truncate_metadata_value(
        &value,
        MAX_SYSTEM_MOUNT_DESCRIPTION_CHARS,
    ))
}

fn first_plist_dict_scalar_string(dict: &plist::Dictionary, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| plist_dict_scalar_string(dict, key))
        .map(|value| truncate_metadata_value(&value, 120))
}

fn plist_dict_scalar_string(dict: &plist::Dictionary, key: &str) -> Option<String> {
    match dict.get(key)? {
        plist::Value::String(value) => {
            let value = value.trim();
            (!value.is_empty()).then(|| value.to_string())
        }
        plist::Value::Integer(value) => value.as_signed().map(|value| value.to_string()),
        plist::Value::Boolean(value) => Some(value.to_string()),
        _ => None,
    }
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

fn find_plist_string_array(value: &plist::Value, key: &str) -> Option<Vec<String>> {
    match value {
        plist::Value::Dictionary(dict) => {
            if let Some(values) = plist_dict_string_array(dict, key) {
                return Some(values);
            }
            dict.values()
                .find_map(|value| find_plist_string_array(value, key))
        }
        plist::Value::Array(values) => values
            .iter()
            .find_map(|value| find_plist_string_array(value, key)),
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

fn plist_dict_string_array(dict: &plist::Dictionary, key: &str) -> Option<Vec<String>> {
    let plist::Value::Array(values) = dict.get(key)? else {
        return None;
    };
    let strings: Vec<String> = values
        .iter()
        .filter_map(plist::Value::as_string)
        .map(ToString::to_string)
        .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
        .collect();
    (!strings.is_empty()).then_some(strings)
}

fn plist_dict_first_string(dict: &plist::Dictionary, key: &str) -> Option<String> {
    plist_dict_string_array(dict, key)
        .and_then(|values| values.into_iter().next())
        .or_else(|| plist_dict_string(dict, key).map(ToString::to_string))
        .map(|value| truncate_metadata_value(&value, 180))
}

fn macos_uid_is_regular(uid: &str) -> bool {
    uid.parse::<u32>()
        .map(|uid| (MACOS_REGULAR_USER_MIN_UID..=UNIX_REGULAR_USER_MAX_UID).contains(&uid))
        .unwrap_or(false)
}

fn describe_macos_local_user(account_name: &str, real_name: Option<String>) -> String {
    let account_name = truncate_metadata_value(account_name, 120);
    match real_name.filter(|value| value != &account_name) {
        Some(real_name) => format!(
            "{} ({})",
            account_name,
            truncate_metadata_value(&real_name, 120)
        ),
        None => account_name,
    }
}

fn plist_dict_date(dict: &plist::Dictionary, key: &str) -> Option<String> {
    let plist::Value::Date(value) = dict.get(key)? else {
        return None;
    };
    Some(value.to_xml_format())
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

fn insert_optional_metadata(
    metadata: &mut BTreeMap<String, String>,
    key: &str,
    value: Option<&str>,
) {
    if let Some(value) = value {
        insert_trimmed_metadata(metadata, key, value);
    }
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

async fn enrich_image_artifact_metadata(
    source: &HashSourceInput,
    artifact: &mut NormalizedArtifact,
) -> Result<(), String> {
    if !is_image_artifact(artifact) {
        return Ok(());
    }

    let source = source.clone();
    let metadata_result =
        tauri::async_runtime::spawn_blocking(move || image_artifact_metadata_from_source(&source))
            .await
            .map_err(|e| format!("Image artifact metadata task failed: {e}"))?;

    match metadata_result {
        Ok(metadata) => {
            artifact.metadata.extend(metadata);
        }
        Err(error) => {
            artifact.metadata.insert(
                "image.analysisStatus".to_string(),
                "unavailable".to_string(),
            );
            artifact.metadata.insert(
                "image.analysisError".to_string(),
                truncate_metadata_value(&error, 180),
            );
        }
    }

    Ok(())
}

fn is_image_artifact(artifact: &NormalizedArtifact) -> bool {
    artifact.category == "image"
        || artifact
            .mime_type
            .as_deref()
            .is_some_and(|mime| mime.starts_with("image/"))
        || matches!(
            artifact.extension.as_deref(),
            Some(
                "jpg"
                    | "jpeg"
                    | "png"
                    | "gif"
                    | "webp"
                    | "bmp"
                    | "tif"
                    | "tiff"
                    | "ico"
                    | "heic"
                    | "heif"
                    | "avif"
                    | "raw"
                    | "cr2"
                    | "nef"
                    | "arw"
                    | "dng"
                    | "orf"
                    | "rw2"
            )
        )
}

fn image_artifact_metadata_from_source(
    source: &HashSourceInput,
) -> Result<BTreeMap<String, String>, String> {
    let byte_source = open_hash_source(source)?;
    let source_id = byte_source.source_ref().display_id();
    let size = byte_source.len().map_err(|e| e.to_string())?;
    if size > IMAGE_ARTIFACT_SOURCE_MAX_BYTES {
        return Err(format!(
            "Image artifact source is too large for metadata extraction: {size} bytes > {IMAGE_ARTIFACT_SOURCE_MAX_BYTES} bytes"
        ));
    }

    let read_len = usize::try_from(size)
        .map_err(|_| format!("Image artifact source size does not fit this platform: {size}"))?;
    let data = if read_len > 0 {
        read_range_fully(byte_source.as_ref(), 0, read_len).map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };

    let mut metadata = BTreeMap::new();
    match image_dimensions_from_bytes(&data) {
        Ok((width, height)) => {
            metadata.insert("image.analysisStatus".to_string(), "parsed".to_string());
            metadata.insert("image.width".to_string(), width.to_string());
            metadata.insert("image.height".to_string(), height.to_string());
            metadata.insert(
                "image.pixelCount".to_string(),
                u64::from(width)
                    .saturating_mul(u64::from(height))
                    .to_string(),
            );
        }
        Err(error) => {
            metadata.insert(
                "image.dimensionStatus".to_string(),
                "unavailable".to_string(),
            );
            metadata.insert(
                "image.dimensionError".to_string(),
                truncate_metadata_value(&error, 180),
            );
        }
    }

    if ensure_exif_size_allowed(size).is_ok() {
        match extract_exif_from_reader(source_id, Cursor::new(data.as_slice())) {
            Ok(exif) => {
                metadata.extend(image_artifact_metadata_from_exif(&exif));
            }
            Err(error) => {
                metadata.insert("exif.analysisStatus".to_string(), "unavailable".to_string());
                metadata.insert(
                    "exif.analysisError".to_string(),
                    truncate_metadata_value(&error.to_string(), 180),
                );
            }
        }
    }

    Ok(metadata)
}

fn image_dimensions_from_bytes(data: &[u8]) -> Result<(u32, u32), String> {
    image::ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .into_dimensions()
        .map_err(|e| e.to_string())
}

fn image_artifact_metadata_from_exif(info: &ExifMetadata) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    metadata.insert("exif.analysisStatus".to_string(), "parsed".to_string());
    insert_optional_metadata(&mut metadata, "exif.make", info.make.as_deref());
    insert_optional_metadata(&mut metadata, "exif.model", info.model.as_deref());
    insert_optional_metadata(&mut metadata, "exif.software", info.software.as_deref());
    insert_optional_metadata(&mut metadata, "exif.lensModel", info.lens_model.as_deref());
    insert_optional_metadata(
        &mut metadata,
        "exif.exposureTime",
        info.exposure_time.as_deref(),
    );
    insert_optional_metadata(&mut metadata, "exif.fNumber", info.f_number.as_deref());
    insert_optional_metadata(
        &mut metadata,
        "exif.focalLength",
        info.focal_length.as_deref(),
    );
    insert_optional_metadata(&mut metadata, "exif.flash", info.flash.as_deref());
    insert_optional_metadata(
        &mut metadata,
        "exif.dateTimeOriginal",
        info.date_time_original.as_deref(),
    );
    insert_optional_metadata(
        &mut metadata,
        "exif.dateTimeDigitized",
        info.date_time_digitized.as_deref(),
    );
    insert_optional_metadata(&mut metadata, "exif.dateTime", info.date_time.as_deref());
    insert_optional_metadata(
        &mut metadata,
        "exif.gpsTimestamp",
        info.gps_timestamp.as_deref(),
    );
    insert_optional_metadata(
        &mut metadata,
        "exif.colorSpace",
        info.color_space.as_deref(),
    );
    insert_optional_metadata(
        &mut metadata,
        "exif.imageUniqueId",
        info.image_unique_id.as_deref(),
    );
    insert_optional_metadata(&mut metadata, "exif.ownerName", info.owner_name.as_deref());
    insert_optional_metadata(
        &mut metadata,
        "exif.serialNumber",
        info.serial_number.as_deref(),
    );
    if let Some(iso) = info.iso {
        metadata.insert("exif.iso".to_string(), iso.to_string());
    }
    if let Some(width) = info.width {
        metadata.insert("exif.width".to_string(), width.to_string());
    }
    if let Some(height) = info.height {
        metadata.insert("exif.height".to_string(), height.to_string());
    }
    if let Some(orientation) = info.orientation {
        metadata.insert("exif.orientation".to_string(), orientation.to_string());
    }
    if let Some(gps) = &info.gps {
        metadata.insert("exif.gpsLatitude".to_string(), gps.latitude.to_string());
        metadata.insert("exif.gpsLongitude".to_string(), gps.longitude.to_string());
        if let Some(altitude) = gps.altitude {
            metadata.insert("exif.gpsAltitude".to_string(), altitude.to_string());
        }
    }
    metadata.insert(
        "exif.rawTagCount".to_string(),
        info.raw_tags.len().to_string(),
    );
    if !info.raw_tags.is_empty() {
        let raw_tags: Vec<String> = info
            .raw_tags
            .iter()
            .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
            .map(|(key, value)| format!("{key}={value}"))
            .collect();
        insert_joined_metadata(&mut metadata, "exif.rawTags", &raw_tags);
    }

    metadata
}

async fn enrich_binary_artifact_metadata(
    source: &HashSourceInput,
    artifact: &mut NormalizedArtifact,
) -> Result<(), String> {
    if !is_binary_artifact(artifact) {
        return Ok(());
    }

    let source = source.clone();
    let metadata_result =
        tauri::async_runtime::spawn_blocking(move || binary_artifact_metadata_from_source(&source))
            .await
            .map_err(|e| format!("Binary artifact metadata task failed: {e}"))?;

    match metadata_result {
        Ok(metadata) => {
            let is_driver = metadata
                .get("pe.isDriver")
                .is_some_and(|value| value == "true");
            if is_driver {
                artifact.category = "system".to_string();
                artifact.type_description = metadata
                    .get("pe.driverType")
                    .map(|driver_type| format!("Windows {driver_type}"))
                    .unwrap_or_else(|| "Windows Driver Artifact".to_string());
            }
            artifact.metadata.extend(metadata);
        }
        Err(error) => {
            artifact.metadata.insert(
                "binary.analysisStatus".to_string(),
                "unavailable".to_string(),
            );
            artifact.metadata.insert(
                "binary.analysisError".to_string(),
                truncate_metadata_value(&error, 180),
            );
        }
    }

    Ok(())
}

fn is_binary_artifact(artifact: &NormalizedArtifact) -> bool {
    artifact.category == "executable"
        || artifact
            .mime_type
            .as_deref()
            .is_some_and(|mime| mime.contains("executable") || mime.contains("mach-binary"))
        || matches!(
            artifact.extension.as_deref(),
            Some(
                "sys"
                    | "drv"
                    | "ko"
                    | "exe"
                    | "dll"
                    | "ocx"
                    | "efi"
                    | "elf"
                    | "so"
                    | "dylib"
                    | "kext",
            )
        )
}

fn binary_artifact_metadata_from_source(
    source: &HashSourceInput,
) -> Result<BTreeMap<String, String>, String> {
    let byte_source = open_hash_source(source)?;
    let source_id = byte_source.source_ref().display_id();
    let size = byte_source.len().map_err(|e| e.to_string())?;
    if size > BINARY_ARTIFACT_SOURCE_MAX_BYTES {
        return Err(format!(
            "Binary artifact source is too large for analysis: {size} bytes > {BINARY_ARTIFACT_SOURCE_MAX_BYTES} bytes"
        ));
    }

    let read_len = usize::try_from(size)
        .map_err(|_| format!("Binary artifact source size does not fit this platform: {size}"))?;
    let data = if read_len > 0 {
        read_range_fully(byte_source.as_ref(), 0, read_len).map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };
    let info = analyze_binary_bytes(source_id, &data).map_err(|e| e.to_string())?;
    Ok(binary_artifact_metadata_from_info(&info))
}

fn binary_artifact_metadata_from_info(info: &BinaryInfo) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    metadata.insert("binary.analysisStatus".to_string(), "parsed".to_string());
    metadata.insert(
        "binary.format".to_string(),
        binary_format_name(&info.format).to_string(),
    );
    metadata.insert("binary.architecture".to_string(), info.architecture.clone());
    metadata.insert("binary.is64Bit".to_string(), info.is_64bit.to_string());
    metadata.insert("binary.fileSize".to_string(), info.file_size.to_string());
    metadata.insert(
        "binary.importLibraryCount".to_string(),
        info.imports.len().to_string(),
    );
    metadata.insert(
        "binary.exportCount".to_string(),
        info.exports.len().to_string(),
    );
    metadata.insert(
        "binary.sectionCount".to_string(),
        info.sections.len().to_string(),
    );
    metadata.insert(
        "binary.stringCount".to_string(),
        info.strings.len().to_string(),
    );
    if let Some(entry_point) = info.entry_point {
        metadata.insert(
            "binary.entryPoint".to_string(),
            format!("0x{entry_point:x}"),
        );
    }
    if !info.imports.is_empty() {
        let libraries: Vec<String> = info
            .imports
            .iter()
            .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
            .map(|import| import.library.clone())
            .collect();
        insert_joined_metadata(&mut metadata, "binary.importLibraries", &libraries);
    }
    if !info.exports.is_empty() {
        let exports: Vec<String> = info
            .exports
            .iter()
            .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
            .map(|export| export.name.clone())
            .collect();
        insert_joined_metadata(&mut metadata, "binary.exports", &exports);
    }
    if !info.sections.is_empty() {
        let sections: Vec<String> = info
            .sections
            .iter()
            .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
            .map(|section| section.name.clone())
            .collect();
        insert_joined_metadata(&mut metadata, "binary.sections", &sections);

        let section_characteristics: Vec<String> = info
            .sections
            .iter()
            .filter(|section| !section.characteristics_detail.is_empty())
            .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
            .map(|section| {
                format!(
                    "{}={}",
                    section.name,
                    section.characteristics_detail.join(",")
                )
            })
            .collect();
        insert_joined_metadata(
            &mut metadata,
            "binary.sectionCharacteristics",
            &section_characteristics,
        );

        let executable_sections: Vec<String> = info
            .sections
            .iter()
            .filter(|section| {
                section
                    .characteristics_detail
                    .iter()
                    .any(|value| value == "executable" || value == "contains-code")
            })
            .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
            .map(|section| section.name.clone())
            .collect();
        insert_joined_metadata(
            &mut metadata,
            "binary.executableSections",
            &executable_sections,
        );

        let writable_sections: Vec<String> = info
            .sections
            .iter()
            .filter(|section| {
                section
                    .characteristics_detail
                    .iter()
                    .any(|value| value == "writable")
            })
            .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
            .map(|section| section.name.clone())
            .collect();
        insert_joined_metadata(&mut metadata, "binary.writableSections", &writable_sections);

        if let Some(max_entropy) = info
            .sections
            .iter()
            .filter_map(|section| section.entropy)
            .max_by(f64::total_cmp)
        {
            metadata.insert(
                "binary.maxSectionEntropy".to_string(),
                format!("{max_entropy:.3}"),
            );
        }

        let high_entropy_sections: Vec<String> = info
            .sections
            .iter()
            .filter_map(|section| {
                let entropy = section.entropy?;
                (entropy >= 7.0).then(|| format!("{}={entropy:.3}", section.name))
            })
            .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
            .collect();
        insert_joined_metadata(
            &mut metadata,
            "binary.highEntropySections",
            &high_entropy_sections,
        );
    }
    if !info.strings.is_empty() {
        let strings: Vec<String> = info
            .strings
            .iter()
            .take(MAX_SYSTEM_IDENTITY_LIST_ITEMS)
            .cloned()
            .collect();
        insert_joined_metadata(&mut metadata, "binary.strings", &strings);
    }

    metadata.insert(
        "binary.hasDebugInfo".to_string(),
        info.has_debug_info.to_string(),
    );
    metadata.insert(
        "binary.isStripped".to_string(),
        info.is_stripped.to_string(),
    );
    metadata.insert(
        "binary.hasCodeSigning".to_string(),
        info.has_code_signing.to_string(),
    );

    if let Some(timestamp) = info.pe_timestamp {
        metadata.insert("pe.timestamp".to_string(), timestamp.to_string());
    }
    if let Some(checksum) = info.pe_checksum {
        metadata.insert("pe.checksum".to_string(), format!("0x{checksum:08x}"));
    }
    if let Some(subsystem) = &info.pe_subsystem {
        metadata.insert("pe.subsystem".to_string(), subsystem.clone());
    }
    metadata.insert("pe.isDriver".to_string(), info.pe_is_driver.to_string());
    if let Some(driver_type) = &info.pe_driver_type {
        metadata.insert("pe.driverType".to_string(), driver_type.clone());
    }
    insert_joined_metadata(
        &mut metadata,
        "pe.driverIndicators",
        &info.pe_driver_indicators,
    );
    for (key, value) in &info.pe_version_info {
        metadata.insert(format!("pe.version.{key}"), value.clone());
    }
    if info.pe_is_driver {
        insert_pe_driver_string_metadata(&mut metadata, &info.strings);
    }
    if is_linux_kernel_module_path(&info.path) {
        insert_linux_kernel_module_string_metadata(&mut metadata, &info.strings);
    }

    if let Some(cpu_type) = &info.macho_cpu_type {
        metadata.insert("macho.cpuType".to_string(), cpu_type.clone());
    }
    if let Some(filetype) = &info.macho_filetype {
        metadata.insert("macho.fileType".to_string(), filetype.clone());
    }

    metadata
}

#[derive(Default)]
struct LinuxKernelModuleMetadata {
    names: Vec<String>,
    versions: Vec<String>,
    vermagic: Vec<String>,
    licenses: Vec<String>,
    authors: Vec<String>,
    descriptions: Vec<String>,
    aliases: Vec<String>,
    dependencies: Vec<String>,
    firmware: Vec<String>,
    signers: Vec<String>,
    signatures: Vec<String>,
}

fn is_linux_kernel_module_path(path: &str) -> bool {
    path.replace('\\', "/")
        .to_ascii_lowercase()
        .ends_with(".ko")
}

fn insert_linux_kernel_module_string_metadata(
    metadata: &mut BTreeMap<String, String>,
    strings: &[String],
) {
    let values = linux_kernel_module_metadata_from_strings(strings);
    if !values.has_values() {
        return;
    }

    metadata.insert("linux.moduleDetected".to_string(), "true".to_string());
    insert_joined_metadata(metadata, "linux.moduleNames", &values.names);
    insert_joined_metadata(metadata, "linux.moduleVersions", &values.versions);
    insert_joined_metadata(metadata, "linux.moduleVermagic", &values.vermagic);
    insert_joined_metadata(metadata, "linux.moduleLicenses", &values.licenses);
    insert_joined_metadata(metadata, "linux.moduleAuthors", &values.authors);
    insert_joined_metadata(metadata, "linux.moduleDescriptions", &values.descriptions);
    insert_joined_metadata(metadata, "linux.moduleAliases", &values.aliases);
    insert_joined_metadata(metadata, "linux.moduleDependencies", &values.dependencies);
    insert_joined_metadata(metadata, "linux.moduleFirmware", &values.firmware);
    insert_joined_metadata(metadata, "linux.moduleSigners", &values.signers);
    insert_joined_metadata(metadata, "linux.moduleSignatures", &values.signatures);
}

fn linux_kernel_module_metadata_from_strings(strings: &[String]) -> LinuxKernelModuleMetadata {
    let mut values = LinuxKernelModuleMetadata::default();
    for value in strings {
        let Some((key, value)) = split_linux_module_info(value) else {
            continue;
        };
        match key {
            "name" => push_unique_limited(&mut values.names, value),
            "version" => push_unique_limited(&mut values.versions, value),
            "vermagic" => push_unique_limited(&mut values.vermagic, value),
            "license" => push_unique_limited(&mut values.licenses, value),
            "author" => push_unique_limited(&mut values.authors, value),
            "description" => push_unique_limited(&mut values.descriptions, value),
            "alias" => push_unique_limited(&mut values.aliases, value),
            "depends" => {
                for dependency in value.split(',') {
                    let dependency = dependency.trim();
                    if !dependency.is_empty() {
                        push_unique_limited(
                            &mut values.dependencies,
                            truncate_metadata_value(dependency, 120),
                        );
                    }
                }
            }
            "firmware" => push_unique_limited(&mut values.firmware, value),
            "signer" => push_unique_limited(&mut values.signers, value),
            "sig_key" | "sig_hashalgo" => push_unique_limited(
                &mut values.signatures,
                truncate_metadata_value(&format!("{key}={value}"), 180),
            ),
            _ => {}
        }
    }
    values
}

impl LinuxKernelModuleMetadata {
    fn has_values(&self) -> bool {
        !self.names.is_empty()
            || !self.versions.is_empty()
            || !self.vermagic.is_empty()
            || !self.licenses.is_empty()
            || !self.authors.is_empty()
            || !self.descriptions.is_empty()
            || !self.aliases.is_empty()
            || !self.dependencies.is_empty()
            || !self.firmware.is_empty()
            || !self.signers.is_empty()
            || !self.signatures.is_empty()
    }
}

fn split_linux_module_info(value: &str) -> Option<(&str, String)> {
    let (key, value) = value.split_once('=')?;
    let key = key.trim();
    if !linux_module_info_key_is_supported(key) {
        return None;
    }
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    Some((key, truncate_metadata_value(value, 180)))
}

fn linux_module_info_key_is_supported(key: &str) -> bool {
    matches!(
        key,
        "name"
            | "version"
            | "vermagic"
            | "license"
            | "author"
            | "description"
            | "alias"
            | "depends"
            | "firmware"
            | "signer"
            | "sig_key"
            | "sig_hashalgo"
    )
}

fn insert_pe_driver_string_metadata(metadata: &mut BTreeMap<String, String>, strings: &[String]) {
    let mut service_names = Vec::new();
    let mut device_names = Vec::new();
    let mut dos_device_names = Vec::new();

    for value in strings {
        if let Some(service_name) = extract_windows_driver_service_name(value) {
            push_unique_limited(&mut service_names, service_name);
        }
        if let Some(device_name) = extract_windows_object_name(value, "\\device\\") {
            push_unique_limited(&mut device_names, device_name);
        }
        if let Some(dos_device_name) = extract_windows_object_name(value, "\\dosdevices\\") {
            push_unique_limited(&mut dos_device_names, dos_device_name);
        }
    }

    insert_joined_metadata(metadata, "pe.driverServiceNames", &service_names);
    insert_joined_metadata(metadata, "pe.driverDeviceNames", &device_names);
    insert_joined_metadata(metadata, "pe.driverDosDeviceNames", &dos_device_names);
}

fn extract_windows_driver_service_name(value: &str) -> Option<String> {
    let normalized = value.replace('/', "\\");
    for marker in [
        "\\currentcontrolset\\services\\",
        "\\controlset001\\services\\",
        "\\controlset002\\services\\",
        "\\controlset003\\services\\",
    ] {
        if let Some(name) = extract_after_marker(&normalized, marker) {
            return Some(name);
        }
    }
    None
}

fn extract_windows_object_name(value: &str, marker: &str) -> Option<String> {
    let normalized = value.replace('/', "\\");
    extract_after_marker(&normalized, marker)
}

fn extract_after_marker(value: &str, marker: &str) -> Option<String> {
    let lower = value.to_ascii_lowercase();
    let start = lower.find(marker)?.checked_add(marker.len())?;
    let raw = value.get(start..)?;
    let end = raw
        .find(|ch: char| {
            ch == '\\' || ch == '/' || ch == ';' || ch == '"' || ch == '\'' || ch.is_whitespace()
        })
        .unwrap_or(raw.len());
    let candidate = raw.get(..end)?.trim_matches([':', '.']);
    if candidate.is_empty()
        || !candidate.chars().any(|ch| ch.is_ascii_alphanumeric())
        || !candidate
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
    {
        return None;
    }
    Some(truncate_metadata_value(candidate, 120))
}

fn binary_format_name(format: &BinaryFormat) -> &'static str {
    match format {
        BinaryFormat::PE32 => "PE32",
        BinaryFormat::PE64 => "PE64",
        BinaryFormat::ELF32 => "ELF32",
        BinaryFormat::ELF64 => "ELF64",
        BinaryFormat::MachO32 => "MachO32",
        BinaryFormat::MachO64 => "MachO64",
        BinaryFormat::MachOFat => "MachOFat",
        BinaryFormat::Unknown => "Unknown",
    }
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
    fn binary_artifact_metadata_from_info_flattens_driver_analysis() {
        let mut version_info = BTreeMap::new();
        version_info.insert("CompanyName".to_string(), "Contoso Driver Labs".to_string());
        version_info.insert("OriginalFilename".to_string(), "contosoflt.sys".to_string());

        let info = BinaryInfo {
            path: "case.ad1:/Windows/System32/drivers/contosoflt.sys".to_string(),
            format: BinaryFormat::PE64,
            architecture: "x86_64".to_string(),
            is_64bit: true,
            entry_point: Some(0x140001000),
            imports: vec![crate::viewer::document::binary::ImportInfo {
                library: "fltmgr.sys".to_string(),
                functions: vec!["FltRegisterFilter".to_string()],
                function_count: 1,
            }],
            exports: vec![crate::viewer::document::binary::ExportInfo {
                name: "DriverEntry".to_string(),
                ordinal: None,
                address: 0x1000,
            }],
            sections: vec![crate::viewer::document::binary::SectionInfo {
                name: ".text".to_string(),
                virtual_address: 0x1000,
                virtual_size: 0x2000,
                raw_size: 0x2000,
                characteristics: "0x60000020".to_string(),
                characteristics_detail: vec![
                    "contains-code".to_string(),
                    "executable".to_string(),
                    "readable".to_string(),
                ],
                entropy: Some(7.812),
            }],
            strings: vec![
                "\\Registry\\Machine\\System\\CurrentControlSet\\Services\\contosoflt".to_string(),
                "\\Registry\\Machine\\System\\ControlSet001\\Services\\legacyflt\\Parameters"
                    .to_string(),
                "\\Device\\ContosoFilter".to_string(),
                "\\DosDevices\\ContosoFilter".to_string(),
            ],
            file_size: 4096,
            pe_timestamp: Some(1_717_260_000),
            pe_checksum: Some(0x1234abcd),
            pe_subsystem: Some("Native".to_string()),
            pe_is_driver: true,
            pe_driver_type: Some("File system minifilter driver".to_string()),
            pe_driver_indicators: vec![
                "driver file extension".to_string(),
                "file-system filter driver APIs".to_string(),
            ],
            pe_version_info: version_info,
            macho_cpu_type: None,
            macho_filetype: None,
            has_debug_info: false,
            is_stripped: true,
            has_code_signing: true,
        };

        let metadata = binary_artifact_metadata_from_info(&info);

        assert_eq!(
            metadata.get("binary.analysisStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("binary.format").map(String::as_str),
            Some("PE64")
        );
        assert_eq!(
            metadata.get("binary.entryPoint").map(String::as_str),
            Some("0x140001000")
        );
        assert_eq!(
            metadata.get("pe.isDriver").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            metadata.get("pe.driverType").map(String::as_str),
            Some("File system minifilter driver")
        );
        assert_eq!(
            metadata.get("binary.importLibraries").map(String::as_str),
            Some("fltmgr.sys")
        );
        assert_eq!(
            metadata.get("binary.exports").map(String::as_str),
            Some("DriverEntry")
        );
        assert_eq!(
            metadata
                .get("binary.sectionCharacteristics")
                .map(String::as_str),
            Some(".text=contains-code,executable,readable")
        );
        assert_eq!(
            metadata
                .get("binary.executableSections")
                .map(String::as_str),
            Some(".text")
        );
        assert_eq!(
            metadata.get("binary.maxSectionEntropy").map(String::as_str),
            Some("7.812")
        );
        assert_eq!(
            metadata
                .get("binary.highEntropySections")
                .map(String::as_str),
            Some(".text=7.812")
        );
        assert_eq!(
            metadata.get("binary.stringCount").map(String::as_str),
            Some("4")
        );
        assert_eq!(
            metadata.get("binary.strings").map(String::as_str),
            Some(
                "\\Registry\\Machine\\System\\CurrentControlSet\\Services\\contosoflt; \\Registry\\Machine\\System\\ControlSet001\\Services\\legacyflt\\Parameters; \\Device\\ContosoFilter; \\DosDevices\\ContosoFilter"
            )
        );
        assert_eq!(
            metadata.get("pe.driverServiceNames").map(String::as_str),
            Some("contosoflt; legacyflt")
        );
        assert_eq!(
            metadata.get("pe.driverDeviceNames").map(String::as_str),
            Some("ContosoFilter")
        );
        assert_eq!(
            metadata.get("pe.driverDosDeviceNames").map(String::as_str),
            Some("ContosoFilter")
        );
        assert_eq!(
            metadata.get("pe.version.CompanyName").map(String::as_str),
            Some("Contoso Driver Labs")
        );
    }

    #[test]
    fn image_artifact_metadata_from_exif_flattens_forensic_fields() {
        let info = ExifMetadata {
            path: "case.ad1:/DCIM/IMG_0001.JPG".to_string(),
            make: Some("Canon".to_string()),
            model: Some("EOS R5".to_string()),
            software: Some("Digital Photo Professional".to_string()),
            lens_model: Some("RF24-70mm F2.8 L IS USM".to_string()),
            exposure_time: Some("1/125".to_string()),
            f_number: Some("f/2.8".to_string()),
            iso: Some(400),
            focal_length: Some("35 mm".to_string()),
            flash: Some("No flash".to_string()),
            date_time_original: Some("2026:07:04 12:34:56".to_string()),
            date_time_digitized: Some("2026:07:04 12:34:57".to_string()),
            date_time: Some("2026:07:04 12:35:00".to_string()),
            gps_timestamp: Some("20:34:56".to_string()),
            gps: Some(
                crate::viewer::document::exif::GpsCoordinates::new(61.2176, -149.8997)
                    .with_altitude(31.5),
            ),
            width: Some(8192),
            height: Some(5464),
            orientation: Some(1),
            color_space: Some("sRGB".to_string()),
            image_unique_id: Some("unique-image-id".to_string()),
            owner_name: Some("Investigator".to_string()),
            serial_number: Some("CAM123456".to_string()),
            raw_tags: vec![
                ("Make".to_string(), "Canon".to_string()),
                ("Model".to_string(), "EOS R5".to_string()),
            ],
        };

        let metadata = image_artifact_metadata_from_exif(&info);

        assert_eq!(
            metadata.get("exif.analysisStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(metadata.get("exif.make").map(String::as_str), Some("Canon"));
        assert_eq!(
            metadata.get("exif.model").map(String::as_str),
            Some("EOS R5")
        );
        assert_eq!(metadata.get("exif.iso").map(String::as_str), Some("400"));
        assert_eq!(
            metadata.get("exif.dateTimeOriginal").map(String::as_str),
            Some("2026:07:04 12:34:56")
        );
        assert_eq!(
            metadata.get("exif.gpsLatitude").map(String::as_str),
            Some("61.2176")
        );
        assert_eq!(
            metadata.get("exif.gpsLongitude").map(String::as_str),
            Some("-149.8997")
        );
        assert_eq!(
            metadata.get("exif.serialNumber").map(String::as_str),
            Some("CAM123456")
        );
        assert_eq!(
            metadata.get("exif.rawTagCount").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata.get("exif.rawTags").map(String::as_str),
            Some("Make=Canon; Model=EOS R5")
        );
    }

    #[test]
    fn binary_artifact_metadata_extracts_linux_kernel_module_strings() {
        let info = BinaryInfo {
            path: "case.e01:/lib/modules/6.8.0/kernel/drivers/usb/contoso_sensor.ko".to_string(),
            format: BinaryFormat::ELF64,
            architecture: "x86_64".to_string(),
            is_64bit: true,
            entry_point: None,
            imports: vec![],
            exports: vec![],
            sections: vec![],
            strings: vec![
                "name=contoso_sensor".to_string(),
                "version=1.2.3".to_string(),
                "license=GPL".to_string(),
                "author=Contoso Driver Labs".to_string(),
                "description=Contoso USB sensor driver".to_string(),
                "alias=usb:v1234p5678d*dc*dsc*dp*ic*isc*ip*in*".to_string(),
                "depends=usbcore,industrialio".to_string(),
                "firmware=contoso/sensor.bin".to_string(),
                "vermagic=6.8.0-31-generic SMP preempt mod_unload modversions".to_string(),
                "signer=Secure Boot Module Signature key".to_string(),
                "sig_key=AA:BB:CC:DD".to_string(),
                "sig_hashalgo=sha256".to_string(),
            ],
            file_size: 8192,
            pe_timestamp: None,
            pe_checksum: None,
            pe_subsystem: None,
            pe_is_driver: false,
            pe_driver_type: None,
            pe_driver_indicators: vec![],
            pe_version_info: BTreeMap::new(),
            macho_cpu_type: None,
            macho_filetype: None,
            has_debug_info: false,
            is_stripped: true,
            has_code_signing: false,
        };

        let metadata = binary_artifact_metadata_from_info(&info);

        assert_eq!(
            metadata.get("linux.moduleDetected").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            metadata.get("linux.moduleNames").map(String::as_str),
            Some("contoso_sensor")
        );
        assert_eq!(
            metadata.get("linux.moduleVersions").map(String::as_str),
            Some("1.2.3")
        );
        assert_eq!(
            metadata.get("linux.moduleLicenses").map(String::as_str),
            Some("GPL")
        );
        assert_eq!(
            metadata.get("linux.moduleAuthors").map(String::as_str),
            Some("Contoso Driver Labs")
        );
        assert_eq!(
            metadata.get("linux.moduleDescriptions").map(String::as_str),
            Some("Contoso USB sensor driver")
        );
        assert_eq!(
            metadata.get("linux.moduleAliases").map(String::as_str),
            Some("usb:v1234p5678d*dc*dsc*dp*ic*isc*ip*in*")
        );
        assert_eq!(
            metadata.get("linux.moduleDependencies").map(String::as_str),
            Some("usbcore; industrialio")
        );
        assert_eq!(
            metadata.get("linux.moduleFirmware").map(String::as_str),
            Some("contoso/sensor.bin")
        );
        assert_eq!(
            metadata.get("linux.moduleVermagic").map(String::as_str),
            Some("6.8.0-31-generic SMP preempt mod_unload modversions")
        );
        assert_eq!(
            metadata.get("linux.moduleSigners").map(String::as_str),
            Some("Secure Boot Module Signature key")
        );
        assert_eq!(
            metadata.get("linux.moduleSignatures").map(String::as_str),
            Some("sig_key=AA:BB:CC:DD; sig_hashalgo=sha256")
        );
    }

    #[test]
    fn linux_kernel_module_metadata_only_triggers_for_ko_paths() {
        assert!(is_linux_kernel_module_path("/lib/modules/example.ko"));
        assert!(is_linux_kernel_module_path(r"C:\case\modules\example.ko"));
        assert!(!is_linux_kernel_module_path("/usr/bin/example"));

        let values = linux_kernel_module_metadata_from_strings(&[
            "license=GPL".to_string(),
            "depends=usbcore, industrialio".to_string(),
            "unrelated=value".to_string(),
        ]);
        assert!(values.has_values());
        assert_eq!(values.licenses, vec!["GPL".to_string()]);
        assert_eq!(
            values.dependencies,
            vec!["usbcore".to_string(), "industrialio".to_string()]
        );
    }

    #[test]
    fn is_image_artifact_matches_common_image_extensions_and_mime() {
        let mut artifact = NormalizedArtifact {
            id: "artifact_image".to_string(),
            source_ref: EvidenceSourceRef::LocalFile {
                path: "photo.jpg".to_string(),
            },
            source_id: "photo.jpg".to_string(),
            name: "photo.jpg".to_string(),
            extension: Some("jpg".to_string()),
            size: 1024,
            mime_type: None,
            type_description: "JPEG Image".to_string(),
            category: "unknown".to_string(),
            confidence: "medium".to_string(),
            is_text: false,
            content_preview: None,
            metadata: BTreeMap::new(),
        };

        assert!(is_image_artifact(&artifact));

        artifact.extension = None;
        artifact.mime_type = Some("image/png".to_string());
        assert!(is_image_artifact(&artifact));

        artifact.mime_type = Some("application/octet-stream".to_string());
        artifact.category = "image".to_string();
        assert!(is_image_artifact(&artifact));
    }

    #[test]
    fn is_binary_artifact_matches_driver_extension_even_when_magic_is_unknown() {
        for extension in ["sys", "ko", "kext"] {
            let name = format!("example.{extension}");
            let path = format!("/case/{name}");
            let artifact = NormalizedArtifact {
                id: format!("artifact_{extension}"),
                source_ref: EvidenceSourceRef::LocalFile { path: path.clone() },
                source_id: path,
                name,
                extension: Some(extension.to_string()),
                size: 1024,
                mime_type: None,
                type_description: "Unknown".to_string(),
                category: "unknown".to_string(),
                confidence: "low".to_string(),
                is_text: false,
                content_preview: None,
                metadata: BTreeMap::new(),
            };

            assert!(is_binary_artifact(&artifact), "{extension}");
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
        let product_version =
            system_identity_metadata_from_bytes("/image/sys/class/dmi/id/product_version", b"01\n");
        let product_family = system_identity_metadata_from_bytes(
            "/image/sys/class/dmi/id/product_family",
            b"Precision\n",
        );
        let product_sku =
            system_identity_metadata_from_bytes("/image/sys/class/dmi/id/product_sku", b"SKU-42\n");
        let board_asset_tag = system_identity_metadata_from_bytes(
            "/image/sys/class/dmi/id/board_asset_tag",
            b"BOARD-ASSET\n",
        );
        let board_vendor = system_identity_metadata_from_bytes(
            "/image/sys/class/dmi/id/board_vendor",
            b"Dell Inc.\n",
        );
        let chassis_asset_tag = system_identity_metadata_from_bytes(
            "/image/sys/class/dmi/id/chassis_asset_tag",
            b"CHASSIS-ASSET\n",
        );
        let chassis_serial = system_identity_metadata_from_bytes(
            "/image/sys/class/dmi/id/chassis_serial",
            b"CHS123\n",
        );
        let bios_date = system_identity_metadata_from_bytes(
            "/image/sys/class/dmi/id/bios_date",
            b"05/01/2026\n",
        );

        assert_eq!(
            serial.get("system.serialNumber").map(String::as_str),
            Some("ABC123")
        );
        assert_eq!(
            vendor.get("system.manufacturer").map(String::as_str),
            Some("Dell Inc.")
        );
        assert_eq!(
            product_version
                .get("system.productVersion")
                .map(String::as_str),
            Some("01")
        );
        assert_eq!(
            product_family.get("system.family").map(String::as_str),
            Some("Precision")
        );
        assert_eq!(
            product_sku.get("system.sku").map(String::as_str),
            Some("SKU-42")
        );
        assert_eq!(
            board_asset_tag
                .get("system.boardAssetTag")
                .map(String::as_str),
            Some("BOARD-ASSET")
        );
        assert_eq!(
            board_vendor.get("system.boardVendor").map(String::as_str),
            Some("Dell Inc.")
        );
        assert_eq!(
            chassis_asset_tag
                .get("system.chassisAssetTag")
                .map(String::as_str),
            Some("CHASSIS-ASSET")
        );
        assert_eq!(
            chassis_serial
                .get("system.chassisSerial")
                .map(String::as_str),
            Some("CHS123")
        );
        assert_eq!(
            bios_date.get("system.biosDate").map(String::as_str),
            Some("05/01/2026")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_timezone_file() {
        let metadata =
            system_identity_metadata_from_bytes("/image/etc/timezone", b"America/Anchorage\n");

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("system.timeZone").map(String::as_str),
            Some("America/Anchorage")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_zoneinfo_localtime_target() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/etc/localtime",
            b"/usr/share/zoneinfo/America/Anchorage\n",
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("system.timeZone").map(String::as_str),
            Some("America/Anchorage")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_tzif_localtime() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/etc/localtime",
            &minimal_tzif_v2(b"AKST\0AKDT\0", b"AKST9AKDT,M3.2.0,M11.1.0"),
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("system.timeZoneFormat").map(String::as_str),
            Some("TZif")
        );
        assert_eq!(
            metadata
                .get("system.timeZoneFileVersion")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata.get("system.timeZoneRule").map(String::as_str),
            Some("AKST9AKDT,M3.2.0,M11.1.0")
        );
        assert_eq!(
            metadata
                .get("system.timeZoneAbbreviations")
                .map(String::as_str),
            Some("AKST; AKDT")
        );
    }

    fn minimal_tzif_v2(abbreviations: &[u8], posix_rule: &[u8]) -> Vec<u8> {
        let mut data = Vec::new();
        append_tzif_header(&mut data, b'2', 0, 0, 0, 0, 1, abbreviations.len() as u32);
        data.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
        data.extend_from_slice(abbreviations);
        append_tzif_header(&mut data, b'2', 0, 0, 0, 0, 1, abbreviations.len() as u32);
        data.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
        data.extend_from_slice(abbreviations);
        data.push(b'\n');
        data.extend_from_slice(posix_rule);
        data.push(b'\n');
        data
    }

    fn append_tzif_header(
        data: &mut Vec<u8>,
        version: u8,
        ttisgmt_count: u32,
        ttisstd_count: u32,
        leap_count: u32,
        time_count: u32,
        type_count: u32,
        char_count: u32,
    ) {
        data.extend_from_slice(b"TZif");
        data.push(version);
        data.extend_from_slice(&[0; 15]);
        for value in [
            ttisgmt_count,
            ttisstd_count,
            leap_count,
            time_count,
            type_count,
            char_count,
        ] {
            data.extend_from_slice(&value.to_be_bytes());
        }
    }

    #[test]
    fn system_identity_metadata_extracts_linux_machine_info() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/etc/machine-info",
            br#"PRETTY_HOSTNAME="Evidence Workstation"
ICON_NAME=computer-desktop
CHASSIS=desktop
DEPLOYMENT=lab
LOCATION="Forensic Bench 2"
"#,
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("system.prettyHostname").map(String::as_str),
            Some("Evidence Workstation")
        );
        assert_eq!(
            metadata.get("system.chassis").map(String::as_str),
            Some("desktop")
        );
        assert_eq!(
            metadata.get("system.deployment").map(String::as_str),
            Some("lab")
        );
        assert_eq!(
            metadata.get("system.location").map(String::as_str),
            Some("Forensic Bench 2")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_dmidecode_hardware_inventory() {
        let metadata = system_identity_metadata_from_bytes(
            "/case/reports/dmidecode.txt",
            br#"System Information
        Manufacturer: Dell Inc.
        Product Name: Precision 7780
        Version: 01
        Serial Number: ABC1234
        UUID: 00112233-4455-6677-8899-aabbccddeeff
        SKU Number: SKU-7780
        Family: Precision

Base Board Information
        Manufacturer: Dell Inc.
        Product Name: 0TEST1
        Version: A00
        Serial Number: BOARD123
        Asset Tag: BOARD-ASSET

BIOS Information
        Vendor: Dell Inc.
        Version: 1.12.0
        Release Date: 05/01/2026

Chassis Information
        Manufacturer: Dell Inc.
        Type: Laptop
        Serial Number: CHS123
        Asset Tag: CHASSIS-ASSET
"#,
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata
                .get("system.hardwareInventorySource")
                .map(String::as_str),
            Some("dmidecode")
        );
        assert_eq!(
            metadata.get("system.manufacturer").map(String::as_str),
            Some("Dell Inc.")
        );
        assert_eq!(
            metadata.get("system.model").map(String::as_str),
            Some("Precision 7780")
        );
        assert_eq!(
            metadata.get("system.serialNumber").map(String::as_str),
            Some("ABC1234")
        );
        assert_eq!(
            metadata.get("system.hardwareUuid").map(String::as_str),
            Some("00112233-4455-6677-8899-aabbccddeeff")
        );
        assert_eq!(
            metadata.get("system.sku").map(String::as_str),
            Some("SKU-7780")
        );
        assert_eq!(
            metadata.get("system.boardName").map(String::as_str),
            Some("0TEST1")
        );
        assert_eq!(
            metadata.get("system.boardSerial").map(String::as_str),
            Some("BOARD123")
        );
        assert_eq!(
            metadata.get("system.biosVersion").map(String::as_str),
            Some("1.12.0")
        );
        assert_eq!(
            metadata.get("system.chassisType").map(String::as_str),
            Some("Laptop")
        );
        assert_eq!(
            metadata.get("system.chassisSerial").map(String::as_str),
            Some("CHS123")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_lshw_hardware_inventory() {
        let metadata = system_identity_metadata_from_bytes(
            "/case/inventory/lshw.txt",
            br#"evidence-host
    description: Notebook
    product: ThinkPad P1 Gen 6
    vendor: LENOVO
    version: ThinkPad P1 Gen 6
    serial: PF4TEST
    *-core
         description: Motherboard
         product: 21FVCTO1WW
         vendor: LENOVO
         version: SDK0T76530 WIN
         serial: L1HFTEST
       *-firmware
            description: BIOS
            vendor: LENOVO
            version: N3JET99W
            date: 04/04/2026
"#,
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata
                .get("system.hardwareInventorySource")
                .map(String::as_str),
            Some("lshw")
        );
        assert_eq!(
            metadata.get("system.manufacturer").map(String::as_str),
            Some("LENOVO")
        );
        assert_eq!(
            metadata.get("system.model").map(String::as_str),
            Some("ThinkPad P1 Gen 6")
        );
        assert_eq!(
            metadata.get("system.serialNumber").map(String::as_str),
            Some("PF4TEST")
        );
        assert_eq!(
            metadata.get("system.boardName").map(String::as_str),
            Some("21FVCTO1WW")
        );
        assert_eq!(
            metadata.get("system.boardSerial").map(String::as_str),
            Some("L1HFTEST")
        );
        assert_eq!(
            metadata.get("system.biosVendor").map(String::as_str),
            Some("LENOVO")
        );
        assert_eq!(
            metadata.get("system.biosVersion").map(String::as_str),
            Some("N3JET99W")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_dbus_machine_id_and_locale() {
        let dbus = system_identity_metadata_from_bytes(
            "/image/var/lib/dbus/machine-id",
            b"0123456789abcdef0123456789abcdef\n",
        );
        let locale = system_identity_metadata_from_bytes(
            "/image/etc/default/locale",
            b"LANG=en_US.UTF-8\nLANGUAGE=en_US:en\nLC_TIME=en_GB.UTF-8\n",
        );

        assert_eq!(
            dbus.get("system.machineId").map(String::as_str),
            Some("0123456789abcdef0123456789abcdef")
        );
        assert_eq!(
            dbus.get("system.machineIdSource").map(String::as_str),
            Some("dbus-machine-id")
        );
        assert_eq!(
            locale.get("system.locale").map(String::as_str),
            Some("en_US.UTF-8")
        );
        assert_eq!(
            locale.get("system.language").map(String::as_str),
            Some("en_US:en")
        );
        assert_eq!(
            locale.get("system.localeTime").map(String::as_str),
            Some("en_GB.UTF-8")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_mount_table() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/etc/fstab",
            br#"# /etc/fstab
UUID=root-uuid / ext4 defaults 0 1
/dev/disk/by-label/Case\040Data /mnt/case\040data xfs ro,nosuid 0 0
tmpfs /run tmpfs rw,nosuid,nodev 0 0
"#,
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("system.rootDevice").map(String::as_str),
            Some("UUID=root-uuid")
        );
        assert_eq!(
            metadata.get("system.mountCount").map(String::as_str),
            Some("3")
        );
        let mounts = metadata
            .get("system.mounts")
            .expect("mount descriptions are captured");
        assert!(mounts.contains("UUID=root-uuid on / (ext4, defaults)"));
        assert!(mounts.contains("/dev/disk/by-label/Case Data on /mnt/case data (xfs, ro,nosuid)"));
    }

    #[test]
    fn system_identity_metadata_extracts_debian_network_interfaces() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/etc/network/interfaces",
            br#"# primary interface
auto lo eth0
iface lo inet loopback
iface eth0 inet static
    address 192.168.10.5/24
    gateway 192.168.10.1
    dns-nameservers 1.1.1.1 8.8.8.8
"#,
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("system.networkConfigType").map(String::as_str),
            Some("debian-interfaces")
        );
        assert!(metadata
            .get("system.networkInterfaces")
            .is_some_and(|value| value.contains("eth0")));
        assert_eq!(
            metadata.get("system.ipv4Addresses").map(String::as_str),
            Some("192.168.10.5/24")
        );
        assert_eq!(
            metadata.get("system.gateways").map(String::as_str),
            Some("192.168.10.1")
        );
        assert_eq!(
            metadata.get("system.dnsServers").map(String::as_str),
            Some("1.1.1.1; 8.8.8.8")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_ifcfg_network_config() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/etc/sysconfig/network-scripts/ifcfg-ens192",
            br#"DEVICE=ens192
BOOTPROTO=static
IPADDR=10.0.0.20
PREFIX=24
GATEWAY=10.0.0.1
DNS1=9.9.9.9
DNS2=149.112.112.112
"#,
        );

        assert_eq!(
            metadata.get("system.networkConfigType").map(String::as_str),
            Some("ifcfg")
        );
        assert_eq!(
            metadata.get("system.networkInterfaces").map(String::as_str),
            Some("ens192")
        );
        assert_eq!(
            metadata.get("system.ipv4Addresses").map(String::as_str),
            Some("10.0.0.20/24")
        );
        assert_eq!(
            metadata.get("system.gateways").map(String::as_str),
            Some("10.0.0.1")
        );
        assert_eq!(
            metadata.get("system.dnsServers").map(String::as_str),
            Some("9.9.9.9; 149.112.112.112")
        );
        assert_eq!(
            metadata.get("system.networkMethods").map(String::as_str),
            Some("ens192:inet:static")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_netplan_network_config() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/etc/netplan/01-netcfg.yaml",
            br#"network:
  version: 2
  ethernets:
    ens18:
      dhcp4: false
      addresses: [172.16.1.5/24]
      gateway4: 172.16.1.1
      nameservers:
        addresses: [1.1.1.1, 8.8.4.4]
"#,
        );

        assert_eq!(
            metadata.get("system.networkConfigType").map(String::as_str),
            Some("netplan")
        );
        assert_eq!(
            metadata.get("system.networkInterfaces").map(String::as_str),
            Some("ens18")
        );
        assert_eq!(
            metadata.get("system.ipv4Addresses").map(String::as_str),
            Some("172.16.1.5/24")
        );
        assert_eq!(
            metadata.get("system.gateways").map(String::as_str),
            Some("172.16.1.1")
        );
        assert_eq!(
            metadata.get("system.dnsServers").map(String::as_str),
            Some("1.1.1.1; 8.8.4.4")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_resolver_config() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/etc/resolv.conf",
            br#"# generated by NetworkManager
search corp.example.com lab.example.com
nameserver 10.10.0.2
nameserver 1.1.1.1
"#,
        );

        assert_eq!(
            metadata.get("system.networkConfigType").map(String::as_str),
            Some("resolver")
        );
        assert_eq!(
            metadata.get("system.dnsServers").map(String::as_str),
            Some("10.10.0.2; 1.1.1.1")
        );
        assert_eq!(
            metadata.get("system.dnsSearchDomains").map(String::as_str),
            Some("corp.example.com; lab.example.com")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_hosts_aliases() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/etc/hosts",
            br#"127.0.0.1 localhost
10.10.0.25 workstation01 workstation01.corp.example.com
::1 localhost ip6-localhost
"#,
        );

        assert_eq!(
            metadata.get("system.networkConfigType").map(String::as_str),
            Some("hosts")
        );
        let aliases = metadata
            .get("system.hostAliases")
            .expect("host aliases are captured");
        assert!(aliases.contains("127.0.0.1=localhost"));
        assert!(aliases.contains("10.10.0.25=workstation01,workstation01.corp.example.com"));
    }

    #[test]
    fn system_identity_metadata_extracts_unix_passwd_accounts() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/etc/passwd",
            b"root:x:0:0:root:/root:/bin/bash\n\
              daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin\n\
              alice:x:1000:1000:Alice Analyst:/home/alice:/bin/bash\n\
              bob:x:1001:1001:Bob User:/home/bob:/bin/zsh\n",
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("system.accountConfigType").map(String::as_str),
            Some("unix-passwd")
        );
        assert_eq!(
            metadata.get("system.localUserCount").map(String::as_str),
            Some("4")
        );
        assert_eq!(
            metadata.get("system.regularUserCount").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata.get("system.loginUserCount").map(String::as_str),
            Some("3")
        );
        assert_eq!(
            metadata
                .get("system.rootAccountPresent")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(
            metadata.get("system.userUidRange").map(String::as_str),
            Some("0-1001")
        );
        assert_eq!(
            metadata.get("system.regularUsers").map(String::as_str),
            Some("alice (Alice Analyst); bob (Bob User)")
        );
        assert_eq!(
            metadata.get("system.homeDirectories").map(String::as_str),
            Some("/root; /usr/sbin; /home/alice; /home/bob")
        );
        assert_eq!(
            metadata.get("system.loginShells").map(String::as_str),
            Some("/bin/bash; /bin/zsh")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_macos_dslocal_user_safely() {
        let metadata = system_identity_metadata_from_bytes(
            "/private/var/db/dslocal/nodes/Default/users/alice.plist",
            br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>name</key><array><string>alice</string></array>
  <key>realname</key><array><string>Alice Analyst</string></array>
  <key>uid</key><array><string>501</string></array>
  <key>gid</key><array><string>20</string></array>
  <key>home</key><array><string>/Users/alice</string></array>
  <key>shell</key><array><string>/bin/zsh</string></array>
  <key>generateduid</key><array><string>AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE</string></array>
  <key>ShadowHashData</key><array><data>c2VjcmV0LWhhc2g=</data></array>
</dict>
</plist>"#,
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("system.accountConfigType").map(String::as_str),
            Some("macos-dslocal-user")
        );
        assert_eq!(
            metadata.get("system.localUsers").map(String::as_str),
            Some("alice")
        );
        assert_eq!(
            metadata.get("system.regularUsers").map(String::as_str),
            Some("alice (Alice Analyst)")
        );
        assert_eq!(
            metadata.get("system.loginUsers").map(String::as_str),
            Some("alice (Alice Analyst)")
        );
        assert_eq!(
            metadata.get("system.homeDirectories").map(String::as_str),
            Some("/Users/alice")
        );
        assert_eq!(
            metadata.get("system.loginShells").map(String::as_str),
            Some("/bin/zsh")
        );
        assert_eq!(
            metadata.get("system.userGeneratedUids").map(String::as_str),
            Some("AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE")
        );
        assert!(!metadata
            .keys()
            .any(|key| key.to_ascii_lowercase().contains("hash")));
        assert!(!metadata.values().any(|value| value.contains("secret-hash")));
    }

    #[test]
    fn system_identity_metadata_extracts_macos_dslocal_group_membership() {
        let metadata = system_identity_metadata_from_bytes(
            "/private/var/db/dslocal/nodes/Default/groups/admin.plist",
            br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>name</key><array><string>admin</string></array>
  <key>gid</key><array><string>80</string></array>
  <key>users</key><array><string>root</string><string>alice</string></array>
  <key>groupmembers</key><array>
    <string>FFFFFFFF-1111-2222-3333-444444444444</string>
    <string>AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE</string>
  </array>
</dict>
</plist>"#,
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("system.accountConfigType").map(String::as_str),
            Some("macos-dslocal-group")
        );
        assert_eq!(
            metadata.get("system.localGroups").map(String::as_str),
            Some("admin:gid=80")
        );
        assert_eq!(
            metadata.get("system.groupMembers").map(String::as_str),
            Some("admin=root,alice")
        );
        assert_eq!(
            metadata.get("system.adminGroups").map(String::as_str),
            Some("admin:members=root,alice")
        );
        assert_eq!(
            metadata
                .get("system.groupGeneratedUids")
                .map(String::as_str),
            Some("FFFFFFFF-1111-2222-3333-444444444444; AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_unix_group_accounts() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/etc/group",
            b"root:x:0:\nwheel:x:10:root,alice\nsudo:x:27:alice,bob\nusers:x:100:alice,bob\n",
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("system.accountConfigType").map(String::as_str),
            Some("unix-group")
        );
        assert_eq!(
            metadata.get("system.localGroupCount").map(String::as_str),
            Some("4")
        );
        assert_eq!(
            metadata.get("system.localGroups").map(String::as_str),
            Some("root:gid=0; wheel:gid=10; sudo:gid=27; users:gid=100")
        );
        assert_eq!(
            metadata.get("system.adminGroups").map(String::as_str),
            Some("wheel:members=root,alice; sudo:members=alice,bob")
        );
        assert_eq!(
            metadata.get("system.groupMembers").map(String::as_str),
            Some("wheel=root,alice; sudo=alice,bob; users=alice,bob")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_unix_shadow_safely() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/etc/shadow",
            b"root:$6$salt$abcdef:19500:0:99999:7:::\n\
              alice:$y$j9T$salt$hash:19501:0:99999:7:::\n\
              bob:!:19502:0:99999:7:::\n\
              daemon:*:19503:0:99999:7:::\n\
              test::19504:0:99999:7:::\n",
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("system.accountConfigType").map(String::as_str),
            Some("unix-shadow")
        );
        assert_eq!(
            metadata.get("system.shadowEntryCount").map(String::as_str),
            Some("5")
        );
        assert_eq!(
            metadata
                .get("system.passwordHashUserCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata
                .get("system.passwordLockedUserCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            metadata
                .get("system.passwordDisabledUserCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            metadata
                .get("system.passwordEmptyUserCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            metadata.get("system.passwordHashUsers").map(String::as_str),
            Some("root; alice")
        );
        assert_eq!(
            metadata
                .get("system.passwordHashAlgorithms")
                .map(String::as_str),
            Some("sha512-crypt; yescrypt")
        );
        assert!(!metadata
            .values()
            .any(|value| value.contains("$6$salt") || value.contains("$y$j9T")));
    }

    #[test]
    fn system_identity_metadata_extracts_unix_gshadow_accounts() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/etc/gshadow",
            b"root:*::\nwheel:!:root:alice\nsudo:!:alice:bob,carol\nusers:!::alice,bob\n",
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("system.accountConfigType").map(String::as_str),
            Some("unix-gshadow")
        );
        assert_eq!(
            metadata.get("system.localGroupCount").map(String::as_str),
            Some("4")
        );
        assert_eq!(
            metadata.get("system.localGroups").map(String::as_str),
            Some("root; wheel; sudo; users")
        );
        assert_eq!(
            metadata.get("system.adminGroups").map(String::as_str),
            Some("wheel:admins=root; sudo:admins=alice")
        );
        assert_eq!(
            metadata.get("system.groupMembers").map(String::as_str),
            Some("wheel=alice; sudo=bob,carol; users=alice,bob")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_network_manager_profile() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/etc/NetworkManager/system-connections/Corp WiFi.nmconnection",
            br#"[connection]
id=Corp WiFi
uuid=11111111-2222-3333-4444-555555555555
type=wifi
interface-name=wlp2s0

[wifi]
ssid=CorpNet
mac-address=aa:bb:cc:dd:ee:ff

[ipv4]
method=manual
address1=192.168.50.25/24,192.168.50.1
dns=10.10.0.2;1.1.1.1;
dns-search=corp.example.com;
"#,
        );

        assert_eq!(
            metadata.get("system.networkConfigType").map(String::as_str),
            Some("networkmanager")
        );
        assert_eq!(
            metadata.get("system.connectionIds").map(String::as_str),
            Some("Corp WiFi")
        );
        assert_eq!(
            metadata.get("system.connectionUuids").map(String::as_str),
            Some("11111111-2222-3333-4444-555555555555")
        );
        assert_eq!(
            metadata.get("system.networkInterfaces").map(String::as_str),
            Some("wlp2s0")
        );
        assert_eq!(
            metadata.get("system.wifiSsids").map(String::as_str),
            Some("CorpNet")
        );
        assert_eq!(
            metadata.get("system.ipv4Addresses").map(String::as_str),
            Some("192.168.50.25/24")
        );
        assert_eq!(
            metadata.get("system.gateways").map(String::as_str),
            Some("192.168.50.1")
        );
        assert_eq!(
            metadata.get("system.dnsServers").map(String::as_str),
            Some("10.10.0.2; 1.1.1.1")
        );
        assert_eq!(
            metadata.get("system.networkMethods").map(String::as_str),
            Some("wlp2s0:inet:manual")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_windows_wifi_profile() {
        let metadata = system_identity_metadata_from_bytes(
            "/ProgramData/Microsoft/Wlansvc/Profiles/Interfaces/{iface}/{profile}.xml",
            br#"<?xml version="1.0"?>
<WLANProfile xmlns="http://www.microsoft.com/networking/WLAN/profile/v1">
  <name>CorpNet Profile</name>
  <SSIDConfig>
    <SSID>
      <hex>436F72704E6574</hex>
      <name>CorpNet</name>
    </SSID>
  </SSIDConfig>
  <connectionType>ESS</connectionType>
  <connectionMode>auto</connectionMode>
  <MSM>
    <security>
      <authEncryption>
        <authentication>WPA2PSK</authentication>
        <encryption>AES</encryption>
        <useOneX>false</useOneX>
      </authEncryption>
    </security>
  </MSM>
</WLANProfile>
"#,
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("system.networkConfigType").map(String::as_str),
            Some("windows-wlan-profile")
        );
        assert_eq!(
            metadata.get("system.connectionIds").map(String::as_str),
            Some("CorpNet Profile")
        );
        assert_eq!(
            metadata.get("system.wifiSsids").map(String::as_str),
            Some("CorpNet")
        );
        assert_eq!(
            metadata.get("system.wifiAuthTypes").map(String::as_str),
            Some("WPA2PSK")
        );
        assert_eq!(
            metadata
                .get("system.wifiEncryptionTypes")
                .map(String::as_str),
            Some("AES")
        );
        assert_eq!(
            metadata
                .get("system.networkConnectionModes")
                .map(String::as_str),
            Some("auto")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_shell_history_summary() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/home/alice/.bash_history",
            br#"ls -la
sudo cat /etc/shadow
ssh admin@example.com
curl -O https://example.com/tool
"#,
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata
                .get("activity.commandHistoryType")
                .map(String::as_str),
            Some("bash")
        );
        assert_eq!(
            metadata.get("activity.commandCount").map(String::as_str),
            Some("4")
        );
        assert_eq!(
            metadata
                .get("activity.networkCommandCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata
                .get("activity.privilegedCommandCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            metadata
                .get("activity.fileTransferCommandCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            metadata.get("activity.commandNames").map(String::as_str),
            Some("ls; sudo; ssh; curl")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_zsh_extended_history_summary() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/Users/alice/.zsh_history",
            br#": 1717260000:0;git status
: 1717260001:0;rsync -av /a /b
"#,
        );

        assert_eq!(
            metadata
                .get("activity.commandHistoryType")
                .map(String::as_str),
            Some("zsh")
        );
        assert_eq!(
            metadata.get("activity.commandCount").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata
                .get("activity.fileTransferCommandCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            metadata.get("activity.commandNames").map(String::as_str),
            Some("git; rsync")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_powershell_history_summary() {
        let metadata = system_identity_metadata_from_bytes(
            "/Users/Alice/AppData/Roaming/Microsoft/Windows/PowerShell/PSReadLine/ConsoleHost_history.txt",
            br#"Get-ChildItem C:\
Invoke-WebRequest https://example.com/payload -OutFile payload.bin
runas /user:Administrator cmd
"#,
        );

        assert_eq!(
            metadata
                .get("activity.commandHistoryType")
                .map(String::as_str),
            Some("powershell")
        );
        assert_eq!(
            metadata.get("activity.commandCount").map(String::as_str),
            Some("3")
        );
        assert_eq!(
            metadata
                .get("activity.networkCommandCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            metadata
                .get("activity.privilegedCommandCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            metadata.get("activity.commandNames").map(String::as_str),
            Some("Get-ChildItem; Invoke-WebRequest; runas")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_iptables_rules() {
        let metadata = system_identity_metadata_from_bytes(
            "/image/etc/sysconfig/iptables",
            br#"# sample rules
*filter
:INPUT DROP [0:0]
:FORWARD DROP [0:0]
:OUTPUT ACCEPT [0:0]
-A INPUT -p tcp --dport 22 -j ACCEPT
-A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
COMMIT
"#,
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata
                .get("system.firewallConfigType")
                .map(String::as_str),
            Some("iptables")
        );
        assert_eq!(
            metadata.get("system.firewallRuleCount").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata.get("system.firewallTables").map(String::as_str),
            Some("filter")
        );
        assert_eq!(
            metadata.get("system.firewallPolicies").map(String::as_str),
            Some("INPUT:DROP; FORWARD:DROP; OUTPUT:ACCEPT")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_windows_firewall_log_summary() {
        let metadata = system_identity_metadata_from_bytes(
            "/Windows/System32/LogFiles/Firewall/pfirewall.log",
            br#"#Version: 1.5
#Fields: date time action protocol src-ip dst-ip src-port dst-port size tcpflags tcpsyn tcpack tcpwin icmptype icmpcode info path
2026-06-01 12:00:00 DROP TCP 10.0.0.5 10.0.0.10 51515 445 60 S 1 0 8192 - - - RECEIVE
2026-06-01 12:00:01 ALLOW UDP 10.0.0.5 8.8.8.8 51516 53 80 - - - - - - - SEND
"#,
        );

        assert_eq!(
            metadata
                .get("system.firewallConfigType")
                .map(String::as_str),
            Some("windows-firewall-log")
        );
        assert_eq!(
            metadata
                .get("system.firewallLogEntryCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata
                .get("system.firewallDroppedCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            metadata
                .get("system.firewallAllowedCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            metadata.get("system.firewallProtocols").map(String::as_str),
            Some("TCP; UDP")
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
    fn system_identity_metadata_extracts_macos_hardware_identity_plist() {
        let metadata = system_identity_metadata_from_bytes(
            "/case/SystemProfiler/SPHardwareDataType.plist",
            br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>_items</key>
  <array>
    <dict>
      <key>machine_name</key><string>MacBook Pro</string>
      <key>machine_model</key><string>MacBookPro18,3</string>
      <key>serial_number</key><string>C02TEST12345</string>
      <key>platform_UUID</key><string>00000000-1111-2222-3333-444444444444</string>
      <key>boot_rom_version</key><string>11881.120.56</string>
      <key>smc_version_system</key><string>1.0f0</string>
      <key>cpu_type</key><string>Apple M1 Pro</string>
      <key>current_processor_speed</key><string>3.2 GHz</string>
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
            metadata.get("system.model").map(String::as_str),
            Some("MacBook Pro")
        );
        assert_eq!(
            metadata.get("system.modelIdentifier").map(String::as_str),
            Some("MacBookPro18,3")
        );
        assert_eq!(
            metadata.get("system.serialNumber").map(String::as_str),
            Some("C02TEST12345")
        );
        assert_eq!(
            metadata.get("system.hardwareUuid").map(String::as_str),
            Some("00000000-1111-2222-3333-444444444444")
        );
        assert_eq!(
            metadata.get("system.bootRomVersion").map(String::as_str),
            Some("11881.120.56")
        );
        assert_eq!(
            metadata.get("system.smcVersion").map(String::as_str),
            Some("1.0f0")
        );
        assert_eq!(
            metadata.get("system.cpuType").map(String::as_str),
            Some("Apple M1 Pro")
        );
        assert_eq!(
            metadata.get("system.cpuSpeed").map(String::as_str),
            Some("3.2 GHz")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_macos_kernel_extension_info_plist() {
        let metadata = system_identity_metadata_from_bytes(
            "/System/Library/Extensions/ContosoSensor.kext/Contents/Info.plist",
            br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key><string>com.contoso.driver.Sensor</string>
  <key>CFBundleName</key><string>ContosoSensor</string>
  <key>CFBundleVersion</key><string>1.2.3</string>
  <key>CFBundleShortVersionString</key><string>1.2</string>
  <key>IOKitPersonalities</key>
  <dict>
    <key>Contoso Sensor Device</key>
    <dict>
      <key>IOClass</key><string>ContosoSensorDriver</string>
      <key>IOProviderClass</key><string>IOUSBHostInterface</string>
      <key>IONameMatch</key><string>contoso-sensor</string>
    </dict>
  </dict>
  <key>OSBundleLibraries</key>
  <dict>
    <key>com.apple.iokit.IOUSBHostFamily</key><string>1.2</string>
    <key>com.apple.kpi.libkern</key><string>20.0</string>
  </dict>
</dict>
</plist>"#,
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata
                .get("system.kernelExtensionConfigType")
                .map(String::as_str),
            Some("macos-kext")
        );
        assert_eq!(
            metadata
                .get("system.kernelExtensionIdentifiers")
                .map(String::as_str),
            Some("com.contoso.driver.Sensor")
        );
        assert_eq!(
            metadata
                .get("system.kernelExtensionVersions")
                .map(String::as_str),
            Some("1.2.3; 1.2")
        );
        assert_eq!(
            metadata
                .get("system.kernelExtensionPersonalityCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            metadata
                .get("system.kernelExtensionPersonalities")
                .map(String::as_str),
            Some("Contoso Sensor Device")
        );
        assert_eq!(
            metadata
                .get("system.kernelExtensionClasses")
                .map(String::as_str),
            Some("ContosoSensorDriver")
        );
        assert_eq!(
            metadata
                .get("system.kernelExtensionProviderClasses")
                .map(String::as_str),
            Some("IOUSBHostInterface")
        );
        assert_eq!(
            metadata
                .get("system.kernelExtensionMatches")
                .map(String::as_str),
            Some("contoso-sensor")
        );
        assert_eq!(
            metadata
                .get("system.kernelExtensionLibraries")
                .map(String::as_str),
            Some("com.apple.iokit.IOUSBHostFamily>=1.2; com.apple.kpi.libkern>=20.0")
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
    fn system_identity_metadata_extracts_macos_wifi_preferences() {
        let metadata = system_identity_metadata_from_bytes(
            "/Library/Preferences/SystemConfiguration/com.apple.airport.preferences.plist",
            br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>KnownNetworks</key>
  <dict>
    <key>wifi.network.ssid.CorpNet</key>
    <dict>
      <key>SSIDString</key><string>CorpNet</string>
      <key>SecurityType</key><string>WPA2 Personal</string>
      <key>AutoJoin</key><true/>
      <key>LastConnected</key><date>2026-06-01T12:34:56Z</date>
    </dict>
    <key>wifi.network.ssid.Guest</key>
    <dict>
      <key>SSIDString</key><string>Guest</string>
      <key>SecurityType</key><string>Open</string>
      <key>AutoJoin</key><false/>
    </dict>
  </dict>
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
                .get("system.wifiKnownNetworkCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata.get("system.wifiSsids").map(String::as_str),
            Some("CorpNet; Guest")
        );
        assert_eq!(
            metadata.get("system.wifiSecurityTypes").map(String::as_str),
            Some("WPA2 Personal; Open")
        );
        assert_eq!(
            metadata.get("system.wifiAutoJoinSsids").map(String::as_str),
            Some("CorpNet")
        );
        assert_eq!(
            metadata.get("system.wifiLastConnected").map(String::as_str),
            Some("CorpNet=2026-06-01T12:34:56Z")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_macos_known_networks_plist() {
        let metadata = system_identity_metadata_from_bytes(
            "/Library/Preferences/com.apple.wifi.known-networks.plist",
            br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>KnownNetworks</key>
  <array>
    <dict>
      <key>SSID</key><data>Q2FzZUxhYg==</data>
      <key>SupportedSecurityTypes</key>
      <array>
        <string>WPA3 Personal</string>
        <string>WPA2 Personal</string>
      </array>
      <key>AutoLogin</key><true/>
    </dict>
  </array>
</dict>
</plist>
"#,
        );

        assert_eq!(
            metadata
                .get("system.wifiKnownNetworkCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            metadata.get("system.wifiSsids").map(String::as_str),
            Some("CaseLab")
        );
        assert_eq!(
            metadata.get("system.wifiSecurityTypes").map(String::as_str),
            Some("WPA3 Personal; WPA2 Personal")
        );
        assert_eq!(
            metadata.get("system.wifiAutoJoinSsids").map(String::as_str),
            Some("CaseLab")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_macos_firewall_preferences() {
        let metadata = system_identity_metadata_from_bytes(
            "/Library/Preferences/com.apple.alf.plist",
            br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>globalstate</key><integer>1</integer>
  <key>stealthenabled</key><true/>
  <key>allowsignedenabled</key><false/>
  <key>loggingenabled</key><true/>
  <key>applications</key>
  <array>
    <dict><key>path</key><string>/Applications/Test.app</string></dict>
    <dict><key>path</key><string>/Applications/Other.app</string></dict>
  </array>
</dict>
</plist>
"#,
        );

        assert_eq!(
            metadata
                .get("system.firewallConfigType")
                .map(String::as_str),
            Some("macos-alf")
        );
        assert_eq!(
            metadata
                .get("system.firewallGlobalState")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            metadata
                .get("system.firewallStealthEnabled")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(
            metadata
                .get("system.firewallAllowSignedEnabled")
                .map(String::as_str),
            Some("false")
        );
        assert_eq!(
            metadata
                .get("system.firewallApplicationRuleCount")
                .map(String::as_str),
            Some("2")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_macos_global_preferences() {
        let metadata = system_identity_metadata_from_bytes(
            "/Library/Preferences/.GlobalPreferences.plist",
            br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>AppleLocale</key><string>en_US</string>
  <key>AppleCountry</key><string>US</string>
  <key>AppleLanguages</key>
  <array>
    <string>en-US</string>
    <string>es-US</string>
  </array>
  <key>AppleMeasurementUnits</key><string>Inches</string>
  <key>AppleTemperatureUnit</key><string>Fahrenheit</string>
</dict>
</plist>
"#,
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata.get("system.locale").map(String::as_str),
            Some("en_US")
        );
        assert_eq!(
            metadata.get("system.languages").map(String::as_str),
            Some("en-US, es-US")
        );
        assert_eq!(
            metadata.get("system.temperatureUnit").map(String::as_str),
            Some("Fahrenheit")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_macos_install_history() {
        let metadata = system_identity_metadata_from_bytes(
            "/Library/Receipts/InstallHistory.plist",
            br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<array>
  <dict>
    <key>date</key><date>2024-01-01T00:00:00Z</date>
    <key>displayName</key><string>macOS Security Response</string>
    <key>displayVersion</key><string>1.0</string>
  </dict>
  <dict>
    <key>date</key><date>2026-06-01T12:34:56Z</date>
    <key>displayName</key><string>macOS Update</string>
    <key>displayVersion</key><string>15.5</string>
    <key>packageIdentifiers</key>
    <array>
      <string>com.apple.pkg.update.os</string>
      <string>com.apple.pkg.update.firmware</string>
    </array>
  </dict>
</array>
</plist>
"#,
        );

        assert_eq!(
            metadata.get("system.identityStatus").map(String::as_str),
            Some("parsed")
        );
        assert_eq!(
            metadata
                .get("system.installHistoryCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata.get("system.latestInstallName").map(String::as_str),
            Some("macOS Update")
        );
        assert_eq!(
            metadata
                .get("system.latestInstallVersion")
                .map(String::as_str),
            Some("15.5")
        );
        assert_eq!(
            metadata
                .get("system.latestInstallPackages")
                .map(String::as_str),
            Some("com.apple.pkg.update.os, com.apple.pkg.update.firmware")
        );
        assert_eq!(
            metadata.get("system.latestInstallDate").map(String::as_str),
            Some("2026-06-01T12:34:56Z")
        );
    }

    #[test]
    fn system_identity_metadata_extracts_macos_disk_management() {
        let metadata = system_identity_metadata_from_bytes(
            "/var/db/DiskManagement.plist",
            br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>AllDisksAndPartitions</key>
  <array>
    <dict>
      <key>VolumeName</key><string>Macintosh HD</string>
      <key>VolumeUUID</key><string>11111111-2222-3333-4444-555555555555</string>
      <key>BSD Name</key><string>disk3s1</string>
      <key>DAVolumeKind</key><string>apfs</string>
      <key>DAVolumePath</key><string>/</string>
      <key>DAMediaSize</key><integer>512000000000</integer>
    </dict>
    <dict>
      <key>DAVolumeName</key><string>Case Data</string>
      <key>DAVolumeUUID</key><string>aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee</string>
      <key>DAMediaBSDName</key><string>disk4s2</string>
      <key>DAMediaContent</key><string>Apple_HFS</string>
      <key>MountPoint</key><string>/Volumes/Case Data</string>
      <key>Size</key><integer>1024</integer>
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
            metadata.get("system.volumeCount").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata.get("system.volumeNames").map(String::as_str),
            Some("Macintosh HD; Case Data")
        );
        assert_eq!(
            metadata.get("system.diskIdentifiers").map(String::as_str),
            Some("disk3s1; disk4s2")
        );
        assert_eq!(
            metadata.get("system.volumeFilesystems").map(String::as_str),
            Some("apfs; Apple_HFS")
        );
        assert_eq!(
            metadata.get("system.volumeMounts").map(String::as_str),
            Some("/; /Volumes/Case Data")
        );
        assert_eq!(
            metadata.get("system.totalVolumeBytes").map(String::as_str),
            Some("512000001024")
        );
        let volumes = metadata
            .get("system.volumes")
            .expect("volume descriptions should be captured");
        assert!(volumes.contains("Macintosh HD (disk3s1, apfs, mounted=/"));
        assert!(volumes.contains("Case Data (disk4s2, Apple_HFS, mounted=/Volumes/Case Data"));
    }

    #[test]
    fn system_identity_source_classifier_matches_known_identity_files() {
        assert!(is_system_identity_source("/Windows/System32/config/SYSTEM"));
        assert!(is_system_identity_source(
            "/Windows/System32/config/SOFTWARE"
        ));
        assert!(is_system_identity_source("/Windows/System32/config/SAM"));
        assert!(is_system_identity_source("/etc/machine-id"));
        assert!(is_system_identity_source("/var/lib/dbus/machine-id"));
        assert!(is_system_identity_source("/etc/machine-info"));
        assert!(is_system_identity_source("/etc/default/locale"));
        assert!(is_system_identity_source("/etc/timezone"));
        assert!(is_system_identity_source("/etc/localtime"));
        assert!(is_system_identity_source("/private/etc/localtime"));
        assert!(is_system_identity_source("/var/db/timezone/localtime"));
        assert!(is_system_identity_source("/etc/fstab"));
        assert!(is_system_identity_source("/etc/mtab"));
        assert!(is_system_identity_source("/etc/network/interfaces"));
        assert!(is_system_identity_source("/etc/resolv.conf"));
        assert!(is_system_identity_source("/private/etc/resolv.conf"));
        assert!(is_system_identity_source("/etc/hosts"));
        assert!(is_system_identity_source("/private/etc/hosts"));
        assert!(is_system_identity_source("/etc/passwd"));
        assert!(is_system_identity_source("/etc/group"));
        assert!(is_system_identity_source("/etc/shadow"));
        assert!(is_system_identity_source("/etc/gshadow"));
        assert!(is_system_identity_source(
            "/private/var/db/dslocal/nodes/Default/users/alice.plist"
        ));
        assert!(is_system_identity_source(
            "/private/var/db/dslocal/nodes/Default/groups/admin.plist"
        ));
        assert!(is_system_identity_source(
            "/etc/NetworkManager/system-connections/Corp WiFi.nmconnection"
        ));
        assert!(is_system_identity_source(
            "/etc/sysconfig/network-scripts/ifcfg-ens192"
        ));
        assert!(is_system_identity_source("/etc/netplan/01-netcfg.yaml"));
        assert!(is_system_identity_source(
            "/ProgramData/Microsoft/Wlansvc/Profiles/Interfaces/{iface}/{profile}.xml"
        ));
        assert!(is_system_identity_source("/home/alice/.bash_history"));
        assert!(is_system_identity_source("/Users/alice/.zsh_history"));
        assert!(is_system_identity_source(
            "/Users/Alice/AppData/Roaming/Microsoft/Windows/PowerShell/PSReadLine/ConsoleHost_history.txt"
        ));
        assert!(is_system_identity_source("/sys/class/dmi/id/product_uuid"));
        assert!(is_system_identity_source(
            "/sys/class/dmi/id/product_version"
        ));
        assert!(is_system_identity_source(
            "/sys/devices/virtual/dmi/id/product_sku"
        ));
        assert!(is_system_identity_source(
            "/sys/devices/virtual/dmi/id/board_asset_tag"
        ));
        assert!(is_system_identity_source("/sys/class/dmi/id/board_vendor"));
        assert!(is_system_identity_source(
            "/sys/devices/virtual/dmi/id/chassis_asset_tag"
        ));
        assert!(is_system_identity_source(
            "/sys/class/dmi/id/chassis_serial"
        ));
        assert!(is_system_identity_source("/sys/class/dmi/id/bios_date"));
        assert!(is_system_identity_source(
            "/System/Library/CoreServices/SystemVersion.plist"
        ));
        assert!(is_system_identity_source("/var/db/DiskManagement.plist"));
        assert!(is_system_identity_source(
            "/case/SystemProfiler/SPHardwareDataType.plist"
        ));
        assert!(is_system_identity_source(
            "/case/IORegistry/IOPlatformExpertDevice.plist"
        ));
        assert!(is_system_identity_source(
            "/Library/Preferences/SystemConfiguration/preferences.plist"
        ));
        assert!(is_system_identity_source(
            "/Library/Preferences/SystemConfiguration/NetworkInterfaces.plist"
        ));
        assert!(is_system_identity_source(
            "/Library/Preferences/SystemConfiguration/com.apple.airport.preferences.plist"
        ));
        assert!(is_system_identity_source(
            "/Library/Preferences/com.apple.wifi.known-networks.plist"
        ));
        assert!(is_system_identity_source(
            "/Library/Preferences/com.apple.alf.plist"
        ));
        assert!(is_system_identity_source("/etc/sysconfig/iptables"));
        assert!(is_system_identity_source("/etc/iptables/rules.v4"));
        assert!(is_system_identity_source(
            "/Windows/System32/LogFiles/Firewall/pfirewall.log"
        ));
        assert!(is_system_identity_source(
            "/Library/Preferences/.GlobalPreferences.plist"
        ));
        assert!(is_system_identity_source(
            "/Library/Receipts/InstallHistory.plist"
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
        assert!(is_windows_registry_identity_source(
            "ad1:/Windows/System32/config/SAM"
        ));
        assert!(!is_windows_registry_identity_source(
            "ad1:/Users/test/NTUSER.DAT"
        ));
    }

    #[test]
    fn windows_sam_account_metadata_summarizes_names_without_hash_values() {
        let metadata = registry_sam_account_metadata_from_names(
            vec![
                "Administrator".to_string(),
                "Guest".to_string(),
                "Alice".to_string(),
            ],
            vec![
                "Administrators".to_string(),
                "Remote Desktop Users".to_string(),
            ],
        );

        assert_eq!(
            metadata.get("system.accountConfigType").map(String::as_str),
            Some("windows-sam")
        );
        assert_eq!(
            metadata.get("system.localUserCount").map(String::as_str),
            Some("3")
        );
        assert_eq!(
            metadata.get("system.localUsers").map(String::as_str),
            Some("Administrator; Guest; Alice")
        );
        assert_eq!(
            metadata
                .get("system.administratorAccountPresent")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(
            metadata
                .get("system.guestAccountPresent")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(
            metadata.get("system.localGroupCount").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata.get("system.adminGroups").map(String::as_str),
            Some("Administrators")
        );
        assert!(!metadata
            .keys()
            .any(|key| key.to_ascii_lowercase().contains("hash")));
    }

    #[test]
    fn windows_profile_list_metadata_summarizes_profile_sids_paths_and_names() {
        let mut values = WindowsProfileListMetadata::default();
        collect_windows_profile_list_metadata(
            &mut values,
            "S-1-5-21-111-222-333-1001",
            Some(r"C:\Users\Alice".to_string()),
        );
        collect_windows_profile_list_metadata(
            &mut values,
            "S-1-5-21-111-222-333-1002",
            Some("C:/Users/Bob".to_string()),
        );
        collect_windows_profile_list_metadata(&mut values, "S-1-5-18", None);

        let metadata = windows_profile_list_metadata_to_map(values);

        assert_eq!(
            metadata.get("system.profileCount").map(String::as_str),
            Some("3")
        );
        assert_eq!(
            metadata.get("system.profileSids").map(String::as_str),
            Some("S-1-5-21-111-222-333-1001; S-1-5-21-111-222-333-1002; S-1-5-18")
        );
        assert_eq!(
            metadata.get("system.profileNames").map(String::as_str),
            Some("Alice; Bob")
        );
        assert_eq!(
            metadata.get("system.profilePaths").map(String::as_str),
            Some(r"C:\Users\Alice; C:/Users/Bob")
        );
        let profiles = metadata
            .get("system.profiles")
            .expect("profile descriptions should be captured");
        assert!(profiles.contains(r"Alice (S-1-5-21-111-222-333-1001, C:\Users\Alice)"));
        assert!(profiles.contains("Bob (S-1-5-21-111-222-333-1002, C:/Users/Bob)"));
        assert!(profiles.contains("S-1-5-18"));
    }

    #[test]
    fn windows_profile_name_from_path_handles_windows_and_normalized_paths() {
        assert_eq!(
            windows_profile_name_from_path(r"C:\Users\Alice").as_deref(),
            Some("Alice")
        );
        assert_eq!(
            windows_profile_name_from_path("/Users/Bob/").as_deref(),
            Some("Bob")
        );
        assert_eq!(windows_profile_name_from_path("").as_deref(), None);
    }

    #[test]
    fn windows_network_list_metadata_summarizes_profiles_and_categories() {
        let mut values = WindowsNetworkListMetadata::default();
        collect_windows_network_list_metadata(
            &mut values,
            Some("Corp WiFi".to_string()),
            Some("Managed network".to_string()),
            Some("domain-authenticated".to_string()),
        );
        collect_windows_network_list_metadata(
            &mut values,
            Some("Guest".to_string()),
            None,
            Some("public".to_string()),
        );
        collect_windows_network_list_metadata(
            &mut values,
            None,
            Some("Ethernet 2".to_string()),
            Some("private".to_string()),
        );

        let metadata = windows_network_list_metadata_to_map(values);

        assert_eq!(
            metadata
                .get("system.networkProfileCount")
                .map(String::as_str),
            Some("3")
        );
        assert_eq!(
            metadata
                .get("system.networkProfileNames")
                .map(String::as_str),
            Some("Corp WiFi; Guest")
        );
        assert_eq!(
            metadata
                .get("system.networkProfileCategories")
                .map(String::as_str),
            Some("domain-authenticated; public; private")
        );
        assert_eq!(
            metadata.get("system.networkProfiles").map(String::as_str),
            Some("Corp WiFi (domain-authenticated); Guest (public); Ethernet 2 (private)")
        );
    }

    #[test]
    fn windows_network_category_name_maps_registry_values() {
        assert_eq!(
            windows_network_category_name(CellValue::U32(0)).as_deref(),
            Some("public")
        );
        assert_eq!(
            windows_network_category_name(CellValue::I32(1)).as_deref(),
            Some("private")
        );
        assert_eq!(
            windows_network_category_name(CellValue::U32(2)).as_deref(),
            Some("domain-authenticated")
        );
        assert_eq!(
            windows_network_category_name(CellValue::U32(9)).as_deref(),
            Some("category-9")
        );
    }

    #[test]
    fn windows_driver_service_metadata_summarizes_driver_services() {
        let mut values = WindowsDriverServiceMetadata::default();
        collect_windows_driver_service_metadata(
            &mut values,
            "storflt",
            Some("Contoso Storage Filter".to_string()),
            Some(r"\SystemRoot\System32\drivers\storflt.sys".to_string()),
            Some("FSFilter Activity Monitor".to_string()),
            Some("boot".to_string()),
            Some("file-system-driver".to_string()),
        );
        collect_windows_driver_service_metadata(
            &mut values,
            "ndiswan",
            None,
            Some(r"System32\Drivers\ndiswan.sys".to_string()),
            Some("NDIS".to_string()),
            Some("system".to_string()),
            Some("kernel-driver".to_string()),
        );

        let metadata = windows_driver_service_metadata_to_map(values);

        assert_eq!(
            metadata
                .get("system.driverServiceCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata.get("system.driverServices").map(String::as_str),
            Some("storflt; ndiswan")
        );
        assert_eq!(
            metadata.get("system.driverImagePaths").map(String::as_str),
            Some(r"\SystemRoot\System32\drivers\storflt.sys; System32\Drivers\ndiswan.sys")
        );
        assert_eq!(
            metadata.get("system.driverGroups").map(String::as_str),
            Some("FSFilter Activity Monitor; NDIS")
        );
        assert_eq!(
            metadata.get("system.driverStartTypes").map(String::as_str),
            Some("boot; system")
        );
        assert_eq!(
            metadata
                .get("system.driverServiceDetails")
                .map(String::as_str),
            Some(
                r"Contoso Storage Filter (file-system-driver; start=boot; group=FSFilter Activity Monitor; image=\SystemRoot\System32\drivers\storflt.sys); ndiswan (kernel-driver; start=system; group=NDIS; image=System32\Drivers\ndiswan.sys)"
            )
        );
    }

    #[test]
    fn windows_driver_service_helpers_map_registry_values_and_paths() {
        assert_eq!(
            windows_driver_service_type_name(CellValue::U32(1)).as_deref(),
            Some("kernel-driver")
        );
        assert_eq!(
            windows_driver_service_type_name(CellValue::I32(2)).as_deref(),
            Some("file-system-driver")
        );
        assert_eq!(
            windows_driver_service_type_name(CellValue::U32(16)).as_deref(),
            None
        );
        assert_eq!(
            windows_service_start_name(CellValue::String("0x2".to_string())).as_deref(),
            Some("auto")
        );
        assert_eq!(
            windows_service_start_name(CellValue::U32(4)).as_deref(),
            Some("disabled")
        );
        assert!(is_windows_driver_image_path(
            r"\SystemRoot\System32\drivers\kbdclass.sys"
        ));
        assert!(!is_windows_driver_image_path(
            r"\SystemRoot\System32\svchost.exe"
        ));
    }

    #[test]
    fn windows_mounted_devices_metadata_summarizes_drive_letters_and_volume_guids() {
        let mut values = WindowsMountedDevicesMetadata::default();
        collect_windows_mounted_device_name(&mut values, r"\DosDevices\c:");
        collect_windows_mounted_device_name(
            &mut values,
            r"\??\Volume{12345678-1234-1234-1234-123456789abc}",
        );
        collect_windows_mounted_device_name(&mut values, r"\Device\HarddiskVolumeShadowCopy1");

        let metadata = windows_mounted_devices_metadata_to_map(values);

        assert_eq!(
            metadata
                .get("system.mountedDeviceCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            metadata.get("system.driveLetters").map(String::as_str),
            Some("C:")
        );
        assert_eq!(
            metadata.get("system.volumeGuids").map(String::as_str),
            Some("Volume{12345678-1234-1234-1234-123456789abc}")
        );
        assert_eq!(
            metadata.get("system.mountedDevices").map(String::as_str),
            Some("drive C:; Volume{12345678-1234-1234-1234-123456789abc}")
        );
    }

    #[test]
    fn windows_mounted_device_helpers_validate_drive_and_volume_names() {
        assert!(is_windows_drive_letter("C:"));
        assert!(is_windows_drive_letter("z:"));
        assert!(!is_windows_drive_letter("CD:"));
        assert!(!is_windows_drive_letter("1:"));
        assert_eq!(
            extract_windows_volume_guid_name(r"\??\Volume{aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee}\")
                .as_deref(),
            Some("Volume{aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee}")
        );
        assert_eq!(
            extract_windows_volume_guid_name(r"\Device\Harddisk0").as_deref(),
            None
        );
    }

    #[test]
    fn windows_registry_identity_mappings_cover_os_provenance_values() {
        assert!(WINDOWS_SOFTWARE_CURRENT_VERSION_REGISTRY_VALUES
            .iter()
            .any(|mapping| mapping.value_name == "UBR"
                && mapping.metadata_key == "system.osUpdateBuildRevision"));
        assert!(WINDOWS_SOFTWARE_CURRENT_VERSION_REGISTRY_VALUES
            .iter()
            .any(|mapping| mapping.value_name == "BuildLabEx"
                && mapping.metadata_key == "system.osBuildLabExtended"));
        assert!(WINDOWS_SOFTWARE_CURRENT_VERSION_REGISTRY_VALUES
            .iter()
            .any(|mapping| mapping.value_name == "InstallDate"
                && mapping.metadata_key == "system.osInstallDateEpoch"));
        assert!(WINDOWS_SOFTWARE_CURRENT_VERSION_REGISTRY_VALUES
            .iter()
            .any(|mapping| mapping.value_name == "SystemRoot"
                && mapping.metadata_key == "system.systemRoot"));
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
        assert_eq!(
            registry_value_text(CellValue::U64(1_713_312_000)).as_deref(),
            Some("1713312000")
        );
        assert_eq!(
            registry_value_texts(CellValue::MultiString(vec![
                "192.0.2.10".to_string(),
                "192.0.2.11".to_string()
            ])),
            vec!["192.0.2.10".to_string(), "192.0.2.11".to_string()]
        );
        assert!(registry_value_text(CellValue::Binary(vec![1, 2, 3])).is_none());
    }

    #[test]
    fn windows_tcpip_interface_metadata_flattens_network_values() {
        let mut values = WindowsTcpipInterfaceMetadata::default();
        push_unique_limited(
            &mut values.interfaces,
            "{12345678-1234-1234-1234-123456789abc}".to_string(),
        );
        for value in split_windows_registry_network_list("192.0.2.10 0.0.0.0,192.0.2.11") {
            if is_useful_windows_network_value(&value) {
                push_unique_limited(&mut values.addresses, value);
            }
        }
        for value in split_windows_registry_network_list("192.0.2.1; 255.255.255.255") {
            if is_useful_windows_network_value(&value) {
                push_unique_limited(&mut values.gateways, value);
            }
        }
        for value in split_windows_registry_network_list("1.1.1.1 8.8.8.8") {
            if is_useful_windows_network_value(&value) {
                push_unique_limited(&mut values.dns_servers, value);
            }
        }
        push_unique_limited(&mut values.domains, "corp.example".to_string());
        push_unique_limited(&mut values.dhcp_servers, "192.0.2.254".to_string());
        push_unique_limited(
            &mut values.descriptions,
            "{12345678-1234-1234-1234-123456789abc} (ip=192.0.2.10,192.0.2.11; gateway=192.0.2.1; dns=1.1.1.1,8.8.8.8)".to_string(),
        );

        let metadata = windows_tcpip_interface_metadata_to_map(values);

        assert_eq!(
            metadata.get("system.networkInterfaces").map(String::as_str),
            Some("{12345678-1234-1234-1234-123456789abc}")
        );
        assert_eq!(
            metadata.get("system.ipv4Addresses").map(String::as_str),
            Some("192.0.2.10; 192.0.2.11")
        );
        assert_eq!(
            metadata.get("system.gateways").map(String::as_str),
            Some("192.0.2.1")
        );
        assert_eq!(
            metadata.get("system.dnsServers").map(String::as_str),
            Some("1.1.1.1; 8.8.8.8")
        );
        assert_eq!(
            metadata.get("system.networkDomains").map(String::as_str),
            Some("corp.example")
        );
        assert_eq!(
            metadata.get("system.dhcpServers").map(String::as_str),
            Some("192.0.2.254")
        );
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
