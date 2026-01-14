// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Format-specific header parsers for hex view metadata extraction

pub mod ad1;
pub mod archives;
pub mod disk_images;
pub mod ewf;

// Re-export all parsers for convenient access
pub use ad1::parse_ad1_header;
pub use archives::{parse_7z_header, parse_gzip_header, parse_rar_header, parse_zip_header};
pub use disk_images::{parse_qcow2_header, parse_raw_header, parse_vhdx_header, parse_vmdk_header};
pub use ewf::parse_ewf_header;
