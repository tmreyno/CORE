// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Memory capture commands for live RAM acquisition.
//!
//! Platform support:
//! - **Linux**: Direct reading of `/proc/kcore` with ELF header parsing to
//!   extract physical memory ranges identified via `/proc/iomem`.
//! - **Windows**: Locates and invokes WinPmem (`winpmem_mini_x64.exe`) to
//!   capture physical memory through a signed kernel driver.
//! - **macOS**: Not supported — System Integrity Protection (SIP) blocks
//!   kernel memory access since macOS Catalina (10.15).

#![allow(unused_imports)]

use super::ewf_helpers::validate_snapshot_byte_count;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;
use tauri::Emitter;
use tracing::{info, warn};

/// Cancel flag for memory capture operations.
static MEMORY_CANCEL_FLAG: LazyLock<AtomicBool> = LazyLock::new(|| AtomicBool::new(false));

#[cfg(any(target_os = "windows", test))]
const PROCESS_OUTPUT_MAX_BYTES: u64 = 1024 * 1024;

#[cfg(any(target_os = "windows", test))]
fn read_process_output_with_limit<R: std::io::Read>(reader: R) -> String {
    use std::io::Read;

    let mut limited = reader.take(PROCESS_OUTPUT_MAX_BYTES.saturating_add(1));
    let mut bytes = Vec::new();
    if limited.read_to_end(&mut bytes).is_err() {
        return String::new();
    }

    let truncated = bytes.len() as u64 > PROCESS_OUTPUT_MAX_BYTES;
    if truncated {
        bytes.truncate(PROCESS_OUTPUT_MAX_BYTES as usize);
    }

    let mut output = String::from_utf8_lossy(&bytes).into_owned();
    if truncated {
        output.push_str("\n[process output truncated]");
    }
    output
}

// =============================================================================
// Types
// =============================================================================

/// Information about system memory and capture capability.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryCaptureInfo {
    pub total_memory_bytes: u64,
    pub available_memory_bytes: u64,
    pub platform: String,
    pub capture_supported: bool,
    pub capture_method: String,
    pub requires_elevation: bool,
    pub elevation_instructions: String,
    pub unsupported_reason: Option<String>,
}

/// Progress during memory capture.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryCaptureProgress {
    pub bytes_captured: u64,
    pub total_bytes: u64,
    pub percent: f64,
    pub phase: String,
}

/// Result of a completed memory capture.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryCaptureResult {
    pub output_path: String,
    pub bytes_captured: u64,
    pub total_bytes: u64,
    pub duration_secs: f64,
    pub hash_md5: Option<String>,
    pub hash_sha256: Option<String>,
}

// =============================================================================
// Tauri Commands
// =============================================================================

/// Query system memory information and capture support.
#[tauri::command]
pub async fn memory_capture_info() -> Result<MemoryCaptureInfo, String> {
    let (total, available) = get_memory_sizes();

    #[cfg(target_os = "linux")]
    {
        let kcore_exists = std::path::Path::new("/proc/kcore").exists();
        let is_root = privilege::user::privileged();

        return Ok(MemoryCaptureInfo {
            total_memory_bytes: total,
            available_memory_bytes: available,
            platform: "linux".to_string(),
            capture_supported: kcore_exists,
            capture_method: if kcore_exists {
                "/proc/kcore (ELF)".to_string()
            } else {
                String::new()
            },
            requires_elevation: !is_root,
            elevation_instructions: if !is_root {
                "Run as root (sudo) to capture memory.".to_string()
            } else {
                String::new()
            },
            unsupported_reason: if !kcore_exists {
                Some("/proc/kcore is not available on this system.".to_string())
            } else {
                None
            },
        });
    }

    #[cfg(target_os = "windows")]
    {
        let tool_path = find_winpmem();
        let is_admin = privilege::user::privileged();

        return Ok(MemoryCaptureInfo {
            total_memory_bytes: total,
            available_memory_bytes: available,
            platform: "windows".to_string(),
            capture_supported: tool_path.is_some(),
            capture_method: tool_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            requires_elevation: !is_admin,
            elevation_instructions: if !is_admin {
                "Run as Administrator to capture memory.".to_string()
            } else {
                String::new()
            },
            unsupported_reason: if tool_path.is_none() {
                Some(
                    "WinPmem not found. Place winpmem_mini_x64.exe in the same \
                     directory as the application, or add it to PATH."
                        .to_string(),
                )
            } else {
                None
            },
        });
    }

    #[cfg(target_os = "macos")]
    {
        return Ok(MemoryCaptureInfo {
            total_memory_bytes: total,
            available_memory_bytes: available,
            platform: "macos".to_string(),
            capture_supported: false,
            capture_method: String::new(),
            requires_elevation: false,
            elevation_instructions: String::new(),
            unsupported_reason: Some(
                "Memory capture is not supported on macOS. System Integrity \
                 Protection (SIP) blocks kernel memory access since macOS \
                 Catalina (10.15). Use a target machine running Linux or \
                 Windows, or disable SIP and use osxpmem."
                    .to_string(),
            ),
        });
    }

    // Fallback for other platforms
    #[allow(unreachable_code)]
    Ok(MemoryCaptureInfo {
        total_memory_bytes: total,
        available_memory_bytes: available,
        platform: std::env::consts::OS.to_string(),
        capture_supported: false,
        capture_method: String::new(),
        requires_elevation: false,
        elevation_instructions: String::new(),
        unsupported_reason: Some("Memory capture is not supported on this platform.".to_string()),
    })
}

/// Capture system memory to a file.
///
/// Emits `"memory-capture-progress"` events with [`MemoryCaptureProgress`] payload.
#[tauri::command]
#[allow(unused_variables)]
pub async fn memory_capture(
    output_path: String,
    compute_hashes: bool,
    window: tauri::Window,
) -> Result<MemoryCaptureResult, String> {
    // Reset cancel flag
    MEMORY_CANCEL_FLAG.store(false, Ordering::Relaxed);

    // Privilege check
    if !privilege::user::privileged() {
        return Err("Memory capture requires elevated privileges. \
             Run as root (Linux) or Administrator (Windows)."
            .to_string());
    }

    #[cfg(target_os = "linux")]
    {
        return capture_linux(&output_path, compute_hashes, &window).await;
    }

    #[cfg(target_os = "windows")]
    {
        return capture_windows(&output_path, compute_hashes, &window).await;
    }

    #[cfg(target_os = "macos")]
    {
        return Err(
            "Memory capture is not supported on macOS due to System Integrity Protection (SIP)."
                .to_string(),
        );
    }

    #[allow(unreachable_code)]
    Err("Memory capture is not supported on this platform.".to_string())
}

/// Cancel a running memory capture.
#[tauri::command]
pub fn memory_capture_cancel() {
    MEMORY_CANCEL_FLAG.store(true, Ordering::Relaxed);
    info!("Memory capture cancel requested");
}

// =============================================================================
// Helpers
// =============================================================================

/// Get total and available memory via sysinfo.
fn get_memory_sizes() -> (u64, u64) {
    use sysinfo::System;
    let mut sys = System::new();
    sys.refresh_memory();
    (sys.total_memory(), sys.available_memory())
}

// =============================================================================
// Linux: /proc/kcore + /proc/iomem
// =============================================================================

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::io::{Read, Seek, SeekFrom, Write};

    /// A physical memory range from /proc/iomem.
    struct MemRange {
        start: u64,
        end: u64, // inclusive
    }

    /// Minimal ELF64 program header fields.
    struct Elf64Phdr {
        p_offset: u64,
        p_paddr: u64,
        p_filesz: u64,
    }

    const PT_LOAD: u32 = 1;
    const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

    /// Parse /proc/iomem for "System RAM" ranges.
    fn parse_iomem() -> Result<Vec<MemRange>, String> {
        let content = std::fs::read_to_string("/proc/iomem")
            .map_err(|e| format!("Cannot read /proc/iomem: {}. Root access required.", e))?;

        let mut ranges = Vec::new();
        for line in content.lines() {
            // Lines look like: "00001000-0009ffff : System RAM"
            if !line.contains("System RAM") {
                continue;
            }
            let addr_part = line.split(':').next().unwrap_or("").trim();
            let parts: Vec<&str> = addr_part.split('-').collect();
            if parts.len() != 2 {
                continue;
            }
            if let (Ok(start), Ok(end)) = (
                u64::from_str_radix(parts[0].trim(), 16),
                u64::from_str_radix(parts[1].trim(), 16),
            ) {
                // Only include ranges that are non-trivial
                if end > start {
                    ranges.push(MemRange { start, end });
                }
            }
        }

        Ok(ranges)
    }

    /// Parse ELF64 headers from /proc/kcore to get LOAD segment offsets.
    fn parse_kcore_headers(file: &mut std::fs::File) -> Result<Vec<Elf64Phdr>, String> {
        file.seek(SeekFrom::Start(0))
            .map_err(|e| format!("Cannot seek kcore: {}", e))?;

        // Read ELF header (64 bytes)
        let mut ehdr = [0u8; 64];
        file.read_exact(&mut ehdr)
            .map_err(|e| format!("Cannot read ELF header: {}", e))?;

        // Verify ELF magic
        if ehdr[0..4] != ELF_MAGIC {
            return Err("Invalid ELF magic in /proc/kcore".to_string());
        }
        // Verify 64-bit
        if ehdr[4] != 2 {
            return Err("/proc/kcore is not 64-bit ELF".to_string());
        }

        // Determine endianness (ehdr[5]: 1=LE, 2=BE)
        let le = ehdr[5] == 1;

        let read_u16 = |buf: &[u8], off: usize| -> u16 {
            if le {
                u16::from_le_bytes([buf[off], buf[off + 1]])
            } else {
                u16::from_be_bytes([buf[off], buf[off + 1]])
            }
        };
        let read_u32 = |buf: &[u8], off: usize| -> u32 {
            if le {
                u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
            } else {
                u32::from_be_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
            }
        };
        let read_u64 = |buf: &[u8], off: usize| -> u64 {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&buf[off..off + 8]);
            if le {
                u64::from_le_bytes(bytes)
            } else {
                u64::from_be_bytes(bytes)
            }
        };

        // e_phoff (offset 32), e_phentsize (offset 54), e_phnum (offset 56)
        let e_phoff = read_u64(&ehdr, 32);
        let e_phentsize = read_u16(&ehdr, 54) as usize;
        let e_phnum = read_u16(&ehdr, 56) as usize;

        if e_phentsize < 56 {
            return Err(format!(
                "ELF program header too small: {} bytes",
                e_phentsize
            ));
        }

        // Read program headers
        file.seek(SeekFrom::Start(e_phoff))
            .map_err(|e| format!("Cannot seek to program headers: {}", e))?;

        let mut segments = Vec::new();
        let mut phdr_buf = vec![0u8; e_phentsize];

        for _ in 0..e_phnum {
            file.read_exact(&mut phdr_buf)
                .map_err(|e| format!("Cannot read program header: {}", e))?;

            let p_type = read_u32(&phdr_buf, 0);
            if p_type == PT_LOAD {
                segments.push(Elf64Phdr {
                    p_offset: read_u64(&phdr_buf, 8),
                    p_paddr: read_u64(&phdr_buf, 24),
                    p_filesz: read_u64(&phdr_buf, 32),
                });
            }
        }

        Ok(segments)
    }

    /// Find the kcore segment that contains the given physical address range.
    fn find_segment_for_range(
        segments: &[Elf64Phdr],
        phys_start: u64,
        phys_end: u64,
    ) -> Option<&Elf64Phdr> {
        segments.iter().find(|seg| {
            let seg_phys_start = seg.p_paddr;
            let seg_phys_end = seg.p_paddr.saturating_add(seg.p_filesz);
            seg_phys_start <= phys_start && seg_phys_end >= phys_end
        })
    }

    pub async fn capture_linux(
        output_path: &str,
        compute_hashes: bool,
        window: &tauri::Window,
    ) -> Result<MemoryCaptureResult, String> {
        let output = output_path.to_string();
        let win = window.clone();

        tokio::task::spawn_blocking(move || capture_linux_blocking(&output, compute_hashes, &win))
            .await
            .map_err(|e| format!("Memory capture task failed: {}", e))?
    }

    fn capture_linux_blocking(
        output_path: &str,
        compute_hashes: bool,
        window: &tauri::Window,
    ) -> Result<MemoryCaptureResult, String> {
        use md5::Digest as _;

        let start_time = std::time::Instant::now();

        // Emit initial progress
        let _ = window.emit(
            "memory-capture-progress",
            MemoryCaptureProgress {
                bytes_captured: 0,
                total_bytes: 0,
                percent: 0.0,
                phase: "parsing".to_string(),
            },
        );

        // 1. Parse /proc/iomem for System RAM ranges
        let ram_ranges = parse_iomem()?;
        if ram_ranges.is_empty() {
            return Err("No System RAM ranges found in /proc/iomem. \
                 Addresses may be hidden — root access is required."
                .to_string());
        }

        let total_bytes: u64 = ram_ranges.iter().map(|r| r.end - r.start + 1).sum();
        let max_addr = ram_ranges.last().map(|r| r.end + 1).unwrap_or(0);

        info!(
            "Memory capture: {} RAM ranges, {} total bytes, max addr 0x{:x}",
            ram_ranges.len(),
            total_bytes,
            max_addr
        );

        // 2. Open /proc/kcore and parse ELF headers
        let mut kcore = std::fs::File::open("/proc/kcore")
            .map_err(|e| format!("Cannot open /proc/kcore: {}. Root access required.", e))?;

        let segments = parse_kcore_headers(&mut kcore)?;
        info!("Memory capture: {} LOAD segments in kcore", segments.len());

        // 3. Create output file
        let mut output = std::fs::File::create(output_path)
            .map_err(|e| format!("Cannot create output file: {}", e))?;

        // Allocate as sparse file
        output
            .set_len(max_addr)
            .map_err(|e| format!("Cannot set output file size to {}: {}", max_addr, e))?;

        // 4. Set up hashing
        let mut md5_hasher = if compute_hashes {
            Some(md5::Md5::new())
        } else {
            None
        };
        let mut sha256_hasher = if compute_hashes {
            Some(sha2::Sha256::new())
        } else {
            None
        };

        // 5. Read ranges from kcore and write to output
        const CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4 MB — larger reads reduce syscall overhead
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut bytes_captured: u64 = 0;
        let mut last_percent: f64 = -1.0;

        for range in &ram_ranges {
            if MEMORY_CANCEL_FLAG.load(Ordering::Relaxed) {
                // Clean up partial file
                let _ = std::fs::remove_file(output_path);
                return Err("Memory capture cancelled.".to_string());
            }

            let range_size = range.end - range.start + 1;

            // Find the kcore segment covering this range
            let segment = find_segment_for_range(&segments, range.start, range.end);

            if let Some(seg) = segment {
                let file_offset = seg.p_offset + (range.start - seg.p_paddr);

                kcore
                    .seek(SeekFrom::Start(file_offset))
                    .map_err(|e| format!("Seek error in kcore at 0x{:x}: {}", file_offset, e))?;

                output
                    .seek(SeekFrom::Start(range.start))
                    .map_err(|e| format!("Seek error in output at 0x{:x}: {}", range.start, e))?;

                let mut remaining = range_size;
                while remaining > 0 {
                    if MEMORY_CANCEL_FLAG.load(Ordering::Relaxed) {
                        let _ = std::fs::remove_file(output_path);
                        return Err("Memory capture cancelled.".to_string());
                    }

                    let to_read = (remaining as usize).min(CHUNK_SIZE);
                    let n = kcore
                        .read(&mut buffer[..to_read])
                        .map_err(|e| format!("Read error from kcore: {}", e))?;

                    if n == 0 {
                        let range_captured = range_size.saturating_sub(remaining);
                        validate_snapshot_byte_count(
                            "Memory capture range",
                            std::path::Path::new("/proc/kcore"),
                            range_size,
                            range_captured,
                        )?;
                        break;
                    }

                    output
                        .write_all(&buffer[..n])
                        .map_err(|e| format!("Write error: {}", e))?;

                    if let Some(ref mut h) = md5_hasher {
                        md5::Digest::update(h, &buffer[..n]);
                    }
                    if let Some(ref mut h) = sha256_hasher {
                        sha2::Digest::update(h, &buffer[..n]);
                    }

                    bytes_captured += n as u64;
                    remaining -= n as u64;

                    // Emit progress at 0.5% granularity
                    let percent = (bytes_captured as f64 / total_bytes as f64 * 100.0).min(100.0);
                    if percent - last_percent >= 0.5 || remaining == 0 {
                        let _ = window.emit(
                            "memory-capture-progress",
                            MemoryCaptureProgress {
                                bytes_captured,
                                total_bytes,
                                percent,
                                phase: "capturing".to_string(),
                            },
                        );
                        last_percent = percent;
                    }
                }
            } else {
                // No kcore segment for this range — write zeros (gap/hole)
                output
                    .seek(SeekFrom::Start(range.start))
                    .map_err(|e| format!("Seek error in output: {}", e))?;

                let zero_buf = vec![0u8; CHUNK_SIZE];
                let mut remaining = range_size;
                while remaining > 0 {
                    let to_write = (remaining as usize).min(CHUNK_SIZE);
                    output
                        .write_all(&zero_buf[..to_write])
                        .map_err(|e| format!("Write error: {}", e))?;

                    if let Some(ref mut h) = md5_hasher {
                        md5::Digest::update(h, &zero_buf[..to_write]);
                    }
                    if let Some(ref mut h) = sha256_hasher {
                        sha2::Digest::update(h, &zero_buf[..to_write]);
                    }

                    bytes_captured += to_write as u64;
                    remaining -= to_write as u64;
                }

                warn!(
                    "No kcore segment for RAM 0x{:x}-0x{:x}, wrote zeros",
                    range.start, range.end
                );
            }
        }

        validate_snapshot_byte_count(
            "Memory capture total",
            std::path::Path::new(output_path),
            total_bytes,
            bytes_captured,
        )?;

        output.flush().map_err(|e| format!("Flush error: {}", e))?;

        // Final progress
        let _ = window.emit(
            "memory-capture-progress",
            MemoryCaptureProgress {
                bytes_captured,
                total_bytes,
                percent: 100.0,
                phase: "complete".to_string(),
            },
        );

        let duration = start_time.elapsed().as_secs_f64();

        let hash_md5 = md5_hasher.map(|h| format!("{:x}", md5::Digest::finalize(h)));
        let hash_sha256 = sha256_hasher.map(|h| format!("{:x}", sha2::Digest::finalize(h)));

        info!(
            "Memory capture complete: {} bytes in {:.1}s",
            bytes_captured, duration
        );

        Ok(MemoryCaptureResult {
            output_path: output_path.to_string(),
            bytes_captured,
            total_bytes,
            duration_secs: duration,
            hash_md5,
            hash_sha256,
        })
    }
}

#[cfg(target_os = "linux")]
use linux::capture_linux;

// =============================================================================
// Windows: WinPmem
// =============================================================================

#[cfg(target_os = "windows")]
mod windows_capture {
    use super::*;

    pub async fn capture_windows(
        output_path: &str,
        compute_hashes: bool,
        window: &tauri::Window,
    ) -> Result<MemoryCaptureResult, String> {
        let output = output_path.to_string();
        let win = window.clone();

        tokio::task::spawn_blocking(move || capture_windows_blocking(&output, compute_hashes, &win))
            .await
            .map_err(|e| format!("Memory capture task failed: {}", e))?
    }

    fn capture_windows_blocking(
        output_path: &str,
        compute_hashes: bool,
        window: &tauri::Window,
    ) -> Result<MemoryCaptureResult, String> {
        let start_time = std::time::Instant::now();

        // Find WinPmem
        let tool_path = super::find_winpmem().ok_or_else(|| {
            "WinPmem not found. Place winpmem_mini_x64.exe in the same \
                 directory as the application, or add it to PATH."
                .to_string()
        })?;

        info!("Using WinPmem at: {}", tool_path.display());

        // Emit initial progress
        let _ = window.emit(
            "memory-capture-progress",
            MemoryCaptureProgress {
                bytes_captured: 0,
                total_bytes: 0,
                percent: 0.0,
                phase: "starting WinPmem".to_string(),
            },
        );

        // Get total memory for progress estimation
        let (total_memory, _) = super::get_memory_sizes();

        // Run WinPmem: winpmem_mini_x64.exe <output_file>
        let mut child = std::process::Command::new(&tool_path)
            .arg(output_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start WinPmem: {}", e))?;

        // Monitor the output file size for progress while WinPmem runs
        let output_path_buf = std::path::PathBuf::from(output_path);
        let poll_interval = std::time::Duration::from_millis(150);
        let mut last_percent: f64 = -1.0;

        loop {
            if MEMORY_CANCEL_FLAG.load(Ordering::Relaxed) {
                let _ = child.kill();
                let _ = std::fs::remove_file(output_path);
                return Err("Memory capture cancelled.".to_string());
            }

            match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        let stderr = child
                            .stderr
                            .take()
                            .map(read_process_output_with_limit)
                            .unwrap_or_default();
                        return Err(format!("WinPmem exited with status {}: {}", status, stderr));
                    }
                    break;
                }
                Ok(None) => {
                    // Still running — report progress based on output file size
                    if let Ok(meta) = std::fs::metadata(&output_path_buf) {
                        let written = meta.len();
                        let percent = if total_memory > 0 {
                            (written as f64 / total_memory as f64 * 100.0).min(99.0)
                        } else {
                            0.0
                        };
                        if percent - last_percent >= 0.5 {
                            let _ = window.emit(
                                "memory-capture-progress",
                                MemoryCaptureProgress {
                                    bytes_captured: written,
                                    total_bytes: total_memory,
                                    percent,
                                    phase: "capturing".to_string(),
                                },
                            );
                            last_percent = percent;
                        }
                    }
                    std::thread::sleep(poll_interval);
                }
                Err(e) => {
                    return Err(format!("Error waiting for WinPmem: {}", e));
                }
            }
        }

        // Get final file size
        let bytes_captured = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

        // Compute hashes if requested
        let (hash_md5, hash_sha256) = if compute_hashes {
            let _ = window.emit(
                "memory-capture-progress",
                MemoryCaptureProgress {
                    bytes_captured,
                    total_bytes: bytes_captured,
                    percent: 100.0,
                    phase: "hashing".to_string(),
                },
            );
            compute_file_hashes(output_path)?
        } else {
            (None, None)
        };

        // Final progress
        let _ = window.emit(
            "memory-capture-progress",
            MemoryCaptureProgress {
                bytes_captured,
                total_bytes: bytes_captured,
                percent: 100.0,
                phase: "complete".to_string(),
            },
        );

        let duration = start_time.elapsed().as_secs_f64();

        info!(
            "Memory capture complete: {} bytes in {:.1}s",
            bytes_captured, duration
        );

        Ok(MemoryCaptureResult {
            output_path: output_path.to_string(),
            bytes_captured,
            total_bytes: total_memory,
            duration_secs: duration,
            hash_md5,
            hash_sha256,
        })
    }

    /// Hash an already-written file (used after WinPmem completes).
    fn compute_file_hashes(path: &str) -> Result<(Option<String>, Option<String>), String> {
        use md5::Digest as _;
        use std::io::Read;

        let mut file =
            std::fs::File::open(path).map_err(|e| format!("Cannot open for hashing: {}", e))?;

        let mut md5_h = md5::Md5::new();
        let mut sha256_h = sha2::Sha256::new();
        let mut buf = vec![0u8; 1024 * 1024];

        loop {
            let n = file
                .read(&mut buf)
                .map_err(|e| format!("Read error during hashing: {}", e))?;
            if n == 0 {
                break;
            }
            md5::Digest::update(&mut md5_h, &buf[..n]);
            sha2::Digest::update(&mut sha256_h, &buf[..n]);
        }

        Ok((
            Some(format!("{:x}", md5::Digest::finalize(md5_h))),
            Some(format!("{:x}", sha2::Digest::finalize(sha256_h))),
        ))
    }
}

#[cfg(target_os = "windows")]
use windows_capture::capture_windows;

// =============================================================================
// macOS: Not supported
// =============================================================================

// macOS capture is handled by the memory_capture command returning an error.
// No platform-specific module needed.

// =============================================================================
// WinPmem Discovery (Windows only)
// =============================================================================

#[cfg(target_os = "windows")]
fn find_winpmem() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;

    let candidates = [
        "winpmem_mini_x64.exe",
        "winpmem.exe",
        "winpmem_mini_x86.exe",
    ];

    // 1. Check alongside the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            for name in &candidates {
                let path = exe_dir.join(name);
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    // 2. Check PATH
    for name in &candidates {
        if let Ok(output) = std::process::Command::new("where").arg(name).output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = path_str.lines().next() {
                    let path = PathBuf::from(first_line.trim());
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }
    }

    None
}

// Provide a stub on non-Windows so cfg-free code can reference it
#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
fn find_winpmem() -> Option<std::path::PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_process_output_with_limit_preserves_small_output() {
        let output = read_process_output_with_limit(Cursor::new(b"capture failed".to_vec()));
        assert_eq!(output, "capture failed");
    }

    #[test]
    fn read_process_output_with_limit_truncates_large_output() {
        let output =
            read_process_output_with_limit(Cursor::new(vec![
                b'x';
                PROCESS_OUTPUT_MAX_BYTES as usize + 1
            ]));

        assert_eq!(
            output.matches('x').count(),
            PROCESS_OUTPUT_MAX_BYTES as usize
        );
        assert!(output.ends_with("[process output truncated]"));
    }
}
