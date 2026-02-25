// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! EWF (Expert Witness Format) - E01/L01/Ex01/Lx01 forensic image parser
//!
//! This module provides parsing, verification, and extraction capabilities for
//! EnCase's Expert Witness Format (EWF), including all variants:
//!
//! - **E01**: Physical disk images (EWF v1)
//! - **L01**: Logical evidence files (EWF v1)
//! - **Ex01**: Physical disk images (EWF v2)
//! - **Lx01**: Logical evidence files (EWF v2)
//!
//! ## EWF Format Overview
//!
//! EWF is a segmented disk image format developed by Guidance Software (now OpenText)
//! for their EnCase forensic suite. Key features:
//!
//! - **Segmented storage**: Large images split across multiple .E01, .E02, etc. files
//! - **Zlib compression**: Chunks are individually compressed for space efficiency
//! - **Chunk-based access**: Random access to any part of the image
//! - **Embedded hashes**: MD5/SHA1 verification built into the format
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │ EVF/LVF Signature (8 bytes)                                  │
//! │  - E01:  "EVF\x09\x0d\x0a\xff\x00"                           │
//! │  - L01:  "LVF\x09\x0d\x0a\xff\x00"                           │
//! │  - Ex01: "EVF2\x0d\x0a\x81\x00"                              │
//! │  - Lx01: "LVF2\x0d\x0a\x81\x00"                              │
//! ├──────────────────────────────────────────────────────────────┤
//! │ Section Chain (linked list of sections)                      │
//! │  ┌─────────────────────────────────────────────────────────┐ │
//! │  │ header: Case info, examiner, etc.                       │ │
//! │  │ volume: Chunk count, sector info, compression           │ │
//! │  │ sectors: Compressed chunk data                          │ │
//! │  │ table: Chunk offset table                               │ │
//! │  │ hash/digest: Embedded MD5/SHA1 hashes                   │ │
//! │  │ done: End of segment marker                             │ │
//! │  └─────────────────────────────────────────────────────────┘ │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Multi-Segment Support
//!
//! E01/L01 files can span multiple segments:
//! - First segment (.E01/.L01) contains all metadata
//! - Subsequent segments (.E02, .E03, etc.) contain additional chunk data
//! - "next" sections indicate continuation to next segment
//!
//! ## Usage
//!
//! ```rust,ignore
//! use ewf::{info, verify, extract, is_ewf, is_l01_file};
//!
//! // Check if file is any EWF format (E01/L01/Ex01/Lx01)
//! if is_ewf("/path/to/image.E01")? {
//!     // Get container info
//!     let info = info("/path/to/image.E01")?;
//!     println!("Image size: {} bytes", info.total_size);
//!     
//!     // Verify image integrity
//!     let hash = verify("/path/to/image.E01", "md5")?;
//!     
//!     // Extract to raw image
//!     extract("/path/to/image.E01", "/output/dir")?;
//! }
//! 
//! // Check specifically for L01 (logical evidence)
//! if is_l01_file("/path/to/evidence.L01")? {
//!     let info = info("/path/to/evidence.L01")?;
//! }
//! ```

mod types;
mod cache;
mod handle;
pub mod l01_reader;
mod operations;
pub mod parser;  // Hex viewer parser for detailed metadata
pub mod vfs;

// Re-export public types
pub use types::{
    StoredImageHash, VolumeSection, EwfInfo, VerifyResult, HeaderInfo,
    EwfStats, EwfSearchResult, ChunkVerifyResult,
};

// Re-export parser types for hex viewer
pub use parser::{
    EwfVariant, EwfDetailedInfo, EwfSectionHeader, EwfVolumeInfo, 
    EwfCaseInfo, EwfHashInfo, EwfErrorEntry,
    parse_ewf_file, ewf_detailed_info_to_metadata, 
    is_ewf_file, is_l01_file,
};

// Re-export the handle for advanced usage
pub use handle::EwfHandle;

// Re-export VFS
pub use vfs::EwfVfs;

// Re-export L01 reader
pub use l01_reader::{L01FileTree, L01Entry, parse_l01_file_tree};

// Re-export public functions
pub use operations::{
    info, info_fast, is_e01, is_ewf, get_segment_paths, hash_single_segment,
    verify, verify_with_progress, verify_chunks,
    extract, extract_with_progress, get_stats, export_metadata_json, export_metadata_csv,
};
