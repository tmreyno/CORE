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

// =============================================================================
// Configuration
// =============================================================================

/// Maximum number of cache entries before LRU eviction
const MAX_CACHE_ENTRIES: usize = 1000;

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
        let path_obj = Path::new(path);
        let canonical = path_obj.canonicalize().ok()?;
        let metadata = fs::metadata(&canonical).ok()?;

        Some(HashCacheKey {
            path: canonical.to_string_lossy().to_string(),
            algorithm: algorithm.to_lowercase(),
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
        let mut entries = self.entries.write();

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
        let path_lower = path.to_lowercase();
        let mut entries = self.entries.write();
        entries.retain(|k, _| !k.path.to_lowercase().contains(&path_lower));
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

/// Cache a computed hash result
///
/// Convenience function that creates the cache key and stores the hash.
/// Does nothing if the file doesn't exist.
pub fn cache_hash(path: &str, algorithm: &str, hash: String) {
    if let Some(key) = HashCache::make_key(path, algorithm) {
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
        cache.insert(key.clone(), "abc123".to_string());
        assert_eq!(cache.get(&key), Some("abc123".to_string()));

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
            cache.insert(key, format!("hash{}", i));
        }

        // Access first entry to increase its count
        let key0 = HashCache::make_key(&files[0].path().to_string_lossy(), "md5").unwrap();
        cache.get(&key0);
        cache.get(&key0);

        // Insert 4th entry - should evict entry with lowest access count
        let path3 = files[3].path().to_string_lossy().to_string();
        let key3 = HashCache::make_key(&path3, "md5").unwrap();
        cache.insert(key3, "hash3".to_string());

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
            cache.insert(key, format!("hash_{}", algo));
        }

        assert_eq!(cache.len(), 3);

        // Invalidate all entries for this path
        cache.invalidate_path(&path);

        assert_eq!(cache.len(), 0);
    }
}
