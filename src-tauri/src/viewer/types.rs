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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // FileChunk
    // =========================================================================

    #[test]
    fn file_chunk_new_basic() {
        let chunk = FileChunk::new(vec![1, 2, 3], 0, 10);
        assert_eq!(chunk.bytes, vec![1, 2, 3]);
        assert_eq!(chunk.offset, 0);
        assert_eq!(chunk.total_size, 10);
        assert!(chunk.has_more);
        assert!(!chunk.has_prev);
    }

    #[test]
    fn file_chunk_new_at_end() {
        let chunk = FileChunk::new(vec![0; 5], 5, 10);
        assert!(!chunk.has_more); // 5 + 5 = 10 == total
        assert!(chunk.has_prev);
    }

    #[test]
    fn file_chunk_new_middle() {
        let chunk = FileChunk::new(vec![0; 3], 3, 10);
        assert!(chunk.has_more);
        assert!(chunk.has_prev);
    }

    #[test]
    fn file_chunk_new_entire_file() {
        let chunk = FileChunk::new(vec![0; 10], 0, 10);
        assert!(!chunk.has_more);
        assert!(!chunk.has_prev);
    }

    #[test]
    fn file_chunk_empty() {
        let chunk = FileChunk::empty();
        assert!(chunk.bytes.is_empty());
        assert_eq!(chunk.offset, 0);
        assert_eq!(chunk.total_size, 0);
        assert!(!chunk.has_more);
        assert!(!chunk.has_prev);
    }

    // =========================================================================
    // FileTypeInfo
    // =========================================================================

    #[test]
    fn file_type_info_new() {
        let info = FileTypeInfo::new("JPEG Image", "jpg", "FFD8FF");
        assert_eq!(info.description, "JPEG Image");
        assert_eq!(info.extension, "jpg");
        assert_eq!(info.magic_hex, "FFD8FF");
        assert!(info.mime_type.is_none());
        assert!(!info.is_text);
        assert!(!info.is_forensic_format);
    }

    #[test]
    fn file_type_info_with_mime() {
        let info = FileTypeInfo::new("PNG", "png", "89504E47")
            .with_mime("image/png");
        assert_eq!(info.mime_type.as_deref(), Some("image/png"));
    }

    #[test]
    fn file_type_info_as_text() {
        let info = FileTypeInfo::new("Plain Text", "txt", "")
            .as_text();
        assert!(info.is_text);
        assert!(!info.is_forensic_format);
    }

    #[test]
    fn file_type_info_as_forensic() {
        let info = FileTypeInfo::new("AD1", "ad1", "41445345")
            .as_forensic();
        assert!(info.is_forensic_format);
        assert!(!info.is_text);
    }

    #[test]
    fn file_type_info_builder_chain() {
        let info = FileTypeInfo::new("EWF", "e01", "455646")
            .with_mime("application/x-ewf")
            .as_forensic();
        assert_eq!(info.mime_type.as_deref(), Some("application/x-ewf"));
        assert!(info.is_forensic_format);
    }

    // =========================================================================
    // HeaderRegion
    // =========================================================================

    #[test]
    fn header_region_new() {
        let region = HeaderRegion::new(0, 16, "Magic", color_class::SIGNATURE, "File signature");
        assert_eq!(region.start, 0);
        assert_eq!(region.end, 16);
        assert_eq!(region.name, "Magic");
        assert_eq!(region.color_class, "region-signature");
        assert_eq!(region.description, "File signature");
    }

    #[test]
    fn header_region_size() {
        let region = HeaderRegion::new(10, 30, "Data", color_class::DATA, "");
        assert_eq!(region.size(), 20);
    }

    #[test]
    fn header_region_size_zero_length() {
        let region = HeaderRegion::new(5, 5, "Empty", color_class::RESERVED, "");
        assert_eq!(region.size(), 0);
    }

    #[test]
    fn header_region_size_saturating() {
        // end < start shouldn't underflow
        let region = HeaderRegion {
            start: 10,
            end: 5,
            name: String::new(),
            color_class: String::new(),
            description: String::new(),
        };
        assert_eq!(region.size(), 0);
    }

    #[test]
    fn header_region_contains() {
        let region = HeaderRegion::new(10, 20, "Test", "", "");
        assert!(!region.contains(9));
        assert!(region.contains(10));
        assert!(region.contains(15));
        assert!(region.contains(19));
        assert!(!region.contains(20)); // exclusive end
        assert!(!region.contains(100));
    }

    // =========================================================================
    // MetadataField
    // =========================================================================

    #[test]
    fn metadata_field_new() {
        let field = MetadataField::new("File Size", "1024 bytes", "General");
        assert_eq!(field.key, "File Size");
        assert_eq!(field.value, "1024 bytes");
        assert_eq!(field.category, "General");
        assert!(field.linked_region.is_none());
        assert!(field.source_offset.is_none());
    }

    #[test]
    fn metadata_field_with_region() {
        let field = MetadataField::new("Magic", "ADSEGMENTEDFILE", "Header")
            .with_region("signature");
        assert_eq!(field.linked_region.as_deref(), Some("signature"));
    }

    #[test]
    fn metadata_field_with_offset() {
        let field = MetadataField::new("Version", "2", "Header")
            .with_offset(16);
        assert_eq!(field.source_offset, Some(16));
    }

    #[test]
    fn metadata_field_full_chain() {
        let field = MetadataField::new("Hash", "abc123", "Checksum")
            .with_region("checksum_region")
            .with_offset(512);
        assert_eq!(field.key, "Hash");
        assert_eq!(field.linked_region.as_deref(), Some("checksum_region"));
        assert_eq!(field.source_offset, Some(512));
    }

    // =========================================================================
    // ParsedMetadata
    // =========================================================================

    #[test]
    fn parsed_metadata_new() {
        let meta = ParsedMetadata::new("AD1");
        assert_eq!(meta.format, "AD1");
        assert!(meta.version.is_none());
        assert!(meta.fields.is_empty());
        assert!(meta.regions.is_empty());
    }

    #[test]
    fn parsed_metadata_with_version() {
        let meta = ParsedMetadata::new("EWF")
            .with_version("1.0");
        assert_eq!(meta.version.as_deref(), Some("1.0"));
    }

    #[test]
    fn parsed_metadata_with_field() {
        let meta = ParsedMetadata::new("Test")
            .with_field(MetadataField::new("Key", "Val", "Cat"));
        assert_eq!(meta.fields.len(), 1);
        assert_eq!(meta.fields[0].key, "Key");
    }

    #[test]
    fn parsed_metadata_with_fields() {
        let fields = vec![
            MetadataField::new("A", "1", "Cat"),
            MetadataField::new("B", "2", "Cat"),
        ];
        let meta = ParsedMetadata::new("Test")
            .with_fields(fields);
        assert_eq!(meta.fields.len(), 2);
    }

    #[test]
    fn parsed_metadata_with_region() {
        let meta = ParsedMetadata::new("Test")
            .with_region(HeaderRegion::new(0, 8, "Sig", color_class::SIGNATURE, "Desc"));
        assert_eq!(meta.regions.len(), 1);
        assert_eq!(meta.regions[0].name, "Sig");
    }

    #[test]
    fn parsed_metadata_with_regions() {
        let regions = vec![
            HeaderRegion::new(0, 8, "Sig", color_class::SIGNATURE, ""),
            HeaderRegion::new(8, 16, "Hdr", color_class::HEADER, ""),
        ];
        let meta = ParsedMetadata::new("Test")
            .with_regions(regions);
        assert_eq!(meta.regions.len(), 2);
    }

    #[test]
    fn parsed_metadata_add_field() {
        let mut meta = ParsedMetadata::new("Test");
        meta.add_field("Size", "1024", "General");
        assert_eq!(meta.fields.len(), 1);
        assert_eq!(meta.fields[0].key, "Size");
        assert_eq!(meta.fields[0].value, "1024");
        assert_eq!(meta.fields[0].category, "General");
    }

    #[test]
    fn parsed_metadata_add_region() {
        let mut meta = ParsedMetadata::new("Test");
        meta.add_region(0, 16, "Magic", color_class::SIGNATURE, "File signature");
        assert_eq!(meta.regions.len(), 1);
        assert_eq!(meta.regions[0].start, 0);
        assert_eq!(meta.regions[0].end, 16);
        assert_eq!(meta.regions[0].name, "Magic");
    }

    #[test]
    fn parsed_metadata_builder_chain() {
        let meta = ParsedMetadata::new("AD1")
            .with_version("2.0")
            .with_field(MetadataField::new("Type", "Logical", "Info"))
            .with_region(HeaderRegion::new(0, 4, "Sig", color_class::SIGNATURE, ""));
        assert_eq!(meta.format, "AD1");
        assert_eq!(meta.version.as_deref(), Some("2.0"));
        assert_eq!(meta.fields.len(), 1);
        assert_eq!(meta.regions.len(), 1);
    }

    // =========================================================================
    // color_class constants
    // =========================================================================

    #[test]
    fn color_class_constants_are_prefixed() {
        assert!(color_class::SIGNATURE.starts_with("region-"));
        assert!(color_class::HEADER.starts_with("region-"));
        assert!(color_class::SEGMENT.starts_with("region-"));
        assert!(color_class::METADATA.starts_with("region-"));
        assert!(color_class::DATA.starts_with("region-"));
        assert!(color_class::CHECKSUM.starts_with("region-"));
        assert!(color_class::RESERVED.starts_with("region-"));
        assert!(color_class::FOOTER.starts_with("region-"));
    }

    #[test]
    fn color_class_constants_are_unique() {
        let classes = vec![
            color_class::SIGNATURE,
            color_class::HEADER,
            color_class::SEGMENT,
            color_class::METADATA,
            color_class::DATA,
            color_class::CHECKSUM,
            color_class::RESERVED,
            color_class::FOOTER,
        ];
        let unique: std::collections::HashSet<_> = classes.iter().collect();
        // HEADER and SEGMENT both use "region-" prefix but should be unique strings
        // Actually HEADER="region-header" and SEGMENT="region-segment" so they are unique
        assert_eq!(unique.len(), classes.len());
    }
}
