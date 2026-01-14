// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Viewer types - shared data structures for file viewing

use serde::{Deserialize, Serialize};

/// Result of reading a file chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// File type detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Header region for color coding in hex view
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Parsed metadata from file header
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}
