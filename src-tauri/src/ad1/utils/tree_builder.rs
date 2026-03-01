// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # AD1 Tree Building Logic
//!
//! ## Section Brief
//! Functions for building and navigating the AD1 item tree:
//!
//! ### Item Finding
//! - `find_item_by_path()` - Find an item by path in the item tree
//! - `find_children_at_path()` - Find children of an item at a specific path
//! - `FoundItem` - Result struct for item search
//!
//! ### TreeEntry Construction
//! - `build_tree_entry()` - Convert an Item to a TreeEntry with full metadata
//! - `build_tree_entry_lazy()` - Convert a lazy-loaded Item to a TreeEntry
//!
//! ### Tree Collection
//! - `collect_tree()` - Collect tree entries recursively with full metadata
//! - `collect_tree_simple()` - Collect tree entries without metadata (faster)
//! - `collect_children_at_path()` - Collect children at a specific path
//! - `count_files()` - Count total files (non-folders) in item tree

use crate::ad1::types::*;

use super::join_path;
use super::parsing::{extract_attributes, find_hash, find_timestamp};

// =============================================================================
// Unified Item Finding
// =============================================================================

/// Result of finding an item in the tree
#[derive(Debug)]
pub struct FoundItem<'a> {
    /// The found item
    pub item: &'a Item,
    /// Path to the parent of this item
    pub parent_path: String,
    /// Full path to this item (may be useful for debugging)
    #[allow(dead_code)]
    pub full_path: String,
}

/// Find an item by path in the item tree
///
/// # Arguments
/// * `items` - Root items to search
/// * `target_path` - Path to find (e.g., "/folder/file.txt")
///
/// # Returns
/// Some(FoundItem) if found, None otherwise
pub fn find_item_by_path<'a>(items: &'a [Item], target_path: &str) -> Option<FoundItem<'a>> {
    fn search<'a>(items: &'a [Item], target: &str, current_path: &str) -> Option<FoundItem<'a>> {
        for item in items {
            let item_path = join_path(current_path, &item.name);
            if item_path == target {
                return Some(FoundItem {
                    item,
                    parent_path: current_path.to_string(),
                    full_path: item_path,
                });
            }
            // If target is deeper, recurse
            if target.starts_with(&item_path) && !item.children.is_empty() {
                if let Some(found) = search(&item.children, target, &item_path) {
                    return Some(found);
                }
            }
        }
        None
    }

    // Normalize: strip leading slash since join_path builds paths without leading slashes
    let target = target_path.trim_start_matches('/');
    if target.is_empty() {
        return None;
    }
    search(items, target, "")
}

/// Find children of an item at a specific path
///
/// # Arguments
/// * `items` - Root items to search
/// * `parent_path` - Parent path (empty or "/" for root)
///
/// # Returns
/// Slice of child items, or root items if parent_path is empty/root
#[allow(dead_code)]
pub fn find_children_at_path<'a>(items: &'a [Item], parent_path: &str) -> Option<&'a [Item]> {
    let normalized = parent_path.trim_matches('/');

    if normalized.is_empty() {
        return Some(items);
    }

    // Navigate to the parent and return its children
    if let Some(found) = find_item_by_path(items, parent_path) {
        Some(&found.item.children)
    } else {
        None
    }
}

// =============================================================================
// TreeEntry Construction
// =============================================================================

/// Convert an Item to a TreeEntry with full metadata
///
/// # Arguments
/// * `item` - The item to convert
/// * `parent_path` - Path to the parent of this item
/// * `include_metadata` - If true, extract hashes and timestamps
pub fn build_tree_entry(item: &Item, parent_path: &str, include_metadata: bool) -> TreeEntry {
    let path = join_path(parent_path, &item.name);
    let is_dir = item.item_type == AD1_FOLDER_SIGNATURE;
    let size = if is_dir { 0 } else { item.decompressed_size };

    // Address fields for lazy loading and hex view
    let addr = if item.zlib_metadata_addr > 0 {
        Some(item.zlib_metadata_addr)
    } else {
        None
    };
    let first_child_addr = if is_dir && !item.children.is_empty() {
        addr
    } else {
        None
    };
    let data_addr = if !is_dir { addr } else { None };
    let child_count = if is_dir {
        Some(item.children.len())
    } else {
        None
    };

    if include_metadata {
        // Extract metadata (hashes for files, timestamps for both)
        let (md5_hash, sha1_hash, attributes) = if !is_dir {
            (
                find_hash(&item.metadata, MD5_HASH),
                find_hash(&item.metadata, SHA1_HASH),
                extract_attributes(&item.metadata),
            )
        } else {
            (None, None, None)
        };

        TreeEntry {
            path,
            name: item.name.clone(),
            is_dir,
            size,
            item_type: item.item_type,
            first_child_addr,
            data_addr,
            item_addr: addr,
            compressed_size: None,
            data_end_addr: None,
            metadata_addr: None,
            md5_hash,
            sha1_hash,
            created: find_timestamp(&item.metadata, CREATED),
            accessed: find_timestamp(&item.metadata, ACCESS),
            modified: find_timestamp(&item.metadata, MODIFIED),
            attributes,
            child_count,
        }
    } else {
        TreeEntry {
            path,
            name: item.name.clone(),
            is_dir,
            size,
            item_type: item.item_type,
            first_child_addr,
            data_addr,
            item_addr: addr,
            compressed_size: None,
            data_end_addr: None,
            metadata_addr: None,
            md5_hash: None,
            sha1_hash: None,
            created: None,
            accessed: None,
            modified: None,
            attributes: None,
            child_count,
        }
    }
}

/// Convert an Item from lazy loading to a TreeEntry
/// For lazy-loaded items, zlib_metadata_addr contains first_child_addr for folders
///
/// # Arguments
/// * `item` - The item to convert (from read_item_shallow)
/// * `parent_path` - Path to the parent of this item
pub fn build_tree_entry_lazy(item: &Item, parent_path: &str) -> TreeEntry {
    let path = join_path(parent_path, &item.name);
    let is_dir = item.item_type == AD1_FOLDER_SIGNATURE;
    let size = if is_dir { 0 } else { item.decompressed_size };

    // For lazy-loaded items:
    // - Folders: zlib_metadata_addr holds first_child_addr
    // - Files: zlib_metadata_addr holds actual zlib metadata address
    let addr = if item.zlib_metadata_addr > 0 {
        Some(item.zlib_metadata_addr)
    } else {
        None
    };
    let first_child_addr = if is_dir { addr } else { None };
    let data_addr = if !is_dir { addr } else { None };

    // Extract metadata (hashes for files, timestamps for both)
    let (md5_hash, sha1_hash, attributes) = if !is_dir {
        (
            find_hash(&item.metadata, MD5_HASH),
            find_hash(&item.metadata, SHA1_HASH),
            extract_attributes(&item.metadata),
        )
    } else {
        (None, None, None)
    };

    TreeEntry {
        path,
        name: item.name.clone(),
        is_dir,
        size,
        item_type: item.item_type,
        first_child_addr,
        data_addr,
        item_addr: addr,
        compressed_size: None,
        data_end_addr: None,
        metadata_addr: None,
        md5_hash,
        sha1_hash,
        created: find_timestamp(&item.metadata, CREATED),
        accessed: find_timestamp(&item.metadata, ACCESS),
        modified: find_timestamp(&item.metadata, MODIFIED),
        attributes,
        child_count: None, // Unknown for lazy-loaded items
    }
}

// =============================================================================
// Tree Collection
// =============================================================================

/// Collect tree entries recursively with full metadata
pub fn collect_tree(items: &[Item], parent_path: &str, out: &mut Vec<TreeEntry>) {
    for item in items {
        let entry = build_tree_entry(item, parent_path, true);
        let path = entry.path.clone();
        out.push(entry);
        collect_tree(&item.children, &path, out);
    }
}

/// Collect tree entries without metadata (faster, for tree display only)
#[allow(dead_code)]
pub fn collect_tree_simple(items: &[Item], parent_path: &str, out: &mut Vec<TreeEntry>) {
    for item in items {
        let entry = build_tree_entry(item, parent_path, false);
        let path = entry.path.clone();
        out.push(entry);
        collect_tree_simple(&item.children, &path, out);
    }
}

/// Collect children at a specific path (for lazy loading)
/// If parent_path is empty or "/", returns root items
pub fn collect_children_at_path(
    items: &[Item],
    target_path: &str,
    current_path: &str,
    out: &mut Vec<TreeEntry>,
) {
    // Normalize target path
    let target = target_path.trim_matches('/');

    if target.is_empty() {
        // Return root-level items
        for item in items {
            let path = join_path(current_path, &item.name);
            let is_dir = item.item_type == AD1_FOLDER_SIGNATURE;
            let size = if is_dir { 0 } else { item.decompressed_size };
            let addr = if item.zlib_metadata_addr > 0 {
                Some(item.zlib_metadata_addr)
            } else {
                None
            };
            let first_child_addr = if is_dir && !item.children.is_empty() {
                addr
            } else {
                None
            };
            let data_addr = if !is_dir { addr } else { None };
            out.push(TreeEntry {
                path,
                name: item.name.clone(),
                is_dir,
                size,
                item_type: item.item_type,
                first_child_addr,
                data_addr,
                item_addr: addr,
                compressed_size: None,
                data_end_addr: None,
                metadata_addr: None,
                md5_hash: find_hash(&item.metadata, MD5_HASH),
                sha1_hash: find_hash(&item.metadata, SHA1_HASH),
                created: find_timestamp(&item.metadata, CREATED),
                accessed: find_timestamp(&item.metadata, ACCESS),
                modified: find_timestamp(&item.metadata, MODIFIED),
                attributes: None,
                child_count: if is_dir {
                    Some(item.children.len())
                } else {
                    None
                },
            });
        }
        return;
    }

    // Navigate to the target path and return its children
    for item in items {
        let item_path = join_path(current_path, &item.name);
        let relative = item_path.trim_start_matches('/');

        if relative == target {
            // Found the target - return its children
            for child in &item.children {
                let child_path = join_path(&item_path, &child.name);
                let is_dir = child.item_type == AD1_FOLDER_SIGNATURE;
                let size = if is_dir { 0 } else { child.decompressed_size };
                let addr = if child.zlib_metadata_addr > 0 {
                    Some(child.zlib_metadata_addr)
                } else {
                    None
                };
                let first_child_addr = if is_dir && !child.children.is_empty() {
                    addr
                } else {
                    None
                };
                let data_addr = if !is_dir { addr } else { None };
                out.push(TreeEntry {
                    path: child_path,
                    name: child.name.clone(),
                    is_dir,
                    size,
                    item_type: child.item_type,
                    first_child_addr,
                    data_addr,
                    item_addr: addr,
                    compressed_size: None,
                    data_end_addr: None,
                    metadata_addr: None,
                    md5_hash: find_hash(&child.metadata, MD5_HASH),
                    sha1_hash: find_hash(&child.metadata, SHA1_HASH),
                    created: find_timestamp(&child.metadata, CREATED),
                    accessed: find_timestamp(&child.metadata, ACCESS),
                    modified: find_timestamp(&child.metadata, MODIFIED),
                    attributes: None,
                    child_count: if is_dir {
                        Some(child.children.len())
                    } else {
                        None
                    },
                });
            }
            return;
        } else if target.starts_with(relative) && !item.children.is_empty() {
            // Target is deeper - recurse
            collect_children_at_path(&item.children, target_path, &item_path, out);
            return;
        }
    }
}

/// Count total files (non-folders) in item tree
pub fn count_files(items: &[Item]) -> u64 {
    items
        .iter()
        .map(|item| {
            let self_count: u64 = if item.item_type != AD1_FOLDER_SIGNATURE {
                1
            } else {
                0
            };
            self_count + count_files(&item.children)
        })
        .sum()
}
