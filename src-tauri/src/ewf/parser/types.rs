// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Parser-specific types and constants for EWF format analysis.

use serde::{Deserialize, Serialize};

// ============================================================================
// Constants
// ============================================================================

/// Section header size
pub(crate) const SECTION_HEADER_SIZE: usize = 76;

/// Known section types
pub(crate) const SECTION_TYPES: &[&str] = &[
    "header", "header2", "volume", "disk", "sectors", "table", "table2", "hash", "digest",
    "error2", "session", "data", "next", "done", "ltree", "ltypes",
];

// ============================================================================
// Types
// ============================================================================

/// EWF format variant (covers all E01/L01/Ex01/Lx01 variants)
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum EwfVariant {
    /// E01 - Physical disk image (EWF v1)
    E01,
    /// L01 - Logical evidence file (EWF v1)
    L01,
    /// Ex01 - Physical disk image (EWF v2)
    Ex01,
    /// Lx01 - Logical evidence file (EWF v2)
    Lx01,
    /// Unknown variant
    #[default]
    Unknown,
}

impl EwfVariant {
    /// Returns true if this is a logical evidence format (L01/Lx01)
    pub fn is_logical(&self) -> bool {
        matches!(self, EwfVariant::L01 | EwfVariant::Lx01)
    }

    /// Returns true if this is a physical disk image (E01/Ex01)
    pub fn is_physical(&self) -> bool {
        matches!(self, EwfVariant::E01 | EwfVariant::Ex01)
    }

    /// Returns true if this is EWF v2 format
    pub fn is_v2(&self) -> bool {
        matches!(self, EwfVariant::Ex01 | EwfVariant::Lx01)
    }
}

impl std::fmt::Display for EwfVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EwfVariant::E01 => write!(f, "E01"),
            EwfVariant::L01 => write!(f, "L01"),
            EwfVariant::Ex01 => write!(f, "Ex01"),
            EwfVariant::Lx01 => write!(f, "Lx01"),
            EwfVariant::Unknown => write!(f, "Unknown"),
        }
    }
}

/// EWF section header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EwfSectionHeader {
    /// Section type (e.g., "header", "volume", "sectors")
    pub section_type: String,
    /// Offset of next section (0 if none)
    pub next_offset: u64,
    /// Size of this section (including 76-byte header)
    pub section_size: u64,
    /// Checksum (Adler-32)
    pub checksum: u32,
    /// Offset where this section starts in the file
    pub file_offset: u64,
}

/// Volume section data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EwfVolumeInfo {
    pub chunk_count: u32,
    pub sectors_per_chunk: u32,
    pub bytes_per_sector: u32,
    pub sector_count: u64,
    pub chs_cylinders: u32,
    pub chs_heads: u32,
    pub chs_sectors: u32,
    pub media_type: u32,
    pub compression_level: u8,
    pub guid: Option<String>,
}

impl EwfVolumeInfo {
    /// Create a new EwfVolumeInfo
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set chunk count
    #[inline]
    pub fn with_chunk_count(mut self, count: u32) -> Self {
        self.chunk_count = count;
        self
    }

    /// Set sectors per chunk
    #[inline]
    pub fn with_sectors_per_chunk(mut self, sectors: u32) -> Self {
        self.sectors_per_chunk = sectors;
        self
    }

    /// Set bytes per sector
    #[inline]
    pub fn with_bytes_per_sector(mut self, bytes: u32) -> Self {
        self.bytes_per_sector = bytes;
        self
    }

    /// Set sector count
    #[inline]
    pub fn with_sector_count(mut self, count: u64) -> Self {
        self.sector_count = count;
        self
    }

    /// Set CHS geometry
    #[inline]
    pub fn with_chs(mut self, cylinders: u32, heads: u32, sectors: u32) -> Self {
        self.chs_cylinders = cylinders;
        self.chs_heads = heads;
        self.chs_sectors = sectors;
        self
    }

    /// Set media type
    #[inline]
    pub fn with_media_type(mut self, media_type: u32) -> Self {
        self.media_type = media_type;
        self
    }

    /// Set compression level
    #[inline]
    pub fn with_compression_level(mut self, level: u8) -> Self {
        self.compression_level = level;
        self
    }

    /// Set GUID
    #[inline]
    pub fn with_guid(mut self, guid: impl Into<String>) -> Self {
        self.guid = Some(guid.into());
        self
    }
}

/// Parsed case information from header/header2 sections
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EwfCaseInfo {
    pub description: Option<String>,
    pub case_number: Option<String>,
    pub evidence_number: Option<String>,
    pub examiner: Option<String>,
    pub notes: Option<String>,
    pub acquisition_date: Option<String>,
    pub system_date: Option<String>,
    pub acquisition_software: Option<String>,
    pub acquisition_os: Option<String>,
    pub device_model: Option<String>,
    pub device_serial: Option<String>,
    pub device_label: Option<String>,
    pub device_total_bytes: Option<u64>,
}

impl EwfCaseInfo {
    /// Create a new EwfCaseInfo
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set description
    #[inline]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set case number
    #[inline]
    pub fn with_case_number(mut self, num: impl Into<String>) -> Self {
        self.case_number = Some(num.into());
        self
    }

    /// Set evidence number
    #[inline]
    pub fn with_evidence_number(mut self, num: impl Into<String>) -> Self {
        self.evidence_number = Some(num.into());
        self
    }

    /// Set examiner
    #[inline]
    pub fn with_examiner(mut self, examiner: impl Into<String>) -> Self {
        self.examiner = Some(examiner.into());
        self
    }

    /// Set notes
    #[inline]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Set acquisition date
    #[inline]
    pub fn with_acquisition_date(mut self, date: impl Into<String>) -> Self {
        self.acquisition_date = Some(date.into());
        self
    }

    /// Set acquisition software
    #[inline]
    pub fn with_acquisition_software(mut self, software: impl Into<String>) -> Self {
        self.acquisition_software = Some(software.into());
        self
    }

    /// Set device info
    #[inline]
    pub fn with_device(
        mut self,
        model: Option<String>,
        serial: Option<String>,
        label: Option<String>,
    ) -> Self {
        self.device_model = model;
        self.device_serial = serial;
        self.device_label = label;
        self
    }

    /// Set device total bytes
    #[inline]
    pub fn with_device_size(mut self, bytes: u64) -> Self {
        self.device_total_bytes = Some(bytes);
        self
    }
}

/// Hash information from hash/digest sections
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EwfHashInfo {
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
}

impl EwfHashInfo {
    /// Create a new EwfHashInfo
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set MD5 hash
    #[inline]
    pub fn with_md5(mut self, hash: impl Into<String>) -> Self {
        self.md5 = Some(hash.into());
        self
    }

    /// Set SHA-1 hash
    #[inline]
    pub fn with_sha1(mut self, hash: impl Into<String>) -> Self {
        self.sha1 = Some(hash.into());
        self
    }

    /// Set SHA-256 hash
    #[inline]
    pub fn with_sha256(mut self, hash: impl Into<String>) -> Self {
        self.sha256 = Some(hash.into());
        self
    }
}

/// Error/bad sector information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EwfErrorEntry {
    pub first_sector: u32,
    pub sector_count: u32,
}

impl EwfErrorEntry {
    /// Create a new EwfErrorEntry
    #[inline]
    pub fn new(first_sector: u32, sector_count: u32) -> Self {
        Self {
            first_sector,
            sector_count,
        }
    }
}

/// Complete parsed EWF information for hex viewer
/// This is the detailed parsing result, distinct from EwfInfo (public API container info)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EwfDetailedInfo {
    /// Format variant
    pub variant: EwfVariant,
    /// EWF version (1 or 2)
    pub version: u8,
    /// Segment number
    pub segment_number: u16,
    /// List of sections found
    pub sections: Vec<EwfSectionHeader>,
    /// Volume information (if found)
    pub volume: Option<EwfVolumeInfo>,
    /// Case information (from header/header2)
    pub case_info: EwfCaseInfo,
    /// Hash information (from hash/digest)
    pub hashes: EwfHashInfo,
    /// Error sectors (from error2)
    pub errors: Vec<EwfErrorEntry>,
    /// Total file size
    pub file_size: u64,
}

impl EwfDetailedInfo {
    /// Create a new EwfDetailedInfo
    #[inline]
    pub fn new(variant: EwfVariant, version: u8) -> Self {
        Self {
            variant,
            version,
            ..Default::default()
        }
    }

    /// Set segment number
    #[inline]
    pub fn with_segment_number(mut self, num: u16) -> Self {
        self.segment_number = num;
        self
    }

    /// Set sections
    #[inline]
    pub fn with_sections(mut self, sections: Vec<EwfSectionHeader>) -> Self {
        self.sections = sections;
        self
    }

    /// Set volume info
    #[inline]
    pub fn with_volume(mut self, volume: EwfVolumeInfo) -> Self {
        self.volume = Some(volume);
        self
    }

    /// Set case info
    #[inline]
    pub fn with_case_info(mut self, info: EwfCaseInfo) -> Self {
        self.case_info = info;
        self
    }

    /// Set hash info
    #[inline]
    pub fn with_hashes(mut self, hashes: EwfHashInfo) -> Self {
        self.hashes = hashes;
        self
    }

    /// Add error entry
    #[inline]
    pub fn add_error(mut self, error: EwfErrorEntry) -> Self {
        self.errors.push(error);
        self
    }

    /// Set file size
    #[inline]
    pub fn with_file_size(mut self, size: u64) -> Self {
        self.file_size = size;
        self
    }
}
