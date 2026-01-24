// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Cryptographic Hash Utilities for Forensic Verification
//!
//! This module provides unified hashing functionality across all container formats
//! (AD1, E01, RAW, L01) with support for both forensic-standard and high-performance
//! hash algorithms.
//!
//! # Supported Algorithms
//!
//! | Algorithm | Output Size | Use Case | Performance |
//! |-----------|-------------|----------|-------------|
//! | **MD5** | 128-bit | Legacy verification, AD1 metadata | Fast |
//! | **SHA-1** | 160-bit | Legacy verification, E01 digests | Fast |
//! | **SHA-256** | 256-bit | Court-accepted forensic standard | Moderate |
//! | **SHA-512** | 512-bit | High-security forensic verification | Slower |
//! | **BLAKE3** | 256-bit | Modern, parallel-optimized hash | Very Fast |
//! | **BLAKE2b** | 512-bit | Modern cryptographic hash | Fast |
//! | **XXH3** | 128-bit | Integrity checks (non-crypto) | Ultra Fast |
//! | **XXH64** | 64-bit | Quick checksums (non-crypto) | Ultra Fast |
//! | **CRC32** | 32-bit | Simple checksums | Ultra Fast |
//!
//! # Forensic Considerations
//!
//! - **Court Admissibility**: SHA-256 and SHA-1 are most widely accepted
//! - **Legacy Compatibility**: MD5/SHA-1 for older tool interoperability
//! - **Performance**: BLAKE3/XXH3 for large dataset integrity verification
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::common::hash::{HashAlgorithm, StreamingHasher, compute_hash};
//!
//! // Compute hash of a byte slice
//! let data = b"evidence data";
//! let hash = compute_hash(data, HashAlgorithm::Sha256);
//! assert_eq!(hash.len(), 64); // 256 bits = 64 hex chars
//!
//! // Streaming hash for large files
//! let mut hasher = StreamingHasher::new(HashAlgorithm::Blake3);
//! hasher.update(b"chunk 1");
//! hasher.update(b"chunk 2");
//! let final_hash = hasher.finalize();
//! ```

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;
use md5::Md5;
use sha1::{Sha1, Digest};
use sha2::{Sha256, Sha512};
use blake2::Blake2b512;
use blake3::Hasher as Blake3Hasher;
use xxhash_rust::xxh3::Xxh3;
use xxhash_rust::xxh64::Xxh64;
use crc32fast::Hasher as Crc32Hasher;
use serde::Serialize;
use tracing::{debug, trace, instrument};
use crate::containers::ContainerError;

use super::{AdaptiveBuffer, IoOperation};

// =============================================================================
// Hash Algorithm Enum
// =============================================================================

/// Supported hash algorithms for forensic container verification.
///
/// Each algorithm has different characteristics making it suitable for
/// different use cases:
///
/// - **Cryptographic** (MD5, SHA-*, BLAKE*): Collision-resistant, suitable
///   for forensic integrity verification and chain-of-custody
/// - **Non-cryptographic** (XXH*, CRC32): Extremely fast but not
///   collision-resistant, suitable for quick integrity checks
///
/// # Forensic Standards
///
/// For court-admissible evidence, SHA-256 is the recommended standard.
/// MD5 and SHA-1 are still widely used for backward compatibility with
/// older forensic tools and container formats.
///
/// # Example
///
/// ```rust,ignore
/// use crate::common::hash::HashAlgorithm;
///
/// let algo: HashAlgorithm = "sha256".parse().unwrap();
/// assert_eq!(algo.name(), "SHA-256");
/// assert_eq!(algo.hash_length(), 64); // hex characters
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// MD5 (128-bit) - Legacy algorithm, widely supported
    Md5,
    /// SHA-1 (160-bit) - Legacy algorithm, E01 format standard
    Sha1,
    /// SHA-256 (256-bit) - NIST standard, court-accepted
    Sha256,
    /// SHA-512 (512-bit) - High-security variant
    Sha512,
    /// BLAKE3 (256-bit) - Modern, parallel-optimized
    Blake3,
    /// BLAKE2b (512-bit) - Modern cryptographic hash
    Blake2,
    /// XXH3 (128-bit) - Ultra-fast non-cryptographic
    Xxh3,
    /// XXH64 (64-bit) - Ultra-fast non-cryptographic
    Xxh64,
    /// CRC32 (32-bit) - Simple checksum
    Crc32,
}

impl FromStr for HashAlgorithm {
    type Err = ContainerError;

    /// Parse algorithm name from string (case-insensitive).
    ///
    /// Accepts common variants like "sha256", "SHA-256", "sha-256".
    ///
    /// # Errors
    ///
    /// Returns `ContainerError::ConfigError` for unknown algorithm names.
    fn from_str(algorithm: &str) -> Result<Self, Self::Err> {
        match algorithm.trim().to_lowercase().as_str() {
            "md5" => Ok(HashAlgorithm::Md5),
            "sha1" | "sha-1" => Ok(HashAlgorithm::Sha1),
            "sha256" | "sha-256" => Ok(HashAlgorithm::Sha256),
            "sha512" | "sha-512" => Ok(HashAlgorithm::Sha512),
            "blake3" => Ok(HashAlgorithm::Blake3),
            "blake2" | "blake2b" => Ok(HashAlgorithm::Blake2),
            "xxh3" | "xxhash3" => Ok(HashAlgorithm::Xxh3),
            "xxh64" | "xxhash64" => Ok(HashAlgorithm::Xxh64),
            "crc32" | "crc-32" => Ok(HashAlgorithm::Crc32),
            _ => Err(ContainerError::ConfigError(format!(
                "Unsupported hash algorithm: '{}'. Supported: md5, sha1, sha256, sha512, blake3, blake2, xxh3, xxh64, crc32",
                algorithm
            ))),
        }
    }
}

impl HashAlgorithm {
    /// Get the canonical display name for this algorithm.
    ///
    /// Returns a standardized name suitable for display and reports.
    pub fn name(&self) -> &'static str {
        match self {
            HashAlgorithm::Md5 => "MD5",
            HashAlgorithm::Sha1 => "SHA-1",
            HashAlgorithm::Sha256 => "SHA-256",
            HashAlgorithm::Sha512 => "SHA-512",
            HashAlgorithm::Blake3 => "BLAKE3",
            HashAlgorithm::Blake2 => "BLAKE2b",
            HashAlgorithm::Xxh3 => "XXH3",
            HashAlgorithm::Xxh64 => "XXH64",
            HashAlgorithm::Crc32 => "CRC32",
        }
    }

    /// Get expected hash length in hexadecimal characters.
    ///
    /// Useful for validating hash string lengths or pre-allocating buffers.
    pub fn hash_length(&self) -> usize {
        match self {
            HashAlgorithm::Md5 => 32,
            HashAlgorithm::Sha1 => 40,
            HashAlgorithm::Sha256 => 64,
            HashAlgorithm::Sha512 => 128,
            HashAlgorithm::Blake3 => 64,
            HashAlgorithm::Blake2 => 128,
            HashAlgorithm::Xxh3 => 32,  // 128-bit = 32 hex chars
            HashAlgorithm::Xxh64 => 16, // 64-bit = 16 hex chars
            HashAlgorithm::Crc32 => 8,  // 32-bit = 8 hex chars
        }
    }
}

/// Convenience function to parse algorithm string.
///
/// Equivalent to `algorithm.parse::<HashAlgorithm>()`.
pub fn parse_algorithm(algorithm: &str) -> Result<HashAlgorithm, ContainerError> {
    algorithm.parse()
}

// =============================================================================
// Streaming Hasher - Unified interface for incremental hashing
// =============================================================================

/// Unified streaming hasher for incremental hashing of large files.
///
/// `StreamingHasher` provides a consistent interface across all supported
/// hash algorithms, enabling chunk-by-chunk processing of large forensic
/// images without loading entire files into memory.
///
/// # Memory Optimization
///
/// BLAKE3 and XXH3 hashers are boxed because their internal state is large
/// (~1920 and ~576 bytes respectively), while other algorithms are ~20-224 bytes.
/// Boxing keeps the enum size reasonable for stack allocation.
///
/// # Example
///
/// ```rust,ignore
/// use crate::common::hash::{HashAlgorithm, StreamingHasher};
///
/// let mut hasher = StreamingHasher::new(HashAlgorithm::Sha256);
///
/// // Feed data in chunks (e.g., from file reads)
/// let chunk1 = vec![0u8; 65536];
/// let chunk2 = vec![1u8; 65536];
/// hasher.update(&chunk1);
/// hasher.update(&chunk2);
///
/// // Get final hash as lowercase hex string
/// let hash = hasher.finalize();
/// println!("SHA-256: {}", hash);
/// ```
pub enum StreamingHasher {
    /// MD5 streaming hasher
    Md5(Md5),
    /// SHA-1 streaming hasher
    Sha1(Sha1),
    /// SHA-256 streaming hasher
    Sha256(Sha256),
    /// SHA-512 streaming hasher
    Sha512(Sha512),
    /// BLAKE3 streaming hasher (boxed due to large state ~1920 bytes)
    Blake3(Box<Blake3Hasher>),
    /// BLAKE2b streaming hasher
    Blake2(Blake2b512),
    /// XXH3 streaming hasher (boxed due to large state ~576 bytes)
    Xxh3(Box<Xxh3>),
    /// XXH64 streaming hasher
    Xxh64(Xxh64),
    /// CRC32 streaming hasher
    Crc32(Crc32Hasher),
}

impl FromStr for StreamingHasher {
    type Err = String;

    /// Create a streaming hasher from algorithm string.
    fn from_str(algorithm: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(algorithm.parse()?))
    }
}

impl StreamingHasher {
    /// Create a new streaming hasher for the specified algorithm.
    pub fn new(algorithm: HashAlgorithm) -> Self {
        match algorithm {
            HashAlgorithm::Md5 => StreamingHasher::Md5(Md5::new()),
            HashAlgorithm::Sha1 => StreamingHasher::Sha1(Sha1::new()),
            HashAlgorithm::Sha256 => StreamingHasher::Sha256(Sha256::new()),
            HashAlgorithm::Sha512 => StreamingHasher::Sha512(Sha512::new()),
            HashAlgorithm::Blake3 => StreamingHasher::Blake3(Box::new(Blake3Hasher::new())),
            HashAlgorithm::Blake2 => StreamingHasher::Blake2(Blake2b512::new()),
            HashAlgorithm::Xxh3 => StreamingHasher::Xxh3(Box::new(Xxh3::new())),
            HashAlgorithm::Xxh64 => StreamingHasher::Xxh64(Xxh64::new(0)),
            HashAlgorithm::Crc32 => StreamingHasher::Crc32(Crc32Hasher::new()),
        }
    }

    /// Update the hash with more data
    pub fn update(&mut self, data: &[u8]) {
        match self {
            StreamingHasher::Md5(h) => Digest::update(h, data),
            StreamingHasher::Sha1(h) => Digest::update(h, data),
            StreamingHasher::Sha256(h) => Digest::update(h, data),
            StreamingHasher::Sha512(h) => Digest::update(h, data),
            StreamingHasher::Blake3(h) => { h.update(data); }
            StreamingHasher::Blake2(h) => Digest::update(h, data),
            StreamingHasher::Xxh3(h) => h.update(data),
            StreamingHasher::Xxh64(h) => h.update(data),
            StreamingHasher::Crc32(h) => h.update(data),
        }
    }

    /// Update with parallel hashing (only effective for BLAKE3)
    /// Falls back to regular update for other algorithms
    pub fn update_parallel(&mut self, data: &[u8]) {
        match self {
            StreamingHasher::Blake3(h) => { h.update_rayon(data); }
            _ => self.update(data),
        }
    }

    /// Finalize and return the hash as a hex string
    pub fn finalize(self) -> String {
        match self {
            StreamingHasher::Md5(h) => hex::encode(h.finalize()),
            StreamingHasher::Sha1(h) => hex::encode(h.finalize()),
            StreamingHasher::Sha256(h) => hex::encode(h.finalize()),
            StreamingHasher::Sha512(h) => hex::encode(h.finalize()),
            StreamingHasher::Blake3(h) => h.finalize().to_hex().to_string(),
            StreamingHasher::Blake2(h) => hex::encode(h.finalize()),
            StreamingHasher::Xxh3(h) => format!("{:032x}", h.digest128()),
            StreamingHasher::Xxh64(h) => format!("{:016x}", h.digest()),
            StreamingHasher::Crc32(h) => format!("{:08x}", h.finalize()),
        }
    }
}

// =============================================================================
// One-shot Hash Computation
// =============================================================================

/// Compute hash of data using specified algorithm (one-shot, for small data)
pub fn compute_hash(data: &[u8], algorithm: HashAlgorithm) -> String {
    match algorithm {
        HashAlgorithm::Md5 => {
            let mut hasher = Md5::new();
            hasher.update(data);
            hex::encode(hasher.finalize())
        }
        HashAlgorithm::Sha1 => {
            let mut hasher = Sha1::new();
            hasher.update(data);
            hex::encode(hasher.finalize())
        }
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            hex::encode(hasher.finalize())
        }
        HashAlgorithm::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            hex::encode(hasher.finalize())
        }
        HashAlgorithm::Blake3 => {
            let mut hasher = Blake3Hasher::new();
            hasher.update(data);
            hasher.finalize().to_hex().to_string()
        }
        HashAlgorithm::Blake2 => {
            let mut hasher = Blake2b512::new();
            hasher.update(data);
            hex::encode(hasher.finalize())
        }
        HashAlgorithm::Xxh3 => {
            let mut hasher = Xxh3::new();
            hasher.update(data);
            format!("{:032x}", hasher.digest128())
        }
        HashAlgorithm::Xxh64 => {
            let hash = xxhash_rust::xxh64::xxh64(data, 0);
            format!("{:016x}", hash)
        }
        HashAlgorithm::Crc32 => {
            let mut hasher = Crc32Hasher::new();
            hasher.update(data);
            format!("{:08x}", hasher.finalize())
        }
    }
}

/// Compute hash from algorithm string (convenience wrapper)
pub fn compute_hash_str(data: &[u8], algorithm: &str) -> Result<String, ContainerError> {
    let algo: HashAlgorithm = algorithm.parse()?;
    Ok(compute_hash(data, algo))
}

// =============================================================================
// File Hashing with Progress
// =============================================================================

/// Hash a file with progress reporting
/// 
/// # Arguments
/// * `path` - Path to the file to hash
/// * `algorithm` - Hash algorithm to use
/// * `progress_callback` - Called with (bytes_processed, total_bytes)
/// 
/// # Returns
/// The hex-encoded hash string
#[instrument(skip(progress_callback), fields(path = %path.display()))]
pub fn hash_file_with_progress<F>(
    path: &Path,
    algorithm: &str,
    mut progress_callback: F,
) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64),
{
    if !path.exists() {
        return Err(ContainerError::FileNotFound(format!("File not found: {}", path.display())));
    }

    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    let total_size = metadata.len();
    
    debug!(algorithm, total_size, "Starting file hash");

    // Use adaptive buffer sizing based on file size
    let buffer_size = AdaptiveBuffer::optimal_size(total_size, IoOperation::Hash);
    trace!(buffer_size, "Using adaptive buffer for hashing");

    let file = File::open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    let mut reader = BufReader::with_capacity(buffer_size, file);

    let algorithm_lower = algorithm.to_lowercase();

    // For BLAKE3, use parallel hashing for best performance
    if algorithm_lower == "blake3" {
        trace!("Using BLAKE3 parallel hashing");
        return hash_file_blake3_parallel(&mut reader, total_size, &mut progress_callback);
    }

    // For other algorithms, use streaming hasher
    trace!("Using streaming hasher for {}", algorithm);
    let mut hasher: StreamingHasher = algorithm.parse()?;
    let mut bytes_read_total = 0u64;
    
    // Use adaptive progress chunk count
    let progress_chunks = AdaptiveBuffer::progress_chunks(total_size);
    let report_interval = (total_size / progress_chunks).max(buffer_size as u64);
    let mut last_report = 0u64;

    loop {
        let buf = reader.fill_buf()
            .map_err(|e| format!("Read error: {}", e))?;
        let len = buf.len();
        if len == 0 {
            break;
        }
        hasher.update(buf);
        reader.consume(len);

        bytes_read_total += len as u64;
        if bytes_read_total - last_report >= report_interval {
            progress_callback(bytes_read_total, total_size);
            last_report = bytes_read_total;
        }
    }

    progress_callback(total_size, total_size);
    let hash = hasher.finalize();
    debug!(hash = %hash, "File hash complete");
    Ok(hash)
}

/// BLAKE3 optimized path using rayon parallel hashing
fn hash_file_blake3_parallel<R, F>(
    reader: &mut BufReader<R>,
    total_size: u64,
    progress_callback: &mut F,
) -> Result<String, ContainerError>
where
    R: std::io::Read,
    F: FnMut(u64, u64),
{
    let mut hasher = Blake3Hasher::new();
    let mut bytes_read_total = 0u64;
    
    // Use adaptive progress chunk count
    let progress_chunks = AdaptiveBuffer::progress_chunks(total_size);
    let buffer_size = AdaptiveBuffer::optimal_size(total_size, IoOperation::Hash);
    let report_interval = (total_size / progress_chunks).max(buffer_size as u64);
    let mut last_report = 0u64;

    loop {
        let buf = reader.fill_buf()
            .map_err(|e| format!("Read error: {}", e))?;
        let len = buf.len();
        if len == 0 {
            break;
        }

        // BLAKE3 update_rayon uses all cores for parallel hashing
        hasher.update_rayon(buf);
        reader.consume(len);

        bytes_read_total += len as u64;
        if bytes_read_total - last_report >= report_interval {
            progress_callback(bytes_read_total, total_size);
            last_report = bytes_read_total;
        }
    }

    progress_callback(total_size, total_size);
    Ok(hasher.finalize().to_hex().to_string())
}

/// Hash a file without progress reporting (convenience wrapper)
pub fn hash_file(path: &Path, algorithm: &str) -> Result<String, ContainerError> {
    hash_file_with_progress(path, algorithm, |_, _| {})
}

// =============================================================================
// Hash Validation Utilities
// =============================================================================

/// Validate that a string looks like a valid hash for the given algorithm
pub fn is_valid_hash(hash: &str, algorithm: HashAlgorithm) -> bool {
    let expected_len = algorithm.hash_length();
    hash.len() == expected_len && hash.chars().all(|c| c.is_ascii_hexdigit())
}

/// Guess the hash algorithm from a hash string based on length
pub fn guess_algorithm_from_hash(hash: &str) -> Option<HashAlgorithm> {
    match hash.len() {
        32 => Some(HashAlgorithm::Md5),
        40 => Some(HashAlgorithm::Sha1),
        64 => Some(HashAlgorithm::Sha256), // Could also be BLAKE3
        128 => Some(HashAlgorithm::Sha512), // Could also be BLAKE2b
        _ => None,
    }
}

/// Compare two hashes (case-insensitive)
pub fn hashes_match(hash1: &str, hash2: &str) -> bool {
    hash1.to_lowercase() == hash2.to_lowercase()
}

// =============================================================================
// Hash Comparison and Verification
// =============================================================================

/// Result of hash comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HashMatchResult {
    /// Exact match (same case)
    Exact,
    /// Match but different case (e.g., "abc" vs "ABC")
    CaseInsensitive,
    /// Hashes do not match
    Mismatch,
    /// One or both hashes are invalid format
    Invalid,
}

impl HashMatchResult {
    /// Returns true if the hashes match (exact or case-insensitive)
    pub fn is_match(&self) -> bool {
        matches!(self, HashMatchResult::Exact | HashMatchResult::CaseInsensitive)
    }
}

/// Compare two hash strings with detailed result
pub fn compare_hashes(computed: &str, expected: &str) -> HashMatchResult {
    let computed = computed.trim();
    let expected = expected.trim();
    
    // Check for invalid characters
    if !computed.chars().all(|c| c.is_ascii_hexdigit()) 
        || !expected.chars().all(|c| c.is_ascii_hexdigit()) {
        return HashMatchResult::Invalid;
    }
    
    // Length mismatch
    if computed.len() != expected.len() {
        return HashMatchResult::Mismatch;
    }
    
    // Exact match
    if computed == expected {
        return HashMatchResult::Exact;
    }
    
    // Case-insensitive match
    if computed.eq_ignore_ascii_case(expected) {
        return HashMatchResult::CaseInsensitive;
    }
    
    HashMatchResult::Mismatch
}

/// Detailed hash verification result
#[derive(Debug, Clone, Serialize)]
pub struct HashVerificationResult {
    /// Algorithm used for hashing
    pub algorithm: String,
    /// Computed hash value
    pub computed: String,
    /// Expected/stored hash value
    pub expected: String,
    /// Whether the hashes match
    pub matches: bool,
    /// Detailed match result
    pub match_result: HashMatchResult,
    /// Time taken to compute (in milliseconds)
    pub compute_time_ms: Option<u64>,
    /// Size of data hashed
    pub data_size: Option<u64>,
}

impl HashVerificationResult {
    /// Create a new verification result
    pub fn new(
        algorithm: HashAlgorithm,
        computed: String,
        expected: String,
    ) -> Self {
        let match_result = compare_hashes(&computed, &expected);
        Self {
            algorithm: algorithm.name().to_string(),
            computed,
            expected,
            matches: match_result.is_match(),
            match_result,
            compute_time_ms: None,
            data_size: None,
        }
    }

    /// Add timing information
    pub fn with_timing(mut self, compute_time_ms: u64) -> Self {
        self.compute_time_ms = Some(compute_time_ms);
        self
    }

    /// Add data size information
    pub fn with_data_size(mut self, size: u64) -> Self {
        self.data_size = Some(size);
        self
    }
}

/// Batch verification result for multiple algorithms
#[derive(Debug, Clone, Serialize)]
pub struct MultiHashVerification {
    /// File or data identifier
    pub identifier: String,
    /// Individual verification results
    pub results: Vec<HashVerificationResult>,
    /// Overall pass/fail (all must match)
    pub all_passed: bool,
    /// Number of algorithms verified
    pub algorithm_count: usize,
}

impl MultiHashVerification {
    /// Create from a list of verification results
    pub fn from_results(identifier: String, results: Vec<HashVerificationResult>) -> Self {
        let all_passed = results.iter().all(|r| r.matches);
        let algorithm_count = results.len();
        Self {
            identifier,
            results,
            all_passed,
            algorithm_count,
        }
    }
}

/// Verify a computed hash against an expected value
pub fn verify_hash(
    data: &[u8],
    expected: &str,
    algorithm: HashAlgorithm,
) -> HashVerificationResult {
    use std::time::Instant;
    
    let start = Instant::now();
    let computed = compute_hash(data, algorithm);
    let elapsed = start.elapsed().as_millis() as u64;
    
    HashVerificationResult::new(algorithm, computed, expected.to_string())
        .with_timing(elapsed)
        .with_data_size(data.len() as u64)
}

/// Verify a file's hash against an expected value
pub fn verify_file_hash(
    path: &std::path::Path,
    expected: &str,
    algorithm: HashAlgorithm,
) -> Result<HashVerificationResult, ContainerError> {
    use std::time::Instant;
    
    let start = Instant::now();
    let computed = hash_file(path, algorithm.name())?;
    let elapsed = start.elapsed().as_millis() as u64;
    
    let file_size = std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0);
    
    Ok(HashVerificationResult::new(algorithm, computed, expected.to_string())
        .with_timing(elapsed)
        .with_data_size(file_size))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_parsing() {
        assert_eq!("md5".parse::<HashAlgorithm>().unwrap(), HashAlgorithm::Md5);
        assert_eq!("MD5".parse::<HashAlgorithm>().unwrap(), HashAlgorithm::Md5);
        assert_eq!("sha1".parse::<HashAlgorithm>().unwrap(), HashAlgorithm::Sha1);
        assert_eq!("SHA-1".parse::<HashAlgorithm>().unwrap(), HashAlgorithm::Sha1);
        assert_eq!("sha256".parse::<HashAlgorithm>().unwrap(), HashAlgorithm::Sha256);
        assert_eq!("SHA-256".parse::<HashAlgorithm>().unwrap(), HashAlgorithm::Sha256);
        assert_eq!("blake3".parse::<HashAlgorithm>().unwrap(), HashAlgorithm::Blake3);
        assert!("invalid".parse::<HashAlgorithm>().is_err());
    }

    #[test]
    fn test_compute_hash() {
        let data = b"hello world";
        
        let md5 = compute_hash(data, HashAlgorithm::Md5);
        assert_eq!(md5, "5eb63bbbe01eeed093cb22bb8f5acdc3");
        
        let sha1 = compute_hash(data, HashAlgorithm::Sha1);
        assert_eq!(sha1, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
    }

    #[test]
    fn test_streaming_hasher() {
        let mut hasher = StreamingHasher::new(HashAlgorithm::Md5);
        hasher.update(b"hello ");
        hasher.update(b"world");
        let hash = hasher.finalize();
        assert_eq!(hash, "5eb63bbbe01eeed093cb22bb8f5acdc3");
    }

    #[test]
    fn test_hash_validation() {
        assert!(is_valid_hash("5eb63bbbe01eeed093cb22bb8f5acdc3", HashAlgorithm::Md5));
        assert!(!is_valid_hash("invalid", HashAlgorithm::Md5));
        assert!(!is_valid_hash("5eb63bbbe01eeed093cb22bb8f5acdc3", HashAlgorithm::Sha1));
    }

    #[test]
    fn test_guess_algorithm() {
        assert_eq!(guess_algorithm_from_hash("5eb63bbbe01eeed093cb22bb8f5acdc3"), Some(HashAlgorithm::Md5));
        assert_eq!(guess_algorithm_from_hash("2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"), Some(HashAlgorithm::Sha1));
    }
}
