// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Chunk compression for L01 file data.
//!
//! File data in L01 is stored as a series of compressed chunks in the
//! "sectors" section. Each chunk is a fixed-size block (default 32KB)
//! that is zlib-compressed. The "table" section maps chunk offsets.
//!
//! Compression uses flate2 (zlib-rs backend) for compatibility with
//! EnCase's expected zlib format.

use flate2::write::ZlibEncoder;
use flate2::Compression;
use md5::Digest as _;
use std::io::Write;

use super::types::{ChunkTable, CompressionLevel, L01WriteError};

/// Default chunk size: 32KB (64 sectors × 512 bytes)
pub const DEFAULT_CHUNK_SIZE: usize = 32768;

/// Compress file data into chunks suitable for the EWF sectors section.
///
/// Returns:
/// - The concatenated compressed chunk data (to be written into sectors section)
/// - A `ChunkTable` with offsets for each chunk
///
/// # Arguments
/// - `data` — The raw file data to compress
/// - `chunk_size` — Size of each uncompressed chunk (default 32KB)
/// - `compression_level` — Compression level (None, Fast, Best)
/// - `base_offset` — Offset in the output file where sectors data begins
pub fn compress_file_data(
    data: &[u8],
    chunk_size: usize,
    compression_level: CompressionLevel,
    base_offset: u64,
) -> Result<(Vec<u8>, ChunkTable), L01WriteError> {
    let mut compressed_data = Vec::new();
    let mut table = ChunkTable::new(base_offset);
    let chunk_size = if chunk_size == 0 {
        DEFAULT_CHUNK_SIZE
    } else {
        chunk_size
    };

    let mut offset: u64 = 0;
    let mut pos = 0;

    while pos < data.len() {
        let end = std::cmp::min(pos + chunk_size, data.len());
        let chunk = &data[pos..end];

        let (chunk_bytes, is_compressed) = compress_chunk(chunk, compression_level)?;

        let compressed_size = chunk_bytes.len() as u32;
        table.add_chunk(offset, compressed_size, is_compressed);

        compressed_data.extend_from_slice(&chunk_bytes);
        offset += compressed_size as u64;
        pos = end;
    }

    Ok((compressed_data, table))
}

/// Compress a single chunk of data.
///
/// If compression doesn't reduce size, stores the chunk uncompressed.
/// Returns (compressed_bytes, is_compressed).
fn compress_chunk(data: &[u8], level: CompressionLevel) -> Result<(Vec<u8>, bool), L01WriteError> {
    if level == CompressionLevel::None {
        return Ok((data.to_vec(), false));
    }

    let flate_level = match level {
        CompressionLevel::None => unreachable!(),
        CompressionLevel::Fast => Compression::fast(),
        CompressionLevel::Best => Compression::best(),
    };

    let mut encoder = ZlibEncoder::new(Vec::new(), flate_level);
    encoder
        .write_all(data)
        .map_err(|e| L01WriteError::CompressionError(e.to_string()))?;

    let compressed = encoder
        .finish()
        .map_err(|e| L01WriteError::CompressionError(e.to_string()))?;

    // Only use compressed version if it's actually smaller
    if compressed.len() < data.len() {
        Ok((compressed, true))
    } else {
        Ok((data.to_vec(), false))
    }
}

/// Compress data from a reader in streaming fashion.
///
/// This is more memory-efficient for large files — reads and compresses
/// chunk by chunk without loading the entire file into memory.
///
/// # Arguments
/// - `reader` — Source data reader
/// - `total_size` — Total bytes to read (used for buffer pre-allocation)
/// - `chunk_size` — Size of each uncompressed chunk
/// - `compression_level` — Compression level
/// - `base_offset` — Offset in the output file where sectors data begins
/// - `progress_fn` — Optional callback for progress reporting (bytes_processed)
pub fn compress_from_reader<R: std::io::Read>(
    reader: &mut R,
    total_size: u64,
    chunk_size: usize,
    compression_level: CompressionLevel,
    base_offset: u64,
    progress_fn: Option<&mut dyn FnMut(u64)>,
) -> Result<(Vec<u8>, ChunkTable), L01WriteError> {
    let (compressed_data, table, _hashes) = compress_and_hash_from_reader(
        reader,
        total_size,
        chunk_size,
        compression_level,
        base_offset,
        progress_fn,
        None,
    )?;
    Ok((compressed_data, table))
}

/// Hashers that can be fed data inline during compression.
///
/// Contains per-file hashers (MD5 + SHA1) and image-level hashers.
/// All hashers are updated with each chunk of raw data during the
/// compression pass, eliminating the need for separate file reads.
pub struct InlineHashers<'a> {
    pub file_md5: &'a mut md5::Md5,
    pub file_sha1: &'a mut sha1::Sha1,
    pub image_md5: &'a mut md5::Md5,
    pub image_sha1: &'a mut sha1::Sha1,
}

/// Result of inline hashing during compression.
pub struct InlineHashResult {
    pub file_md5: [u8; 16],
    pub file_sha1: [u8; 20],
}

/// Compress data from a reader while simultaneously computing hashes.
///
/// When `hashers` is provided, updates all four hashers (per-file MD5/SHA1
/// and image-level MD5/SHA1) with raw data during the compression pass.
/// This eliminates the need to re-read each source file for hashing,
/// reducing I/O from 3 reads per file down to 1.
pub fn compress_and_hash_from_reader<R: std::io::Read>(
    reader: &mut R,
    total_size: u64,
    chunk_size: usize,
    compression_level: CompressionLevel,
    base_offset: u64,
    mut progress_fn: Option<&mut dyn FnMut(u64)>,
    mut hashers: Option<InlineHashers<'_>>,
) -> Result<(Vec<u8>, ChunkTable, Option<InlineHashResult>), L01WriteError> {
    let chunk_size = if chunk_size == 0 {
        DEFAULT_CHUNK_SIZE
    } else {
        chunk_size
    };

    let estimated_chunks = (total_size as usize).div_ceil(chunk_size);
    let mut compressed_data = Vec::with_capacity(total_size as usize);
    let mut table = ChunkTable::new(base_offset);

    let mut buf = vec![0u8; chunk_size];
    let mut offset: u64 = 0;
    let mut bytes_processed: u64 = 0;

    loop {
        let mut filled = 0;
        // Read exactly chunk_size bytes (or until EOF)
        while filled < chunk_size {
            match reader.read(&mut buf[filled..chunk_size]) {
                Ok(0) => break, // EOF
                Ok(n) => filled += n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(L01WriteError::Io(e)),
            }
        }

        if filled == 0 {
            break; // No more data
        }

        let chunk = &buf[..filled];

        // Update hashers inline with raw data before compression
        if let Some(ref mut h) = hashers {
            h.file_md5.update(chunk);
            h.file_sha1.update(chunk);
            h.image_md5.update(chunk);
            h.image_sha1.update(chunk);
        }

        let (chunk_bytes, is_compressed) = compress_chunk(chunk, compression_level)?;

        let compressed_size = chunk_bytes.len() as u32;
        table.add_chunk(offset, compressed_size, is_compressed);

        compressed_data.extend_from_slice(&chunk_bytes);
        offset += compressed_size as u64;
        bytes_processed += filled as u64;

        if let Some(ref mut f) = progress_fn {
            f(bytes_processed);
        }
    }

    // Pre-size hint was helpful
    let _ = estimated_chunks;

    // Finalize per-file hashes if hashers were provided
    let hash_result = hashers.map(|h| {
        let file_md5_result = h.file_md5.clone().finalize();
        let file_sha1_result = h.file_sha1.clone().finalize();

        let mut md5_bytes = [0u8; 16];
        md5_bytes.copy_from_slice(&file_md5_result);

        let mut sha1_bytes = [0u8; 20];
        sha1_bytes.copy_from_slice(&file_sha1_result);

        InlineHashResult {
            file_md5: md5_bytes,
            file_sha1: sha1_bytes,
        }
    });

    Ok((compressed_data, table, hash_result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_empty_data() {
        let (compressed, table) =
            compress_file_data(b"", DEFAULT_CHUNK_SIZE, CompressionLevel::Fast, 0).unwrap();
        assert!(compressed.is_empty());
        assert_eq!(table.chunk_count(), 0);
    }

    #[test]
    fn test_compress_small_data() {
        let data = b"Hello, forensic world!";
        let (compressed, table) =
            compress_file_data(data, DEFAULT_CHUNK_SIZE, CompressionLevel::Fast, 0).unwrap();

        assert_eq!(table.chunk_count(), 1);
        // Small data might not compress well
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_compress_no_compression() {
        let data = vec![0xAB; 1000];
        let (compressed, table) =
            compress_file_data(&data, DEFAULT_CHUNK_SIZE, CompressionLevel::None, 0).unwrap();

        assert_eq!(table.chunk_count(), 1);
        assert!(!table.chunks[0].is_compressed);
        assert_eq!(compressed.len(), 1000);
        assert_eq!(compressed, data);
    }

    #[test]
    fn test_compress_multiple_chunks() {
        // Create data larger than one chunk
        let data = vec![0x42; DEFAULT_CHUNK_SIZE * 3 + 100];
        let (compressed, table) =
            compress_file_data(&data, DEFAULT_CHUNK_SIZE, CompressionLevel::Fast, 0).unwrap();

        assert_eq!(table.chunk_count(), 4); // 3 full + 1 partial
        assert!(!compressed.is_empty());

        // Highly repetitive data should compress well
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_compress_with_base_offset() {
        let data = vec![0xFF; 100];
        let base = 4096u64;
        let (_, table) =
            compress_file_data(&data, DEFAULT_CHUNK_SIZE, CompressionLevel::Fast, base).unwrap();

        assert_eq!(table.base_offset, base);
    }

    #[test]
    fn test_compress_best_vs_fast() {
        let data: Vec<u8> = (0..DEFAULT_CHUNK_SIZE * 2)
            .map(|i| (i % 256) as u8)
            .collect();

        let (fast_compressed, _) =
            compress_file_data(&data, DEFAULT_CHUNK_SIZE, CompressionLevel::Fast, 0).unwrap();
        let (best_compressed, _) =
            compress_file_data(&data, DEFAULT_CHUNK_SIZE, CompressionLevel::Best, 0).unwrap();

        // Best compression should be <= fast compression size
        assert!(best_compressed.len() <= fast_compressed.len());
    }

    #[test]
    fn test_compress_from_reader() {
        let data = vec![0x42; DEFAULT_CHUNK_SIZE * 2 + 500];
        let mut cursor = std::io::Cursor::new(&data);

        let (compressed, table) = compress_from_reader(
            &mut cursor,
            data.len() as u64,
            DEFAULT_CHUNK_SIZE,
            CompressionLevel::Fast,
            0,
            None,
        )
        .unwrap();

        assert_eq!(table.chunk_count(), 3);
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_compress_from_reader_with_progress() {
        let data = vec![0x42; DEFAULT_CHUNK_SIZE * 2];
        let mut cursor = std::io::Cursor::new(&data);
        let mut last_progress: u64 = 0;

        let (_, table) = compress_from_reader(
            &mut cursor,
            data.len() as u64,
            DEFAULT_CHUNK_SIZE,
            CompressionLevel::Fast,
            0,
            Some(&mut |bytes| {
                last_progress = bytes;
            }),
        )
        .unwrap();

        assert_eq!(table.chunk_count(), 2);
        assert_eq!(last_progress, data.len() as u64);
    }

    #[test]
    fn test_incompressible_data_stored_raw() {
        // Random-looking data that won't compress well
        let data: Vec<u8> = (0..1000)
            .map(|i| {
                let x = (i as u64)
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                (x >> 33) as u8
            })
            .collect();

        let (compressed, table) =
            compress_file_data(&data, DEFAULT_CHUNK_SIZE, CompressionLevel::Fast, 0).unwrap();

        // If compression didn't help, it should store raw
        if !table.chunks[0].is_compressed {
            assert_eq!(compressed.len(), data.len());
        }
    }
}
