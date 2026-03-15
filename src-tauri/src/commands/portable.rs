// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Portable mode detection and path management.
//!
//! When CORE Acquire runs from removable media (USB, SD card, etc.) or when a
//! `portable.marker` file exists next to the executable, the app enters
//! **portable mode** — all writes (cache, config, temp, logs) are redirected to
//! the same removable volume, producing zero footprint on the host system.
//!
//! ## Detection (in priority order)
//!
//! 1. **Marker file**: `portable.marker` adjacent to the executable
//! 2. **Removable media**: executable resides on a removable drive (USB, SD)
//!
//! ## Path Redirection
//!
//! In portable mode, all mutable paths resolve under `<exe_dir>/CoreAcquireData/`:
//!
//! ```text
//! /Volumes/MyUSB/CORE-Acquisition.app/
//! /Volumes/MyUSB/CoreAcquireData/
//!   ├── config/      # Preferences, profiles
//!   ├── cache/       # WebView2 cache, preview cache
//!   ├── temp/        # Temporary extraction files
//!   ├── logs/        # Audit logs, session logs
//!   └── projects/    # Default project output location
//! ```

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tracing::info;

/// Data directory name created alongside the executable in portable mode.
const PORTABLE_DATA_DIR: &str = "CoreAcquireData";

/// Marker file name — if present next to the exe, forces portable mode.
const PORTABLE_MARKER: &str = "portable.marker";

/// Cached portable mode configuration (computed once at startup).
static PORTABLE_CONFIG: OnceLock<Option<PortableConfig>> = OnceLock::new();

/// Configuration for portable mode path redirection.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableConfig {
    /// Root directory for all portable data (e.g. `/Volumes/MyUSB/CoreAcquireData`)
    pub data_dir: String,
    /// Configuration storage
    pub config_dir: String,
    /// Cache storage (WebView2, preview)
    pub cache_dir: String,
    /// Temporary files (extraction, preview)
    pub temp_dir: String,
    /// Audit and session logs
    pub log_dir: String,
    /// Default project output directory
    pub projects_dir: String,
    /// How portable mode was detected
    pub detection_reason: String,
    /// Mount point of the volume the executable resides on
    pub volume_mount_point: String,
    /// Whether the volume has enough free space (> 100 MB)
    pub has_sufficient_space: bool,
    /// Free space on the portable volume in bytes
    pub free_space_bytes: u64,
}

/// Portable mode status returned to the frontend.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableStatus {
    /// Whether the app is running in portable mode
    pub is_portable: bool,
    /// Configuration details (None if not portable)
    pub config: Option<PortableConfig>,
}

// =============================================================================
// Detection Logic
// =============================================================================

/// Resolve the directory containing the current executable.
///
/// On macOS, if inside a .app bundle, walks up to the .app's parent directory
/// so that the data dir is created alongside the .app bundle, not buried inside
/// `Contents/MacOS/`.
fn resolve_exe_dir() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;

    // macOS .app bundle: walk up from Contents/MacOS/ to the .app parent
    #[cfg(target_os = "macos")]
    {
        let path_str = exe_dir.to_string_lossy();
        if path_str.contains(".app/Contents/MacOS") {
            // Walk up to find the .app directory, then its parent
            let mut current = exe_dir.to_path_buf();
            while let Some(parent) = current.parent() {
                if current
                    .file_name()
                    .map(|n| n.to_string_lossy().ends_with(".app"))
                    .unwrap_or(false)
                {
                    return Some(parent.to_path_buf());
                }
                current = parent.to_path_buf();
            }
        }
    }

    Some(exe_dir.to_path_buf())
}

/// Check if a `portable.marker` file exists next to the executable.
fn has_marker_file(exe_dir: &Path) -> bool {
    exe_dir.join(PORTABLE_MARKER).exists()
}

/// Check if the executable directory resides on removable media.
///
/// Uses `sysinfo::Disks` to enumerate volumes and checks the `is_removable()`
/// flag for the mount point that contains the exe directory.
fn is_on_removable_media(exe_dir: &Path) -> Option<String> {
    use sysinfo::Disks;

    let disks = Disks::new_with_refreshed_list();
    let exe_str = exe_dir.to_string_lossy();

    let mut best_mount = String::new();
    let mut best_len = 0usize;
    let mut best_removable = false;

    for disk in disks.iter() {
        let mount = disk.mount_point().to_string_lossy().into_owned();
        if exe_str.starts_with(&mount) && mount.len() > best_len {
            best_len = mount.len();
            best_mount = mount;
            best_removable = disk.is_removable();
        }
    }

    if best_removable {
        Some(best_mount)
    } else {
        None
    }
}

/// Get free space for a given path using platform-native APIs.
fn get_free_space(path: &Path) -> u64 {
    #[cfg(unix)]
    {
        use std::ffi::CString;
        let c_path = match CString::new(path.to_string_lossy().as_bytes()) {
            Ok(p) => p,
            Err(_) => return 0,
        };
        unsafe {
            let mut stat: libc::statvfs = std::mem::zeroed();
            if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
                stat.f_bavail as u64 * stat.f_frsize as u64
            } else {
                0
            }
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

        let wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut free_bytes: u64 = 0;
        unsafe {
            GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free_bytes as *mut u64,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
        }
        free_bytes
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        0
    }
}

/// Detect portable mode and build the configuration.
///
/// Called once at startup. The result is cached in `PORTABLE_CONFIG`.
fn detect_portable_mode() -> Option<PortableConfig> {
    let exe_dir = resolve_exe_dir()?;

    // Priority 1: Marker file
    let detection_reason;
    let volume_mount_point;

    if has_marker_file(&exe_dir) {
        detection_reason = "portable.marker file found".to_string();
        // Still try to find the mount point for space checks
        volume_mount_point = is_on_removable_media(&exe_dir).unwrap_or_else(|| {
            // Not on removable, but marker forces portable mode
            exe_dir.to_string_lossy().into_owned()
        });
    } else if let Some(mount) = is_on_removable_media(&exe_dir) {
        detection_reason = format!("executable on removable volume {}", mount);
        volume_mount_point = mount;
    } else {
        // Not portable
        return None;
    }

    let data_dir = exe_dir.join(PORTABLE_DATA_DIR);

    let free_space = get_free_space(&exe_dir);
    let has_sufficient_space = free_space > 100 * 1024 * 1024; // 100 MB minimum

    Some(PortableConfig {
        data_dir: data_dir.to_string_lossy().into_owned(),
        config_dir: data_dir.join("config").to_string_lossy().into_owned(),
        cache_dir: data_dir.join("cache").to_string_lossy().into_owned(),
        temp_dir: data_dir.join("temp").to_string_lossy().into_owned(),
        log_dir: data_dir.join("logs").to_string_lossy().into_owned(),
        projects_dir: data_dir.join("projects").to_string_lossy().into_owned(),
        detection_reason,
        volume_mount_point,
        has_sufficient_space,
        free_space_bytes: free_space,
    })
}

// =============================================================================
// Initialization
// =============================================================================

/// Initialize portable mode detection. Call once during app setup.
///
/// Creates the data directory structure if portable mode is detected.
pub fn init_portable_mode() {
    let config = PORTABLE_CONFIG.get_or_init(detect_portable_mode);

    if let Some(cfg) = config {
        info!(
            reason = %cfg.detection_reason,
            data_dir = %cfg.data_dir,
            free_space_mb = cfg.free_space_bytes / (1024 * 1024),
            "Portable mode ACTIVE"
        );

        // Create directory structure
        let dirs = [
            &cfg.config_dir,
            &cfg.cache_dir,
            &cfg.temp_dir,
            &cfg.log_dir,
            &cfg.projects_dir,
        ];
        for dir in &dirs {
            if let Err(e) = std::fs::create_dir_all(dir) {
                tracing::warn!(dir = %dir, error = %e, "Failed to create portable directory");
            }
        }
    } else {
        info!("Portable mode: inactive (installed mode)");
    }
}

/// Returns `true` if the app is running in portable mode.
pub fn is_portable() -> bool {
    PORTABLE_CONFIG
        .get()
        .map(|c| c.is_some())
        .unwrap_or(false)
}

/// Get the portable configuration, if active.
pub fn get_config() -> Option<&'static PortableConfig> {
    PORTABLE_CONFIG.get().and_then(|c| c.as_ref())
}

/// Get the portable temp directory, or fall back to system temp.
pub fn portable_temp_dir() -> PathBuf {
    get_config()
        .map(|c| PathBuf::from(&c.temp_dir))
        .unwrap_or_else(std::env::temp_dir)
}

/// Get the portable cache directory, or fall back to `None` (use default).
pub fn portable_cache_dir() -> Option<PathBuf> {
    get_config().map(|c| PathBuf::from(&c.cache_dir))
}

// =============================================================================
// Tauri Commands
// =============================================================================

/// Query whether the app is running in portable mode.
#[tauri::command]
pub fn portable_get_status() -> PortableStatus {
    let config = get_config().cloned();
    PortableStatus {
        is_portable: config.is_some(),
        config,
    }
}

/// Ensure the portable data directory structure exists.
/// Returns the data directory path or an error.
#[tauri::command]
pub fn portable_ensure_dirs() -> Result<String, String> {
    let cfg = get_config().ok_or("Not running in portable mode")?;

    let dirs = [
        &cfg.config_dir,
        &cfg.cache_dir,
        &cfg.temp_dir,
        &cfg.log_dir,
        &cfg.projects_dir,
    ];
    for dir in &dirs {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create {}: {}", dir, e))?;
    }

    Ok(cfg.data_dir.clone())
}
