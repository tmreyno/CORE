// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # AD1 Information Display V2 - Based on libad1
//!
//! Information display functionality matching libad1_printer.c implementation

use serde::Serialize;
use std::path::Path;

use super::reader_v2::{ItemHeader, SessionV2};
use super::types::*;

/// Detailed AD1 container information
#[derive(Debug, Clone, Serialize)]
pub struct Ad1InfoV2 {
    /// Segment header information
    pub segment_header: SegmentHeaderInfo,
    /// Logical header information
    pub logical_header: LogicalHeaderInfo,
    /// Total number of items in container
    pub total_items: u64,
    /// Total size of all files (decompressed)
    pub total_size: u64,
    /// Number of files
    pub file_count: u64,
    /// Number of directories
    pub dir_count: u64,
    /// Tree structure (optional, can be large)
    pub tree: Option<Vec<TreeItem>>,
}

/// Tree item for display
#[derive(Debug, Clone, Serialize)]
pub struct TreeItem {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub depth: usize,
    pub path: String,
    pub children: Option<Vec<TreeItem>>,
}

/// Accumulator for tree statistics
#[derive(Default)]
struct TreeStats {
    total_items: u64,
    total_size: u64,
    file_count: u64,
    dir_count: u64,
}

/// Get comprehensive container information
///
/// Based on libad1's info functionality
pub fn get_container_info<P: AsRef<Path>>(
    path: P,
    include_tree: bool,
) -> Result<Ad1InfoV2, Ad1Error> {
    let session = SessionV2::open(path)?;

    let segment_header = session.segment_header.clone();
    let logical_header = session.logical_header.clone();

    // Count items and calculate statistics
    let mut stats = TreeStats::default();

    let tree = if include_tree {
        let first_item_addr = session.logical_header.first_item_addr;
        if first_item_addr != 0 {
            let root_item = session.read_item_at(first_item_addr)?;
            let mut tree_items = Vec::new();

            build_tree_item(&session, &root_item, "", 0, &mut tree_items, &mut stats)?;

            Some(tree_items)
        } else {
            None
        }
    } else {
        // Just count without building tree
        let first_item_addr = session.logical_header.first_item_addr;
        if first_item_addr != 0 {
            let root_item = session.read_item_at(first_item_addr)?;
            count_items(&session, &root_item, &mut stats)?;
        }
        None
    };

    Ok(Ad1InfoV2 {
        segment_header,
        logical_header,
        total_items: stats.total_items,
        total_size: stats.total_size,
        file_count: stats.file_count,
        dir_count: stats.dir_count,
        tree,
    })
}

/// Build tree item structure recursively
fn build_tree_item(
    session: &SessionV2,
    item: &ItemHeader,
    parent_path: &str,
    depth: usize,
    tree_items: &mut Vec<TreeItem>,
    stats: &mut TreeStats,
) -> Result<(), Ad1Error> {
    stats.total_items += 1;

    let is_dir = item.item_type == 0x05;
    let item_path = if parent_path.is_empty() {
        item.name.clone()
    } else {
        format!("{}/{}", parent_path, item.name)
    };

    if is_dir {
        stats.dir_count += 1;
    } else {
        stats.file_count += 1;
        stats.total_size += item.decompressed_size;
    }

    // Get children if directory
    let children = if is_dir && item.first_child_addr != 0 {
        let child_items = session.read_children_at(item.first_child_addr)?;
        let mut children_tree = Vec::new();

        for child in child_items {
            build_tree_item(
                session,
                &child,
                &item_path,
                depth + 1,
                &mut children_tree,
                stats,
            )?;
        }

        Some(children_tree)
    } else {
        None
    };

    tree_items.push(TreeItem {
        name: item.name.clone(),
        is_dir,
        size: item.decompressed_size,
        depth,
        path: item_path,
        children,
    });

    Ok(())
}

/// Count items without building tree (faster)
fn count_items(
    session: &SessionV2,
    item: &ItemHeader,
    stats: &mut TreeStats,
) -> Result<(), Ad1Error> {
    stats.total_items += 1;

    let is_dir = item.item_type == 0x05;
    if is_dir {
        stats.dir_count += 1;
    } else {
        stats.file_count += 1;
        stats.total_size += item.decompressed_size;
    }

    // Count children if directory
    if is_dir && item.first_child_addr != 0 {
        let child_items = session.read_children_at(item.first_child_addr)?;
        for child in child_items {
            count_items(session, &child, stats)?;
        }
    }

    Ok(())
}
