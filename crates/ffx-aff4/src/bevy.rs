// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Bevy read/write for AFF4 containers.
//!
//! A bevy is a contiguous sequence of compressed chunks that forms one segment
//! of an AFF4 image stream. Each bevy has:
//! - A **data file** (ZIP member): concatenated compressed chunks
//! - An **index file** (ZIP member): packed array of `BevyIndexEntry`,
//!   each 12 bytes (u64 offset + u32 length), little-endian
//! - Optional **block hash files** (ZIP member per algorithm): concatenated
//!   raw hash digests, one per chunk

use crate::compression::{compress_chunk, decompress_chunk};
use crate::error::{Aff4Error, Aff4Result};
use crate::hashing::hash_chunk;
use crate::types::{Aff4Compression, Aff4HashAlgorithm, BEVY_INDEX_ENTRY_SIZE};

// ─── Bevy Index Entry ────────────────────────────────────────────────────────

/// A single entry in a bevy index file.
///
/// Layout (12 bytes, little-endian):
/// ```text
/// offset:  u64  — byte offset of this chunk within the bevy data file
/// length:  u32  — compressed size of this chunk in bytes
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BevyIndexEntry {
    /// Byte offset within the bevy data file.
    pub offset: u64,
    /// Compressed length in bytes.
    pub length: u32,
}

impl BevyIndexEntry {
    /// Serialize to 12 bytes (little-endian).
    pub fn to_bytes(&self) -> [u8; BEVY_INDEX_ENTRY_SIZE] {
        let mut buf = [0u8; BEVY_INDEX_ENTRY_SIZE];
        buf[..8].copy_from_slice(&self.offset.to_le_bytes());
        buf[8..12].copy_from_slice(&self.length.to_le_bytes());
        buf
    }

    /// Deserialize from 12 bytes (little-endian).
    pub fn from_bytes(data: &[u8; BEVY_INDEX_ENTRY_SIZE]) -> Self {
        Self {
            offset: u64::from_le_bytes(data[..8].try_into().unwrap()),
            length: u32::from_le_bytes(data[8..12].try_into().unwrap()),
        }
    }
}

// ─── Bevy Writer ─────────────────────────────────────────────────────────────

/// Accumulates compressed chunks for a single bevy and produces the data,
/// index, and block-hash byte buffers.
pub struct BevyWriter {
    /// Compression algorithm.
    compression: Aff4Compression,
    /// Chunk size for compression threshold.
    chunk_size: u32,
    /// Block hash algorithms to compute per chunk.
    block_hash_algorithms: Vec<Aff4HashAlgorithm>,

    /// Accumulated compressed chunk data (the bevy data file).
    data: Vec<u8>,
    /// Index entries, one per chunk.
    index: Vec<BevyIndexEntry>,
    /// Per-algorithm block hashes: algorithm index → concatenated raw digests.
    block_hashes: Vec<Vec<u8>>,
}

impl BevyWriter {
    /// Create a new bevy writer.
    pub fn new(
        compression: Aff4Compression,
        chunk_size: u32,
        block_hash_algorithms: &[Aff4HashAlgorithm],
    ) -> Self {
        Self {
            compression,
            chunk_size,
            block_hash_algorithms: block_hash_algorithms.to_vec(),
            data: Vec::new(),
            index: Vec::new(),
            block_hashes: vec![Vec::new(); block_hash_algorithms.len()],
        }
    }

    /// Add a chunk of raw (uncompressed) data to this bevy.
    ///
    /// The chunk is compressed, indexed, and block-hashed.
    pub fn add_chunk(&mut self, raw_chunk: &[u8]) -> Aff4Result<()> {
        let offset = self.data.len() as u64;

        // Compress
        let (compressed, _is_stored) =
            compress_chunk(raw_chunk, self.compression, self.chunk_size)?;

        let compressed_len =
            u32::try_from(compressed.len()).map_err(|_| Aff4Error::InvalidBevyIndex {
                offset,
                reason: format!(
                    "Compressed chunk length {} exceeds AFF4 bevy index u32 length",
                    compressed.len()
                ),
            })?;

        // Index entry
        self.index.push(BevyIndexEntry {
            offset,
            length: compressed_len,
        });

        // Block hashes — hash the raw (uncompressed) data
        for (i, alg) in self.block_hash_algorithms.iter().enumerate() {
            let digest = hash_chunk(raw_chunk, *alg);
            self.block_hashes[i].extend_from_slice(&digest);
        }

        // Append compressed data
        self.data.extend_from_slice(&compressed);

        Ok(())
    }

    /// Number of chunks stored.
    pub fn chunk_count(&self) -> usize {
        self.index.len()
    }

    /// Consume the writer and return the bevy data, index, and block hashes.
    pub fn finish(self) -> BevyWriteResult {
        // Serialize index
        let mut index_bytes = Vec::with_capacity(self.index.len() * BEVY_INDEX_ENTRY_SIZE);
        for entry in &self.index {
            index_bytes.extend_from_slice(&entry.to_bytes());
        }

        // Collect block hashes as (algorithm, raw bytes)
        let block_hashes: Vec<(Aff4HashAlgorithm, Vec<u8>)> = self
            .block_hash_algorithms
            .into_iter()
            .zip(self.block_hashes)
            .collect();

        BevyWriteResult {
            data: self.data,
            index: index_bytes,
            block_hashes,
        }
    }
}

/// Result of finishing a bevy write.
pub struct BevyWriteResult {
    /// Compressed chunk data (the bevy data file content).
    pub data: Vec<u8>,
    /// Serialized index entries (12 bytes each).
    pub index: Vec<u8>,
    /// Block hash data per algorithm: (algorithm, concatenated raw digests).
    pub block_hashes: Vec<(Aff4HashAlgorithm, Vec<u8>)>,
}

// ─── Bevy Reader ─────────────────────────────────────────────────────────────

/// Parsed bevy index for reading chunks from a bevy data buffer.
pub struct BevyReader {
    /// Parsed index entries.
    entries: Vec<BevyIndexEntry>,
    /// Compression algorithm used.
    compression: Aff4Compression,
    /// Expected decompressed chunk size.
    chunk_size: u32,
}

impl BevyReader {
    /// Parse a bevy index from raw bytes.
    pub fn from_index(
        index_data: &[u8],
        compression: Aff4Compression,
        chunk_size: u32,
    ) -> Aff4Result<Self> {
        if !index_data.len().is_multiple_of(BEVY_INDEX_ENTRY_SIZE) {
            return Err(Aff4Error::InvalidBevyIndex {
                offset: 0,
                reason: format!(
                    "Index size {} is not a multiple of {}",
                    index_data.len(),
                    BEVY_INDEX_ENTRY_SIZE
                ),
            });
        }

        let count = index_data.len() / BEVY_INDEX_ENTRY_SIZE;
        let mut entries = Vec::with_capacity(count);

        for i in 0..count {
            let start = i * BEVY_INDEX_ENTRY_SIZE;
            let chunk: [u8; BEVY_INDEX_ENTRY_SIZE] = index_data
                [start..start + BEVY_INDEX_ENTRY_SIZE]
                .try_into()
                .unwrap();
            entries.push(BevyIndexEntry::from_bytes(&chunk));
        }

        Ok(Self {
            entries,
            compression,
            chunk_size,
        })
    }

    /// Number of chunks in this bevy.
    pub fn chunk_count(&self) -> usize {
        self.entries.len()
    }

    /// Read and decompress a single chunk from the bevy data buffer.
    pub fn read_chunk(&self, chunk_index: usize, bevy_data: &[u8]) -> Aff4Result<Vec<u8>> {
        let entry = self
            .entries
            .get(chunk_index)
            .ok_or_else(|| Aff4Error::InvalidBevyIndex {
                offset: chunk_index as u64,
                reason: format!(
                    "Chunk index {} out of range (bevy has {} chunks)",
                    chunk_index,
                    self.entries.len()
                ),
            })?;

        let (start, end) = checked_bevy_data_range(entry, bevy_data.len())?;
        let compressed = &bevy_data[start..end];

        // If the compressed length equals chunk_size, it was stored uncompressed
        let is_stored = entry.length == self.chunk_size;

        decompress_chunk(compressed, self.compression, is_stored)
    }

    /// Read and decompress all chunks from the bevy data buffer.
    pub fn read_all_chunks(&self, bevy_data: &[u8]) -> Aff4Result<Vec<Vec<u8>>> {
        let mut chunks = Vec::with_capacity(self.entries.len());
        for i in 0..self.entries.len() {
            chunks.push(self.read_chunk(i, bevy_data)?);
        }
        Ok(chunks)
    }

    /// Verify block hashes for all chunks in this bevy.
    pub fn verify_block_hashes(
        &self,
        bevy_data: &[u8],
        algorithm: Aff4HashAlgorithm,
        expected_hashes: &[u8],
    ) -> Aff4Result<Vec<bool>> {
        let digest_size = algorithm.digest_size();
        let expected_count = expected_hashes.len() / digest_size;

        if expected_count != self.entries.len() {
            return Err(Aff4Error::InvalidBevyIndex {
                offset: 0,
                reason: format!(
                    "Block hash count {} does not match chunk count {}",
                    expected_count,
                    self.entries.len()
                ),
            });
        }

        let mut results = Vec::with_capacity(self.entries.len());

        for i in 0..self.entries.len() {
            let chunk = self.read_chunk(i, bevy_data)?;
            let actual = hash_chunk(&chunk, algorithm);
            let expected = &expected_hashes[i * digest_size..(i + 1) * digest_size];
            results.push(actual == expected);
        }

        Ok(results)
    }
}

fn checked_bevy_data_range(
    entry: &BevyIndexEntry,
    bevy_data_len: usize,
) -> Aff4Result<(usize, usize)> {
    let start = usize::try_from(entry.offset).map_err(|_| Aff4Error::InvalidBevyIndex {
        offset: entry.offset,
        reason: "Chunk offset exceeds addressable memory size".to_string(),
    })?;
    let length = usize::try_from(entry.length).map_err(|_| Aff4Error::InvalidBevyIndex {
        offset: entry.offset,
        reason: "Chunk length exceeds addressable memory size".to_string(),
    })?;
    let end = start
        .checked_add(length)
        .ok_or_else(|| Aff4Error::InvalidBevyIndex {
            offset: entry.offset,
            reason: format!("Chunk data range starts at {start} and overflows usize"),
        })?;

    if end > bevy_data_len {
        return Err(Aff4Error::InvalidBevyIndex {
            offset: entry.offset,
            reason: format!(
                "Chunk data range {}..{} exceeds bevy data size {}",
                start, end, bevy_data_len
            ),
        });
    }

    Ok((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_entry_roundtrip() {
        let entry = BevyIndexEntry {
            offset: 0x1234567890ABCDEF,
            length: 0xDEADBEEF,
        };
        let bytes = entry.to_bytes();
        let parsed = BevyIndexEntry::from_bytes(&bytes);
        assert_eq!(entry, parsed);
    }

    #[test]
    fn test_checked_bevy_data_range_accepts_valid_range() {
        let entry = BevyIndexEntry {
            offset: 2,
            length: 4,
        };

        assert_eq!(checked_bevy_data_range(&entry, 8).unwrap(), (2, 6));
    }

    #[test]
    fn test_checked_bevy_data_range_rejects_overflow() {
        let entry = BevyIndexEntry {
            offset: usize::MAX as u64,
            length: 1,
        };

        let err = checked_bevy_data_range(&entry, usize::MAX)
            .expect_err("overflowing bevy range should fail");

        assert!(
            matches!(err, Aff4Error::InvalidBevyIndex { reason, .. } if reason.contains("overflows"))
        );
    }

    #[test]
    fn test_checked_bevy_data_range_rejects_past_data_end() {
        let entry = BevyIndexEntry {
            offset: 8,
            length: 4,
        };

        let err =
            checked_bevy_data_range(&entry, 10).expect_err("bevy range past data end should fail");

        assert!(
            matches!(err, Aff4Error::InvalidBevyIndex { reason, .. } if reason.contains("exceeds bevy data size"))
        );
    }

    #[test]
    fn test_bevy_write_read_stored() {
        let mut writer = BevyWriter::new(Aff4Compression::Stored, 32768, &[]);
        let chunk1 = vec![0xAA; 1024];
        let chunk2 = vec![0xBB; 512];

        writer.add_chunk(&chunk1).unwrap();
        writer.add_chunk(&chunk2).unwrap();

        assert_eq!(writer.chunk_count(), 2);

        let result = writer.finish();

        // Read back
        let reader = BevyReader::from_index(&result.index, Aff4Compression::Stored, 32768).unwrap();
        assert_eq!(reader.chunk_count(), 2);

        let c1 = reader.read_chunk(0, &result.data).unwrap();
        let c2 = reader.read_chunk(1, &result.data).unwrap();

        assert_eq!(c1, chunk1);
        assert_eq!(c2, chunk2);
    }

    #[test]
    fn test_bevy_write_read_deflate() {
        let mut writer = BevyWriter::new(Aff4Compression::Deflate, 32768, &[]);
        let chunk = vec![0x42; 8192]; // compressible data

        writer.add_chunk(&chunk).unwrap();
        let result = writer.finish();

        // Data should be compressed (smaller than original)
        assert!(result.data.len() < chunk.len());

        let reader =
            BevyReader::from_index(&result.index, Aff4Compression::Deflate, 32768).unwrap();
        let decompressed = reader.read_chunk(0, &result.data).unwrap();
        assert_eq!(decompressed, chunk);
    }

    #[test]
    fn test_bevy_block_hashes() {
        let alg = Aff4HashAlgorithm::Sha256;
        let mut writer = BevyWriter::new(Aff4Compression::Stored, 32768, &[alg]);

        let chunk1 = vec![0x11; 1024];
        let chunk2 = vec![0x22; 1024];
        writer.add_chunk(&chunk1).unwrap();
        writer.add_chunk(&chunk2).unwrap();

        let result = writer.finish();

        // Block hashes should have 2 * 32 = 64 bytes of SHA-256 digests
        assert_eq!(result.block_hashes.len(), 1);
        assert_eq!(result.block_hashes[0].0, alg);
        assert_eq!(result.block_hashes[0].1.len(), 64); // 2 chunks × 32 bytes

        // Verify block hashes
        let reader = BevyReader::from_index(&result.index, Aff4Compression::Stored, 32768).unwrap();
        let verified = reader
            .verify_block_hashes(&result.data, alg, &result.block_hashes[0].1)
            .unwrap();
        assert!(verified.iter().all(|&v| v));
    }

    #[test]
    fn test_bevy_read_all() {
        let mut writer = BevyWriter::new(Aff4Compression::Lz4, 32768, &[]);

        for i in 0..5u8 {
            let chunk = vec![i; 4096];
            writer.add_chunk(&chunk).unwrap();
        }

        let result = writer.finish();
        let reader = BevyReader::from_index(&result.index, Aff4Compression::Lz4, 32768).unwrap();
        let all = reader.read_all_chunks(&result.data).unwrap();

        assert_eq!(all.len(), 5);
        for (i, chunk) in all.iter().enumerate() {
            assert_eq!(chunk.len(), 4096);
            assert!(chunk.iter().all(|&b| b == i as u8));
        }
    }
}
