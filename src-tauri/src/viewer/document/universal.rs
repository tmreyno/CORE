// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Universal Document Viewer - Read-Only
//!
//! This module provides read-only document viewing capabilities.
//! It NEVER modifies original files - forensic integrity is paramount.
//!
//! # Architecture
//!
//! The viewer follows a simple principle:
//! - **Frontend handles rendering** using established libraries (PDF.js, native <img>, etc.)
//! - **Backend provides data** (bytes, metadata, thumbnails to temp dir)
//!
//! # Supported Formats
//!
//! ## Direct Frontend Rendering (Recommended)
//! - **PDF**: PDF.js (already installed) - renders in browser
//! - **Images**: Native `<img>` tag - PNG, JPEG, GIF, WebP, BMP, ICO, SVG
//! - **Text**: TextViewer component - TXT, LOG, JSON, XML, CSV, MD, etc.
//! - **HTML**: Sanitized iframe or DOMPurify
//!
//! ## Backend-Assisted (for metadata/thumbnails only)
//! - Office docs: Metadata extraction only, no conversion
//! - Archives: List contents, no extraction to disk
//!
//! # Safety Guarantees
//!
//! 1. Original files are NEVER modified
//! 2. Temporary files only created in system temp dir
//! 3. No external process execution (no LibreOffice, etc.)
//! 4. Memory-mapped reading where possible

use std::path::{Path, PathBuf};
use std::fs;
use std::io::Read;
use serde::{Serialize, Deserialize};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use super::error::{DocumentError, DocumentResult};

// =============================================================================
// FORMAT DETECTION
// =============================================================================

/// Supported universal formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UniversalFormat {
    // Images (direct frontend rendering)
    Png,
    Jpeg,
    Gif,
    WebP,
    Bmp,
    Tiff,
    Ico,
    Svg,
    Heic,
    
    // Documents (PDF.js or text rendering)
    Pdf,
    Text,
    Html,
    Markdown,
    Json,
    Xml,
    Csv,
    
    // Office (metadata only, no rendering)
    Docx,
    Xlsx,
    Pptx,
    Doc,
    Xls,
    Ppt,
    Odt,
    Ods,
    Odp,
    Rtf,
    
    // Archives (listing only)
    Zip,
    SevenZ,
    Rar,
    Tar,
    Gz,
    
    // Email
    Eml,
    Msg,
    Mbox,
    
    // Apple/iOS
    Plist,
    Mobileprovision,
    
    // Executables
    Exe,
    Dll,
    So,
    Dylib,
    MachO,
    
    // Database
    Sqlite,
    Db,
    
    // Binary (hex view)
    #[default]
    Binary,
}

impl UniversalFormat {
    /// Detect format from file path
    pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        let ext = path.as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())?;
        
        Self::from_extension(&ext)
    }
    
    /// Detect format from extension string
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            // Images
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "gif" => Some(Self::Gif),
            "webp" => Some(Self::WebP),
            "bmp" => Some(Self::Bmp),
            "tiff" | "tif" => Some(Self::Tiff),
            "ico" => Some(Self::Ico),
            "svg" => Some(Self::Svg),
            "heic" | "heif" => Some(Self::Heic),
            
            // Documents
            "pdf" => Some(Self::Pdf),
            "txt" | "log" | "cfg" | "ini" | "conf" | "env" => Some(Self::Text),
            "html" | "htm" => Some(Self::Html),
            "md" | "markdown" => Some(Self::Markdown),
            "json" => Some(Self::Json),
            "xml" => Some(Self::Xml),
            "csv" | "tsv" => Some(Self::Csv),
            
            // Source code (treat as text)
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "c" | "cpp" | "h" | "hpp" |
            "java" | "go" | "rb" | "php" | "swift" | "kt" | "scala" | "sh" | "bash" |
            "zsh" | "ps1" | "bat" | "cmd" | "yaml" | "yml" | "toml" | "sql" |
            "css" | "scss" | "sass" | "less" => Some(Self::Text),
            
            // Office
            "docx" => Some(Self::Docx),
            "xlsx" => Some(Self::Xlsx),
            "pptx" => Some(Self::Pptx),
            "doc" => Some(Self::Doc),
            "xls" => Some(Self::Xls),
            "ppt" => Some(Self::Ppt),
            "odt" => Some(Self::Odt),
            "ods" => Some(Self::Ods),
            "odp" => Some(Self::Odp),
            "rtf" => Some(Self::Rtf),
            
            // Archives
            "zip" => Some(Self::Zip),
            "7z" => Some(Self::SevenZ),
            "rar" => Some(Self::Rar),
            "tar" => Some(Self::Tar),
            "gz" | "gzip" => Some(Self::Gz),
            
            // Email
            "eml" => Some(Self::Eml),
            "msg" => Some(Self::Msg),
            "mbox" => Some(Self::Mbox),
            
            // Apple/iOS
            "plist" => Some(Self::Plist),
            "mobileprovision" => Some(Self::Mobileprovision),
            
            // Executables
            "exe" => Some(Self::Exe),
            "dll" => Some(Self::Dll),
            "so" => Some(Self::So),
            "dylib" => Some(Self::Dylib),
            
            // Database
            "sqlite" | "sqlite3" | "sqlitedb" => Some(Self::Sqlite),
            "db" | "db3" => Some(Self::Db),
            
            _ => Some(Self::Binary),
        }
    }
    
    /// Get recommended viewer type
    pub fn viewer_type(&self) -> ViewerType {
        match self {
            // Frontend renders these directly
            Self::Png | Self::Jpeg | Self::Gif | Self::WebP | 
            Self::Bmp | Self::Ico | Self::Heic => ViewerType::Image,
            Self::Svg => ViewerType::Svg,
            Self::Pdf => ViewerType::Pdf,
            Self::Text | Self::Json | Self::Xml | Self::Csv |
            Self::Markdown => ViewerType::Text,
            Self::Html => ViewerType::Html,
            Self::Tiff => ViewerType::Image, // Most browsers support TIFF now
            
            // Backend provides metadata only
            Self::Docx | Self::Xlsx | Self::Pptx | Self::Doc | 
            Self::Xls | Self::Ppt | Self::Odt | Self::Ods | 
            Self::Odp | Self::Rtf => ViewerType::Office,
            
            // Archive listing
            Self::Zip | Self::SevenZ | Self::Rar | 
            Self::Tar | Self::Gz => ViewerType::Archive,
            
            // Email viewer
            Self::Eml | Self::Msg | Self::Mbox => ViewerType::Email,
            
            // Plist viewer (structured data)
            Self::Plist | Self::Mobileprovision => ViewerType::Plist,
            
            // Binary analysis
            Self::Exe | Self::Dll | Self::So | Self::Dylib | Self::MachO => ViewerType::Binary,
            
            // Database viewer
            Self::Sqlite | Self::Db => ViewerType::Database,
            
            // Fallback
            Self::Binary => ViewerType::Hex,
        }
    }
    
    /// Get MIME type
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Gif => "image/gif",
            Self::WebP => "image/webp",
            Self::Bmp => "image/bmp",
            Self::Tiff => "image/tiff",
            Self::Ico => "image/x-icon",
            Self::Svg => "image/svg+xml",
            Self::Heic => "image/heic",
            Self::Pdf => "application/pdf",
            Self::Text => "text/plain",
            Self::Html => "text/html",
            Self::Markdown => "text/markdown",
            Self::Json => "application/json",
            Self::Xml => "application/xml",
            Self::Csv => "text/csv",
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Self::Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            Self::Pptx => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            Self::Doc => "application/msword",
            Self::Xls => "application/vnd.ms-excel",
            Self::Ppt => "application/vnd.ms-powerpoint",
            Self::Odt => "application/vnd.oasis.opendocument.text",
            Self::Ods => "application/vnd.oasis.opendocument.spreadsheet",
            Self::Odp => "application/vnd.oasis.opendocument.presentation",
            Self::Rtf => "application/rtf",
            Self::Zip => "application/zip",
            Self::SevenZ => "application/x-7z-compressed",
            Self::Rar => "application/vnd.rar",
            Self::Tar => "application/x-tar",
            Self::Gz => "application/gzip",
            Self::Eml => "message/rfc822",
            Self::Msg => "application/vnd.ms-outlook",
            Self::Mbox => "application/mbox",
            Self::Plist => "application/x-plist",
            Self::Mobileprovision => "application/x-apple-aspen-config",
            Self::Exe => "application/vnd.microsoft.portable-executable",
            Self::Dll => "application/vnd.microsoft.portable-executable",
            Self::So => "application/x-sharedlib",
            Self::Dylib => "application/x-mach-dylib",
            Self::MachO => "application/x-mach-binary",
            Self::Sqlite => "application/x-sqlite3",
            Self::Db => "application/x-sqlite3",
            Self::Binary => "application/octet-stream",
        }
    }
    
    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Png => "PNG Image",
            Self::Jpeg => "JPEG Image",
            Self::Gif => "GIF Image",
            Self::WebP => "WebP Image",
            Self::Bmp => "Bitmap Image",
            Self::Tiff => "TIFF Image",
            Self::Ico => "Icon",
            Self::Svg => "SVG Vector",
            Self::Heic => "HEIC Image",
            Self::Pdf => "PDF Document",
            Self::Text => "Text File",
            Self::Html => "HTML Document",
            Self::Markdown => "Markdown",
            Self::Json => "JSON Data",
            Self::Xml => "XML Document",
            Self::Csv => "CSV Spreadsheet",
            Self::Docx => "Word Document",
            Self::Xlsx => "Excel Spreadsheet",
            Self::Pptx => "PowerPoint",
            Self::Doc => "Word Document (Legacy)",
            Self::Xls => "Excel (Legacy)",
            Self::Ppt => "PowerPoint (Legacy)",
            Self::Odt => "OpenDocument Text",
            Self::Ods => "OpenDocument Spreadsheet",
            Self::Odp => "OpenDocument Presentation",
            Self::Rtf => "Rich Text",
            Self::Zip => "ZIP Archive",
            Self::SevenZ => "7-Zip Archive",
            Self::Rar => "RAR Archive",
            Self::Tar => "TAR Archive",
            Self::Gz => "Gzip Archive",
            Self::Eml => "Email Message",
            Self::Msg => "Outlook Message",
            Self::Mbox => "Mailbox Archive",
            Self::Plist => "Property List",
            Self::Mobileprovision => "iOS Provisioning Profile",
            Self::Exe => "Windows Executable",
            Self::Dll => "Windows DLL",
            Self::So => "Shared Library",
            Self::Dylib => "macOS Dynamic Library",
            Self::MachO => "macOS Executable",
            Self::Sqlite => "SQLite Database",
            Self::Db => "Database File",
            Self::Binary => "Binary File",
        }
    }
    
    /// Get all supported extensions
    pub fn all_extensions() -> &'static [&'static str] {
        &[
            // Images
            "png", "jpg", "jpeg", "gif", "webp", "bmp", "tiff", "tif", "ico", "svg",
            // Documents
            "pdf", "txt", "log", "cfg", "ini", "conf", "html", "htm", "md", "markdown",
            "json", "xml", "csv", "tsv",
            // Code
            "rs", "py", "js", "ts", "jsx", "tsx", "c", "cpp", "h", "hpp",
            "java", "go", "rb", "php", "swift", "kt", "sh", "yaml", "yml", "toml",
            // Office
            "docx", "xlsx", "pptx", "doc", "xls", "ppt", "odt", "ods", "odp", "rtf",
            // Archives
            "zip", "7z", "rar", "tar", "gz",
        ]
    }
}

/// Viewer type recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewerType {
    /// Use native <img> tag
    Image,
    /// Use inline SVG or <img>
    Svg,
    /// Use PDF.js
    Pdf,
    /// Use TextViewer component
    Text,
    /// Use sanitized iframe or DOMPurify
    Html,
    /// Show metadata only (no content rendering)
    Office,
    /// Show archive contents listing
    Archive,
    /// Use HexViewer
    Hex,
    /// Email message viewer
    Email,
    /// Property list viewer (structured data)
    Plist,
    /// Binary/executable analysis viewer
    Binary,
    /// Database viewer
    Database,
}

// =============================================================================
// FILE INFO (READ-ONLY)
// =============================================================================

/// File information (read-only metadata extraction)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub format: UniversalFormat,
    pub viewer_type: ViewerType,
    pub mime_type: String,
    pub description: String,
    pub size: u64,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub is_readable: bool,
    pub is_binary: bool,
}

impl FileInfo {
    /// Get file info without reading content (fast)
    pub fn from_path(path: impl AsRef<Path>) -> DocumentResult<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(DocumentError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.display())
            )));
        }
        
        let meta = fs::metadata(path)?;
        let format = UniversalFormat::from_path(path).unwrap_or(UniversalFormat::Binary);
        
        // Check if binary by reading first bytes
        let is_binary = Self::check_binary(path);
        
        Ok(Self {
            path: path.to_string_lossy().to_string(),
            name: path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            format,
            viewer_type: format.viewer_type(),
            mime_type: format.mime_type().to_string(),
            description: format.description().to_string(),
            size: meta.len(),
            created: meta.created().ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| format_timestamp(d.as_secs())),
            modified: meta.modified().ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| format_timestamp(d.as_secs())),
            is_readable: path.is_file(),
            is_binary,
        })
    }
    
    /// Quick check if file appears to be binary
    fn check_binary(path: &Path) -> bool {
        if let Ok(mut file) = fs::File::open(path) {
            let mut buffer = [0u8; 8192];
            if let Ok(n) = file.read(&mut buffer) {
                // Count null bytes and non-printable chars
                let null_count = buffer[..n].iter().filter(|&&b| b == 0).count();
                let non_printable = buffer[..n].iter()
                    .filter(|&&b| b < 0x09 || (b > 0x0D && b < 0x20 && b != 0x1B))
                    .count();
                
                // If more than 10% null or non-printable, likely binary
                return null_count > n / 10 || non_printable > n / 10;
            }
        }
        true // Default to binary if can't read
    }
}

fn format_timestamp(secs: u64) -> String {
    use chrono::{DateTime, Utc};
    DateTime::<Utc>::from_timestamp(secs as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

// =============================================================================
// CONTENT READING (READ-ONLY)
// =============================================================================

/// Read file as base64 data URL (for images)
pub fn read_as_data_url(path: impl AsRef<Path>) -> DocumentResult<String> {
    let path = path.as_ref();
    let format = UniversalFormat::from_path(path).unwrap_or(UniversalFormat::Binary);
    let data = fs::read(path)?;
    let mime = format.mime_type();
    Ok(format!("data:{};base64,{}", mime, BASE64.encode(&data)))
}

/// Read file as text (with size limit)
pub fn read_as_text(path: impl AsRef<Path>, max_bytes: usize) -> DocumentResult<(String, bool)> {
    let path = path.as_ref();
    let meta = fs::metadata(path)?;
    
    let truncated = meta.len() > max_bytes as u64;
    
    if truncated {
        let mut file = fs::File::open(path)?;
        let mut buffer = vec![0u8; max_bytes];
        let n = file.read(&mut buffer)?;
        buffer.truncate(n);
        let text = String::from_utf8_lossy(&buffer).to_string();
        Ok((text, true))
    } else {
        let text = fs::read_to_string(path)?;
        Ok((text, false))
    }
}

/// Read file bytes (with size limit)
pub fn read_bytes(path: impl AsRef<Path>, max_bytes: usize) -> DocumentResult<(Vec<u8>, bool)> {
    let path = path.as_ref();
    let meta = fs::metadata(path)?;
    
    let truncated = meta.len() > max_bytes as u64;
    
    if truncated {
        let mut file = fs::File::open(path)?;
        let mut buffer = vec![0u8; max_bytes];
        let n = file.read(&mut buffer)?;
        buffer.truncate(n);
        Ok((buffer, true))
    } else {
        let data = fs::read(path)?;
        Ok((data, false))
    }
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
    
    let (width, height) = reader.into_dimensions()
        .map_err(|e| DocumentError::Io(std::io::Error::other(e.to_string())))?;
    
    Ok(ImageDimensions { width, height })
}

/// Create thumbnail in temp directory (does NOT modify original)
pub fn create_thumbnail(
    path: impl AsRef<Path>,
    max_size: u32,
) -> DocumentResult<PathBuf> {
    let path = path.as_ref();
    
    // Load image
    let img = image::open(path)
        .map_err(|e| DocumentError::Io(std::io::Error::other(e.to_string())))?;
    
    // Resize maintaining aspect ratio
    let thumbnail = img.thumbnail(max_size, max_size);
    
    // Save to temp directory
    let temp_dir = std::env::temp_dir().join("core-ffx-thumbnails");
    fs::create_dir_all(&temp_dir)?;
    
    let file_stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("thumb");
    let thumb_path = temp_dir.join(format!("{}_{}.png", file_stem, max_size));
    
    thumbnail.save(&thumb_path)
        .map_err(|e| DocumentError::Io(std::io::Error::other(e.to_string())))?;
    
    Ok(thumb_path)
}

/// Create thumbnail as base64 data URL (in memory, no temp file)
pub fn create_thumbnail_data_url(
    path: impl AsRef<Path>,
    max_size: u32,
) -> DocumentResult<String> {
    let path = path.as_ref();
    
    // Load image
    let img = image::open(path)
        .map_err(|e| DocumentError::Io(std::io::Error::other(e.to_string())))?;
    
    // Resize maintaining aspect ratio
    let thumbnail = img.thumbnail(max_size, max_size);
    
    // Encode to PNG in memory
    let mut buffer = Vec::new();
    thumbnail.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
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
        ViewerType::Image | ViewerType::Svg | ViewerType::Pdf | 
        ViewerType::Text | ViewerType::Html
    );
    
    let can_search = matches!(
        info.viewer_type,
        ViewerType::Text | ViewerType::Html | ViewerType::Pdf
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

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(UniversalFormat::from_extension("png"), Some(UniversalFormat::Png));
        assert_eq!(UniversalFormat::from_path("test.PDF"), Some(UniversalFormat::Pdf)); // from_path handles case
        assert_eq!(UniversalFormat::from_extension("docx"), Some(UniversalFormat::Docx));
        assert_eq!(UniversalFormat::from_extension("unknown"), Some(UniversalFormat::Binary));
    }

    #[test]
    fn test_viewer_type() {
        assert_eq!(UniversalFormat::Png.viewer_type(), ViewerType::Image);
        assert_eq!(UniversalFormat::Pdf.viewer_type(), ViewerType::Pdf);
        assert_eq!(UniversalFormat::Docx.viewer_type(), ViewerType::Office);
        assert_eq!(UniversalFormat::Zip.viewer_type(), ViewerType::Archive);
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(UniversalFormat::Png.mime_type(), "image/png");
        assert_eq!(UniversalFormat::Pdf.mime_type(), "application/pdf");
        assert_eq!(UniversalFormat::Json.mime_type(), "application/json");
    }
}
