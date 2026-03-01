// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Viewer hint generation and image utilities.
//!
//! Provides recommendations for the frontend on which viewer to use,
//! plus read-only image dimension reading and thumbnail creation.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::file_info::{read_as_text, FileInfo};
use super::{UniversalFormat, ViewerType};
use crate::viewer::document::error::{DocumentError, DocumentResult};

// =============================================================================
// IMAGE UTILITIES (READ-ONLY, OUTPUT TO TEMP)
// =============================================================================

/// Image dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

/// Get image dimensions without loading full image
pub fn get_image_dimensions(path: impl AsRef<Path>) -> DocumentResult<ImageDimensions> {
    let path = path.as_ref();

    // Use image crate to read dimensions only
    let reader = image::ImageReader::open(path)
        .map_err(|e| DocumentError::Io(std::io::Error::other(e.to_string())))?;

    let (width, height) = reader
        .into_dimensions()
        .map_err(|e| DocumentError::Io(std::io::Error::other(e.to_string())))?;

    Ok(ImageDimensions { width, height })
}

/// Create thumbnail in temp directory (does NOT modify original)
pub fn create_thumbnail(path: impl AsRef<Path>, max_size: u32) -> DocumentResult<PathBuf> {
    let path = path.as_ref();

    // Load image
    let img =
        image::open(path).map_err(|e| DocumentError::Io(std::io::Error::other(e.to_string())))?;

    // Resize maintaining aspect ratio
    let thumbnail = img.thumbnail(max_size, max_size);

    // Save to temp directory
    let temp_dir = std::env::temp_dir().join("core-ffx-thumbnails");
    fs::create_dir_all(&temp_dir)?;

    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("thumb");
    let thumb_path = temp_dir.join(format!("{}_{}.png", file_stem, max_size));

    thumbnail
        .save(&thumb_path)
        .map_err(|e| DocumentError::Io(std::io::Error::other(e.to_string())))?;

    Ok(thumb_path)
}

/// Create thumbnail as base64 data URL (in memory, no temp file)
pub fn create_thumbnail_data_url(path: impl AsRef<Path>, max_size: u32) -> DocumentResult<String> {
    let path = path.as_ref();

    // Load image
    let img =
        image::open(path).map_err(|e| DocumentError::Io(std::io::Error::other(e.to_string())))?;

    // Resize maintaining aspect ratio
    let thumbnail = img.thumbnail(max_size, max_size);

    // Encode to PNG in memory
    let mut buffer = Vec::new();
    thumbnail
        .write_to(
            &mut std::io::Cursor::new(&mut buffer),
            image::ImageFormat::Png,
        )
        .map_err(|e| DocumentError::Io(std::io::Error::other(e.to_string())))?;

    Ok(format!("data:image/png;base64,{}", BASE64.encode(&buffer)))
}

// =============================================================================
// VIEWER HINT GENERATION
// =============================================================================

/// Viewer hint for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewerHint {
    /// Recommended viewer type
    pub viewer: ViewerType,
    /// Format details
    pub format: UniversalFormat,
    /// MIME type for Content-Type header
    pub mime_type: String,
    /// Whether content can be rendered (vs just showing metadata)
    pub can_render: bool,
    /// Whether text search is supported
    pub can_search: bool,
    /// Whether content can be copied
    pub can_copy: bool,
    /// Suggested display mode
    pub display_mode: DisplayMode,
    /// Any viewer-specific config
    pub config: ViewerConfig,
}

/// Display mode suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisplayMode {
    /// Inline in viewer panel
    Inline,
    /// Full screen / modal
    Fullscreen,
    /// Side panel
    SidePanel,
    /// New tab/window
    NewTab,
}

/// Viewer-specific configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViewerConfig {
    /// For images: dimensions
    pub dimensions: Option<ImageDimensions>,
    /// For text: detected encoding
    pub encoding: Option<String>,
    /// For text: detected language (for syntax highlighting)
    pub language: Option<String>,
    /// For text: line count
    pub line_count: Option<usize>,
    /// For archives: entry count
    pub entry_count: Option<usize>,
}

/// Get viewer hint for a file
pub fn get_viewer_hint(path: impl AsRef<Path>) -> DocumentResult<ViewerHint> {
    let path = path.as_ref();
    let info = FileInfo::from_path(path)?;

    let can_render = matches!(
        info.viewer_type,
        ViewerType::Image
            | ViewerType::Svg
            | ViewerType::Pdf
            | ViewerType::Text
            | ViewerType::Html
            | ViewerType::Spreadsheet
            | ViewerType::Email
            | ViewerType::Plist
            | ViewerType::Database
            | ViewerType::Binary
            | ViewerType::Registry
    );

    let can_search = matches!(
        info.viewer_type,
        ViewerType::Text | ViewerType::Html | ViewerType::Pdf | ViewerType::Spreadsheet
    );

    let can_copy = matches!(
        info.viewer_type,
        ViewerType::Text | ViewerType::Html | ViewerType::Hex
    );

    let display_mode = match info.viewer_type {
        ViewerType::Pdf | ViewerType::Image => DisplayMode::Fullscreen,
        ViewerType::Office | ViewerType::Archive => DisplayMode::SidePanel,
        _ => DisplayMode::Inline,
    };

    // Build config based on type
    let mut config = ViewerConfig::default();

    match info.viewer_type {
        ViewerType::Image | ViewerType::Svg => {
            if let Ok(dims) = get_image_dimensions(path) {
                config.dimensions = Some(dims);
            }
        }
        ViewerType::Text => {
            config.language = detect_language(path);
            config.encoding = Some("utf-8".to_string()); // Assume UTF-8
            if let Ok((text, _)) = read_as_text(path, 1024 * 1024) {
                config.line_count = Some(text.lines().count());
            }
        }
        _ => {}
    }

    Ok(ViewerHint {
        viewer: info.viewer_type,
        format: info.format,
        mime_type: info.mime_type,
        can_render,
        can_search,
        can_copy,
        display_mode,
        config,
    })
}

/// Detect programming language from file extension
fn detect_language(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_lowercase();

    let lang = match ext.as_str() {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "jsx" | "tsx" => "javascript",
        "c" | "h" => "c",
        "cpp" | "hpp" | "cc" | "cxx" => "cpp",
        "java" => "java",
        "go" => "go",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "kt" => "kotlin",
        "sh" | "bash" | "zsh" => "bash",
        "ps1" => "powershell",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" | "sass" => "scss",
        "sql" => "sql",
        "md" | "markdown" => "markdown",
        _ => return None,
    };

    Some(lang.to_string())
}
