// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Nested container support - inline expansion of containers within containers.
//!
//! Provides extraction, caching, and browsing of nested containers (archives within
//! archives, forensic images within archives, etc.). Uses temp directory caching
//! to avoid repeated extraction of the same nested container.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use tracing::{debug, info, warn};

use crate::archive;
use crate::common::sanitize_filename;
use crate::common::vfs::VirtualFileSystem;
use crate::{ad1, ewf, raw, ufed};

/// Nested container entry information
/// Unified type that works for any nested container type (archive, AD1, forensic image)
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NestedContainerEntry {
    /// Path within the nested container
    pub path: String,
    /// Filename only
    pub name: String,
    /// Whether this is a directory
    pub is_dir: bool,
    /// Uncompressed size
    pub size: u64,
    /// Hash/checksum if available (CRC32 for archives, MD5/SHA for AD1)
    pub hash: Option<String>,
    /// Last modified timestamp
    pub modified: Option<String>,
    /// Type of nested container this entry is from
    pub source_type: String,
    /// Whether this entry is itself a nested container
    pub is_nested_container: bool,
    /// Container type if this is a nested container
    pub nested_type: Option<String>,
}

/// Nested container info returned when first accessed
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NestedContainerInfo {
    /// Type of container (zip, ad1, e01, etc.)
    pub container_type: String,
    /// Total entry count
    pub entry_count: usize,
    /// Total size (uncompressed)
    pub total_size: u64,
    /// Whether container is encrypted
    pub encrypted: bool,
    /// Path where container was extracted (for forensic logging)
    pub temp_path: String,
    /// Original path within parent container
    pub original_path: String,
}

/// Maximum cached nested container extractions (bounded to prevent unbounded memory growth)
const NESTED_CACHE_MAX_ENTRIES: usize = 128;
const NESTED_RANGE_MAX_BYTES: usize = 16 * 1024 * 1024;
const NESTED_STREAM_COPY_CHUNK_BYTES: usize = 1024 * 1024;

/// Cache for extracted nested containers (avoids re-extraction).
/// Bounded: evicts oldest entries when capacity is reached.
static NESTED_CONTAINER_CACHE: std::sync::LazyLock<
    parking_lot::Mutex<std::collections::HashMap<String, String>>,
> = std::sync::LazyLock::new(|| {
    parking_lot::Mutex::new(std::collections::HashMap::with_capacity(
        NESTED_CACHE_MAX_ENTRIES / 2,
    ))
});

#[cfg(test)]
pub(crate) fn nested_container_clear_cache_for_tests() {
    NESTED_CONTAINER_CACHE.lock().clear();
}

fn l01_entry_size(container_path: &str, entry_path: &str) -> Result<u64, String> {
    let tree = ewf::parse_l01_file_tree(container_path)
        .map_err(|e| format!("Failed to parse L01 file tree: {}", e))?;
    let entry = tree
        .entry_at_path(entry_path)
        .ok_or_else(|| format!("Entry not found in L01: {}", entry_path))?;

    if entry.is_directory {
        return Err(format!(
            "Cannot read directory entry from L01: {}",
            entry_path
        ));
    }

    Ok(if entry.size > 0 {
        entry.size
    } else {
        entry.data_size
    })
}

fn read_l01_entry_range(
    container_path: &str,
    entry_path: &str,
    offset: u64,
    size: usize,
) -> Result<Vec<u8>, String> {
    let tree = ewf::parse_l01_file_tree(container_path)
        .map_err(|e| format!("Failed to parse L01 file tree: {}", e))?;
    let entry = tree
        .entry_at_path(entry_path)
        .ok_or_else(|| format!("Entry not found in L01: {}", entry_path))?;

    if entry.is_directory {
        return Err(format!(
            "Cannot read directory entry from L01: {}",
            entry_path
        ));
    }

    let total_size = if entry.size > 0 {
        entry.size
    } else {
        entry.data_size
    };
    let read_size = bounded_nested_read_size(total_size, offset, size)?;
    if read_size == 0 {
        return Ok(Vec::new());
    }

    let mut handle = ewf::EwfHandle::open(container_path)
        .map_err(|e| format!("Failed to open L01 handle: {}", e))?;
    let read_offset = checked_l01_entry_read_offset(entry.data_offset, offset)?;
    let data = handle
        .read_at(read_offset, read_size)
        .map_err(|e| format!("Failed to read L01 entry '{}': {}", entry_path, e))?;
    if data.len() > read_size {
        return Err(format!(
            "L01 reader returned too many bytes for '{}': requested {read_size}, received {}",
            entry_path,
            data.len()
        ));
    }
    ensure_nested_range_read_len(data.len(), read_size, "L01 reader", entry_path)?;
    Ok(data)
}

fn checked_l01_entry_read_offset(data_offset: u64, offset: u64) -> Result<u64, String> {
    data_offset
        .checked_add(offset)
        .ok_or_else(|| "Invalid L01 entry read offset".to_string())
}

fn copy_l01_entry_to_file(
    container_path: &str,
    entry_path: &str,
    output_path: &Path,
) -> Result<u64, String> {
    let tree = ewf::parse_l01_file_tree(container_path)
        .map_err(|e| format!("Failed to parse L01 file tree: {}", e))?;
    let entry = tree
        .entry_at_path(entry_path)
        .ok_or_else(|| format!("Entry not found in L01: {}", entry_path))?;

    if entry.is_directory {
        return Err(format!(
            "Cannot read directory entry from L01: {}",
            entry_path
        ));
    }

    let total_size = if entry.size > 0 {
        entry.size
    } else {
        entry.data_size
    };
    let mut handle = ewf::EwfHandle::open(container_path)
        .map_err(|e| format!("Failed to open L01 handle: {}", e))?;
    copy_chunked_to_file(output_path, total_size, entry_path, |offset, read_size| {
        let read_offset = checked_l01_entry_read_offset(entry.data_offset, offset)?;
        handle
            .read_at(read_offset, read_size)
            .map_err(|e| format!("Failed to read L01 entry '{}': {}", entry_path, e))
    })
}

fn ufed_entry_size(container_path: &str, entry_path: &str) -> Result<u64, String> {
    let vfs =
        ufed::UfedVfs::open(container_path).map_err(|e| format!("Failed to open UFED: {:?}", e))?;

    if let Ok(size) = vfs.file_size(entry_path) {
        return Ok(size);
    }

    ufed::get_tree(container_path)
        .ok()
        .and_then(|entries| {
            entries
                .into_iter()
                .find(|entry| entry.path == entry_path)
                .map(|entry| entry.size)
        })
        .ok_or_else(|| format!("Failed to determine UFED entry size: {}", entry_path))
}

fn read_ufed_entry_range(
    container_path: &str,
    entry_path: &str,
    offset: u64,
    size: usize,
) -> Result<Vec<u8>, String> {
    let vfs =
        ufed::UfedVfs::open(container_path).map_err(|e| format!("Failed to open UFED: {:?}", e))?;
    let total_size = ufed_entry_size(container_path, entry_path)?;
    let read_size = bounded_nested_read_size(total_size, offset, size)?;
    if read_size == 0 {
        return Ok(Vec::new());
    }

    let data = vfs
        .read(entry_path, offset, read_size)
        .map_err(|e| format!("Failed to read UFED entry '{}': {:?}", entry_path, e))?;
    if data.len() > read_size {
        return Err(format!(
            "UFED VFS returned too many bytes for '{}': requested {read_size}, received {}",
            entry_path,
            data.len()
        ));
    }
    ensure_nested_range_read_len(data.len(), read_size, "UFED VFS", entry_path)?;
    Ok(data)
}

fn checked_nested_range_request_size(size: usize) -> Result<usize, String> {
    if size > NESTED_RANGE_MAX_BYTES {
        return Err(format!(
            "Nested entry chunk request is too large: {size} bytes > {NESTED_RANGE_MAX_BYTES} bytes"
        ));
    }
    Ok(size)
}

fn bounded_nested_read_size(total_size: u64, offset: u64, size: usize) -> Result<usize, String> {
    let size = checked_nested_range_request_size(size)?;
    if size == 0 || offset == total_size {
        return Ok(0);
    }
    if offset > total_size {
        return Err(format!(
            "Nested entry chunk offset is beyond EOF: offset {offset} > size {total_size}"
        ));
    }
    let available = total_size - offset;
    usize::try_from(available.min(size as u64))
        .map_err(|_| "Nested entry range is too large to read".to_string())
}

fn ensure_nested_range_read_len(
    actual: usize,
    expected: usize,
    context: &str,
    entry_path: &str,
) -> Result<(), String> {
    if actual == expected {
        return Ok(());
    }

    if actual < expected {
        Err(format!(
            "{context} returned incomplete data for '{entry_path}': expected {expected} bytes, received {actual}"
        ))
    } else {
        Err(format!(
            "{context} returned too many bytes for '{entry_path}': expected {expected} bytes, received {actual}"
        ))
    }
}

fn checked_nested_copy_offset(
    offset: u64,
    chunk_len: usize,
    total_size: u64,
    context: &str,
) -> Result<u64, String> {
    let next = offset.checked_add(chunk_len as u64).ok_or_else(|| {
        format!(
            "Nested container copy byte count overflow for '{}': {} + {}",
            context, offset, chunk_len
        )
    })?;
    if next > total_size {
        return Err(format!(
            "Nested container reader returned too many bytes for '{}': copied {} of {} bytes",
            context, next, total_size
        ));
    }
    Ok(next)
}

fn copy_chunked_to_file(
    output_path: &Path,
    total_size: u64,
    context: &str,
    mut read_chunk: impl FnMut(u64, usize) -> Result<Vec<u8>, String>,
) -> Result<u64, String> {
    let copy_result = (|| {
        let mut output = File::create(output_path)
            .map_err(|e| format!("Failed to create extracted nested container: {}", e))?;
        let mut offset = 0u64;

        while offset < total_size {
            let read_size =
                bounded_nested_read_size(total_size, offset, NESTED_STREAM_COPY_CHUNK_BYTES)?;
            if read_size == 0 {
                break;
            }

            let chunk = read_chunk(offset, read_size)?;
            if chunk.is_empty() {
                return Err(format!(
                    "Short read while extracting nested container '{}': copied {} of {} bytes",
                    context, offset, total_size
                ));
            }
            if chunk.len() > read_size {
                return Err(format!(
                    "Nested container reader returned too many bytes for '{}': requested {}, received {}",
                    context,
                    read_size,
                    chunk.len()
                ));
            }

            output
                .write_all(&chunk)
                .map_err(|e| format!("Failed to write extracted nested container: {}", e))?;
            offset = checked_nested_copy_offset(offset, chunk.len(), total_size, context)?;
        }

        output
            .flush()
            .map_err(|e| format!("Failed to flush extracted nested container: {}", e))?;
        Ok(offset)
    })();

    if copy_result.is_err() {
        let _ = std::fs::remove_file(output_path);
    }

    copy_result
}

fn copy_vfs_entry_to_file(
    vfs: &dyn VirtualFileSystem,
    entry_path: &str,
    output_path: &Path,
) -> Result<u64, String> {
    let total_size = vfs
        .file_size(entry_path)
        .map_err(|e| format!("Failed to get VFS entry size '{}': {:?}", entry_path, e))?;
    copy_chunked_to_file(output_path, total_size, entry_path, |offset, read_size| {
        vfs.read(entry_path, offset, read_size)
            .map_err(|e| format!("Failed to read VFS entry '{}': {:?}", entry_path, e))
    })
}

fn copy_ad1_entry_to_file(
    container_path: &str,
    entry_path: &str,
    output_path: &Path,
) -> Result<u64, String> {
    let total_size = ad1::get_entry_info(container_path, entry_path)
        .map(|entry| entry.size)
        .map_err(|e| format!("Failed to inspect AD1 entry '{}': {}", entry_path, e))?;
    copy_chunked_to_file(output_path, total_size, entry_path, |offset, read_size| {
        ad1::read_entry_chunk(container_path, entry_path, offset, read_size)
            .map_err(|e| format!("Failed to read AD1 entry '{}': {}", entry_path, e))
    })
}

pub(crate) fn nested_container_entry_size(
    parent_container_path: &str,
    nested_container_path: &str,
    entry_path: &str,
) -> Result<u64, String> {
    let temp_path = get_or_create_nested_temp(parent_container_path, nested_container_path)?;
    let nested_type = detect_nested_container_type(nested_container_path)
        .unwrap_or_else(|| "unknown".to_string());

    match nested_type.as_str() {
        "ad1" => ad1::get_entry_info(&temp_path, entry_path)
            .map(|entry| entry.size)
            .map_err(|e| format!("Failed to inspect nested AD1 entry '{}': {}", entry_path, e)),
        "l01" => l01_entry_size(&temp_path, entry_path),
        "ufed" | "ufd" | "ufdr" | "ufdx" => ufed_entry_size(&temp_path, entry_path),
        _ => {
            let vfs = archive::ArchiveVfs::open(&temp_path)
                .map_err(|e| format!("Failed to open nested archive '{}': {}", temp_path, e))?;
            vfs.file_size(entry_path).map_err(|e| {
                format!(
                    "Failed to inspect nested archive entry '{}': {}",
                    entry_path, e
                )
            })
        }
    }
}

pub(crate) fn read_nested_container_entry_range(
    parent_container_path: &str,
    nested_container_path: &str,
    entry_path: &str,
    offset: u64,
    size: usize,
) -> Result<Vec<u8>, String> {
    let temp_path = get_or_create_nested_temp(parent_container_path, nested_container_path)?;
    let nested_type = detect_nested_container_type(nested_container_path)
        .unwrap_or_else(|| "unknown".to_string());

    match nested_type.as_str() {
        "ad1" => {
            let total_size = ad1::get_entry_info(&temp_path, entry_path)
                .map(|entry| entry.size)
                .map_err(|e| {
                    format!("Failed to inspect nested AD1 entry '{}': {}", entry_path, e)
                })?;
            let read_size = bounded_nested_read_size(total_size, offset, size)?;
            if read_size == 0 {
                return Ok(Vec::new());
            }
            let data = ad1::read_entry_chunk(&temp_path, entry_path, offset, read_size)
                .map_err(|e| format!("Failed to read nested AD1 entry '{}': {}", entry_path, e))?;
            ensure_nested_range_read_len(data.len(), read_size, "Nested AD1 reader", entry_path)?;
            Ok(data)
        }
        "l01" => read_l01_entry_range(&temp_path, entry_path, offset, size),
        "ufed" | "ufd" | "ufdr" | "ufdx" => {
            read_ufed_entry_range(&temp_path, entry_path, offset, size)
        }
        _ => {
            let vfs = archive::ArchiveVfs::open(&temp_path)
                .map_err(|e| format!("Failed to open nested archive '{}': {}", temp_path, e))?;
            let total_size = vfs.file_size(entry_path).map_err(|e| {
                format!(
                    "Failed to inspect nested archive entry '{}': {}",
                    entry_path, e
                )
            })?;
            let read_size = bounded_nested_read_size(total_size, offset, size)?;
            if read_size == 0 {
                return Ok(Vec::new());
            }
            let data = vfs.read(entry_path, offset, read_size).map_err(|e| {
                format!(
                    "Failed to read nested archive entry '{}': {}",
                    entry_path, e
                )
            })?;
            if data.len() > read_size {
                return Err(format!(
                    "Nested archive VFS returned too many bytes for '{}': requested {read_size}, received {}",
                    entry_path,
                    data.len()
                ));
            }
            ensure_nested_range_read_len(data.len(), read_size, "Nested archive VFS", entry_path)?;
            Ok(data)
        }
    }
}

/// Get or create the temp path for a nested container
pub(crate) fn get_or_create_nested_temp(
    parent_path: &str,
    nested_path: &str,
) -> Result<String, String> {
    let cache_key = format!("{}::{}", parent_path, nested_path);

    // Check cache first
    {
        let cache = NESTED_CONTAINER_CACHE.lock();
        if let Some(temp_path) = cache.get(&cache_key) {
            // Verify file still exists
            if std::path::Path::new(temp_path).exists() {
                return Ok(temp_path.clone());
            }
        }
    }

    // Extract to temp
    let temp_dir =
        crate::commands::portable::portable_temp_dir().join(crate::app_paths::NESTED_TEMP_DIR_NAME);
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    // Generate unique filename based on simple hash of path
    // Use FNV-1a hash for fast, deterministic hashing
    let hash = {
        let mut h: u64 = 0xcbf29ce484222325; // FNV offset basis
        for byte in cache_key.as_bytes() {
            h ^= *byte as u64;
            h = h.wrapping_mul(0x100000001b3); // FNV prime
        }
        format!("{:016x}", h)
    };
    let nested_filename = std::path::Path::new(nested_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("nested");
    let nested_filename = {
        let sanitized = sanitize_filename(nested_filename);
        if sanitized.is_empty() {
            "nested".to_string()
        } else {
            sanitized
        }
    };
    let unique_name = format!("{}_{}", &hash[..8], nested_filename);
    let temp_path = temp_dir.join(&unique_name);
    let temp_str = temp_path.to_string_lossy().to_string();

    // Extract the nested container based on parent container type
    // First check by file format (more reliable than extension alone)
    let is_ewf = ewf::is_ewf(parent_path).unwrap_or(false);
    let is_l01 = ewf::is_l01_file(parent_path).unwrap_or(false);
    let is_raw = raw::is_raw(parent_path).unwrap_or(false);
    let is_ad1 = ad1::is_ad1(parent_path).unwrap_or(false);
    let is_ufed = ufed::is_ufed(parent_path);

    let parent_ext = std::path::Path::new(parent_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if is_ewf || is_raw {
        // VFS parent (E01, Raw) — stream the file via VFS and write to temp
        if is_ewf {
            let vfs = ewf::vfs::EwfVfs::open(parent_path)
                .map_err(|e| format!("Failed to open E01 for nested extraction: {:?}", e))?;
            copy_vfs_entry_to_file(&vfs, nested_path, &temp_path)
                .map_err(|e| format!("Failed to stream file from E01: {}", e))?;
        } else {
            let vfs = raw::vfs::RawVfs::open_with_physical_fallback(parent_path)
                .map_err(|e| format!("Failed to open raw image for nested extraction: {:?}", e))?;
            copy_vfs_entry_to_file(&vfs, nested_path, &temp_path)
                .map_err(|e| format!("Failed to stream file from raw image: {}", e))?;
        }
    } else if is_l01 {
        copy_l01_entry_to_file(parent_path, nested_path, &temp_path)
            .map_err(|e| format!("Failed to stream file from L01: {}", e))?;
    } else if is_ufed {
        let vfs = ufed::UfedVfs::open(parent_path)
            .map_err(|e| format!("Failed to open UFED for nested extraction: {:?}", e))?;
        copy_vfs_entry_to_file(&vfs, nested_path, &temp_path)
            .map_err(|e| format!("Failed to stream file from UFED: {}", e))?;
    } else if is_ad1 {
        copy_ad1_entry_to_file(parent_path, nested_path, &temp_path)
            .map_err(|e| format!("Failed to stream file from AD1: {}", e))?;
    } else {
        // Archive parents — match by extension
        match parent_ext.as_str() {
            "zip" => {
                archive::extract_zip_entry(parent_path, nested_path, &temp_str)
                    .map_err(|e| e.to_string())?;
            }
            _ => {
                let data = crate::commands::archive::extraction::read_archive_entry_bytes(
                    parent_path,
                    nested_path,
                )
                .map_err(|e| {
                    format!(
                        "Failed to extract nested file from parent container. \
                         The parent container may use an unsupported format \
                         or the nested entry path may be invalid.\n\
                         Parent: {}\nNested: {}\nError: {}",
                        parent_path, nested_path, e
                    )
                })?;
                std::fs::write(&temp_path, data)
                    .map_err(|e| format!("Failed to write extracted file: {}", e))?;
            }
        }
    }

    // Log forensic access
    info!(
        parent = %parent_path,
        nested = %nested_path,
        temp = %temp_str,
        "Extracted nested container for inline viewing"
    );

    // Cache the result (with bounded eviction)
    {
        let mut cache = NESTED_CONTAINER_CACHE.lock();

        // Evict oldest entries if at capacity
        if cache.len() >= NESTED_CACHE_MAX_ENTRIES {
            // Remove ~25% of entries to avoid evicting on every insert
            let evict_count = NESTED_CACHE_MAX_ENTRIES / 4;
            let keys_to_remove: Vec<String> = cache.keys().take(evict_count).cloned().collect();
            for key in &keys_to_remove {
                if let Some(old_path) = cache.remove(key) {
                    // Best-effort cleanup of evicted temp files
                    let p = std::path::Path::new(&old_path);
                    if p.is_dir() {
                        let _ = std::fs::remove_dir_all(p);
                    } else {
                        let _ = std::fs::remove_file(p);
                    }
                }
            }
            info!(
                evicted = evict_count,
                remaining = cache.len(),
                "Evicted nested container cache entries"
            );
        }

        cache.insert(cache_key, temp_str.clone());
    }

    Ok(temp_str)
}

/// Detect container type from filename extension
fn detect_nested_container_type(filename: &str) -> Option<String> {
    let lower = filename.to_lowercase();

    // Forensic containers
    if lower.ends_with(".ad1") {
        return Some("ad1".to_string());
    }
    if lower.ends_with(".e01") || lower.ends_with(".ex01") {
        return Some("e01".to_string());
    }
    if lower.ends_with(".l01") || lower.ends_with(".lx01") {
        return Some("l01".to_string());
    }

    // Archives
    if lower.ends_with(".zip") {
        return Some("zip".to_string());
    }
    if lower.ends_with(".7z") {
        return Some("7z".to_string());
    }
    if lower.ends_with(".rar") {
        return Some("rar".to_string());
    }
    if lower.ends_with(".tar")
        || lower.ends_with(".tar.gz")
        || lower.ends_with(".tgz")
        || lower.ends_with(".tar.bz2")
        || lower.ends_with(".tar.xz")
    {
        return Some("tar".to_string());
    }

    // UFED
    if lower.ends_with(".ufd") || lower.ends_with(".ufdr") || lower.ends_with(".ufdx") {
        return Some("ufed".to_string());
    }

    // Disk images
    if lower.ends_with(".dmg") {
        return Some("dmg".to_string());
    }
    if lower.ends_with(".iso") {
        return Some("iso".to_string());
    }
    if lower.ends_with(".raw") || lower.ends_with(".dd") || lower.ends_with(".img") {
        return Some("raw".to_string());
    }

    None
}

/// Check if a filename is a recognized container type
fn is_container_filename(filename: &str) -> bool {
    detect_nested_container_type(filename).is_some()
}

/// Read a chunk of bytes from an entry within a nested archive
///
/// Used for HexViewer to show nested archive file contents.
/// The entry path format is "nestedArchivePath::entryPath"
///
/// For example: "inner.zip::readme.txt" reads readme.txt from inner.zip
/// which is itself inside the parent container.
#[tauri::command]
pub async fn nested_archive_read_entry_chunk(
    #[allow(non_snake_case)] containerPath: String,
    #[allow(non_snake_case)] nestedArchivePath: String,
    #[allow(non_snake_case)] entryPath: String,
    offset: u64,
    size: u64,
) -> Result<Vec<u8>, String> {
    debug!("nested_archive_read_entry_chunk: container={}, nestedArchive='{}', entry='{}', offset={}, size={}", 
           containerPath, nestedArchivePath, entryPath, offset, size);

    tauri::async_runtime::spawn_blocking(move || {
        let size = usize::try_from(size)
            .map_err(|_| "Nested archive chunk size is too large to read".to_string())?;
        let data = read_nested_container_entry_range(
            &containerPath,
            &nestedArchivePath,
            &entryPath,
            offset,
            size,
        )?;
        debug!(
            "nested_archive_read_entry_chunk: Read {} bytes from nested entry",
            data.len()
        );

        Ok(data)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get the tree of entries for a nested container
///
/// This extracts the nested container from its parent (caching the extraction)
/// and then lists its entries. Used for inline expansion in the evidence tree.
///
/// # Arguments
/// * `parentContainerPath` - Path to the outer container (e.g., "/path/to/outer.zip")
/// * `nestedEntryPath` - Path within the outer container (e.g., "folder/inner.ad1")
///
/// # Returns
/// Vector of entries from the nested container, with nested container detection
#[tauri::command]
pub async fn nested_container_get_tree(
    #[allow(non_snake_case)] parentContainerPath: String,
    #[allow(non_snake_case)] nestedEntryPath: String,
) -> Result<Vec<NestedContainerEntry>, String> {
    debug!(
        "nested_container_get_tree: parent={}, nested={}",
        parentContainerPath, nestedEntryPath
    );

    tauri::async_runtime::spawn_blocking(move || {
        // Extract nested container to temp (or get from cache)
        let temp_path = get_or_create_nested_temp(&parentContainerPath, &nestedEntryPath)?;

        // Detect the nested container type
        let nested_type =
            detect_nested_container_type(&nestedEntryPath).unwrap_or_else(|| "unknown".to_string());

        // List entries based on container type
        let entries: Vec<NestedContainerEntry> = match nested_type.as_str() {
            "ad1" => {
                // Use AD1 module
                match crate::ad1::get_tree(&temp_path) {
                    Ok(tree) => tree
                        .into_iter()
                        .map(|e| NestedContainerEntry {
                            path: e.path.clone(),
                            name: e.name.clone(),
                            is_dir: e.is_dir,
                            size: e.size,
                            // AD1 has md5_hash/sha1_hash, use md5 preferentially
                            hash: e.md5_hash.clone().or(e.sha1_hash.clone()),
                            modified: e.modified.clone(),
                            source_type: "ad1".to_string(),
                            is_nested_container: is_container_filename(&e.name),
                            nested_type: detect_nested_container_type(&e.name),
                        })
                        .collect(),
                    Err(e) => return Err(format!("Failed to read AD1 container: {}", e)),
                }
            }
            "zip" => match archive::list_zip_entries(&temp_path) {
                Ok(entries) => entries
                    .into_iter()
                    .map(|e| {
                        let name = std::path::Path::new(&e.path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&e.path)
                            .to_string();
                        NestedContainerEntry {
                            path: e.path.clone(),
                            name: name.clone(),
                            is_dir: e.is_directory,
                            size: e.size,
                            hash: Some(format!("{:08X}", e.crc32)),
                            modified: Some(e.last_modified),
                            source_type: "zip".to_string(),
                            is_nested_container: is_container_filename(&name),
                            nested_type: detect_nested_container_type(&name),
                        }
                    })
                    .collect(),
                Err(e) => return Err(format!("Failed to read ZIP container: {}", e)),
            },
            "7z" => match archive::sevenz::list_entries(&temp_path) {
                Ok(entries) => entries
                    .into_iter()
                    .map(|e| {
                        let name = std::path::Path::new(&e.path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&e.path)
                            .to_string();
                        NestedContainerEntry {
                            path: e.path.clone(),
                            name: name.clone(),
                            is_dir: e.is_directory,
                            size: e.size,
                            hash: Some(format!("{:08X}", e.crc32)),
                            modified: Some(e.last_modified),
                            source_type: "7z".to_string(),
                            is_nested_container: is_container_filename(&name),
                            nested_type: detect_nested_container_type(&name),
                        }
                    })
                    .collect(),
                Err(e) => return Err(format!("Failed to read 7z container: {}", e)),
            },
            "rar" => match archive::rar::list_entries(&temp_path) {
                Ok(entries) => entries
                    .into_iter()
                    .map(|e| {
                        let name = std::path::Path::new(&e.path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&e.path)
                            .to_string();
                        NestedContainerEntry {
                            path: e.path.clone(),
                            name: name.clone(),
                            is_dir: e.is_directory,
                            size: e.size,
                            hash: Some(format!("{:08X}", e.crc32)),
                            modified: Some(e.last_modified),
                            source_type: "rar".to_string(),
                            is_nested_container: is_container_filename(&name),
                            nested_type: detect_nested_container_type(&name),
                        }
                    })
                    .collect(),
                Err(e) => return Err(format!("Failed to read RAR container: {}", e)),
            },
            "tar" => {
                match archive::tar::list_entries(&temp_path) {
                    Ok(entries) => entries
                        .into_iter()
                        .map(|e| {
                            let name = std::path::Path::new(&e.path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(&e.path)
                                .to_string();
                            NestedContainerEntry {
                                path: e.path.clone(),
                                name: name.clone(),
                                is_dir: e.is_directory,
                                size: e.size,
                                hash: None, // TAR doesn't have checksums
                                modified: Some(e.last_modified),
                                source_type: "tar".to_string(),
                                is_nested_container: is_container_filename(&name),
                                nested_type: detect_nested_container_type(&name),
                            }
                        })
                        .collect(),
                    Err(e) => return Err(format!("Failed to read TAR container: {}", e)),
                }
            }
            "ufed" | "ufd" | "ufdr" | "ufdx" => match crate::ufed::get_tree(&temp_path) {
                Ok(entries) => entries
                    .into_iter()
                    .map(|e| NestedContainerEntry {
                        path: e.path.clone(),
                        name: e.name.clone(),
                        is_dir: e.is_dir,
                        size: e.size,
                        hash: e.hash.clone(),
                        modified: e.modified.clone(),
                        source_type: "ufed".to_string(),
                        is_nested_container: is_container_filename(&e.name),
                        nested_type: detect_nested_container_type(&e.name),
                    })
                    .collect(),
                Err(e) => return Err(format!("Failed to read UFED container: {}", e)),
            },
            "l01" => match crate::ewf::parse_l01_file_tree(&temp_path) {
                Ok(tree) => tree
                    .entries
                    .into_iter()
                    .map(|entry| NestedContainerEntry {
                        path: entry.path.clone(),
                        name: entry.name.clone(),
                        is_dir: entry.is_directory,
                        size: entry.size,
                        hash: entry.md5_hash.clone().or(entry.sha1_hash.clone()),
                        modified: if entry.modification_time != 0 {
                            Some(
                                chrono::DateTime::from_timestamp(entry.modification_time, 0)
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_default(),
                            )
                        } else {
                            None
                        },
                        source_type: "l01".to_string(),
                        is_nested_container: is_container_filename(&entry.name),
                        nested_type: detect_nested_container_type(&entry.name),
                    })
                    .collect(),
                Err(e) => return Err(format!("Failed to read L01 container: {}", e)),
            },
            _ => {
                // Try libarchive as universal fallback
                match archive::libarchive_list_all(&temp_path) {
                    Ok(entries) => entries
                        .into_iter()
                        .map(|e| NestedContainerEntry {
                            path: e.path.clone(),
                            name: e.name.clone(),
                            is_dir: e.is_dir,
                            size: e.size,
                            hash: None,
                            modified: e.mtime.map(|t| {
                                chrono::DateTime::from_timestamp(t, 0)
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_default()
                            }),
                            source_type: nested_type.clone(),
                            is_nested_container: is_container_filename(&e.name),
                            nested_type: detect_nested_container_type(&e.name),
                        })
                        .collect(),
                    Err(e) => {
                        return Err(format!(
                            "Unsupported nested container type '{}': {}",
                            nested_type, e
                        ))
                    }
                }
            }
        };

        debug!("nested_container_get_tree: found {} entries", entries.len());
        Ok(entries)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get info about a nested container without listing all entries
#[tauri::command]
pub async fn nested_container_get_info(
    #[allow(non_snake_case)] parentContainerPath: String,
    #[allow(non_snake_case)] nestedEntryPath: String,
) -> Result<NestedContainerInfo, String> {
    debug!(
        "nested_container_get_info: parent={}, nested={}",
        parentContainerPath, nestedEntryPath
    );

    tauri::async_runtime::spawn_blocking(move || {
        // Extract nested container to temp (or get from cache)
        let temp_path = get_or_create_nested_temp(&parentContainerPath, &nestedEntryPath)?;

        // Detect the nested container type
        let container_type =
            detect_nested_container_type(&nestedEntryPath).unwrap_or_else(|| "unknown".to_string());

        // Get quick info based on type
        let (entry_count, total_size, encrypted) = match container_type.as_str() {
            "zip" => {
                let meta = archive::extraction::get_zip_entry_count(&temp_path)
                    .map_err(|e| e.to_string())?;
                (meta, 0u64, false)
            }
            "7z" => match archive::sevenz::list_entries(&temp_path) {
                Ok(entries) => {
                    let total: u64 = entries.iter().map(|e| e.size).sum();
                    (entries.len(), total, false)
                }
                Err(_) => (0, 0, false),
            },
            "ad1" => {
                // Use get_stats which gathers container statistics
                match crate::ad1::get_stats(&temp_path) {
                    Ok(stats) => (stats.total_items as usize, stats.total_size, false),
                    Err(_) => (0, 0, false),
                }
            }
            "ufed" | "ufd" | "ufdr" | "ufdx" => match crate::ufed::get_tree(&temp_path) {
                Ok(entries) => {
                    let total: u64 = entries.iter().map(|entry| entry.size).sum();
                    (entries.len(), total, false)
                }
                Err(_) => (0, 0, false),
            },
            "l01" => match crate::ewf::parse_l01_file_tree(&temp_path) {
                Ok(tree) => {
                    let total: u64 = tree.entries.iter().map(|entry| entry.size).sum();
                    (tree.entries.len(), total, false)
                }
                Err(_) => (0, 0, false),
            },
            _ => (0, 0, false),
        };

        Ok(NestedContainerInfo {
            container_type,
            entry_count,
            total_size,
            encrypted,
            temp_path,
            original_path: nestedEntryPath,
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Clear the nested container cache (called on app exit or manually)
#[tauri::command]
pub async fn nested_container_clear_cache() -> Result<usize, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut cache = NESTED_CONTAINER_CACHE.lock();
        let count = cache.len();

        // Delete temp files
        for (_key, path) in cache.iter() {
            if let Err(e) = std::fs::remove_file(path) {
                warn!("Failed to remove temp file {}: {}", path, e);
            }
        }

        cache.clear();
        info!("Cleared {} nested container cache entries", count);
        Ok(count)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::vfs::{DirEntry, FileAttr, VfsError};
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    struct MockVfs {
        data: Vec<u8>,
        max_chunk: usize,
        empty_at: Option<u64>,
    }

    impl MockVfs {
        fn new(data: &[u8], max_chunk: usize) -> Self {
            Self {
                data: data.to_vec(),
                max_chunk,
                empty_at: None,
            }
        }

        fn with_empty_at(mut self, offset: u64) -> Self {
            self.empty_at = Some(offset);
            self
        }
    }

    impl VirtualFileSystem for MockVfs {
        fn getattr(&self, _path: &str) -> Result<FileAttr, VfsError> {
            Ok(FileAttr {
                size: self.data.len() as u64,
                is_directory: false,
                ..Default::default()
            })
        }

        fn readdir(&self, _path: &str) -> Result<Vec<DirEntry>, VfsError> {
            Ok(Vec::new())
        }

        fn read(&self, _path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
            if self.empty_at.is_some_and(|empty_at| offset >= empty_at) {
                return Ok(Vec::new());
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

    fn zip_bytes(entry_path: &str, bytes: &[u8], method: CompressionMethod) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            let options = SimpleFileOptions::default().compression_method(method);
            zip.start_file(entry_path, options).unwrap();
            zip.write_all(bytes).unwrap();
            zip.finish().unwrap();
        }
        cursor.into_inner()
    }

    fn write_zip_file(path: &std::path::Path, entry_path: &str, bytes: &[u8]) {
        let file = std::fs::File::create(path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        zip.start_file(entry_path, options).unwrap();
        zip.write_all(bytes).unwrap();
        zip.finish().unwrap();
    }

    #[test]
    fn checked_l01_entry_read_offset_rejects_overflow() {
        let err = checked_l01_entry_read_offset(u64::MAX, 1).unwrap_err();

        assert!(err.contains("Invalid L01 entry read offset"));
    }

    #[test]
    fn checked_nested_range_request_size_allows_limit() {
        assert_eq!(
            checked_nested_range_request_size(NESTED_RANGE_MAX_BYTES).unwrap(),
            NESTED_RANGE_MAX_BYTES
        );
    }

    #[test]
    fn checked_nested_range_request_size_rejects_oversized_request() {
        let err = checked_nested_range_request_size(NESTED_RANGE_MAX_BYTES + 1).unwrap_err();

        assert!(err.contains("Nested entry chunk request is too large"));
    }

    #[test]
    fn bounded_nested_read_size_clamps_to_remaining_entry_bytes() {
        assert_eq!(
            bounded_nested_read_size(100, 90, NESTED_RANGE_MAX_BYTES).unwrap(),
            10
        );
    }

    #[test]
    fn bounded_nested_read_size_returns_zero_at_eof() {
        assert_eq!(bounded_nested_read_size(100, 100, 64).unwrap(), 0);
    }

    #[test]
    fn bounded_nested_read_size_rejects_offset_past_eof() {
        let err = bounded_nested_read_size(100, 150, 64).unwrap_err();

        assert!(err.contains("offset 150 > size 100"), "unexpected: {err}");
    }

    #[test]
    fn checked_nested_copy_offset_rejects_overrun() {
        let err = checked_nested_copy_offset(90, 16, 100, "/nested.zip").unwrap_err();

        assert!(err.contains("too many bytes"), "unexpected: {}", err);
    }

    #[test]
    fn ensure_nested_range_read_len_rejects_short_read() {
        let err = ensure_nested_range_read_len(2, 3, "Nested archive VFS", "file.bin").unwrap_err();

        assert!(err.contains("incomplete data"), "unexpected: {err}");
        assert!(err.contains("expected 3 bytes"), "unexpected: {err}");
        assert!(err.contains("received 2"), "unexpected: {err}");
    }

    #[test]
    fn nested_archive_entry_size_and_range_read_archive_entry() {
        NESTED_CONTAINER_CACHE.lock().clear();
        let tmp = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
        let inner_zip = zip_bytes("nested/file.txt", b"abcdef", CompressionMethod::Deflated);
        write_zip_file(tmp.path(), "inner.zip", &inner_zip);

        let outer_path = tmp.path().to_string_lossy();

        assert_eq!(
            nested_container_entry_size(&outer_path, "inner.zip", "nested/file.txt").unwrap(),
            6
        );
        assert_eq!(
            read_nested_container_entry_range(&outer_path, "inner.zip", "nested/file.txt", 2, 3)
                .unwrap(),
            b"cde"
        );
        assert!(read_nested_container_entry_range(
            &outer_path,
            "inner.zip",
            "nested/file.txt",
            6,
            3
        )
        .unwrap()
        .is_empty());
        let err =
            read_nested_container_entry_range(&outer_path, "inner.zip", "nested/file.txt", 7, 3)
                .unwrap_err();
        assert!(err.contains("offset 7 > size 6"), "unexpected: {err}");
    }

    #[test]
    fn copy_vfs_entry_to_file_streams_partial_reads() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let vfs = MockVfs::new(b"abcdef", 2);

        let copied = copy_vfs_entry_to_file(&vfs, "/nested.zip", tmp.path()).unwrap();

        assert_eq!(copied, 6);
        assert_eq!(std::fs::read(tmp.path()).unwrap(), b"abcdef");
    }

    #[test]
    fn copy_vfs_entry_to_file_rejects_empty_short_read() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let vfs = MockVfs::new(b"abcdef", 2).with_empty_at(4);

        let err = copy_vfs_entry_to_file(&vfs, "/nested.zip", tmp.path()).unwrap_err();

        assert!(err.contains("Short read"));
        assert!(
            !tmp.path().exists(),
            "failed nested temp extraction should remove partial file"
        );
    }

    #[test]
    fn copy_vfs_entry_to_file_rejects_oversized_reader_chunk() {
        let tmp = tempfile::NamedTempFile::new().unwrap();

        let err = copy_chunked_to_file(tmp.path(), 2, "/nested.zip", |_offset, _size| {
            Ok(vec![0; 3])
        })
        .unwrap_err();

        assert!(
            err.contains("returned too many bytes"),
            "unexpected: {}",
            err
        );
        assert!(
            !tmp.path().exists(),
            "oversized nested temp extraction should remove partial file"
        );
    }

    #[test]
    fn nested_temp_filename_sanitizes_leaf_name() {
        let nested_filename = std::path::Path::new("folder/inner?.zip")
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("nested");
        let sanitized = sanitize_filename(nested_filename);
        let normalized = if sanitized.is_empty() {
            "nested".to_string()
        } else {
            sanitized
        };

        assert_eq!(normalized, "inner.zip");
    }
}
