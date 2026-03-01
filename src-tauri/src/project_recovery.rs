// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Project recovery, backup, and version history system.
//!
//! Provides:
//! - Automatic backup on save (.cffx.backup)
//! - Version history tracking (.cffx.versions/)
//! - Crash recovery from autosave
//! - Project health validation

use crate::project::FFXProject;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Maximum number of backup versions to keep
pub const MAX_BACKUP_VERSIONS: usize = 10;

/// Backup file suffix
pub const BACKUP_SUFFIX: &str = ".backup";

/// Autosave file suffix (for crash recovery)
pub const AUTOSAVE_SUFFIX: &str = ".autosave";

/// Version directory name
pub const VERSION_DIR: &str = ".cffx.versions";

/// Project backup metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Original project path
    pub original_path: String,
    /// Backup creation timestamp
    pub created_at: String,
    /// App version that created backup
    pub app_version: String,
    /// File size in bytes
    pub file_size: u64,
    /// Backup reason (save, autosave, manual)
    pub backup_type: BackupType,
    /// User who triggered backup
    pub user: Option<String>,
}

/// Type of backup
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BackupType {
    /// Manual save backup
    ManualSave,
    /// Auto-save backup
    AutoSave,
    /// Manual backup request
    ManualBackup,
    /// Pre-operation backup (before risky changes)
    PreOperation,
}

/// Backup file with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFile {
    /// Path to backup file
    pub path: PathBuf,
    /// Backup metadata
    pub metadata: BackupMetadata,
}

/// Project health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHealth {
    /// Overall health status
    pub status: HealthStatus,
    /// Detected issues
    pub issues: Vec<HealthIssue>,
    /// Health check timestamp
    pub checked_at: String,
    /// Project file size
    pub file_size: u64,
    /// Activity log size
    pub activity_log_size: usize,
    /// Number of tabs
    pub tab_count: usize,
    /// Number of sessions
    pub session_count: usize,
    /// Whether backup exists
    pub has_backup: bool,
    /// Number of version history files
    pub version_count: usize,
}

/// Health status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// Project is healthy
    Healthy,
    /// Minor issues detected
    Warning,
    /// Serious issues detected
    Critical,
}

/// Health issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    /// Issue severity
    pub severity: IssueSeverity,
    /// Issue category
    pub category: IssueCategory,
    /// Issue description
    pub message: String,
    /// Recommendation to fix
    pub recommendation: Option<String>,
}

/// Issue severity
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Issue category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssueCategory {
    FileSize,
    ActivityLog,
    MissingFiles,
    Corruption,
    Performance,
    Security,
}

/// Recovery information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryInfo {
    /// Autosave file exists
    pub has_autosave: bool,
    /// Autosave path
    pub autosave_path: Option<String>,
    /// Autosave age in seconds
    pub autosave_age_seconds: Option<u64>,
    /// Whether autosave is newer than saved project
    pub autosave_is_newer: bool,
    /// Backup file exists
    pub has_backup: bool,
    /// Backup path
    pub backup_path: Option<String>,
}

// =============================================================================
// BACKUP OPERATIONS
// =============================================================================

/// Create a backup of the project file
pub fn create_backup(
    project_path: &Path,
    backup_type: BackupType,
    user: Option<String>,
) -> Result<PathBuf, String> {
    if !project_path.exists() {
        return Err("Project file does not exist".to_string());
    }

    // Read project file
    let content =
        fs::read_to_string(project_path).map_err(|e| format!("Failed to read project: {}", e))?;

    let project: FFXProject =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse project: {}", e))?;

    // Determine backup path
    let backup_path = match backup_type {
        BackupType::AutoSave => {
            let mut path = project_path.to_path_buf();
            path.set_extension(format!("cffx{}", AUTOSAVE_SUFFIX));
            path
        }
        _ => {
            let mut path = project_path.to_path_buf();
            path.set_extension(format!("cffx{}", BACKUP_SUFFIX));
            path
        }
    };

    // Create metadata
    let metadata = BackupMetadata {
        original_path: project_path.to_string_lossy().to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        file_size: content.len() as u64,
        backup_type,
        user: user.or(project.current_user.clone()),
    };

    // Write backup
    fs::write(&backup_path, &content).map_err(|e| format!("Failed to write backup: {}", e))?;

    // Write metadata
    let metadata_path = backup_path.with_extension("cffx.backup.meta");
    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    fs::write(&metadata_path, metadata_json)
        .map_err(|e| format!("Failed to write metadata: {}", e))?;

    info!(
        "Created {:?} backup: {}",
        backup_type,
        backup_path.display()
    );

    Ok(backup_path)
}

/// Create a versioned backup
pub fn create_version_backup(project_path: &Path) -> Result<PathBuf, String> {
    if !project_path.exists() {
        return Err("Project file does not exist".to_string());
    }

    // Create version directory
    let version_dir = project_path
        .parent()
        .ok_or("Invalid project path")?
        .join(VERSION_DIR);
    fs::create_dir_all(&version_dir)
        .map_err(|e| format!("Failed to create version directory: {}", e))?;

    // Create timestamped version file
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = project_path
        .file_stem()
        .ok_or("Invalid project filename")?
        .to_string_lossy();
    let version_path = version_dir.join(format!("{}_{}.cffx", filename, timestamp));

    // Copy to version
    fs::copy(project_path, &version_path)
        .map_err(|e| format!("Failed to create version: {}", e))?;

    // Clean up old versions
    cleanup_old_versions(&version_dir, MAX_BACKUP_VERSIONS)?;

    info!("Created version backup: {}", version_path.display());

    Ok(version_path)
}

/// List all version backups for a project
pub fn list_version_backups(project_path: &Path) -> Result<Vec<BackupFile>, String> {
    let version_dir = project_path
        .parent()
        .ok_or("Invalid project path")?
        .join(VERSION_DIR);

    if !version_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();

    let entries = fs::read_dir(&version_dir)
        .map_err(|e| format!("Failed to read version directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("cffx") {
            // Try to load metadata
            let metadata_path = path.with_extension("cffx.meta");
            let metadata = if metadata_path.exists() {
                match fs::read_to_string(&metadata_path) {
                    Ok(content) => serde_json::from_str(&content).ok(),
                    Err(_) => None,
                }
            } else {
                None
            };

            // Create default metadata if not found
            let metadata = metadata.unwrap_or_else(|| {
                let modified = entry.metadata().ok().and_then(|m| m.modified().ok());
                BackupMetadata {
                    original_path: project_path.to_string_lossy().to_string(),
                    created_at: modified
                        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                    app_version: "unknown".to_string(),
                    file_size: entry.metadata().ok().map(|m| m.len()).unwrap_or(0),
                    backup_type: BackupType::ManualBackup,
                    user: None,
                }
            });

            backups.push(BackupFile { path, metadata });
        }
    }

    // Sort by creation time (newest first)
    backups.sort_by(|a, b| b.metadata.created_at.cmp(&a.metadata.created_at));

    Ok(backups)
}

/// Clean up old version backups
fn cleanup_old_versions(version_dir: &Path, max_versions: usize) -> Result<(), String> {
    let mut entries: Vec<_> = fs::read_dir(version_dir)
        .map_err(|e| format!("Failed to read version directory: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| ext == "cffx")
                .unwrap_or(false)
        })
        .collect();

    if entries.len() <= max_versions {
        return Ok(());
    }

    // Sort by modification time (oldest first)
    entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());

    // Remove oldest entries
    let to_remove = entries.len() - max_versions;
    for entry in entries.iter().take(to_remove) {
        let path = entry.path();
        if let Err(e) = fs::remove_file(&path) {
            warn!("Failed to remove old version {}: {}", path.display(), e);
        } else {
            info!("Removed old version: {}", path.display());
        }

        // Also remove metadata file
        let metadata_path = path.with_extension("cffx.meta");
        if metadata_path.exists() {
            let _ = fs::remove_file(&metadata_path);
        }
    }

    Ok(())
}

// =============================================================================
// RECOVERY OPERATIONS
// =============================================================================

/// Check if recovery is available for a project
pub fn check_recovery(project_path: &Path) -> RecoveryInfo {
    let autosave_path = {
        let mut path = project_path.to_path_buf();
        path.set_extension(format!("cffx{}", AUTOSAVE_SUFFIX));
        path
    };

    let backup_path = {
        let mut path = project_path.to_path_buf();
        path.set_extension(format!("cffx{}", BACKUP_SUFFIX));
        path
    };

    let has_autosave = autosave_path.exists();
    let has_backup = backup_path.exists();

    let (autosave_age_seconds, autosave_is_newer) = if has_autosave {
        let autosave_modified = fs::metadata(&autosave_path).and_then(|m| m.modified()).ok();
        let project_modified = fs::metadata(project_path).and_then(|m| m.modified()).ok();

        let age = autosave_modified.and_then(|t| t.elapsed().ok().map(|d| d.as_secs()));

        let is_newer = match (autosave_modified, project_modified) {
            (Some(auto), Some(proj)) => auto > proj,
            _ => false,
        };

        (age, is_newer)
    } else {
        (None, false)
    };

    RecoveryInfo {
        has_autosave,
        autosave_path: if has_autosave {
            Some(autosave_path.to_string_lossy().to_string())
        } else {
            None
        },
        autosave_age_seconds,
        autosave_is_newer,
        has_backup,
        backup_path: if has_backup {
            Some(backup_path.to_string_lossy().to_string())
        } else {
            None
        },
    }
}

/// Recover project from autosave
pub fn recover_from_autosave(project_path: &Path) -> Result<FFXProject, String> {
    let autosave_path = {
        let mut path = project_path.to_path_buf();
        path.set_extension(format!("cffx{}", AUTOSAVE_SUFFIX));
        path
    };

    if !autosave_path.exists() {
        return Err("No autosave file found".to_string());
    }

    let content = fs::read_to_string(&autosave_path)
        .map_err(|e| format!("Failed to read autosave: {}", e))?;

    let project: FFXProject =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse autosave: {}", e))?;

    info!(
        "Recovered project from autosave: {}",
        autosave_path.display()
    );

    Ok(project)
}

/// Delete autosave file after successful save
pub fn clear_autosave(project_path: &Path) -> Result<(), String> {
    let autosave_path = {
        let mut path = project_path.to_path_buf();
        path.set_extension(format!("cffx{}", AUTOSAVE_SUFFIX));
        path
    };

    if autosave_path.exists() {
        fs::remove_file(&autosave_path).map_err(|e| format!("Failed to remove autosave: {}", e))?;
        info!("Cleared autosave: {}", autosave_path.display());
    }

    Ok(())
}

// =============================================================================
// HEALTH CHECKING
// =============================================================================

/// Check project health
pub fn check_project_health(project_path: &Path) -> Result<ProjectHealth, String> {
    if !project_path.exists() {
        return Err("Project file does not exist".to_string());
    }

    let content =
        fs::read_to_string(project_path).map_err(|e| format!("Failed to read project: {}", e))?;

    let project: FFXProject =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse project: {}", e))?;

    let file_size = content.len() as u64;
    let activity_log_size = project.activity_log.len();
    let tab_count = project.tabs.len();
    let session_count = project.sessions.len();

    // Check for backup
    let backup_path = {
        let mut path = project_path.to_path_buf();
        path.set_extension(format!("cffx{}", BACKUP_SUFFIX));
        path
    };
    let has_backup = backup_path.exists();

    // Check version count
    let version_count = list_version_backups(project_path)?.len();

    // Detect issues
    let mut issues = Vec::new();

    // Large file size (> 10 MB)
    if file_size > 10 * 1024 * 1024 {
        issues.push(HealthIssue {
            severity: IssueSeverity::Warning,
            category: IssueCategory::FileSize,
            message: format!("Large project file: {} MB", file_size / (1024 * 1024)),
            recommendation: Some("Consider archiving old activity logs or sessions".to_string()),
        });
    }

    // Large activity log (> 5000 entries)
    if activity_log_size > 5000 {
        issues.push(HealthIssue {
            severity: IssueSeverity::Warning,
            category: IssueCategory::ActivityLog,
            message: format!("Large activity log: {} entries", activity_log_size),
            recommendation: Some("Consider trimming activity log to recent entries".to_string()),
        });
    }

    // Very large activity log (> 10000 entries)
    if activity_log_size > 10000 {
        issues.push(HealthIssue {
            severity: IssueSeverity::Error,
            category: IssueCategory::Performance,
            message: format!("Very large activity log: {} entries", activity_log_size),
            recommendation: Some("Trim activity log to improve performance".to_string()),
        });
    }

    // No backup exists
    if !has_backup {
        issues.push(HealthIssue {
            severity: IssueSeverity::Info,
            category: IssueCategory::Security,
            message: "No backup file found".to_string(),
            recommendation: Some("Create a backup for safety".to_string()),
        });
    }

    // Many open tabs (> 20)
    if tab_count > 20 {
        issues.push(HealthIssue {
            severity: IssueSeverity::Info,
            category: IssueCategory::Performance,
            message: format!("Many open tabs: {}", tab_count),
            recommendation: Some("Consider closing unused tabs".to_string()),
        });
    }

    // Determine overall status
    let status = if issues
        .iter()
        .any(|i| matches!(i.severity, IssueSeverity::Critical))
    {
        HealthStatus::Critical
    } else if issues
        .iter()
        .any(|i| matches!(i.severity, IssueSeverity::Error | IssueSeverity::Warning))
    {
        HealthStatus::Warning
    } else {
        HealthStatus::Healthy
    };

    Ok(ProjectHealth {
        status,
        issues,
        checked_at: chrono::Utc::now().to_rfc3339(),
        file_size,
        activity_log_size,
        tab_count,
        session_count,
        has_backup,
        version_count,
    })
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_metadata_serialization() {
        let metadata = BackupMetadata {
            original_path: "/test/project.cffx".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            app_version: "1.0.0".to_string(),
            file_size: 1024,
            backup_type: BackupType::ManualSave,
            user: Some("tester".to_string()),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: BackupMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.original_path, metadata.original_path);
        assert_eq!(deserialized.backup_type, BackupType::ManualSave);
    }

    #[test]
    fn test_project_health_defaults() {
        let health = ProjectHealth {
            status: HealthStatus::Healthy,
            issues: Vec::new(),
            checked_at: chrono::Utc::now().to_rfc3339(),
            file_size: 1024,
            activity_log_size: 10,
            tab_count: 3,
            session_count: 1,
            has_backup: true,
            version_count: 5,
        };

        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.issues.is_empty());
    }

    #[test]
    fn test_recovery_info_defaults() {
        let recovery = RecoveryInfo {
            has_autosave: false,
            autosave_path: None,
            autosave_age_seconds: None,
            autosave_is_newer: false,
            has_backup: false,
            backup_path: None,
        };

        assert!(!recovery.has_autosave);
        assert!(!recovery.has_backup);
    }
}
