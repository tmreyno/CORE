// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Forensic Format Definitions
//!
//! This module provides centralized definitions for all supported forensic
//! container and evidence formats. It serves as the single source of truth
//! for format detection, display names, and metadata.
//!
//! # Supported Format Categories
//!
//! - **Forensic Containers**: E01/Ex01, AD1, L01/Lx01
//! - **Raw Images**: DD, RAW, IMG
//! - **Mobile Forensics**: UFED (UFD, UFDR, UFDX)
//! - **Archives**: ZIP, 7z, RAR, TAR
//! - **Virtual Disks**: VMDK, VHD, VHDX, QCOW2
//!
//! # Usage
//!
//! ```rust
//! use ffx_check_lib::formats::{FormatCategory, detect_format_by_extension};
//!
//! let format = detect_format_by_extension("evidence.E01");
//! if let Some(fmt) = format {
//!     println!("Format: {} ({})", fmt.display_name, fmt.category.name());
//!     assert_eq!(fmt.display_name, "Expert Witness Format");
//!     assert_eq!(fmt.category.name(), "Forensic Container");
//! }
//! ```

use serde::Serialize;
use strum::{AsRefStr, Display, EnumIter, EnumString};

// =============================================================================
// FORMAT CATEGORIES
// =============================================================================

/// High-level category of forensic format with automatic string conversions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Display, EnumIter, EnumString, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum FormatCategory {
    /// Forensic evidence containers (E01, AD1, L01)
    #[strum(serialize = "forensic-container", to_string = "Forensic Container")]
    ForensicContainer,
    /// Raw disk images (DD, RAW, IMG)
    #[strum(serialize = "raw-image", to_string = "Raw Disk Image")]
    RawImage,
    /// Mobile forensic extractions (UFED)
    #[strum(serialize = "mobile-forensic", to_string = "Mobile Forensic")]
    MobileForensic,
    /// Compressed archives (ZIP, 7z)
    #[strum(serialize = "archive", to_string = "Archive")]
    Archive,
    /// Virtual machine disk images (VMDK, VHD)
    #[strum(serialize = "virtual-disk", to_string = "Virtual Disk")]
    VirtualDisk,
    /// Optical disc images (ISO, DMG)
    #[strum(serialize = "optical-disc", to_string = "Optical Disc")]
    OpticalDisc,
    /// Unknown or unsupported format
    #[strum(serialize = "unknown", to_string = "Unknown")]
    Unknown,
}

impl FormatCategory {
    /// Human-readable name for the category (use Display trait instead)
    #[deprecated(note = "Use Display trait (.to_string()) instead")]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::ForensicContainer => "Forensic Container",
            Self::RawImage => "Raw Disk Image",
            Self::MobileForensic => "Mobile Forensic",
            Self::Archive => "Archive",
            Self::VirtualDisk => "Virtual Disk",
            Self::OpticalDisc => "Optical Disc",
            Self::Unknown => "Unknown",
        }
    }

    /// Icon emoji for the category
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::ForensicContainer => "📦",
            Self::RawImage => "💾",
            Self::MobileForensic => "📱",
            Self::Archive => "🗜️",
            Self::VirtualDisk => "💿",
            Self::OpticalDisc => "📀",
            Self::Unknown => "❓",
        }
    }
}

// =============================================================================
// CONTAINER FORMAT DEFINITION
// =============================================================================

/// Definition of a forensic container format
///
/// Note: This struct uses `&'static` references for compile-time constants.
/// It cannot be deserialized, only serialized for frontend consumption.
#[derive(Debug, Clone, Serialize)]
pub struct ContainerFormat {
    /// Internal identifier (e.g., "e01", "ad1")
    pub id: &'static str,
    /// Display name (e.g., "Expert Witness Format")
    pub display_name: &'static str,
    /// Short type name (e.g., "E01", "AD1")
    pub type_name: &'static str,
    /// File extensions (lowercase, without dot)
    pub extensions: &'static [&'static str],
    /// Format category
    pub category: FormatCategory,
    /// Whether format supports segmentation
    pub supports_segments: bool,
    /// Magic bytes signature (if applicable)
    pub magic_bytes: Option<&'static [u8]>,
    /// Description
    pub description: &'static str,
    /// Vendor/creator
    pub vendor: Option<&'static str>,
}

// =============================================================================
// FORMAT DEFINITIONS
// =============================================================================

/// Expert Witness Format (EnCase physical images)
pub const FORMAT_E01: ContainerFormat = ContainerFormat {
    id: "e01",
    display_name: "Expert Witness Format",
    type_name: "E01",
    extensions: &[
        "e01", "e02", "e03", "e04", "e05", "e06", "e07", "e08", "e09", "e10", "e11", "e12", "e13",
        "e14", "e15", "e16", "e17", "e18", "e19", "e20", "e21", "e22", "e23", "e24", "e25", "e26",
        "e27", "e28", "e29", "e30", "e31", "e32", "e33", "e34", "e35", "e36", "e37", "e38", "e39",
        "e40", "e41", "e42", "e43", "e44", "e45", "e46", "e47", "e48", "e49", "e50", "e51", "e52",
        "e53", "e54", "e55", "e56", "e57", "e58", "e59", "e60", "e61", "e62", "e63", "e64", "e65",
        "e66", "e67", "e68", "e69", "e70", "e71", "e72", "e73", "e74", "e75", "e76", "e77", "e78",
        "e79", "e80", "e81", "e82", "e83", "e84", "e85", "e86", "e87", "e88", "e89", "e90", "e91",
        "e92", "e93", "e94", "e95", "e96", "e97", "e98", "e99", "ewf",
    ],
    category: FormatCategory::ForensicContainer,
    supports_segments: true,
    magic_bytes: Some(b"EVF\x09\r\n\xff\x00"),
    description: "EnCase Expert Witness disk image format",
    vendor: Some("OpenText (formerly Guidance Software)"),
};

/// Expert Witness Format v2 (EnCase 7+ physical images)
pub const FORMAT_EX01: ContainerFormat = ContainerFormat {
    id: "ex01",
    display_name: "Expert Witness Format v2",
    type_name: "Ex01",
    extensions: &["ex01"],
    category: FormatCategory::ForensicContainer,
    supports_segments: true,
    magic_bytes: Some(b"EVF2\r\n\x81"),
    description: "EnCase Expert Witness v2 format with improved compression",
    vendor: Some("OpenText (formerly Guidance Software)"),
};

/// EnCase Logical Evidence (L01/Lx01)
pub const FORMAT_L01: ContainerFormat = ContainerFormat {
    id: "l01",
    display_name: "EnCase Logical Evidence",
    type_name: "L01",
    extensions: &[
        "l01", "l02", "l03", "l04", "l05", "l06", "l07", "l08", "l09",
    ],
    category: FormatCategory::ForensicContainer,
    supports_segments: true,
    magic_bytes: Some(b"LVF\x09\r\n\xff\x00"),
    description: "EnCase logical evidence container",
    vendor: Some("OpenText (formerly Guidance Software)"),
};

/// EnCase Logical Evidence v2
pub const FORMAT_LX01: ContainerFormat = ContainerFormat {
    id: "lx01",
    display_name: "EnCase Logical Evidence v2",
    type_name: "Lx01",
    extensions: &["lx01"],
    category: FormatCategory::ForensicContainer,
    supports_segments: true,
    magic_bytes: Some(b"LEF2\r\n\x81"),
    description: "EnCase logical evidence v2 format",
    vendor: Some("OpenText (formerly Guidance Software)"),
};

/// AccessData AD1 Logical
pub const FORMAT_AD1: ContainerFormat = ContainerFormat {
    id: "ad1",
    display_name: "AccessData Logical Image",
    type_name: "AD1",
    extensions: &[
        "ad1", "ad2", "ad3", "ad4", "ad5", "ad6", "ad7", "ad8", "ad9",
    ],
    category: FormatCategory::ForensicContainer,
    supports_segments: true,
    magic_bytes: Some(b"ADSEGMENTEDFILE"),
    description: "AccessData FTK Imager logical evidence format",
    vendor: Some("AccessData (now Exterro)"),
};

/// Raw Disk Image
pub const FORMAT_RAW: ContainerFormat = ContainerFormat {
    id: "raw",
    display_name: "Raw Disk Image",
    type_name: "Raw",
    extensions: &["dd", "raw", "img", "bin", "001", "002", "003"],
    category: FormatCategory::RawImage,
    supports_segments: true,
    magic_bytes: None, // Raw images have no magic bytes
    description: "Uncompressed bit-for-bit disk image",
    vendor: None,
};

/// Cellebrite UFED UFD
pub const FORMAT_UFED_UFD: ContainerFormat = ContainerFormat {
    id: "ufed_ufd",
    display_name: "Cellebrite UFED Extraction",
    type_name: "UFD",
    extensions: &["ufd"],
    category: FormatCategory::MobileForensic,
    supports_segments: false,
    magic_bytes: None, // UFD is XML-based
    description: "Cellebrite UFED extraction metadata file",
    vendor: Some("Cellebrite"),
};

/// Cellebrite UFED UFDR
pub const FORMAT_UFED_UFDR: ContainerFormat = ContainerFormat {
    id: "ufed_ufdr",
    display_name: "Cellebrite UFED Report",
    type_name: "UFDR",
    extensions: &["ufdr"],
    category: FormatCategory::MobileForensic,
    supports_segments: false,
    magic_bytes: None,
    description: "Cellebrite UFED report package",
    vendor: Some("Cellebrite"),
};

/// 7-Zip Archive
pub const FORMAT_7Z: ContainerFormat = ContainerFormat {
    id: "7z",
    display_name: "7-Zip Archive",
    type_name: "7z",
    extensions: &["7z"],
    category: FormatCategory::Archive,
    supports_segments: true,
    magic_bytes: Some(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]),
    description: "7-Zip compressed archive",
    vendor: Some("Igor Pavlov"),
};

/// ZIP Archive
pub const FORMAT_ZIP: ContainerFormat = ContainerFormat {
    id: "zip",
    display_name: "ZIP Archive",
    type_name: "ZIP",
    extensions: &["zip", "z01", "z02"],
    category: FormatCategory::Archive,
    supports_segments: true,
    magic_bytes: Some(&[0x50, 0x4B]),
    description: "ZIP compressed archive",
    vendor: None,
};

/// AFF (Advanced Forensic Format)
pub const FORMAT_AFF: ContainerFormat = ContainerFormat {
    id: "aff",
    display_name: "Advanced Forensic Format",
    type_name: "AFF",
    extensions: &["aff", "afd"],
    category: FormatCategory::ForensicContainer,
    supports_segments: false,
    magic_bytes: Some(b"AFF10\r\n"),
    description: "Open-source forensic disk image format",
    vendor: None,
};

/// AFF4
pub const FORMAT_AFF4: ContainerFormat = ContainerFormat {
    id: "aff4",
    display_name: "Advanced Forensic Format 4",
    type_name: "AFF4",
    extensions: &["aff4"],
    category: FormatCategory::ForensicContainer,
    supports_segments: false,
    magic_bytes: None, // AFF4 is ZIP-based
    description: "AFF4 forensic container (ZIP-based)",
    vendor: None,
};

/// VMDK (VMware Virtual Disk)
pub const FORMAT_VMDK: ContainerFormat = ContainerFormat {
    id: "vmdk",
    display_name: "VMware Virtual Disk",
    type_name: "VMDK",
    extensions: &["vmdk"],
    category: FormatCategory::VirtualDisk,
    supports_segments: true,
    magic_bytes: Some(b"KDMV"),
    description: "VMware virtual machine disk image",
    vendor: Some("VMware"),
};

/// VHD (Microsoft Virtual Hard Disk)
pub const FORMAT_VHD: ContainerFormat = ContainerFormat {
    id: "vhd",
    display_name: "Virtual Hard Disk",
    type_name: "VHD",
    extensions: &["vhd"],
    category: FormatCategory::VirtualDisk,
    supports_segments: false,
    magic_bytes: Some(b"conectix"),
    description: "Microsoft Virtual Hard Disk format",
    vendor: Some("Microsoft"),
};

/// VHDX (Microsoft Virtual Hard Disk v2)
pub const FORMAT_VHDX: ContainerFormat = ContainerFormat {
    id: "vhdx",
    display_name: "Virtual Hard Disk v2",
    type_name: "VHDX",
    extensions: &["vhdx"],
    category: FormatCategory::VirtualDisk,
    supports_segments: false,
    magic_bytes: Some(b"vhdxfile"),
    description: "Microsoft VHDX format (Hyper-V)",
    vendor: Some("Microsoft"),
};

/// QCOW2 (QEMU Copy-on-Write)
pub const FORMAT_QCOW2: ContainerFormat = ContainerFormat {
    id: "qcow2",
    display_name: "QEMU Copy-on-Write v2",
    type_name: "QCOW2",
    extensions: &["qcow2", "qcow"],
    category: FormatCategory::VirtualDisk,
    supports_segments: false,
    magic_bytes: Some(&[0x51, 0x46, 0x49, 0xFB]),
    description: "QEMU/KVM virtual disk format",
    vendor: Some("QEMU"),
};

/// ISO 9660
pub const FORMAT_ISO: ContainerFormat = ContainerFormat {
    id: "iso",
    display_name: "ISO 9660 Disc Image",
    type_name: "ISO",
    extensions: &["iso"],
    category: FormatCategory::OpticalDisc,
    supports_segments: false,
    magic_bytes: None, // ISO signature is at offset 0x8001
    description: "Optical disc image (CD/DVD/BD)",
    vendor: None,
};

/// macOS DMG
pub const FORMAT_DMG: ContainerFormat = ContainerFormat {
    id: "dmg",
    display_name: "Apple Disk Image",
    type_name: "DMG",
    extensions: &["dmg"],
    category: FormatCategory::OpticalDisc,
    supports_segments: false,
    magic_bytes: None, // DMG has trailer signature
    description: "macOS disk image format",
    vendor: Some("Apple"),
};

// =============================================================================
// FORMAT REGISTRY
// =============================================================================

/// All supported formats
pub const ALL_FORMATS: &[&ContainerFormat] = &[
    // Forensic containers
    &FORMAT_E01,
    &FORMAT_EX01,
    &FORMAT_L01,
    &FORMAT_LX01,
    &FORMAT_AD1,
    &FORMAT_AFF,
    &FORMAT_AFF4,
    // Raw images
    &FORMAT_RAW,
    // Mobile forensic
    &FORMAT_UFED_UFD,
    &FORMAT_UFED_UFDR,
    // Archives
    &FORMAT_7Z,
    &FORMAT_ZIP,
    // Virtual disks
    &FORMAT_VMDK,
    &FORMAT_VHD,
    &FORMAT_VHDX,
    &FORMAT_QCOW2,
    // Optical discs
    &FORMAT_ISO,
    &FORMAT_DMG,
];

// =============================================================================
// FORMAT DETECTION
// =============================================================================

/// Detect format by file extension (case-insensitive)
pub fn detect_format_by_extension(path: &str) -> Option<&'static ContainerFormat> {
    let lower = path.to_lowercase();

    // Extract extension
    let ext = if let Some(dot_pos) = lower.rfind('.') {
        &lower[dot_pos + 1..]
    } else {
        return None;
    };

    // Search all formats
    for format in ALL_FORMATS {
        if format.extensions.contains(&ext) {
            return Some(format);
        }
    }

    // Handle numbered segments (.001, .002, etc.)
    if ext.len() == 3 && ext.chars().all(|c| c.is_ascii_digit()) {
        return Some(&FORMAT_RAW);
    }

    // Handle E01 extended segments (.e10 - .e99)
    if ext.len() == 3 && ext.starts_with('e') && ext[1..].chars().all(|c| c.is_ascii_digit()) {
        return Some(&FORMAT_E01);
    }

    None
}

/// Get format by ID
pub fn get_format_by_id(id: &str) -> Option<&'static ContainerFormat> {
    ALL_FORMATS.iter().find(|f| f.id == id).copied()
}

/// Get all formats in a category
pub fn get_formats_by_category(category: FormatCategory) -> Vec<&'static ContainerFormat> {
    ALL_FORMATS
        .iter()
        .filter(|f| f.category == category)
        .copied()
        .collect()
}

/// Get display name for a file path based on detected format
pub fn get_format_display_name(path: &str) -> &'static str {
    detect_format_by_extension(path)
        .map(|f| f.display_name)
        .unwrap_or("Unknown Format")
}

/// Get format type name for a file path
pub fn get_format_type_name(path: &str) -> &'static str {
    detect_format_by_extension(path)
        .map(|f| f.type_name)
        .unwrap_or("Unknown")
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_e01() {
        assert_eq!(detect_format_by_extension("test.E01").unwrap().id, "e01");
        assert_eq!(detect_format_by_extension("test.e01").unwrap().id, "e01");
        assert_eq!(detect_format_by_extension("test.E02").unwrap().id, "e01");
        assert_eq!(detect_format_by_extension("test.E99").unwrap().id, "e01");
    }

    #[test]
    fn test_detect_ad1() {
        assert_eq!(detect_format_by_extension("test.ad1").unwrap().id, "ad1");
        assert_eq!(detect_format_by_extension("test.AD1").unwrap().id, "ad1");
    }

    #[test]
    fn test_detect_raw() {
        assert_eq!(detect_format_by_extension("test.dd").unwrap().id, "raw");
        assert_eq!(detect_format_by_extension("test.001").unwrap().id, "raw");
        assert_eq!(detect_format_by_extension("test.raw").unwrap().id, "raw");
    }

    #[test]
    fn test_detect_ufed() {
        assert_eq!(
            detect_format_by_extension("test.ufd").unwrap().id,
            "ufed_ufd"
        );
        assert_eq!(
            detect_format_by_extension("test.ufdr").unwrap().id,
            "ufed_ufdr"
        );
    }

    #[test]
    fn test_format_category() {
        assert_eq!(FORMAT_E01.category.to_string(), "Forensic Container");
        assert_eq!(FORMAT_RAW.category.to_string(), "Raw Disk Image");
        assert_eq!(FORMAT_UFED_UFD.category.to_string(), "Mobile Forensic");
    }

    #[test]
    fn test_get_formats_by_category() {
        let forensic = get_formats_by_category(FormatCategory::ForensicContainer);
        assert!(forensic.iter().any(|f| f.id == "e01"));
        assert!(forensic.iter().any(|f| f.id == "ad1"));
        assert!(forensic.iter().any(|f| f.id == "l01"));
    }
}
