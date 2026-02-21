// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
//
// # libewf-ffi — Safe Rust bindings for libewf
//
// Provides read/write access to EWF forensic image formats:
// - **E01** (EnCase physical image)
// - **L01** (EnCase logical evidence file)
// - **Ex01/Lx01** (EnCase 7 V2 formats)
//
// ## Quick Start
//
// ### Create an E01 image
// ```no_run
// use libewf_ffi::{EwfWriter, EwfWriterConfig, EwfFormat, EwfCompression, EwfCaseInfo};
//
// let config = EwfWriterConfig {
//     format: EwfFormat::Encase5,
//     compression: EwfCompression::Fast,
//     case_info: EwfCaseInfo {
//         case_number: Some("2024-001".to_string()),
//         examiner_name: Some("J. Doe".to_string()),
//         ..Default::default()
//     },
//     media_size: Some(1024 * 1024),
//     ..Default::default()
// };
//
// let mut writer = EwfWriter::create("/tmp/evidence", config).unwrap();
// writer.write_all(&[0u8; 1024 * 1024]).unwrap();
// writer.finalize().unwrap();
// ```
//
// ### Read an E01 image
// ```no_run
// use libewf_ffi::reader::EwfReader;
//
// let reader = EwfReader::open("/tmp/evidence.E01").unwrap();
// let info = reader.image_info().unwrap();
// println!("Format: {}, Size: {} bytes", info.format, info.media_size);
//
// let mut buf = vec![0u8; 512];
// let n = reader.read_at(0, &mut buf).unwrap();
// ```
//
// ### Get library version
// ```no_run
// let version = libewf_ffi::libewf_version();
// println!("libewf version: {}", version);
// ```

pub mod error;
pub mod ffi;
pub mod reader;
pub mod writer;

// Re-export key types at crate root
pub use error::{Error, Result};
pub use writer::{
    EwfCaseInfo, EwfCompression, EwfCompressionMethod, EwfFormat, EwfWriter, EwfWriterConfig,
    libewf_version,
};
pub use reader::{
    EwfDetectedCompressionMethod, EwfDetectedFormat, EwfImageInfo, EwfReadCaseInfo, EwfReader,
};
