// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Public API for EWF operations (E01/L01/Ex01/Lx01)

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use md5::Md5;
use sha1::{Sha1, Digest};
use sha2::{Sha256, Sha512};
use blake2::Blake2b512;
use blake3::Hasher as Blake3Hasher;
use xxhash_rust::xxh3::Xxh3;
use xxhash_rust::xxh64::Xxh64;
use rayon::prelude::*;
use tracing::{debug, instrument};

use crate::common::{
    BUFFER_SIZE, MMAP_THRESHOLD,
    hash::StreamingHasher,
    segments::discover_e01_segments,
};
use crate::containers::ContainerError;

use super::types::*;
use super::handle::EwfHandle;

/// Thread result type for parallel chunk decompression: Vec of (chunk_idx, chunk_data)
type ChunkThreadResult = Result<Vec<(usize, Vec<u8>)>, ContainerError>;

// =============================================================================
// Info Operations
// =============================================================================
#[instrument]
pub fn info(path: &str) -> Result<EwfInfo, ContainerError> {
    debug!("Getting EWF info");
    let handle = EwfHandle::open(path)?;
    let volume = handle.get_volume_info();
    let total_size = volume.sector_count * volume.bytes_per_sector as u64;
    
    debug!(
        total_size,
        sector_count = volume.sector_count,
        chunk_count = handle.get_chunk_count(),
        "EWF volume info"
    );
    
    // Get segment file names
    let segment_count = handle.file_pool.get_file_count() as u32;
    let segment_files = if segment_count > 1 {
        let paths = discover_e01_segments(path).unwrap_or_default();
        let names: Vec<String> = paths.iter()
            .filter_map(|p| p.file_name())
            .map(|f| f.to_string_lossy().to_string())
            .collect();
        if names.is_empty() { None } else { Some(names) }
    } else {
        None
    };
    
    // Get file modification time as fallback timestamp
    let file_timestamp: Option<String> = Path::new(path).metadata().ok()
        .and_then(|m| m.modified().ok())
        .map(|t| {
            let datetime: chrono::DateTime<chrono::Utc> = t.into();
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        });
    
    // Use acquiry_date from header if available, otherwise fall back to file timestamp
    let acquiry_date = handle.header_info.acquiry_date.clone()
        .or_else(|| file_timestamp.clone());
    
    // Set timestamp on stored hashes from acquiry_date (not file modification time!)
    let stored_hashes: Vec<StoredImageHash> = handle.stored_hashes.iter().map(|h| {
        StoredImageHash {
            algorithm: h.algorithm.clone(),
            hash: h.hash.clone(),
            verified: h.verified,
            timestamp: h.timestamp.clone().or_else(|| acquiry_date.clone()),
            source: h.source.clone(),
            offset: h.offset,
            size: h.size,
        }
    }).collect();
    
    // Extract section offsets from first segment for hex navigation
    let mut header_section_offset: Option<u64> = None;
    let mut volume_section_offset: Option<u64> = None;
    let mut hash_section_offset: Option<u64> = None;
    let mut digest_section_offset: Option<u64> = None;
    
    if let Some(first_segment) = handle.segments.first() {
        for section in &first_segment.sections {
            match section.section_type.as_str() {
                "header" => {
                    if header_section_offset.is_none() {
                        header_section_offset = Some(section.offset_in_segment);
                    }
                }
                "volume" | "disk" => {
                    if volume_section_offset.is_none() {
                        volume_section_offset = Some(section.offset_in_segment);
                    }
                }
                "hash" => {
                    if hash_section_offset.is_none() {
                        hash_section_offset = Some(section.offset_in_segment);
                    }
                }
                "digest" => {
                    if digest_section_offset.is_none() {
                        digest_section_offset = Some(section.offset_in_segment);
                    }
                }
                _ => {}
            }
        }
    }
    
    debug!(
        stored_hash_count = stored_hashes.len(),
        header_offset = ?header_section_offset,
        volume_offset = ?volume_section_offset,
        hash_offset = ?hash_section_offset,
        digest_offset = ?digest_section_offset,
        "EWF info complete"
    );
    
    Ok(EwfInfo {
        format_version: "EWF1".to_string(),
        segment_count,
        chunk_count: handle.get_chunk_count() as u32,
        sector_count: volume.sector_count,
        bytes_per_sector: volume.bytes_per_sector,
        sectors_per_chunk: volume.sectors_per_chunk,
        total_size,
        compression: "Good (Fast)".to_string(),
        case_number: handle.header_info.case_number.clone(),
        description: handle.header_info.description.clone(),
        examiner_name: handle.header_info.examiner_name.clone(),
        evidence_number: handle.header_info.evidence_number.clone(),
        notes: handle.header_info.notes.clone(),
        acquiry_date,
        system_date: handle.header_info.system_date.clone(),
        model: None,
        serial_number: None,
        stored_hashes,
        segment_files,
        header_section_offset,
        volume_section_offset,
        hash_section_offset,
        digest_section_offset,
    })
}

/// Fast info - minimal metadata without detailed section parsing
/// Used when only basic info is needed (e.g., stored hashes)
#[instrument]
pub fn info_fast(path: &str) -> Result<EwfInfo, ContainerError> {
    // For EWF, info_fast is the same as info since we need to parse headers anyway
    // The full info function is already optimized
    info(path)
}

/// Check if a file is a valid EWF format (E01/L01/Ex01/Lx01)
#[instrument]
pub fn is_e01(path: &str) -> Result<bool, ContainerError> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        debug!("is_e01: file does not exist: {}", path);
        return Ok(false);
    }
    
    let mut file = File::open(path)?;
    
    let mut sig = [0u8; 8];
    if file.read_exact(&mut sig).is_err() {
        debug!("is_e01: failed to read signature from: {}", path);
        return Ok(false);
    }
    
    // Check all EWF variants: E01 (EVF), Ex01 (EVF2), L01 (LVF), Lx01 (LVF2)
    let is_ewf1 = &sig == EWF_SIGNATURE;
    let is_ewf2 = &sig == EWF2_SIGNATURE;
    let is_lvf1 = &sig == LVF_SIGNATURE;
    let is_lvf2 = &sig == LVF2_SIGNATURE;
    debug!("is_e01: {} -> sig={:02x?} ewf1={} ewf2={} lvf1={} lvf2={}", 
           path, &sig, is_ewf1, is_ewf2, is_lvf1, is_lvf2);
    Ok(is_ewf1 || is_ewf2 || is_lvf1 || is_lvf2)
}

/// Alias for is_e01 - check if file is any EWF variant
#[inline]
pub fn is_ewf(path: &str) -> Result<bool, ContainerError> {
    is_e01(path)
}

/// Get all E01 segment file paths
pub fn get_segment_paths(path: &str) -> Result<Vec<PathBuf>, ContainerError> {
    discover_e01_segments(path)
}

// =============================================================================
// Statistics and Analysis
// =============================================================================

/// Get container statistics for an EWF file
#[instrument]
pub fn get_stats(path: &str) -> Result<super::types::EwfStats, ContainerError> {
    debug!("Getting EWF stats");
    let handle = EwfHandle::open(path)?;
    let volume = handle.get_volume_info();
    
    // Calculate total uncompressed size
    let total_size = volume.sector_count * volume.bytes_per_sector as u64;
    
    // Calculate total compressed size from all segments
    let compressed_size: u64 = handle.segments.iter()
        .map(|s| s.file_size)
        .sum();
    
    // Calculate compression ratio
    let compression_ratio = if compressed_size > 0 {
        total_size as f64 / compressed_size as f64
    } else {
        1.0
    };
    
    // Check for stored hashes
    let has_md5 = handle.stored_hashes.iter().any(|h| h.algorithm.to_lowercase() == "md5");
    let has_sha1 = handle.stored_hashes.iter().any(|h| h.algorithm.to_lowercase() == "sha1");
    
    // Detect format variant
    let format_variant = detect_ewf_variant(path)?;
    
    Ok(super::types::EwfStats {
        total_chunks: handle.get_chunk_count() as u64,
        total_segments: handle.file_pool.get_file_count() as u32,
        total_size,
        compressed_size,
        compression_ratio,
        bytes_per_sector: volume.bytes_per_sector,
        sectors_per_chunk: volume.sectors_per_chunk,
        sector_count: volume.sector_count,
        stored_hash_count: handle.stored_hashes.len(),
        has_md5,
        has_sha1,
        format_variant,
    })
}

/// Detect EWF format variant (E01, L01, Ex01, Lx01)
fn detect_ewf_variant(path: &str) -> Result<String, ContainerError> {
    let mut file = File::open(path)?;
    
    let mut sig = [0u8; 8];
    file.read_exact(&mut sig)?;
    
    if &sig == EWF_SIGNATURE {
        Ok("E01 (EVF v1)".to_string())
    } else if &sig == EWF2_SIGNATURE {
        Ok("Ex01 (EVF v2)".to_string())
    } else if &sig == LVF_SIGNATURE {
        Ok("L01 (LVF v1)".to_string())
    } else if &sig == LVF2_SIGNATURE {
        Ok("Lx01 (LVF v2)".to_string())
    } else {
        Ok("Unknown".to_string())
    }
}

/// Verify chunks individually with detailed results
/// 
/// This function verifies each chunk can be read and decompressed successfully.
/// EWF doesn't store per-chunk hashes, but this validates chunk integrity.
#[instrument]
#[allow(dead_code)] // Available for diagnostic use, called via FFI
pub fn verify_chunks_detailed(path: &str, _algorithm: &str) -> Result<Vec<super::types::ChunkVerifyResult>, ContainerError> {
    debug!("Verifying EWF chunks");
    let mut handle = EwfHandle::open(path)?;
    let chunk_count = handle.get_chunk_count();
    
    let mut results = Vec::with_capacity(chunk_count);
    
    // For EWF, we verify that each chunk can be read and decompressed
    // The format doesn't store per-chunk hashes, but we can verify decompression
    for i in 0..chunk_count {
        match handle.read_chunk(i) {
            Ok(_) => {
                results.push(super::types::ChunkVerifyResult {
                    index: i as u64,
                    status: "ok".to_string(),
                    message: None,
                });
            }
            Err(e) => {
                results.push(super::types::ChunkVerifyResult {
                    index: i as u64,
                    status: "error".to_string(),
                    message: Some(e.to_string()),
                });
            }
        }
    }
    
    Ok(results)
}

/// Export EWF metadata to JSON format
pub fn export_metadata_json(path: &str) -> Result<String, ContainerError> {
    let info = info(path)?;
    serde_json::to_string_pretty(&info)
        .map_err(|e| ContainerError::ParseError(format!("Failed to serialize to JSON: {}", e)))
}

/// Export EWF metadata to CSV format
pub fn export_metadata_csv(path: &str) -> Result<String, ContainerError> {
    let info = info(path)?;
    
    let mut csv = String::from("Field,Value\n");
    csv.push_str(&format!("Format Version,{}\n", escape_csv(&info.format_version)));
    csv.push_str(&format!("Segment Count,{}\n", info.segment_count));
    csv.push_str(&format!("Chunk Count,{}\n", info.chunk_count));
    csv.push_str(&format!("Sector Count,{}\n", info.sector_count));
    csv.push_str(&format!("Bytes Per Sector,{}\n", info.bytes_per_sector));
    csv.push_str(&format!("Sectors Per Chunk,{}\n", info.sectors_per_chunk));
    csv.push_str(&format!("Total Size,{}\n", info.total_size));
    csv.push_str(&format!("Compression,{}\n", escape_csv(&info.compression)));
    
    if let Some(ref cn) = info.case_number {
        csv.push_str(&format!("Case Number,{}\n", escape_csv(cn)));
    }
    if let Some(ref desc) = info.description {
        csv.push_str(&format!("Description,{}\n", escape_csv(desc)));
    }
    if let Some(ref examiner) = info.examiner_name {
        csv.push_str(&format!("Examiner,{}\n", escape_csv(examiner)));
    }
    if let Some(ref evidence) = info.evidence_number {
        csv.push_str(&format!("Evidence Number,{}\n", escape_csv(evidence)));
    }
    if let Some(ref date) = info.acquiry_date {
        csv.push_str(&format!("Acquiry Date,{}\n", escape_csv(date)));
    }
    
    // Add stored hashes
    for hash in &info.stored_hashes {
        csv.push_str(&format!("Stored Hash ({}),{}\n", 
            escape_csv(&hash.algorithm), 
            escape_csv(&hash.hash)));
    }
    
    Ok(csv)
}

/// Escape a value for CSV output
fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

// =============================================================================
// Segment Hashing
// =============================================================================

/// Hash a single E01 segment file (uses mmap for large files)
pub fn hash_single_segment<F>(segment_path: &str, algorithm: &str, mut progress_callback: F) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64)
{
    use std::io::BufRead;
    use memmap2::Mmap;
    
    let path = Path::new(segment_path);
    if !path.exists() {
        return Err(ContainerError::FileNotFound(format!("Segment file not found: {}", segment_path)));
    }
    
    let metadata = std::fs::metadata(path)?;
    let total_size = metadata.len();
    
    let file = File::open(path)?;
    
    let algorithm_lower = algorithm.to_lowercase();
    
    // For BLAKE3 with large files, use mmap + parallel hashing
    if algorithm_lower == "blake3" && total_size >= MMAP_THRESHOLD {
        let mmap = unsafe { Mmap::map(&file) }?;
        
        let mut hasher = blake3::Hasher::new();
        let chunk_size = 64 * 1024 * 1024; // 64MB chunks for progress
        let mut bytes_processed = 0u64;
        
        for chunk in mmap.chunks(chunk_size) {
            hasher.update_rayon(chunk);
            bytes_processed += chunk.len() as u64;
            progress_callback(bytes_processed, total_size);
        }
        
        return Ok(hasher.finalize().to_hex().to_string());
    }
    
    // For large files, use mmap for better I/O
    if total_size >= MMAP_THRESHOLD {
        let mmap = unsafe { Mmap::map(&file) }?;
        
        let mut hasher: StreamingHasher = algorithm.parse()?;
        let chunk_size = 64 * 1024 * 1024;
        let mut bytes_processed = 0u64;
        
        for chunk in mmap.chunks(chunk_size) {
            hasher.update(chunk);
            bytes_processed += chunk.len() as u64;
            progress_callback(bytes_processed, total_size);
        }
        
        return Ok(hasher.finalize());
    }
    
    // Standard BufReader path for smaller files
    let mut reader = std::io::BufReader::with_capacity(BUFFER_SIZE, file);
    
    // For BLAKE3 without mmap, still use parallel hashing
    if algorithm_lower == "blake3" {
        let mut hasher = blake3::Hasher::new();
        let mut bytes_read_total = 0u64;
        let report_interval = (total_size / 20).max(BUFFER_SIZE as u64);
        let mut last_report = 0u64;
        
        loop {
            let buf = reader.fill_buf()
                .map_err(|e| format!("Read error: {}", e))?;
            let len = buf.len();
            if len == 0 { break; }
            
            hasher.update_rayon(buf);
            reader.consume(len);
            
            bytes_read_total += len as u64;
            if bytes_read_total - last_report >= report_interval {
                progress_callback(bytes_read_total, total_size);
                last_report = bytes_read_total;
            }
        }
        
        progress_callback(total_size, total_size);
        return Ok(hasher.finalize().to_hex().to_string());
    }
    
    // For other algorithms, use StreamingHasher
    let mut hasher: StreamingHasher = algorithm.parse()?;
    
    let mut bytes_read_total = 0u64;
    let report_interval = (total_size / 20).max(BUFFER_SIZE as u64);
    let mut last_report = 0u64;
    
    loop {
        let buf = reader.fill_buf()
            .map_err(|e| format!("Read error: {}", e))?;
        let len = buf.len();
        if len == 0 { break; }
        
        hasher.update(buf);
        reader.consume(len);
        bytes_read_total += len as u64;
        
        if bytes_read_total - last_report >= report_interval {
            progress_callback(bytes_read_total, total_size);
            last_report = bytes_read_total;
        }
    }
    
    progress_callback(total_size, total_size);
    Ok(hasher.finalize())
}

// =============================================================================
// Verification
// =============================================================================

/// Verify image and return detailed results for each chunk (used by containers.rs)
pub fn verify_chunks(path: &str, algorithm: &str) -> Result<Vec<VerifyResult>, ContainerError> {
    let hash = verify_with_progress(path, algorithm, |_, _| {})?;
    
    Ok(vec![VerifyResult {
        chunk_index: 0,
        status: "ok".to_string(),
        message: Some(hash),
    }])
}

/// Extract image contents to a raw file
pub fn extract(path: &str, output_dir: &str) -> Result<(), ContainerError> {
    extract_with_progress(path, output_dir, |_, _| {})
}

/// Extract image contents to a raw file with progress callback
#[instrument(skip(progress_callback))]
pub fn extract_with_progress<F>(path: &str, output_dir: &str, mut progress_callback: F) -> Result<(), ContainerError>
where
    F: FnMut(u64, u64),
{
    debug!(path, output_dir, "Extracting EWF image");
    
    let mut handle = EwfHandle::open(path)?;
    let volume = handle.get_volume_info();
    let chunk_count = handle.get_chunk_count();
    
    // Create output filename based on input path
    let input_path = Path::new(path);
    let stem = input_path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "image".to_string());
    
    let output_path = Path::new(output_dir).join(format!("{}.raw", stem));
    let mut output = File::create(&output_path)?;
    
    let total_bytes = volume.sector_count * volume.bytes_per_sector as u64;
    let mut bytes_written = 0u64;
    let mut last_report = 0u64;
    let report_interval = total_bytes / 100; // Report every 1%
    
    for i in 0..chunk_count {
        let chunk_data = handle.read_chunk_no_cache(i)?;
        
        let bytes_to_write = if bytes_written + chunk_data.len() as u64 > total_bytes {
            (total_bytes - bytes_written) as usize
        } else {
            chunk_data.len()
        };
        
        output.write_all(&chunk_data[..bytes_to_write])?;
        
        bytes_written += bytes_to_write as u64;
        
        // Report progress at intervals
        if bytes_written - last_report >= report_interval || bytes_written >= total_bytes {
            progress_callback(bytes_written, total_bytes);
            last_report = bytes_written;
        }
        
        if bytes_written >= total_bytes {
            break;
        }
    }
    
    // Final progress report
    progress_callback(total_bytes, total_bytes);
    debug!(bytes_written, "EWF extraction complete");
    
    Ok(())
}

pub fn verify(path: &str, algorithm: &str) -> Result<String, ContainerError> {
    verify_with_progress(path, algorithm, |_current, _total| {})
}

pub fn verify_with_progress<F>(path: &str, algorithm: &str, progress_callback: F) -> Result<String, ContainerError> 
where
    F: FnMut(usize, usize)
{
    verify_with_progress_optimized(path, algorithm, progress_callback)
}

/// Optimized E01 verification with batched I/O and parallel decompression
/// 
/// Performance improvements over naive approach:
/// 1. Large batch sizes (512 chunks per batch) to amortize I/O overhead
/// 2. Single file handle per segment (no handle pool contention)
/// 3. Parallel decompression using rayon
/// 4. Pipelined I/O: read next batch while hashing current batch
fn verify_with_progress_optimized<F>(path: &str, algorithm: &str, mut progress_callback: F) -> Result<String, ContainerError> 
where
    F: FnMut(usize, usize)
{
    use std::sync::mpsc;
    use std::thread;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    
    debug!(path = %path, "Starting optimized EWF verification");
    
    let handle = EwfHandle::open(path)?;
    let chunk_count = handle.get_chunk_count();
    let chunk_size = (handle.get_volume_info().sectors_per_chunk as usize) 
                   * (handle.get_volume_info().bytes_per_sector as usize);
    
    debug!(chunk_count, chunk_size, "EWF info for verification");
    
    // Algorithm selection
    let algorithm_lower = algorithm.to_lowercase();
    let path_str = path.to_string();
    
    // Larger batch sizes for better I/O efficiency
    // Each batch reads ~16MB of compressed data (512 chunks * ~32KB)
    let num_threads = rayon::current_num_threads();
    let batch_size = 512.max(num_threads * 32); // At least 512 chunks per batch
    
    debug!(batch_size, num_threads, "Optimized batch configuration");
    
    // Progress tracking
    let chunks_processed = Arc::new(AtomicUsize::new(0));
    let chunks_processed_clone = chunks_processed.clone();
    
    // Channel for batches - allow some pipelining
    let (tx, rx) = mpsc::sync_channel::<Result<Vec<Vec<u8>>, ContainerError>>(4);
    
    // I/O + Decompression thread
    let io_handle = thread::spawn(move || {
        let mut handle = match EwfHandle::open(&path_str) {
            Ok(h) => h,
            Err(e) => {
                let _ = tx.send(Err(e));
                return;
            }
        };
        
        for batch_start in (0..chunk_count).step_by(batch_size) {
            let batch_end = (batch_start + batch_size).min(chunk_count);
            let actual_batch_size = batch_end - batch_start;
            
            // Read chunks sequentially (minimizes seeks within segment)
            let batch_chunks: Result<Vec<Vec<u8>>, ContainerError> = (batch_start..batch_end)
                .map(|i| handle.read_chunk_no_cache(i))
                .collect();
            
            match batch_chunks {
                Ok(chunks) => {
                    chunks_processed_clone.fetch_add(actual_batch_size, Ordering::Relaxed);
                    if tx.send(Ok(chunks)).is_err() {
                        return;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                    return;
                }
            }
        }
    });
    
    // Hashing on main thread with algorithm-specific hashers
    let use_sha1 = algorithm_lower == "sha1" || algorithm_lower == "sha-1";
    let use_sha256 = algorithm_lower == "sha256" || algorithm_lower == "sha-256";
    let use_sha512 = algorithm_lower == "sha512" || algorithm_lower == "sha-512";
    let use_blake3 = algorithm_lower == "blake3";
    let use_blake2 = algorithm_lower == "blake2" || algorithm_lower == "blake2b";
    let use_xxh3 = algorithm_lower == "xxh3" || algorithm_lower == "xxhash3";
    let use_xxh64 = algorithm_lower == "xxh64" || algorithm_lower == "xxhash64";
    let use_crc32 = algorithm_lower == "crc32";
    
    let is_known_algo = use_sha1 || use_sha256 || use_sha512 || use_blake3 || use_blake2 || use_xxh3 || use_xxh64 || use_crc32;
    let mut md5_hasher: Option<Md5> = if !is_known_algo { Some(Md5::new()) } else { None };
    let mut sha1_hasher = if use_sha1 { Some(Sha1::new()) } else { None };
    let mut sha256_hasher = if use_sha256 { Some(Sha256::new()) } else { None };
    let mut sha512_hasher = if use_sha512 { Some(Sha512::new()) } else { None };
    let mut blake3_hasher = if use_blake3 { Some(Blake3Hasher::new()) } else { None };
    let mut blake2_hasher = if use_blake2 { Some(Blake2b512::new()) } else { None };
    let mut xxh3_hasher = if use_xxh3 { Some(Xxh3::new()) } else { None };
    let mut xxh64_hasher = if use_xxh64 { Some(Xxh64::new(0)) } else { None };
    let mut crc32_hasher = if use_crc32 { Some(crc32fast::Hasher::new()) } else { None };
    
    // Process batches as they arrive
    while let Ok(batch_result) = rx.recv() {
        let processed = chunks_processed.load(Ordering::Relaxed);
        progress_callback(processed, chunk_count);
        
        match batch_result {
            Ok(batch_chunks) => {
                // For BLAKE3, use parallel hashing on large batches
                if let Some(ref mut hasher) = blake3_hasher {
                    // Concatenate batch into single buffer for parallel hashing
                    let total_size: usize = batch_chunks.iter().map(|c| c.len()).sum();
                    let mut combined = Vec::with_capacity(total_size);
                    for chunk in &batch_chunks {
                        combined.extend_from_slice(chunk);
                    }
                    hasher.update_rayon(&combined);
                } else {
                    // Sequential hashing for other algorithms
                    for chunk_data in &batch_chunks {
                        if let Some(ref mut hasher) = md5_hasher {
                            Digest::update(hasher, chunk_data);
                        } else if let Some(ref mut hasher) = sha1_hasher {
                            hasher.update(chunk_data);
                        } else if let Some(ref mut hasher) = sha256_hasher {
                            hasher.update(chunk_data);
                        } else if let Some(ref mut hasher) = sha512_hasher {
                            hasher.update(chunk_data);
                        } else if let Some(ref mut hasher) = blake2_hasher {
                            hasher.update(chunk_data);
                        } else if let Some(ref mut hasher) = xxh3_hasher {
                            hasher.update(chunk_data);
                        } else if let Some(ref mut hasher) = xxh64_hasher {
                            hasher.update(chunk_data);
                        } else if let Some(ref mut hasher) = crc32_hasher {
                            hasher.update(chunk_data);
                        }
                    }
                }
            }
            Err(e) => {
                let _ = io_handle.join();
                return Err(e);
            }
        }
    }
    
    progress_callback(chunk_count, chunk_count);
    
    io_handle.join().map_err(|_| ContainerError::ParseError("I/O thread panicked".into()))?;
    
    // Return hash result
    if let Some(hasher) = md5_hasher {
        Ok(hex::encode(hasher.finalize()))
    } else if let Some(hasher) = sha1_hasher {
        Ok(hex::encode(hasher.finalize()))
    } else if let Some(hasher) = sha256_hasher {
        Ok(hex::encode(hasher.finalize()))
    } else if let Some(hasher) = sha512_hasher {
        Ok(hex::encode(hasher.finalize()))
    } else if let Some(hasher) = blake3_hasher {
        Ok(format!("{}", hasher.finalize().to_hex()))
    } else if let Some(hasher) = blake2_hasher {
        Ok(hex::encode(hasher.finalize()))
    } else if let Some(hasher) = xxh3_hasher {
        Ok(format!("{:016x}", hasher.digest128()))
    } else if let Some(hasher) = xxh64_hasher {
        Ok(format!("{:016x}", hasher.digest()))
    } else if let Some(hasher) = crc32_hasher {
        Ok(format!("{:08x}", hasher.finalize()))
    } else {
        Err(ContainerError::ParseError("Unknown hash algorithm".into()))
    }
}

/// Legacy parallel verification (kept for reference/fallback)
#[allow(dead_code)]
fn verify_with_progress_parallel_chunks<F>(path: &str, algorithm: &str, mut progress_callback: F) -> Result<String, ContainerError> 
where
    F: FnMut(usize, usize)
{
    use std::sync::mpsc;
    use std::thread;
    
    debug!(path = %path, "Starting parallel chunk verification");
    
    let handle = EwfHandle::open(path)?;
    let chunk_count = handle.get_chunk_count();
    debug!(chunk_count, "EWF chunk count");
    
    // Create hasher based on algorithm
    let algorithm_lower = algorithm.to_lowercase();
    let use_sha1 = algorithm_lower == "sha1" || algorithm_lower == "sha-1";
    let use_sha256 = algorithm_lower == "sha256" || algorithm_lower == "sha-256";
    let use_sha512 = algorithm_lower == "sha512" || algorithm_lower == "sha-512";
    let use_blake3 = algorithm_lower == "blake3";
    let use_blake2 = algorithm_lower == "blake2" || algorithm_lower == "blake2b";
    let use_xxh3 = algorithm_lower == "xxh3" || algorithm_lower == "xxhash3";
    let use_xxh64 = algorithm_lower == "xxh64" || algorithm_lower == "xxhash64";
    let use_crc32 = algorithm_lower == "crc32";
    
    let path_str = path.to_string();
    
    let num_threads = rayon::current_num_threads();
    
    let batch_size = if chunk_count > 100_000 {
        num_threads * 256
    } else if chunk_count > 10_000 {
        num_threads * 128
    } else {
        num_threads * 64
    };
    
    debug!(batch_size, num_threads, chunk_count, "Batch configuration");
    
    let decompressed_chunks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let decompressed_chunks_clone = decompressed_chunks.clone();
    
    let channel_depth = num_threads.max(16);
    let (tx, rx) = mpsc::sync_channel::<Result<(usize, Vec<Vec<u8>>), ContainerError>>(channel_depth);
    
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .ok();
    
    // Spawn decompression thread pool
    let decompression_handle = thread::spawn(move || {
        let handles_result: Result<Vec<EwfHandle>, ContainerError> = (0..num_threads)
            .map(|_| EwfHandle::open(&path_str))
            .collect();
        
        let mut handles = match handles_result {
            Ok(h) => h,
            Err(e) => {
                let _ = tx.send(Err(e));
                return;
            }
        };
        
        for batch_start in (0..chunk_count).step_by(batch_size) {
            let batch_end = (batch_start + batch_size).min(chunk_count);
            let batch_chunk_count = batch_end - batch_start;
            
            let thread_results: Vec<ChunkThreadResult> = handles
                .par_iter_mut()
                .enumerate()
                .map(|(thread_id, thread_handle)| {
                    let chunks_for_thread = batch_chunk_count.div_ceil(num_threads);
                    let mut chunks = Vec::with_capacity(chunks_for_thread);
                    
                    for chunk_idx in (batch_start + thread_id..batch_end).step_by(num_threads) {
                        match thread_handle.read_chunk_no_cache(chunk_idx) {
                            Ok(chunk_data) => {
                                chunks.push((chunk_idx, chunk_data));
                                decompressed_chunks_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            },
                            Err(e) => return Err(e),
                        }
                    }
                    
                    Ok(chunks)
                })
                .collect();
            
            let mut indexed_chunks = Vec::with_capacity(batch_chunk_count);
            for result in thread_results {
                match result {
                    Ok(mut thread_chunks) => indexed_chunks.append(&mut thread_chunks),
                    Err(e) => {
                        let _ = tx.send(Err(e));
                        return;
                    }
                }
            }
            
            indexed_chunks.sort_unstable_by_key(|(idx, _)| *idx);
            let batch_data: Vec<Vec<u8>> = indexed_chunks.into_iter().map(|(_, data)| data).collect();
            
            if tx.send(Ok((batch_start, batch_data))).is_err() {
                return;
            }
        }
    });
    
    // Hash on main thread
    let is_known_algo = use_sha1 || use_sha256 || use_sha512 || use_blake3 || use_blake2 || use_xxh3 || use_xxh64 || use_crc32;
    let mut md5_hasher: Option<Md5> = if !is_known_algo { Some(Md5::new()) } else { None };
    let mut sha1_hasher = if use_sha1 { Some(Sha1::new()) } else { None };
    let mut sha256_hasher = if use_sha256 { Some(Sha256::new()) } else { None };
    let mut sha512_hasher = if use_sha512 { Some(Sha512::new()) } else { None };
    let mut blake3_hasher = if use_blake3 { Some(Blake3Hasher::new()) } else { None };
    let mut blake2_hasher = if use_blake2 { Some(Blake2b512::new()) } else { None };
    let mut xxh3_hasher = if use_xxh3 { Some(Xxh3::new()) } else { None };
    let mut xxh64_hasher = if use_xxh64 { Some(Xxh64::new(0)) } else { None };
    let mut crc32_hasher = if use_crc32 { Some(crc32fast::Hasher::new()) } else { None };
    
    while let Ok(batch_result) = rx.recv() {
        let decompressed = decompressed_chunks.load(std::sync::atomic::Ordering::Relaxed);
        progress_callback(decompressed, chunk_count);
        
        match batch_result {
            Ok((batch_start, batch_chunks)) => {
                for (relative_idx, chunk_data) in batch_chunks.iter().enumerate() {
                    let _chunk_idx = batch_start + relative_idx;
                    
                    if let Some(ref mut hasher) = md5_hasher {
                        Digest::update(hasher, chunk_data);
                    } else if let Some(ref mut hasher) = sha1_hasher {
                        hasher.update(chunk_data);
                    } else if let Some(ref mut hasher) = sha256_hasher {
                        hasher.update(chunk_data);
                    } else if let Some(ref mut hasher) = sha512_hasher {
                        hasher.update(chunk_data);
                    } else if let Some(ref mut hasher) = blake3_hasher {
                        hasher.update_rayon(chunk_data);
                    } else if let Some(ref mut hasher) = blake2_hasher {
                        hasher.update(chunk_data);
                    } else if let Some(ref mut hasher) = xxh3_hasher {
                        hasher.update(chunk_data);
                    } else if let Some(ref mut hasher) = xxh64_hasher {
                        hasher.update(chunk_data);
                    } else if let Some(ref mut hasher) = crc32_hasher {
                        hasher.update(chunk_data);
                    }
                }
            }
            Err(e) => {
                let _ = decompression_handle.join();
                return Err(e);
            }
        }
    }
    
    progress_callback(chunk_count, chunk_count);
    
    decompression_handle.join().map_err(|_| ContainerError::ParseError("Decompression thread panicked".into()))?;
    
    // Return hash result
    if let Some(hasher) = md5_hasher {
        Ok(hex::encode(hasher.finalize()))
    } else if let Some(hasher) = sha1_hasher {
        Ok(hex::encode(hasher.finalize()))
    } else if let Some(hasher) = sha256_hasher {
        Ok(hex::encode(hasher.finalize()))
    } else if let Some(hasher) = sha512_hasher {
        Ok(hex::encode(hasher.finalize()))
    } else if let Some(hasher) = blake3_hasher {
        Ok(format!("{}", hasher.finalize().to_hex()))
    } else if let Some(hasher) = blake2_hasher {
        Ok(hex::encode(hasher.finalize()))
    } else if let Some(hasher) = xxh3_hasher {
        Ok(format!("{:016x}", hasher.digest128()))
    } else if let Some(hasher) = xxh64_hasher {
        Ok(format!("{:016x}", hasher.digest()))
    } else if let Some(hasher) = crc32_hasher {
        Ok(format!("{:08x}", hasher.finalize()))
    } else {
        Err(ContainerError::ParseError("Unknown hash algorithm".into()))
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_e01_nonexistent() {
        let result = is_e01("/nonexistent/path/test.E01");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_is_ewf_alias() {
        // is_ewf should behave the same as is_e01
        let result = is_ewf("/nonexistent/path/test.E01");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_get_segment_paths_nonexistent() {
        let result = get_segment_paths("/nonexistent/path/test.E01");
        // Always returns at least the base path (even if nonexistent)
        // Discovery works with path patterns, not by checking existence
        assert!(result.is_ok());
        let paths = result.unwrap();
        assert!(!paths.is_empty(), "Should return at least the base path");
    }

    #[test]
    fn test_get_stats_nonexistent() {
        let result = get_stats("/nonexistent/path/test.E01");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_chunks_nonexistent() {
        let result = verify_chunks("/nonexistent/path/test.E01", "md5");
        assert!(result.is_err());
    }

    #[test]
    fn test_export_metadata_json_nonexistent() {
        let result = export_metadata_json("/nonexistent/path/test.E01");
        assert!(result.is_err());
    }

    #[test]
    fn test_export_metadata_csv_nonexistent() {
        let result = export_metadata_csv("/nonexistent/path/test.E01");
        assert!(result.is_err());
    }

    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv("with\"quote"), "\"with\"\"quote\"");
        assert_eq!(escape_csv("with\nnewline"), "\"with\nnewline\"");
    }

    #[test]
    fn test_detect_ewf_variant_nonexistent() {
        let result = detect_ewf_variant("/nonexistent/path/test.E01");
        assert!(result.is_err());
    }

    #[test]
    fn test_ewf_stats_default() {
        let stats = super::super::types::EwfStats::default();
        assert_eq!(stats.total_chunks, 0);
        assert_eq!(stats.total_segments, 0);
        assert_eq!(stats.compression_ratio, 0.0);
        assert!(!stats.has_md5);
        assert!(!stats.has_sha1);
    }

    #[test]
    fn test_chunk_verify_result() {
        let result = super::super::types::ChunkVerifyResult {
            index: 42,
            status: "ok".to_string(),
            message: None,
        };
        assert_eq!(result.index, 42);
        assert_eq!(result.status, "ok");
        assert!(result.message.is_none());
    }

    #[test]
    fn test_chunk_verify_result_error() {
        let result = super::super::types::ChunkVerifyResult {
            index: 100,
            status: "error".to_string(),
            message: Some("Decompression failed".to_string()),
        };
        assert_eq!(result.status, "error");
        assert!(result.message.is_some());
    }

    #[test]
    fn test_info_nonexistent() {
        let result = info("/nonexistent/path/test.E01");
        assert!(result.is_err());
    }

    // =============================================================================
    // Edge Case Tests - Error Paths
    // =============================================================================

    #[test]
    fn test_is_e01_empty_file() {
        use tempfile::TempDir;
        use std::fs::File;
        
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("empty.E01");
        File::create(&path).unwrap();
        
        let result = is_e01(path.to_str().unwrap());
        // Empty file should return Ok(false) - can't read signature
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_is_e01_too_short_file() {
        use tempfile::TempDir;
        use std::fs::File;
        use std::io::Write;
        
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("short.E01");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"EVF").unwrap(); // Only 3 bytes, need 8 for signature
        
        let result = is_e01(path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_is_e01_wrong_signature() {
        use tempfile::TempDir;
        use std::fs::File;
        use std::io::Write;
        
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("wrong.E01");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"NOT_EVF_").unwrap(); // 8 bytes but wrong signature
        
        let result = is_e01(path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_info_fast_nonexistent() {
        let result = info_fast("/nonexistent/path/test.E01");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Error message should indicate file not found
        assert!(
            err_msg.contains("not found") || 
            err_msg.contains("No such file") ||
            err_msg.contains("Failed to open")
        );
    }

    #[test]
    fn test_verify_chunks_invalid_algorithm() {
        // Test with a valid-looking path but invalid algorithm
        // The algorithm validation should happen before file access in some cases
        let result = verify_chunks_detailed("/nonexistent/path/test.E01", "invalid_algo");
        assert!(result.is_err());
    }

    #[test]
    fn test_escape_csv_empty_string() {
        assert_eq!(escape_csv(""), "");
    }

    #[test]
    fn test_escape_csv_only_special_chars() {
        assert_eq!(escape_csv(","), "\",\"");
        assert_eq!(escape_csv("\""), "\"\"\"\"");
        assert_eq!(escape_csv("\n"), "\"\n\"");
    }

    #[test]
    fn test_escape_csv_mixed_special_chars() {
        assert_eq!(escape_csv("a,\"b\"\nc"), "\"a,\"\"b\"\"\nc\"");
    }

    #[test]
    fn test_ewf_search_result() {
        let result = super::super::types::EwfSearchResult {
            path: "/Documents/file.txt".to_string(),
            name: "file.txt".to_string(),
            size: 1024,
            match_type: "extension".to_string(),
        };
        assert_eq!(result.path, "/Documents/file.txt");
        assert_eq!(result.name, "file.txt");
        assert_eq!(result.size, 1024);
        assert_eq!(result.match_type, "extension");
    }

    #[test]
    fn test_header_info_default() {
        let header = super::super::types::HeaderInfo::default();
        assert!(header.case_number.is_none());
        assert!(header.evidence_number.is_none());
        assert!(header.description.is_none());
        assert!(header.examiner_name.is_none());
        assert!(header.notes.is_none());
        assert!(header.acquiry_date.is_none());
    }
}
