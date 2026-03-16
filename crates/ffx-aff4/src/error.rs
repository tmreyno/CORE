// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Error types for AFF4 operations.

use std::io;

/// All possible errors from AFF4 read/write operations.
#[derive(Debug, thiserror::Error)]
pub enum Aff4Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Invalid AFF4 container: {0}")]
    InvalidContainer(String),

    #[error("Missing required member: {0}")]
    MissingMember(String),

    #[error("Invalid RDF metadata: {0}")]
    InvalidRdf(String),

    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    #[error("Unsupported compression: {0}")]
    UnsupportedCompression(String),

    #[error("Hash verification failed for {stream}: expected {expected}, got {actual}")]
    HashMismatch {
        stream: String,
        expected: String,
        actual: String,
    },

    #[error("Stream not found: {0}")]
    StreamNotFound(String),

    #[error("Invalid bevy index at offset {offset}: {reason}")]
    InvalidBevyIndex { offset: u64, reason: String },

    #[error("Invalid map entry: {0}")]
    InvalidMapEntry(String),

    #[error("No output path specified")]
    NoOutputPath,

    #[error("No source data provided")]
    NoSource,

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Write error: {0}")]
    WriteError(String),

    #[error("Decompression error for chunk {chunk_index} in bevy {bevy_index}: {reason}")]
    DecompressionError {
        bevy_index: u32,
        chunk_index: u32,
        reason: String,
    },

    #[error("Compression error: {reason}")]
    CompressionError { reason: String },

    #[error("Seek out of range: offset {offset} exceeds stream size {size}")]
    SeekOutOfRange { offset: u64, size: u64 },
}

/// Convenience result type for AFF4 operations.
pub type Aff4Result<T> = Result<T, Aff4Error>;
