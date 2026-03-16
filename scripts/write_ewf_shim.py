#!/usr/bin/env python3
"""Write the ewf shim files for the crate extraction."""
import os

BASE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EWF_DIR = os.path.join(BASE, "src-tauri", "src", "ewf")
PARSER_DIR = os.path.join(EWF_DIR, "parser")

# 1. Read the original metadata.rs to extract the viewer function (lines 249+)
orig_metadata = os.path.join(PARSER_DIR, "metadata.rs")
with open(orig_metadata, "r") as f:
    lines = f.readlines()

# Find the line with "fn format_media_type" - that's where we start keeping
start_idx = None
for i, line in enumerate(lines):
    if "fn format_media_type" in line:
        start_idx = i - 1  # include the doc comment
        break

# Find the doc comment before format_media_type
if start_idx is not None:
    while start_idx > 0 and (lines[start_idx].strip().startswith("///") or lines[start_idx].strip().startswith("//")):
        start_idx -= 1
    start_idx += 1  # move forward past comment separator

viewer_code = "".join(lines[start_idx:]) if start_idx else ""

# 2. Write the new metadata.rs (viewer-only)
new_metadata = """// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Viewer metadata conversion for EWF files.
//!
//! Contains only the viewer-coupled function that cannot be in the ffx-ewf crate.
//! All parsing functions are in ffx-ewf::parser::metadata.

use crate::common::format_size;
use crate::viewer::{HeaderRegion, MetadataField, ParsedMetadata};

use super::types::{EwfDetailedInfo, EwfSectionHeader, EwfVariant, SECTION_HEADER_SIZE};

""" + viewer_code

with open(orig_metadata, "w") as f:
    f.write(new_metadata)
print(f"  metadata.rs: {len(new_metadata.splitlines())} lines (viewer-only)")

# 3. Write the new parser/mod.rs (shim)
parser_mod = """// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! EWF parser re-export shim.
//!
//! Core parsing logic lives in the `ffx-ewf` crate.
//! This module re-exports everything from the crate and adds the
//! viewer-coupled `ewf_detailed_info_to_metadata` function which
//! depends on `crate::viewer` types.

mod metadata;

// Re-export types submodule so local metadata.rs can use `super::types::{...}`
pub mod types {
    pub use ffx_ewf::parser::types::*;
}

// Re-export all public items from the crate's parser
pub use ffx_ewf::parser::{
    parse_ewf_file, is_ewf_file, is_l01_file,
    EwfCaseInfo, EwfDetailedInfo, EwfErrorEntry, EwfHashInfo,
    EwfSectionHeader, EwfVariant, EwfVolumeInfo,
};

// Re-export the local viewer function
pub use self::metadata::ewf_detailed_info_to_metadata;
"""

with open(os.path.join(PARSER_DIR, "mod.rs"), "w") as f:
    f.write(parser_mod)
print(f"  parser/mod.rs: {len(parser_mod.splitlines())} lines (shim)")

# 4. Write the new ewf/mod.rs (shim)
ewf_mod = """// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! EWF module re-export shim.
//!
//! Core EWF logic lives in the `ffx-ewf` crate.
//! This module re-exports everything and adds viewer-coupled parser functions.

// Local parser module (contains viewer-coupled metadata function)
pub mod parser;

// Re-export VFS as a submodule (callers use `ewf::vfs::EwfVfs`)
pub mod vfs {
    pub use ffx_ewf::vfs::*;
}

// Re-export L01 reader as a submodule
pub mod l01_reader {
    pub use ffx_ewf::l01_reader::*;
}

// Re-export all public types from the crate root
pub use ffx_ewf::{
    ChunkVerifyResult, EwfInfo, EwfSearchResult, EwfStats, HeaderInfo,
    StoredImageHash, VerifyResult, VolumeSection,
};

// Re-export parser types (also available via parser::)
pub use parser::{
    ewf_detailed_info_to_metadata, is_ewf_file, is_l01_file, parse_ewf_file,
    EwfCaseInfo, EwfDetailedInfo, EwfErrorEntry, EwfHashInfo, EwfSectionHeader,
    EwfVariant, EwfVolumeInfo,
};

// Re-export handle
pub use ffx_ewf::EwfHandle;

// Re-export VFS type at module level
pub use ffx_ewf::EwfVfs;

// Re-export L01 reader types at module level
pub use ffx_ewf::{parse_l01_file_tree, L01Entry, L01FileTree};

// Re-export public functions from operations
pub use ffx_ewf::{
    export_metadata_csv, export_metadata_json, extract, extract_with_progress,
    get_segment_paths, get_stats, hash_single_segment, info, info_fast, is_e01,
    is_ewf, verify, verify_chunks, verify_with_progress,
};
"""

with open(os.path.join(EWF_DIR, "mod.rs"), "w") as f:
    f.write(ewf_mod)
print(f"  ewf/mod.rs: {len(ewf_mod.splitlines())} lines (shim)")

# 5. Delete source files that are now in the crate
deleted: list[str] = []
for fname in ["cache.rs", "handle.rs", "l01_reader.rs", "operations.rs", "types.rs", "vfs.rs"]:
    fpath = os.path.join(EWF_DIR, fname)
    if os.path.exists(fpath):
        os.remove(fpath)
        deleted.append(fname)

# Delete parser/types.rs (now in crate)
parser_types = os.path.join(PARSER_DIR, "types.rs")
if os.path.exists(parser_types):
    os.remove(parser_types)
    deleted.append("parser/types.rs")

print(f"  Deleted {len(deleted)} files: {', '.join(deleted)}")
print("Done!")
