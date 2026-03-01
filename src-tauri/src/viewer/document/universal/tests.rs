// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

use super::*;

#[test]
fn test_format_detection() {
    assert_eq!(
        UniversalFormat::from_extension("png"),
        Some(UniversalFormat::Png)
    );
    assert_eq!(
        UniversalFormat::from_path("test.PDF"),
        Some(UniversalFormat::Pdf)
    ); // from_path handles case
    assert_eq!(
        UniversalFormat::from_extension("docx"),
        Some(UniversalFormat::Docx)
    );
    assert_eq!(
        UniversalFormat::from_extension("unknown"),
        Some(UniversalFormat::Binary)
    );
}

#[test]
fn test_viewer_type() {
    assert_eq!(UniversalFormat::Png.viewer_type(), ViewerType::Image);
    assert_eq!(UniversalFormat::Pdf.viewer_type(), ViewerType::Pdf);
    assert_eq!(UniversalFormat::Docx.viewer_type(), ViewerType::Office);
    assert_eq!(UniversalFormat::Zip.viewer_type(), ViewerType::Archive);
    // Spreadsheet types route to SpreadsheetViewer
    assert_eq!(UniversalFormat::Csv.viewer_type(), ViewerType::Spreadsheet);
    assert_eq!(UniversalFormat::Xlsx.viewer_type(), ViewerType::Spreadsheet);
    assert_eq!(UniversalFormat::Xls.viewer_type(), ViewerType::Spreadsheet);
    assert_eq!(UniversalFormat::Ods.viewer_type(), ViewerType::Spreadsheet);
}

#[test]
fn test_mime_types() {
    assert_eq!(UniversalFormat::Png.mime_type(), "image/png");
    assert_eq!(UniversalFormat::Pdf.mime_type(), "application/pdf");
    assert_eq!(UniversalFormat::Json.mime_type(), "application/json");
}

// =========================================================================
// Magic-byte detection tests
// =========================================================================

use std::io::Write;
use tempfile::NamedTempFile;

/// Helper: write bytes to a temp file and run detect_by_magic
fn detect_bytes(bytes: &[u8]) -> Option<UniversalFormat> {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(bytes).unwrap();
    f.flush().unwrap();
    UniversalFormat::detect_by_magic(f.path())
}

#[test]
fn test_magic_pdf() {
    assert_eq!(
        detect_bytes(b"%PDF-1.4 some content"),
        Some(UniversalFormat::Pdf)
    );
}

#[test]
fn test_magic_png() {
    let mut header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    header.extend_from_slice(&[0u8; 24]);
    assert_eq!(detect_bytes(&header), Some(UniversalFormat::Png));
}

#[test]
fn test_magic_jpeg() {
    assert_eq!(
        detect_bytes(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10]),
        Some(UniversalFormat::Jpeg)
    );
}

#[test]
fn test_magic_gif() {
    assert_eq!(
        detect_bytes(b"GIF89a\x01\x00\x01\x00"),
        Some(UniversalFormat::Gif)
    );
    assert_eq!(
        detect_bytes(b"GIF87a\x01\x00\x01\x00"),
        Some(UniversalFormat::Gif)
    );
}

#[test]
fn test_magic_bmp() {
    assert_eq!(
        detect_bytes(b"BM\x00\x00\x00\x00\x00\x00"),
        Some(UniversalFormat::Bmp)
    );
}

#[test]
fn test_magic_webp() {
    assert_eq!(
        detect_bytes(b"RIFF\x00\x00\x00\x00WEBP"),
        Some(UniversalFormat::WebP)
    );
}

#[test]
fn test_magic_tiff_le() {
    assert_eq!(
        detect_bytes(&[0x49, 0x49, 0x2A, 0x00, 0x08, 0x00]),
        Some(UniversalFormat::Tiff)
    );
}

#[test]
fn test_magic_tiff_be() {
    assert_eq!(
        detect_bytes(&[0x4D, 0x4D, 0x00, 0x2A, 0x00, 0x08]),
        Some(UniversalFormat::Tiff)
    );
}

#[test]
fn test_magic_zip() {
    assert_eq!(
        detect_bytes(&[0x50, 0x4B, 0x03, 0x04, 0x14, 0x00]),
        Some(UniversalFormat::Zip)
    );
}

#[test]
fn test_magic_7z() {
    assert_eq!(
        detect_bytes(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C, 0x00, 0x04]),
        Some(UniversalFormat::SevenZ)
    );
}

#[test]
fn test_magic_rar() {
    assert_eq!(
        detect_bytes(&[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00]),
        Some(UniversalFormat::Rar)
    );
}

#[test]
fn test_magic_gz() {
    assert_eq!(
        detect_bytes(&[0x1F, 0x8B, 0x08, 0x00]),
        Some(UniversalFormat::Gz)
    );
}

#[test]
fn test_magic_sqlite() {
    assert_eq!(
        detect_bytes(b"SQLite format 3\0extra data here"),
        Some(UniversalFormat::Sqlite)
    );
}

#[test]
fn test_magic_registry_hive() {
    assert_eq!(
        detect_bytes(b"regf\x00\x00\x00\x00"),
        Some(UniversalFormat::RegistryHive)
    );
}

#[test]
fn test_magic_plist() {
    assert_eq!(detect_bytes(b"bplist00\x00"), Some(UniversalFormat::Plist));
}

#[test]
fn test_magic_mz_exe() {
    assert_eq!(
        detect_bytes(b"MZ\x90\x00\x03\x00"),
        Some(UniversalFormat::Exe)
    );
}

#[test]
fn test_magic_elf() {
    assert_eq!(
        detect_bytes(&[0x7F, 0x45, 0x4C, 0x46, 0x02, 0x01]),
        Some(UniversalFormat::Binary)
    );
}

#[test]
fn test_magic_macho_64() {
    // Mach-O 64-bit: 0xFEEDFACF
    assert_eq!(
        detect_bytes(&[0xFE, 0xED, 0xFA, 0xCF, 0x00, 0x00]),
        Some(UniversalFormat::MachO)
    );
}

#[test]
fn test_magic_macho_fat() {
    // Fat/Universal: 0xCAFEBABE
    assert_eq!(
        detect_bytes(&[0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00]),
        Some(UniversalFormat::MachO)
    );
}

#[test]
fn test_magic_json_object() {
    assert_eq!(
        detect_bytes(b"{\"key\": \"value\"}"),
        Some(UniversalFormat::Json)
    );
}

#[test]
fn test_magic_json_array() {
    assert_eq!(detect_bytes(b"[1, 2, 3]"), Some(UniversalFormat::Json));
}

#[test]
fn test_magic_json_with_whitespace() {
    assert_eq!(
        detect_bytes(b"  \n  {\"key\": 1}"),
        Some(UniversalFormat::Json)
    );
}

#[test]
fn test_magic_html() {
    assert_eq!(
        detect_bytes(b"<!DOCTYPE html>"),
        Some(UniversalFormat::Html)
    );
    assert_eq!(detect_bytes(b"<html><body>"), Some(UniversalFormat::Html));
}

#[test]
fn test_magic_xml() {
    assert_eq!(
        detect_bytes(b"<?xml version=\"1.0\"?>"),
        Some(UniversalFormat::Xml)
    );
}

#[test]
fn test_magic_eml() {
    assert_eq!(
        detect_bytes(b"From: user@example.com\r\nTo: other@example.com"),
        Some(UniversalFormat::Eml)
    );
    assert_eq!(
        detect_bytes(b"Received: from mail.example.com"),
        Some(UniversalFormat::Eml)
    );
    assert_eq!(
        detect_bytes(b"MIME-Version: 1.0\r\n"),
        Some(UniversalFormat::Eml)
    );
}

#[test]
fn test_magic_mbox() {
    assert_eq!(
        detect_bytes(b"From user@example.com Mon Jan 1"),
        Some(UniversalFormat::Mbox)
    );
}

#[test]
fn test_magic_csv() {
    assert_eq!(
        detect_bytes(b"name,age,city\nJohn,30,NYC"),
        Some(UniversalFormat::Csv)
    );
}

#[test]
fn test_magic_plain_text() {
    assert_eq!(detect_bytes(b"Hello World"), Some(UniversalFormat::Text));
}

#[test]
fn test_magic_binary_fallback() {
    // Non-text, non-recognized binary data
    assert_eq!(
        detect_bytes(&[0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE]),
        Some(UniversalFormat::Binary)
    );
}

#[test]
fn test_magic_empty_file() {
    let f = NamedTempFile::new().unwrap();
    assert_eq!(UniversalFormat::detect_by_magic(f.path()), None);
}

#[test]
fn test_magic_nonexistent_file() {
    assert_eq!(
        UniversalFormat::detect_by_magic("/nonexistent/path/file.dat"),
        None
    );
}

#[test]
fn test_magic_new_variants() {
    // Verify Sys and RegistryHive enum properties
    assert_eq!(UniversalFormat::Sys.viewer_type(), ViewerType::Binary);
    assert_eq!(
        UniversalFormat::RegistryHive.viewer_type(),
        ViewerType::Registry
    );
    assert_eq!(
        UniversalFormat::Sys.mime_type(),
        "application/x-windows-driver"
    );
    assert_eq!(
        UniversalFormat::RegistryHive.mime_type(),
        "application/x-windows-registry"
    );
    assert_eq!(UniversalFormat::Sys.description(), "Windows Driver");
    assert_eq!(
        UniversalFormat::RegistryHive.description(),
        "Windows Registry Hive"
    );
    assert_eq!(
        UniversalFormat::from_extension("sys"),
        Some(UniversalFormat::Sys)
    );
    assert_eq!(
        UniversalFormat::from_extension("drv"),
        Some(UniversalFormat::Sys)
    );
}

#[test]
fn test_rtf_magic_detection() {
    // RTF files start with {\rtf — should NOT be detected as JSON
    use std::io::Write;
    let dir = std::env::temp_dir().join("test_rtf_magic");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("test.dat"); // no .rtf extension to force magic detection
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(br"{\rtf1\ansi Hello World}").unwrap();
    drop(file);

    let detected = UniversalFormat::detect_by_magic(&path);
    assert_eq!(
        detected,
        Some(UniversalFormat::Rtf),
        "RTF should be detected by magic bytes, not as JSON"
    );

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn test_rtf_viewer_type() {
    assert_eq!(UniversalFormat::Rtf.viewer_type(), ViewerType::Office);
}

#[test]
fn test_pptx_viewer_type() {
    assert_eq!(UniversalFormat::Pptx.viewer_type(), ViewerType::Office);
}

#[test]
fn test_odt_from_extension() {
    assert_eq!(
        UniversalFormat::from_extension("odt"),
        Some(UniversalFormat::Odt)
    );
}

// =========================================================================
// New extension mapping tests
// =========================================================================

#[test]
fn test_xlsm_xlsb_extension() {
    assert_eq!(
        UniversalFormat::from_extension("xlsm"),
        Some(UniversalFormat::Xlsx)
    );
    assert_eq!(
        UniversalFormat::from_extension("xlsb"),
        Some(UniversalFormat::Xlsx)
    );
}

#[test]
fn test_numbers_extension() {
    assert_eq!(
        UniversalFormat::from_extension("numbers"),
        Some(UniversalFormat::Xls)
    );
}

#[test]
fn test_executable_extensions() {
    assert_eq!(
        UniversalFormat::from_extension("com"),
        Some(UniversalFormat::Exe)
    );
    assert_eq!(
        UniversalFormat::from_extension("scr"),
        Some(UniversalFormat::Exe)
    );
    assert_eq!(
        UniversalFormat::from_extension("ocx"),
        Some(UniversalFormat::Exe)
    );
    assert_eq!(
        UniversalFormat::from_extension("cpl"),
        Some(UniversalFormat::Exe)
    );
}

#[test]
fn test_binary_extensions() {
    assert_eq!(
        UniversalFormat::from_extension("bin"),
        Some(UniversalFormat::Binary)
    );
    assert_eq!(
        UniversalFormat::from_extension("elf"),
        Some(UniversalFormat::Binary)
    );
}

// =========================================================================
// New magic-byte detection tests
// =========================================================================

#[test]
fn test_magic_ole_compound() {
    // OLE Compound Document (D0 CF 11 E0 A1 B1 1A E1)
    assert_eq!(
        detect_bytes(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]),
        Some(UniversalFormat::Doc)
    );
}

#[test]
fn test_magic_ico() {
    // ICO file (00 00 01 00)
    assert_eq!(
        detect_bytes(&[0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x20, 0x20]),
        Some(UniversalFormat::Ico)
    );
}

#[test]
fn test_magic_cursor() {
    // CUR file (00 00 02 00) — also detected as Ico
    assert_eq!(
        detect_bytes(&[0x00, 0x00, 0x02, 0x00, 0x01, 0x00, 0x20, 0x20]),
        Some(UniversalFormat::Ico)
    );
}

#[test]
fn test_magic_heic() {
    // HEIC: ftypheic at offset 4
    let mut bytes = vec![0x00, 0x00, 0x00, 0x18]; // box size
    bytes.extend_from_slice(b"ftypheic"); // ftyp + brand
    bytes.extend_from_slice(&[0x00; 20]); // padding
    assert_eq!(detect_bytes(&bytes), Some(UniversalFormat::Heic));
}

#[test]
fn test_magic_heif_mif1() {
    // HEIF with mif1 brand
    let mut bytes = vec![0x00, 0x00, 0x00, 0x1C];
    bytes.extend_from_slice(b"ftypmif1");
    bytes.extend_from_slice(&[0x00; 20]);
    assert_eq!(detect_bytes(&bytes), Some(UniversalFormat::Heic));
}

#[test]
fn test_magic_tar() {
    // TAR: "ustar" at offset 257
    let mut bytes = vec![0u8; 263]; // at least 262 bytes
    bytes[257] = b'u';
    bytes[258] = b's';
    bytes[259] = b't';
    bytes[260] = b'a';
    bytes[261] = b'r';
    assert_eq!(detect_bytes(&bytes), Some(UniversalFormat::Tar));
}

// =========================================================================
// AVIF and RAW image tests
// =========================================================================

#[test]
fn test_avif_from_extension() {
    assert_eq!(
        UniversalFormat::from_extension("avif"),
        Some(UniversalFormat::Avif)
    );
}

#[test]
fn test_avif_viewer_type() {
    assert_eq!(UniversalFormat::Avif.viewer_type(), ViewerType::Image);
}

#[test]
fn test_avif_mime_type() {
    assert_eq!(UniversalFormat::Avif.mime_type(), "image/avif");
}

#[test]
fn test_avif_description() {
    assert_eq!(UniversalFormat::Avif.description(), "AVIF Image");
}

#[test]
fn test_magic_avif() {
    // AVIF: ftypavif at offset 4
    let mut bytes = vec![0x00, 0x00, 0x00, 0x20]; // box size
    bytes.extend_from_slice(b"ftypavif"); // ftyp + brand
    bytes.extend_from_slice(&[0x00; 20]); // padding
    assert_eq!(detect_bytes(&bytes), Some(UniversalFormat::Avif));
}

#[test]
fn test_magic_avif_sequence() {
    // AVIF sequence: ftypavis at offset 4
    let mut bytes = vec![0x00, 0x00, 0x00, 0x20];
    bytes.extend_from_slice(b"ftypavis");
    bytes.extend_from_slice(&[0x00; 20]);
    assert_eq!(detect_bytes(&bytes), Some(UniversalFormat::Avif));
}

#[test]
fn test_raw_image_extensions() {
    assert_eq!(
        UniversalFormat::from_extension("raw"),
        Some(UniversalFormat::RawImage)
    );
    assert_eq!(
        UniversalFormat::from_extension("cr2"),
        Some(UniversalFormat::RawImage)
    );
    assert_eq!(
        UniversalFormat::from_extension("nef"),
        Some(UniversalFormat::RawImage)
    );
    assert_eq!(
        UniversalFormat::from_extension("arw"),
        Some(UniversalFormat::RawImage)
    );
    assert_eq!(
        UniversalFormat::from_extension("dng"),
        Some(UniversalFormat::RawImage)
    );
    assert_eq!(
        UniversalFormat::from_extension("orf"),
        Some(UniversalFormat::RawImage)
    );
    assert_eq!(
        UniversalFormat::from_extension("rw2"),
        Some(UniversalFormat::RawImage)
    );
}

#[test]
fn test_raw_image_viewer_type() {
    assert_eq!(UniversalFormat::RawImage.viewer_type(), ViewerType::Image);
}

#[test]
fn test_raw_image_description() {
    assert_eq!(UniversalFormat::RawImage.description(), "RAW Camera Image");
}

#[test]
fn test_db3_sqlitedb_extensions() {
    assert_eq!(
        UniversalFormat::from_extension("db3"),
        Some(UniversalFormat::Db)
    );
    assert_eq!(
        UniversalFormat::from_extension("sqlitedb"),
        Some(UniversalFormat::Sqlite)
    );
}
