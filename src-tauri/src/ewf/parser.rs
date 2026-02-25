// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! EWF (Expert Witness Format) Hex Parser Module
//! 
//! Parses E01/L01/Ex01/Lx01 forensic image headers and extracts
//! detailed metadata for display in the hex viewer.
//!
//! This module handles all EWF variants:
//! - E01: Physical disk image (EWF v1)
//! - L01: Logical evidence file (EWF v1) 
//! - Ex01: Physical disk image (EWF v2)
//! - Lx01: Logical evidence file (EWF v2)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use crate::common::format_size;
use crate::containers::ContainerError;
use crate::viewer::{HeaderRegion, MetadataField, ParsedMetadata};

// Import all signatures from types module (single source of truth)
use super::types::{EWF_SIGNATURE, EWF2_SIGNATURE, LVF_SIGNATURE, LVF2_SIGNATURE};

// ============================================================================
// Constants
// ============================================================================

/// Section header size
const SECTION_HEADER_SIZE: usize = 76;

/// Known section types
const SECTION_TYPES: &[&str] = &[
    "header", "header2", "volume", "disk", "sectors", 
    "table", "table2", "hash", "digest", "error2", 
    "session", "data", "next", "done",
    "ltree", "ltypes",
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
    pub fn with_device(mut self, model: Option<String>, serial: Option<String>, label: Option<String>) -> Self {
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
        Self { first_sector, sector_count }
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

// ============================================================================
// Parser Implementation
// ============================================================================

/// Parse an EWF file (E01/L01/Ex01/Lx01) and extract all metadata
pub fn parse_ewf_file(path: &str) -> Result<EwfDetailedInfo, ContainerError> {
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();
    
    // Read signature and segment info (first 13 bytes minimum)
    let mut header = [0u8; 32];
    file.read_exact(&mut header)?;
    
    // Detect variant
    let (variant, version) = detect_ewf_variant(&header)?;
    
    // Read segment number
    let segment_number = if version == 1 {
        // Segment number at offset 9 (2 bytes LE)
        u16::from_le_bytes([header[9], header[10]])
    } else {
        // EWF v2 has different layout
        u16::from_le_bytes([header[9], header[10]])
    };
    
    // Parse all sections
    let sections = parse_all_sections(&mut file, 13, file_size)?;
    
    // Extract volume info
    let volume = parse_volume_section(&mut file, &sections)?;
    
    // Extract case info from header/header2
    let case_info = parse_case_info(&mut file, &sections)?;
    
    // Extract hash info from hash/digest
    let hashes = parse_hash_info(&mut file, &sections)?;
    
    // Extract error info from error2
    let errors = parse_error_info(&mut file, &sections)?;
    
    Ok(EwfDetailedInfo {
        variant,
        version,
        segment_number,
        sections,
        volume,
        case_info,
        hashes,
        errors,
        file_size,
    })
}

/// Detect EWF format variant from signature
/// Supports E01, L01, Ex01, Lx01
pub fn detect_ewf_variant(header: &[u8]) -> Result<(EwfVariant, u8), ContainerError> {
    if header.len() < 8 {
        return Err(ContainerError::ParseError("Header too short".to_string()));
    }
    
    // Check for E01 (physical, v1)
    if &header[0..8] == EWF_SIGNATURE {
        return Ok((EwfVariant::E01, 1));
    }
    // Check for L01 (logical, v1)
    if &header[0..8] == LVF_SIGNATURE {
        return Ok((EwfVariant::L01, 1));
    }
    // Check for Ex01 (physical, v2)
    if &header[0..8] == EWF2_SIGNATURE {
        return Ok((EwfVariant::Ex01, 2));
    }
    // Check for Lx01 (logical, v2)
    if &header[0..8] == LVF2_SIGNATURE {
        return Ok((EwfVariant::Lx01, 2));
    }
    
    // Partial match for EVF/LVF (legacy compatibility)
    if &header[0..3] == b"EVF" {
        return Ok((EwfVariant::E01, 1));
    }
    if &header[0..3] == b"LVF" {
        return Ok((EwfVariant::L01, 1));
    }
    
    Err(ContainerError::InvalidFormat("Not a valid EWF file".to_string()))
}

/// Check if a file is any EWF variant (E01/L01/Ex01/Lx01)
pub fn is_ewf_file(path: &str) -> Result<bool, ContainerError> {
    let mut file = File::open(path)?;
    let mut sig = [0u8; 8];
    if file.read_exact(&mut sig).is_err() {
        return Ok(false);
    }
    
    Ok(&sig == EWF_SIGNATURE || 
       &sig == LVF_SIGNATURE || 
       &sig == EWF2_SIGNATURE || 
       &sig == LVF2_SIGNATURE)
}

/// Check if a file is L01 format specifically
pub fn is_l01_file(path: &str) -> Result<bool, ContainerError> {
    let mut file = File::open(path)?;
    let mut sig = [0u8; 8];
    if file.read_exact(&mut sig).is_err() {
        return Ok(false);
    }
    
    Ok(&sig == LVF_SIGNATURE || &sig == LVF2_SIGNATURE)
}

/// Parse all section headers in the file
fn parse_all_sections(file: &mut File, start_offset: u64, file_size: u64) -> Result<Vec<EwfSectionHeader>, ContainerError> {
    let mut sections = Vec::new();
    let mut offset = start_offset;
    
    // Safety limit - max 1000 sections
    let max_sections = 1000;
    
    while offset < file_size && sections.len() < max_sections {
        // Ensure enough space for section header
        if offset + SECTION_HEADER_SIZE as u64 > file_size {
            break;
        }
        
        file.seek(SeekFrom::Start(offset))?;
        
        let mut header = [0u8; SECTION_HEADER_SIZE];
        if file.read_exact(&mut header).is_err() {
            break;
        }
        
        // Parse section type (first 16 bytes, null-terminated)
        let type_bytes = &header[0..16];
        let section_type = type_bytes
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect::<String>();
        
        // Validate section type
        if section_type.is_empty() || !is_valid_section_type(&section_type) {
            // Not a valid section, might be data or end of sections
            break;
        }
        
        // Parse offsets and size
        let next_offset = u64::from_le_bytes([
            header[16], header[17], header[18], header[19],
            header[20], header[21], header[22], header[23],
        ]);
        
        let section_size = u64::from_le_bytes([
            header[24], header[25], header[26], header[27],
            header[28], header[29], header[30], header[31],
        ]);
        
        // Checksum at offset 72
        let checksum = u32::from_le_bytes([
            header[72], header[73], header[74], header[75],
        ]);
        
        let section = EwfSectionHeader {
            section_type: section_type.clone(),
            next_offset,
            section_size,
            checksum,
            file_offset: offset,
        };
        
        sections.push(section);
        
        // Check for terminal sections
        if section_type == "done" {
            break;
        }
        
        // Move to next section
        if next_offset > 0 && next_offset > offset {
            offset = next_offset;
        } else if section_size > 0 {
            offset += section_size;
        } else {
            break;
        }
    }
    
    Ok(sections)
}

/// Check if a section type is valid
fn is_valid_section_type(section_type: &str) -> bool {
    SECTION_TYPES.contains(&section_type)
}

/// Parse volume section data
fn parse_volume_section(file: &mut File, sections: &[EwfSectionHeader]) -> Result<Option<EwfVolumeInfo>, ContainerError> {
    let volume_section = sections.iter().find(|s| s.section_type == "volume");
    
    if let Some(section) = volume_section {
        let data_offset = section.file_offset + SECTION_HEADER_SIZE as u64;
        
        file.seek(SeekFrom::Start(data_offset))?;
        
        let mut data = [0u8; 80];
        if file.read_exact(&mut data).is_err() {
            return Ok(None);
        }
        
        // Parse volume fields
        let chunk_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let sectors_per_chunk = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let bytes_per_sector = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let sector_count = u64::from_le_bytes([
            data[16], data[17], data[18], data[19],
            data[20], data[21], data[22], data[23],
        ]);
        
        let chs_cylinders = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        let chs_heads = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);
        let chs_sectors = u32::from_le_bytes([data[32], data[33], data[34], data[35]]);
        let media_type = u32::from_le_bytes([data[36], data[37], data[38], data[39]]);
        let compression_level = data[56];
        
        // GUID at offset 60 (16 bytes)
        let guid = if data.len() > 76 {
            let guid_bytes = &data[60..76];
            if guid_bytes.iter().any(|&b| b != 0) {
                Some(format_guid(guid_bytes))
            } else {
                None
            }
        } else {
            None
        };
        
        return Ok(Some(EwfVolumeInfo {
            chunk_count,
            sectors_per_chunk,
            bytes_per_sector,
            sector_count,
            chs_cylinders,
            chs_heads,
            chs_sectors,
            media_type,
            compression_level,
            guid,
        }));
    }
    
    Ok(None)
}

/// Parse case information from header/header2 sections
fn parse_case_info(file: &mut File, sections: &[EwfSectionHeader]) -> Result<EwfCaseInfo, ContainerError> {
    let mut case_info = EwfCaseInfo::default();
    
    // Prefer header2 (UTF-16) over header (ASCII)
    let header_section = sections
        .iter()
        .find(|s| s.section_type == "header2")
        .or_else(|| sections.iter().find(|s| s.section_type == "header"));
    
    if let Some(section) = header_section {
        let data_offset = section.file_offset + SECTION_HEADER_SIZE as u64;
        let data_size = section.section_size.saturating_sub(SECTION_HEADER_SIZE as u64) as usize;
        
        // Limit to reasonable size
        let read_size = data_size.min(65536);
        
        file.seek(SeekFrom::Start(data_offset))?;
        
        let mut data = vec![0u8; read_size];
        if file.read_exact(&mut data).is_err() {
            return Ok(case_info);
        }
        
        // Check if data is zlib compressed (starts with 0x78)
        // Zlib header: first byte is 0x78 (compression method + flags),
        // second byte varies based on compression level (0x01, 0x5E, 0x9C, 0xDA)
        if data.len() >= 2 && data[0] == 0x78 {
            // Try to decompress
            if let Ok(decompressed) = decompress_zlib(&data) {
                case_info = parse_header_content(&decompressed, section.section_type == "header2");
            }
        } else {
            // Try parsing as-is
            case_info = parse_header_content(&data, section.section_type == "header2");
        }
    }
    
    Ok(case_info)
}

/// Parse decompressed header content
fn parse_header_content(data: &[u8], is_utf16: bool) -> EwfCaseInfo {
    let mut case_info = EwfCaseInfo::default();
    
    // Convert to string
    let content = if is_utf16 {
        // UTF-16 LE decoding
        let utf16_data: Vec<u16> = data
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some(u16::from_le_bytes([chunk[0], chunk[1]]))
                } else {
                    None
                }
            })
            .collect();
        String::from_utf16_lossy(&utf16_data)
    } else {
        String::from_utf8_lossy(data).to_string()
    };
    
    // Parse key=value pairs
    let fields = parse_ewf_header_fields(&content);
    
    // Map known fields
    case_info.description = fields.get("a").cloned();
    case_info.case_number = fields.get("c").cloned();
    case_info.evidence_number = fields.get("n").cloned();
    case_info.examiner = fields.get("e").cloned();
    case_info.notes = fields.get("t").cloned();
    case_info.acquisition_date = fields.get("m").cloned();
    case_info.system_date = fields.get("u").cloned();
    case_info.acquisition_software = fields.get("av").cloned();
    case_info.acquisition_os = fields.get("ov").cloned();
    case_info.device_model = fields.get("md").cloned();
    case_info.device_serial = fields.get("sn").cloned();
    case_info.device_label = fields.get("l").cloned();
    
    // Parse total bytes if present
    if let Some(tb) = fields.get("tb") {
        case_info.device_total_bytes = tb.parse().ok();
    }
    
    case_info
}

/// Parse EWF header key=value fields
fn parse_ewf_header_fields(content: &str) -> HashMap<String, String> {
    let mut fields = HashMap::new();
    
    // Split by tabs and newlines
    for line in content.split(['\n', '\r']) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Look for key=value or key\tvalue patterns
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();
            if !key.is_empty() && !value.is_empty() {
                fields.insert(key.to_string(), value.to_string());
            }
        } else if let Some(tab_pos) = line.find('\t') {
            let key = line[..tab_pos].trim();
            let value = line[tab_pos + 1..].trim();
            if !key.is_empty() && !value.is_empty() {
                fields.insert(key.to_string(), value.to_string());
            }
        }
    }
    
    fields
}

/// Parse hash information from hash/digest sections
fn parse_hash_info(file: &mut File, sections: &[EwfSectionHeader]) -> Result<EwfHashInfo, ContainerError> {
    let mut hash_info = EwfHashInfo::default();
    
    // Check digest section first (preferred, has more hashes)
    if let Some(section) = sections.iter().find(|s| s.section_type == "digest") {
        let data_offset = section.file_offset + SECTION_HEADER_SIZE as u64;
        
        file.seek(SeekFrom::Start(data_offset))?;
        
        let mut data = [0u8; 72]; // MD5(16) + SHA1(20) + SHA256(32) + checksum(4)
        let bytes_read = file.read(&mut data).unwrap_or(0);
        
        if bytes_read >= 16 {
            hash_info.md5 = Some(bytes_to_hex(&data[0..16]));
        }
        if bytes_read >= 36 {
            hash_info.sha1 = Some(bytes_to_hex(&data[16..36]));
        }
        if bytes_read >= 68 {
            // Check if SHA256 is present (not all zeros)
            if data[36..68].iter().any(|&b| b != 0) {
                hash_info.sha256 = Some(bytes_to_hex(&data[36..68]));
            }
        }
    }
    // Fallback to hash section (MD5 only)
    else if let Some(section) = sections.iter().find(|s| s.section_type == "hash") {
        let data_offset = section.file_offset + SECTION_HEADER_SIZE as u64;
        
        file.seek(SeekFrom::Start(data_offset))?;
        
        let mut data = [0u8; 16];
        if file.read_exact(&mut data).is_ok() {
            hash_info.md5 = Some(bytes_to_hex(&data));
        }
    }
    
    Ok(hash_info)
}

/// Parse error information from error2 section
fn parse_error_info(file: &mut File, sections: &[EwfSectionHeader]) -> Result<Vec<EwfErrorEntry>, ContainerError> {
    let mut errors = Vec::new();
    
    if let Some(section) = sections.iter().find(|s| s.section_type == "error2") {
        let data_offset = section.file_offset + SECTION_HEADER_SIZE as u64;
        
        file.seek(SeekFrom::Start(data_offset))?;
        
        let mut header = [0u8; 12];
        if file.read_exact(&mut header).is_err() {
            return Ok(errors);
        }
        
        let error_count = u32::from_le_bytes([header[0], header[1], header[2], header[3]]) as usize;
        
        // Limit to reasonable number
        let max_errors = error_count.min(10000);
        
        for _ in 0..max_errors {
            let mut entry = [0u8; 8];
            if file.read_exact(&mut entry).is_err() {
                break;
            }
            
            let first_sector = u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]);
            let sector_count = u32::from_le_bytes([entry[4], entry[5], entry[6], entry[7]]);
            
            errors.push(EwfErrorEntry {
                first_sector,
                sector_count,
            });
        }
    }
    
    Ok(errors)
}

/// Simple zlib decompression using flate2
fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>, ContainerError> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;
    
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Format bytes as hex string
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Format GUID bytes
fn format_guid(bytes: &[u8]) -> String {
    if bytes.len() < 16 {
        return bytes_to_hex(bytes);
    }
    format!(
        "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u16::from_le_bytes([bytes[4], bytes[5]]),
        u16::from_le_bytes([bytes[6], bytes[7]]),
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

/// Format media type as string
fn format_media_type(media_type: u32) -> &'static str {
    match media_type {
        0 => "Removable",
        1 => "Fixed Disk",
        3 => "Optical Disc",
        _ => "Unknown",
    }
}

/// Format compression level
fn format_compression(level: u8) -> String {
    match level {
        0 => "None".to_string(),
        1..=9 => format!("zlib level {}", level),
        _ => format!("Unknown ({})", level),
    }
}

// ============================================================================
// Conversion to ParsedMetadata for UI Display
// ============================================================================

/// Convert EwfDetailedInfo to ParsedMetadata for the HexViewer/MetadataPanel
pub fn ewf_detailed_info_to_metadata(info: &EwfDetailedInfo) -> ParsedMetadata {
    let mut fields = Vec::new();
    let mut regions = Vec::new();
    
    // Find section offsets for linking metadata fields to hex positions
    let header_offset = info.sections.iter()
        .find(|s| s.section_type == "header2" || s.section_type == "header")
        .map(|s| s.file_offset);
    let volume_offset = info.sections.iter()
        .find(|s| s.section_type == "volume")
        .map(|s| s.file_offset);
    let hash_offset = info.sections.iter()
        .find(|s| s.section_type == "hash" || s.section_type == "digest")
        .map(|s| s.file_offset);
    
    // ---- Format Information ----
    let format_desc = match info.variant {
        EwfVariant::E01 => "E01 (Physical Image)",
        EwfVariant::L01 => "L01 (Logical Evidence)",
        EwfVariant::Ex01 => "Ex01 (Physical Image v2)",
        EwfVariant::Lx01 => "Lx01 (Logical Evidence v2)",
        EwfVariant::Unknown => "Unknown",
    };
    
    fields.push(MetadataField {
        key: "Format".to_string(),
        value: format_desc.to_string(),
        category: "Format".to_string(),
        linked_region: Some("signature".to_string()),
        source_offset: Some(0),
    });
    
    fields.push(MetadataField {
        key: "EWF Version".to_string(),
        value: format!("v{}", info.version),
        category: "Format".to_string(),
        linked_region: Some("signature".to_string()),
        source_offset: Some(0),
    });
    
    fields.push(MetadataField {
        key: "Segment Number".to_string(),
        value: format!("{}", info.segment_number),
        category: "Format".to_string(),
        linked_region: Some("segment".to_string()),
        source_offset: Some(8),
    });
    
    fields.push(MetadataField {
        key: "File Size".to_string(),
        value: format_size(info.file_size),
        category: "Format".to_string(),
        ..Default::default()
    });
    
    fields.push(MetadataField {
        key: "Sections Found".to_string(),
        value: format!("{}", info.sections.len()),
        category: "Format".to_string(),
        ..Default::default()
    });
    
    // ---- Header Regions ----
    
    // Signature region
    regions.push(HeaderRegion {
        start: 0,
        end: 8,
        name: "EWF Signature".to_string(),
        color_class: "region-signature".to_string(),
        description: format!("{} file signature", info.variant),
    });
    
    // Segment info region
    regions.push(HeaderRegion {
        start: 8,
        end: 13,
        name: "Segment Info".to_string(),
        color_class: "region-segment".to_string(),
        description: format!("Segment {} identifier", info.segment_number),
    });
    
    // Section header regions
    for section in &info.sections {
        let header_end = section.file_offset + SECTION_HEADER_SIZE as u64;
        
        // Section type field (16 bytes)
        regions.push(HeaderRegion {
            start: section.file_offset,
            end: section.file_offset + 16,
            name: format!("Section: {}", section.section_type),
            color_class: "region-section-type".to_string(),
            description: format!("{} section type identifier", section.section_type),
        });
        
        // Next offset field (8 bytes)
        regions.push(HeaderRegion {
            start: section.file_offset + 16,
            end: section.file_offset + 24,
            name: "Next Offset".to_string(),
            color_class: "region-offset".to_string(),
            description: format!("Next section at 0x{:X}", section.next_offset),
        });
        
        // Section size field (8 bytes)
        regions.push(HeaderRegion {
            start: section.file_offset + 24,
            end: section.file_offset + 32,
            name: "Section Size".to_string(),
            color_class: "region-offset".to_string(),
            description: format!("{} bytes", section.section_size),
        });
        
        // Padding (40 bytes)
        regions.push(HeaderRegion {
            start: section.file_offset + 32,
            end: section.file_offset + 72,
            name: "Padding".to_string(),
            color_class: "region-reserved".to_string(),
            description: "Reserved padding bytes".to_string(),
        });
        
        // Checksum (4 bytes)
        regions.push(HeaderRegion {
            start: section.file_offset + 72,
            end: header_end,
            name: "Checksum".to_string(),
            color_class: "region-checksum".to_string(),
            description: format!("Adler-32: 0x{:08X}", section.checksum),
        });
        
        // Section data region (if not too large)
        if section.section_size > SECTION_HEADER_SIZE as u64 {
            let data_start = header_end;
            let data_end = section.file_offset + section.section_size;
            
            let data_class = match section.section_type.as_str() {
                "header" | "header2" => "region-metadata",
                "volume" | "disk" => "region-metadata",
                "sectors" | "data" => "region-data",
                "table" | "table2" => "region-offset",
                "hash" | "digest" => "region-hash",
                "error2" => "region-error",
                _ => "region-data",
            };
            
            regions.push(HeaderRegion {
                start: data_start,
                end: data_end.min(data_start + 1024), // Limit displayed region
                name: format!("{} Data", section.section_type),
                color_class: data_class.to_string(),
                description: format!("{} section data ({} bytes)", section.section_type, section.section_size - SECTION_HEADER_SIZE as u64),
            });
        }
    }
    
    // ---- Case Information ----
    if let Some(ref desc) = info.case_info.description {
        fields.push(MetadataField {
            key: "Description".to_string(),
            value: desc.clone(),
            category: "Case Info".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }
    
    if let Some(ref case_num) = info.case_info.case_number {
        fields.push(MetadataField {
            key: "Case Number".to_string(),
            value: case_num.clone(),
            category: "Case Info".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }
    
    if let Some(ref evidence_num) = info.case_info.evidence_number {
        fields.push(MetadataField {
            key: "Evidence Number".to_string(),
            value: evidence_num.clone(),
            category: "Case Info".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }
    
    if let Some(ref examiner) = info.case_info.examiner {
        fields.push(MetadataField {
            key: "Examiner".to_string(),
            value: examiner.clone(),
            category: "Case Info".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }
    
    if let Some(ref notes) = info.case_info.notes {
        fields.push(MetadataField {
            key: "Notes".to_string(),
            value: notes.clone(),
            category: "Case Info".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }
    
    if let Some(ref acq_date) = info.case_info.acquisition_date {
        fields.push(MetadataField {
            key: "Acquisition Date".to_string(),
            value: acq_date.clone(),
            category: "Acquisition".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }
    
    if let Some(ref acq_sw) = info.case_info.acquisition_software {
        fields.push(MetadataField {
            key: "Acquisition Software".to_string(),
            value: acq_sw.clone(),
            category: "Acquisition".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }
    
    if let Some(ref acq_os) = info.case_info.acquisition_os {
        fields.push(MetadataField {
            key: "Acquisition OS".to_string(),
            value: acq_os.clone(),
            category: "Acquisition".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }
    
    // ---- Device Information ----
    if let Some(ref model) = info.case_info.device_model {
        fields.push(MetadataField {
            key: "Device Model".to_string(),
            value: model.clone(),
            category: "Device".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }
    
    if let Some(ref serial) = info.case_info.device_serial {
        fields.push(MetadataField {
            key: "Serial Number".to_string(),
            value: serial.clone(),
            category: "Device".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }
    
    if let Some(total_bytes) = info.case_info.device_total_bytes {
        fields.push(MetadataField {
            key: "Total Bytes".to_string(),
            value: format_size(total_bytes),
            category: "Device".to_string(),
            linked_region: Some("header".to_string()),
            source_offset: header_offset,
        });
    }
    
    // ---- Volume Information ----
    if let Some(ref volume) = info.volume {
        fields.push(MetadataField {
            key: "Chunk Count".to_string(),
            value: format!("{}", volume.chunk_count),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });
        
        fields.push(MetadataField {
            key: "Sectors per Chunk".to_string(),
            value: format!("{}", volume.sectors_per_chunk),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });
        
        fields.push(MetadataField {
            key: "Bytes per Sector".to_string(),
            value: format!("{}", volume.bytes_per_sector),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });
        
        fields.push(MetadataField {
            key: "Total Sectors".to_string(),
            value: format!("{}", volume.sector_count),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });
        
        let image_size = volume.sector_count * volume.bytes_per_sector as u64;
        fields.push(MetadataField {
            key: "Image Size".to_string(),
            value: format_size(image_size),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });
        
        fields.push(MetadataField {
            key: "Chunk Size".to_string(),
            value: format_size((volume.sectors_per_chunk * volume.bytes_per_sector) as u64),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });
        
        fields.push(MetadataField {
            key: "Media Type".to_string(),
            value: format_media_type(volume.media_type).to_string(),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });
        
        fields.push(MetadataField {
            key: "Compression".to_string(),
            value: format_compression(volume.compression_level),
            category: "Volume".to_string(),
            linked_region: Some("volume".to_string()),
            source_offset: volume_offset,
        });
        
        if volume.chs_cylinders > 0 {
            fields.push(MetadataField {
                key: "CHS Geometry".to_string(),
                value: format!("{} / {} / {}", volume.chs_cylinders, volume.chs_heads, volume.chs_sectors),
                category: "Volume".to_string(),
                linked_region: Some("volume".to_string()),
                source_offset: volume_offset,
            });
        }
        
        if let Some(ref guid) = volume.guid {
            fields.push(MetadataField {
                key: "GUID".to_string(),
                value: guid.clone(),
                category: "Volume".to_string(),
                linked_region: Some("volume".to_string()),
                source_offset: volume_offset,
            });
        }
    }
    
    // ---- Hash Information ----
    if let Some(ref md5) = info.hashes.md5 {
        fields.push(MetadataField {
            key: "MD5".to_string(),
            value: md5.clone(),
            category: "Hashes".to_string(),
            linked_region: Some("hash".to_string()),
            source_offset: hash_offset,
        });
    }
    
    if let Some(ref sha1) = info.hashes.sha1 {
        fields.push(MetadataField {
            key: "SHA1".to_string(),
            value: sha1.clone(),
            category: "Hashes".to_string(),
            linked_region: Some("hash".to_string()),
            source_offset: hash_offset,
        });
    }
    
    if let Some(ref sha256) = info.hashes.sha256 {
        fields.push(MetadataField {
            key: "SHA256".to_string(),
            value: sha256.clone(),
            category: "Hashes".to_string(),
            linked_region: Some("hash".to_string()),
            source_offset: hash_offset,
        });
    }
    
    // ---- Error Information ----
    if !info.errors.is_empty() {
        fields.push(MetadataField {
            key: "Acquisition Errors".to_string(),
            value: format!("{} error regions", info.errors.len()),
            category: "Errors".to_string(), ..Default::default()
        });
        
        // Show first few errors
        for (i, error) in info.errors.iter().take(5).enumerate() {
            fields.push(MetadataField {
                key: format!("Error Region {}", i + 1),
                value: format!("Sectors {} - {} ({} sectors)", 
                    error.first_sector, 
                    error.first_sector + error.sector_count - 1,
                    error.sector_count
                ),
                category: "Errors".to_string(), ..Default::default()
            });
        }
        
        if info.errors.len() > 5 {
            fields.push(MetadataField {
                key: "...".to_string(),
                value: format!("and {} more error regions", info.errors.len() - 5),
                category: "Errors".to_string(), ..Default::default()
            });
        }
    }
    
    // ---- Section List ----
    for (i, section) in info.sections.iter().enumerate() {
        fields.push(MetadataField {
            key: format!("Section {}: {}", i + 1, section.section_type),
            value: format!("{} bytes", section.section_size),
            category: "Sections".to_string(),
            linked_region: Some(section.section_type.clone()),
            source_offset: Some(section.file_offset),
        });
    }
    
    ParsedMetadata {
        format: info.variant.to_string(),
        version: Some(format!("EWF v{}", info.version)),
        fields,
        regions,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::format_size;
    
    #[test]
    fn test_detect_ewf_variant() {
        // E01
        let e01_header = b"EVF\x09\x0d\x0a\xff\x00\x01\x00\x01\x00\x00";
        let (variant, version) = detect_ewf_variant(e01_header).unwrap();
        assert_eq!(variant, EwfVariant::E01);
        assert_eq!(version, 1);
        
        // L01
        let l01_header = b"LVF\x09\x0d\x0a\xff\x00\x01\x00\x01\x00\x00";
        let (variant, version) = detect_ewf_variant(l01_header).unwrap();
        assert_eq!(variant, EwfVariant::L01);
        assert_eq!(version, 1);
        
        // Ex01
        let ex01_header = b"EVF2\x0d\x0a\x81\x00\x01\x00\x01\x00";
        let (variant, version) = detect_ewf_variant(ex01_header).unwrap();
        assert_eq!(variant, EwfVariant::Ex01);
        assert_eq!(version, 2);
        
        // Lx01
        let lx01_header = b"LVF2\x0d\x0a\x81\x00\x01\x00\x01\x00";
        let (variant, version) = detect_ewf_variant(lx01_header).unwrap();
        assert_eq!(variant, EwfVariant::Lx01);
        assert_eq!(version, 2);
    }
    
    #[test]
    fn test_ewf_variant_methods() {
        assert!(EwfVariant::L01.is_logical());
        assert!(EwfVariant::Lx01.is_logical());
        assert!(!EwfVariant::E01.is_logical());
        
        assert!(EwfVariant::E01.is_physical());
        assert!(EwfVariant::Ex01.is_physical());
        assert!(!EwfVariant::L01.is_physical());
        
        assert!(EwfVariant::Ex01.is_v2());
        assert!(EwfVariant::Lx01.is_v2());
        assert!(!EwfVariant::E01.is_v2());
    }
    
    #[test]
    fn test_bytes_to_hex() {
        let bytes = [0x45, 0x56, 0x46, 0x09];
        assert_eq!(bytes_to_hex(&bytes), "45564609");
    }
    
    #[test]
    fn test_format_size() {
        assert_eq!(format_size(1024), "1.00 KB (1024 bytes)");
        assert_eq!(format_size(1048576), "1.00 MB (1048576 bytes)");
    }
}
