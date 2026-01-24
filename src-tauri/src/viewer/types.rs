// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Viewer types - shared data structures for file viewing
//!
//! This module provides types for:
//! - File chunk reading for hex/text viewers
//! - File type detection results
//! - Header region highlighting
//! - Parsed metadata with builder patterns

use serde::{Deserialize, Serialize};

// =============================================================================
// File Chunk - Chunked file reading
// =============================================================================

/// Result of reading a file chunk
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileChunk {
    /// Raw bytes as a vector (will be serialized as array)
    pub bytes: Vec<u8>,
    /// Starting offset of this chunk
    pub offset: u64,
    /// Total file size
    pub total_size: u64,
    /// Whether there's more data after this chunk
    pub has_more: bool,
    /// Whether there's data before this chunk
    pub has_prev: bool,
}

impl FileChunk {
    /// Create a new file chunk
    #[inline]
    pub fn new(bytes: Vec<u8>, offset: u64, total_size: u64) -> Self {
        let chunk_end = offset + bytes.len() as u64;
        Self {
            bytes,
            offset,
            total_size,
            has_more: chunk_end < total_size,
            has_prev: offset > 0,
        }
    }
    
    /// Create an empty chunk (for error cases)
    #[inline]
    pub fn empty() -> Self {
        Self::default()
    }
}

// =============================================================================
// File Type Info - Detection results
// =============================================================================

/// File type detection result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileTypeInfo {
    /// Detected MIME type
    pub mime_type: Option<String>,
    /// Human-readable type description
    pub description: String,
    /// File extension
    pub extension: String,
    /// Whether this is likely a text file
    pub is_text: bool,
    /// Whether this is a known forensic format
    pub is_forensic_format: bool,
    /// Magic bytes (first 16 bytes as hex)
    pub magic_hex: String,
}

impl FileTypeInfo {
    /// Create a new file type info with all fields
    pub fn new(
        description: impl Into<String>,
        extension: impl Into<String>,
        magic_hex: impl Into<String>,
    ) -> Self {
        Self {
            mime_type: None,
            description: description.into(),
            extension: extension.into(),
            is_text: false,
            is_forensic_format: false,
            magic_hex: magic_hex.into(),
        }
    }
    
    /// Set the MIME type
    #[inline]
    pub fn with_mime(mut self, mime: impl Into<String>) -> Self {
        self.mime_type = Some(mime.into());
        self
    }
    
    /// Mark as text file
    #[inline]
    pub fn as_text(mut self) -> Self {
        self.is_text = true;
        self
    }
    
    /// Mark as forensic format
    #[inline]
    pub fn as_forensic(mut self) -> Self {
        self.is_forensic_format = true;
        self
    }
}

// =============================================================================
// Header Region - Hex view highlighting
// =============================================================================

/// Header region for color coding in hex view
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeaderRegion {
    /// Start offset
    pub start: u64,
    /// End offset (exclusive)
    pub end: u64,
    /// Region name/label
    pub name: String,
    /// Color class for styling
    pub color_class: String,
    /// Description/tooltip
    pub description: String,
}

// =============================================================================
// Parsed Metadata - File header analysis
// =============================================================================

/// Parsed metadata from file header
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParsedMetadata {
    /// File format name
    pub format: String,
    /// Version if detected
    pub version: Option<String>,
    /// Key-value metadata fields
    pub fields: Vec<MetadataField>,
    /// Header regions for hex highlighting
    pub regions: Vec<HeaderRegion>,
}

impl ParsedMetadata {
    /// Create a new parsed metadata instance
    pub fn new(format: impl Into<String>) -> Self {
        Self {
            format: format.into(),
            version: None,
            fields: Vec::new(),
            regions: Vec::new(),
        }
    }
    
    /// Set the version
    #[inline]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
    
    /// Add a metadata field
    #[inline]
    pub fn with_field(mut self, field: MetadataField) -> Self {
        self.fields.push(field);
        self
    }
    
    /// Add multiple metadata fields
    #[inline]
    pub fn with_fields(mut self, fields: impl IntoIterator<Item = MetadataField>) -> Self {
        self.fields.extend(fields);
        self
    }
    
    /// Add a header region
    #[inline]
    pub fn with_region(mut self, region: HeaderRegion) -> Self {
        self.regions.push(region);
        self
    }
    
    /// Add multiple header regions
    #[inline]
    pub fn with_regions(mut self, regions: impl IntoIterator<Item = HeaderRegion>) -> Self {
        self.regions.extend(regions);
        self
    }
    
    /// Add a simple key-value field
    #[inline]
    pub fn add_field(&mut self, key: impl Into<String>, value: impl Into<String>, category: impl Into<String>) {
        self.fields.push(MetadataField::new(key, value, category));
    }
    
    /// Add a region with standard parameters
    #[inline]
    pub fn add_region(&mut self, start: u64, end: u64, name: impl Into<String>, color_class: impl Into<String>, description: impl Into<String>) {
        self.regions.push(HeaderRegion::new(start, end, name, color_class, description));
    }
}

// =============================================================================
// Metadata Field - Individual key-value with linking
// =============================================================================

/// A single metadata field with optional linking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetadataField {
    pub key: String,
    pub value: String,
    pub category: String,
    /// Optional link to a hex region (region name) for click-to-highlight
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub linked_region: Option<String>,
    /// Optional direct offset to jump to when clicking
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub source_offset: Option<u64>,
}

impl MetadataField {
    /// Create a new metadata field
    pub fn new(key: impl Into<String>, value: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            category: category.into(),
            linked_region: None,
            source_offset: None,
        }
    }

    /// Create a new metadata field with a linked region
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.linked_region = Some(region.into());
        self
    }

    /// Create a new metadata field with a source offset
    pub fn with_offset(mut self, offset: u64) -> Self {
        self.source_offset = Some(offset);
        self
    }
}

impl HeaderRegion {
    /// Create a new header region
    pub fn new(
        start: u64,
        end: u64,
        name: impl Into<String>,
        color_class: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            start,
            end,
            name: name.into(),
            color_class: color_class.into(),
            description: description.into(),
        }
    }
    
    /// Get the size of this region in bytes
    #[inline]
    pub fn size(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }
    
    /// Check if an offset falls within this region
    #[inline]
    pub fn contains(&self, offset: u64) -> bool {
        offset >= self.start && offset < self.end
    }
}

// =============================================================================
// Color Class Constants - Consistent styling
// =============================================================================

/// Standard color classes for header regions
pub mod color_class {
    /// File signature/magic bytes (red)
    pub const SIGNATURE: &str = "region-signature";
    /// File header structure (orange)
    pub const HEADER: &str = "region-header";
    /// Segment markers (orange)
    pub const SEGMENT: &str = "region-segment";
    /// Metadata sections (yellow)
    pub const METADATA: &str = "region-metadata";
    /// Data payload (green)
    pub const DATA: &str = "region-data";
    /// Checksums/hashes (blue)
    pub const CHECKSUM: &str = "region-checksum";
    /// Reserved/padding (purple)
    pub const RESERVED: &str = "region-reserved";
    /// File footer (pink)
    pub const FOOTER: &str = "region-footer";
}
