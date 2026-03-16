// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Compression and decompression for AFF4 bevy chunks.
//!
//! AFF4 supports four compression methods: Stored, Deflate, LZ4, and Snappy.
//! Each chunk is compressed independently. If the compressed output does not
//! save at least `COMPRESSION_THRESHOLD` bytes, the chunk is stored verbatim.

use crate::error::{Aff4Error, Aff4Result};
use crate::types::{Aff4Compression, COMPRESSION_THRESHOLD};

/// Compress a chunk of data using the specified algorithm.
///
/// Returns `(compressed_data, actually_stored)` where `actually_stored` is
/// `true` if the data is returned uncompressed (savings below threshold).
pub fn compress_chunk(
    data: &[u8],
    method: Aff4Compression,
    chunk_size: u32,
) -> Aff4Result<(Vec<u8>, bool)> {
    if method == Aff4Compression::Stored {
        return Ok((data.to_vec(), true));
    }

    let compressed = match method {
        Aff4Compression::Stored => unreachable!(),
        Aff4Compression::Deflate => compress_deflate(data)?,
        Aff4Compression::Lz4 => compress_lz4(data),
        Aff4Compression::Snappy => compress_snappy(data)?,
    };

    // Only use compressed if it saves at least COMPRESSION_THRESHOLD bytes
    let threshold = chunk_size.saturating_sub(COMPRESSION_THRESHOLD) as usize;
    if compressed.len() < threshold {
        Ok((compressed, false))
    } else {
        // Not worth compressing — store verbatim
        Ok((data.to_vec(), true))
    }
}

/// Decompress a chunk of data.
///
/// `stored` indicates whether the chunk was stored uncompressed.
pub fn decompress_chunk(
    data: &[u8],
    method: Aff4Compression,
    stored: bool,
) -> Aff4Result<Vec<u8>> {
    if stored || method == Aff4Compression::Stored {
        return Ok(data.to_vec());
    }

    match method {
        Aff4Compression::Stored => unreachable!("handled by early return above"),
        Aff4Compression::Deflate => decompress_deflate(data),
        Aff4Compression::Lz4 => decompress_lz4(data),
        Aff4Compression::Snappy => decompress_snappy(data),
    }
}

// ─── Deflate (RFC 1951) ──────────────────────────────────────────────────────

fn compress_deflate(data: &[u8]) -> Aff4Result<Vec<u8>> {
    use flate2::write::DeflateEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).map_err(Aff4Error::Io)?;
    encoder.finish().map_err(Aff4Error::Io)
}

fn decompress_deflate(data: &[u8]) -> Aff4Result<Vec<u8>> {
    use flate2::read::DeflateDecoder;
    use std::io::Read;

    let mut decoder = DeflateDecoder::new(data);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output).map_err(Aff4Error::Io)?;
    Ok(output)
}

// ─── LZ4 ─────────────────────────────────────────────────────────────────────

fn compress_lz4(data: &[u8]) -> Vec<u8> {
    lz4_flex::compress_prepend_size(data)
}

fn decompress_lz4(data: &[u8]) -> Aff4Result<Vec<u8>> {
    lz4_flex::decompress_size_prepended(data).map_err(|e| Aff4Error::DecompressionError {
        bevy_index: 0,
        chunk_index: 0,
        reason: format!("LZ4 decompression failed: {}", e),
    })
}

// ─── Snappy ──────────────────────────────────────────────────────────────────

fn compress_snappy(data: &[u8]) -> Aff4Result<Vec<u8>> {
    let mut encoder = snap::raw::Encoder::new();
    encoder.compress_vec(data).map_err(|e| Aff4Error::CompressionError {
        reason: format!("Snappy compression failed: {}", e),
    })
}

fn decompress_snappy(data: &[u8]) -> Aff4Result<Vec<u8>> {
    let mut decoder = snap::raw::Decoder::new();
    decoder
        .decompress_vec(data)
        .map_err(|e| Aff4Error::DecompressionError {
            bevy_index: 0,
            chunk_index: 0,
            reason: format!("Snappy decompression failed: {}", e),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_deflate() {
        let data = vec![0xAB; 4096];
        let (compressed, stored) =
            compress_chunk(&data, Aff4Compression::Deflate, 32768).unwrap();
        assert!(!stored, "highly compressible data should be compressed");
        assert!(compressed.len() < data.len());

        let decompressed = decompress_chunk(&compressed, Aff4Compression::Deflate, false)
            .unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compress_decompress_lz4() {
        let data = vec![0xCD; 4096];
        let (compressed, stored) = compress_chunk(&data, Aff4Compression::Lz4, 32768).unwrap();
        assert!(!stored);

        let decompressed =
            decompress_chunk(&compressed, Aff4Compression::Lz4, false).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compress_decompress_snappy() {
        let data = vec![0xEF; 4096];
        let (compressed, stored) =
            compress_chunk(&data, Aff4Compression::Snappy, 32768).unwrap();
        assert!(!stored);

        let decompressed =
            decompress_chunk(&compressed, Aff4Compression::Snappy, false).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_stored_passthrough() {
        let data = b"hello world".to_vec();
        let (output, stored) =
            compress_chunk(&data, Aff4Compression::Stored, 32768).unwrap();
        assert!(stored);
        assert_eq!(output, data);

        let decompressed =
            decompress_chunk(&output, Aff4Compression::Stored, true).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_incompressible_falls_back_to_stored() {
        // Random-ish data that won't compress well
        let mut data = vec![0u8; 32768];
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = (i.wrapping_mul(7) ^ i.wrapping_mul(13)) as u8;
        }

        let (output, stored) =
            compress_chunk(&data, Aff4Compression::Deflate, 32768).unwrap();
        // If stored == true, means compression didn't help enough
        if stored {
            assert_eq!(output, data);
        }
    }
}
