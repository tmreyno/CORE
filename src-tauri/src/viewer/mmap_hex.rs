// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Memory-Mapped Hex Viewer with LRU Caching
//! 
//! Provides efficient hex viewing for large files using:
//! - Memory-mapped I/O for zero-copy access
//! - LRU cache for recently accessed pages
//! - Page-based windowing (64KB pages)
//! - Lazy loading of only visible + adjacent pages

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, RwLock};
use lru::LruCache;
use memmap2::Mmap;
use std::num::NonZeroUsize;

const PAGE_SIZE: usize = 64 * 1024; // 64KB pages
const MAX_CACHED_PAGES: usize = 256; // ~16MB cache
const ADJACENT_PAGES: usize = 2; // Pre-load 2 pages before and after visible

#[derive(Clone)]
pub struct HexViewPage {
    pub page_index: usize,
    pub offset: u64,
    pub data: Vec<u8>,
    pub size: usize,
}

pub struct MmapHexViewer {
    file_cache: Arc<RwLock<HashMap<String, Arc<Mmap>>>>,
    page_cache: Arc<RwLock<LruCache<(String, usize), Arc<HexViewPage>>>>,
}

impl MmapHexViewer {
    pub fn new() -> Self {
        Self {
            file_cache: Arc::new(RwLock::new(HashMap::new())),
            page_cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(MAX_CACHED_PAGES).unwrap())
            )),
        }
    }

    /// Open a file for memory-mapped access
    pub fn open_file(&self, path: &str) -> Result<Arc<Mmap>, String> {
        // Check cache first
        {
            let cache = self.file_cache.read().map_err(|e| e.to_string())?;
            if let Some(mmap) = cache.get(path) {
                return Ok(Arc::clone(mmap));
            }
        }

        // Open and map file
        let file = File::open(Path::new(path))
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        // Safety: File is read-only and remains open while mapped
        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| format!("Failed to memory-map file: {}", e))?;
        
        let mmap_arc = Arc::new(mmap);
        
        // Store in cache
        let mut cache = self.file_cache.write().map_err(|e| e.to_string())?;
        cache.insert(path.to_string(), Arc::clone(&mmap_arc));
        
        Ok(mmap_arc)
    }

    /// Get a page of data from the file
    pub fn get_page(&self, path: &str, page_index: usize) -> Result<Arc<HexViewPage>, String> {
        let cache_key = (path.to_string(), page_index);

        // Check page cache
        {
            let mut cache = self.page_cache.write().map_err(|e| e.to_string())?;
            if let Some(page) = cache.get(&cache_key) {
                return Ok(Arc::clone(page));
            }
        }

        // Load page from mmap
        let mmap = self.open_file(path)?;
        let file_size = mmap.len();
        let offset = (page_index * PAGE_SIZE) as u64;

        if offset >= file_size as u64 {
            return Err("Page offset beyond file size".to_string());
        }

        let start = offset as usize;
        let end = ((offset as usize) + PAGE_SIZE).min(file_size);
        let size = end - start;
        
        let data = mmap[start..end].to_vec();

        let page = Arc::new(HexViewPage {
            page_index,
            offset,
            data,
            size,
        });

        // Store in cache
        let mut cache = self.page_cache.write().map_err(|e| e.to_string())?;
        cache.put(cache_key, Arc::clone(&page));

        Ok(page)
    }

    /// Get multiple pages (visible + adjacent) for efficient scrolling
    pub fn get_pages_window(
        &self,
        path: &str,
        center_page: usize,
        visible_pages: usize,
    ) -> Result<Vec<Arc<HexViewPage>>, String> {
        let mmap = self.open_file(path)?;
        let file_size = mmap.len();
        let total_pages = (file_size + PAGE_SIZE - 1) / PAGE_SIZE;

        let start_page = center_page.saturating_sub(ADJACENT_PAGES);
        let end_page = (center_page + visible_pages + ADJACENT_PAGES).min(total_pages);

        let mut pages = Vec::new();
        for page_idx in start_page..end_page {
            match self.get_page(path, page_idx) {
                Ok(page) => pages.push(page),
                Err(e) => eprintln!("Failed to load page {}: {}", page_idx, e),
            }
        }

        Ok(pages)
    }

    /// Get file size
    pub fn get_file_size(&self, path: &str) -> Result<u64, String> {
        let mmap = self.open_file(path)?;
        Ok(mmap.len() as u64)
    }

    /// Clear file from cache (call when file is closed)
    pub fn close_file(&self, path: &str) -> Result<(), String> {
        let mut file_cache = self.file_cache.write().map_err(|e| e.to_string())?;
        file_cache.remove(path);

        // Clear all pages for this file
        let mut page_cache = self.page_cache.write().map_err(|e| e.to_string())?;
        // LRU doesn't have a remove_if, so we recreate the cache (acceptable since close is rare)
        let keys_to_remove: Vec<_> = page_cache
            .iter()
            .filter(|(k, _)| k.0 == path)
            .map(|(k, _)| k.clone())
            .collect();
        
        for key in keys_to_remove {
            page_cache.pop(&key);
        }

        Ok(())
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> Result<(usize, usize, usize), String> {
        let file_cache = self.file_cache.read().map_err(|e| e.to_string())?;
        let page_cache = self.page_cache.read().map_err(|e| e.to_string())?;
        
        Ok((
            file_cache.len(),
            page_cache.len(),
            PAGE_SIZE * page_cache.len(),
        ))
    }

    /// Clear all caches
    pub fn clear_caches(&self) -> Result<(), String> {
        let mut file_cache = self.file_cache.write().map_err(|e| e.to_string())?;
        let mut page_cache = self.page_cache.write().map_err(|e| e.to_string())?;
        
        file_cache.clear();
        page_cache.clear();
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_mmap_viewer_basic() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let data: Vec<u8> = (0..=255u8).cycle().take(128 * 1024).collect(); // 128KB
        temp_file.write_all(&data).unwrap();
        temp_file.flush().unwrap();
        
        let viewer = MmapHexViewer::new();
        let path = temp_file.path().to_str().unwrap();
        
        // Test file size
        let size = viewer.get_file_size(path).unwrap();
        assert_eq!(size, 128 * 1024);
        
        // Test page access
        let page0 = viewer.get_page(path, 0).unwrap();
        assert_eq!(page0.page_index, 0);
        assert_eq!(page0.size, PAGE_SIZE);
        assert_eq!(page0.data[0], 0);
        assert_eq!(page0.data[255], 255);
        
        let page1 = viewer.get_page(path, 1).unwrap();
        assert_eq!(page1.page_index, 1);
        assert_eq!(page1.size, PAGE_SIZE);
        
        // Test cache hit - just verify we get the same page back
        let page0_cached = viewer.get_page(path, 0).unwrap();
        assert_eq!(page0_cached.page_index, 0);
        assert_eq!(page0_cached.size, PAGE_SIZE);
    }

    #[test]
    fn test_mmap_viewer_window() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let data: Vec<u8> = (0..=255u8).cycle().take(256 * 1024).collect(); // 256KB = 4 pages
        temp_file.write_all(&data).unwrap();
        temp_file.flush().unwrap();
        
        let viewer = MmapHexViewer::new();
        let path = temp_file.path().to_str().unwrap();
        
        // Get window around page 1 (should include pages 0-3 with ADJACENT_PAGES=2)
        let pages = viewer.get_pages_window(path, 1, 1).unwrap();
        assert!(pages.len() >= 2); // At least visible + some adjacent
        
        // Verify page order
        assert_eq!(pages[0].page_index, 0);
    }

    #[test]
    fn test_mmap_viewer_cache_stats() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let data = vec![0u8; 64 * 1024];
        temp_file.write_all(&data).unwrap();
        temp_file.flush().unwrap();
        
        let viewer = MmapHexViewer::new();
        let path = temp_file.path().to_str().unwrap();
        
        let _ = viewer.get_page(path, 0).unwrap();
        
        let (files, pages, bytes) = viewer.get_cache_stats().unwrap();
        assert_eq!(files, 1);
        assert_eq!(pages, 1);
        assert_eq!(bytes, PAGE_SIZE);
        
        viewer.clear_caches().unwrap();
        let (files, pages, _) = viewer.get_cache_stats().unwrap();
        assert_eq!(files, 0);
        assert_eq!(pages, 0);
    }
}
