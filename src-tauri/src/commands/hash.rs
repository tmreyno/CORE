// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Parallel batch hashing operations for multiple files.

use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Emitter;
use tracing::{debug, info, instrument, warn};

use crate::ad1;
use crate::common::health::QUEUE_METRICS;
use crate::common::{
    hash_byte_source_with_progress, hash_cache, EvidenceByteSource, EvidenceSourceError,
    EvidenceSourceRef, HashAlgorithm, LocalFileByteSource,
};
use crate::containers::{open_container_entry_source_with_options, ContainerEntrySourceOptions};
use crate::ewf;
use crate::raw;
use ffx_aff4::Aff4Reader;

// =============================================================================
// Global Queue State
// =============================================================================

/// Global pause flag for hash queue operations.
/// When set to true, the `batch_hash_smart` worker loop will wait before
/// starting new jobs, effectively pausing the queue.
static QUEUE_PAUSED: AtomicBool = AtomicBool::new(false);

const MAX_HASH_SOURCE_FIELD_CHARS: usize = 4096;
const MAX_HASH_SOURCE_CONTAINER_TYPE_CHARS: usize = 128;
const MAX_HASH_SOURCE_ID_CHARS: usize = 16_384;
const MAX_BATCH_HASH_FILES: usize = 50_000;
const MAX_BATCH_HASH_OVERRIDE_ENTRIES: usize = 16;
const MAX_BATCH_HASH_CONCURRENCY_OVERRIDE: usize = 128;
const MAX_EFFECTIVE_BATCH_HASH_CONCURRENCY: usize = 64;

/// Check if the queue is currently paused
pub fn is_queue_paused() -> bool {
    QUEUE_PAUSED.load(Ordering::Relaxed)
}

// Batch hashing result for a single file
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchHashResult {
    pub path: String,
    pub algorithm: String,
    pub hash: Option<String>,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
    pub throughput_mbs: Option<f64>,
    /// Storage classification of the file's drive (e.g., "Internal SSD", "Removable")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drive_kind: Option<String>,
}

// Progress update for batch hashing - includes hash result when completed
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchProgress {
    pub path: String,
    pub status: String, // "started", "progress", "completed", "error"
    pub percent: f64,
    pub files_completed: usize,
    pub files_total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    // Decompression progress (for E01/compressed containers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunks_processed: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunks_total: Option<usize>,
}

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchFileInput {
    pub path: String,
    pub container_type: String,
}

/// Hash input that can describe either a normal filesystem file or a file-like
/// entry inside a supported container.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HashSourceInput {
    /// Plain filesystem path. Used for local files and as a fallback source id.
    pub path: Option<String>,
    /// Container path for entries inside AD1, L01, archives, E01/RAW VFS views.
    pub container_path: Option<String>,
    /// Entry path inside the container, or a local file path when
    /// `container_type` is `"disk"`.
    pub entry_path: Option<String>,
    /// Path to a nested container inside `container_path`. When present, the
    /// `entry_path` is resolved inside this nested container.
    pub nested_archive_path: Option<String>,
    /// Container type identifier such as `"ad1"`, `"l01"`, `"zip"`, `"e01"`,
    /// `"raw"`, or `"disk"`.
    pub container_type: Option<String>,
    /// Optional known byte size. Avoids metadata reads for some container types.
    pub size: Option<u64>,
    /// AD1 zlib metadata/data address for direct entry reads when available.
    pub data_addr: Option<u64>,
    /// AD1 item header address. Preserved for source metadata and future engines.
    pub item_addr: Option<u64>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HashSourceResult {
    pub source_ref: EvidenceSourceRef,
    pub source_id: String,
    pub path: Option<String>,
    pub container_path: Option<String>,
    pub entry_path: Option<String>,
    pub container_type: Option<String>,
    pub algorithm: String,
    pub hash: String,
    pub bytes_hashed: u64,
    pub duration_ms: u64,
    pub throughput_mbs: Option<f64>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HashSourceProgress {
    pub source_id: String,
    pub current: u64,
    pub total: u64,
    pub percent: f64,
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if a container type string represents an EWF-based format (E01, Ex01, L01, Lx01)
fn is_ewf_type(container_type: &str) -> bool {
    let container_type = container_type.trim().to_lowercase();
    container_type.contains("e01")
        || container_type.contains("encase")
        || container_type.contains("ewf")
        || container_type.contains("ex01")
        || container_type.contains("l01")
        || container_type.contains("lx01")
}

/// Check if a container type string represents an AD1 format
fn is_ad1_type(container_type: &str) -> bool {
    let container_type = container_type.trim().to_lowercase();
    container_type.contains("ad1")
}

/// Check if a container type string represents an AFF4 format
fn is_aff4_type(container_type: &str) -> bool {
    let container_type = container_type.trim().to_lowercase();
    container_type.contains("aff4") || container_type.contains("aff")
}

#[derive(Clone, Debug)]
struct NestedArchiveEntryByteSource {
    container_path: String,
    nested_archive_path: String,
    entry_path: String,
    container_type: Option<String>,
    known_size: Option<u64>,
}

impl NestedArchiveEntryByteSource {
    fn new(
        container_path: String,
        nested_archive_path: String,
        entry_path: String,
        container_type: Option<String>,
        known_size: Option<u64>,
    ) -> Self {
        Self {
            container_path,
            nested_archive_path,
            entry_path,
            container_type,
            known_size,
        }
    }

    fn nested_entry_size(&self) -> Result<u64, EvidenceSourceError> {
        crate::commands::archive::nested::nested_container_entry_size(
            &self.container_path,
            &self.nested_archive_path,
            &self.entry_path,
        )
        .map_err(|message| EvidenceSourceError::Container {
            source_id: self.source_ref().display_id(),
            message,
        })
    }

    fn read_nested_range(&self, offset: u64, size: usize) -> Result<Vec<u8>, EvidenceSourceError> {
        crate::commands::archive::nested::read_nested_container_entry_range(
            &self.container_path,
            &self.nested_archive_path,
            &self.entry_path,
            offset,
            size,
        )
        .map_err(|message| EvidenceSourceError::Container {
            source_id: self.source_ref().display_id(),
            message,
        })
    }
}

impl EvidenceByteSource for NestedArchiveEntryByteSource {
    fn source_ref(&self) -> EvidenceSourceRef {
        EvidenceSourceRef::NestedContainerEntry {
            container_path: self.container_path.clone(),
            nested_container_path: self.nested_archive_path.clone(),
            entry_path: self.entry_path.clone(),
            container_type: self.container_type.clone(),
        }
    }

    fn len(&self) -> Result<u64, EvidenceSourceError> {
        if let Some(size) = self.known_size {
            return Ok(size);
        }
        self.nested_entry_size()
    }

    fn read_range(&self, offset: u64, size: usize) -> Result<Vec<u8>, EvidenceSourceError> {
        let total_size = self.len()?;
        let read_size =
            crate::common::bounded_read_size(&self.source_ref(), total_size, offset, size)?;
        if read_size == 0 {
            return Ok(Vec::new());
        }
        self.read_nested_range(offset, read_size)
    }
}

fn split_nested_entry_path(
    nested_archive_path: Option<&str>,
    entry_path: &str,
) -> Option<(String, String)> {
    if let Some(nested_archive_path) = nested_archive_path {
        if nested_archive_path.is_empty() || entry_path.is_empty() {
            return None;
        }
        return Some((nested_archive_path.to_string(), entry_path.to_string()));
    }

    let (nested, entry) = entry_path.split_once("::")?;
    if nested.is_empty() || entry.is_empty() {
        return None;
    }
    Some((nested.to_string(), entry.to_string()))
}

fn validate_hash_source_request(
    source: &HashSourceInput,
    algorithm: &str,
) -> Result<String, String> {
    validate_hash_source_field(source.path.as_deref(), "path", MAX_HASH_SOURCE_FIELD_CHARS)?;
    validate_hash_source_field(
        source.container_path.as_deref(),
        "containerPath",
        MAX_HASH_SOURCE_FIELD_CHARS,
    )?;
    validate_hash_source_field(
        source.entry_path.as_deref(),
        "entryPath",
        MAX_HASH_SOURCE_FIELD_CHARS,
    )?;
    validate_hash_source_field(
        source.nested_archive_path.as_deref(),
        "nestedArchivePath",
        MAX_HASH_SOURCE_FIELD_CHARS,
    )?;
    validate_hash_source_field(
        source.container_type.as_deref(),
        "containerType",
        MAX_HASH_SOURCE_CONTAINER_TYPE_CHARS,
    )?;

    let container_type = source
        .container_type
        .as_deref()
        .unwrap_or("disk")
        .trim()
        .to_lowercase();

    if container_type == "disk" {
        if source
            .entry_path
            .as_deref()
            .or(source.path.as_deref())
            .is_none()
        {
            return Err("Hash source requires a path or entryPath".to_string());
        }
    } else {
        if source
            .container_path
            .as_deref()
            .or(source.path.as_deref())
            .is_none()
        {
            return Err("Container hash source requires containerPath".to_string());
        }
        if source.entry_path.is_none() {
            return Err("Container hash source requires entryPath".to_string());
        }
    }

    if source.nested_archive_path.is_some() {
        let entry_path = source
            .entry_path
            .as_deref()
            .ok_or_else(|| "Nested hash source requires entryPath".to_string())?;
        if split_nested_entry_path(source.nested_archive_path.as_deref(), entry_path).is_none() {
            return Err(
                "Nested hash source requires non-empty nestedArchivePath and entryPath".to_string(),
            );
        }
    } else if let Some(entry_path) = source.entry_path.as_deref() {
        if entry_path.contains("::") && split_nested_entry_path(None, entry_path).is_none() {
            return Err(
                "Nested hash source compact entryPath must be '<nested>::<entry>'".to_string(),
            );
        }
    }

    let algorithm = algorithm
        .parse::<HashAlgorithm>()
        .map_err(|e| e.to_string())?;
    Ok(algorithm.name().to_string())
}

fn validate_hash_source_field(
    value: Option<&str>,
    field_name: &str,
    max_chars: usize,
) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.trim().is_empty() {
        return Err(format!("Hash source {field_name} cannot be empty"));
    }
    if value.chars().count() > max_chars {
        return Err(format!(
            "Hash source {field_name} exceeds limit of {max_chars} characters"
        ));
    }
    Ok(())
}

fn bounded_hash_source_id(source_id: String) -> String {
    truncate_hash_source_text(source_id, MAX_HASH_SOURCE_ID_CHARS)
}

fn truncate_hash_source_text(value: String, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value;
    }
    value.chars().take(max_chars).collect()
}

pub(crate) fn open_hash_source(
    input: &HashSourceInput,
) -> Result<Box<dyn EvidenceByteSource>, String> {
    let container_type = input
        .container_type
        .as_deref()
        .unwrap_or("disk")
        .trim()
        .to_lowercase();

    if container_type == "disk" {
        let path = input
            .entry_path
            .as_deref()
            .or(input.path.as_deref())
            .ok_or_else(|| "Hash source requires a path or entryPath".to_string())?;
        return Ok(Box::new(LocalFileByteSource::new(path)));
    }

    let container_path = input
        .container_path
        .as_deref()
        .or(input.path.as_deref())
        .ok_or_else(|| "Container hash source requires containerPath".to_string())?;
    let entry_path = input
        .entry_path
        .as_deref()
        .ok_or_else(|| "Container hash source requires entryPath".to_string())?;

    if let Some((nested_archive_path, nested_entry_path)) =
        split_nested_entry_path(input.nested_archive_path.as_deref(), entry_path)
    {
        return Ok(Box::new(NestedArchiveEntryByteSource::new(
            container_path.to_string(),
            nested_archive_path,
            nested_entry_path,
            input.container_type.clone(),
            input.size,
        )));
    }

    open_container_entry_source_with_options(
        container_path,
        entry_path,
        &container_type,
        ContainerEntrySourceOptions {
            known_size: input.size,
            data_addr: input.data_addr,
        },
    )
    .map_err(|e| e.to_string())
}

fn hash_source_id(input: &HashSourceInput, source_ref: Option<&EvidenceSourceRef>) -> String {
    if let Some(source_ref) = source_ref {
        match source_ref {
            EvidenceSourceRef::LocalFile { path } => path.clone(),
            EvidenceSourceRef::ContainerEntry {
                container_path,
                entry_path,
                container_type,
            } => format!(
                "{container_type}:{container_path}:{entry_path}{}",
                hash_source_address_suffix(input, Some(container_type))
            ),
            EvidenceSourceRef::NestedContainerEntry {
                container_path,
                nested_container_path,
                entry_path,
                container_type,
            } => format!(
                "nested:{}:{container_path}:{nested_container_path}::{entry_path}",
                container_type.as_deref().unwrap_or("container")
            ),
            EvidenceSourceRef::VfsEntry {
                container_path,
                entry_path,
                container_type,
            } => format!(
                "vfs:{}:{container_path}:{entry_path}",
                container_type.as_deref().unwrap_or("container")
            ),
        }
    } else if let Some(path) = &input.path {
        path.clone()
    } else if let (Some(container), Some(entry)) = (&input.container_path, &input.entry_path) {
        bounded_hash_source_id(format!(
            "{}:{}{}",
            container,
            entry,
            hash_source_address_suffix(input, input.container_type.as_deref())
        ))
    } else {
        "unknown-source".to_string()
    }
}

fn hash_source_address_suffix(input: &HashSourceInput, container_type: Option<&str>) -> String {
    if !container_type.is_some_and(is_ad1_type) {
        return String::new();
    }

    match (input.item_addr, input.data_addr) {
        (Some(item_addr), Some(data_addr)) => {
            format!("#item=0x{item_addr:x};data=0x{data_addr:x}")
        }
        (Some(item_addr), None) => format!("#item=0x{item_addr:x}"),
        (None, Some(data_addr)) => format!("#data=0x{data_addr:x}"),
        (None, None) => String::new(),
    }
}

/// Hash a local file or supported container entry through the common
/// byte-source layer.
#[tauri::command]
pub async fn hash_source(
    source: HashSourceInput,
    algorithm: String,
    app: tauri::AppHandle,
) -> Result<HashSourceResult, String> {
    let canonical_algorithm = validate_hash_source_request(&source, &algorithm)?;
    let source_for_worker = source.clone();
    let algorithm_for_worker = canonical_algorithm.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let start_time = std::time::Instant::now();
        let byte_source = open_hash_source(&source_for_worker)?;
        let source_ref = byte_source.source_ref();
        let source_id =
            bounded_hash_source_id(hash_source_id(&source_for_worker, Some(&source_ref)));
        let total_size = byte_source.len().map_err(|e| e.to_string())?;

        let hash = hash_byte_source_with_progress(
            byte_source.as_ref(),
            &algorithm_for_worker,
            |current, total| {
                let percent = if total > 0 {
                    (current as f64 / total as f64 * 100.0).min(100.0)
                } else {
                    100.0
                };
                let _ = app.emit(
                    "hash-source-progress",
                    HashSourceProgress {
                        source_id: source_id.clone(),
                        current,
                        total,
                        percent,
                    },
                );
            },
        )
        .map_err(|e| e.to_string())?;

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let throughput_mbs = if duration_ms > 0 && total_size > 0 {
            Some((total_size as f64 / (1024.0 * 1024.0)) / (duration_ms as f64 / 1000.0))
        } else {
            None
        };

        Ok(HashSourceResult {
            source_ref,
            source_id,
            path: source_for_worker.path,
            container_path: source_for_worker.container_path,
            entry_path: source_for_worker.entry_path,
            container_type: source_for_worker.container_type,
            algorithm: canonical_algorithm,
            hash,
            bytes_hashed: total_size,
            duration_ms,
            throughput_mbs,
        })
    })
    .await
    .map_err(|e| format!("Internal hash source error: {e}"))?
}

/// Verify an AFF4 container by reading the decoded image stream and computing
/// a hash with the user-selected algorithm.  This decompresses bevy data
/// through the AFF4 reader (like EWF verify decompresses chunks), so the
/// resulting hash represents the **original source data**, not the ZIP
/// container bytes.
fn aff4_verify_with_progress(
    path: &str,
    algorithm: &str,
    progress_cb: &mut dyn FnMut(u64, u64),
) -> Result<String, String> {
    let mut reader = Aff4Reader::open(path).map_err(|e| format!("Failed to open AFF4: {e}"))?;
    let info = reader.info();
    let stream = info
        .streams
        .first()
        .ok_or_else(|| "AFF4 container has no image streams".to_string())?
        .clone();

    let total = stream.size;
    let chunk_size: usize = 1024 * 1024; // 1 MB read chunks
    let mut buf = vec![0u8; chunk_size];
    let mut bytes_read: u64 = 0;

    let mut hasher: crate::common::StreamingHasher = algorithm
        .parse()
        .map_err(|e| format!("Unsupported hash algorithm for AFF4: {e}"))?;
    let use_parallel_updates = algorithm.eq_ignore_ascii_case("blake3");

    progress_cb(0, total);

    while bytes_read < total {
        let Some(to_read) = aff4_read_chunk_size(total, bytes_read, chunk_size)? else {
            break;
        };
        let read_buf = &mut buf[..to_read];

        let n = reader
            .read_at(&stream.urn, bytes_read, read_buf)
            .map_err(|e| format!("AFF4 read error at offset {bytes_read}: {e}"))?;
        if n == 0 {
            return Err(format!(
                "AFF4 short read at offset {bytes_read}: expected {total} bytes from stream {}, hashed {bytes_read} bytes",
                stream.urn
            ));
        }

        if use_parallel_updates {
            hasher.update_parallel(&read_buf[..n]);
        } else {
            hasher.update(&read_buf[..n]);
        }

        bytes_read = checked_aff4_bytes_read_advance(bytes_read, n, &stream.urn)?;
        progress_cb(bytes_read, total);
    }

    Ok(hasher.finalize())
}

fn aff4_read_chunk_size(
    total: u64,
    bytes_read: u64,
    chunk_size: usize,
) -> Result<Option<usize>, String> {
    if chunk_size == 0 {
        return Err("AFF4 read chunk size cannot be zero".to_string());
    }

    let remaining = total.checked_sub(bytes_read).ok_or_else(|| {
        format!(
            "AFF4 byte counter exceeded stream size: read {} bytes > expected {} bytes",
            bytes_read, total
        )
    })?;
    if remaining == 0 {
        return Ok(None);
    }

    let chunk_size = u64::try_from(chunk_size)
        .map_err(|_| "AFF4 read chunk size does not fit in u64".to_string())?;
    usize::try_from(remaining.min(chunk_size))
        .map(Some)
        .map_err(|_| "AFF4 read chunk length does not fit in usize".to_string())
}

fn checked_aff4_bytes_read_advance(
    bytes_read: u64,
    chunk_read: usize,
    stream_urn: &str,
) -> Result<u64, String> {
    let chunk_read = u64::try_from(chunk_read)
        .map_err(|_| "AFF4 read byte count does not fit in u64".to_string())?;
    bytes_read.checked_add(chunk_read).ok_or_else(|| {
        format!(
            "AFF4 byte counter overflowed while hashing stream {stream_urn}: read {bytes_read} bytes, next chunk {chunk_read} bytes"
        )
    })
}

fn progress_counter_value(value: u64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

fn batch_hash_cache_scope(container_type: &str) -> String {
    let normalized = container_type.trim().to_lowercase();
    if is_ewf_type(&normalized) {
        "decoded-ewf".to_string()
    } else if is_ad1_type(&normalized) {
        "ad1-segments".to_string()
    } else if is_aff4_type(&normalized) {
        "decoded-aff4".to_string()
    } else if normalized.is_empty() {
        "raw-file:disk".to_string()
    } else {
        format!("raw-file:{normalized}")
    }
}

fn batch_hash_cache_scope_for_path(container_type: &str, path: &str) -> String {
    let base_scope = batch_hash_cache_scope(container_type);
    if is_ad1_type(container_type) {
        if let Ok(segment_paths) = ad1::get_segment_paths(path) {
            return segmented_batch_hash_cache_scope(&base_scope, &segment_paths);
        }
    } else if is_ewf_type(container_type) {
        let lower_path = path.to_ascii_lowercase();
        let segments = if lower_path.ends_with(".l01") || lower_path.ends_with(".lx01") {
            crate::common::segments::discover_l01_segments(path)
        } else {
            crate::common::segments::discover_e01_segments(path)
        };
        if let Ok(segment_paths) = segments {
            return segmented_batch_hash_cache_scope(&base_scope, &segment_paths);
        }
    }

    base_scope
}

fn segmented_batch_hash_cache_scope(
    base_scope: &str,
    segment_paths: &[std::path::PathBuf],
) -> String {
    if segment_paths.len() <= 1 {
        return base_scope.to_string();
    }

    let mut fingerprint = DefaultHasher::new();
    segment_paths.len().hash(&mut fingerprint);
    for segment_path in segment_paths {
        segment_path.to_string_lossy().hash(&mut fingerprint);
        match std::fs::metadata(segment_path) {
            Ok(metadata) => {
                metadata.len().hash(&mut fingerprint);
                if let Ok(modified) = metadata.modified() {
                    if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                        duration.as_secs().hash(&mut fingerprint);
                        duration.subsec_nanos().hash(&mut fingerprint);
                    }
                }
            }
            Err(_) => {
                "missing".hash(&mut fingerprint);
            }
        }
    }

    format!("{base_scope}:segments-{:#016x}", fingerprint.finish())
}

fn hash_algorithm_worker_name(algorithm: HashAlgorithm) -> &'static str {
    match algorithm {
        HashAlgorithm::Md5 => "md5",
        HashAlgorithm::Sha1 => "sha1",
        HashAlgorithm::Sha256 => "sha256",
        HashAlgorithm::Sha512 => "sha512",
        HashAlgorithm::Blake3 => "blake3",
        HashAlgorithm::Blake2 => "blake2",
        HashAlgorithm::Xxh3 => "xxh3",
        HashAlgorithm::Xxh64 => "xxh64",
        HashAlgorithm::Crc32 => "crc32",
    }
}

fn validate_batch_hash_request(
    files: &[BatchFileInput],
    algorithm: &str,
    concurrency_overrides: Option<HashMap<String, usize>>,
) -> Result<(String, String, HashMap<String, usize>), String> {
    if files.len() > MAX_BATCH_HASH_FILES {
        return Err(format!(
            "Batch hash requested {} files, exceeding limit {}",
            files.len(),
            MAX_BATCH_HASH_FILES
        ));
    }
    for file in files {
        validate_batch_file_input(file)?;
    }

    let algorithm = algorithm
        .parse::<HashAlgorithm>()
        .map_err(|e| e.to_string())?;
    let display_algorithm = algorithm.name().to_string();
    let worker_algorithm = hash_algorithm_worker_name(algorithm).to_string();
    let overrides = validate_batch_hash_overrides(concurrency_overrides.unwrap_or_default())?;

    Ok((display_algorithm, worker_algorithm, overrides))
}

fn validate_batch_file_input(file: &BatchFileInput) -> Result<(), String> {
    validate_hash_source_field(Some(&file.path), "batch path", MAX_HASH_SOURCE_FIELD_CHARS)?;
    validate_hash_source_field(
        Some(&file.container_type),
        "batch containerType",
        MAX_HASH_SOURCE_CONTAINER_TYPE_CHARS,
    )
}

fn validate_batch_hash_overrides(
    overrides: HashMap<String, usize>,
) -> Result<HashMap<String, usize>, String> {
    if overrides.len() > MAX_BATCH_HASH_OVERRIDE_ENTRIES {
        return Err(format!(
            "Batch hash concurrency overrides contain {} entries, exceeding limit {}",
            overrides.len(),
            MAX_BATCH_HASH_OVERRIDE_ENTRIES
        ));
    }

    let mut normalized = HashMap::with_capacity(overrides.len());
    for (key, value) in overrides {
        let normalized_key = key.trim().to_lowercase();
        if normalized_key.is_empty() {
            return Err("Batch hash concurrency override keys cannot be empty".to_string());
        }
        if normalized_key.chars().count() > MAX_HASH_SOURCE_CONTAINER_TYPE_CHARS {
            return Err(format!(
                "Batch hash concurrency override key exceeds limit of {} characters",
                MAX_HASH_SOURCE_CONTAINER_TYPE_CHARS
            ));
        }
        if !is_valid_storage_override_key(&normalized_key) {
            return Err(format!(
                "Unknown batch hash concurrency override key: {normalized_key}"
            ));
        }
        if value > MAX_BATCH_HASH_CONCURRENCY_OVERRIDE {
            return Err(format!(
                "Batch hash concurrency override for {normalized_key} is {value}, exceeding limit {}",
                MAX_BATCH_HASH_CONCURRENCY_OVERRIDE
            ));
        }
        normalized.insert(normalized_key, value);
    }

    Ok(normalized)
}

/// Verify an EWF container using the libewf C library (via libewf-ffi).
///
/// This uses the battle-tested libewf C library for all I/O — multi-segment
/// handling, decompression, and media boundary truncation are handled natively
/// by libewf.  The resulting hash represents the original source data.
///
/// Supports MD5, SHA-1, and SHA-256.  For other algorithms the caller should
/// fall back to the pure-Rust `ewf::verify_with_progress` path.
fn libewf_verify_with_progress(
    path: &str,
    algorithm: &str,
    progress_cb: &mut dyn FnMut(u64, u64),
) -> Result<String, String> {
    use libewf_ffi::EwfReader;

    let reader = EwfReader::open(path).map_err(|e| format!("libewf open failed: {e}"))?;

    reader
        .verify_media_hash(algorithm, |current, total| {
            progress_cb(current, total);
        })
        .map_err(|e| format!("libewf verify failed: {e}"))
}

/// Spawn a progress reporter thread that periodically emits batch-progress events.
///
/// Emits an immediate 0% event, then polls with adaptive frequency based on
/// batch size. Uses a 3-second heartbeat (1-second during startup) to prove
/// the operation is alive. For large batches (>50 files), the poll interval
/// and minimum percent change are increased to reduce IPC flood on Windows.
fn spawn_progress_reporter(
    app: tauri::AppHandle,
    path: String,
    idx: usize,
    num_files: usize,
    progress_current: Arc<std::sync::atomic::AtomicUsize>,
    progress_total: Arc<std::sync::atomic::AtomicUsize>,
    done_flag: Arc<std::sync::atomic::AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut last_percent_key = 0u32;
        let mut last_emit = std::time::Instant::now();
        let heartbeat_interval = std::time::Duration::from_secs(3);
        let startup_heartbeat = std::time::Duration::from_secs(1);

        // Adaptive progress: larger batches use longer intervals and coarser steps
        // to avoid flooding the Tauri IPC bridge on Windows
        let poll_interval_ms: u64 = if num_files > 50 {
            2000
        } else if num_files > 10 {
            1000
        } else {
            250
        };
        let min_percent_change: u32 = if num_files > 50 {
            10
        } else if num_files > 10 {
            6
        } else {
            1
        };

        // Emit immediate 0% so the UI shows activity right away
        let _ = app.emit(
            "batch-progress",
            BatchProgress {
                path: path.clone(),
                status: "progress".to_string(),
                percent: 0.0,
                files_completed: idx,
                files_total: num_files,
                hash: None,
                algorithm: None,
                error: None,
                chunks_processed: Some(0),
                chunks_total: None,
            },
        );

        loop {
            std::thread::sleep(std::time::Duration::from_millis(poll_interval_ms));
            if done_flag.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            let current = progress_current.load(std::sync::atomic::Ordering::Relaxed);
            let total = progress_total.load(std::sync::atomic::Ordering::Relaxed);

            if total > 1 {
                let percent_f64 = (current as f64 / total as f64) * 100.0;
                let percent_key = (percent_f64 * 2.0) as u32; // 0.5% steps

                let should_emit = percent_key >= last_percent_key + min_percent_change
                    || last_emit.elapsed() >= heartbeat_interval;

                if should_emit {
                    let _ = app.emit(
                        "batch-progress",
                        BatchProgress {
                            path: path.clone(),
                            status: "progress".to_string(),
                            percent: percent_f64.min(100.0),
                            files_completed: idx,
                            files_total: num_files,
                            hash: None,
                            algorithm: None,
                            error: None,
                            chunks_processed: Some(current),
                            chunks_total: Some(total),
                        },
                    );
                    last_percent_key = percent_key;
                    last_emit = std::time::Instant::now();
                }
            } else {
                // Total not yet set — still emit heartbeat so frontend knows we're alive
                // Use shorter interval during startup (file open phase)
                if last_emit.elapsed() >= startup_heartbeat {
                    let _ = app.emit(
                        "batch-progress",
                        BatchProgress {
                            path: path.clone(),
                            status: "progress".to_string(),
                            percent: 0.0,
                            files_completed: idx,
                            files_total: num_files,
                            hash: None,
                            algorithm: None,
                            error: None,
                            chunks_processed: None,
                            chunks_total: None,
                        },
                    );
                    last_emit = std::time::Instant::now();
                }
            }
        }
    })
}

/// Storage classification for I/O scheduling in batch hash operations.
///
/// Hash verification is **I/O-bound** (reading from disk), not CPU-bound.
/// Different storage media have vastly different concurrent I/O characteristics.
/// The batch hasher detects each file's storage type and schedules accordingly:
///
/// | Class         | Default Concurrency | Rationale |
/// |---------------|---------------------|-------------------------------------------|
/// | NVMe/PCIe SSD | 6                   | High queue depth, no seek penalty |
/// | Internal SSD  | 3                   | SATA SSD, good parallelism |
/// | RAID Array    | 4                   | Multiple spindles, moderate parallelism |
/// | Internal HDD  | 1                   | Seek-limited; even 2 concurrent causes head thrashing |
/// | Removable     | 1                   | USB bus is typically the bottleneck |
/// | Network Share | 2                   | Latency-bound; moderate parallelism hides round-trips |
/// | Unknown       | 2                   | Conservative default when media type is undetectable |
#[derive(Debug, Clone, Copy, PartialEq)]
enum StorageClass {
    NvmePcie,
    InternalSsd,
    RaidArray,
    InternalHdd,
    Removable,
    NetworkShare,
    Unknown,
}

impl StorageClass {
    /// Default concurrent hash I/O operations for this storage class.
    ///
    /// `Unknown` scales with available CPU cores (min 2, max 8) because when
    /// we cannot detect the storage class, a hardcoded low value like 2 is
    /// catastrophically slow on high-core-count machines (e.g., 28-core i9).
    fn default_concurrency(self) -> usize {
        match self {
            Self::NvmePcie => 6,
            Self::InternalSsd => 3,
            Self::RaidArray => 4,
            Self::InternalHdd => 1,
            Self::Removable => 1,
            Self::NetworkShare => 2,
            Self::Unknown => {
                // Scale with available cores: cores / 4, clamped to [2, 8].
                // On a 28-core machine this gives 7; on a 4-core laptop, 2.
                let cores = std::thread::available_parallelism()
                    .map(|p| p.get())
                    .unwrap_or(4);
                (cores / 4).clamp(2, 8)
            }
        }
    }

    /// Human-readable label for logging and progress events.
    fn label(self) -> &'static str {
        match self {
            Self::NvmePcie => "NVMe/PCIe SSD",
            Self::InternalSsd => "Internal SSD",
            Self::RaidArray => "RAID Array",
            Self::InternalHdd => "Internal HDD",
            Self::Removable => "Removable",
            Self::NetworkShare => "Network Share",
            Self::Unknown => "Unknown",
        }
    }

    /// Key string for matching against user override map.
    fn key(self) -> &'static str {
        match self {
            Self::NvmePcie => "nvme",
            Self::InternalSsd => "ssd",
            Self::RaidArray => "raid",
            Self::InternalHdd => "hdd",
            Self::Removable => "removable",
            Self::NetworkShare => "network",
            Self::Unknown => "unknown",
        }
    }
}

fn is_valid_storage_override_key(key: &str) -> bool {
    matches!(
        key,
        "nvme" | "ssd" | "raid" | "hdd" | "removable" | "network" | "unknown"
    )
}

/// Resolve effective concurrency for a storage class, considering user overrides.
/// A user-set value of 0 means "use auto/default".
fn resolve_batch_hash_concurrency(
    class: StorageClass,
    overrides: &HashMap<String, usize>,
) -> usize {
    let concurrency = if let Some(&user_val) = overrides.get(class.key()) {
        if user_val > 0 {
            user_val
        } else {
            class.default_concurrency()
        }
    } else {
        class.default_concurrency()
    };

    concurrency.clamp(1, MAX_EFFECTIVE_BATCH_HASH_CONCURRENCY)
}

/// Classify the storage device backing a file path.
///
/// Detection priority:
/// 1. UNC/network paths (\\server\share or //server/share) → NetworkShare
/// 2. Longest-prefix mount point match against `sysinfo::Disks`:
///    - Removable media → Removable
///    - NVMe device name heuristic → NvmePcie
///    - RAID device heuristic (Linux /dev/md*) → RaidArray
///    - SSD → InternalSsd
///    - HDD → InternalHdd
/// 3. Windows drive letter fallback (e.g., `I:\` from `I:\path\to\file`) when
///    sysinfo disk enumeration fails — ensures per-drive semaphore grouping
///    even without storage class detection
/// 4. Unknown (no match) → Unknown
///
/// On Windows, all path comparisons are case-insensitive because Windows
/// drive letters and paths are case-insensitive (e.g., `I:\` == `i:\`).
#[allow(unused_assignments)]
fn classify_storage(path: &str, disks: &sysinfo::Disks) -> (StorageClass, String) {
    // ── Network path detection ──────────────────────────────────────
    if path.starts_with("\\\\")
        || path.starts_with("//")
        || path.starts_with("/Volumes/") && is_network_mount(path, disks)
    {
        let mount = extract_network_mount(path);
        return (StorageClass::NetworkShare, mount);
    }

    // ── Disk-based classification ───────────────────────────────────
    let mut best_mount = String::new();
    let mut best_kind = None;
    let mut best_removable = false;
    let mut best_name = String::new();
    #[allow(unused_variables, unused_assignments)]
    let mut best_fs = String::new();

    // On Windows, path comparison must be case-insensitive because drive
    // letters and directory names are case-insensitive. sysinfo may return
    // mount points like "C:\\" while the frontend passes paths like "c:\\".
    #[cfg(target_os = "windows")]
    let path_lower = path.to_lowercase();

    for d in disks.iter() {
        let mount = d.mount_point().to_string_lossy();

        #[cfg(target_os = "windows")]
        let matches = {
            let mount_lower = mount.to_lowercase();
            path_lower.starts_with(&mount_lower) && mount.len() > best_mount.len()
        };

        #[cfg(not(target_os = "windows"))]
        let matches = path.starts_with(mount.as_ref()) && mount.len() > best_mount.len();

        if matches {
            best_mount = mount.into_owned();
            best_kind = Some(d.kind());
            best_removable = d.is_removable();
            best_name = d.name().to_string_lossy().into_owned();
            best_fs = d.file_system().to_string_lossy().into_owned();
        }
    }

    let class = match (best_kind, best_removable) {
        (_, true) => StorageClass::Removable,
        (Some(sysinfo::DiskKind::SSD), false) => {
            // Heuristic: NVMe devices often have "nvme" in their device name
            if is_nvme_device(&best_name, &best_mount) {
                StorageClass::NvmePcie
            } else {
                StorageClass::InternalSsd
            }
        }
        (Some(sysinfo::DiskKind::HDD), false) => {
            // Heuristic: Linux software RAID devices are /dev/md*
            if is_raid_device(&best_name, &best_mount) {
                StorageClass::RaidArray
            } else {
                StorageClass::InternalHdd
            }
        }
        _ => {
            // Unknown disk kind — check for network filesystem types
            #[cfg(target_os = "linux")]
            {
                let fs_lower = best_fs.to_lowercase();
                if fs_lower == "cifs"
                    || fs_lower == "smb"
                    || fs_lower == "nfs"
                    || fs_lower == "nfs4"
                {
                    return (StorageClass::NetworkShare, best_mount);
                }
            }
            StorageClass::Unknown
        }
    };

    // ── Fallback: extract mount point even when class is Unknown ────
    // On Windows, if sysinfo returned no matching disk (disk_count=0 on
    // some Windows 11 builds), extract the drive letter so files on the
    // same drive still share a per-drive semaphore instead of all piling
    // into a single "unknown" semaphore.
    let mount = if best_mount.is_empty() {
        extract_drive_letter_mount(path).unwrap_or_else(|| "unknown".to_string())
    } else {
        best_mount
    };

    (class, mount)
}

/// Extract a Windows drive letter mount point from a path (e.g., `I:\...` → `I:\`).
/// Also handles paths like `D:path` (relative to drive). Returns None for
/// non-Windows-style paths (Unix paths, UNC paths).
fn extract_drive_letter_mount(path: &str) -> Option<String> {
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        // Normalize to uppercase drive letter + colon + backslash
        let letter = (bytes[0] as char).to_ascii_uppercase();
        Some(format!("{}:\\", letter))
    } else {
        None
    }
}

/// Check if a path is on a network mount (macOS /Volumes/ heuristic).
fn is_network_mount(path: &str, disks: &sysinfo::Disks) -> bool {
    for d in disks.iter() {
        let mount = d.mount_point().to_string_lossy();
        if path.starts_with(mount.as_ref()) {
            let fs = d.file_system().to_string_lossy().to_lowercase();
            if fs == "smbfs" || fs == "nfs" || fs == "afpfs" || fs == "cifs" {
                return true;
            }
        }
    }
    false
}

/// Extract a reasonable mount identifier from a network path.
fn extract_network_mount(path: &str) -> String {
    // UNC: \\server\share\... → \\server\share
    if path.starts_with("\\\\") || path.starts_with("//") {
        let sep = if path.starts_with("\\\\") { '\\' } else { '/' };
        let stripped = &path[2..];
        if let Some(slash_pos) = stripped.find(sep) {
            let after_server = &stripped[slash_pos + 1..];
            if let Some(next_slash) = after_server.find(sep) {
                return path[..2 + slash_pos + 1 + next_slash].to_string();
            }
        }
        return path.to_string();
    }
    // macOS /Volumes/ShareName → /Volumes/ShareName
    if let Some(rest) = path.strip_prefix("/Volumes/") {
        if let Some(pos) = rest.find('/') {
            return path[..("/Volumes/".len() + pos)].to_string();
        }
    }
    path.to_string()
}

/// Heuristic: detect NVMe devices by name or mount path.
fn is_nvme_device(device_name: &str, mount_path: &str) -> bool {
    let name_lower = device_name.to_lowercase();
    let mount_lower = mount_path.to_lowercase();
    name_lower.contains("nvme") || mount_lower.contains("nvme") || name_lower.contains("pcie")
}

/// Heuristic: detect RAID arrays by device name or mount path.
fn is_raid_device(device_name: &str, mount_path: &str) -> bool {
    let name_lower = device_name.to_lowercase();
    let mount_lower = mount_path.to_lowercase();
    // Linux software RAID: /dev/md0, /dev/md127, etc.
    name_lower.starts_with("md") || name_lower.contains("raid") || mount_lower.contains("/dev/md")
}

/// Summary of drive detection results emitted as `"batch-drive-info"` event.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchDriveInfo {
    drives: Vec<DriveDetection>,
    total_files: usize,
}

/// Per-drive detection result within a batch hash operation.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DriveDetection {
    mount_point: String,
    storage_class: String,
    concurrency: usize,
    file_count: usize,
}

/// Hash multiple files in parallel with storage-aware scheduling.
///
/// Detects the storage type (SSD, HDD, NVMe, RAID, NAS, removable) for each
/// file and creates per-drive semaphores with optimized concurrency limits.
/// Files on different drives hash in parallel independently; files on the same
/// drive are limited to prevent I/O thrashing.
///
/// Optional `concurrency_overrides` map lets the frontend pass user-configured
/// per-storage-class concurrency limits. Keys are storage class keys (e.g.
/// "ssd", "hdd", "nvme", "raid", "network", "removable", "unknown").
/// Value 0 means "use default". Missing keys also use default.
#[tauri::command]
#[instrument(skip(files, app), fields(num_files = files.len(), algorithm = %algorithm))]
pub async fn batch_hash(
    files: Vec<BatchFileInput>,
    algorithm: String,
    concurrency_overrides: Option<HashMap<String, usize>>,
    app: tauri::AppHandle,
) -> Result<Vec<BatchHashResult>, String> {
    let cmd_start = std::time::Instant::now();
    debug!("batch_hash command started");

    let num_files = files.len();
    info!("Starting parallel batch hash");
    if num_files == 0 {
        return Ok(Vec::new());
    }
    let (display_algorithm, worker_algorithm, overrides) =
        validate_batch_hash_request(&files, &algorithm, concurrency_overrides)?;

    // ── Drive detection ────────────────────────────────────────────────
    // Detect the storage type for each file and create per-drive semaphores.
    // Files on different drives can hash in parallel independently; files on
    // the same drive are limited to the drive's optimal concurrency.
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let sysinfo_disk_count = disks.iter().count();
    if sysinfo_disk_count == 0 {
        warn!(
            "sysinfo::Disks returned 0 disks — drive detection will use fallback heuristics. \
             This may indicate a permissions issue or an unsupported Windows configuration."
        );
    } else {
        debug!(sysinfo_disk_count, "sysinfo::Disks enumerated successfully");
    }
    let mut drive_classes: HashMap<String, StorageClass> = HashMap::new();
    let mut file_mounts: Vec<String> = Vec::with_capacity(num_files);
    let mut file_drive_labels: Vec<String> = Vec::with_capacity(num_files);

    for file in &files {
        let (class, mount) = classify_storage(&file.path, &disks);
        file_drive_labels.push(class.label().to_string());
        file_mounts.push(mount.clone());
        drive_classes.entry(mount).or_insert(class);
    }

    // Create per-drive semaphores with storage-appropriate concurrency
    let drive_semaphores: Arc<HashMap<String, Arc<tokio::sync::Semaphore>>> = Arc::new(
        drive_classes
            .iter()
            .map(|(mount, class)| {
                let concurrency = resolve_batch_hash_concurrency(*class, &overrides).min(num_files);
                (
                    mount.clone(),
                    Arc::new(tokio::sync::Semaphore::new(concurrency)),
                )
            })
            .collect(),
    );

    for (mount, class) in &drive_classes {
        let effective = resolve_batch_hash_concurrency(*class, &overrides);
        info!(
            mount = %mount,
            storage = class.label(),
            concurrency = effective,
            files_on_drive = file_mounts.iter().filter(|m| *m == mount).count(),
            "Drive detected for hash scheduling"
        );
    }

    // Emit drive detection summary to frontend
    let _ = app.emit(
        "batch-drive-info",
        BatchDriveInfo {
            drives: drive_classes
                .iter()
                .map(|(mount, class)| {
                    let effective = resolve_batch_hash_concurrency(*class, &overrides);
                    DriveDetection {
                        mount_point: mount.clone(),
                        storage_class: class.label().to_string(),
                        concurrency: effective,
                        file_count: file_mounts.iter().filter(|m| *m == mount).count(),
                    }
                })
                .collect(),
            total_files: num_files,
        },
    );

    debug!(
        elapsed_ms = cmd_start.elapsed().as_millis(),
        drives = drive_classes.len(),
        num_files,
        "Drive detection complete, spawning tasks"
    );

    // Spawn all file processing tasks
    let mut handles = Vec::with_capacity(num_files);

    // For large batches, skip per-file "queued" events to avoid flooding IPC
    // on Windows. Each emit() crosses the WebView bridge and can stall the UI.
    let emit_queued = num_files <= 20;

    for (idx, file) in files.into_iter().enumerate() {
        let path = file.path.clone();
        let container_type = file.container_type.to_lowercase();
        let algo = worker_algorithm.clone();
        let display_algo = display_algorithm.clone();
        let app_clone = app.clone();
        let file_mount = file_mounts[idx].clone();
        let drive_label = file_drive_labels[idx].clone();
        let sems = drive_semaphores.clone();

        // Emit progress: queued (only for small batches to avoid IPC flood)
        if emit_queued {
            let _ = app.emit(
                "batch-progress",
                BatchProgress {
                    path: path.clone(),
                    status: "queued".to_string(),
                    percent: 0.0,
                    files_completed: 0,
                    files_total: num_files,
                    hash: None,
                    algorithm: None,
                    error: None,
                    chunks_processed: None,
                    chunks_total: None,
                },
            );
        }

        let handle = tauri::async_runtime::spawn(async move {
            // Wait while the queue is paused
            while QUEUE_PAUSED.load(Ordering::Relaxed) {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }

            // Acquire per-drive semaphore permit (limits concurrent files on same drive)
            let sem = sems[&file_mount].clone();
            let _permit = match sem.acquire_owned().await {
                Ok(p) => p,
                Err(e) => {
                    let err_msg = format!("Semaphore error: {}", e);
                    let _ = app_clone.emit(
                        "batch-progress",
                        BatchProgress {
                            path: path.clone(),
                            status: "error".to_string(),
                            percent: 0.0,
                            files_completed: idx,
                            files_total: num_files,
                            hash: None,
                            algorithm: None,
                            error: Some(err_msg.clone()),
                            chunks_processed: None,
                            chunks_total: None,
                        },
                    );
                    return BatchHashResult {
                        path,
                        algorithm: display_algo,
                        hash: None,
                        error: Some(err_msg),
                        duration_ms: None,
                        throughput_mbs: None,
                        drive_kind: Some(drive_label.clone()),
                    };
                }
            };

            debug!(idx = idx + 1, total = num_files, path = %path, "File started");

            // Emit progress: started
            let _ = app_clone.emit(
                "batch-progress",
                BatchProgress {
                    path: path.clone(),
                    status: "started".to_string(),
                    percent: 0.0,
                    files_completed: idx,
                    files_total: num_files,
                    hash: None,
                    algorithm: None,
                    error: None,
                    chunks_processed: None,
                    chunks_total: None,
                },
            );

            let path_for_hash = path.clone();
            let algo_for_hash = algo.clone();
            let container_for_hash = container_type.clone();
            let app_for_hash = app_clone.clone();
            let path_for_error = path.clone();
            let algo_for_error = display_algo.clone();

            // Run blocking hash in spawn_blocking
            let hash_result = tauri::async_runtime::spawn_blocking(move || {
                let blocking_start = std::time::Instant::now();
                debug!(path = %path_for_hash, "spawn_blocking started");

                let start_time = std::time::Instant::now();
                let file_size = std::fs::metadata(&path_for_hash).map(|m| m.len()).unwrap_or(0);
                debug!(idx = idx + 1, size_mb = file_size / 1024 / 1024, "Processing file");
                debug!(elapsed_ms = blocking_start.elapsed().as_millis(), "File metadata read");

                // Progress counters
                let progress_current = Arc::new(std::sync::atomic::AtomicUsize::new(0));
                let progress_total = Arc::new(std::sync::atomic::AtomicUsize::new(1)); // Start with 1 to avoid div by zero
                let done_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

                // Spawn progress reporter thread
                let progress_thread = spawn_progress_reporter(
                    app_for_hash.clone(),
                    path_for_hash.clone(),
                    idx,
                    num_files,
                    progress_current.clone(),
                    progress_total.clone(),
                    done_flag.clone(),
                );

                info!(container_type = %container_for_hash, algorithm = %algo_for_hash, path = %path_for_hash, "[HASH-DIAG] About to start hashing");
                let _hash_start = std::time::Instant::now();
                let cache_scope =
                    batch_hash_cache_scope_for_path(&container_for_hash, &path_for_hash);

                // Check cache first - this can skip expensive recomputation
                let cached_hash = hash_cache::get_cached_hash_scoped(
                    &path_for_hash,
                    &algo_for_hash,
                    &cache_scope,
                );

                // Hash based on container type (or use cached result)
                let result: Result<String, String> = if let Some(hash) = cached_hash {
                    debug!(path = %path_for_hash, algorithm = %algo_for_hash, "Cache hit");
                    // Signal 100% progress immediately for cached results
                    progress_total.store(1, std::sync::atomic::Ordering::Relaxed);
                    progress_current.store(1, std::sync::atomic::Ordering::Relaxed);
                    Ok(hash)
                } else {
                    // Shared progress callback — all hash functions use the same pattern
                    let mut progress_cb = |current: u64, total: u64| {
                        progress_total.store(
                            progress_counter_value(total),
                            std::sync::atomic::Ordering::Relaxed,
                        );
                        progress_current.store(
                            progress_counter_value(current),
                            std::sync::atomic::Ordering::Relaxed,
                        );
                    };

                    // Route to the appropriate hash function (3 paths:
                    //   EWF → libewf-ffi (C library) with pure-Rust fallback
                    //   AD1 → ad1::hash_segments_with_progress
                    //   Everything else → raw::verify_with_progress)
                    if is_ewf_type(&container_for_hash) {
                        // Try the battle-tested libewf C library first — it handles
                        // multi-segment, decompression, and media boundaries natively.
                        // Fall back to the pure-Rust parser if libewf is unavailable
                        // (stub build) or if the algorithm is not supported by libewf
                        // (only MD5/SHA-1/SHA-256).
                        if libewf_ffi::is_available() {
                            let libewf_result = libewf_verify_with_progress(
                                &path_for_hash,
                                &algo_for_hash,
                                &mut progress_cb,
                            );
                            match libewf_result {
                                Ok(hash) => Ok(hash),
                                Err(e) => {
                                    debug!(
                                        error = %e,
                                        "libewf verify failed, falling back to pure-Rust parser"
                                    );
                                    ewf::verify_with_progress(
                                        &path_for_hash,
                                        &algo_for_hash,
                                        &mut progress_cb,
                                    )
                                    .map_err(|e| e.to_string())
                                }
                            }
                        } else {
                            ewf::verify_with_progress(
                                &path_for_hash,
                                &algo_for_hash,
                                &mut progress_cb,
                            )
                            .map_err(|e| e.to_string())
                        }
                    } else if is_ad1_type(&container_for_hash) {
                        ad1::hash_segments_with_progress(&path_for_hash, &algo_for_hash, &mut progress_cb)
                            .map_err(|e| e.to_string())
                    } else if is_aff4_type(&container_for_hash) {
                        aff4_verify_with_progress(&path_for_hash, &algo_for_hash, &mut progress_cb)
                    } else {
                        // Raw, UFED, archives, unknown — hash file bytes directly
                        info!(path = %path_for_hash, container_type = %container_for_hash, "[HASH-DIAG] Routing to raw::verify_with_progress (archive/raw/unknown)");
                        let raw_result = raw::verify_with_progress(&path_for_hash, &algo_for_hash, &mut progress_cb)
                            .map_err(|e| e.to_string());
                        info!(path = %path_for_hash, success = raw_result.is_ok(), "[HASH-DIAG] raw::verify_with_progress returned");
                        raw_result
                    }
                };

                // Cache successful hash results for future lookups
                if let Ok(ref hash) = result {
                    hash_cache::cache_hash_scoped(
                        &path_for_hash,
                        &algo_for_hash,
                        &cache_scope,
                        hash.clone(),
                    );
                }

                // Stop progress thread
                done_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                let _ = progress_thread.join();

                let duration = start_time.elapsed();
                let duration_ms = duration.as_millis() as u64;
                let throughput_mbs = if duration_ms > 0 && file_size > 0 {
                    Some((file_size as f64 / (1024.0 * 1024.0)) / (duration_ms as f64 / 1000.0))
                } else {
                    None
                };

                (result, duration_ms, throughput_mbs)
            }).await;

            // Handle spawn_blocking failure (panics) — always emit error event
            let (result, duration_ms, throughput_mbs) = match hash_result {
                Ok(r) => r,
                Err(e) => {
                    let err_msg = format!("Internal hash error: {}", e);
                    debug!(error = %err_msg, "spawn_blocking failed");
                    let _ = app_clone.emit(
                        "batch-progress",
                        BatchProgress {
                            path: path_for_error.clone(),
                            status: "error".to_string(),
                            percent: 0.0,
                            files_completed: idx + 1,
                            files_total: num_files,
                            hash: None,
                            algorithm: None,
                            error: Some(err_msg.clone()),
                            chunks_processed: None,
                            chunks_total: None,
                        },
                    );
                    return BatchHashResult {
                        path: path_for_error,
                        algorithm: algo_for_error,
                        hash: None,
                        error: Some(err_msg),
                        duration_ms: None,
                        throughput_mbs: None,
                        drive_kind: Some(drive_label.clone()),
                    };
                }
            };

            // Build result
            let batch_result = match result {
                Ok(hash) => {
                    info!(idx = idx + 1, path = %path, hash_prefix = %&hash[..8.min(hash.len())], algorithm = %algo, "[HASH-DIAG] File completed — emitting batch-progress completed");
                    let _ = app_clone.emit(
                        "batch-progress",
                        BatchProgress {
                            path: path.clone(),
                            status: "completed".to_string(),
                            percent: 100.0,
                            files_completed: idx + 1,
                            files_total: num_files,
                            hash: Some(hash.clone()),
                            algorithm: Some(display_algo.clone()),
                            error: None,
                            chunks_processed: None,
                            chunks_total: None,
                        },
                    );
                    BatchHashResult {
                        path,
                        algorithm: display_algo.clone(),
                        hash: Some(hash),
                        error: None,
                        duration_ms: Some(duration_ms),
                        throughput_mbs,
                        drive_kind: Some(drive_label.clone()),
                    }
                }
                Err(e) => {
                    info!(idx = idx + 1, path = %path, error = %e, "[HASH-DIAG] File error — emitting batch-progress error");
                    let _ = app_clone.emit(
                        "batch-progress",
                        BatchProgress {
                            path: path.clone(),
                            status: "error".to_string(),
                            percent: 0.0,
                            files_completed: idx + 1,
                            files_total: num_files,
                            hash: None,
                            algorithm: None,
                            error: Some(e.clone()),
                            chunks_processed: None,
                            chunks_total: None,
                        },
                    );
                    BatchHashResult {
                        path,
                        algorithm: display_algo,
                        hash: None,
                        error: Some(e),
                        duration_ms: Some(duration_ms),
                        throughput_mbs: None,
                        drive_kind: Some(drive_label),
                    }
                }
            };

            batch_result
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete and collect results
    let mut results = Vec::with_capacity(num_files);
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => {
                // Extremely rare: the outer spawn itself panicked
                debug!(error = %e, "Outer task join error");
            }
        }
    }

    info!(num_files, results = results.len(), "Batch hash complete");
    Ok(results)
}

/// Pause queue processing
///
/// Sets a global pause flag that prevents batch operations from starting
/// new jobs. Jobs already in progress will continue to completion.
#[tauri::command]
pub async fn hash_queue_pause() -> Result<(), String> {
    QUEUE_PAUSED.store(true, Ordering::Relaxed);
    info!("Hash queue paused");
    Ok(())
}

/// Resume queue processing
///
/// Clears the global pause flag, allowing `batch_hash_smart` to resume
/// dispatching new jobs from the queue.
#[tauri::command]
pub async fn hash_queue_resume() -> Result<(), String> {
    QUEUE_PAUSED.store(false, Ordering::Relaxed);
    info!("Hash queue resumed");
    Ok(())
}

/// Clear completed items from queue
///
/// Resets the global queue metrics counters (completed/failed counts,
/// throughput tracking). Active and pending jobs are unaffected.
#[tauri::command]
pub async fn hash_queue_clear_completed() -> Result<(), String> {
    // Reset the completed/failed counters while preserving active state
    let submitted = QUEUE_METRICS.jobs_submitted.load(Ordering::Relaxed);
    let completed = QUEUE_METRICS.jobs_completed.load(Ordering::Relaxed);
    let failed = QUEUE_METRICS.jobs_failed.load(Ordering::Relaxed);

    // Subtract completed+failed from submitted to keep only pending count accurate
    let pending = submitted.saturating_sub(completed + failed);
    QUEUE_METRICS
        .jobs_submitted
        .store(pending, Ordering::Relaxed);
    QUEUE_METRICS.jobs_completed.store(0, Ordering::Relaxed);
    QUEUE_METRICS.jobs_failed.store(0, Ordering::Relaxed);
    QUEUE_METRICS.bytes_processed.store(0, Ordering::Relaxed);
    QUEUE_METRICS.processing_time_ms.store(0, Ordering::Relaxed);

    info!(
        cleared_completed = completed,
        cleared_failed = failed,
        remaining_pending = pending,
        "Hash queue completed items cleared"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::hash_byte_source;
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    fn write_temp_file(bytes: &[u8]) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(bytes).unwrap();
        file.flush().unwrap();
        file
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

    fn disk_hash_source(path: Option<String>, entry_path: Option<String>) -> HashSourceInput {
        HashSourceInput {
            path,
            container_path: None,
            entry_path,
            nested_archive_path: None,
            container_type: None,
            size: None,
            data_addr: None,
            item_addr: None,
        }
    }

    fn batch_file(path: &str, container_type: &str) -> BatchFileInput {
        BatchFileInput {
            path: path.to_string(),
            container_type: container_type.to_string(),
        }
    }

    #[test]
    fn aff4_read_chunk_size_clamps_to_chunk_size_before_usize_conversion() {
        assert_eq!(
            aff4_read_chunk_size(u64::MAX, 0, 1024 * 1024).unwrap(),
            Some(1024 * 1024)
        );
    }

    #[test]
    fn aff4_read_chunk_size_uses_remaining_tail() {
        assert_eq!(aff4_read_chunk_size(4096, 4000, 1024).unwrap(), Some(96));
    }

    #[test]
    fn aff4_read_chunk_size_returns_none_when_complete() {
        assert_eq!(aff4_read_chunk_size(4096, 4096, 1024).unwrap(), None);
    }

    #[test]
    fn aff4_read_chunk_size_rejects_counter_past_stream_size() {
        let err = aff4_read_chunk_size(4096, 8192, 1024).unwrap_err();

        assert!(err.contains("exceeded stream size"));
    }

    #[test]
    fn aff4_read_chunk_size_rejects_zero_chunk_size() {
        let err = aff4_read_chunk_size(4096, 0, 0).unwrap_err();

        assert!(err.contains("cannot be zero"));
    }

    #[test]
    fn checked_aff4_bytes_read_advance_adds_chunk_size() {
        assert_eq!(
            checked_aff4_bytes_read_advance(40, 2, "aff4://stream").unwrap(),
            42
        );
    }

    #[test]
    fn checked_aff4_bytes_read_advance_rejects_overflow() {
        let err = checked_aff4_bytes_read_advance(u64::MAX, 1, "aff4://stream").unwrap_err();

        assert!(err.contains("overflowed"));
    }

    #[test]
    fn progress_counter_value_preserves_representable_values() {
        assert_eq!(progress_counter_value(42), 42);
    }

    #[test]
    fn progress_counter_value_saturates_large_values() {
        assert_eq!(progress_counter_value(u64::MAX), usize::MAX);
    }

    #[test]
    fn is_ewf_type_matches_extension_and_mime_identifiers() {
        assert!(is_ewf_type("e01"));
        assert!(is_ewf_type("EWF"));
        assert!(is_ewf_type("application/x-ewf"));
        assert!(is_ewf_type("EWF-E01"));
        assert!(is_ewf_type("EnCase (E01)"));
        assert!(is_ewf_type("Lx01"));
        assert!(!is_ewf_type("ad1"));
        assert!(!is_ewf_type("raw"));
    }

    #[test]
    fn batch_hash_cache_scope_separates_container_semantics() {
        assert_eq!(batch_hash_cache_scope("E01"), "decoded-ewf");
        assert_eq!(batch_hash_cache_scope("ewf"), "decoded-ewf");
        assert_eq!(batch_hash_cache_scope("application/x-ewf"), "decoded-ewf");
        assert_eq!(batch_hash_cache_scope("ad1"), "ad1-segments");
        assert_eq!(batch_hash_cache_scope("AFF4"), "decoded-aff4");
        assert_eq!(batch_hash_cache_scope("raw"), "raw-file:raw");
        assert_eq!(batch_hash_cache_scope(" "), "raw-file:disk");
    }

    #[test]
    fn segmented_batch_hash_cache_scope_changes_when_companion_segment_changes() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let first = temp_dir.path().join("case.ad1");
        let second = temp_dir.path().join("case.ad2");
        std::fs::write(&first, b"segment one").unwrap();
        std::fs::write(&second, b"segment two").unwrap();
        let segment_paths = vec![first, second.clone()];

        let before = segmented_batch_hash_cache_scope("ad1-segments", &segment_paths);
        std::fs::write(&second, b"segment two changed").unwrap();
        let after = segmented_batch_hash_cache_scope("ad1-segments", &segment_paths);

        assert_ne!(before, after);
    }

    #[test]
    fn batch_hash_cache_scope_for_e01_includes_companion_segment_metadata() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let first = temp_dir.path().join("image.E01");
        let second = temp_dir.path().join("image.E02");
        std::fs::write(&first, b"segment one").unwrap();
        std::fs::write(&second, b"segment two").unwrap();

        let before = batch_hash_cache_scope_for_path("E01", first.to_str().unwrap());
        std::fs::write(&second, b"segment two changed").unwrap();
        let after = batch_hash_cache_scope_for_path("E01", first.to_str().unwrap());

        assert!(before.starts_with("decoded-ewf:segments-"));
        assert_ne!(before, after);
    }

    #[test]
    fn batch_hash_cache_scope_for_l01_includes_companion_segment_metadata() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let first = temp_dir.path().join("logical.L01");
        let second = temp_dir.path().join("logical.L02");
        std::fs::write(&first, b"segment one").unwrap();
        std::fs::write(&second, b"segment two").unwrap();

        let before = batch_hash_cache_scope_for_path("L01", first.to_str().unwrap());
        std::fs::write(&second, b"segment two changed").unwrap();
        let after = batch_hash_cache_scope_for_path("L01", first.to_str().unwrap());

        assert!(before.starts_with("decoded-ewf:segments-"));
        assert_ne!(before, after);
    }

    #[test]
    fn batch_hash_cache_scope_for_lx01_includes_companion_segment_metadata() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let first = temp_dir.path().join("logical.Lx01");
        let second = temp_dir.path().join("logical.Lx02");
        std::fs::write(&first, b"segment one").unwrap();
        std::fs::write(&second, b"segment two").unwrap();

        let before = batch_hash_cache_scope_for_path("Lx01", first.to_str().unwrap());
        std::fs::write(&second, b"segment two changed").unwrap();
        let after = batch_hash_cache_scope_for_path("Lx01", first.to_str().unwrap());

        assert!(before.starts_with("decoded-ewf:segments-"));
        assert_ne!(before, after);
    }

    #[test]
    fn open_hash_source_reads_local_path() {
        let file = write_temp_file(b"source hash test");
        let input = disk_hash_source(Some(file.path().to_string_lossy().into_owned()), None);

        let source = open_hash_source(&input).unwrap();
        let hash = hash_byte_source(source.as_ref(), "sha256").unwrap();

        assert_eq!(
            hash,
            "0217d36ce769599b7e301a7c3f8f6a2a692d4da3ac36eddafcced83390851867"
        );
    }

    #[test]
    fn open_hash_source_accepts_disk_entry_path() {
        let file = write_temp_file(b"disk entry path");
        let mut input = disk_hash_source(None, Some(file.path().to_string_lossy().into_owned()));
        input.container_type = Some("disk".to_string());

        let source = open_hash_source(&input).unwrap();
        assert!(matches!(
            source.source_ref(),
            EvidenceSourceRef::LocalFile { .. }
        ));
    }

    #[test]
    fn open_hash_source_accepts_ad1_address_metadata() {
        let input = HashSourceInput {
            path: None,
            container_path: Some("/cases/evidence.ad1".to_string()),
            entry_path: Some("/Documents/file.txt".to_string()),
            nested_archive_path: None,
            container_type: Some("ad1".to_string()),
            size: Some(128),
            data_addr: Some(8192),
            item_addr: Some(4096),
        };

        let source = open_hash_source(&input).unwrap();

        assert_eq!(source.len().unwrap(), 128);
        assert_eq!(
            source.source_ref(),
            EvidenceSourceRef::ContainerEntry {
                container_path: "/cases/evidence.ad1".to_string(),
                entry_path: "/Documents/file.txt".to_string(),
                container_type: "ad1".to_string(),
            }
        );
    }

    #[test]
    fn open_hash_source_requires_container_entry_path() {
        let input = HashSourceInput {
            path: Some("/cases/evidence.ad1".to_string()),
            container_path: None,
            entry_path: None,
            nested_archive_path: None,
            container_type: Some("ad1".to_string()),
            size: None,
            data_addr: None,
            item_addr: None,
        };

        let err = match open_hash_source(&input) {
            Ok(_) => panic!("expected container source validation to fail"),
            Err(err) => err,
        };
        assert!(err.contains("entryPath"));
    }

    #[test]
    fn split_nested_entry_path_accepts_explicit_and_compact_forms() {
        assert_eq!(
            split_nested_entry_path(Some("inner.zip"), "nested/file.txt"),
            Some(("inner.zip".to_string(), "nested/file.txt".to_string()))
        );
        assert_eq!(
            split_nested_entry_path(None, "inner.zip::nested/file.txt"),
            Some(("inner.zip".to_string(), "nested/file.txt".to_string()))
        );
        assert_eq!(split_nested_entry_path(None, "plain/file.txt"), None);
    }

    #[test]
    fn split_nested_entry_path_rejects_empty_nested_parts() {
        assert_eq!(split_nested_entry_path(Some(""), "nested/file.txt"), None);
        assert_eq!(split_nested_entry_path(Some("inner.zip"), ""), None);
        assert_eq!(split_nested_entry_path(None, "::nested/file.txt"), None);
        assert_eq!(split_nested_entry_path(None, "inner.zip::"), None);
    }

    #[test]
    fn validate_hash_source_request_normalizes_algorithm() {
        let input = disk_hash_source(Some("/cases/evidence.bin".to_string()), None);

        let algorithm = validate_hash_source_request(&input, "sha-256").unwrap();

        assert_eq!(algorithm, "SHA-256");
    }

    #[test]
    fn validate_hash_source_request_rejects_unknown_algorithm() {
        let input = disk_hash_source(Some("/cases/evidence.bin".to_string()), None);

        let err = validate_hash_source_request(&input, "rot13").unwrap_err();

        assert!(err.contains("Unsupported hash algorithm"));
    }

    #[test]
    fn validate_hash_source_request_rejects_missing_disk_path() {
        let input = disk_hash_source(None, None);

        let err = validate_hash_source_request(&input, "sha256").unwrap_err();

        assert!(err.contains("requires a path or entryPath"));
    }

    #[test]
    fn validate_hash_source_request_rejects_empty_fields() {
        let input = disk_hash_source(Some(" ".to_string()), None);

        let err = validate_hash_source_request(&input, "sha256").unwrap_err();

        assert!(err.contains("path cannot be empty"));
    }

    #[test]
    fn validate_hash_source_request_rejects_oversized_path() {
        let input = disk_hash_source(Some("a".repeat(MAX_HASH_SOURCE_FIELD_CHARS + 1)), None);

        let err = validate_hash_source_request(&input, "sha256").unwrap_err();

        assert!(err.contains("path exceeds limit"));
    }

    #[test]
    fn validate_hash_source_request_rejects_missing_container_entry_path() {
        let input = HashSourceInput {
            path: Some("/cases/evidence.ad1".to_string()),
            container_path: None,
            entry_path: None,
            nested_archive_path: None,
            container_type: Some("ad1".to_string()),
            size: None,
            data_addr: None,
            item_addr: None,
        };

        let err = validate_hash_source_request(&input, "sha256").unwrap_err();

        assert!(err.contains("entryPath"));
    }

    #[test]
    fn validate_hash_source_request_rejects_invalid_compact_nested_path() {
        let input = HashSourceInput {
            path: None,
            container_path: Some("/cases/outer.zip".to_string()),
            entry_path: Some("inner.zip::".to_string()),
            nested_archive_path: None,
            container_type: Some("zip".to_string()),
            size: None,
            data_addr: None,
            item_addr: None,
        };

        let err = validate_hash_source_request(&input, "sha256").unwrap_err();

        assert!(err.contains("compact entryPath"));
    }

    #[test]
    fn truncate_hash_source_text_caps_character_count() {
        let value = format!("{}{}", "a".repeat(MAX_HASH_SOURCE_ID_CHARS), "tail");

        let truncated = truncate_hash_source_text(value, MAX_HASH_SOURCE_ID_CHARS);

        assert_eq!(truncated.chars().count(), MAX_HASH_SOURCE_ID_CHARS);
        assert!(!truncated.ends_with("tail"));
    }

    #[test]
    fn hash_source_id_includes_ad1_item_and_data_addresses() {
        let input = HashSourceInput {
            path: None,
            container_path: Some("/cases/evidence.ad1".to_string()),
            entry_path: Some("/Documents/file.txt".to_string()),
            nested_archive_path: None,
            container_type: Some("ad1".to_string()),
            size: Some(128),
            data_addr: Some(0x2000),
            item_addr: Some(0x1000),
        };
        let source_ref = EvidenceSourceRef::ContainerEntry {
            container_path: "/cases/evidence.ad1".to_string(),
            entry_path: "/Documents/file.txt".to_string(),
            container_type: "ad1".to_string(),
        };

        let source_id = hash_source_id(&input, Some(&source_ref));

        assert_eq!(
            source_id,
            "ad1:/cases/evidence.ad1:/Documents/file.txt#item=0x1000;data=0x2000"
        );
    }

    #[test]
    fn hash_source_id_omits_address_suffix_for_non_ad1_entries() {
        let input = HashSourceInput {
            path: None,
            container_path: Some("/cases/disk.E01".to_string()),
            entry_path: Some("/Documents/file.txt".to_string()),
            nested_archive_path: None,
            container_type: Some("e01".to_string()),
            size: Some(128),
            data_addr: Some(0x2000),
            item_addr: Some(0x1000),
        };
        let source_ref = EvidenceSourceRef::ContainerEntry {
            container_path: "/cases/disk.E01".to_string(),
            entry_path: "/Documents/file.txt".to_string(),
            container_type: "e01".to_string(),
        };

        let source_id = hash_source_id(&input, Some(&source_ref));

        assert_eq!(source_id, "e01:/cases/disk.E01:/Documents/file.txt");
    }

    #[test]
    fn hash_source_id_distinguishes_ad1_entries_with_same_path() {
        let first = HashSourceInput {
            path: None,
            container_path: Some("/cases/evidence.ad1".to_string()),
            entry_path: Some("/Documents/file.txt".to_string()),
            nested_archive_path: None,
            container_type: Some("ad1".to_string()),
            size: Some(128),
            data_addr: Some(0x2000),
            item_addr: Some(0x1000),
        };
        let mut second = first.clone();
        second.data_addr = Some(0x4000);
        second.item_addr = Some(0x3000);
        let source_ref = EvidenceSourceRef::ContainerEntry {
            container_path: "/cases/evidence.ad1".to_string(),
            entry_path: "/Documents/file.txt".to_string(),
            container_type: "ad1".to_string(),
        };

        let first_id = hash_source_id(&first, Some(&source_ref));
        let second_id = hash_source_id(&second, Some(&source_ref));

        assert_ne!(first_id, second_id);
        assert!(first_id.ends_with("#item=0x1000;data=0x2000"));
        assert!(second_id.ends_with("#item=0x3000;data=0x4000"));
    }

    #[test]
    fn validate_batch_hash_request_normalizes_algorithm_and_overrides() {
        let files = vec![batch_file("/cases/evidence.bin", "raw")];
        let mut overrides = HashMap::new();
        overrides.insert(" SSD ".to_string(), 3);

        let (display, worker, overrides) =
            validate_batch_hash_request(&files, "sha-256", Some(overrides)).unwrap();

        assert_eq!(display, "SHA-256");
        assert_eq!(worker, "sha256");
        assert_eq!(overrides.get("ssd"), Some(&3));
    }

    #[test]
    fn validate_batch_hash_request_rejects_excessive_file_count() {
        let files: Vec<_> = (0..=MAX_BATCH_HASH_FILES)
            .map(|index| batch_file(&format!("/cases/{index}.bin"), "raw"))
            .collect();

        let err = validate_batch_hash_request(&files, "sha256", None).unwrap_err();

        assert!(err.contains("exceeding limit"));
    }

    #[test]
    fn validate_batch_hash_request_rejects_empty_path() {
        let files = vec![batch_file(" ", "raw")];

        let err = validate_batch_hash_request(&files, "sha256", None).unwrap_err();

        assert!(err.contains("batch path cannot be empty"));
    }

    #[test]
    fn validate_batch_hash_request_rejects_oversized_container_type() {
        let files = vec![batch_file(
            "/cases/evidence.bin",
            &"r".repeat(MAX_HASH_SOURCE_CONTAINER_TYPE_CHARS + 1),
        )];

        let err = validate_batch_hash_request(&files, "sha256", None).unwrap_err();

        assert!(err.contains("batch containerType exceeds limit"));
    }

    #[test]
    fn validate_batch_hash_request_rejects_unknown_algorithm() {
        let files = vec![batch_file("/cases/evidence.bin", "raw")];

        let err = validate_batch_hash_request(&files, "rot13", None).unwrap_err();

        assert!(err.contains("Unsupported hash algorithm"));
    }

    #[test]
    fn validate_batch_hash_overrides_rejects_unknown_key() {
        let mut overrides = HashMap::new();
        overrides.insert("gpu".to_string(), 1);

        let err = validate_batch_hash_overrides(overrides).unwrap_err();

        assert!(err.contains("Unknown batch hash concurrency override key"));
    }

    #[test]
    fn validate_batch_hash_overrides_rejects_excessive_value() {
        let mut overrides = HashMap::new();
        overrides.insert("ssd".to_string(), MAX_BATCH_HASH_CONCURRENCY_OVERRIDE + 1);

        let err = validate_batch_hash_overrides(overrides).unwrap_err();

        assert!(err.contains("exceeding limit"));
    }

    #[test]
    fn resolve_batch_hash_concurrency_clamps_effective_value() {
        let mut overrides = HashMap::new();
        overrides.insert("ssd".to_string(), MAX_EFFECTIVE_BATCH_HASH_CONCURRENCY + 10);

        assert_eq!(
            resolve_batch_hash_concurrency(StorageClass::InternalSsd, &overrides),
            MAX_EFFECTIVE_BATCH_HASH_CONCURRENCY
        );
    }

    #[test]
    fn resolve_batch_hash_concurrency_treats_zero_as_default() {
        let mut overrides = HashMap::new();
        overrides.insert("hdd".to_string(), 0);

        assert_eq!(
            resolve_batch_hash_concurrency(StorageClass::InternalHdd, &overrides),
            StorageClass::InternalHdd.default_concurrency()
        );
    }

    #[test]
    fn open_hash_source_identifies_nested_archive_source_with_known_size() {
        let input = HashSourceInput {
            path: None,
            container_path: Some("/cases/outer.zip".to_string()),
            entry_path: Some("nested/file.txt".to_string()),
            nested_archive_path: Some("inner.zip".to_string()),
            container_type: Some("zip".to_string()),
            size: Some(42),
            data_addr: None,
            item_addr: None,
        };

        let source = open_hash_source(&input).unwrap();

        assert_eq!(source.len().unwrap(), 42);
        assert_eq!(
            source.source_ref(),
            EvidenceSourceRef::NestedContainerEntry {
                container_path: "/cases/outer.zip".to_string(),
                nested_container_path: "inner.zip".to_string(),
                entry_path: "nested/file.txt".to_string(),
                container_type: Some("zip".to_string()),
            }
        );
    }

    #[test]
    fn open_hash_source_reads_nested_archive_range() {
        crate::commands::archive::nested::nested_container_clear_cache_for_tests();
        let tmp = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
        let inner_zip = zip_bytes("nested/file.txt", b"abcdef", CompressionMethod::Deflated);
        write_zip_file(tmp.path(), "inner.zip", &inner_zip);
        let input = HashSourceInput {
            path: None,
            container_path: Some(tmp.path().to_string_lossy().to_string()),
            entry_path: Some("nested/file.txt".to_string()),
            nested_archive_path: Some("inner.zip".to_string()),
            container_type: Some("zip".to_string()),
            size: None,
            data_addr: None,
            item_addr: None,
        };

        let source = open_hash_source(&input).unwrap();

        assert_eq!(source.len().unwrap(), 6);
        assert_eq!(source.read_range(2, 3).unwrap(), b"cde");
        assert!(source.read_range(6, 3).unwrap().is_empty());
    }
}
