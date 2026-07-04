// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Hash Result Caching for Forensic Container Verification
//!
//! This module provides a thread-safe cache for computed hash results, avoiding
//! expensive recomputation of large forensic container hashes.
//!
//! # Cache Key Strategy
//!
//! Cache entries are keyed by:
//! - **File path** (canonical/absolute)
//! - **Algorithm** (SHA-256, MD5, etc.)
//! - **File modification time** (invalidates stale entries)
//! - **File size** (additional staleness check)
//!
//! # Thread Safety
//!
//! The cache uses `parking_lot::RwLock` for efficient concurrent access:
//! - Multiple readers can access cache simultaneously
//! - Writers get exclusive access for updates
//!
//! # Memory Management
//!
//! The cache has a configurable maximum size (default: 1000 entries).
//! When full, the least-recently-used entries are evicted.
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::common::hash_cache::{HashCache, GLOBAL_HASH_CACHE};
//!
//! // Check cache before computing
//! let cache_key = HashCache::make_key("/path/to/file.e01", "sha256")?;
//!
//! if let Some(cached) = GLOBAL_HASH_CACHE.get(&cache_key) {
//!     println!("Cache hit: {}", cached.hash);
//! } else {
//!     // Compute hash...
//!     let hash = compute_expensive_hash(path);
//!     GLOBAL_HASH_CACHE.insert(cache_key, hash);
//! }
//! ```

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;
use std::time::SystemTime;
use tracing::{debug, trace};

use crate::hash::{is_valid_hash, HashAlgorithm};

// =============================================================================
// Configuration
// =============================================================================

/// Maximum number of cache entries before LRU eviction
const MAX_CACHE_ENTRIES: usize = 1000;
const MAX_CACHE_PATH_CHARS: usize = 4096;
const MAX_CACHE_SCOPE_CHARS: usize = 128;
const MAX_CACHE_HASH_CHARS: usize = 256;

// =============================================================================
// Cache Key and Entry
// =============================================================================

/// Unique key for a cached hash result
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct HashCacheKey {
    /// Canonical file path
    pub path: String,
    /// Hash algorithm used (lowercase)
    pub algorithm: String,
    /// File modification time at cache time
    pub modified: SystemTime,
    /// File size at cache time  
    pub size: u64,
}

/// Cached hash result with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HashCacheEntry {
    /// The computed hash value (hex string)
    pub hash: String,
    /// When this entry was cached
    pub cached_at: SystemTime,
    /// Number of times this entry was accessed
    pub access_count: u64,
}

// =============================================================================
// Hash Cache Implementation
// =============================================================================

/// Thread-safe hash result cache with LRU eviction
pub struct HashCache {
    entries: RwLock<HashMap<HashCacheKey, HashCacheEntry>>,
    max_entries: usize,
}

impl HashCache {
    /// Create a new hash cache with specified capacity
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::with_capacity(max_entries / 2)),
            max_entries,
        }
    }

    /// Create a cache key from a file path and algorithm
    ///
    /// This reads file metadata to capture modification time and size,
    /// which are used to invalidate stale cache entries.
    ///
    /// # Errors
    ///
    /// Returns `None` if the file doesn't exist or metadata can't be read.
    pub fn make_key(path: &str, algorithm: &str) -> Option<HashCacheKey> {
        Self::make_key_with_scope(path, algorithm, None)
    }

    /// Create a cache key with an additional semantic scope.
    ///
    /// Use this when the same file path can be hashed through different byte
    /// streams, such as raw container bytes versus decoded EWF media bytes.
    pub fn make_scoped_key(path: &str, algorithm: &str, scope: &str) -> Option<HashCacheKey> {
        let scope = normalize_cache_scope(scope)?;
        Self::make_key_with_scope(path, algorithm, Some(&scope))
    }

    fn make_key_with_scope(
        path: &str,
        algorithm: &str,
        scope: Option<&str>,
    ) -> Option<HashCacheKey> {
        if path.trim().is_empty() || path.chars().count() > MAX_CACHE_PATH_CHARS {
            return None;
        }
        let algorithm = normalize_hash_cache_algorithm(algorithm)?;
        let path_obj = Path::new(path);
        let canonical = path_obj.canonicalize().ok()?;
        let metadata = fs::metadata(&canonical).ok()?;
        let canonical = canonical.to_string_lossy().to_string();
        let path = match scope {
            Some(scope) => format!("{scope}:{canonical}"),
            None => canonical,
        };
        if path.chars().count() > MAX_CACHE_PATH_CHARS {
            return None;
        }

        Some(HashCacheKey {
            path,
            algorithm,
            modified: metadata.modified().ok()?,
            size: metadata.len(),
        })
    }

    /// Get a cached hash result if available and still valid
    ///
    /// Returns `None` if:
    /// - No cache entry exists
    /// - The file has been modified since caching
    /// - The file size has changed
    pub fn get(&self, key: &HashCacheKey) -> Option<String> {
        let mut entries = self.entries.write();

        if let Some(entry) = entries.get_mut(key) {
            // Update access count for LRU tracking
            entry.access_count += 1;
            trace!(path = %key.path, algorithm = %key.algorithm, "Cache hit");
            return Some(entry.hash.clone());
        }

        trace!(path = %key.path, algorithm = %key.algorithm, "Cache miss");
        None
    }

    /// Check if a hash is cached without updating access count
    pub fn contains(&self, key: &HashCacheKey) -> bool {
        self.entries.read().contains_key(key)
    }

    /// Insert a hash result into the cache
    ///
    /// If the cache is full, evicts the least-recently-used entry first.
    pub fn insert(&self, key: HashCacheKey, hash: String) {
        if !is_valid_cache_entry(&key, &hash) {
            debug!(path = %key.path, algorithm = %key.algorithm, "Skipping invalid hash cache entry");
            return;
        }

        let mut entries = self.entries.write();
        if self.max_entries == 0 {
            return;
        }

        // Evict if at capacity
        if entries.len() >= self.max_entries {
            self.evict_lru(&mut entries);
        }

        debug!(path = %key.path, algorithm = %key.algorithm, "Caching hash result");

        entries.insert(
            key,
            HashCacheEntry {
                hash,
                cached_at: SystemTime::now(),
                access_count: 1,
            },
        );
    }

    /// Remove a specific cache entry
    pub fn remove(&self, key: &HashCacheKey) -> Option<HashCacheEntry> {
        self.entries.write().remove(key)
    }

    /// Clear all entries for a specific file path (all algorithms)
    pub fn invalidate_path(&self, path: &str) {
        let path_lower = std::fs::canonicalize(path)
            .ok()
            .map(|path| path.to_string_lossy().to_lowercase())
            .unwrap_or_else(|| path.to_lowercase());
        let mut entries = self.entries.write();
        entries.retain(|k, _| k.path.to_lowercase() != path_lower);
        debug!(path = %path, "Invalidated cache entries for path");
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        self.entries.write().clear();
        debug!("Cleared hash cache");
    }

    /// Get current number of cached entries
    pub fn len(&self) -> usize {
        self.entries.read().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.read().is_empty()
    }

    /// Get cache statistics
    pub fn stats(&self) -> HashCacheStats {
        let entries = self.entries.read();
        let total_accesses: u64 = entries.values().map(|e| e.access_count).sum();

        HashCacheStats {
            entry_count: entries.len(),
            max_entries: self.max_entries,
            total_accesses,
        }
    }

    /// Evict the least-recently-used entry (lowest access count)
    fn evict_lru(&self, entries: &mut HashMap<HashCacheKey, HashCacheEntry>) {
        if let Some((key_to_remove, _)) = entries
            .iter()
            .min_by_key(|(_, entry)| entry.access_count)
            .map(|(k, e)| (k.clone(), e.clone()))
        {
            trace!(path = %key_to_remove.path, "Evicting LRU cache entry");
            entries.remove(&key_to_remove);
        }
    }
}

fn normalize_hash_cache_algorithm(algorithm: &str) -> Option<String> {
    let algorithm = algorithm.parse::<HashAlgorithm>().ok()?;
    Some(hash_cache_algorithm_key(algorithm).to_string())
}

fn normalize_cache_scope(scope: &str) -> Option<String> {
    let scope = scope.trim().to_lowercase();
    if scope.is_empty() || scope.chars().count() > MAX_CACHE_SCOPE_CHARS {
        return None;
    }
    Some(scope)
}

fn hash_cache_algorithm_key(algorithm: HashAlgorithm) -> &'static str {
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

fn is_valid_cache_entry(key: &HashCacheKey, hash: &str) -> bool {
    if key.path.trim().is_empty()
        || key.path.chars().count() > MAX_CACHE_PATH_CHARS
        || hash.trim().is_empty()
        || hash.chars().count() > MAX_CACHE_HASH_CHARS
    {
        return false;
    }
    let Ok(algorithm) = key.algorithm.parse::<HashAlgorithm>() else {
        return false;
    };
    is_valid_hash(hash, algorithm)
}

impl Default for HashCache {
    fn default() -> Self {
        Self::new(MAX_CACHE_ENTRIES)
    }
}

// =============================================================================
// Global Cache Instance
// =============================================================================

/// Global hash cache instance for application-wide caching
pub static GLOBAL_HASH_CACHE: LazyLock<HashCache> = LazyLock::new(HashCache::default);

// =============================================================================
// Cache Statistics
// =============================================================================

/// Statistics about cache usage
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HashCacheStats {
    /// Number of entries currently in cache
    pub entry_count: usize,
    /// Maximum cache capacity
    pub max_entries: usize,
    /// Total number of cache accesses
    pub total_accesses: u64,
}

// =============================================================================
// Convenience Functions
// =============================================================================

/// Try to get a cached hash for a file
///
/// Convenience function that creates the cache key and looks up the hash.
/// Returns `None` if not cached or file has changed.
pub fn get_cached_hash(path: &str, algorithm: &str) -> Option<String> {
    let key = HashCache::make_key(path, algorithm)?;
    GLOBAL_HASH_CACHE.get(&key)
}

/// Try to get a cached hash for a file and semantic read scope.
pub fn get_cached_hash_scoped(path: &str, algorithm: &str, scope: &str) -> Option<String> {
    let key = HashCache::make_scoped_key(path, algorithm, scope)?;
    GLOBAL_HASH_CACHE.get(&key)
}

/// Cache a computed hash result
///
/// Convenience function that creates the cache key and stores the hash.
/// Does nothing if the file doesn't exist.
pub fn cache_hash(path: &str, algorithm: &str, hash: String) {
    if let Some(key) = HashCache::make_key(path, algorithm) {
        GLOBAL_HASH_CACHE.insert(key, hash);
    }
}

/// Cache a computed hash result for a file and semantic read scope.
pub fn cache_hash_scoped(path: &str, algorithm: &str, scope: &str, hash: String) {
    if let Some(key) = HashCache::make_scoped_key(path, algorithm, scope) {
        GLOBAL_HASH_CACHE.insert(key, hash);
    }
}

/// Get a cached hash or compute it
///
/// This is the primary interface for cached hashing. If a valid cache entry
/// exists, returns it immediately. Otherwise, calls the compute function
/// and caches the result.
///
/// # Example
///
/// ```rust,ignore
/// let hash = get_or_compute_hash("/path/to/image.e01", "sha256", || {
///     // Expensive hash computation
///     compute_hash_slowly(path)
/// })?;
/// ```
pub fn get_or_compute_hash<F>(path: &str, algorithm: &str, compute: F) -> Option<String>
where
    F: FnOnce() -> Option<String>,
{
    // Try cache first
    if let Some(cached) = get_cached_hash(path, algorithm) {
        return Some(cached);
    }

    // Compute and cache
    let hash = compute()?;
    cache_hash(path, algorithm, hash.clone());
    Some(hash)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn valid_hash_for_algorithm(algorithm: &str) -> String {
        match algorithm {
            "md5" => "a".repeat(32),
            "sha1" => "b".repeat(40),
            "sha256" | "blake3" => "c".repeat(64),
            "sha512" | "blake2" => "d".repeat(128),
            "xxh3" => "e".repeat(32),
            "xxh64" => "f".repeat(16),
            "crc32" => "1".repeat(8),
            _ => panic!("unsupported test algorithm: {algorithm}"),
        }
    }

    #[test]
    fn test_cache_basic_operations() {
        let cache = HashCache::new(10);

        // Create a temp file to get valid metadata
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "test content").unwrap();
        let path = temp.path().to_string_lossy().to_string();

        let key = HashCache::make_key(&path, "sha256").unwrap();

        // Initially empty
        assert!(cache.get(&key).is_none());

        // Insert and retrieve
        let hash = valid_hash_for_algorithm("sha256");
        cache.insert(key.clone(), hash.clone());
        assert_eq!(cache.get(&key), Some(hash));

        // Remove
        cache.remove(&key);
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = HashCache::new(3);

        // Create temp files
        let files: Vec<_> = (0..4)
            .map(|i| {
                let mut temp = NamedTempFile::new().unwrap();
                writeln!(temp, "content {}", i).unwrap();
                temp
            })
            .collect();

        // Insert 3 entries
        for (i, f) in files.iter().take(3).enumerate() {
            let path = f.path().to_string_lossy().to_string();
            let key = HashCache::make_key(&path, "md5").unwrap();
            cache.insert(key, format!("{:032x}", i));
        }

        // Access first entry to increase its count
        let key0 = HashCache::make_key(&files[0].path().to_string_lossy(), "md5").unwrap();
        cache.get(&key0);
        cache.get(&key0);

        // Insert 4th entry - should evict entry with lowest access count
        let path3 = files[3].path().to_string_lossy().to_string();
        let key3 = HashCache::make_key(&path3, "md5").unwrap();
        cache.insert(key3, "3".repeat(32));

        // Entry 0 should still exist (high access count)
        assert!(cache.get(&key0).is_some());

        // Cache should be at max capacity
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_cache_invalidate_path() {
        let cache = HashCache::new(10);

        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "test").unwrap();
        // Use canonicalized path so invalidate_path matches the keys
        // (on Windows, canonicalize resolves 8.3 short names like RUNNER~1)
        let path = std::fs::canonicalize(temp.path())
            .unwrap()
            .to_string_lossy()
            .to_string();

        // Insert with multiple algorithms
        for algo in ["md5", "sha256", "blake3"] {
            let key = HashCache::make_key(&path, algo).unwrap();
            cache.insert(key, valid_hash_for_algorithm(algo));
        }

        assert_eq!(cache.len(), 3);

        // Invalidate all entries for this path
        cache.invalidate_path(&path);

        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_make_key_normalizes_algorithm_aliases() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "test").unwrap();
        let path = temp.path().to_string_lossy().to_string();

        let plain = HashCache::make_key(&path, "sha256").unwrap();
        let dashed = HashCache::make_key(&path, "SHA-256").unwrap();

        assert_eq!(plain.algorithm, "sha256");
        assert_eq!(plain, dashed);
    }

    #[test]
    fn test_make_scoped_key_separates_same_file_and_algorithm() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "test").unwrap();
        let path = temp.path().to_string_lossy().to_string();

        let raw = HashCache::make_scoped_key(&path, "sha256", "raw-file").unwrap();
        let decoded = HashCache::make_scoped_key(&path, "sha256", "decoded-ewf").unwrap();

        assert_ne!(raw, decoded);
        assert!(raw.path.starts_with("raw-file:"));
        assert!(decoded.path.starts_with("decoded-ewf:"));
        assert_eq!(raw.algorithm, decoded.algorithm);
        assert_eq!(raw.modified, decoded.modified);
        assert_eq!(raw.size, decoded.size);
    }

    #[test]
    fn test_make_key_rejects_invalid_algorithm_and_path() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "test").unwrap();
        let path = temp.path().to_string_lossy().to_string();

        assert!(HashCache::make_key(&path, "rot13").is_none());
        assert!(HashCache::make_scoped_key(&path, "sha256", " ").is_none());
        assert!(HashCache::make_key(" ", "sha256").is_none());
        assert!(HashCache::make_key(&"a".repeat(MAX_CACHE_PATH_CHARS + 1), "sha256").is_none());
    }

    #[test]
    fn test_insert_rejects_invalid_hash_value() {
        let cache = HashCache::new(10);
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "test").unwrap();
        let path = temp.path().to_string_lossy().to_string();
        let key = HashCache::make_key(&path, "sha256").unwrap();

        cache.insert(key.clone(), "abc123".to_string());

        assert!(cache.get(&key).is_none());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_insert_noops_when_capacity_is_zero() {
        let cache = HashCache::new(0);
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "test").unwrap();
        let path = temp.path().to_string_lossy().to_string();
        let key = HashCache::make_key(&path, "md5").unwrap();

        cache.insert(key, valid_hash_for_algorithm("md5"));

        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_invalidate_path_uses_exact_canonical_path() {
        let cache = HashCache::new(10);
        let mut first = NamedTempFile::new().unwrap();
        let mut second = NamedTempFile::new().unwrap();
        writeln!(first, "first").unwrap();
        writeln!(second, "second").unwrap();
        let first_path = std::fs::canonicalize(first.path())
            .unwrap()
            .to_string_lossy()
            .to_string();
        let second_path = std::fs::canonicalize(second.path())
            .unwrap()
            .to_string_lossy()
            .to_string();
        let first_key = HashCache::make_key(&first_path, "md5").unwrap();
        let second_key = HashCache::make_key(&second_path, "md5").unwrap();

        cache.insert(first_key.clone(), valid_hash_for_algorithm("md5"));
        cache.insert(second_key.clone(), valid_hash_for_algorithm("md5"));
        cache.invalidate_path(&first_path);

        assert!(cache.get(&first_key).is_none());
        assert!(cache.get(&second_key).is_some());
    }
}
