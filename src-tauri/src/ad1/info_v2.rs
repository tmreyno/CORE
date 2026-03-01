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

/// Print tree structure to string (matches libad1's print_tree)
pub fn tree_to_string(tree: &[TreeItem]) -> String {
    let mut output = String::new();

    for item in tree {
        print_tree_item(&mut output, item);
    }

    output
}

/// Recursively print tree item
fn print_tree_item(output: &mut String, item: &TreeItem) {
    // Indentation
    for _ in 0..item.depth.saturating_sub(1) {
        output.push_str("    ");
    }

    if item.depth > 0 {
        output.push_str("└───");
    }

    // Icon
    if item.is_dir {
        output.push_str("📁 ");
    } else {
        output.push_str("📄 ");
    }

    // Name and size
    output.push_str(&item.name);
    if !item.is_dir {
        output.push_str(&format!(" ({} bytes)", item.size));
    }
    output.push('\n');

    // Children
    if let Some(children) = &item.children {
        for child in children {
            print_tree_item(output, child);
        }
    }
}

/// Format segment header info for display
pub fn format_segment_header(header: &SegmentHeaderInfo) -> String {
    format!(
        "Segment Header:\n\
         Signature:      {}\n\
         Segment Index:  {}\n\
         Total Segments: {}\n\
         Fragment Size:  {}\n\
         Header Size:    {}\n",
        header.signature,
        header.segment_index,
        header.segment_number,
        header.fragments_size,
        header.header_size
    )
}

/// Format logical header info for display
pub fn format_logical_header(header: &LogicalHeaderInfo) -> String {
    format!(
        "Logical Header:\n\
         Signature:            {}\n\
         Image Version:        {}\n\
         Zlib Chunk Size:      {}\n\
         Metadata Address:     0x{:016x}\n\
         First Item Address:   0x{:016x}\n\
         Data Source Name Len: {}\n\
         AD Signature:         {}\n\
         Data Source Name Addr:0x{:016x}\n\
         ATTRGUID Footer:      0x{:016x}\n\
         LOCSGUID Footer:      0x{:016x}\n\
         Data Source Name:     {}\n",
        header.signature,
        header.image_version,
        header.zlib_chunk_size,
        header.logical_metadata_addr,
        header.first_item_addr,
        header.data_source_name_length,
        header.ad_signature,
        header.data_source_name_addr,
        header.attrguid_footer_addr,
        header.locsguid_footer_addr,
        header.data_source_name
    )
}

/// Format item header for display (debug purposes)
pub fn format_item_header(item: &ItemHeader) -> String {
    format!(
        "Item Header:\n\
         Name:               {}\n\
         Type:               {} ({})\n\
         Size:               {} bytes\n\
         Next Item:          0x{:016x}\n\
         First Child:        0x{:016x}\n\
         Metadata Address:   0x{:016x}\n\
         Zlib Data Address:  0x{:016x}\n\
         Parent Folder:      0x{:016x}\n",
        item.name,
        if item.item_type == 0x05 {
            "Directory"
        } else {
            "File"
        },
        item.item_type,
        item.decompressed_size,
        item.next_item_addr,
        item.first_child_addr,
        item.first_metadata_addr,
        item.zlib_metadata_addr,
        item.parent_folder
    )
}

/// Format complete container info
pub fn format_container_info(info: &Ad1InfoV2) -> String {
    let mut output = String::new();

    output.push_str("=== AD1 Container Information ===\n\n");
    output.push_str(&format_segment_header(&info.segment_header));
    output.push('\n');
    output.push_str(&format_logical_header(&info.logical_header));
    output.push('\n');

    output.push_str("Statistics:\n");
    output.push_str(&format!("  Total Items:  {}\n", info.total_items));
    output.push_str(&format!("  Files:        {}\n", info.file_count));
    output.push_str(&format!("  Directories:  {}\n", info.dir_count));
    output.push_str(&format!(
        "  Total Size:   {} bytes ({:.2} MB)\n",
        info.total_size,
        info.total_size as f64 / 1_048_576.0
    ));

    if let Some(tree) = &info.tree {
        output.push_str("\n=== File Tree ===\n");
        output.push_str(&tree_to_string(tree));
    }

    output
}
