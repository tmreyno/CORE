// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Container Detection Utilities
//!
//! Provides unified container type detection for forensic containers, archives,
//! and disk images. This is the canonical module for determining container types
//! by extension - use this instead of ad-hoc detection logic.
//!
//! # Container Categories
//!
//! | Category | Types |
//! |----------|-------|
//! | Forensic | E01, L01, AD1, UFED |
//! | Archive | ZIP, 7z, RAR, TAR |
//! | Disk Image | DMG, ISO, RAW, DD |
//!
//! # Examples
//!
//! ```rust,ignore
//! use crate::common::container_detect::{detect_container_type, ForensicContainerType, is_forensic_container};
//!
//! // Check if file is a forensic container
//! if let Some(container_type) = detect_container_type("evidence.E01") {
//!     if container_type.is_forensic() {
//!         println!("Forensic container: {:?}", container_type);
//!     }
//! }
//!
//! // Quick check for forensic containers
//! if is_forensic_container("image.ad1") {
//!     // Use segment-aware hashing
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::path::Path;

// =============================================================================
// Container Type Enum
// =============================================================================

/// All recognized container types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerType {
    // Forensic containers (require special handling)
    E01,  // EnCase Evidence File
    L01,  // EnCase Logical Evidence
    AD1,  // FTK/AccessData Logical Image
    UFED, // Cellebrite UFED (UFD, UFDR, UFDX)

    // Archive formats
    Zip,
    SevenZ,
    Rar,
    Tar,

    // Disk images
    Raw, // DD, IMG, RAW
    Dmg, // macOS disk image
    Iso, // ISO 9660
}

impl ContainerType {
    /// Returns true if this is a forensic container that requires special handling
    pub fn is_forensic(&self) -> bool {
        matches!(self, Self::E01 | Self::L01 | Self::AD1 | Self::UFED)
    }

    /// Returns true if this is an archive format
    pub fn is_archive(&self) -> bool {
        matches!(self, Self::Zip | Self::SevenZ | Self::Rar | Self::Tar)
    }

    /// Returns true if this is a disk image format
    pub fn is_disk_image(&self) -> bool {
        matches!(
            self,
            Self::Raw | Self::Dmg | Self::Iso | Self::E01 | Self::L01
        )
    }

    /// Returns true if this format supports segmentation (.E01/.E02, .ad1/.ad2)
    pub fn is_segmented(&self) -> bool {
        matches!(self, Self::E01 | Self::L01 | Self::AD1)
    }

    /// Returns the canonical lowercase string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::E01 => "e01",
            Self::L01 => "l01",
            Self::AD1 => "ad1",
            Self::UFED => "ufed",
            Self::Zip => "zip",
            Self::SevenZ => "7z",
            Self::Rar => "rar",
            Self::Tar => "tar",
            Self::Raw => "raw",
            Self::Dmg => "dmg",
            Self::Iso => "iso",
        }
    }

    /// Returns a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::E01 => "EnCase Evidence File",
            Self::L01 => "EnCase Logical Evidence",
            Self::AD1 => "FTK/AccessData Logical Image",
            Self::UFED => "Cellebrite UFED",
            Self::Zip => "ZIP Archive",
            Self::SevenZ => "7-Zip Archive",
            Self::Rar => "RAR Archive",
            Self::Tar => "TAR Archive",
            Self::Raw => "Raw Disk Image",
            Self::Dmg => "macOS Disk Image",
            Self::Iso => "ISO 9660",
        }
    }

    /// Returns the emoji icon for UI display
    pub fn icon(&self) -> &'static str {
        match self {
            Self::E01 | Self::L01 => "💿",
            Self::AD1 => "📦",
            Self::UFED => "📱",
            Self::Zip | Self::SevenZ | Self::Rar | Self::Tar => "🗜️",
            Self::Raw => "💾",
            Self::Dmg | Self::Iso => "📀",
        }
    }
}

impl std::fmt::Display for ContainerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// =============================================================================
// Detection Functions
// =============================================================================

/// Detect container type from a filename or path
///
/// This is the canonical function for container type detection.
/// Works with full paths or just filenames.
///
/// # Arguments
///
/// * `path` - File path or filename to check
///
/// # Returns
///
/// `Some(ContainerType)` if recognized, `None` otherwise
///
/// # Example
///
/// ```rust,ignore
/// assert_eq!(detect_container_type("evidence.E01"), Some(ContainerType::E01));
/// assert_eq!(detect_container_type("/path/to/image.ad1"), Some(ContainerType::AD1));
/// assert_eq!(detect_container_type("document.pdf"), None);
/// ```
pub fn detect_container_type(path: &str) -> Option<ContainerType> {
    detect_container_type_from_path(Path::new(path))
}

/// Detect container type from a Path
pub fn detect_container_type_from_path(path: &Path) -> Option<ContainerType> {
    let lower = path.to_string_lossy().to_lowercase();

    // Forensic containers - check first as they're most important
    if lower.ends_with(".e01") || lower.ends_with(".ex01") {
        return Some(ContainerType::E01);
    }
    if lower.ends_with(".l01") || lower.ends_with(".lx01") {
        return Some(ContainerType::L01);
    }
    if lower.ends_with(".ad1") {
        return Some(ContainerType::AD1);
    }
    if lower.ends_with(".ufd") || lower.ends_with(".ufdr") || lower.ends_with(".ufdx") {
        return Some(ContainerType::UFED);
    }

    // Archives
    if lower.ends_with(".zip") {
        return Some(ContainerType::Zip);
    }
    if lower.ends_with(".7z") {
        return Some(ContainerType::SevenZ);
    }
    if lower.ends_with(".rar") {
        return Some(ContainerType::Rar);
    }
    if lower.ends_with(".tar")
        || lower.ends_with(".tar.gz")
        || lower.ends_with(".tgz")
        || lower.ends_with(".tar.bz2")
        || lower.ends_with(".tar.xz")
        || lower.ends_with(".tar.zst")
    {
        return Some(ContainerType::Tar);
    }

    // Disk images
    if lower.ends_with(".dmg") {
        return Some(ContainerType::Dmg);
    }
    if lower.ends_with(".iso") {
        return Some(ContainerType::Iso);
    }
    if lower.ends_with(".raw") || lower.ends_with(".dd") || lower.ends_with(".img") {
        return Some(ContainerType::Raw);
    }

    None
}

/// Check if a file is a forensic container (E01, L01, AD1, UFED)
///
/// This is a convenience function for the common case of checking
/// if special forensic handling is needed.
///
/// # Example
///
/// ```rust,ignore
/// if is_forensic_container(path) {
///     // Use segment-aware hashing
/// }
/// ```
pub fn is_forensic_container(path: &str) -> bool {
    detect_container_type(path)
        .map(|t| t.is_forensic())
        .unwrap_or(false)
}

/// Check if a file is a forensic container from Path
pub fn is_forensic_container_path(path: &Path) -> bool {
    detect_container_type_from_path(path)
        .map(|t| t.is_forensic())
        .unwrap_or(false)
}

/// Check if a file is any recognized container type
pub fn is_container(path: &str) -> bool {
    detect_container_type(path).is_some()
}

/// Check if a file is a segmented container (E01, L01, AD1)
pub fn is_segmented_container(path: &str) -> bool {
    detect_container_type(path)
        .map(|t| t.is_segmented())
        .unwrap_or(false)
}

/// Get all supported forensic container extensions
pub fn forensic_extensions() -> &'static [&'static str] {
    &[
        ".e01", ".ex01", ".l01", ".lx01", ".ad1", ".ufd", ".ufdr", ".ufdx",
    ]
}

/// Get all supported archive extensions
pub fn archive_extensions() -> &'static [&'static str] {
    &[
        ".zip", ".7z", ".rar", ".tar", ".tar.gz", ".tgz", ".tar.bz2", ".tar.xz", ".tar.zst",
    ]
}

/// Get all supported disk image extensions
pub fn disk_image_extensions() -> &'static [&'static str] {
    &[
        ".e01", ".ex01", ".l01", ".lx01", ".raw", ".dd", ".img", ".dmg", ".iso",
    ]
}

/// Get all supported container extensions
pub fn all_extensions() -> &'static [&'static str] {
    &[
        ".e01", ".ex01", ".l01", ".lx01", ".ad1", ".ufd", ".ufdr", ".ufdx", ".zip", ".7z", ".rar",
        ".tar", ".tar.gz", ".tgz", ".tar.bz2", ".tar.xz", ".tar.zst", ".raw", ".dd", ".img",
        ".dmg", ".iso",
    ]
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_forensic_containers() {
        assert_eq!(
            detect_container_type("evidence.E01"),
            Some(ContainerType::E01)
        );
        assert_eq!(
            detect_container_type("evidence.e01"),
            Some(ContainerType::E01)
        );
        assert_eq!(
            detect_container_type("logical.L01"),
            Some(ContainerType::L01)
        );
        assert_eq!(detect_container_type("image.ad1"), Some(ContainerType::AD1));
        assert_eq!(
            detect_container_type("phone.ufd"),
            Some(ContainerType::UFED)
        );
        assert_eq!(
            detect_container_type("phone.ufdr"),
            Some(ContainerType::UFED)
        );
    }

    #[test]
    fn test_detect_archives() {
        assert_eq!(detect_container_type("files.zip"), Some(ContainerType::Zip));
        assert_eq!(
            detect_container_type("files.7z"),
            Some(ContainerType::SevenZ)
        );
        assert_eq!(detect_container_type("files.rar"), Some(ContainerType::Rar));
        assert_eq!(
            detect_container_type("files.tar.gz"),
            Some(ContainerType::Tar)
        );
    }

    #[test]
    fn test_detect_disk_images() {
        assert_eq!(detect_container_type("disk.raw"), Some(ContainerType::Raw));
        assert_eq!(detect_container_type("disk.dd"), Some(ContainerType::Raw));
        assert_eq!(detect_container_type("disk.dmg"), Some(ContainerType::Dmg));
        assert_eq!(detect_container_type("disk.iso"), Some(ContainerType::Iso));
    }

    #[test]
    fn test_is_forensic() {
        assert!(is_forensic_container("evidence.E01"));
        assert!(is_forensic_container("image.ad1"));
        assert!(!is_forensic_container("files.zip"));
        assert!(!is_forensic_container("document.pdf"));
    }

    #[test]
    fn test_container_type_methods() {
        assert!(ContainerType::E01.is_forensic());
        assert!(ContainerType::E01.is_disk_image());
        assert!(ContainerType::E01.is_segmented());

        assert!(!ContainerType::Zip.is_forensic());
        assert!(ContainerType::Zip.is_archive());

        assert_eq!(ContainerType::E01.as_str(), "e01");
        assert_eq!(ContainerType::AD1.icon(), "📦");
    }

    #[test]
    fn test_path_with_directory() {
        assert_eq!(
            detect_container_type("/path/to/evidence.E01"),
            Some(ContainerType::E01)
        );
        assert_eq!(
            detect_container_type("C:\\Users\\analyst\\evidence.ad1"),
            Some(ContainerType::AD1)
        );
    }
}
