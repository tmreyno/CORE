// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Raw device access, privilege detection, and physical disk operations.
//!
//! This module provides cross-platform abstractions for:
//! - **Privilege detection**: Whether the current process has elevated/root privileges
//! - **Device size**: Query the byte-level size of a raw block device
//! - **Raw device reading**: Open and stream data from raw device nodes for forensic imaging
//!
//! ## Platform Details
//!
//! | Capability          | macOS               | Linux               | Windows                      |
//! |---------------------|---------------------|---------------------|------------------------------|
//! | Privilege detection | `privilege::user_is_root` | `privilege::user_is_root` | `privilege::user_is_admin` |
//! | Device size         | `ioctl(DKIOCGETBLOCKCOUNT × DKIOCGETBLOCKSIZE)` | `ioctl(BLKGETSIZE64)` | `DeviceIoControl(IOCTL_DISK_GET_LENGTH_INFO)` |
//! | Raw device path     | `/dev/rdiskN`       | `/dev/sdX`          | `\\.\PhysicalDriveN`        |

use tracing::info;

// =============================================================================
// Privilege Detection
// =============================================================================

/// Information about the current process's privilege level.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivilegeInfo {
    /// Whether the process is running with elevated privileges (root/admin)
    pub is_elevated: bool,
    /// Human-readable description of the privilege level
    pub description: String,
    /// The OS username of the current user
    pub username: String,
    /// Whether elevation is required for raw device access on this platform
    pub elevation_required: bool,
}

/// Check the current process's privilege level.
///
/// On macOS/Linux, checks if running as root (UID 0).
/// On Windows, checks if running with Administrator privileges.
///
/// Raw device access typically requires elevation:
/// - **macOS**: `/dev/rdiskN` requires root
/// - **Linux**: `/dev/sdX` requires root or `disk` group membership
/// - **Windows**: `\\.\PhysicalDriveN` requires Administrator
#[tauri::command]
pub fn check_privilege() -> PrivilegeInfo {
    let is_elevated = privilege::user::privileged();

    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());

    let description = if is_elevated {
        "Running with elevated privileges (root/admin).".to_string()
    } else {
        "Running as a standard user. Elevation is required for raw device access.".to_string()
    };

    // Raw device access requires elevation on all platforms
    let elevation_required = !is_elevated;

    PrivilegeInfo {
        is_elevated,
        description,
        username,
        elevation_required,
    }
}

// =============================================================================
// Device Size Detection
// =============================================================================

/// Get the total size in bytes of a raw block device.
///
/// This uses platform-specific ioctl/DeviceIoControl calls:
/// - **macOS**: `DKIOCGETBLOCKCOUNT` × `DKIOCGETBLOCKSIZE`
/// - **Linux**: `BLKGETSIZE64`
/// - **Windows**: `IOCTL_DISK_GET_LENGTH_INFO`
#[tauri::command]
pub fn get_device_size(device_path: String) -> Result<u64, String> {
    info!("Getting device size for: {}", device_path);
    device_size_impl(&device_path)
}

#[cfg(target_os = "macos")]
fn device_size_impl(device_path: &str) -> Result<u64, String> {
    use std::fs::File;
    use std::os::unix::io::AsRawFd;

    // ioctl request codes for macOS disk block count and block size
    // From <sys/disk.h>:
    //   DKIOCGETBLOCKCOUNT = _IOR('d', 25, uint64_t) = 0x40086419
    //   DKIOCGETBLOCKSIZE  = _IOR('d', 24, uint32_t) = 0x40046418
    const DKIOCGETBLOCKCOUNT: libc::c_ulong = 0x40086419;
    const DKIOCGETBLOCKSIZE: libc::c_ulong = 0x40046418;

    // Prefer /dev/rdisk (raw, character device) for block-level queries
    let raw_path = if device_path.contains("/dev/disk") {
        device_path.replace("/dev/disk", "/dev/rdisk")
    } else {
        device_path.to_string()
    };

    let file = File::open(&raw_path)
        .or_else(|_| File::open(device_path))
        .map_err(|e| format!("Cannot open device {}: {}", device_path, e))?;

    let fd = file.as_raw_fd();

    let mut block_count: u64 = 0;
    let mut block_size: u32 = 0;

    // SAFETY: ioctl with valid fd and correctly-sized output buffers
    unsafe {
        if libc::ioctl(fd, DKIOCGETBLOCKCOUNT, &mut block_count) != 0 {
            return Err(format!(
                "ioctl DKIOCGETBLOCKCOUNT failed for {}: {}",
                device_path,
                std::io::Error::last_os_error()
            ));
        }
        if libc::ioctl(fd, DKIOCGETBLOCKSIZE, &mut block_size) != 0 {
            return Err(format!(
                "ioctl DKIOCGETBLOCKSIZE failed for {}: {}",
                device_path,
                std::io::Error::last_os_error()
            ));
        }
    }

    let total_bytes = block_count * block_size as u64;
    info!(
        "Device {} size: {} bytes ({} blocks × {} byte/block)",
        device_path, total_bytes, block_count, block_size
    );
    Ok(total_bytes)
}

#[cfg(target_os = "linux")]
fn device_size_impl(device_path: &str) -> Result<u64, String> {
    use std::fs::File;
    use std::os::unix::io::AsRawFd;

    // BLKGETSIZE64 = _IOR(0x12, 114, sizeof(u64)) = 0x80081272
    const BLKGETSIZE64: libc::c_ulong = 0x80081272;

    let file = File::open(device_path)
        .map_err(|e| format!("Cannot open device {}: {}", device_path, e))?;

    let fd = file.as_raw_fd();
    let mut size: u64 = 0;

    // SAFETY: ioctl with valid fd and correctly-sized output buffer
    unsafe {
        if libc::ioctl(fd, BLKGETSIZE64, &mut size) != 0 {
            return Err(format!(
                "ioctl BLKGETSIZE64 failed for {}: {}",
                device_path,
                std::io::Error::last_os_error()
            ));
        }
    }

    info!("Device {} size: {} bytes", device_path, size);
    Ok(size)
}

#[cfg(target_os = "windows")]
fn device_size_impl(device_path: &str) -> Result<u64, String> {
    use std::fs::File;
    use std::os::windows::io::AsRawHandle;

    // IOCTL_DISK_GET_LENGTH_INFO = CTL_CODE(IOCTL_DISK_BASE, 0x0017, METHOD_BUFFERED, FILE_READ_ACCESS)
    // = (0x00000007 << 16) | (0x0001 << 14) | (0x0017 << 2) | 0 = 0x0007405C
    const IOCTL_DISK_GET_LENGTH_INFO: u32 = 0x0007405C;

    // GET_LENGTH_INFORMATION struct: just a single i64
    #[repr(C)]
    struct GetLengthInformation {
        length: i64,
    }

    let file = File::open(device_path)
        .map_err(|e| format!("Cannot open device {}: {}", device_path, e))?;

    let handle = file.as_raw_handle();
    let mut length_info = GetLengthInformation { length: 0 };
    let mut bytes_returned: u32 = 0;

    // SAFETY: DeviceIoControl with valid handle and correctly-sized buffer
    let result = unsafe {
        windows_sys::Win32::System::IO::DeviceIoControl(
            handle as isize,
            IOCTL_DISK_GET_LENGTH_INFO,
            std::ptr::null(),
            0,
            &mut length_info as *mut _ as *mut std::ffi::c_void,
            std::mem::size_of::<GetLengthInformation>() as u32,
            &mut bytes_returned,
            std::ptr::null_mut(),
        )
    };

    if result == 0 {
        return Err(format!(
            "DeviceIoControl failed for {}: {}",
            device_path,
            std::io::Error::last_os_error()
        ));
    }

    let size = length_info.length as u64;
    info!("Device {} size: {} bytes", device_path, size);
    Ok(size)
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn device_size_impl(device_path: &str) -> Result<u64, String> {
    Err(format!(
        "Device size detection is not supported on this platform. Device: {}",
        device_path
    ))
}

// =============================================================================
// Physical Disk Enumeration
// =============================================================================

/// A physical disk (not just a partition/volume) on the system.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalDisk {
    /// Raw device path (e.g. "/dev/rdisk2" on macOS, "/dev/sda" on Linux)
    pub device_path: String,
    /// Whole-disk device path (partition-less, e.g. "/dev/disk2" on macOS)
    pub whole_disk_path: String,
    /// Display name / model (e.g. "Samsung SSD 980")
    pub model: String,
    /// Total size in bytes
    pub size_bytes: u64,
    /// Media type: "SSD", "HDD", "USB", "Unknown"
    pub media_type: String,
    /// Whether this is the boot disk
    pub is_boot_disk: bool,
    /// Whether this is removable (USB, SD card)
    pub is_removable: bool,
    /// Serial number if available
    pub serial: String,
    /// Partitions / volumes on this disk
    pub partitions: Vec<String>,
}

/// Enumerate physical disks on the system.
///
/// Unlike `list_drives` which returns mounted *volumes*, this returns *physical
/// disks* — the actual hardware devices. Each physical disk may have multiple
/// partitions/volumes.
///
/// - **macOS**: Parses `diskutil list -plist` for whole disks
/// - **Linux**: Reads `/sys/block/` entries
/// - **Windows**: Uses `SetupDi` or WMI (future)
#[tauri::command]
pub fn list_physical_disks() -> Result<Vec<PhysicalDisk>, String> {
    info!("Enumerating physical disks");
    list_physical_disks_impl()
}

#[cfg(target_os = "macos")]
fn list_physical_disks_impl() -> Result<Vec<PhysicalDisk>, String> {
    // Parse `diskutil list -plist` to find whole disks
    let output = std::process::Command::new("diskutil")
        .args(["list", "-plist"])
        .output()
        .map_err(|e| format!("Failed to run diskutil list: {e}"))?;

    if !output.status.success() {
        return Err("diskutil list failed".to_string());
    }

    // Parse the plist to extract disk identifiers
    let plist: plist::Value = plist::from_bytes(&output.stdout)
        .map_err(|e| format!("Failed to parse diskutil plist: {e}"))?;

    let mut disks = Vec::new();

    // AllDisksAndPartitions is an array of dicts, each with DeviceIdentifier
    if let Some(all_disks) = plist
        .as_dictionary()
        .and_then(|d| d.get("AllDisksAndPartitions"))
        .and_then(|v| v.as_array())
    {
        for disk_entry in all_disks {
            let dict = match disk_entry.as_dictionary() {
                Some(d) => d,
                None => continue,
            };

            let dev_id = match dict.get("DeviceIdentifier").and_then(|v| v.as_string()) {
                Some(id) => id.to_string(),
                None => continue,
            };

            // Skip synthesized/virtual (APFS containers, etc.)
            let content = dict
                .get("Content")
                .and_then(|v| v.as_string())
                .unwrap_or("");
            if content == "GUID_partition_scheme"
                || content == "Apple_APFS"
                || content == "FDisk_partition_scheme"
            {
                // This is a real disk (has a partition scheme)
            } else if content.is_empty() {
                // Might be synthesized — check via diskutil info
            }

            // Get info for this whole disk
            let disk_path = format!("/dev/{}", dev_id);
            let raw_path = format!("/dev/r{}", dev_id);

            // Query detailed info
            let info_output = std::process::Command::new("diskutil")
                .args(["info", "-plist", &disk_path])
                .output();

            let (model, serial, media_type, size_bytes, is_removable, is_boot) =
                if let Ok(info_out) = info_output {
                    if let Ok(info_plist) = plist::from_bytes::<plist::Value>(&info_out.stdout) {
                        let info_dict = info_plist.as_dictionary();
                        let model = info_dict
                            .and_then(|d| d.get("MediaName"))
                            .and_then(|v| v.as_string())
                            .unwrap_or("Unknown")
                            .to_string();
                        let serial = info_dict
                            .and_then(|d| d.get("SerialNumber"))
                            .and_then(|v| v.as_string())
                            .unwrap_or("")
                            .to_string();
                        let is_ssd = info_dict
                            .and_then(|d| d.get("SolidState"))
                            .and_then(|v| v.as_boolean())
                            .unwrap_or(false);
                        let removable = info_dict
                            .and_then(|d| d.get("Removable"))
                            .or_else(|| info_dict.and_then(|d| d.get("RemovableMedia")))
                            .and_then(|v| v.as_boolean())
                            .unwrap_or(false);
                        // External is often more reliable than Removable for USB drives
                        let external = info_dict
                            .and_then(|d| d.get("External"))
                            .and_then(|v| v.as_boolean())
                            .unwrap_or(false);
                        let size = info_dict
                            .and_then(|d| d.get("TotalSize"))
                            .or_else(|| info_dict.and_then(|d| d.get("Size")))
                            .and_then(|v| v.as_unsigned_integer())
                            .unwrap_or(0);
                        let is_internal = info_dict
                            .and_then(|d| d.get("Internal"))
                            .and_then(|v| v.as_boolean())
                            .unwrap_or(false);
                        // A rough heuristic: the boot disk is internal + not removable + disk0
                        let boot = is_internal && dev_id == "disk0";

                        let media = if is_ssd {
                            "SSD"
                        } else if external || removable {
                            "USB"
                        } else {
                            "HDD"
                        };
                        (model, serial, media.to_string(), size, removable || external, boot)
                    } else {
                        ("Unknown".to_string(), String::new(), "Unknown".to_string(), 0, false, false)
                    }
                } else {
                    ("Unknown".to_string(), String::new(), "Unknown".to_string(), 0, false, false)
                };

            // Collect partition identifiers
            let partitions: Vec<String> = dict
                .get("Partitions")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|p| {
                            p.as_dictionary()
                                .and_then(|d| d.get("DeviceIdentifier"))
                                .and_then(|v| v.as_string())
                                .map(|s| format!("/dev/{}", s))
                        })
                        .collect()
                })
                .unwrap_or_default();

            disks.push(PhysicalDisk {
                device_path: raw_path, // Use raw device for imaging
                whole_disk_path: disk_path,
                model,
                size_bytes,
                media_type,
                is_boot_disk: is_boot,
                is_removable,
                serial,
                partitions,
            });
        }
    }

    info!("Found {} physical disks", disks.len());
    Ok(disks)
}

#[cfg(target_os = "linux")]
fn list_physical_disks_impl() -> Result<Vec<PhysicalDisk>, String> {
    let mut disks = Vec::new();

    // Read /sys/block/ to find block devices
    let entries = std::fs::read_dir("/sys/block")
        .map_err(|e| format!("Cannot read /sys/block: {e}"))?;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();

        // Skip virtual devices (loop, dm, ram, zram, etc.)
        if name.starts_with("loop")
            || name.starts_with("dm-")
            || name.starts_with("ram")
            || name.starts_with("zram")
            || name.starts_with("sr")
            || name.starts_with("nbd")
        {
            continue;
        }

        let sys_path = format!("/sys/block/{}", name);
        let device_path = format!("/dev/{}", name);

        // Read size (in 512-byte sectors)
        let size_bytes = std::fs::read_to_string(format!("{}/size", sys_path))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map(|sectors| sectors * 512)
            .unwrap_or(0);

        // Skip zero-size devices
        if size_bytes == 0 {
            continue;
        }

        // Read model
        let model = std::fs::read_to_string(format!("{}/device/model", sys_path))
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string();

        // Read serial
        let serial = std::fs::read_to_string(format!("{}/device/serial", sys_path))
            .unwrap_or_default()
            .trim()
            .to_string();

        // Detect SSD vs HDD
        let rotational = std::fs::read_to_string(format!("{}/queue/rotational", sys_path))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(1);

        let removable = std::fs::read_to_string(format!("{}/removable", sys_path))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0) == 1;

        let media_type = if removable {
            "USB"
        } else if rotational == 0 {
            "SSD"
        } else {
            "HDD"
        };

        // List partitions
        let partitions: Vec<String> = std::fs::read_dir(&sys_path)
            .ok()
            .map(|rd| {
                rd.flatten()
                    .filter_map(|e| {
                        let pname = e.file_name().to_string_lossy().into_owned();
                        if pname.starts_with(&name) && pname != name {
                            Some(format!("/dev/{}", pname))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Boot disk heuristic: check if "/" is on one of our partitions
        let is_boot = is_boot_disk_linux(&device_path, &partitions);

        disks.push(PhysicalDisk {
            device_path: device_path.clone(),
            whole_disk_path: device_path,
            model,
            size_bytes,
            media_type: media_type.to_string(),
            is_boot_disk: is_boot,
            is_removable: removable,
            serial,
            partitions,
        });
    }

    info!("Found {} physical disks", disks.len());
    Ok(disks)
}

#[cfg(target_os = "linux")]
fn is_boot_disk_linux(device_path: &str, partitions: &[String]) -> bool {
    // Check /proc/mounts for the root filesystem
    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "/" {
                let root_dev = parts[0];
                // Check if root device is this disk or one of its partitions
                if root_dev == device_path || partitions.iter().any(|p| p == root_dev) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(target_os = "windows")]
fn list_physical_disks_impl() -> Result<Vec<PhysicalDisk>, String> {
    // Enumerate physical drives by probing \\.\PhysicalDrive0, 1, 2, ...
    let mut disks = Vec::new();

    for drive_num in 0..32 {
        let device_path = format!("\\\\.\\PhysicalDrive{}", drive_num);

        // Try to open the drive — if it fails, no more drives
        let file = match std::fs::File::open(&device_path) {
            Ok(f) => f,
            Err(_) => continue,
        };

        // Try to get size
        let size_bytes = device_size_impl(&device_path).unwrap_or(0);
        drop(file);

        if size_bytes == 0 {
            continue;
        }

        disks.push(PhysicalDisk {
            device_path: device_path.clone(),
            whole_disk_path: device_path,
            model: format!("Physical Drive {}", drive_num),
            size_bytes,
            media_type: "Unknown".to_string(),
            is_boot_disk: drive_num == 0, // Heuristic: drive 0 is usually boot
            is_removable: false,
            serial: String::new(),
            partitions: Vec::new(),
        });
    }

    info!("Found {} physical disks", disks.len());
    Ok(disks)
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn list_physical_disks_impl() -> Result<Vec<PhysicalDisk>, String> {
    Err("Physical disk enumeration is not supported on this platform.".to_string())
}

// =============================================================================
// Elevation Request
// =============================================================================

/// Request that the application be restarted with elevated privileges.
///
/// This uses platform-specific mechanisms:
/// - **macOS**: `osascript` to prompt for admin authentication
/// - **Linux**: `pkexec` to run via PolicyKit
/// - **Windows**: `ShellExecuteW` with "runas" verb
///
/// Returns a message describing the outcome. The actual restart is handled
/// by asking the user to relaunch — we don't automatically restart.
#[tauri::command]
pub fn request_elevation() -> Result<String, String> {
    info!("Elevation requested");

    if privilege::user::privileged() {
        return Ok("Already running with elevated privileges.".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, we can't easily re-exec as root from a .app bundle.
        // Instead, return instructions for the user.
        Ok(
            "To access raw devices on macOS, run CORE-FFX from Terminal with sudo:\n\
             sudo /Applications/CORE-FFX.app/Contents/MacOS/CORE-FFX\n\n\
             Alternatively, use 'diskutil mount readOnly' to mount drives read-only \
             before imaging (no elevation required for removable drives)."
                .to_string(),
        )
    }

    #[cfg(target_os = "linux")]
    {
        Ok(
            "To access raw devices on Linux, run CORE-FFX with pkexec or as root:\n\
             pkexec /usr/bin/core-ffx\n\
             or: sudo core-ffx\n\n\
             Alternatively, add your user to the 'disk' group:\n\
             sudo usermod -aG disk $USER\n\
             (requires logout/login to take effect)."
                .to_string(),
        )
    }

    #[cfg(target_os = "windows")]
    {
        Ok(
            "To access raw physical drives on Windows, right-click the CORE-FFX \
             shortcut and select 'Run as administrator'.\n\n\
             Alternatively, use an elevated Command Prompt:\n\
             runas /user:Administrator \"C:\\Program Files\\CORE-FFX\\CORE-FFX.exe\""
                .to_string(),
        )
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("Elevation is not supported on this platform.".to_string())
    }
}

// =============================================================================
// Raw Device Streaming (for forensic imaging)
// =============================================================================

/// Progress event emitted during raw device reading.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceReadProgress {
    pub device_path: String,
    pub bytes_read: u64,
    pub total_bytes: u64,
    pub percent: f64,
}

/// Read raw bytes from a device into a temporary file, streaming with progress.
///
/// This is the low-level building block for physical disk imaging.
/// The caller (e.g., `acquire_physical_image`) uses the output file
/// to feed into EwfWriter or other containers.
///
/// Returns the path to the temporary file containing the raw device data.
/// For large devices, this streams in chunks to avoid memory exhaustion.
#[tauri::command]
pub async fn read_raw_device(
    device_path: String,
    output_path: String,
    window: tauri::Window,
) -> Result<u64, String> {
    use std::io::{Read, Write};
    use tauri::Emitter;

    info!(
        "Starting raw device read: {} -> {}",
        device_path, output_path
    );

    // Privilege check
    if !privilege::user::privileged() {
        return Err(
            "Raw device access requires elevated privileges. \
             Restart CORE-FFX with administrator/root permissions."
                .to_string(),
        );
    }

    // Get device size first
    let total_bytes = device_size_impl(&device_path)?;
    if total_bytes == 0 {
        return Err(format!("Device {} reports zero size", device_path));
    }

    // spawn_blocking for the heavy I/O
    let window_clone = window.clone();
    let device = device_path.clone();
    let output = output_path.clone();

    tokio::task::spawn_blocking(move || {
        let mut source = std::fs::File::open(&device).map_err(|e| {
            format!(
                "Cannot open device {}: {}. Ensure you have root/admin privileges.",
                device, e
            )
        })?;

        let mut dest = std::fs::File::create(&output)
            .map_err(|e| format!("Cannot create output file {}: {}", output, e))?;

        const CHUNK_SIZE: usize = 1024 * 1024; // 1 MB chunks
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut bytes_read_total: u64 = 0;
        let mut last_percent: f64 = -1.0;

        loop {
            let n = source
                .read(&mut buffer)
                .map_err(|e| format!("Read error at byte {}: {}", bytes_read_total, e))?;

            if n == 0 {
                break;
            }

            dest.write_all(&buffer[..n])
                .map_err(|e| format!("Write error at byte {}: {}", bytes_read_total, e))?;

            bytes_read_total += n as u64;

            // Emit progress at 0.5% granularity
            let percent = (bytes_read_total as f64 / total_bytes as f64 * 100.0).min(100.0);
            if percent - last_percent >= 0.5 || bytes_read_total >= total_bytes {
                let _ = window_clone.emit(
                    "device-read-progress",
                    DeviceReadProgress {
                        device_path: device.clone(),
                        bytes_read: bytes_read_total,
                        total_bytes,
                        percent,
                    },
                );
                last_percent = percent;
            }
        }

        dest.flush()
            .map_err(|e| format!("Flush error: {}", e))?;

        info!(
            "Raw device read complete: {} bytes from {}",
            bytes_read_total, device
        );

        Ok::<u64, String>(bytes_read_total)
    })
    .await
    .map_err(|e| format!("Device read task failed: {e}"))?
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // PrivilegeInfo struct + serialization
    // -------------------------------------------------------------------------

    #[test]
    fn check_privilege_returns_valid_info() {
        let info = check_privilege();
        // username should never be empty on any platform
        assert!(!info.username.is_empty(), "username must not be empty");
        // description should mention either "elevated" or "standard"
        assert!(
            info.description.contains("elevated") || info.description.contains("standard"),
            "description must mention privilege level"
        );
        // elevation_required is the inverse of is_elevated
        assert_eq!(info.elevation_required, !info.is_elevated);
    }

    #[test]
    fn privilege_info_serializes_camel_case() {
        let info = PrivilegeInfo {
            is_elevated: false,
            description: "test".to_string(),
            username: "user1".to_string(),
            elevation_required: true,
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["isElevated"], false);
        assert_eq!(json["description"], "test");
        assert_eq!(json["username"], "user1");
        assert_eq!(json["elevationRequired"], true);
    }

    // -------------------------------------------------------------------------
    // PhysicalDisk serialization
    // -------------------------------------------------------------------------

    #[test]
    fn physical_disk_serializes_camel_case() {
        let disk = PhysicalDisk {
            device_path: "/dev/rdisk2".to_string(),
            whole_disk_path: "/dev/disk2".to_string(),
            model: "Samsung SSD 980".to_string(),
            size_bytes: 500_000_000_000,
            media_type: "SSD".to_string(),
            is_boot_disk: false,
            is_removable: true,
            serial: "S123456".to_string(),
            partitions: vec!["/dev/disk2s1".to_string(), "/dev/disk2s2".to_string()],
        };
        let json = serde_json::to_value(&disk).unwrap();
        assert_eq!(json["devicePath"], "/dev/rdisk2");
        assert_eq!(json["wholeDiskPath"], "/dev/disk2");
        assert_eq!(json["sizeBytes"], 500_000_000_000u64);
        assert_eq!(json["mediaType"], "SSD");
        assert_eq!(json["isBootDisk"], false);
        assert_eq!(json["isRemovable"], true);
        assert_eq!(json["serial"], "S123456");
        assert_eq!(json["partitions"].as_array().unwrap().len(), 2);
    }

    // -------------------------------------------------------------------------
    // DeviceReadProgress serialization
    // -------------------------------------------------------------------------

    #[test]
    fn device_read_progress_serializes_camel_case() {
        let progress = DeviceReadProgress {
            device_path: "/dev/rdisk2".to_string(),
            bytes_read: 1_000_000,
            total_bytes: 500_000_000_000,
            percent: 0.0002,
        };
        let json = serde_json::to_value(&progress).unwrap();
        assert_eq!(json["devicePath"], "/dev/rdisk2");
        assert_eq!(json["bytesRead"], 1_000_000u64);
        assert_eq!(json["totalBytes"], 500_000_000_000u64);
        assert!(json["percent"].as_f64().unwrap() < 1.0);
    }

    // -------------------------------------------------------------------------
    // Device size — non-existent device
    // -------------------------------------------------------------------------

    #[test]
    fn get_device_size_nonexistent_returns_error() {
        let result = get_device_size("/dev/nonexistent_device_xyz".to_string());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Cannot open device"),
            "error should mention cannot open: {}",
            err
        );
    }

    // -------------------------------------------------------------------------
    // Physical disk enumeration (smoke test)
    // -------------------------------------------------------------------------

    #[test]
    fn list_physical_disks_does_not_panic() {
        // This test just verifies the function runs without panicking.
        // On CI or restricted environments it may return an empty list or error.
        let result = list_physical_disks();
        match result {
            Ok(disks) => {
                // Each disk should have a non-empty device_path
                for disk in &disks {
                    assert!(!disk.device_path.is_empty());
                    assert!(disk.size_bytes > 0 || disk.model != "Unknown");
                }
            }
            Err(e) => {
                // On some sandboxed CI environments, diskutil may not be available
                eprintln!("list_physical_disks returned error (may be expected in CI): {}", e);
            }
        }
    }

    // -------------------------------------------------------------------------
    // request_elevation — when not elevated, returns platform-specific instructions
    // -------------------------------------------------------------------------

    #[test]
    fn request_elevation_returns_instructions() {
        let result = request_elevation();
        assert!(result.is_ok(), "request_elevation should not fail");
        let msg = result.unwrap();
        // Should contain some guidance regardless of platform
        assert!(
            !msg.is_empty(),
            "elevation instructions should not be empty"
        );
        // On any platform, the response should mention CORE-FFX or "elevated"
        assert!(
            msg.contains("CORE-FFX") || msg.contains("elevated") || msg.contains("privileges"),
            "message should contain relevant guidance: {}",
            msg
        );
    }

    // -------------------------------------------------------------------------
    // check_privilege consistency
    // -------------------------------------------------------------------------

    #[test]
    fn check_privilege_is_consistent_across_calls() {
        let info1 = check_privilege();
        let info2 = check_privilege();
        // Privilege level should not change between calls
        assert_eq!(info1.is_elevated, info2.is_elevated);
        assert_eq!(info1.username, info2.username);
        assert_eq!(info1.elevation_required, info2.elevation_required);
    }
}
