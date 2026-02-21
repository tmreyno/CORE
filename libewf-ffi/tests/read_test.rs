// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
//
// Integration tests for libewf-ffi read operations
//
// These tests use write-then-read round-trips to validate the reader
// since we create known E01 images with the writer and then read them back.

use libewf_ffi::{
    EwfCaseInfo, EwfCompression, EwfCompressionMethod, EwfFormat, EwfWriter, EwfWriterConfig,
};
use libewf_ffi::reader::{EwfDetectedFormat, EwfReader};
use tempfile::TempDir;

/// Helper: create a test E01 image with known data and return (dir, base_path)
fn create_test_image(
    format: EwfFormat,
    size: usize,
    case_info: EwfCaseInfo,
    set_md5: bool,
    set_sha1: bool,
) -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("test_image");

    let config = EwfWriterConfig {
        format,
        compression: EwfCompression::Fast,
        media_size: Some(size as u64),
        case_info,
        ..Default::default()
    };

    let mut writer = EwfWriter::create(&output_path, config).unwrap();

    // Write deterministic data: repeating 0x00..0xFF pattern
    let mut data = vec![0u8; size];
    for (i, byte) in data.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    writer.write_all(&data).unwrap();

    if set_md5 {
        writer
            .set_md5_hash("0123456789abcdef0123456789abcdef")
            .unwrap();
    }
    if set_sha1 {
        writer
            .set_sha1_hash("0123456789abcdef01230123456789abcdef0123")
            .unwrap();
    }

    writer.finalize().unwrap();

    // Determine the actual output file path (with extension)
    let ext = format.extension();
    let actual_path = tmp.path().join(format!("test_image{}", ext));

    (tmp, actual_path)
}

// =============================================================================
// Basic open / close tests
// =============================================================================

#[test]
fn test_reader_open_e01() {
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase5,
        32 * 1024,
        EwfCaseInfo::default(),
        false,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();
    assert_eq!(reader.media_size(), 32 * 1024);
}

#[test]
fn test_reader_open_nonexistent_fails() {
    let result = EwfReader::open("/nonexistent/path/image.E01");
    assert!(result.is_err());
    match result {
        Err(e) => {
            let err_msg = e.to_string();
            assert!(err_msg.contains("does not exist"), "Error: {}", err_msg);
        }
        Ok(_) => panic!("Expected error for nonexistent file"),
    }
}

// =============================================================================
// Read data round-trip tests
// =============================================================================

#[test]
fn test_reader_read_at_offset_zero() {
    let size = 64 * 1024;
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase5,
        size,
        EwfCaseInfo::default(),
        false,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();

    // Read first 512 bytes
    let mut buf = vec![0u8; 512];
    let n = reader.read_at(0, &mut buf).unwrap();
    assert_eq!(n, 512);

    // Verify data matches our deterministic pattern
    for (i, byte) in buf.iter().enumerate() {
        assert_eq!(*byte, (i % 256) as u8, "Mismatch at byte {}", i);
    }
}

#[test]
fn test_reader_read_at_arbitrary_offset() {
    let size = 128 * 1024;
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase5,
        size,
        EwfCaseInfo::default(),
        false,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();

    // Read 256 bytes at offset 1024
    let mut buf = vec![0u8; 256];
    let n = reader.read_at(1024, &mut buf).unwrap();
    assert_eq!(n, 256);

    // Verify data: at offset 1024, pattern byte = (1024 + i) % 256
    for (i, byte) in buf.iter().enumerate() {
        let expected = ((1024 + i) % 256) as u8;
        assert_eq!(*byte, expected, "Mismatch at offset {}", 1024 + i);
    }
}

#[test]
fn test_reader_read_entire_image() {
    let size = 32 * 1024;
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase5,
        size,
        EwfCaseInfo::default(),
        false,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();
    assert_eq!(reader.media_size(), size as u64);

    // Read entire image
    let mut buf = vec![0u8; size];
    let n = reader.read_at(0, &mut buf).unwrap();
    assert_eq!(n, size);

    // Verify all data
    for (i, byte) in buf.iter().enumerate() {
        assert_eq!(*byte, (i % 256) as u8, "Mismatch at byte {}", i);
    }
}

#[test]
fn test_reader_sequential_read() {
    let size = 32 * 1024;
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase5,
        size,
        EwfCaseInfo::default(),
        false,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();

    // Seek to beginning
    reader.seek(0, 0).unwrap(); // SEEK_SET = 0

    // Read in chunks
    let mut all_data = Vec::new();
    let mut buf = vec![0u8; 4096];
    loop {
        let n = reader.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        all_data.extend_from_slice(&buf[..n]);
    }

    assert_eq!(all_data.len(), size);
    for (i, byte) in all_data.iter().enumerate() {
        assert_eq!(*byte, (i % 256) as u8, "Mismatch at byte {}", i);
    }
}

#[test]
fn test_reader_empty_buffer() {
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase5,
        32 * 1024,
        EwfCaseInfo::default(),
        false,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();
    let mut buf = vec![];
    let n = reader.read_at(0, &mut buf).unwrap();
    assert_eq!(n, 0);
}

// =============================================================================
// Format detection tests
// =============================================================================

#[test]
fn test_reader_detect_encase5_format() {
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase5,
        32 * 1024,
        EwfCaseInfo::default(),
        false,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();
    let format = reader.format().unwrap();
    assert_eq!(format, EwfDetectedFormat::Encase5);
    assert_eq!(format.name(), "EnCase 5");
    assert_eq!(format.extension(), ".E01");
    assert!(!format.is_logical());
    assert!(!format.is_v2());
}

#[test]
fn test_reader_detect_encase6_format() {
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase6,
        32 * 1024,
        EwfCaseInfo::default(),
        false,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();
    let format = reader.format().unwrap();
    assert_eq!(format, EwfDetectedFormat::Encase6);
    assert!(!format.is_v2());
}

#[test]
fn test_reader_detect_encase7_format() {
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase7,
        32 * 1024,
        EwfCaseInfo::default(),
        false,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();
    let format = reader.format().unwrap();
    // Encase7 (0x07) produces .E01 (EWF1 segment type), NOT .Ex01
    assert_eq!(format, EwfDetectedFormat::Encase7);
    assert_eq!(format.extension(), ".E01");
    assert!(!format.is_v2());
}

#[test]
fn test_reader_detect_v2encase7_format() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("test_v2");

    let config = EwfWriterConfig {
        format: EwfFormat::V2Encase7,
        compression: EwfCompression::Fast,
        compression_method: EwfCompressionMethod::Deflate,
        media_size: Some(32 * 1024),
        ..Default::default()
    };

    let mut writer = EwfWriter::create(&output_path, config).unwrap();
    writer.write_all(&vec![0u8; 32 * 1024]).unwrap();
    writer.finalize().unwrap();

    let ex01_path = tmp.path().join("test_v2.Ex01");
    let reader = EwfReader::open(&ex01_path).unwrap();
    let format = reader.format().unwrap();
    // V2Encase7 (0x37) produces .Ex01 (EWF2 segment type)
    assert_eq!(format, EwfDetectedFormat::V2Encase7);
    assert_eq!(format.extension(), ".Ex01");
    assert!(format.is_v2());
    assert!(!format.is_logical());
}

// =============================================================================
// Metadata / case info tests
// =============================================================================

#[test]
fn test_reader_case_info_round_trip() {
    let case_info = EwfCaseInfo {
        case_number: Some("CASE-2024-001".to_string()),
        evidence_number: Some("EV-42".to_string()),
        examiner_name: Some("Jane Forensic".to_string()),
        description: Some("Test disk image".to_string()),
        notes: Some("Created for unit test".to_string()),
        model: Some("WD Blue 1TB".to_string()),
        serial_number: Some("SN12345678".to_string()),
        ..Default::default()
    };

    let (_tmp, path) = create_test_image(
        EwfFormat::Encase5,
        32 * 1024,
        case_info,
        false,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();
    let info = reader.image_info().unwrap();

    // These fields are reliably stored in all EWF formats
    assert_eq!(
        info.case_info.case_number.as_deref(),
        Some("CASE-2024-001")
    );
    assert_eq!(info.case_info.evidence_number.as_deref(), Some("EV-42"));
    assert_eq!(
        info.case_info.examiner_name.as_deref(),
        Some("Jane Forensic")
    );
    assert_eq!(
        info.case_info.description.as_deref(),
        Some("Test disk image")
    );
    assert_eq!(
        info.case_info.notes.as_deref(),
        Some("Created for unit test")
    );

    // NOTE: "model" and "serial_number" are NOT stored by libewf in Encase5
    // format (the header section format doesn't include these fields).
    // These identifiers are registered with empty values but not persisted.
    // This is a libewf limitation — the header format for Encase1-5 only
    // supports: case_number, description, examiner_name, evidence_number,
    // notes, acquiry_date, system_date, acquiry_operating_system,
    // acquiry_software_version, and password.
}

#[test]
fn test_reader_individual_header_values() {
    let case_info = EwfCaseInfo {
        case_number: Some("HDR-TEST".to_string()),
        ..Default::default()
    };

    let (_tmp, path) = create_test_image(
        EwfFormat::Encase5,
        32 * 1024,
        case_info,
        false,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();

    // Check individual header value retrieval
    assert_eq!(
        reader.get_header_value("case_number").as_deref(),
        Some("HDR-TEST")
    );

    // The writer always sets acquiry_software_version to "CORE-FFX"
    assert_eq!(
        reader.get_header_value("acquiry_software_version").as_deref(),
        Some("CORE-FFX")
    );

    // Header value count should be > 0
    let count = reader.header_value_count().unwrap();
    assert!(count > 0, "Expected at least 1 header value, got {}", count);

    // Verify list_header_values returns consistent results
    let all_headers = reader.list_header_values();
    assert!(!all_headers.is_empty());
    let case_entry = all_headers.iter().find(|(k, _)| k == "case_number");
    assert_eq!(
        case_entry.and_then(|(_, v)| v.as_deref()),
        Some("HDR-TEST")
    );
}

// =============================================================================
// Image info tests
// =============================================================================

#[test]
fn test_reader_image_info_complete() {
    // Use Encase6 because SHA1 binary hash storage is only supported in
    // Encase6+ (EWF1 Encase5 only stores MD5 in the hash section)
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase6,
        64 * 1024,
        EwfCaseInfo::default(),
        true,
        true,
    );

    let reader = EwfReader::open(&path).unwrap();
    let info = reader.image_info().unwrap();

    assert_eq!(info.format, EwfDetectedFormat::Encase6);
    assert_eq!(info.media_size, 64 * 1024);
    assert_eq!(info.bytes_per_sector, 512);
    assert_eq!(info.sectors_per_chunk, 64);
    assert!(!info.is_corrupted);
    assert!(!info.is_encrypted);

    // Verify hashes were stored
    assert!(info.md5_hash.is_some(), "Expected MD5 hash to be stored");
    assert!(info.sha1_hash.is_some(), "Expected SHA1 hash to be stored (requires Encase6+)");
}

#[test]
fn test_reader_md5_only_encase5() {
    // Encase5 only supports MD5 in the binary hash section — SHA1 is not stored
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase5,
        32 * 1024,
        EwfCaseInfo::default(),
        true,
        true, // SHA1 set attempt — will succeed on handle but not persist
    );

    let reader = EwfReader::open(&path).unwrap();
    let info = reader.image_info().unwrap();

    // MD5 is stored in the hash section
    assert!(info.md5_hash.is_some(), "Expected MD5 hash in Encase5");
    // SHA1 is NOT stored in Encase5's binary hash section — this is a known
    // libewf/format limitation. The UTF-8 hash value may or may not persist.
}

#[test]
fn test_reader_no_hashes_when_not_set() {
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase5,
        32 * 1024,
        EwfCaseInfo::default(),
        false,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();
    let info = reader.image_info().unwrap();

    assert!(info.md5_hash.is_none());
    assert!(info.sha1_hash.is_none());
}

#[test]
fn test_reader_corruption_check() {
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase5,
        32 * 1024,
        EwfCaseInfo::default(),
        false,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();
    // A freshly created image should not be corrupted
    assert!(!reader.is_corrupted());
    assert!(!reader.is_encrypted());
}

// =============================================================================
// Seek / offset tests
// =============================================================================

#[test]
fn test_reader_seek_and_offset() {
    let size = 64 * 1024;
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase5,
        size,
        EwfCaseInfo::default(),
        false,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();

    // Seek to offset 1024 (SEEK_SET = 0)
    let new_offset = reader.seek(1024, 0).unwrap();
    assert_eq!(new_offset, 1024);

    // Verify current offset
    let current = reader.current_offset().unwrap();
    assert_eq!(current, 1024);

    // Read and verify we're at the right position
    let mut buf = vec![0u8; 4];
    let n = reader.read(&mut buf).unwrap();
    assert_eq!(n, 4);
    for (i, byte) in buf.iter().enumerate() {
        assert_eq!(*byte, ((1024 + i) % 256) as u8);
    }
}

// =============================================================================
// Zero-on-error flag test
// =============================================================================

#[test]
fn test_reader_set_zero_on_error() {
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase5,
        32 * 1024,
        EwfCaseInfo::default(),
        false,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();

    // Just verify the API doesn't error — we can't easily create a
    // damaged image to test the actual behavior
    reader.set_zero_on_error(true).unwrap();
    reader.set_zero_on_error(false).unwrap();
}

// =============================================================================
// EwfDetectedFormat display / properties tests
// =============================================================================

#[test]
fn test_detected_format_properties() {
    // Logical formats
    assert!(EwfDetectedFormat::LogicalEncase5.is_logical());
    assert!(EwfDetectedFormat::LogicalEncase6.is_logical());
    assert!(EwfDetectedFormat::LogicalEncase7.is_logical());
    assert!(EwfDetectedFormat::V2LogicalEncase7.is_logical());

    // Non-logical formats
    assert!(!EwfDetectedFormat::Encase5.is_logical());
    assert!(!EwfDetectedFormat::V2Encase7.is_logical());

    // V2 formats
    assert!(EwfDetectedFormat::V2Encase7.is_v2());
    assert!(EwfDetectedFormat::V2LogicalEncase7.is_v2());
    assert!(!EwfDetectedFormat::Encase7.is_v2());

    // Extensions
    assert_eq!(EwfDetectedFormat::Encase5.extension(), ".E01");
    assert_eq!(EwfDetectedFormat::Encase7.extension(), ".E01");
    assert_eq!(EwfDetectedFormat::V2Encase7.extension(), ".Ex01");
    assert_eq!(EwfDetectedFormat::LogicalEncase5.extension(), ".L01");
    assert_eq!(EwfDetectedFormat::V2LogicalEncase7.extension(), ".Lx01");

    // Display
    assert_eq!(format!("{}", EwfDetectedFormat::Encase5), "EnCase 5");
    assert_eq!(format!("{}", EwfDetectedFormat::V2Encase7), "EnCase 7 V2");
    assert_eq!(
        format!("{}", EwfDetectedFormat::Unknown(0xFF)),
        "Unknown"
    );
}

// =============================================================================
// V2 format with BZIP2 round-trip
// =============================================================================

#[test]
#[ignore = "BZIP2 V2 read-back may require libewf fix — write succeeds but open for reading fails with file IO pool error"]
fn test_reader_v2_bzip2_round_trip() {
    let tmp = TempDir::new().unwrap();
    let output_path = tmp.path().join("test_bzip2");

    let config = EwfWriterConfig {
        format: EwfFormat::V2Encase7,
        compression: EwfCompression::Best,
        compression_method: EwfCompressionMethod::Bzip2,
        media_size: Some(32 * 1024),
        case_info: EwfCaseInfo {
            case_number: Some("BZIP2-TEST".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut writer = EwfWriter::create(&output_path, config).unwrap();
    let data: Vec<u8> = (0..32 * 1024).map(|i| (i % 256) as u8).collect();
    writer.write_all(&data).unwrap();
    writer.finalize().unwrap();

    // Read back — V2Encase7 uses .Ex01 extension
    let ex01_path = tmp.path().join("test_bzip2.Ex01");
    let reader = EwfReader::open(&ex01_path).unwrap();
    let info = reader.image_info().unwrap();

    assert_eq!(info.format, EwfDetectedFormat::V2Encase7);
    assert!(info.format.is_v2());
    assert_eq!(info.media_size, 32 * 1024);
    assert_eq!(
        info.case_info.case_number.as_deref(),
        Some("BZIP2-TEST")
    );

    // Read data back and verify
    let mut read_buf = vec![0u8; 32 * 1024];
    let n = reader.read_at(0, &mut read_buf).unwrap();
    assert_eq!(n, 32 * 1024);
    for (i, byte) in read_buf.iter().enumerate() {
        assert_eq!(*byte, (i % 256) as u8, "BZIP2 round-trip mismatch at byte {}", i);
    }
}

// =============================================================================
// Large write + read test
// =============================================================================

#[test]
fn test_reader_large_image_round_trip() {
    let size = 1024 * 1024; // 1 MB
    let (_tmp, path) = create_test_image(
        EwfFormat::Encase5,
        size,
        EwfCaseInfo {
            case_number: Some("LARGE-001".to_string()),
            ..Default::default()
        },
        true,
        false,
    );

    let reader = EwfReader::open(&path).unwrap();
    assert_eq!(reader.media_size(), size as u64);

    // Read in chunks and verify
    let chunk_size = 64 * 1024;
    let mut offset = 0i64;
    let mut buf = vec![0u8; chunk_size];

    while (offset as usize) < size {
        let remaining = size - offset as usize;
        let to_read = remaining.min(chunk_size);
        let n = reader.read_at(offset, &mut buf[..to_read]).unwrap();
        assert!(n > 0, "Expected to read data at offset {}", offset);

        for (i, byte) in buf[..n].iter().enumerate() {
            let expected = ((offset as usize + i) % 256) as u8;
            assert_eq!(
                *byte, expected,
                "Mismatch at absolute offset {}",
                offset as usize + i
            );
        }

        offset += n as i64;
    }

    assert_eq!(offset as usize, size);
}
