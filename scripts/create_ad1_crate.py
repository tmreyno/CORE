#!/usr/bin/env python3
"""Create ffx-ad1 crate by copying files and fixing imports."""

import os

BASE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(BASE, "src-tauri", "src", "ad1")
DST = os.path.join(BASE, "crates", "ffx-ad1", "src")

# Files to copy (relative to ad1/)
FILES = [
    "types.rs",
    "parser.rs",
    "reader_v2.rs",
    "info_v2.rs",
    "operations_v2.rs",
    "vfs.rs",
    "operations/mod.rs",
    "operations/export.rs",
    "operations/search.rs",
    "operations/tree.rs",
    "operations/verify.rs",
    "utils/mod.rs",
    "utils/parsing.rs",
    "utils/tree_builder.rs",
]

def fix_imports(content: str, filepath: str) -> str:
    """Replace crate:: imports with ffx_common:: or ffx_errors:: equivalents."""
    
    # Replace crate::common:: with ffx_common::
    content = content.replace("use crate::common::binary::", "use ffx_common::binary::")
    content = content.replace("use crate::common::hash::", "use ffx_common::hash::")
    content = content.replace("use crate::common::segments::", "use ffx_common::segments::")
    content = content.replace("use crate::common::escape_csv", "use ffx_common::escape_csv")
    content = content.replace("use crate::common::vfs::", "use ffx_common::vfs::")
    content = content.replace("use crate::common::{AdaptiveBuffer", "use ffx_common::{AdaptiveBuffer")
    
    # Replace crate::containers::ContainerError with ffx_errors::ContainerError
    content = content.replace(
        "use crate::containers::ContainerError;",
        "use ffx_errors::ContainerError;"
    )
    
    # Replace crate::ad1:: with crate:: (internal refs)
    content = content.replace("use crate::ad1::", "use crate::")
    
    # Handle the companion log import specially
    # Replace it with an #[allow(unused)] marker and a conditional compile or just remove it
    # We'll make it a cfg-gated optional dependency
    content = content.replace(
        "use crate::containers::companion::find_companion_log as find_shared_companion_log;",
        "// Companion log finder is injected from the app layer\n"
        "// use crate::containers::companion::find_companion_log as find_shared_companion_log;"
    )
    
    return content


def fix_companion_log_function(content: str) -> str:
    """Replace the parse_companion_log function to not depend on containers::companion.
    
    The function finds `find_shared_companion_log(ad1_path)?;` and replaces it with
    a call to a new trait/callback parameter. But for simplicity, we'll make it 
    accept an optional closure parameter.
    """
    # Find the function and make it accept an optional external finder
    # Replace: pub fn parse_companion_log(ad1_path: &str) -> Option<CompanionLogInfo>
    # With a version that takes an optional finder callback
    
    # Actually, simplest approach: make the function take an optional ContainerCompanionLogInfo
    # and convert from it. Add a second function that just takes the path.
    
    old_fn_sig = "pub fn parse_companion_log(ad1_path: &str) -> Option<CompanionLogInfo> {"
    new_fn = '''/// Parse companion log file for case metadata.
///
/// Accepts an optional shared companion log finder function.
/// In standalone mode, returns None if no finder is provided.
pub fn parse_companion_log(ad1_path: &str) -> Option<CompanionLogInfo> {
    parse_companion_log_with_finder(ad1_path, None::<fn(&str) -> Option<SharedCompanionInfo>>)
}

/// Shared companion log info (matches containers::companion::CompanionLogInfo fields)
#[derive(Debug, Default)]
pub struct SharedCompanionInfo {
    pub case_number: Option<String>,
    pub evidence_number: Option<String>,
    pub examiner: Option<String>,
    pub notes: Option<String>,
    pub acquisition_started: Option<String>,
    pub unique_description: Option<String>,
    pub created_by: Option<String>,
    pub stored_hashes: Vec<SharedStoredHash>,
}

/// Stored hash from companion log
#[derive(Debug)]
pub struct SharedStoredHash {
    pub algorithm: String,
    pub hash: String,
}

/// Parse companion log with an optional external finder function.
pub fn parse_companion_log_with_finder<F>(ad1_path: &str, finder: Option<F>) -> Option<CompanionLogInfo>
where
    F: Fn(&str) -> Option<SharedCompanionInfo>,
{'''
    
    content = content.replace(old_fn_sig, new_fn)
    
    # Replace the find_shared_companion_log call
    content = content.replace(
        "    // Use shared companion log finder\n"
        "    let shared_info = find_shared_companion_log(ad1_path)?;",
        "    // Use provided companion log finder\n"
        "    let finder = finder?;\n"
        "    let shared_info = finder(ad1_path)?;"
    )
    
    # Fix hash field references - shared_info.stored_hashes items use .hash
    # Shouldn't need changes since we defined SharedStoredHash with .hash and .algorithm
    
    return content


def create_cargo_toml():
    return '''[package]
name = "ffx-ad1"
version = "0.1.0"
edition = "2021"
description = "AD1 (AccessData Logical Image) forensic container parser"
license = "MIT"

[dependencies]
ffx-errors = { workspace = true }
ffx-common = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
flate2 = { workspace = true }
chrono = { workspace = true }
tracing = { workspace = true }
rayon = "1.10"
filetime = "0.2"
parking_lot = "0.12"
'''


def create_lib_rs():
    return '''// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # ffx-ad1 - AD1 (AccessData Logical Image) Parser
//!
//! Pure-Rust parser for AccessData's AD1 logical evidence container format,
//! commonly used in FTK (Forensic Toolkit).

mod info_v2;
pub mod operations;
mod operations_v2;
mod parser;
mod reader_v2;
pub mod types;
pub mod utils;
pub mod vfs;

// Re-export public types
pub use types::{
    Ad1Info, Ad1Stats, ChunkVerifyResult, CompanionLogInfo, ItemMetadata,
    LogicalHeaderInfo, SearchResult, SegmentHeaderInfo, TreeEntry, VerifyEntry,
    VerifyStatus,
};

// Re-export public functions from operations
pub use operations::{
    export_metadata_csv, export_metadata_json, export_tree_csv, export_tree_json,
    extract, extract_with_progress, find_by_extension, find_by_hash, find_by_name,
    get_children, get_children_at_addr, get_children_at_addr_lazy, get_entry_info,
    get_segment_paths, get_stats, get_tree, hash_segments, hash_segments_with_progress,
    hash_single_segment, info, info_fast, is_ad1, read_entry_chunk, read_entry_data,
    read_entry_data_by_addr, verify, verify_against_log, verify_chunks,
    verify_with_progress,
};

// Re-export V2 operations
pub use operations_v2::{
    get_children_at_addr as get_children_at_addr_v2,
    get_container_status as get_container_status_v2,
    get_item_info as get_item_info_v2,
    get_item_metadata as get_item_metadata_v2,
    get_items_metadata as get_items_metadata_v2,
    get_root_children as get_root_children_v2,
    read_file_data as read_file_data_v2,
    verify_item_hash as verify_item_hash_v2,
    ContainerStatus,
};

// Re-export V2 info
pub use info_v2::{get_container_info as get_container_info_v2, Ad1InfoV2, TreeItem};

// Re-export VFS
pub use vfs::Ad1Vfs;

// Re-export companion log helper types
pub use utils::parsing::{SharedCompanionInfo, SharedStoredHash};
'''


def main():
    # Create directory structure
    os.makedirs(os.path.join(DST, "operations"), exist_ok=True)
    os.makedirs(os.path.join(DST, "utils"), exist_ok=True)
    
    # Write Cargo.toml
    cargo_path = os.path.join(BASE, "crates", "ffx-ad1", "Cargo.toml")
    with open(cargo_path, "w") as f:
        f.write(create_cargo_toml())
    print(f"Created {cargo_path}")
    
    # Write lib.rs
    lib_path = os.path.join(DST, "lib.rs")
    with open(lib_path, "w") as f:
        f.write(create_lib_rs())
    print(f"Created {lib_path}")
    
    # Copy and fix each file
    copied = 0
    for rel_path in FILES:
        src_path = os.path.join(SRC, rel_path)
        dst_path = os.path.join(DST, rel_path)
        
        if not os.path.exists(src_path):
            print(f"WARNING: Source file not found: {src_path}")
            continue
        
        with open(src_path, "r") as f:
            content = f.read()
        
        content = fix_imports(content, rel_path)
        
        # Special handling for utils/parsing.rs (companion log)
        if rel_path == "utils/parsing.rs":
            content = fix_companion_log_function(content)
        
        with open(dst_path, "w") as f:
            f.write(content)
        
        copied += 1
        print(f"  Copied: {rel_path}")
    
    print(f"\nCopied {copied} files to {DST}")
    print("Done! Run: cargo check -p ffx-ad1")


if __name__ == "__main__":
    main()
