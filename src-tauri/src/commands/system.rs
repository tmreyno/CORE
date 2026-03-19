// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! System monitoring and resource usage commands.

use std::sync::{Mutex as StdMutex, OnceLock};
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::{collections::HashMap, sync::LazyLock};
use tauri::Emitter;
use tracing::info;


/// Cached network interface list with TTL.
/// Networks::new_with_refreshed_list() is expensive (~200-500ms) — cache the result.
static NETWORK_CACHE: StdMutex<Option<(std::time::Instant, Vec<NetworkInterfaceInfo>)>> =
    StdMutex::new(None);

const NETWORK_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(30);

// System Stats Command
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub mac_address: String,
    pub ip_addresses: Vec<String>,
}

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
    // System identification
    pub os_name: String,
    pub os_version: String,
    pub long_os_version: String,
    pub hostname: String,
    pub cpu_brand: String,
    pub cpu_arch: String,
    // Extended forensic info
    pub kernel_version: String,
    pub uptime_secs: u64,
    pub boot_time_epoch: u64,
    pub physical_cores: usize,
    pub cpu_frequency_mhz: u64,
    pub cpu_vendor: String,
    pub total_swap: u64,
    pub used_swap: u64,
    pub timezone: String,
    pub network_interfaces: Vec<NetworkInterfaceInfo>,
    // Hardware identification (machine-level)
    pub system_serial_number: String,
    pub system_model: String,
    pub system_manufacturer: String,
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
        let Ok(mut sys) = get_system().lock() else {
            return;
        };
        // Do the expensive refresh in background
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        // Only refresh our own process, not all processes (much faster)
        let pid = sysinfo::Pid::from_u32(std::process::id());
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
        drop(sys); // Release lock before pre-warming caches

        // Pre-warm caches so runIdentify() is instant when user clicks it
        // 1. Hardware IDs (serial, model, manufacturer) — sysctl/ioreg on macOS, one-shot
        collect_hardware_ids();
        // 2. Physical disk enumeration — diskutil on macOS (populates mount-to-device map too)
        super::device::pre_warm_disk_cache();
        // 3. Network interfaces — avoid expensive Networks::new_with_refreshed_list() on first call
        get_cached_network_interfaces();

        info!(
            elapsed_ms = start.elapsed().as_millis(),
            "System stats + caches init"
        );
    });
}

/// Collect system hardware IDs (serial number, model, manufacturer).
/// These don't change at runtime, so cache the result.
fn collect_hardware_ids() -> (String, String, String) {
    use std::sync::OnceLock;
    static HW_IDS: OnceLock<(String, String, String)> = OnceLock::new();
    HW_IDS
        .get_or_init(|| {
            let ids = collect_hardware_ids_impl();
            tracing::debug!(
                serial = %ids.0,
                model = %ids.1,
                manufacturer = %ids.2,
                "Collected hardware IDs"
            );
            ids
        })
        .clone()
}

#[cfg(target_os = "macos")]
fn collect_hardware_ids_impl() -> (String, String, String) {
    let mut serial = String::new();
    let mut model = String::new();
    let manufacturer = "Apple".to_string();

    // Model identifier (e.g. "Mac14,7" or "MacBookPro18,3")
    if let Ok(output) = std::process::Command::new("sysctl")
        .args(["-n", "hw.model"])
        .output()
    {
        if output.status.success() {
            model = String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }

    // Serial number from IORegistry
    if let Ok(output) = std::process::Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if line.contains("IOPlatformSerialNumber") {
                    if let Some(val) = line.split('=').nth(1) {
                        serial = val.trim().trim_matches('"').trim().to_string();
                    }
                    break;
                }
            }
        }
    }

    (serial, model, manufacturer)
}

#[cfg(target_os = "linux")]
fn collect_hardware_ids_impl() -> (String, String, String) {
    fn read_dmi(file: &str) -> String {
        std::fs::read_to_string(format!("/sys/class/dmi/id/{}", file))
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    }
    let serial = read_dmi("product_serial");
    let model = read_dmi("product_name");
    let manufacturer = read_dmi("sys_vendor");
    (serial, model, manufacturer)
}

#[cfg(target_os = "windows")]
fn collect_hardware_ids_impl() -> (String, String, String) {
    let mut serial = String::new();
    let mut model = String::new();
    let mut manufacturer = String::new();

    if let Ok(output) = std::process::Command::new("wmic")
        .args(["bios", "get", "serialnumber", "/value"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if let Some(val) = line.strip_prefix("SerialNumber=") {
                    serial = val.trim().to_string();
                    break;
                }
            }
        }
    }

    if let Ok(output) = std::process::Command::new("wmic")
        .args(["csproduct", "get", "name,vendor", "/value"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if let Some(val) = line.strip_prefix("Name=") {
                    model = val.trim().to_string();
                } else if let Some(val) = line.strip_prefix("Vendor=") {
                    manufacturer = val.trim().to_string();
                }
            }
        }
    }

    (serial, model, manufacturer)
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn collect_hardware_ids_impl() -> (String, String, String) {
    (String::new(), String::new(), String::new())
}

/// Return cached network interfaces, refreshing only when the cache has expired.
/// Avoids the expensive Networks::new_with_refreshed_list() (~200-500ms) on every call.
fn get_cached_network_interfaces() -> Vec<NetworkInterfaceInfo> {
    let now = std::time::Instant::now();

    // Check cache
    if let Ok(cache) = NETWORK_CACHE.lock() {
        if let Some((ts, ref cached)) = *cache {
            if now.duration_since(ts) < NETWORK_CACHE_TTL {
                return cached.clone();
            }
        }
    }

    // Refresh
    let networks = sysinfo::Networks::new_with_refreshed_list();
    let interfaces: Vec<NetworkInterfaceInfo> = networks
        .list()
        .iter()
        .filter(|(name, _)| {
            !name.starts_with("lo")
                && !name.starts_with("utun")
                && !name.starts_with("awdl")
                && !name.starts_with("llw")
                && !name.starts_with("bridge")
                && !name.starts_with("anpi")
                && !name.starts_with("ap")
        })
        .map(|(name, data)| {
            let mac = format!("{}", data.mac_address());
            let ips: Vec<String> = data
                .ip_networks()
                .iter()
                .map(|n| format!("{}", n))
                .collect();
            NetworkInterfaceInfo {
                name: name.clone(),
                mac_address: mac,
                ip_addresses: ips,
            }
        })
        .collect();

    // Store in cache
    if let Ok(mut cache) = NETWORK_CACHE.lock() {
        *cache = Some((std::time::Instant::now(), interfaces.clone()));
    }

    interfaces
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
            os_name: String::new(),
            os_version: String::new(),
            long_os_version: String::new(),
            hostname: String::new(),
            cpu_brand: String::new(),
            cpu_arch: String::new(),
            kernel_version: String::new(),
            uptime_secs: 0,
            boot_time_epoch: 0,
            physical_cores: 0,
            cpu_frequency_mhz: 0,
            cpu_vendor: String::new(),
            total_swap: 0,
            used_swap: 0,
            timezone: String::new(),
            network_interfaces: Vec::new(),
            system_serial_number: String::new(),
            system_model: String::new(),
            system_manufacturer: String::new(),
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
        let threads = process
            .tasks()
            .map(|t| t.len())
            .unwrap_or_else(rayon::current_num_threads);
        (process.cpu_usage(), process.memory(), threads)
    } else {
        (0.0, 0, rayon::current_num_threads())
    };

    let cpu_cores = sys.cpus().len();
    let cpu_brand = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_default();
    let cpu_vendor = sys
        .cpus()
        .first()
        .map(|c| c.vendor_id().to_string())
        .unwrap_or_default();
    let cpu_frequency_mhz = sys.cpus().first().map(|c| c.frequency()).unwrap_or(0);
    let physical_cores = sys.physical_core_count().unwrap_or(0);
    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();

    // Use cached network interfaces (avoids expensive Networks::new_with_refreshed_list)
    let network_interfaces = get_cached_network_interfaces();

    // Get timezone from chrono
    let timezone = chrono::Local::now().format("%Z (UTC%:z)").to_string();

    // Gather hardware identification (serial, model, manufacturer)
    let (system_serial_number, system_model, system_manufacturer) = collect_hardware_ids();

    SystemStats {
        cpu_usage,
        memory_used,
        memory_total,
        memory_percent,
        app_cpu_usage,
        app_memory,
        app_threads,
        cpu_cores,
        os_name: sysinfo::System::name().unwrap_or_default(),
        os_version: sysinfo::System::os_version().unwrap_or_default(),
        long_os_version: sysinfo::System::long_os_version().unwrap_or_default(),
        hostname: sysinfo::System::host_name().unwrap_or_default(),
        cpu_brand,
        cpu_arch: sysinfo::System::cpu_arch().unwrap_or_default(),
        kernel_version: sysinfo::System::kernel_version().unwrap_or_default(),
        uptime_secs: sysinfo::System::uptime(),
        boot_time_epoch: sysinfo::System::boot_time(),
        physical_cores,
        cpu_frequency_mhz,
        cpu_vendor,
        total_swap,
        used_swap,
        timezone,
        network_interfaces,
        system_serial_number,
        system_model,
        system_manufacturer,
    }
}

#[tauri::command]
pub async fn get_system_stats() -> SystemStats {
    tauri::async_runtime::spawn_blocking(collect_system_stats)
        .await
        .unwrap_or_else(|_| SystemStats {
            cpu_usage: 0.0,
            memory_used: 0,
            memory_total: 0,
            memory_percent: 0.0,
            app_cpu_usage: 0.0,
            app_memory: 0,
            app_threads: 0,
            cpu_cores: 0,
            os_name: String::new(),
            os_version: String::new(),
            long_os_version: String::new(),
            hostname: String::new(),
            cpu_brand: String::new(),
            cpu_arch: String::new(),
            kernel_version: String::new(),
            uptime_secs: 0,
            boot_time_epoch: 0,
            physical_cores: 0,
            cpu_frequency_mhz: 0,
            cpu_vendor: String::new(),
            total_swap: 0,
            used_swap: 0,
            timezone: String::new(),
            network_interfaces: Vec::new(),
            system_serial_number: String::new(),
            system_model: String::new(),
            system_manufacturer: String::new(),
        })
}

/// Start background system stats monitoring - emits "system-stats" events every 2 seconds
pub fn start_system_stats_monitor(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(2));
        let stats = collect_system_stats();
        let _ = app_handle.emit("system-stats", stats);
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
        let temp = super::portable::portable_temp_dir();
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
                                    errors.push(format!(
                                        "Failed to remove {}: {}",
                                        path.display(),
                                        e
                                    ));
                                } else {
                                    files_removed += 1;
                                }
                            }
                            Err(e) => {
                                errors.push(format!(
                                    "Failed to read metadata for {}: {}",
                                    path.display(),
                                    e
                                ));
                            }
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!(
                        "Failed to read directory {}: {}",
                        dir_path.display(),
                        e
                    ));
                }
            }
        }

        info!(files_removed, bytes_freed, "Preview cache cleanup complete");
        Ok(CleanupResult {
            files_removed,
            bytes_freed,
            errors,
        })
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
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {e}"))?;
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
    tauri::async_runtime::spawn_blocking(move || crate::logging::read_audit_logs(limit))
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
    /// Drive model / product name (from physical disk enumeration, empty if unavailable)
    pub model: String,
    /// Drive serial number (from physical disk enumeration, empty if unavailable)
    pub serial: String,
    /// Drive vendor / manufacturer (from physical disk enumeration, empty if unavailable)
    pub vendor: String,
    /// Drive connection interface (e.g. "USB", "NVMe", "SATA", "Thunderbolt", from physical disk enumeration)
    pub connection_type: String,
    /// Whether the volume is encrypted (FileVault, BitLocker, LUKS)
    pub is_encrypted: bool,
    /// Encryption type/scheme if encrypted (e.g. "FileVault", "BitLocker", "LUKS", "")
    pub encryption_type: String,
    /// Partition scheme of the parent disk (e.g. "GPT", "MBR", "APM", "")
    pub partition_scheme: String,
    /// Parent whole-disk device path (e.g. "/dev/disk4" for volume "/dev/disk4s1")
    pub parent_disk: String,
    /// Total size of the parent physical disk in bytes (0 if unknown)
    pub parent_disk_size: u64,
}

/// Returns `true` if the given mount point belongs to a virtual/internal
/// volume that should not be shown as an imaging target.
fn is_virtual_mount(mount_point: &str, file_system: &str) -> bool {
    // Virtual/pseudo filesystems (cross-platform)
    let virtual_fs = [
        "devfs",
        "autofs",
        "vmhgfs-fuse",
        "tmpfs",
        "proc",
        "sysfs",
        "cgroup",
    ];
    if virtual_fs
        .iter()
        .any(|fs| file_system.eq_ignore_ascii_case(fs))
    {
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
        if mount_point == "/dev"
            || mount_point.starts_with("/proc")
            || mount_point.starts_with("/sys")
        {
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

/// Detect whether a volume is encrypted and return (is_encrypted, encryption_type).
fn detect_encryption(device_path: &str, _file_system: &str) -> (bool, String) {
    #[cfg(target_os = "macos")]
    {
        // Query diskutil info for the volume's encryption status
        if let Ok(output) = std::process::Command::new("diskutil")
            .args(["info", "-plist", device_path])
            .output()
        {
            if output.status.success() {
                if let Ok(plist) = plist::from_bytes::<plist::Value>(&output.stdout) {
                    if let Some(dict) = plist.as_dictionary() {
                        // Check for FileVault / APFS encryption
                        let encrypted = dict
                            .get("Encrypted")
                            .and_then(|v| v.as_boolean())
                            .unwrap_or(false);
                        let fv = dict
                            .get("FileVault")
                            .and_then(|v| v.as_boolean())
                            .unwrap_or(false);
                        if encrypted || fv {
                            let enc_type = dict
                                .get("EncryptionType")
                                .and_then(|v| v.as_string())
                                .unwrap_or(if fv { "FileVault" } else { "Encrypted" });
                            return (true, enc_type.to_string());
                        }
                    }
                }
            }
        }
        return (false, String::new());
    }
    #[cfg(target_os = "linux")]
    {
        // Check if the device is a LUKS-mapped dm-crypt device
        let dev_name = std::path::Path::new(device_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let dm_uuid = format!("/sys/block/{}/dm/uuid", dev_name);
        if let Ok(uuid) = std::fs::read_to_string(&dm_uuid) {
            if uuid.starts_with("CRYPT-") {
                return (true, "LUKS".to_string());
            }
        }
        return (false, String::new());
    }
    #[cfg(target_os = "windows")]
    {
        // Attempt to detect BitLocker via manage-bde status
        let drive_letter = device_path.chars().next().unwrap_or('C');
        if let Ok(output) = std::process::Command::new("manage-bde")
            .args(["-status", &format!("{}:", drive_letter)])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Protection On") || stdout.contains("Fully Encrypted") {
                return (true, "BitLocker".to_string());
            }
        }
        return (false, String::new());
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        (false, String::new())
    }
}

/// Detect partition scheme of a whole disk (GPT, MBR, APM, etc.).
fn detect_partition_scheme(parent_disk: &str) -> String {
    if parent_disk.is_empty() {
        return String::new();
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("diskutil")
            .args(["info", "-plist", parent_disk])
            .output()
        {
            if output.status.success() {
                if let Ok(plist) = plist::from_bytes::<plist::Value>(&output.stdout) {
                    if let Some(dict) = plist.as_dictionary() {
                        if let Some(content) =
                            dict.get("Content").and_then(|v| v.as_string())
                        {
                            return match content {
                                "GUID_partition_scheme" => "GPT".to_string(),
                                "FDisk_partition_scheme" => "MBR".to_string(),
                                "Apple_partition_scheme" => "APM".to_string(),
                                other => other.to_string(),
                            };
                        }
                    }
                }
            }
        }
        return String::new();
    }
    #[cfg(not(target_os = "macos"))]
    {
        // On Linux/Windows, partition scheme detection would require
        // reading the first sector or using OS-specific tools.
        String::new()
    }
}

/// Result of a path writability check.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WritabilityCheck {
    /// Whether the path is writable
    pub writable: bool,
    /// Human-readable reason if not writable
    pub reason: String,
    /// Filesystem type (e.g. "ntfs", "apfs") if detected
    pub file_system: String,
    /// Whether the volume is mounted read-only
    pub is_read_only: bool,
}

/// Check whether a path (or its parent volume) is writable.
///
/// Uses a write-probe (create + remove a temporary file) as the ground truth.
/// Falls back to `sysinfo::Disks` mount metadata only for descriptive error
/// messages when the probe fails.
///
/// # macOS firmlink caveat
///
/// On macOS Catalina+, `/Users` is a **firmlink** to `/System/Volumes/Data/Users`.
/// `sysinfo::Disks` reports mount points as `/` (read-only system volume) and
/// `/System/Volumes/Data` (writable data volume).  A path like
/// `/Users/terryreynolds/Documents` does NOT start with `/System/Volumes/Data`,
/// so prefix-based mount matching picks `/` and incorrectly reports read-only.
/// The write-probe bypasses this issue entirely — if the probe file can be
/// created, the path is writable regardless of what sysinfo reports.
#[tauri::command]
pub fn check_path_writable(path: String) -> WritabilityCheck {
    use std::path::Path;
    use sysinfo::Disks;

    let target = Path::new(&path);

    // Walk up to find an existing ancestor directory
    let existing_dir = {
        let mut p = target.to_path_buf();
        while !p.exists() {
            if !p.pop() {
                break;
            }
        }
        p
    };

    // Determine the directory to probe
    let probe_dir = if existing_dir.is_dir() {
        existing_dir.clone()
    } else if let Some(parent) = existing_dir.parent() {
        parent.to_path_buf()
    } else {
        return WritabilityCheck {
            writable: false,
            reason: "Cannot determine a writable directory for this path.".into(),
            file_system: String::new(),
            is_read_only: false,
        };
    };

    // ── Write probe (ground truth) ──────────────────────────────────────
    // Try an actual write FIRST.  This is the only reliable check on macOS
    // because firmlinked paths (/Users, /Library, etc.) cannot be matched
    // to their real mount point via prefix comparison.
    let probe_file = probe_dir.join(".core_ffx_write_probe");
    match std::fs::File::create(&probe_file) {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe_file);

            // Probe succeeded — path is writable.  Grab FS type for info.
            let disks = Disks::new_with_refreshed_list();
            let mut best_fs = String::new();
            let mut best_len = 0usize;
            let dir_str = existing_dir.to_string_lossy();
            for d in disks.iter() {
                let mount = d.mount_point().to_string_lossy();
                if dir_str.starts_with(mount.as_ref()) && mount.len() > best_len {
                    best_len = mount.len();
                    best_fs = d.file_system().to_string_lossy().into_owned();
                }
            }

            return WritabilityCheck {
                writable: true,
                reason: String::new(),
                file_system: best_fs,
                is_read_only: false,
            };
        }
        Err(_) => {
            // Probe failed — fall through to sysinfo for descriptive errors
        }
    }

    // ── Probe failed — gather mount info for a descriptive error ────────
    let disks = Disks::new_with_refreshed_list();
    let mut best_mount = String::new();
    let mut best_fs = String::new();
    let mut mounted_ro = false;

    let dir_str = existing_dir.to_string_lossy();
    for d in disks.iter() {
        let mount = d.mount_point().to_string_lossy().into_owned();
        if dir_str.starts_with(&mount) && mount.len() > best_mount.len() {
            best_mount = mount;
            best_fs = d.file_system().to_string_lossy().into_owned();
            mounted_ro = d.is_read_only();
        }
    }

    // Build a descriptive reason using sysinfo data
    let probe_err_reason = if mounted_ro {
        let fs_upper = best_fs.to_uppercase();
        if fs_upper == "NTFS" {
            "Volume is NTFS (read-only on macOS). Use an exFAT or APFS drive, \
             or install Paragon NTFS for write support."
                .to_string()
        } else {
            format!(
                "Volume at {} is mounted read-only ({} filesystem).",
                best_mount, fs_upper
            )
        }
    } else {
        // Try to re-create the probe to capture the actual OS error
        match std::fs::File::create(&probe_file) {
            Ok(_) => {
                // Race: became writable between attempts (unlikely)
                let _ = std::fs::remove_file(&probe_file);
                return WritabilityCheck {
                    writable: true,
                    reason: String::new(),
                    file_system: best_fs,
                    is_read_only: false,
                };
            }
            Err(e) => match e.raw_os_error() {
                Some(30) => format!(
                    "Read-only file system at {}. Choose a writable volume.",
                    best_mount
                ),
                Some(13) => format!(
                    "Permission denied: cannot write to {}.",
                    probe_dir.display()
                ),
                _ => format!("Cannot write to {}: {e}", probe_dir.display()),
            },
        }
    };

    WritabilityCheck {
        writable: false,
        reason: probe_err_reason,
        file_system: best_fs,
        is_read_only: mounted_ro,
    }
}

/// Resolve the actual OS device path for a volume from its mount point.
///
/// - **macOS**: Queries `diskutil info` for the BSD device node (e.g. "/dev/disk4s1")
/// - **Linux**: Parses `/proc/mounts` to find the device node (e.g. "/dev/sda1")
/// - **Windows**: Extracts the volume root (e.g. "C:") — physical drive enumeration
///   requires separate WMI/SetupDi queries
fn resolve_device_path(mount_point: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        // Reuse the existing diskutil-based resolver
        match device_for_mount_point(mount_point) {
            Ok(dev_id) => format!("/dev/{}", dev_id),
            Err(_) => String::new(),
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Parse /proc/mounts: each line is "device mountpoint fstype options ..."
        if let Ok(contents) = std::fs::read_to_string("/proc/mounts") {
            for line in contents.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1] == mount_point {
                    return parts[0].to_string();
                }
            }
        }
        String::new()
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, sysinfo mount_point is the drive root (e.g. "C:\\")
        // The volume device path requires WMI — return the drive letter path for now
        if mount_point.len() >= 2 {
            // Convert "C:\" to "\\.\C:" for raw device access
            let drive_letter = &mount_point[..2]; // "C:"
            format!("\\\\.\\{}", drive_letter)
        } else {
            String::new()
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = mount_point;
        String::new()
    }
}

/// List all mounted disks / volumes visible to the OS.
///
/// Filters out virtual/system-internal volumes (devfs, VM, Preboot, etc.) that
/// are not useful forensic imaging targets.  Tags system/boot volumes so the UI
/// can warn before imaging them.
///
/// `device_path` is resolved per-platform to the actual OS device node
/// (e.g. `/dev/disk4s1` on macOS, `/dev/sda1` on Linux, `\\.\C:` on Windows).
#[tauri::command]
pub async fn list_drives() -> Vec<DriveInfo> {
    tauri::async_runtime::spawn_blocking(list_drives_impl)
        .await
        .unwrap_or_default()
}

fn list_drives_impl() -> Vec<DriveInfo> {
    use std::collections::HashMap;
    use sysinfo::Disks;

    // Enumerate physical disks and build a lookup by device path / partition.
    // Uses the shared TTL cache in device.rs (no redundant diskutil calls).
    // Struct: model, serial, vendor, connection_type, whole_disk_path, disk_size_bytes, partition_scheme
    struct DiskHwInfo {
        model: String,
        serial: String,
        vendor: String,
        connection_type: String,
        whole_disk_path: String,
        disk_size_bytes: u64,
        partition_scheme: String,
    }

    let mut hw_lookup: HashMap<String, DiskHwInfo> = HashMap::new();
    let physical_disks = super::device::get_physical_disks_cached().unwrap_or_default();

    for pd in &physical_disks {
        // Map whole disk path and raw device path
        hw_lookup.insert(pd.whole_disk_path.clone(), DiskHwInfo {
            model: pd.model.clone(),
            serial: pd.serial.clone(),
            vendor: pd.vendor.clone(),
            connection_type: pd.connection_type.clone(),
            whole_disk_path: pd.whole_disk_path.clone(),
            disk_size_bytes: pd.size_bytes,
            partition_scheme: pd.partition_scheme.clone(),
        });
        hw_lookup.insert(pd.device_path.clone(), DiskHwInfo {
            model: pd.model.clone(),
            serial: pd.serial.clone(),
            vendor: pd.vendor.clone(),
            connection_type: pd.connection_type.clone(),
            whole_disk_path: pd.whole_disk_path.clone(),
            disk_size_bytes: pd.size_bytes,
            partition_scheme: pd.partition_scheme.clone(),
        });
        // Map each partition
        for part in &pd.partitions {
            hw_lookup.insert(part.clone(), DiskHwInfo {
                model: pd.model.clone(),
                serial: pd.serial.clone(),
                vendor: pd.vendor.clone(),
                connection_type: pd.connection_type.clone(),
                whole_disk_path: pd.whole_disk_path.clone(),
                disk_size_bytes: pd.size_bytes,
                partition_scheme: pd.partition_scheme.clone(),
            });
        }
    }

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

            // Fast path: resolve device path from cached mount-to-device map
            // (populated during physical disk enumeration, no subprocess needed)
            let device_path =
                super::device::resolve_device_from_cache(&mount).unwrap_or_else(|| {
                    // Fallback to diskutil subprocess for unmapped mounts
                    resolve_device_path(&mount)
                });

            // Look up hardware info from physical disk enumeration
            let hw_info = hw_lookup
                .get(&device_path)
                .or_else(|| {
                    // macOS: device_path may be "/dev/disk4s1" but hw_lookup has "/dev/disk4"
                    // Strip trailing partition suffix (sN) to match whole-disk path
                    let stripped = device_path
                        .trim_end_matches(|c: char| c.is_ascii_digit())
                        .trim_end_matches('s');
                    hw_lookup.get(stripped)
                });

            let (model, serial, vendor, connection_type, parent_disk, parent_disk_size, partition_scheme) =
                if let Some(info) = hw_info {
                    (
                        info.model.clone(),
                        info.serial.clone(),
                        info.vendor.clone(),
                        info.connection_type.clone(),
                        info.whole_disk_path.clone(),
                        info.disk_size_bytes,
                        info.partition_scheme.clone(),
                    )
                } else {
                    (String::new(), String::new(), String::new(), String::new(), String::new(), 0, String::new())
                };

            // Detect encryption status (still per-volume — no way to batch on macOS)
            let (is_encrypted, encryption_type) = detect_encryption(&device_path, &fs);

            // Use partition_scheme from physical disk data (no extra diskutil call needed)
            // Only fall back to detect_partition_scheme if not available from hw_lookup
            let partition_scheme = if partition_scheme.is_empty() {
                detect_partition_scheme(&parent_disk)
            } else {
                partition_scheme
            };

            Some(DriveInfo {
                device_path,
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
                model,
                serial,
                vendor,
                connection_type,
                is_encrypted,
                encryption_type,
                partition_scheme,
                parent_disk,
                parent_disk_size,
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
#[cfg(any(target_os = "macos", target_os = "linux"))]
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
    Err(format!(
        "Could not find Device Identifier for {mount_point}"
    ))
}

/// Check whether a volume is currently mounted read-only by inspecting `mount`
/// output.
#[cfg(target_os = "macos")]
fn is_currently_read_only(mount_point: &str) -> bool {
    let output = std::process::Command::new("mount").output().ok();
    if let Some(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            // Lines look like: /dev/disk4s1 on /Volumes/USB (apfs, local, nodev, nosuid, read-only, journaled)
            if line.contains(&format!("on {mount_point} "))
                || line.contains(&format!("on {mount_point}\t"))
            {
                return line.contains("read-only");
            }
        }
    }
    false
}

// =============================================================================
// Linux-specific mount helpers
// =============================================================================

/// Find the device node for a mount point by parsing /proc/mounts.
#[cfg(target_os = "linux")]
fn linux_device_for_mount_point(mount_point: &str) -> Result<String, String> {
    let mounts = std::fs::read_to_string("/proc/mounts")
        .map_err(|e| format!("Cannot read /proc/mounts: {e}"))?;
    for line in mounts.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == mount_point {
            return Ok(parts[0].to_string());
        }
    }
    Err(format!(
        "Could not find device for mount point {mount_point} in /proc/mounts"
    ))
}

/// Check whether a volume is currently mounted read-only on Linux.
#[cfg(target_os = "linux")]
fn is_currently_read_only_linux(mount_point: &str) -> bool {
    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // /proc/mounts: device mountpoint fstype options ...
            if parts.len() >= 4 && parts[1] == mount_point {
                // Options field is comma-separated; check for "ro"
                return parts[3].split(',').any(|opt| opt == "ro");
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
                mount_point,
                stderr.trim()
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
                mount_point,
                stderr.trim()
            ));
        }

        info!("Remounted {} as read-only", mount_point);

        Ok(MountResult {
            success: true,
            message: format!("Volume remounted as read-only at {}.", mount_point),
            mount_point,
            is_read_only: true,
        })
    }

    #[cfg(target_os = "linux")]
    {
        // Check if already read-only
        let already_ro = is_currently_read_only_linux(&mount_point);
        if already_ro {
            info!("{} is already read-only", mount_point);
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

        // Find the device for this mount point
        let device = linux_device_for_mount_point(&mount_point)?;
        info!("Device for {}: {}", mount_point, device);

        // Remount read-only via mount -o remount,ro
        let remount = std::process::Command::new("mount")
            .args(["-o", "remount,ro", &device, &mount_point])
            .output()
            .map_err(|e| format!("Failed to run mount -o remount,ro: {e}"))?;

        if !remount.status.success() {
            let stderr = String::from_utf8_lossy(&remount.stderr);
            return Err(format!(
                "Failed to remount {} as read-only: {}. \
                 You may need to run CORE-FFX with sudo or ensure no files are open on this volume.",
                mount_point,
                stderr.trim()
            ));
        }

        info!("Remounted {} as read-only", mount_point);

        Ok(MountResult {
            success: true,
            message: format!("Volume remounted as read-only at {}.", mount_point),
            mount_point,
            is_read_only: true,
        })
    }

    #[cfg(target_os = "windows")]
    {
        Err(format!(
            "Software write-blocking is not supported on Windows. \
             Mount point: {}. Use a hardware write-blocker or a third-party \
             forensic write-blocking tool (e.g., Arsenal Image Mounter, \
             FTK Imager) for write-protection.",
            mount_point
        ))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(format!(
            "Read-only remounting is not supported on this platform. \
             Mount point: {}.",
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
                    message: "No remount was performed for this volume — nothing to restore."
                        .into(),
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
                mount_point,
                stderr.trim()
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
                mount_point,
                stderr.trim()
            ));
        }

        // Clean up tracked state
        if let Ok(mut state) = ORIGINAL_MOUNT_STATE.lock() {
            state.remove(&mount_point);
        }

        info!("Restored {} to read-write", mount_point);

        Ok(MountResult {
            success: true,
            message: format!("Volume restored to read-write at {}.", mount_point),
            mount_point,
            is_read_only: false,
        })
    }

    #[cfg(target_os = "linux")]
    {
        // Check if we have a recorded original state
        let was_already_ro = {
            let state = ORIGINAL_MOUNT_STATE.lock().map_err(|e| e.to_string())?;
            state.get(&mount_point).copied()
        };

        match was_already_ro {
            None => {
                let current_ro = is_currently_read_only_linux(&mount_point);
                return Ok(MountResult {
                    success: true,
                    message: "No remount was performed for this volume — nothing to restore."
                        .into(),
                    mount_point,
                    is_read_only: current_ro,
                });
            }
            Some(true) => {
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
                // Was read-write — restore it
            }
        }

        let device = linux_device_for_mount_point(&mount_point)?;

        // Remount read-write via mount -o remount,rw
        let remount = std::process::Command::new("mount")
            .args(["-o", "remount,rw", &device, &mount_point])
            .output()
            .map_err(|e| format!("Failed to run mount -o remount,rw: {e}"))?;

        if !remount.status.success() {
            let stderr = String::from_utf8_lossy(&remount.stderr);
            return Err(format!(
                "Failed to restore {} to read-write: {}",
                mount_point,
                stderr.trim()
            ));
        }

        if let Ok(mut state) = ORIGINAL_MOUNT_STATE.lock() {
            state.remove(&mount_point);
        }

        info!("Restored {} to read-write", mount_point);

        Ok(MountResult {
            success: true,
            message: format!("Volume restored to read-write at {}.", mount_point),
            mount_point,
            is_read_only: false,
        })
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        // On Windows and other platforms, no remounting was performed
        Ok(MountResult {
            success: true,
            message:
                "No remount was performed (not supported on this platform) — nothing to restore."
                    .into(),
            mount_point,
            is_read_only: false,
        })
    }
}

// =============================================================================
// User & Version Commands
// =============================================================================

/// Get the current OS username.
#[tauri::command]
pub fn get_current_username() -> String {
    // Try USER (macOS/Linux), then USERNAME (Windows), then fall back
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Get the hostname of the current machine.
#[tauri::command]
pub fn get_hostname() -> String {
    sysinfo::System::host_name().unwrap_or_else(|| "unknown".to_string())
}

/// Get the current application version from Cargo.toml.
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get a comprehensive system health report including resource usage,
/// queue metrics, error rates, and health issues.
#[tauri::command]
pub fn get_system_health_report() -> crate::common::health::SystemHealth {
    crate::common::health::get_system_health()
}
