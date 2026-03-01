// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive format detection
//!
//! Provides magic signature detection for various archive formats.
//! Used to identify archive types before parsing.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use tracing::{debug, instrument, trace};

use super::types::ArchiveFormat;
use crate::containers::ContainerError;

// =============================================================================
// Magic Signatures Reference
// =============================================================================
// Format  | Magic Bytes              | Hex                      | Location
// --------|--------------------------|--------------------------|----------
// 7-Zip   | 7z¼¯'\x1c               | 37 7A BC AF 27 1C        | Offset 0
// ZIP     | PK\x03\x04              | 50 4B 03 04              | Offset 0
// ZIP64   | Same + PK\x06\x07       | 50 4B 06 07              | Near end
// RAR4    | Rar!\x1a\x07\x00        | 52 61 72 21 1A 07 00     | Offset 0
// RAR5    | Rar!\x1a\x07\x01\x00    | 52 61 72 21 1A 07 01 00  | Offset 0
// GZIP    | \x1f\x8b                | 1F 8B                    | Offset 0
// TAR     | ustar                   | 75 73 74 61 72 (ASCII)   | Offset 257
// XZ      | \xfd7zXZ\x00            | FD 37 7A 58 5A 00        | Offset 0
// BZIP2   | BZ                      | 42 5A                    | Offset 0
// LZ4     | \x04\x22M\x18           | 04 22 4D 18              | Offset 0
// ZSTD    | \x28\xb5\x2f\xfd        | 28 B5 2F FD              | Offset 0
// AFF     | AFF                     | 41 46 46                 | Offset 0
// AFF4    | (ZIP-based container)   | 50 4B 03 04              | Offset 0
// SMART   | SMART                   | 53 4D 41 52 54           | Offset 0
// VMDK    | KDMV                    | 4B 44 4D 56              | Offset 0
// VHD     | conectix                | 63 6F 6E 65 63 74 69 78  | EOF-512
// VHDX    | vhdxfile                | 76 68 64 78 66 69 6C 65  | Offset 0
// QCOW2   | QFI\xfb                 | 51 46 49 FB              | Offset 0
// VDI     | <<< Oracle VM...        | 3C 3C 3C 20 4F 72 ...    | Offset 0
// DMG     | koly                    | 6B 6F 6C 79              | EOF-512
// ISO     | CD001                   | 43 44 30 30 31           | Offset 32769
// =============================================================================

// ZIP signatures
pub const ZIP_LOCAL_HEADER_SIG: &[u8] = &[0x50, 0x4B, 0x03, 0x04]; // PK\x03\x04
#[allow(dead_code)]
pub const ZIP_CENTRAL_DIR_SIG: &[u8] = &[0x50, 0x4B, 0x01, 0x02]; // PK\x01\x02
pub const ZIP_EOCD_SIG: &[u8] = &[0x50, 0x4B, 0x05, 0x06]; // PK\x05\x06
pub const ZIP64_EOCD_LOC_SIG: &[u8] = &[0x50, 0x4B, 0x06, 0x07]; // ZIP64 EOCD Locator
#[allow(dead_code)]
pub const ZIP64_EOCD_SIG: &[u8] = &[0x50, 0x4B, 0x06, 0x06]; // ZIP64 EOCD

// 7-Zip signature
pub const SEVEN_ZIP_MAGIC: &[u8] = &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];

// RAR signatures (distinguish RAR4 vs RAR5)
pub const RAR4_MAGIC: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00]; // Rar!...
pub const RAR5_MAGIC: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00]; // Rar!....

// Compression formats
pub const GZIP_MAGIC: &[u8] = &[0x1F, 0x8B];
pub const TAR_MAGIC: &[u8] = b"ustar"; // TAR (at offset 257)
pub const XZ_MAGIC: &[u8] = &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]; // \xfd7zXZ\x00
pub const BZIP2_MAGIC: &[u8] = &[0x42, 0x5A, 0x68]; // BZh (h = compression level)
pub const LZ4_MAGIC: &[u8] = &[0x04, 0x22, 0x4D, 0x18];
pub const ZSTD_MAGIC: &[u8] = &[0x28, 0xB5, 0x2F, 0xFD];

// Forensic formats
pub const AFF_MAGIC: &[u8] = b"AFF"; // Advanced Forensic Format
pub const AFF4_CONTAINER_FILE: &str = "container.description"; // AFF4 marker file in ZIP
pub const SMART_MAGIC: &[u8] = b"SMART"; // ASR Data SMART format (.s01)

// Virtual machine formats
pub const VMDK_MAGIC: &[u8] = &[0x4B, 0x44, 0x4D, 0x56]; // KDMV (sparse)
pub const VMDK_DESCRIPTOR_MAGIC: &[u8] = b"# Disk DescriptorFile"; // Text descriptor
pub const VHD_MAGIC: &[u8] = b"conectix"; // At EOF-512
pub const VHDX_MAGIC: &[u8] = b"vhdxfile";
pub const QCOW2_MAGIC: &[u8] = &[0x51, 0x46, 0x49, 0xFB]; // QFI\xfb
pub const VDI_MAGIC: &[u8] = b"<<< Oracle VM VirtualBox Disk Image >>>"; // VirtualBox VDI

// macOS formats
pub const DMG_MAGIC: &[u8] = b"koly"; // At EOF-512 (trailer)

// Optical disc formats
pub const ISO_MAGIC: &[u8] = b"CD001"; // ISO 9660 at offset 32769 (0x8001)

// =============================================================================
// Detection Functions
// =============================================================================

/// Check if a file is an archive format
#[instrument]
pub fn is_archive(path: &str) -> Result<bool, ContainerError> {
    let lower = path.to_lowercase();

    // Quick extension check first - compression formats
    if lower.ends_with(".7z")
        || lower.ends_with(".7z.001")
        || lower.ends_with(".zip")
        || lower.ends_with(".zip.001")
        || lower.ends_with(".rar")
        || lower.ends_with(".r00")
        || lower.ends_with(".r01")
        || lower.ends_with(".gz")
        || lower.ends_with(".gzip")
        || lower.ends_with(".tar")
        || lower.ends_with(".tar.gz")
        || lower.ends_with(".tgz")
        || lower.ends_with(".xz")
        || lower.ends_with(".tar.xz")
        || lower.ends_with(".txz")
        || lower.ends_with(".bz2")
        || lower.ends_with(".tar.bz2")
        || lower.ends_with(".tbz2")
        || lower.ends_with(".lz4")
        || lower.ends_with(".tar.lz4")
        || lower.ends_with(".zst")
        || lower.ends_with(".zstd")
        || lower.ends_with(".tar.zst")
    {
        trace!("Matched archive by extension (compression)");
        return Ok(true);
    }

    // Forensic formats
    if lower.ends_with(".aff")
        || lower.ends_with(".afd")
        || lower.ends_with(".aff4")
        || lower.ends_with(".s01")
        || lower.ends_with(".s02")
    // SMART format
    {
        trace!("Matched archive by extension (forensic)");
        return Ok(true);
    }

    // Virtual machine formats
    if lower.ends_with(".vmdk")
        || lower.ends_with(".vhd")
        || lower.ends_with(".vhdx")
        || lower.ends_with(".qcow2")
        || lower.ends_with(".qcow")
        || lower.ends_with(".vdi")
    // VirtualBox
    {
        trace!("Matched archive by extension (VM)");
        return Ok(true);
    }

    // macOS formats
    if lower.ends_with(".dmg")
        || lower.ends_with(".sparsebundle")
        || lower.ends_with(".sparseimage")
    {
        trace!("Matched archive by extension (macOS)");
        return Ok(true);
    }

    // Optical disc formats
    if lower.ends_with(".iso") || lower.ends_with(".bin") || lower.ends_with(".cue") {
        trace!("Matched archive by extension (optical disc)");
        return Ok(true);
    }

    // Check for numbered 7z segments (.001, .002, etc. after .7z base)
    if is_7z_segment(&lower) {
        trace!("Matched as 7z segment");
        return Ok(true);
    }

    // Signature check for ambiguous extensions
    match detect_archive_format(path) {
        Ok(Some(format)) => {
            trace!(?format, "Matched archive by signature");
            Ok(true)
        }
        Ok(None) => Ok(false),
        Err(_) => Ok(false),
    }
}

/// Detect archive format from file signature
#[instrument]
pub fn detect_archive_format(path: &str) -> Result<Option<ArchiveFormat>, ContainerError> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {e}"))?;

    let file_size = file
        .metadata()
        .map_err(|e| format!("Failed to get file size: {e}"))?
        .len();

    let mut header = [0u8; 262];
    let bytes_read = file
        .read(&mut header)
        .map_err(|e| format!("Failed to read file header: {e}"))?;

    if bytes_read < 2 {
        return Ok(None);
    }

    // =========================================================================
    // Compression Formats (check at offset 0)
    // =========================================================================

    // Check 7z signature (37 7A BC AF 27 1C)
    if bytes_read >= 6 && header[..6] == *SEVEN_ZIP_MAGIC {
        debug!("Detected 7-Zip format");
        return Ok(Some(ArchiveFormat::SevenZip));
    }

    // Check XZ signature (FD 37 7A 58 5A 00)
    if bytes_read >= 6 && header[..6] == *XZ_MAGIC {
        debug!("Detected XZ format");
        return Ok(Some(ArchiveFormat::Xz));
    }

    // Check RAR5 first (longer signature) - 52 61 72 21 1A 07 01 00
    if bytes_read >= 8 && header[..8] == *RAR5_MAGIC {
        debug!("Detected RAR5 format");
        return Ok(Some(ArchiveFormat::Rar5));
    }

    // Check RAR4 - 52 61 72 21 1A 07 00
    if bytes_read >= 7 && header[..7] == *RAR4_MAGIC {
        debug!("Detected RAR4 format");
        return Ok(Some(ArchiveFormat::Rar4));
    }

    // Check ZSTD signature (28 B5 2F FD)
    if bytes_read >= 4 && header[..4] == *ZSTD_MAGIC {
        debug!("Detected ZSTD format");
        return Ok(Some(ArchiveFormat::Zstd));
    }

    // Check LZ4 signature (04 22 4D 18)
    if bytes_read >= 4 && header[..4] == *LZ4_MAGIC {
        debug!("Detected LZ4 format");
        return Ok(Some(ArchiveFormat::Lz4));
    }

    // Check ZIP signatures (PK..) - also used by AFF4
    if bytes_read >= 4 && (header[..4] == *ZIP_LOCAL_HEADER_SIG || header[..4] == *ZIP_EOCD_SIG) {
        // Check if AFF4 (ZIP-based container with marker file)
        if let Ok(true) = check_aff4(&mut file) {
            debug!("Detected AFF4 format");
            return Ok(Some(ArchiveFormat::Aff4));
        }
        // Check if ZIP64 by looking for ZIP64 EOCD
        if let Ok(is_zip64) = check_zip64(&mut file) {
            if is_zip64 {
                debug!("Detected ZIP64 format");
                return Ok(Some(ArchiveFormat::Zip64));
            }
        }
        debug!("Detected ZIP format");
        return Ok(Some(ArchiveFormat::Zip));
    }

    // Check BZIP2 signature (42 5A 68) - BZh
    if bytes_read >= 3 && header[..3] == *BZIP2_MAGIC {
        debug!("Detected BZIP2 format");
        return Ok(Some(ArchiveFormat::Bzip2));
    }

    // Check AFF signature (41 46 46 = "AFF")
    if bytes_read >= 3 && header[..3] == *AFF_MAGIC {
        debug!("Detected AFF format");
        return Ok(Some(ArchiveFormat::Aff));
    }

    // Check SMART signature (53 4D 41 52 54 = "SMART")
    if bytes_read >= 5 && header[..5] == *SMART_MAGIC {
        debug!("Detected SMART format");
        return Ok(Some(ArchiveFormat::Smart));
    }

    // Check GZIP signature (1F 8B)
    if bytes_read >= 2 && header[..2] == *GZIP_MAGIC {
        debug!("Detected GZIP format");
        return Ok(Some(ArchiveFormat::Gzip));
    }

    // Check TAR signature ("ustar" at offset 257)
    if bytes_read >= 262 && &header[257..262] == TAR_MAGIC {
        debug!("Detected TAR format");
        return Ok(Some(ArchiveFormat::Tar));
    }

    // =========================================================================
    // Virtual Machine Formats
    // =========================================================================

    // Check QCOW2 signature (51 46 49 FB)
    if bytes_read >= 4 && header[..4] == *QCOW2_MAGIC {
        debug!("Detected QCOW2 format");
        return Ok(Some(ArchiveFormat::Qcow2));
    }

    // Check VMDK sparse signature (4B 44 4D 56 = KDMV)
    if bytes_read >= 4 && header[..4] == *VMDK_MAGIC {
        debug!("Detected VMDK format (sparse)");
        return Ok(Some(ArchiveFormat::Vmdk));
    }

    // Check VMDK descriptor (text-based)
    if bytes_read >= 21 && &header[..21] == VMDK_DESCRIPTOR_MAGIC {
        debug!("Detected VMDK format (descriptor)");
        return Ok(Some(ArchiveFormat::Vmdk));
    }

    // Check VHDX signature (76 68 64 78 66 69 6C 65 = "vhdxfile")
    if bytes_read >= 8 && header[..8] == *VHDX_MAGIC {
        debug!("Detected VHDX format");
        return Ok(Some(ArchiveFormat::Vhdx));
    }

    // Check VDI signature ("<<< Oracle VM VirtualBox Disk Image >>>")
    if bytes_read >= VDI_MAGIC.len() && header[..VDI_MAGIC.len()] == *VDI_MAGIC {
        debug!("Detected VDI format");
        return Ok(Some(ArchiveFormat::Vdi));
    }

    // =========================================================================
    // Formats with trailer signatures (check at EOF)
    // =========================================================================

    // VHD and DMG have signatures at end of file
    if file_size >= 512 {
        if let Ok(Some(format)) = check_trailer_signature(&mut file, file_size) {
            return Ok(Some(format));
        }
    }

    // =========================================================================
    // Formats with signatures at specific offsets
    // =========================================================================

    // ISO 9660: "CD001" at offset 32769 (0x8001) - primary volume descriptor
    if file_size >= 32774 {
        if let Ok(Some(format)) = check_iso_signature(&mut file) {
            return Ok(Some(format));
        }
    }

    Ok(None)
}

/// Check trailer signatures for VHD and DMG (signature at end of file)
fn check_trailer_signature(
    file: &mut File,
    _file_size: u64,
) -> Result<Option<ArchiveFormat>, ContainerError> {
    let mut trailer = [0u8; 512];

    file.seek(SeekFrom::End(-512))
        .map_err(|e| format!("Failed to seek to trailer: {e}"))?;
    file.read_exact(&mut trailer)
        .map_err(|e| format!("Failed to read trailer: {e}"))?;

    // VHD: "conectix" at start of 512-byte footer
    if &trailer[..8] == VHD_MAGIC {
        debug!("Detected VHD format");
        return Ok(Some(ArchiveFormat::Vhd));
    }

    // DMG: "koly" at start of 512-byte trailer (actually last 512 bytes)
    if &trailer[..4] == DMG_MAGIC {
        debug!("Detected DMG format");
        return Ok(Some(ArchiveFormat::Dmg));
    }

    // Reset file position
    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("Failed to reset file position: {e}"))?;

    Ok(None)
}

/// Check ISO 9660 signature at offset 32769 (0x8001)
fn check_iso_signature(file: &mut File) -> Result<Option<ArchiveFormat>, ContainerError> {
    let mut iso_header = [0u8; 5];

    // ISO 9660 primary volume descriptor starts at offset 32768 (0x8000)
    // The signature "CD001" is at offset 32769 (0x8001)
    file.seek(SeekFrom::Start(32769))
        .map_err(|e| format!("Failed to seek to ISO header: {e}"))?;
    file.read_exact(&mut iso_header)
        .map_err(|e| format!("Failed to read ISO header: {e}"))?;

    if iso_header == ISO_MAGIC {
        debug!("Detected ISO 9660 format");
        return Ok(Some(ArchiveFormat::Iso));
    }

    // Reset file position
    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("Failed to reset file position: {e}"))?;

    Ok(None)
}

/// Check if a ZIP file is an AFF4 container
fn check_aff4(file: &mut File) -> Result<bool, ContainerError> {
    // Reset to start
    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("Failed to seek: {e}"))?;

    // Try to open as ZIP and look for AFF4 marker
    // AFF4 containers have a "container.description" file
    let reader = std::io::BufReader::new(
        file.try_clone()
            .map_err(|e| format!("Failed to clone file handle: {e}"))?,
    );

    if let Ok(mut archive) = zip::ZipArchive::new(reader) {
        for i in 0..archive.len().min(20) {
            // Check first 20 entries
            if let Ok(entry) = archive.by_index(i) {
                let name = entry.name().to_lowercase();
                if name == "container.description" || name.contains("aff4") {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

/// Check if a ZIP file is ZIP64 format
fn check_zip64(file: &mut File) -> Result<bool, ContainerError> {
    let size = file
        .metadata()
        .map_err(|e| format!("Failed to get file size: {e}"))?
        .len();

    // Search backwards for ZIP64 EOCD Locator (PK\x06\x07)
    let search_size = size.min(65557 + 20) as usize; // EOCD max + ZIP64 locator
    let mut buf = vec![0u8; search_size];

    file.seek(SeekFrom::End(-(search_size as i64)))
        .map_err(|e| format!("Failed to seek: {e}"))?;
    file.read_exact(&mut buf)
        .map_err(|e| format!("Failed to read: {e}"))?;

    // Look for ZIP64 EOCD Locator signature
    for i in (0..buf.len().saturating_sub(4)).rev() {
        if &buf[i..i + 4] == ZIP64_EOCD_LOC_SIG {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Check if filename is a 7z numbered segment
pub fn is_7z_segment(lower: &str) -> bool {
    // Match patterns like: file.7z.001, file.7z.002, etc.
    if let Some(pos) = lower.rfind(".7z.") {
        let suffix = &lower[pos + 4..];
        return suffix.chars().all(|c| c.is_ascii_digit()) && !suffix.is_empty();
    }
    false
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test magic signature constants
    #[test]
    fn test_magic_signatures_7zip() {
        assert_eq!(SEVEN_ZIP_MAGIC.len(), 6);
        assert_eq!(SEVEN_ZIP_MAGIC, &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]);
    }

    #[test]
    fn test_magic_signatures_zip() {
        assert_eq!(ZIP_LOCAL_HEADER_SIG, &[0x50, 0x4B, 0x03, 0x04]);
        assert_eq!(ZIP_EOCD_SIG, &[0x50, 0x4B, 0x05, 0x06]);
        assert_eq!(ZIP64_EOCD_LOC_SIG, &[0x50, 0x4B, 0x06, 0x07]);
    }

    #[test]
    fn test_magic_signatures_rar() {
        // RAR4: Rar!\x1a\x07\x00
        assert_eq!(RAR4_MAGIC, &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00]);
        // RAR5: Rar!\x1a\x07\x01\x00
        assert_eq!(
            RAR5_MAGIC,
            &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00]
        );
    }

    #[test]
    fn test_magic_signatures_compression() {
        // GZIP: 1F 8B
        assert_eq!(GZIP_MAGIC, &[0x1F, 0x8B]);
        // BZIP2: 42 5A 68 (BZh)
        assert_eq!(BZIP2_MAGIC, &[0x42, 0x5A, 0x68]);
        // XZ: FD 37 7A 58 5A 00
        assert_eq!(XZ_MAGIC, &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]);
        // ZSTD: 28 B5 2F FD
        assert_eq!(ZSTD_MAGIC, &[0x28, 0xB5, 0x2F, 0xFD]);
        // LZ4: 04 22 4D 18
        assert_eq!(LZ4_MAGIC, &[0x04, 0x22, 0x4D, 0x18]);
    }

    #[test]
    fn test_magic_signatures_disk_images() {
        // QCOW2: QFI\xfb
        assert_eq!(QCOW2_MAGIC, &[0x51, 0x46, 0x49, 0xFB]);
        // VMDK: KDMV
        assert_eq!(VMDK_MAGIC, &[0x4B, 0x44, 0x4D, 0x56]);
        // VHDX: vhdxfile
        assert_eq!(VHDX_MAGIC, b"vhdxfile");
        // VDI: <<< Oracle VM...
        assert!(VDI_MAGIC.starts_with(&[0x3C, 0x3C, 0x3C, 0x20]));
    }

    #[test]
    fn test_is_7z_segment() {
        assert!(is_7z_segment("archive.7z.001"));
        assert!(is_7z_segment("archive.7z.002"));
        assert!(is_7z_segment("archive.7z.100"));
        assert!(!is_7z_segment("archive.7z"));
        assert!(!is_7z_segment("archive.zip.001"));
        assert!(!is_7z_segment("archive.7z.abc"));
        assert!(!is_7z_segment("archive.7z."));
    }

    #[test]
    fn test_tar_magic() {
        // TAR has "ustar" at offset 257
        assert_eq!(TAR_MAGIC, b"ustar");
    }
}
