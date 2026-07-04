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
use std::io::Read;
use std::path::{Path, PathBuf};

use super::file_info::{read_as_text, FileInfo};
use super::{UniversalFormat, ViewerType};
use crate::common::sanitize_filename;
use crate::viewer::document::error::{DocumentError, DocumentResult};

const MAX_THUMBNAIL_SOURCE_BYTES: u64 = 100 * 1024 * 1024;
const MAX_THUMBNAIL_PIXELS: u64 = 100_000_000;
const MAX_THUMBNAIL_OUTPUT_EDGE: u32 = 4096;
const MAX_THUMBNAIL_DATA_URL_PNG_BYTES: usize = 16 * 1024 * 1024;

fn ensure_image_dimensions_allowed(dimensions: &ImageDimensions) -> DocumentResult<()> {
    if dimensions.width == 0 || dimensions.height == 0 {
        return Err(DocumentError::Parse(
            "Image dimensions must be greater than zero".to_string(),
        ));
    }

    let pixels = u64::from(dimensions.width)
        .checked_mul(u64::from(dimensions.height))
        .ok_or_else(|| DocumentError::Parse("Image dimensions overflow pixel count".to_string()))?;
    if pixels > MAX_THUMBNAIL_PIXELS {
        return Err(DocumentError::Parse(format!(
            "Image dimensions too large ({}x{}, max {} pixels)",
            dimensions.width, dimensions.height, MAX_THUMBNAIL_PIXELS
        )));
    }
    Ok(())
}

fn validate_thumbnail_max_size(max_size: u32) -> DocumentResult<()> {
    if max_size == 0 {
        return Err(DocumentError::Parse(
            "Thumbnail size must be greater than zero".to_string(),
        ));
    }
    if max_size > MAX_THUMBNAIL_OUTPUT_EDGE {
        return Err(DocumentError::Parse(format!(
            "Thumbnail size too large ({}px, max {}px)",
            max_size, MAX_THUMBNAIL_OUTPUT_EDGE
        )));
    }
    Ok(())
}

fn ensure_thumbnail_dimensions_allowed(dimensions: &ImageDimensions) -> DocumentResult<()> {
    ensure_image_dimensions_allowed(dimensions).map_err(|_| {
        DocumentError::Parse(format!(
            "Image dimensions too large for thumbnail generation ({}x{}, max {} pixels)",
            dimensions.width, dimensions.height, MAX_THUMBNAIL_PIXELS
        ))
    })
}

fn ensure_thumbnail_data_url_output_allowed(encoded_png_bytes: usize) -> DocumentResult<()> {
    if encoded_png_bytes > MAX_THUMBNAIL_DATA_URL_PNG_BYTES {
        return Err(DocumentError::Parse(format!(
            "Thumbnail data URL too large ({:.1} MiB PNG, max {} MiB)",
            encoded_png_bytes as f64 / (1024.0 * 1024.0),
            MAX_THUMBNAIL_DATA_URL_PNG_BYTES / (1024 * 1024)
        )));
    }
    Ok(())
}

fn ensure_thumbnail_safe(path: &Path) -> DocumentResult<()> {
    let file_size = fs::metadata(path)?.len();
    if file_size > MAX_THUMBNAIL_SOURCE_BYTES {
        return Err(DocumentError::Parse(format!(
            "Image file too large for thumbnail generation ({:.1} MiB, max {} MiB)",
            file_size as f64 / (1024.0 * 1024.0),
            MAX_THUMBNAIL_SOURCE_BYTES / (1024 * 1024)
        )));
    }

    let dimensions = get_image_dimensions(path)?;
    ensure_thumbnail_dimensions_allowed(&dimensions)
}

fn read_source_with_limit<R: Read>(reader: R, max_bytes: u64) -> DocumentResult<Vec<u8>> {
    let mut data = Vec::new();
    reader
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut data)?;
    if data.len() as u64 > max_bytes {
        return Err(DocumentError::Parse(format!(
            "Image file too large for thumbnail generation (actual read exceeded max {} MiB)",
            max_bytes / (1024 * 1024)
        )));
    }
    Ok(data)
}

fn read_thumbnail_source_with_limit<R: Read>(reader: R) -> DocumentResult<Vec<u8>> {
    read_source_with_limit(reader, MAX_THUMBNAIL_SOURCE_BYTES)
}

fn load_thumbnail_image(path: &Path) -> DocumentResult<image::DynamicImage> {
    let data = read_thumbnail_source_with_limit(fs::File::open(path)?)?;
    image::load_from_memory(&data)
        .map_err(|e| DocumentError::Io(std::io::Error::other(e.to_string())))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn thumbnail_source_identity(path: &Path) -> String {
    let mut identity = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string();

    if let Ok(metadata) = fs::metadata(path) {
        identity.push('|');
        identity.push_str(&metadata.len().to_string());
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                identity.push('|');
                identity.push_str(&duration.as_secs().to_string());
                identity.push('.');
                identity.push_str(&duration.subsec_nanos().to_string());
            }
        }
    }

    identity
}

fn thumbnail_temp_file_name(path: &Path, max_size: u32) -> String {
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("thumb");
    let sanitized_stem = {
        let sanitized = sanitize_filename(file_stem);
        if sanitized.is_empty() {
            "thumb".to_string()
        } else {
            sanitized
        }
    };
    let mut identity = thumbnail_source_identity(path);
    identity.push('|');
    identity.push_str(&max_size.to_string());
    let source_hash = fnv1a64(identity.as_bytes());
    format!("{sanitized_stem}_{max_size}_{source_hash:016x}.png")
}

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

    let dimensions = ImageDimensions { width, height };
    ensure_image_dimensions_allowed(&dimensions)?;

    Ok(dimensions)
}

/// Create thumbnail in temp directory (does NOT modify original)
pub fn create_thumbnail(path: impl AsRef<Path>, max_size: u32) -> DocumentResult<PathBuf> {
    let path = path.as_ref();
    validate_thumbnail_max_size(max_size)?;
    ensure_thumbnail_safe(path)?;

    // Load image from capped bytes so decode cannot race past the size check.
    let img = load_thumbnail_image(path)?;

    // Resize maintaining aspect ratio
    let thumbnail = img.thumbnail(max_size, max_size);

    // Save to temp directory
    let temp_dir = crate::commands::portable::portable_temp_dir()
        .join(crate::app_paths::THUMBNAIL_TEMP_DIR_NAME);
    fs::create_dir_all(&temp_dir)?;

    let thumb_path = temp_dir.join(thumbnail_temp_file_name(path, max_size));

    thumbnail
        .save(&thumb_path)
        .map_err(|e| DocumentError::Io(std::io::Error::other(e.to_string())))?;

    Ok(thumb_path)
}

/// Create thumbnail as base64 data URL (in memory, no temp file)
pub fn create_thumbnail_data_url(path: impl AsRef<Path>, max_size: u32) -> DocumentResult<String> {
    let path = path.as_ref();
    validate_thumbnail_max_size(max_size)?;
    ensure_thumbnail_safe(path)?;

    // Load image from capped bytes so decode cannot race past the size check.
    let img = load_thumbnail_image(path)?;

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
    ensure_thumbnail_data_url_output_allowed(buffer.len())?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn thumbnail_rejects_sparse_oversized_source_before_decode() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"not an image").unwrap();
        tmp.as_file_mut()
            .set_len(MAX_THUMBNAIL_SOURCE_BYTES + 1)
            .unwrap();

        let err = create_thumbnail_data_url(tmp.path(), 128).unwrap_err();

        assert!(err
            .to_string()
            .contains("too large for thumbnail generation"));
    }

    #[test]
    fn thumbnail_rejects_zero_output_size() {
        let err = validate_thumbnail_max_size(0).unwrap_err();

        assert!(err.to_string().contains("greater than zero"));
    }

    #[test]
    fn thumbnail_rejects_excessive_output_size() {
        let err = validate_thumbnail_max_size(MAX_THUMBNAIL_OUTPUT_EDGE + 1).unwrap_err();

        assert!(err.to_string().contains("Thumbnail size too large"));
    }

    #[test]
    fn thumbnail_temp_file_name_sanitizes_and_hashes_source_identity() {
        let first = thumbnail_temp_file_name(Path::new("/case-a/folder/photo?.jpg"), 128);
        let second = thumbnail_temp_file_name(Path::new("/case-b/folder/photo?.jpg"), 128);

        assert!(first.starts_with("photo_128_"), "unexpected: {first}");
        assert!(first.ends_with(".png"));
        assert!(!first.contains('?'));
        assert_ne!(first, second);
    }

    #[test]
    fn thumbnail_temp_file_name_falls_back_for_empty_stem() {
        let name = thumbnail_temp_file_name(Path::new("/case/..."), 128);

        assert!(name.starts_with("thumb_128_"), "unexpected: {name}");
    }

    #[test]
    fn thumbnail_source_reader_rejects_actual_read_limit() {
        struct ZeroReader {
            remaining: u64,
        }

        impl std::io::Read for ZeroReader {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                if self.remaining == 0 {
                    return Ok(0);
                }
                let to_read = (buf.len() as u64).min(self.remaining) as usize;
                buf[..to_read].fill(0);
                self.remaining -= to_read as u64;
                Ok(to_read)
            }
        }

        let err = read_source_with_limit(ZeroReader { remaining: 4 }, 3).unwrap_err();

        assert!(err
            .to_string()
            .contains("too large for thumbnail generation"));
    }

    #[test]
    fn thumbnail_rejects_excessive_dimensions() {
        let dimensions = ImageDimensions {
            width: 100_001,
            height: 1_000,
        };

        let err = ensure_thumbnail_dimensions_allowed(&dimensions).unwrap_err();

        assert!(err
            .to_string()
            .contains("dimensions too large for thumbnail generation"));
    }

    #[test]
    fn image_dimensions_reject_zero_or_excessive_values() {
        let zero = ImageDimensions {
            width: 0,
            height: 100,
        };
        assert!(ensure_image_dimensions_allowed(&zero)
            .unwrap_err()
            .to_string()
            .contains("greater than zero"));

        let excessive = ImageDimensions {
            width: 100_001,
            height: 1_000,
        };
        assert!(ensure_image_dimensions_allowed(&excessive)
            .unwrap_err()
            .to_string()
            .contains("dimensions too large"));
    }

    #[test]
    fn thumbnail_data_url_rejects_oversized_encoded_png() {
        let err = ensure_thumbnail_data_url_output_allowed(MAX_THUMBNAIL_DATA_URL_PNG_BYTES + 1)
            .unwrap_err();

        assert!(err.to_string().contains("Thumbnail data URL too large"));
    }

    #[test]
    fn thumbnail_data_url_allows_max_encoded_png_size() {
        ensure_thumbnail_data_url_output_allowed(MAX_THUMBNAIL_DATA_URL_PNG_BYTES).unwrap();
    }
}
