// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! System monitoring and resource usage commands.

use std::collections::HashMap;
use std::sync::{LazyLock, OnceLock, Mutex as StdMutex};
use tauri::Emitter;
use tracing::info;

// System Stats Command
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_percent: f32,
    // App-specific stats
    pub app_cpu_usage: f32,
    pub app_memory: u64,
    pub app_threads: usize,
    pub cpu_cores: usize,
}

static SYSTEM: OnceLock<StdMutex<sysinfo::System>> = OnceLock::new();

fn get_system() -> &'static StdMutex<sysinfo::System> {
    SYSTEM.get_or_init(|| {
        // Use minimal initialization - refresh_all is expensive
        // We'll refresh specific items lazily
        let sys = sysinfo::System::new();
        StdMutex::new(sys)
    })
}

/// Initialize system stats collector in background (call from setup)
pub fn init_system_stats_background() {
    std::thread::spawn(|| {
        let start = std::time::Instant::now();
        let Ok(mut sys) = get_system().lock() else { return };
        // Do the expensive refresh in background
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        // Only refresh our own process, not all processes (much faster)
        let pid = sysinfo::Pid::from_u32(std::process::id());
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
        info!(elapsed_ms = start.elapsed().as_millis(), "System stats init");
    });
}

pub fn collect_system_stats() -> SystemStats {
    let Ok(mut sys) = get_system().lock() else {
        // Return default stats if lock is poisoned
        tracing::warn!("System stats lock poisoned, returning defaults");
        return SystemStats {
            cpu_usage: 0.0,
            memory_used: 0,
            memory_total: 0,
            memory_percent: 0.0,
            app_cpu_usage: 0.0,
            app_memory: 0,
            app_threads: 0,
            cpu_cores: 0,
        };
    };
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    // Only refresh our own process - refreshing ALL processes is extremely slow (2+ seconds)
    let pid = sysinfo::Pid::from_u32(std::process::id());
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
    
    let cpu_usage = sys.global_cpu_usage();
    let memory_used = sys.used_memory();
    let memory_total = sys.total_memory();
    let memory_percent = if memory_total > 0 {
        (memory_used as f32 / memory_total as f32) * 100.0
    } else {
        0.0
    };
    
    // Get app-specific stats
    let (app_cpu_usage, app_memory, app_threads) = if let Some(process) = sys.process(pid) {
        // process.tasks() is not supported on macOS, use rayon thread count as worker threads
        let threads = process.tasks()
            .map(|t| t.len())
            .unwrap_or_else(rayon::current_num_threads);
        (process.cpu_usage(), process.memory(), threads)
    } else {
        (0.0, 0, rayon::current_num_threads())
    };
    
    let cpu_cores = sys.cpus().len();
    
    SystemStats {
        cpu_usage,
        memory_used,
        memory_total,
        memory_percent,
        app_cpu_usage,
        app_memory,
        app_threads,
        cpu_cores,
    }
}

#[tauri::command]
pub fn get_system_stats() -> SystemStats {
    collect_system_stats()
}

/// Start background system stats monitoring - emits "system-stats" events every 2 seconds
pub fn start_system_stats_monitor(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let stats = collect_system_stats();
            let _ = app_handle.emit("system-stats", stats);
        }
    });
}

/// Result of preview cache cleanup
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub files_removed: u64,
    pub bytes_freed: u64,
    pub errors: Vec<String>,
}

/// Clean up temporary files created by preview extraction and thumbnail generation.
/// Removes contents of `core-ffx-preview/` and `core-ffx-thumbnails/` in the system temp directory.
#[tauri::command]
pub async fn cleanup_preview_cache() -> Result<CleanupResult, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let temp = std::env::temp_dir();
        let dirs = ["core-ffx-preview", "core-ffx-thumbnails"];
        let mut files_removed: u64 = 0;
        let mut bytes_freed: u64 = 0;
        let mut errors = Vec::new();

        for dir_name in &dirs {
            let dir_path = temp.join(dir_name);
            if !dir_path.exists() {
                continue;
            }
            match std::fs::read_dir(&dir_path) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        match std::fs::metadata(&path) {
                            Ok(meta) => {
                                bytes_freed += meta.len();
                                if let Err(e) = std::fs::remove_file(&path) {
                                    errors.push(format!("Failed to remove {}: {}", path.display(), e));
                                } else {
                                    files_removed += 1;
                                }
                            }
                            Err(e) => {
                                errors.push(format!("Failed to read metadata for {}: {}", path.display(), e));
                            }
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Failed to read directory {}: {}", dir_path.display(), e));
                }
            }
        }

        info!(files_removed, bytes_freed, "Preview cache cleanup complete");
        Ok(CleanupResult { files_removed, bytes_freed, errors })
    })
    .await
    .map_err(|e| format!("Cleanup task failed: {}", e))?
}

/// Write text content to a file on disk.
/// Used for exporting activity logs, reports, and other text-based data.
#[tauri::command]
pub async fn write_text_file(path: String, content: String) -> Result<(), String> {
    use std::path::Path;

    let file_path = Path::new(&path);

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    std::fs::write(file_path, content.as_bytes())
        .map_err(|e| format!("Failed to write file: {e}"))?;

    info!(path = %path, bytes = content.len(), "Text file written");
    Ok(())
}

/// Get the path to the audit log directory.
/// Returns the platform-specific path where daily-rotating audit logs are stored.
#[tauri::command]
pub fn get_audit_log_path() -> Result<String, String> {
    let dir = crate::logging::audit_log_dir()?;
    Ok(dir.to_string_lossy().into_owned())
}

/// Read recent audit log entries.
///
/// Returns up to `max_lines` recent log entries (newest first).
/// Each entry is a JSON-formatted string from the audit log files.
#[tauri::command]
pub async fn read_audit_log(max_lines: Option<usize>) -> Result<Vec<String>, String> {
    let limit = max_lines.unwrap_or(500);
    tauri::async_runtime::spawn_blocking(move || {
        crate::logging::read_audit_logs(limit)
    })
    .await
    .map_err(|e| format!("Audit log read task failed: {e}"))?
}

// =============================================================================
// Drive / Volume Enumeration
// =============================================================================

/// Information about a single disk/volume on the system.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveInfo {
    /// OS device path (e.g. "/dev/disk0s1" on macOS)
    pub device_path: String,
    /// Human-readable name assigned by the OS
    pub name: String,
    /// Mount point (e.g. "/" or "/Volumes/MyUSB")
    pub mount_point: String,
    /// Filesystem type (e.g. "apfs", "ntfs", "fat32")
    pub file_system: String,
    /// Total capacity in bytes
    pub total_bytes: u64,
    /// Available (free) space in bytes
    pub available_bytes: u64,
    /// Used space in bytes
    pub used_bytes: u64,
    /// Disk media kind: "SSD", "HDD", or "Unknown"
    pub kind: String,
    /// Whether the disk is removable (USB, SD card, etc.)
    pub is_removable: bool,
    /// Whether the disk is mounted read-only
    pub is_read_only: bool,
    /// Whether this is the boot / system volume (e.g. "/" on macOS/Linux, "C:\" on Windows)
    pub is_system_disk: bool,
}

/// Returns `true` if the given mount point belongs to a virtual/internal
/// volume that should not be shown as an imaging target.
fn is_virtual_mount(mount_point: &str, file_system: &str) -> bool {
    // Virtual/pseudo filesystems (cross-platform)
    let virtual_fs = ["devfs", "autofs", "vmhgfs-fuse", "tmpfs", "proc", "sysfs", "cgroup"];
    if virtual_fs.iter().any(|fs| file_system.eq_ignore_ascii_case(fs)) {
        return true;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS system snapshot/preboot volumes that are not meaningful imaging targets
        let skip_prefixes = [
            "/System/Volumes/Preboot",
            "/System/Volumes/Recovery",
            "/System/Volumes/VM",
            "/System/Volumes/Update",
            "/System/Volumes/xarts",
            "/System/Volumes/iSCPreboot",
            "/System/Volumes/Hardware",
            "/private/var/vm",
        ];
        if skip_prefixes.iter().any(|pfx| mount_point.starts_with(pfx)) {
            return true;
        }
        // Skip /dev mount point itself
        if mount_point == "/dev" {
            return true;
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: skip system/recovery partitions and special volumes
        let lower = mount_point.to_lowercase();
        if lower.contains("system volume information")
            || lower.contains("\\recovery")
            || lower.starts_with("\\\\?\\")
        {
            return true;
        }
    }

    #[cfg(target_os = "linux")]
    {
        if mount_point == "/dev" || mount_point.starts_with("/proc") || mount_point.starts_with("/sys") {
            return true;
        }
    }

    false
}

/// Detect whether a mount point is the system / boot volume.
fn is_system_volume(mount_point: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        // macOS: "/" is the root, "/System/Volumes/Data" is the data volume paired with it
        if mount_point == "/" || mount_point == "/System/Volumes/Data" {
            return true;
        }
    }
    #[cfg(target_os = "windows")]
    {
        // Windows: C:\ is typically the system drive
        let upper = mount_point.to_uppercase();
        if upper == "C:\\" || upper == "C:" {
            return true;
        }
    }
    #[cfg(target_os = "linux")]
    {
        if mount_point == "/" {
            return true;
        }
    }
    false
}

/// List all mounted disks / volumes visible to the OS.
///
/// Filters out virtual/system-internal volumes (devfs, VM, Preboot, etc.) that
/// are not useful forensic imaging targets.  Tags system/boot volumes so the UI
/// can warn before imaging them.
#[tauri::command]
pub fn list_drives() -> Vec<DriveInfo> {
    use sysinfo::Disks;

    let disks = Disks::new_with_refreshed_list();
    disks
        .iter()
        .filter_map(|d| {
            let mount = d.mount_point().to_string_lossy().into_owned();
            let fs = d.file_system().to_string_lossy().into_owned();

            // Skip virtual/internal volumes
            if is_virtual_mount(&mount, &fs) {
                return None;
            }

            let total = d.total_space();
            let available = d.available_space();
            let kind_str = match d.kind() {
                sysinfo::DiskKind::SSD => "SSD".to_string(),
                sysinfo::DiskKind::HDD => "HDD".to_string(),
                sysinfo::DiskKind::Unknown(_) => "Unknown".to_string(),
            };
            Some(DriveInfo {
                device_path: d.name().to_string_lossy().into_owned(),
                name: d.name().to_string_lossy().into_owned(),
                mount_point: mount.clone(),
                file_system: fs,
                total_bytes: total,
                available_bytes: available,
                used_bytes: total.saturating_sub(available),
                kind: kind_str,
                is_removable: d.is_removable(),
                is_read_only: d.is_read_only(),
                is_system_disk: is_system_volume(&mount),
            })
        })
        .collect()
}

// =============================================================================
// Read-Only Remount for Forensic Imaging
// =============================================================================

/// Result of a mount-state change operation.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MountResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// Human-readable message
    pub message: String,
    /// The mount point affected
    pub mount_point: String,
    /// Whether the volume is now read-only
    pub is_read_only: bool,
}

/// Tracks the original mount state of a volume so it can be restored later.
/// Key = mount point, Value = was_read_only_before_remount
static ORIGINAL_MOUNT_STATE: LazyLock<StdMutex<HashMap<String, bool>>> =
    LazyLock::new(|| StdMutex::new(HashMap::new()));

// =============================================================================
// macOS-specific mount helpers (diskutil)
// =============================================================================

/// Look up the BSD device identifier (e.g. "disk4s1") for a given mount point
/// by querying `diskutil info`.
#[cfg(target_os = "macos")]
fn device_for_mount_point(mount_point: &str) -> Result<String, String> {
    let output = std::process::Command::new("diskutil")
        .args(["info", mount_point])
        .output()
        .map_err(|e| format!("Failed to run diskutil info: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("diskutil info failed for {mount_point}: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Device Identifier:") {
            if let Some(dev) = trimmed.strip_prefix("Device Identifier:") {
                return Ok(dev.trim().to_string());
            }
        }
    }
    Err(format!("Could not find Device Identifier for {mount_point}"))
}

/// Check whether a volume is currently mounted read-only by inspecting `mount`
/// output.
#[cfg(target_os = "macos")]
fn is_currently_read_only(mount_point: &str) -> bool {
    let output = std::process::Command::new("mount")
        .output()
        .ok();
    if let Some(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            // Lines look like: /dev/disk4s1 on /Volumes/USB (apfs, local, nodev, nosuid, read-only, journaled)
            if line.contains(&format!("on {mount_point} ")) || line.contains(&format!("on {mount_point}\t")) {
                return line.contains("read-only");
            }
        }
    }
    false
}

/// Remount a volume as read-only for forensic imaging.
///
/// On macOS this uses `diskutil unmount` followed by `diskutil mount readOnly`.
/// The original mount state is recorded so it can be restored after imaging.
///
/// **Requirements:**
/// - Removable drives: works without administrator privileges
/// - Internal/system drives: may require admin (and system boot volume is refused)
///
/// Returns an error if the volume cannot be safely remounted.
#[tauri::command]
pub async fn remount_read_only(mount_point: String) -> Result<MountResult, String> {
    info!("Requesting read-only remount for: {}", mount_point);

    // Safety: refuse the boot volume
    if is_system_volume(&mount_point) {
        return Err(format!(
            "Cannot remount the system boot volume ({}) as read-only while the OS is running.",
            mount_point
        ));
    }

    #[cfg(target_os = "macos")]
    {
        // Check if already read-only — nothing to do
        let already_ro = is_currently_read_only(&mount_point);
        if already_ro {
            info!("{} is already read-only", mount_point);
            // Record that it was already RO so restore is a no-op
            if let Ok(mut state) = ORIGINAL_MOUNT_STATE.lock() {
                state.insert(mount_point.clone(), true);
            }
            return Ok(MountResult {
                success: true,
                message: "Volume is already mounted read-only.".into(),
                mount_point,
                is_read_only: true,
            });
        }

        // Record the original state (read-write)
        if let Ok(mut state) = ORIGINAL_MOUNT_STATE.lock() {
            state.insert(mount_point.clone(), false);
        }

        // Resolve the BSD device identifier
        let device_id = device_for_mount_point(&mount_point)?;
        info!("Device identifier for {}: {}", mount_point, device_id);

        // Step 1: Unmount the volume
        let unmount = std::process::Command::new("diskutil")
            .args(["unmount", &mount_point])
            .output()
            .map_err(|e| format!("Failed to run diskutil unmount: {e}"))?;

        if !unmount.status.success() {
            let stderr = String::from_utf8_lossy(&unmount.stderr);
            return Err(format!(
                "Failed to unmount {}: {}. Close any open files on this volume and try again.",
                mount_point, stderr.trim()
            ));
        }
        info!("Unmounted {}", mount_point);

        // Step 2: Remount read-only
        let remount = std::process::Command::new("diskutil")
            .args(["mount", "readOnly", &device_id])
            .output()
            .map_err(|e| format!("Failed to run diskutil mount readOnly: {e}"))?;

        if !remount.status.success() {
            // Try to remount read-write as recovery
            let _ = std::process::Command::new("diskutil")
                .args(["mount", &device_id])
                .output();
            let stderr = String::from_utf8_lossy(&remount.stderr);
            return Err(format!(
                "Failed to remount {} as read-only: {}. The volume has been re-mounted normally.",
                mount_point, stderr.trim()
            ));
        }

        info!("Remounted {} as read-only", mount_point);

        return Ok(MountResult {
            success: true,
            message: format!("Volume remounted as read-only at {}.", mount_point),
            mount_point,
            is_read_only: true,
        });
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(format!(
            "Read-only remounting is not yet supported on this platform. \
             Mount point: {}. On Windows, use a hardware write-blocker or \
             third-party forensic tool for write-protection.",
            mount_point
        ))
    }
}

/// Restore a volume to its original mount state (read-write) after imaging.
///
/// Only restores if the volume was originally read-write before
/// `remount_read_only` was called.
#[tauri::command]
pub async fn restore_mount(mount_point: String) -> Result<MountResult, String> {
    info!("Restoring original mount state for: {}", mount_point);

    #[cfg(target_os = "macos")]
    {
        // Check if we have a recorded original state
        let was_already_ro = {
            let state = ORIGINAL_MOUNT_STATE.lock().map_err(|e| e.to_string())?;
            state.get(&mount_point).copied()
        };

        match was_already_ro {
            None => {
                // We never remounted this volume — nothing to restore
                let current_ro = is_currently_read_only(&mount_point);
                return Ok(MountResult {
                    success: true,
                    message: "No remount was performed for this volume — nothing to restore.".into(),
                    mount_point,
                    is_read_only: current_ro,
                });
            }
            Some(true) => {
                // It was already read-only before we touched it — leave it as-is
                if let Ok(mut state) = ORIGINAL_MOUNT_STATE.lock() {
                    state.remove(&mount_point);
                }
                return Ok(MountResult {
                    success: true,
                    message: "Volume was originally read-only — left unchanged.".into(),
                    mount_point,
                    is_read_only: true,
                });
            }
            Some(false) => {
                // It was read-write before — restore it
            }
        }

        // Resolve the device identifier
        let device_id = device_for_mount_point(&mount_point)?;

        // Unmount then remount read-write
        let unmount = std::process::Command::new("diskutil")
            .args(["unmount", &mount_point])
            .output()
            .map_err(|e| format!("Failed to run diskutil unmount: {e}"))?;

        if !unmount.status.success() {
            let stderr = String::from_utf8_lossy(&unmount.stderr);
            return Err(format!(
                "Failed to unmount {} for restore: {}",
                mount_point, stderr.trim()
            ));
        }

        let remount = std::process::Command::new("diskutil")
            .args(["mount", &device_id])
            .output()
            .map_err(|e| format!("Failed to run diskutil mount: {e}"))?;

        if !remount.status.success() {
            let stderr = String::from_utf8_lossy(&remount.stderr);
            return Err(format!(
                "Failed to restore {} to read-write: {}",
                mount_point, stderr.trim()
            ));
        }

        // Clean up tracked state
        if let Ok(mut state) = ORIGINAL_MOUNT_STATE.lock() {
            state.remove(&mount_point);
        }

        info!("Restored {} to read-write", mount_point);

        return Ok(MountResult {
            success: true,
            message: format!("Volume restored to read-write at {}.", mount_point),
            mount_point,
            is_read_only: false,
        });
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS platforms, no remounting was performed, so nothing to restore
        Ok(MountResult {
            success: true,
            message: "No remount was performed (not supported on this platform) — nothing to restore.".into(),
            mount_point,
            is_read_only: false,
        })
    }
}
