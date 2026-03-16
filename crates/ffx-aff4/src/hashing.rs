// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Multi-algorithm hashing for AFF4 containers.
//!
//! AFF4 uses two kinds of hashes:
//! - **Linear**: Whole-stream hash (e.g., SHA-256 of entire disk image)
//! - **Block**: Per-chunk hash stored as raw binary in `XXXXXXXX.<ext>` files
//!
//! `StreamHasher` accumulates data via `update()` and produces hex digests
//! for all configured algorithms. `hash_chunk` computes a hash for a single
//! chunk and returns the binary digest.

use crate::types::Aff4HashAlgorithm;

use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

// ─── Dynamic Digest Trait ────────────────────────────────────────────────────

/// Trait object wrapper for digest operations.
trait DynDigest: Send {
    fn dyn_update(&mut self, data: &[u8]);
    fn finalize_hex(self: Box<Self>) -> String;
    fn finalize_bytes(self: Box<Self>) -> Vec<u8>;
}

macro_rules! impl_dyn_digest {
    ($ty:ty) => {
        impl DynDigest for $ty {
            fn dyn_update(&mut self, data: &[u8]) {
                Digest::update(self, data);
            }

            fn finalize_hex(self: Box<Self>) -> String {
                hex::encode(Digest::finalize(*self))
            }

            fn finalize_bytes(self: Box<Self>) -> Vec<u8> {
                Digest::finalize(*self).to_vec()
            }
        }
    };
}

impl_dyn_digest!(Md5);
impl_dyn_digest!(Sha1);
impl_dyn_digest!(Sha256);
impl_dyn_digest!(Sha512);

/// Accumulates data for multiple hash algorithms simultaneously.
pub struct StreamHasher {
    hashers: Vec<(Aff4HashAlgorithm, Box<dyn DynDigest>)>,
}

impl StreamHasher {
    /// Create a hasher for the given algorithms.
    pub fn new(algorithms: &[Aff4HashAlgorithm]) -> Self {
        let mut hashers: Vec<(Aff4HashAlgorithm, Box<dyn DynDigest>)> = Vec::new();

        for alg in algorithms {
            let digest: Box<dyn DynDigest> = match alg {
                Aff4HashAlgorithm::Md5 => Box::new(Md5::new()),
                Aff4HashAlgorithm::Sha1 => Box::new(Sha1::new()),
                Aff4HashAlgorithm::Sha256 => Box::new(Sha256::new()),
                Aff4HashAlgorithm::Sha512 => Box::new(Sha512::new()),
                // Blake2b requires a separate crate (blake2); skipped for now.
                // Containers requesting Blake2b will silently omit its hash.
                Aff4HashAlgorithm::Blake2b => continue,
            };
            hashers.push((*alg, digest));
        }

        Self { hashers }
    }

    /// Feed data into all active hashers.
    pub fn update(&mut self, data: &[u8]) {
        for (_, hasher) in &mut self.hashers {
            hasher.dyn_update(data);
        }
    }

    /// Finalize and return `(algorithm, hex_digest)` pairs.
    pub fn finalize(self) -> Vec<(Aff4HashAlgorithm, String)> {
        self.hashers
            .into_iter()
            .map(|(alg, hasher)| (alg, hasher.finalize_hex()))
            .collect()
    }

    /// Finalize and return `(algorithm, binary_digest)` pairs.
    pub fn finalize_bytes(self) -> Vec<(Aff4HashAlgorithm, Vec<u8>)> {
        self.hashers
            .into_iter()
            .map(|(alg, hasher)| (alg, hasher.finalize_bytes()))
            .collect()
    }
}

// ─── Block (Per-Chunk) Hasher ────────────────────────────────────────────────

/// Compute the hash of a single chunk, returning the raw binary digest.
pub fn hash_chunk(data: &[u8], algorithm: Aff4HashAlgorithm) -> Vec<u8> {
    match algorithm {
        Aff4HashAlgorithm::Md5 => Digest::finalize(Md5::new_with_prefix(data)).to_vec(),
        Aff4HashAlgorithm::Sha1 => Digest::finalize(Sha1::new_with_prefix(data)).to_vec(),
        Aff4HashAlgorithm::Sha256 => Digest::finalize(Sha256::new_with_prefix(data)).to_vec(),
        Aff4HashAlgorithm::Sha512 => Digest::finalize(Sha512::new_with_prefix(data)).to_vec(),
        // Blake2b not yet supported — returns empty digest.
        Aff4HashAlgorithm::Blake2b => Vec::new(),
    }
}

/// Compute the hex-encoded hash of data.
pub fn hash_hex(data: &[u8], algorithm: Aff4HashAlgorithm) -> String {
    hex::encode(hash_chunk(data, algorithm))
}

// ─── Block Map Hash ──────────────────────────────────────────────────────────

/// Compute the AFF4 BlockMapHash: `H(block_hashes || map_point_hash || map_idx_hash)`.
///
/// `block_hashes_data`: concatenated binary block hash data for all bevies
/// `map_point_hash`: binary hash of concatenated map point data
/// `map_idx_hash`: binary hash of concatenated map idx data
pub fn compute_block_map_hash(
    algorithm: Aff4HashAlgorithm,
    block_hashes_data: &[u8],
    map_point_hash: &[u8],
    map_idx_hash: &[u8],
) -> Vec<u8> {
    let mut hasher = StreamHasher::new(&[algorithm]);
    hasher.update(block_hashes_data);
    hasher.update(map_point_hash);
    hasher.update(map_idx_hash);
    let results = hasher.finalize_bytes();
    results.into_iter().next().map(|(_, v)| v).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_hasher_sha256() {
        let mut hasher = StreamHasher::new(&[Aff4HashAlgorithm::Sha256]);
        hasher.update(b"hello ");
        hasher.update(b"world");
        let results = hasher.finalize();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, Aff4HashAlgorithm::Sha256);
        // SHA-256 of "hello world"
        assert_eq!(
            results[0].1,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_stream_hasher_multi() {
        let mut hasher = StreamHasher::new(&[
            Aff4HashAlgorithm::Md5,
            Aff4HashAlgorithm::Sha1,
            Aff4HashAlgorithm::Sha256,
        ]);
        hasher.update(b"test");
        let results = hasher.finalize();
        assert_eq!(results.len(), 3);

        // MD5 of "test"
        assert_eq!(results[0].1, "098f6bcd4621d373cade4e832627b4f6");
        // SHA-1 of "test"
        assert_eq!(results[1].1, "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3");
        // SHA-256 of "test"
        assert_eq!(
            results[2].1,
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );
    }

    #[test]
    fn test_hash_chunk_sha256() {
        let digest = hash_chunk(b"hello world", Aff4HashAlgorithm::Sha256);
        assert_eq!(digest.len(), 32);
        assert_eq!(
            hex::encode(&digest),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_hash_hex() {
        let hex_str = hash_hex(b"test", Aff4HashAlgorithm::Md5);
        assert_eq!(hex_str, "098f6bcd4621d373cade4e832627b4f6");
    }

    #[test]
    fn test_block_map_hash() {
        let block_data = b"block-hash-data";
        let map_point = hash_chunk(b"map-point", Aff4HashAlgorithm::Sha256);
        let map_idx = hash_chunk(b"map-idx", Aff4HashAlgorithm::Sha256);

        let result =
            compute_block_map_hash(Aff4HashAlgorithm::Sha256, block_data, &map_point, &map_idx);
        assert_eq!(result.len(), 32);
    }
}
