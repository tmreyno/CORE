// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive type definitions
//!
//! Contains shared types for archive format detection and metadata extraction.
//!
//! ## Archive vs Forensic Container Formats
//!
//! | Aspect | Archives | Forensic Containers |
//! |--------|----------|---------------------|
//! | Purpose | Compression/bundling | Evidence preservation |
//! | Integrity | CRC32 (error detection) | Cryptographic hashes |
//! | Metadata | Basic (name, size, date) | Rich (case, examiner, notes) |
//! | Chain of Custody | No | Yes |
//! | Examples | ZIP, 7z, RAR | E01, AD1, AFF |

use serde::Serialize;

/// Archive format type
///
/// Note: This enum includes both true archives (compression-focused) and
/// disk image formats (VM, forensic) that share detection patterns.
/// True forensic containers (E01, AD1, L01) are handled by dedicated modules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ArchiveFormat {
    // =========================================================================
    // Compression Formats
    // =========================================================================
    SevenZip,
    Zip,
    Zip64,
    Rar4,
    Rar5,
    Gzip,
    Tar,
    TarGz,
    Xz,
    Bzip2,
    Lz4,
    Zstd,

    // =========================================================================
    // Forensic Archive Formats (not full forensic containers)
    // =========================================================================
    /// Advanced Forensic Format (open source)
    Aff,
    /// AFF4 - Modern AFF using ZIP container
    Aff4,
    /// SMART format (.s01) - ASR Data SMART
    Smart,

    // =========================================================================
    // Optical Disc Formats
    // =========================================================================
    /// ISO 9660 - CD/DVD disc image
    Iso,

    // =========================================================================
    // Virtual Machine Disk Formats
    // =========================================================================
    /// VMware Virtual Disk
    Vmdk,
    /// Microsoft Virtual Hard Disk (legacy)
    Vhd,
    /// Microsoft Virtual Hard Disk (modern)
    Vhdx,
    /// QEMU Copy-On-Write v2
    Qcow2,
    /// VirtualBox Virtual Disk Image
    Vdi,

    // =========================================================================
    // macOS Formats
    // =========================================================================
    /// Apple Disk Image
    Dmg,
}

impl std::fmt::Display for ArchiveFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Compression
            ArchiveFormat::SevenZip => write!(f, "7-Zip"),
            ArchiveFormat::Zip => write!(f, "ZIP"),
            ArchiveFormat::Zip64 => write!(f, "ZIP64"),
            ArchiveFormat::Rar4 => write!(f, "RAR4"),
            ArchiveFormat::Rar5 => write!(f, "RAR5"),
            ArchiveFormat::Gzip => write!(f, "GZIP"),
            ArchiveFormat::Tar => write!(f, "TAR"),
            ArchiveFormat::TarGz => write!(f, "TAR.GZ"),
            ArchiveFormat::Xz => write!(f, "XZ"),
            ArchiveFormat::Bzip2 => write!(f, "BZIP2"),
            ArchiveFormat::Lz4 => write!(f, "LZ4"),
            ArchiveFormat::Zstd => write!(f, "ZSTD"),
            // Forensic archives
            ArchiveFormat::Aff => write!(f, "AFF"),
            ArchiveFormat::Aff4 => write!(f, "AFF4"),
            ArchiveFormat::Smart => write!(f, "SMART"),
            // Optical disc
            ArchiveFormat::Iso => write!(f, "ISO 9660"),
            // Virtual machine
            ArchiveFormat::Vmdk => write!(f, "VMDK"),
            ArchiveFormat::Vhd => write!(f, "VHD"),
            ArchiveFormat::Vhdx => write!(f, "VHDX"),
            ArchiveFormat::Qcow2 => write!(f, "QCOW2"),
            ArchiveFormat::Vdi => write!(f, "VDI"),
            // macOS
            ArchiveFormat::Dmg => write!(f, "DMG"),
        }
    }
}

/// Archive information
#[derive(Debug, Clone, Default, Serialize)]
pub struct ArchiveInfo {
    pub format: String,
    pub segment_count: u32,
    pub total_size: u64,
    pub segment_names: Vec<String>,
    pub segment_sizes: Vec<u64>,
    pub first_segment: String,
    pub last_segment: String,
    pub is_multipart: bool,
    /// Number of entries in the archive (from Central Directory for ZIP)
    pub entry_count: Option<u32>,
    /// Whether archive has encrypted headers (filenames hidden)
    pub encrypted_headers: bool,
    /// Whether archive uses AES encryption
    pub aes_encrypted: bool,
    /// ZIP-specific: Central Directory offset
    pub central_dir_offset: Option<u64>,
    /// ZIP-specific: Central Directory size
    pub central_dir_size: Option<u32>,
    /// 7z-specific: Next header offset (absolute, from file start)
    pub next_header_offset: Option<u64>,
    /// 7z-specific: Next header size
    pub next_header_size: Option<u64>,
    /// 7z-specific: Archive version (major.minor)
    pub version: Option<String>,
    /// 7z-specific: Start Header CRC valid
    pub start_header_crc_valid: Option<bool>,
    /// 7z-specific: Next Header CRC (for reference)
    pub next_header_crc: Option<u32>,
    /// UFED extraction detected (UFDR/UFDX/UFD)
    pub ufed_detected: bool,
    /// UFED file paths found inside archive
    pub ufed_files: Vec<String>,
}

impl ArchiveInfo {
    /// Create a new ArchiveInfo with format and single segment
    pub fn new(format: impl Into<String>, path: impl Into<String>, size: u64) -> Self {
        let path_str = path.into();
        Self {
            format: format.into(),
            segment_count: 1,
            total_size: size,
            segment_names: vec![path_str.clone()],
            segment_sizes: vec![size],
            first_segment: path_str.clone(),
            last_segment: path_str,
            is_multipart: false,
            ..Default::default()
        }
    }

    /// Create a new multipart archive info
    pub fn multipart(format: impl Into<String>, segments: Vec<(String, u64)>) -> Self {
        let count = segments.len() as u32;
        let total_size: u64 = segments.iter().map(|(_, s)| s).sum();
        let names: Vec<String> = segments.iter().map(|(n, _)| n.clone()).collect();
        let sizes: Vec<u64> = segments.iter().map(|(_, s)| *s).collect();
        let first = names.first().cloned().unwrap_or_default();
        let last = names.last().cloned().unwrap_or_default();

        Self {
            format: format.into(),
            segment_count: count,
            total_size,
            segment_names: names,
            segment_sizes: sizes,
            first_segment: first,
            last_segment: last,
            is_multipart: count > 1,
            ..Default::default()
        }
    }

    /// Set encryption info
    #[inline]
    pub fn with_encryption(mut self, aes: bool, encrypted_headers: bool) -> Self {
        self.aes_encrypted = aes;
        self.encrypted_headers = encrypted_headers;
        self
    }

    /// Set entry count
    #[inline]
    pub fn with_entry_count(mut self, count: u32) -> Self {
        self.entry_count = Some(count);
        self
    }

    /// Set ZIP-specific central directory info
    #[inline]
    pub fn with_central_dir(mut self, offset: u64, size: u32) -> Self {
        self.central_dir_offset = Some(offset);
        self.central_dir_size = Some(size);
        self
    }

    /// Set 7z-specific header info
    #[inline]
    pub fn with_7z_headers(
        mut self,
        next_offset: u64,
        next_size: u64,
        version: impl Into<String>,
    ) -> Self {
        self.next_header_offset = Some(next_offset);
        self.next_header_size = Some(next_size);
        self.version = Some(version.into());
        self
    }

    /// Set UFED detection info
    #[inline]
    pub fn with_ufed(mut self, files: Vec<String>) -> Self {
        self.ufed_detected = !files.is_empty();
        self.ufed_files = files;
        self
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_format_display_compression() {
        assert_eq!(format!("{}", ArchiveFormat::SevenZip), "7-Zip");
        assert_eq!(format!("{}", ArchiveFormat::Zip), "ZIP");
        assert_eq!(format!("{}", ArchiveFormat::Zip64), "ZIP64");
        assert_eq!(format!("{}", ArchiveFormat::Rar4), "RAR4");
        assert_eq!(format!("{}", ArchiveFormat::Rar5), "RAR5");
        assert_eq!(format!("{}", ArchiveFormat::Gzip), "GZIP");
        assert_eq!(format!("{}", ArchiveFormat::Tar), "TAR");
        assert_eq!(format!("{}", ArchiveFormat::TarGz), "TAR.GZ");
        assert_eq!(format!("{}", ArchiveFormat::Xz), "XZ");
        assert_eq!(format!("{}", ArchiveFormat::Bzip2), "BZIP2");
        assert_eq!(format!("{}", ArchiveFormat::Lz4), "LZ4");
        assert_eq!(format!("{}", ArchiveFormat::Zstd), "ZSTD");
    }

    #[test]
    fn test_archive_format_display_forensic() {
        assert_eq!(format!("{}", ArchiveFormat::Aff), "AFF");
        assert_eq!(format!("{}", ArchiveFormat::Aff4), "AFF4");
        assert_eq!(format!("{}", ArchiveFormat::Smart), "SMART");
    }

    #[test]
    fn test_archive_format_display_optical() {
        assert_eq!(format!("{}", ArchiveFormat::Iso), "ISO 9660");
    }

    #[test]
    fn test_archive_format_display_vm() {
        assert_eq!(format!("{}", ArchiveFormat::Vmdk), "VMDK");
        assert_eq!(format!("{}", ArchiveFormat::Vhd), "VHD");
        assert_eq!(format!("{}", ArchiveFormat::Vhdx), "VHDX");
        assert_eq!(format!("{}", ArchiveFormat::Qcow2), "QCOW2");
        assert_eq!(format!("{}", ArchiveFormat::Vdi), "VDI");
    }

    #[test]
    fn test_archive_format_display_macos() {
        assert_eq!(format!("{}", ArchiveFormat::Dmg), "DMG");
    }

    #[test]
    fn test_archive_format_equality() {
        assert_eq!(ArchiveFormat::Zip, ArchiveFormat::Zip);
        assert_ne!(ArchiveFormat::Zip, ArchiveFormat::Zip64);
        assert_ne!(ArchiveFormat::Rar4, ArchiveFormat::Rar5);
    }

    #[test]
    fn test_archive_info_default() {
        let info = ArchiveInfo::default();
        assert!(info.format.is_empty());
        assert_eq!(info.segment_count, 0);
        assert_eq!(info.total_size, 0);
        assert!(info.segment_names.is_empty());
        assert!(!info.is_multipart);
        assert!(!info.encrypted_headers);
        assert!(!info.aes_encrypted);
        assert!(info.entry_count.is_none());
        assert!(!info.ufed_detected);
    }

    #[test]
    fn test_archive_info_zip() {
        let info = ArchiveInfo {
            format: "ZIP".to_string(),
            segment_count: 1,
            total_size: 1024,
            segment_names: vec!["archive.zip".to_string()],
            segment_sizes: vec![1024],
            first_segment: "archive.zip".to_string(),
            last_segment: "archive.zip".to_string(),
            is_multipart: false,
            entry_count: Some(10),
            encrypted_headers: false,
            aes_encrypted: false,
            central_dir_offset: Some(800),
            central_dir_size: Some(200),
            next_header_offset: None,
            next_header_size: None,
            version: None,
            start_header_crc_valid: None,
            next_header_crc: None,
            ufed_detected: false,
            ufed_files: vec![],
        };

        assert_eq!(info.format, "ZIP");
        assert_eq!(info.entry_count, Some(10));
        assert!(info.central_dir_offset.is_some());
    }

    #[test]
    fn test_archive_info_multipart_7z() {
        let info = ArchiveInfo {
            format: "7-Zip".to_string(),
            segment_count: 3,
            total_size: 3072,
            segment_names: vec![
                "archive.7z.001".to_string(),
                "archive.7z.002".to_string(),
                "archive.7z.003".to_string(),
            ],
            segment_sizes: vec![1024, 1024, 1024],
            first_segment: "archive.7z.001".to_string(),
            last_segment: "archive.7z.003".to_string(),
            is_multipart: true,
            entry_count: None,
            encrypted_headers: true,
            aes_encrypted: true,
            central_dir_offset: None,
            central_dir_size: None,
            next_header_offset: Some(100),
            next_header_size: Some(256),
            version: Some("0.4".to_string()),
            start_header_crc_valid: Some(true),
            next_header_crc: Some(0xABCDEF12),
            ufed_detected: false,
            ufed_files: vec![],
        };

        assert!(info.is_multipart);
        assert_eq!(info.segment_count, 3);
        assert!(info.encrypted_headers);
        assert!(info.aes_encrypted);
        assert_eq!(info.version, Some("0.4".to_string()));
    }

    #[test]
    fn test_archive_info_with_ufed() {
        let info = ArchiveInfo {
            format: "ZIP".to_string(),
            segment_count: 1,
            total_size: 5000,
            segment_names: vec!["extraction.zip".to_string()],
            segment_sizes: vec![5000],
            first_segment: "extraction.zip".to_string(),
            last_segment: "extraction.zip".to_string(),
            is_multipart: false,
            entry_count: Some(100),
            encrypted_headers: false,
            aes_encrypted: false,
            central_dir_offset: None,
            central_dir_size: None,
            next_header_offset: None,
            next_header_size: None,
            version: None,
            start_header_crc_valid: None,
            next_header_crc: None,
            ufed_detected: true,
            ufed_files: vec![
                "extraction.ufd".to_string(),
                "EvidenceCollection.ufdx".to_string(),
            ],
        };

        assert!(info.ufed_detected);
        assert_eq!(info.ufed_files.len(), 2);
    }
}
