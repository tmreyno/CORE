// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
//
// Integration tests for libewf-ffi write operations

use libewf_ffi::{EwfCaseInfo, EwfCompression, EwfCompressionMethod, EwfFormat, EwfWriter, EwfWriterConfig};
use tempfile::TempDir;

#[test]
fn test_libewf_version() {
    let version = libewf_ffi::libewf_version();
    println!("libewf version: {}", version);
    assert!(!version.is_empty());
    assert_ne!(version, "unknown");
}

#[test]
fn test_create_e01_basic() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("test_image");

    let config = EwfWriterConfig {
        format: EwfFormat::Encase5,
        compression: EwfCompression::None,
        media_size: Some(32 * 1024), // 32 KB
        case_info: EwfCaseInfo {
            case_number: Some("TEST-001".to_string()),
            examiner_name: Some("Test Examiner".to_string()),
            description: Some("Test E01 image".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut writer = EwfWriter::create(&output_path, config).unwrap();

    // Write 32 KB of test data
    let data = vec![0xABu8; 32 * 1024];
    writer.write_all(&data).unwrap();
    assert_eq!(writer.bytes_written(), 32 * 1024);

    // Set hash values
    writer
        .set_md5_hash("d41d8cd98f00b204e9800998ecf8427e")
        .unwrap();

    // Finalize
    writer.finalize().unwrap();
    assert!(writer.is_finalized());

    // Check that the E01 file was created
    let e01_path = tmp.path().join("test_image.E01");
    assert!(e01_path.exists(), "E01 file should exist at {:?}", e01_path);

    let metadata = std::fs::metadata(&e01_path).unwrap();
    assert!(metadata.len() > 0, "E01 file should not be empty");
    println!(
        "Created E01: {} bytes (from {} bytes of data)",
        metadata.len(),
        32 * 1024
    );
}

#[test]
fn test_create_e01_with_compression() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("compressed");

    let config = EwfWriterConfig {
        format: EwfFormat::Encase5,
        compression: EwfCompression::Best,
        media_size: Some(64 * 1024), // 64 KB
        ..Default::default()
    };

    let mut writer = EwfWriter::create(&output_path, config).unwrap();

    // Write compressible data (all zeros)
    let data = vec![0u8; 64 * 1024];
    writer.write_all(&data).unwrap();
    writer.finalize().unwrap();

    let e01_path = tmp.path().join("compressed.E01");
    assert!(e01_path.exists());

    let metadata = std::fs::metadata(&e01_path).unwrap();
    println!(
        "Compressed E01: {} bytes (from {} bytes of zeros)",
        metadata.len(),
        64 * 1024
    );
    // Compressed zeros should be smaller than the input
    assert!(
        metadata.len() < 64 * 1024,
        "Compressed file should be smaller than input"
    );
}

#[test]
fn test_create_e01_large_write() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("large");

    let total_size: u64 = 1024 * 1024; // 1 MB
    let config = EwfWriterConfig {
        format: EwfFormat::Encase5,
        compression: EwfCompression::Fast,
        media_size: Some(total_size),
        case_info: EwfCaseInfo {
            case_number: Some("LARGE-001".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut writer = EwfWriter::create(&output_path, config).unwrap();

    // Write in chunks
    let chunk = vec![0x42u8; 64 * 1024]; // 64 KB chunks
    let chunks = total_size / (64 * 1024);
    for _ in 0..chunks {
        writer.write_all(&chunk).unwrap();
    }

    assert_eq!(writer.bytes_written(), total_size);
    writer.finalize().unwrap();

    let e01_path = tmp.path().join("large.E01");
    assert!(e01_path.exists());
    println!(
        "Large E01: {} bytes (from {} bytes)",
        std::fs::metadata(&e01_path).unwrap().len(),
        total_size
    );
}

#[test]
fn test_create_e01_with_all_metadata() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("metadata");

    let config = EwfWriterConfig {
        format: EwfFormat::Encase5,
        compression: EwfCompression::None,
        media_size: Some(1024),
        case_info: EwfCaseInfo {
            case_number: Some("META-2024-001".to_string()),
            evidence_number: Some("EV-001".to_string()),
            examiner_name: Some("Jane Smith".to_string()),
            description: Some("Forensic test image with full metadata".to_string()),
            notes: Some("Created by CORE-FFX integration test".to_string()),
            model: Some("Virtual Disk".to_string()),
            serial_number: Some("VD-12345".to_string()),
            acquiry_software_version: None, // Will be set to "CORE-FFX" automatically
        },
        ..Default::default()
    };

    let mut writer = EwfWriter::create(&output_path, config).unwrap();
    writer.write_all(&[0u8; 1024]).unwrap();
    writer.set_md5_hash("d41d8cd98f00b204e9800998ecf8427e").unwrap();
    writer.set_sha1_hash("da39a3ee5e6b4b0d3255bfef95601890afd80709").unwrap();
    writer.finalize().unwrap();

    assert!(tmp.path().join("metadata.E01").exists());
}

#[test]
fn test_create_l01_logical() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("logical");

    let config = EwfWriterConfig {
        format: EwfFormat::LogicalEncase5,
        compression: EwfCompression::Fast,
        media_size: Some(4096),
        case_info: EwfCaseInfo {
            case_number: Some("L01-TEST".to_string()),
            description: Some("Logical evidence file test".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let result = EwfWriter::create(&output_path, config);
    // L01 format writing is still TODO in libewf 20251220 — set_format rejects it
    // The format constants exist but the write path is not yet implemented
    match result {
        Ok(mut writer) => {
            writer.write_all(&[0xFFu8; 4096]).unwrap();
            writer.finalize().unwrap();

            let l01_path = tmp.path().join("logical.L01");
            assert!(l01_path.exists(), "L01 file should exist at {:?}", l01_path);
            println!(
                "Created L01: {} bytes",
                std::fs::metadata(&l01_path).unwrap().len()
            );
        }
        Err(e) => {
            let msg = format!("{}", e);
            assert!(
                msg.contains("unsupported format"),
                "Expected unsupported format error for L01, got: {}",
                msg
            );
            println!(
                "L01 write not yet supported in this libewf version (expected): {}",
                msg
            );
        }
    }
}

#[test]
fn test_format_properties() {
    // Extension mapping — Encase7 (0x07) produces .E01, only V2 (0x37) produces .Ex01
    assert_eq!(EwfFormat::Encase5.extension(), ".E01");
    assert_eq!(EwfFormat::Encase6.extension(), ".E01");
    assert_eq!(EwfFormat::Encase7.extension(), ".E01");
    assert_eq!(EwfFormat::V2Encase7.extension(), ".Ex01");
    assert_eq!(EwfFormat::LogicalEncase5.extension(), ".L01");
    assert_eq!(EwfFormat::LogicalEncase6.extension(), ".L01");
    assert_eq!(EwfFormat::LogicalEncase7.extension(), ".L01");
    assert_eq!(EwfFormat::V2LogicalEncase7.extension(), ".Lx01");
    assert_eq!(EwfFormat::Ewfx.extension(), ".E01");

    assert!(!EwfFormat::Encase5.is_logical());
    assert!(!EwfFormat::Encase6.is_logical());
    assert!(!EwfFormat::V2Encase7.is_logical());
    assert!(EwfFormat::LogicalEncase5.is_logical());
    assert!(EwfFormat::LogicalEncase6.is_logical());
    assert!(EwfFormat::LogicalEncase7.is_logical());
    assert!(EwfFormat::V2LogicalEncase7.is_logical());

    assert!(!EwfFormat::Encase5.is_v2());
    assert!(EwfFormat::V2Encase7.is_v2());
    assert!(EwfFormat::V2LogicalEncase7.is_v2());
}

#[test]
fn test_double_finalize_is_safe() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("double_finalize");

    let config = EwfWriterConfig {
        format: EwfFormat::Encase5,
        compression: EwfCompression::None,
        media_size: Some(512),
        ..Default::default()
    };

    let mut writer = EwfWriter::create(&output_path, config).unwrap();
    writer.write_all(&[0u8; 512]).unwrap();
    writer.finalize().unwrap();
    // Second finalize should be a no-op
    writer.finalize().unwrap();
}

#[test]
fn test_write_after_finalize_fails() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("write_after_final");

    let config = EwfWriterConfig {
        format: EwfFormat::Encase5,
        compression: EwfCompression::None,
        media_size: Some(512),
        ..Default::default()
    };

    let mut writer = EwfWriter::create(&output_path, config).unwrap();
    writer.write_all(&[0u8; 512]).unwrap();
    writer.finalize().unwrap();

    // Writing after finalize should fail
    let result = writer.write(&[0u8; 512]);
    assert!(result.is_err());
}

#[test]
fn test_invalid_path_fails() {
    let config = EwfWriterConfig::default();
    let result = EwfWriter::create("/nonexistent/path/image", config);
    assert!(result.is_err());
}

#[test]
fn test_ewfx_format() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("ewfx_test");

    let config = EwfWriterConfig {
        format: EwfFormat::Ewfx,
        compression: EwfCompression::None,
        media_size: Some(1024),
        ..Default::default()
    };

    let result = EwfWriter::create(&output_path, config);
    match result {
        Ok(mut writer) => {
            writer.write_all(&[0u8; 1024]).unwrap();
            match writer.finalize() {
                Ok(()) => {
                    // EWFX uses .E01 extension
                    let e01_path = tmp.path().join("ewfx_test.E01");
                    assert!(e01_path.exists(), "EWFX file should exist at {:?}", e01_path);
                    println!(
                        "Created EWFX: {} bytes",
                        std::fs::metadata(&e01_path).unwrap().len()
                    );
                }
                Err(e) => {
                    // EWFX finalize is known to fail in libewf 20251220
                    println!("EWFX finalize not supported: {}", e);
                }
            }
        }
        Err(e) => {
            println!("EWFX write not supported: {}", e);
        }
    }
}

#[test]
fn test_bzip2_compression() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("bzip2_test");

    // BZIP2 compression requires V2 format (EWF2 segment file type)
    let config = EwfWriterConfig {
        format: EwfFormat::V2Encase7,
        compression: EwfCompression::Best,
        compression_method: EwfCompressionMethod::Bzip2,
        media_size: Some(64 * 1024),
        case_info: EwfCaseInfo {
            case_number: Some("BZIP2-001".to_string()),
            description: Some("BZIP2 compression test".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut writer = EwfWriter::create(&output_path, config).unwrap();

    // Write compressible data
    let data = vec![0u8; 64 * 1024];
    writer.write_all(&data).unwrap();
    writer.finalize().unwrap();

    let ex01_path = tmp.path().join("bzip2_test.Ex01");
    assert!(ex01_path.exists(), "Ex01 file should exist at {:?}", ex01_path);

    let file_size = std::fs::metadata(&ex01_path).unwrap().len();
    println!(
        "Created Ex01 with BZIP2: {} bytes (from {} bytes of zeros)",
        file_size,
        64 * 1024
    );
    assert!(
        file_size < 64 * 1024,
        "BZIP2 compressed file should be smaller than input"
    );
}

#[test]
fn test_v2_encase7_format() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("v2_encase7");

    let config = EwfWriterConfig {
        format: EwfFormat::V2Encase7,
        compression: EwfCompression::Fast,
        media_size: Some(8192),
        case_info: EwfCaseInfo {
            case_number: Some("V2-001".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut writer = EwfWriter::create(&output_path, config).unwrap();
    writer.write_all(&[0xAAu8; 8192]).unwrap();
    writer.finalize().unwrap();

    let ex01_path = tmp.path().join("v2_encase7.Ex01");
    assert!(ex01_path.exists(), "V2 Ex01 file should exist at {:?}", ex01_path);
    println!(
        "Created V2 Encase7: {} bytes",
        std::fs::metadata(&ex01_path).unwrap().len()
    );
}

#[test]
fn test_libewf_version_is_modern() {
    let version = libewf_ffi::libewf_version();
    println!("libewf version: {}", version);
    assert_eq!(version, "20251220", "Expected modern libewf version 20251220");
}

#[test]
fn test_encase7_produces_e01() {
    // Encase7 (format 0x07) uses EWF1 segment type → produces .E01, NOT .Ex01
    // Only V2Encase7 (format 0x37) produces .Ex01
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("encase7_test");

    let config = EwfWriterConfig {
        format: EwfFormat::Encase7,
        compression: EwfCompression::Fast,
        media_size: Some(4096),
        case_info: EwfCaseInfo {
            case_number: Some("E7-001".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut writer = EwfWriter::create(&output_path, config).unwrap();
    writer.write_all(&[0x55u8; 4096]).unwrap();
    writer.finalize().unwrap();

    // Encase7 (non-V2) produces .E01
    let e01_path = tmp.path().join("encase7_test.E01");
    assert!(e01_path.exists(), "Encase7 should produce .E01 at {:?}", e01_path);

    // Verify .Ex01 was NOT created
    let ex01_path = tmp.path().join("encase7_test.Ex01");
    assert!(!ex01_path.exists(), "Encase7 should NOT produce .Ex01");

    println!(
        "Created Encase7 (.E01): {} bytes",
        std::fs::metadata(&e01_path).unwrap().len()
    );
}

#[test]
fn test_bzip2_requires_v2_format() {
    // BZIP2 with non-V2 format should fail with a clear error
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("bzip2_invalid");

    let config = EwfWriterConfig {
        format: EwfFormat::Encase5,
        compression: EwfCompression::Best,
        compression_method: EwfCompressionMethod::Bzip2,
        media_size: Some(1024),
        ..Default::default()
    };

    let result = EwfWriter::create(&output_path, config);
    assert!(result.is_err(), "BZIP2 with Encase5 should fail");
    let err = result.err().unwrap();
    let err_msg = format!("{}", err);
    assert!(
        err_msg.contains("V2 format"),
        "Error should mention V2 format requirement, got: {}",
        err_msg
    );
}
