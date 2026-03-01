// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! LRU Chunk Cache for E01 handle (like libfcache)

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// Chunk cache with LRU eviction
/// Uses Arc to avoid cloning large data buffers
pub(crate) struct ChunkCache {
    cache: HashMap<usize, Arc<Vec<u8>>>,
    lru_queue: VecDeque<usize>,
    max_entries: usize,
}

impl ChunkCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: HashMap::new(),
            lru_queue: VecDeque::new(),
            max_entries,
        }
    }

    pub fn get(&mut self, chunk_index: usize) -> Option<Arc<Vec<u8>>> {
        if let Some(data) = self.cache.get(&chunk_index) {
            // Move to front of LRU
            self.lru_queue.retain(|&x| x != chunk_index);
            self.lru_queue.push_front(chunk_index);
            return Some(Arc::clone(data)); // Cheap Arc clone, not data clone
        }
        None
    }

    pub fn insert(&mut self, chunk_index: usize, data: Vec<u8>) {
        // Remove oldest if at capacity
        if self.cache.len() >= self.max_entries {
            if let Some(old_index) = self.lru_queue.pop_back() {
                self.cache.remove(&old_index);
            }
        }

        self.cache.insert(chunk_index, Arc::new(data));
        self.lru_queue.push_front(chunk_index);
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new() {
        let cache = ChunkCache::new(10);
        assert_eq!(cache.max_entries, 10);
        assert!(cache.cache.is_empty());
        assert!(cache.lru_queue.is_empty());
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = ChunkCache::new(10);

        cache.insert(0, vec![1, 2, 3, 4]);
        cache.insert(1, vec![5, 6, 7, 8]);

        let data0 = cache.get(0);
        assert!(data0.is_some());
        assert_eq!(data0.unwrap().as_slice(), &[1, 2, 3, 4]);

        let data1 = cache.get(1);
        assert!(data1.is_some());
        assert_eq!(data1.unwrap().as_slice(), &[5, 6, 7, 8]);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = ChunkCache::new(10);
        cache.insert(0, vec![1, 2, 3]);

        let data = cache.get(99);
        assert!(data.is_none());
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = ChunkCache::new(3);

        // Insert 3 items (fills cache)
        cache.insert(0, vec![0]);
        cache.insert(1, vec![1]);
        cache.insert(2, vec![2]);

        // Access 0 to make it recently used
        let _ = cache.get(0);

        // Insert 4th item - should evict item 1 (oldest unused)
        cache.insert(3, vec![3]);

        // Item 0 should still exist (was recently accessed)
        assert!(cache.get(0).is_some());
        // Item 1 should be evicted
        assert!(cache.get(1).is_none());
        // Item 2 should still exist
        assert!(cache.get(2).is_some());
        // Item 3 should exist
        assert!(cache.get(3).is_some());
    }

    #[test]
    fn test_cache_lru_order() {
        let mut cache = ChunkCache::new(3);

        cache.insert(0, vec![0]);
        cache.insert(1, vec![1]);
        cache.insert(2, vec![2]);

        // Access in order: 2, 0, 1 - so LRU should be: 1, 0, 2
        let _ = cache.get(2);
        let _ = cache.get(0);
        let _ = cache.get(1);

        // Insert new item - should evict 2 (least recently used)
        cache.insert(3, vec![3]);

        assert!(cache.get(2).is_none()); // Evicted
        assert!(cache.get(0).is_some());
        assert!(cache.get(1).is_some());
        assert!(cache.get(3).is_some());
    }

    #[test]
    fn test_cache_overwrite() {
        let mut cache = ChunkCache::new(10);

        cache.insert(0, vec![1, 2, 3]);
        cache.insert(0, vec![4, 5, 6]);

        let data = cache.get(0);
        assert!(data.is_some());
        assert_eq!(data.unwrap().as_slice(), &[4, 5, 6]);
    }

    #[test]
    fn test_cache_arc_sharing() {
        let mut cache = ChunkCache::new(10);
        cache.insert(0, vec![1, 2, 3, 4]);

        // Get multiple references
        let ref1 = cache.get(0).unwrap();
        let ref2 = cache.get(0).unwrap();

        // Both should point to same data
        assert!(Arc::ptr_eq(&ref1, &ref2));
    }

    #[test]
    fn test_cache_capacity_one() {
        let mut cache = ChunkCache::new(1);

        cache.insert(0, vec![0]);
        assert!(cache.get(0).is_some());

        cache.insert(1, vec![1]);
        assert!(cache.get(0).is_none()); // Evicted
        assert!(cache.get(1).is_some());
    }
}
