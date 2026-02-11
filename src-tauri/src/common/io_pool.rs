// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Shared File I/O Pool for managing multiple segment file handles
//
// Provides LRU caching for file handles when working with multi-segment
// forensic images (E01, RAW, etc.) to avoid exceeding OS file descriptor limits.

use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::path::PathBuf;
use tracing::{debug, trace};

use crate::containers::ContainerError;

/// Default maximum number of simultaneously open file handles
pub const DEFAULT_MAX_OPEN_FILES: usize = 32;

// =============================================================================
// File I/O Pool - Like libbfio_pool
// =============================================================================

/// Manages multiple file handles with LRU caching
/// Limits number of simultaneously open files to avoid OS limits
pub struct FileIoPool {
    /// Paths to all segment files in order
    file_paths: Vec<PathBuf>,
    /// Currently open file handles (file_index -> File)
    open_handles: HashMap<usize, File>,
    /// LRU queue for file handle management
    lru_queue: VecDeque<usize>,
    /// Maximum number of simultaneously open files
    max_open: usize,
}

impl FileIoPool {
    /// Create a new file pool with specified paths and max open limit
    pub fn new(file_paths: Vec<PathBuf>, max_open: usize) -> Self {
        Self {
            file_paths,
            open_handles: HashMap::new(),
            lru_queue: VecDeque::new(),
            max_open,
        }
    }

    /// Create a new file pool with default max open limit
    pub fn with_default_limit(file_paths: Vec<PathBuf>) -> Self {
        Self::new(file_paths, DEFAULT_MAX_OPEN_FILES)
    }

    /// Get a file handle, opening it if necessary and managing LRU cache
    pub fn get_file(&mut self, file_index: usize) -> Result<&mut File, ContainerError> {
        if file_index >= self.file_paths.len() {
            return Err(ContainerError::SegmentError(format!(
                "File index {} out of range (have {} files)",
                file_index,
                self.file_paths.len()
            )));
        }

        // If file is already open, move to front of LRU queue
        if self.open_handles.contains_key(&file_index) {
            // Remove from current position in LRU
            self.lru_queue.retain(|&x| x != file_index);
            // Add to front
            self.lru_queue.push_front(file_index);
            trace!(file_index, "File handle cache hit");
            return Ok(self.open_handles.get_mut(&file_index).expect("contains_key was true"));
        }

        // Need to open the file - check if we need to close one first
        if self.open_handles.len() >= self.max_open {
            // Close least recently used file
            if let Some(lru_index) = self.lru_queue.pop_back() {
                trace!(lru_index, "Evicting LRU file handle");
                self.open_handles.remove(&lru_index);
            }
        }

        // Open the new file
        let file_path = &self.file_paths[file_index];
        debug!(file_index, ?file_path, "Opening file handle");
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open segment {}: {}", file_index, e))?;

        self.open_handles.insert(file_index, file);
        self.lru_queue.push_front(file_index);

        Ok(self.open_handles.get_mut(&file_index).expect("just inserted"))
    }

    /// Get the number of files in the pool
    pub fn get_file_count(&self) -> usize {
        self.file_paths.len()
    }

    /// Get the path for a specific file index
    pub fn get_path(&self, file_index: usize) -> Option<&PathBuf> {
        self.file_paths.get(file_index)
    }

    /// Get all file paths
    pub fn get_paths(&self) -> &[PathBuf] {
        &self.file_paths
    }

    /// Get the number of currently open handles
    pub fn open_count(&self) -> usize {
        self.open_handles.len()
    }

    /// Close all open file handles
    pub fn close_all(&mut self) {
        self.open_handles.clear();
        self.lru_queue.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_file_pool_basic() {
        let temp_dir = TempDir::new().unwrap();
        let mut paths = Vec::new();
        
        // Create test files
        for i in 0..5 {
            let path = temp_dir.path().join(format!("test_{}.bin", i));
            let mut file = File::create(&path).unwrap();
            file.write_all(&[i as u8; 100]).unwrap();
            paths.push(path);
        }
        
        let mut pool = FileIoPool::new(paths, 3);
        
        assert_eq!(pool.get_file_count(), 5);
        assert_eq!(pool.open_count(), 0);
        
        // Open first 3 files
        pool.get_file(0).unwrap();
        pool.get_file(1).unwrap();
        pool.get_file(2).unwrap();
        assert_eq!(pool.open_count(), 3);
        
        // Opening a 4th should evict the LRU (file 0)
        pool.get_file(3).unwrap();
        assert_eq!(pool.open_count(), 3);
        
        // Re-opening file 1 should work (still cached)
        pool.get_file(1).unwrap();
        assert_eq!(pool.open_count(), 3);
    }

    #[test]
    fn test_file_pool_out_of_range() {
        let pool_paths: Vec<PathBuf> = vec![];
        let mut pool = FileIoPool::new(pool_paths, 3);
        
        assert!(pool.get_file(0).is_err());
    }
}
