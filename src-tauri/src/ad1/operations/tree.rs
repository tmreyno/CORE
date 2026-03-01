// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AD1 tree navigation and data reading operations.

use super::super::parser::Session;
use super::super::types::{Item, TreeEntry};
use super::super::utils::*;
use crate::containers::ContainerError;

// =============================================================================
// Tree Navigation Functions
// =============================================================================

/// Get the full tree of items in the container
#[must_use = "this returns the tree entries, which should be used"]
pub fn get_tree(path: &str) -> Result<Vec<TreeEntry>, ContainerError> {
    let session = Session::open(path)?;
    let mut entries = Vec::new();
    collect_tree(&session.root_items, "", &mut entries);
    Ok(entries)
}

/// Get children at a specific path in the container
#[must_use = "this returns the children entries, which should be used"]
pub fn get_children(path: &str, parent_path: &str) -> Result<Vec<TreeEntry>, ContainerError> {
    tracing::debug!("get_children: path={}, parent_path={}", path, parent_path);
    let session = Session::open(path)?;
    let mut entries = Vec::new();
    collect_children_at_path(&session.root_items, parent_path, "", &mut entries);
    tracing::debug!("get_children: found {} entries", entries.len());
    Ok(entries)
}

/// Get children at a specific address using lazy loading
/// This is FAST - it only reads the items at the specified address without
/// loading the entire container tree into memory
#[must_use = "this returns the children entries, which should be used"]
pub fn get_children_at_addr_lazy(
    path: &str,
    addr: u64,
    parent_path: &str,
) -> Result<Vec<TreeEntry>, ContainerError> {
    tracing::debug!(
        "get_children_at_addr_lazy: path={}, addr=0x{:x}, parent_path={}",
        path,
        addr,
        parent_path
    );

    let mut session = Session::open_lazy(path)?;

    let target_addr = if addr == 0 {
        // Return root items
        session.first_item_addr()
    } else {
        addr
    };

    let items = session.read_children_lazy(target_addr)?;
    tracing::debug!("get_children_at_addr_lazy: found {} items", items.len());

    let entries: Vec<_> = items
        .iter()
        .map(|item| build_tree_entry_lazy(item, parent_path))
        .collect();

    Ok(entries)
}

/// Get children at a specific address (for lazy loading)
/// This is more efficient than path-based lookup when navigating the tree
#[must_use = "this returns the children entries, which should be used"]
pub fn get_children_at_addr(path: &str, addr: u64) -> Result<Vec<TreeEntry>, ContainerError> {
    tracing::debug!("get_children_at_addr: path={}, addr={}", path, addr);
    let session = Session::open(path)?;

    if addr == 0 {
        // Return root items
        let entries: Vec<_> = session
            .root_items
            .iter()
            .map(|item| build_tree_entry(item, "", true))
            .collect();
        tracing::debug!(
            "get_children_at_addr: addr=0, returning {} root items",
            entries.len()
        );
        return Ok(entries);
    }

    // Find item at address using utility function
    fn find_by_addr<'a>(
        items: &'a [Item],
        addr: u64,
        parent_path: &str,
    ) -> Option<(&'a Item, String)> {
        for item in items {
            let item_path = join_path(parent_path, &item.name);
            if item.zlib_metadata_addr == addr {
                return Some((item, item_path));
            }
            if let Some(found) = find_by_addr(&item.children, addr, &item_path) {
                return Some(found);
            }
        }
        None
    }

    let (item, item_path) = find_by_addr(&session.root_items, addr, "").ok_or_else(|| {
        ContainerError::EntryNotFound(format!("Item not found at address {}", addr))
    })?;

    Ok(item
        .children
        .iter()
        .map(|child| build_tree_entry(child, &item_path, true))
        .collect())
}

/// Get entry information (metadata) by path
#[must_use = "this returns the entry info, which should be used"]
pub fn get_entry_info(path: &str, entry_path: &str) -> Result<TreeEntry, ContainerError> {
    let session = Session::open(path)?;

    // Use unified item finder
    let found = find_item_by_path(&session.root_items, entry_path)
        .ok_or_else(|| ContainerError::EntryNotFound(entry_path.to_string()))?;

    Ok(build_tree_entry(found.item, &found.parent_path, true))
}

// =============================================================================
// Data Reading Functions
// =============================================================================

/// Read file data by path
#[must_use = "this returns the file data, which should be used"]
pub fn read_entry_data(path: &str, entry_path: &str) -> Result<Vec<u8>, ContainerError> {
    let mut session = Session::open(path)?;

    // Use unified item finder and clone to avoid borrow issues
    let found = find_item_by_path(&session.root_items, entry_path)
        .ok_or_else(|| ContainerError::EntryNotFound(entry_path.to_string()))?;
    let item = found.item.clone();

    let data = session.read_file_data(&item)?;
    Ok((*data).clone())
}

/// Read file data by address (for hex viewer)
#[must_use = "this returns the file data, which should be used"]
pub fn read_entry_data_by_addr(
    path: &str,
    data_addr: u64,
    size: u64,
) -> Result<Vec<u8>, ContainerError> {
    let mut session = Session::open(path)?;

    // Create a temporary item with the address info
    let temp_item = Item {
        id: 0,
        name: String::new(),
        item_type: 0,
        decompressed_size: size,
        zlib_metadata_addr: data_addr,
        metadata: Vec::new(),
        children: Vec::new(),
    };

    let data = session.read_file_data(&temp_item)?;
    Ok((*data).clone())
}

/// Read a chunk of file data (for large files / streaming)
#[must_use = "this returns the file data chunk, which should be used"]
pub fn read_entry_chunk(
    path: &str,
    entry_path: &str,
    offset: u64,
    size: usize,
) -> Result<Vec<u8>, ContainerError> {
    let data = read_entry_data(path, entry_path)?;
    let start = offset as usize;
    let end = start + size;

    if start >= data.len() {
        return Ok(Vec::new());
    }

    let actual_end = end.min(data.len());
    Ok(data[start..actual_end].to_vec())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::super::super::types::AD1_FOLDER_SIGNATURE;
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_ad1(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(b"ADSEGMENTEDFILE\0").unwrap();
        file.write_all(&[0u8; 496]).unwrap();
        path
    }

    #[test]
    fn test_get_tree_nonexistent() {
        let result = get_tree("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_children_nonexistent() {
        let result = get_children("/nonexistent/path/file.ad1", "/");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_children_at_addr_nonexistent() {
        let result = get_children_at_addr("/nonexistent/path/file.ad1", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_entry_info_nonexistent() {
        let result = get_entry_info("/nonexistent/path/file.ad1", "/some/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_entry_data_nonexistent() {
        let result = read_entry_data("/nonexistent/path/file.ad1", "/some/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_entry_data_invalid_path_format() {
        let temp_dir = TempDir::new().unwrap();
        let ad1_path = create_test_ad1(temp_dir.path(), "test.ad1");
        let result = read_entry_data(ad1_path.to_str().unwrap(), "");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_entry_chunk_nonexistent() {
        let result = read_entry_chunk("/nonexistent/path/file.ad1", "/some/file.txt", 0, 1024);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_tree_entry_file() {
        let item = Item {
            id: 1,
            name: "test.txt".to_string(),
            item_type: 0,
            decompressed_size: 1024,
            zlib_metadata_addr: 0,
            metadata: vec![],
            children: vec![],
        };
        let entry = build_tree_entry(&item, "/documents", true);
        assert_eq!(entry.path, "/documents/test.txt");
        assert!(!entry.is_dir);
        assert_eq!(entry.size, 1024);
        assert_eq!(entry.item_type, 0);
    }

    #[test]
    fn test_build_tree_entry_folder() {
        let item = Item {
            id: 2,
            name: "folder".to_string(),
            item_type: AD1_FOLDER_SIGNATURE,
            decompressed_size: 0,
            zlib_metadata_addr: 0,
            metadata: vec![],
            children: vec![],
        };
        let entry = build_tree_entry(&item, "", true);
        assert_eq!(entry.path, "folder");
        assert!(entry.is_dir);
        assert_eq!(entry.size, 0);
        assert_eq!(entry.item_type, AD1_FOLDER_SIGNATURE);
    }

    #[test]
    fn test_build_tree_entry_root_path() {
        let item = Item {
            id: 1,
            name: "root_file.txt".to_string(),
            item_type: 0,
            decompressed_size: 512,
            zlib_metadata_addr: 0,
            metadata: vec![],
            children: vec![],
        };
        let entry = build_tree_entry(&item, "", true);
        assert_eq!(entry.path, "root_file.txt");
    }

    #[test]
    fn test_build_tree_entry_nested_path() {
        let item = Item {
            id: 1,
            name: "deep.txt".to_string(),
            item_type: 0,
            decompressed_size: 256,
            zlib_metadata_addr: 0,
            metadata: vec![],
            children: vec![],
        };
        let entry = build_tree_entry(&item, "/a/b/c/d/e", true);
        assert_eq!(entry.path, "/a/b/c/d/e/deep.txt");
    }

    #[test]
    fn test_tree_entry_serialization() {
        let entry = TreeEntry {
            path: "/test.txt".to_string(),
            name: "test.txt".to_string(),
            is_dir: false,
            size: 1024,
            item_type: 0,
            first_child_addr: None,
            data_addr: Some(12345),
            item_addr: Some(12345),
            compressed_size: None,
            data_end_addr: None,
            metadata_addr: None,
            md5_hash: Some("d41d8cd98f00b204e9800998ecf8427e".to_string()),
            sha1_hash: None,
            created: Some("2024-01-01T00:00:00Z".to_string()),
            accessed: None,
            modified: None,
            attributes: None,
            child_count: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("test.txt"));
        assert!(json.contains("d41d8cd98f00b204e9800998ecf8427e"));
        assert!(!json.contains("sha1_hash"));
    }

    /// Integration test with real AD1 file (only runs if file exists)
    #[test]
    fn test_real_ad1_get_children() {
        let path = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/02606-0900_1E_401358/02606-0900_1E_401358_img1.ad1";
        if !std::path::Path::new(path).exists() {
            println!("Skipping test - file not found: {}", path);
            return;
        }

        println!("Testing with real AD1 file: {}", path);

        let is_valid = super::super::is_ad1(path);
        println!("is_ad1 result: {:?}", is_valid);
        assert!(is_valid.is_ok(), "is_ad1 should succeed");
        assert!(is_valid.unwrap(), "File should be recognized as AD1");

        let result = get_children(path, "");
        println!(
            "get_children result: {:?}",
            result.as_ref().map(|v| v.len())
        );

        match result {
            Ok(entries) => {
                println!("SUCCESS: Found {} root entries", entries.len());
                for (i, e) in entries.iter().enumerate().take(10) {
                    println!(
                        "  [{}] {} (dir={}, size={}, item_addr={:?})",
                        i, e.name, e.is_dir, e.size, e.item_addr
                    );
                }
            }
            Err(e) => {
                panic!("get_children failed: {}", e);
            }
        }
    }
}
