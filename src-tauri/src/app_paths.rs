// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

use std::path::{Path, PathBuf};

pub const APP_DATA_DIR_NAME: &str = "com.core-ffx.desktop";
pub const GLOBAL_DB_FILENAME: &str = "ffx.db";
pub const AUDIT_LOG_DIR_NAME: &str = "core-ffx";
pub const AUDIT_LOG_BASENAME: &str = "ffx-audit";
pub const AUDIT_LOG_SUFFIX: &str = "log";
pub const PREVIEW_TEMP_DIR_NAME: &str = "core-ffx-preview";
pub const THUMBNAIL_TEMP_DIR_NAME: &str = "core-ffx-thumbnails";
pub const NESTED_TEMP_DIR_NAME: &str = "core-ffx-nested";

const LEGACY_SHARED_APP_DATA_DIR_NAME: &str = "com.ffxcheck.app";
const LEGACY_AUDIT_LOG_BASENAME: &str = "ffx-audit.log";

fn base_local_data_dir() -> PathBuf {
    dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub fn app_data_dir() -> PathBuf {
    base_local_data_dir().join(APP_DATA_DIR_NAME)
}

pub fn legacy_shared_db_path() -> PathBuf {
    base_local_data_dir()
        .join(LEGACY_SHARED_APP_DATA_DIR_NAME)
        .join(GLOBAL_DB_FILENAME)
}

pub fn global_db_path() -> PathBuf {
    if let Some(config) = crate::commands::portable::get_config() {
        PathBuf::from(&config.config_dir).join(GLOBAL_DB_FILENAME)
    } else {
        app_data_dir().join(GLOBAL_DB_FILENAME)
    }
}

pub fn global_audit_log_dir() -> PathBuf {
    if let Some(config) = crate::commands::portable::get_config() {
        PathBuf::from(&config.log_dir)
    } else {
        base_local_data_dir().join(AUDIT_LOG_DIR_NAME).join("logs")
    }
}

pub fn is_global_audit_log_filename(name: &str) -> bool {
    (name.starts_with(&format!("{AUDIT_LOG_BASENAME}."))
        && name.ends_with(&format!(".{AUDIT_LOG_SUFFIX}")))
        || name.starts_with(LEGACY_AUDIT_LOG_BASENAME)
}

pub fn audit_log_filename_for_date(date: &str) -> String {
    format!("{AUDIT_LOG_BASENAME}.{date}.{AUDIT_LOG_SUFFIX}")
}

pub fn support_bundle_audit_log_name(name: &str) -> String {
    if let Some(date) = name.strip_prefix(&format!("{LEGACY_AUDIT_LOG_BASENAME}.")) {
        audit_log_filename_for_date(date)
    } else {
        name.to_string()
    }
}

pub fn sqlite_sidecar_path(path: &Path, suffix: &str) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| GLOBAL_DB_FILENAME.to_string());
    path.with_file_name(format!("{file_name}{suffix}"))
}
