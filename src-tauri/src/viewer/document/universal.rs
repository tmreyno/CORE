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
    Avif,
    RawImage,
    
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
    
    // Email Archives (PST/OST)
    Pst,
    
    // Apple/iOS
    Plist,
    Mobileprovision,
    
    // Executables
    Exe,
    Dll,
    So,
    Dylib,
    MachO,
    Sys,
    
    // Database
    Sqlite,
    Db,
    
    // Windows Registry
    RegistryHive,
    
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
            "avif" => Some(Self::Avif),
            // RAW camera formats (limited browser support)
            "raw" | "cr2" | "nef" | "arw" | "dng" | "orf" | "rw2" => Some(Self::RawImage),
            
            // Documents
            "pdf" => Some(Self::Pdf),
            "txt" | "log" | "cfg" | "ini" | "conf" | "env" => Some(Self::Text),
            "html" | "htm" => Some(Self::Html),
            "md" | "markdown" => Some(Self::Markdown),
            "json" => Some(Self::Json),
            "xml" => Some(Self::Xml),
            "csv" | "tsv" => Some(Self::Csv),
            
            // Source code (treat as text)
            "rs" | "py" | "pyw" | "js" | "ts" | "jsx" | "tsx" | "mjs" | "cjs" |
            "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "hxx" |
            "java" | "go" | "rb" | "rake" | "php" | "phtml" | "swift" | "kt" | "kts" | "scala" |
            "cs" | "csx" | "vb" | "vbs" | "vba" |
            "pl" | "pm" | "lua" | "r" | "awk" | "sed" |
            "sh" | "bash" | "zsh" | "fish" | "ps1" | "psm1" | "bat" | "cmd" |
            "yaml" | "yml" | "toml" | "sql" |
            "css" | "scss" | "sass" | "less" |
            "reg" | "inf" | "properties" => Some(Self::Text),
            
            // Office
            "docx" => Some(Self::Docx),
            "xlsx" | "xlsm" | "xlsb" => Some(Self::Xlsx),
            "pptx" => Some(Self::Pptx),
            "doc" => Some(Self::Doc),
            "xls" | "numbers" => Some(Self::Xls),
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
            "pst" | "ost" => Some(Self::Pst),
            
            // Apple/iOS
            "plist" => Some(Self::Plist),
            "mobileprovision" => Some(Self::Mobileprovision),
            
            // Executables
            "exe" | "com" | "scr" | "ocx" | "cpl" => Some(Self::Exe),
            "dll" => Some(Self::Dll),
            "so" => Some(Self::So),
            "dylib" => Some(Self::Dylib),
            "sys" | "drv" => Some(Self::Sys),
            
            // Binary (known non-viewable formats)
            "bin" | "elf" => Some(Self::Binary),
            
            // Database
            "sqlite" | "sqlite3" | "sqlitedb" => Some(Self::Sqlite),
            "db" | "db3" => Some(Self::Db),
            
            _ => Some(Self::Binary),
        }
    }

    /// Detect format by reading magic bytes from the file header.
    ///
    /// Reads the first 32 bytes and matches against known file signatures.
    /// Returns `None` if the file cannot be read or is empty.
    /// This is read-only — forensic integrity is preserved.
    pub fn detect_by_magic(path: impl AsRef<std::path::Path>) -> Option<Self> {
        use std::io::{Read, Seek, SeekFrom};

        let mut file = std::fs::File::open(path.as_ref()).ok()?;
        let mut buf = [0u8; 32];
        let bytes_read = file.read(&mut buf).ok()?;
        if bytes_read == 0 {
            return None;
        }
        let header = &buf[..bytes_read];

        // --- 16-byte signatures ---
        if bytes_read >= 16 && &header[..16] == b"SQLite format 3\0" {
            return Some(Self::Sqlite);
        }

        // --- 8-byte signatures ---
        if bytes_read >= 8 {
            // MBOX mail format (From + space)
            if &header[..5] == b"From " {
                return Some(Self::Mbox);
            }
            // HEIC/HEIF/AVIF (ISO BMFF — check for "ftyp" at offset 4)
            if &header[4..8] == b"ftyp" {
                // Check the brand at offset 8 for specific format identification
                if bytes_read >= 12 {
                    let brand = &header[8..12];
                    // AVIF brands
                    if brand == b"avif" || brand == b"avis" {
                        return Some(Self::Avif);
                    }
                    // HEIC/HEIF brands
                    if brand == b"heic" || brand == b"heix" || brand == b"hevc"
                        || brand == b"hevx" || brand == b"heim" || brand == b"heis"
                        || brand == b"mif1" || brand == b"msf1"
                    {
                        return Some(Self::Heic);
                    }
                }
            }
        }

        // --- 6-byte signatures ---
        if bytes_read >= 6 {
            // 7z archive
            if header[..6] == [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C] {
                return Some(Self::SevenZ);
            }
            // RAR archive (Rar!\x1a\x07)
            if header[..6] == [0x52, 0x61, 0x72, 0x21, 0x1A, 0x07] {
                return Some(Self::Rar);
            }
            // Apple binary plist
            if &header[..6] == b"bplist" {
                return Some(Self::Plist);
            }
        }

        // --- 4-byte signatures ---
        if bytes_read >= 4 {
            // PST file (!BDN magic)
            if &header[..4] == b"!BDN" {
                return Some(Self::Pst);
            }
            // OLE Compound Document (legacy .doc, .xls, .ppt, .msg)
            if header[..4] == [0xD0, 0xCF, 0x11, 0xE0] {
                // Can't distinguish between doc/xls/ppt/msg from magic alone,
                // so return Doc as default (handled by DocumentFormat which
                // checks the extension or OLE streams). Extension refinement
                // happens in detect_content_format.
                return Some(Self::Doc);
            }
            // ICO file (00 00 01 00 = icon, 00 00 02 00 = cursor)
            if header[..4] == [0x00, 0x00, 0x01, 0x00] || header[..4] == [0x00, 0x00, 0x02, 0x00] {
                return Some(Self::Ico);
            }
            // Windows Registry hive ("regf")
            if &header[..4] == b"regf" {
                return Some(Self::RegistryHive);
            }
            // PDF
            if &header[..4] == b"%PDF" {
                return Some(Self::Pdf);
            }
            // PNG
            if header[..4] == [0x89, 0x50, 0x4E, 0x47] {
                return Some(Self::Png);
            }
            // GIF87a / GIF89a
            if &header[..4] == b"GIF8" {
                return Some(Self::Gif);
            }
            // TIFF (little-endian II or big-endian MM)
            if (header[..4] == [0x49, 0x49, 0x2A, 0x00])
                || (header[..4] == [0x4D, 0x4D, 0x00, 0x2A])
            {
                return Some(Self::Tiff);
            }
            // ZIP (PK\x03\x04) — also covers DOCX, XLSX, PPTX, JAR, etc.
            if header[..4] == [0x50, 0x4B, 0x03, 0x04] {
                return Some(Self::Zip);
            }
            // Mach-O binaries (32-bit, 64-bit, fat/universal)
            let magic32 = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
            match magic32 {
                0xFEED_FACE | 0xFEED_FACF | 0xCAFE_BABE | 0xCEFA_EDFE | 0xCFFA_EDFE => {
                    return Some(Self::MachO);
                }
                _ => {}
            }
            // ELF binary
            if header[..4] == [0x7F, 0x45, 0x4C, 0x46] {
                return Some(Self::Binary);
            }
            // Apple XML plist (<?xm or <pli start)
            if &header[..4] == b"<?xm" {
                // Could be generic XML too — check further if needed
                return Some(Self::Xml);
            }
        }

        // --- 3-byte signatures ---
        if bytes_read >= 3 {
            // JPEG (SOI marker + APP0/APP1)
            if header[..3] == [0xFF, 0xD8, 0xFF] {
                return Some(Self::Jpeg);
            }
        }

        // --- 2-byte signatures ---
        if bytes_read >= 2 {
            // MZ — DOS/PE executable (covers .exe, .dll, .sys, .ocx, .drv)
            if &header[..2] == b"MZ" {
                return Some(Self::Exe);
            }
            // Gzip
            if header[..2] == [0x1F, 0x8B] {
                return Some(Self::Gz);
            }
            // BMP
            if &header[..2] == b"BM" {
                return Some(Self::Bmp);
            }
        }

        // --- RIFF container (needs 12 bytes: "RIFF" + size + type) ---
        if bytes_read >= 12 && &header[..4] == b"RIFF" && &header[8..12] == b"WEBP" {
            return Some(Self::WebP);
        }

        // --- TAR archive ("ustar" at offset 257) ---
        // Requires a separate read since offset 257 is beyond our 32-byte header
        {
            let mut tar_buf = [0u8; 8];
            if file.seek(SeekFrom::Start(257)).is_ok() {
                if let Ok(n) = file.read(&mut tar_buf) {
                    if n >= 5 && &tar_buf[..5] == b"ustar" {
                        return Some(Self::Tar);
                    }
                }
            }
        }

        // --- Text-based heuristics (must come last) ---
        // Check if content looks like UTF-8 text
        if let Ok(text) = std::str::from_utf8(header) {
            let trimmed = text.trim_start();
            // JSON
            // RTF (starts with {\rtf — must check before JSON heuristic)
            if trimmed.starts_with("{\\rtf") {
                return Some(Self::Rtf);
            }
            // JSON
            if trimmed.starts_with('{') || trimmed.starts_with('[') {
                return Some(Self::Json);
            }
            // HTML
            if trimmed.starts_with("<!DOCTYPE") || trimmed.starts_with("<html") || trimmed.starts_with("<HTML") {
                return Some(Self::Html);
            }
            // XML (including plist XML)
            if trimmed.starts_with("<?xml") || trimmed.starts_with('<') {
                return Some(Self::Xml);
            }
            // EML email (common headers)
            if trimmed.starts_with("From:") || trimmed.starts_with("Received:") || trimmed.starts_with("MIME-Version:") {
                return Some(Self::Eml);
            }
            // CSV heuristic — comma-separated with multiple fields on first line
            if trimmed.lines().next().is_some_and(|line| {
                let commas = line.chars().filter(|&c| c == ',').count();
                commas >= 2 && line.len() > 5
            }) {
                return Some(Self::Csv);
            }
            // Generic text (all bytes look like printable ASCII / UTF-8)
            return Some(Self::Text);
        }

        // Binary fallback
        Some(Self::Binary)
    }

    /// Get recommended viewer type
    pub fn viewer_type(&self) -> ViewerType {
        match self {
            // Frontend renders these directly
            Self::Png | Self::Jpeg | Self::Gif | Self::WebP | 
            Self::Bmp | Self::Ico | Self::Heic | Self::Avif | Self::RawImage => ViewerType::Image,
            Self::Svg => ViewerType::Svg,
            Self::Pdf => ViewerType::Pdf,
            Self::Text | Self::Json | Self::Xml |
            Self::Markdown => ViewerType::Text,
            Self::Html => ViewerType::Html,
            Self::Tiff => ViewerType::Image, // Most browsers support TIFF now
            
            // Spreadsheet viewer (tabular data)
            Self::Csv | Self::Xlsx | Self::Xls | Self::Ods => ViewerType::Spreadsheet,
            
            // Backend provides metadata only
            Self::Docx | Self::Pptx | Self::Doc | 
            Self::Ppt | Self::Odt |
            Self::Odp | Self::Rtf => ViewerType::Office,
            
            // Archive listing
            Self::Zip | Self::SevenZ | Self::Rar | 
            Self::Tar | Self::Gz => ViewerType::Archive,
            
            // Email viewer
            Self::Eml | Self::Msg | Self::Mbox => ViewerType::Email,
            
            // PST/OST email archive viewer
            Self::Pst => ViewerType::Pst,
            
            // Plist viewer (structured data)
            Self::Plist | Self::Mobileprovision => ViewerType::Plist,
            
            // Binary analysis
            Self::Exe | Self::Dll | Self::So | Self::Dylib | Self::MachO |
            Self::Sys => ViewerType::Binary,
            
            // Database viewer
            Self::Sqlite | Self::Db => ViewerType::Database,
            
            // Registry hive viewer
            Self::RegistryHive => ViewerType::Registry,
            
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
            Self::Avif => "image/avif",
            Self::RawImage => "image/x-raw",
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
            Self::Pst => "application/vnd.ms-outlook-pst",
            Self::Plist => "application/x-plist",
            Self::Mobileprovision => "application/x-apple-aspen-config",
            Self::Exe => "application/vnd.microsoft.portable-executable",
            Self::Dll => "application/vnd.microsoft.portable-executable",
            Self::So => "application/x-sharedlib",
            Self::Dylib => "application/x-mach-dylib",
            Self::MachO => "application/x-mach-binary",
            Self::Sys => "application/x-windows-driver",
            Self::Sqlite => "application/x-sqlite3",
            Self::Db => "application/x-sqlite3",
            Self::RegistryHive => "application/x-windows-registry",
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
            Self::Avif => "AVIF Image",
            Self::RawImage => "RAW Camera Image",
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
            Self::Pst => "Outlook PST Archive",
            Self::Plist => "Property List",
            Self::Mobileprovision => "iOS Provisioning Profile",
            Self::Exe => "Windows Executable",
            Self::Dll => "Windows DLL",
            Self::So => "Shared Library",
            Self::Dylib => "macOS Dynamic Library",
            Self::MachO => "macOS Executable",
            Self::Sys => "Windows Driver",
            Self::Sqlite => "SQLite Database",
            Self::Db => "Database File",
            Self::RegistryHive => "Windows Registry Hive",
            Self::Binary => "Binary File",
        }
    }
    
    /// Get all supported extensions
    pub fn all_extensions() -> &'static [&'static str] {
        &[
            // Images
            "png", "jpg", "jpeg", "gif", "webp", "bmp", "tiff", "tif", "ico", "svg",
            "heic", "heif", "avif",
            // RAW camera formats
            "raw", "cr2", "nef", "arw", "dng", "orf", "rw2",
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
    /// Use OfficeViewer for text extraction + metadata (DOCX/DOC/PPTX/PPT/ODT/ODP/RTF)
    Office,
    /// Use SpreadsheetViewer for tabular data
    Spreadsheet,
    /// Show archive contents listing
    Archive,
    /// Use HexViewer
    Hex,
    /// Email message viewer
    Email,
    /// PST/OST email archive viewer
    Pst,
    /// Property list viewer (structured data)
    Plist,
    /// Binary/executable analysis viewer
    Binary,
    /// Database viewer
    Database,
    /// Windows Registry hive viewer
    Registry,
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
        ViewerType::Text | ViewerType::Html | ViewerType::Spreadsheet |
        ViewerType::Email | ViewerType::Plist | ViewerType::Database |
        ViewerType::Binary | ViewerType::Registry
    );
    
    let can_search = matches!(
        info.viewer_type,
        ViewerType::Text | ViewerType::Html | ViewerType::Pdf |
        ViewerType::Spreadsheet
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
        // Spreadsheet types route to SpreadsheetViewer
        assert_eq!(UniversalFormat::Csv.viewer_type(), ViewerType::Spreadsheet);
        assert_eq!(UniversalFormat::Xlsx.viewer_type(), ViewerType::Spreadsheet);
        assert_eq!(UniversalFormat::Xls.viewer_type(), ViewerType::Spreadsheet);
        assert_eq!(UniversalFormat::Ods.viewer_type(), ViewerType::Spreadsheet);
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(UniversalFormat::Png.mime_type(), "image/png");
        assert_eq!(UniversalFormat::Pdf.mime_type(), "application/pdf");
        assert_eq!(UniversalFormat::Json.mime_type(), "application/json");
    }

    // =========================================================================
    // Magic-byte detection tests
    // =========================================================================
    
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    /// Helper: write bytes to a temp file and run detect_by_magic
    fn detect_bytes(bytes: &[u8]) -> Option<UniversalFormat> {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(bytes).unwrap();
        f.flush().unwrap();
        UniversalFormat::detect_by_magic(f.path())
    }

    #[test]
    fn test_magic_pdf() {
        assert_eq!(detect_bytes(b"%PDF-1.4 some content"), Some(UniversalFormat::Pdf));
    }

    #[test]
    fn test_magic_png() {
        let mut header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        header.extend_from_slice(&[0u8; 24]);
        assert_eq!(detect_bytes(&header), Some(UniversalFormat::Png));
    }

    #[test]
    fn test_magic_jpeg() {
        assert_eq!(detect_bytes(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10]), Some(UniversalFormat::Jpeg));
    }

    #[test]
    fn test_magic_gif() {
        assert_eq!(detect_bytes(b"GIF89a\x01\x00\x01\x00"), Some(UniversalFormat::Gif));
        assert_eq!(detect_bytes(b"GIF87a\x01\x00\x01\x00"), Some(UniversalFormat::Gif));
    }

    #[test]
    fn test_magic_bmp() {
        assert_eq!(detect_bytes(b"BM\x00\x00\x00\x00\x00\x00"), Some(UniversalFormat::Bmp));
    }

    #[test]
    fn test_magic_webp() {
        assert_eq!(
            detect_bytes(b"RIFF\x00\x00\x00\x00WEBP"),
            Some(UniversalFormat::WebP)
        );
    }

    #[test]
    fn test_magic_tiff_le() {
        assert_eq!(detect_bytes(&[0x49, 0x49, 0x2A, 0x00, 0x08, 0x00]), Some(UniversalFormat::Tiff));
    }

    #[test]
    fn test_magic_tiff_be() {
        assert_eq!(detect_bytes(&[0x4D, 0x4D, 0x00, 0x2A, 0x00, 0x08]), Some(UniversalFormat::Tiff));
    }

    #[test]
    fn test_magic_zip() {
        assert_eq!(detect_bytes(&[0x50, 0x4B, 0x03, 0x04, 0x14, 0x00]), Some(UniversalFormat::Zip));
    }

    #[test]
    fn test_magic_7z() {
        assert_eq!(
            detect_bytes(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C, 0x00, 0x04]),
            Some(UniversalFormat::SevenZ)
        );
    }

    #[test]
    fn test_magic_rar() {
        assert_eq!(
            detect_bytes(&[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00]),
            Some(UniversalFormat::Rar)
        );
    }

    #[test]
    fn test_magic_gz() {
        assert_eq!(detect_bytes(&[0x1F, 0x8B, 0x08, 0x00]), Some(UniversalFormat::Gz));
    }

    #[test]
    fn test_magic_sqlite() {
        assert_eq!(
            detect_bytes(b"SQLite format 3\0extra data here"),
            Some(UniversalFormat::Sqlite)
        );
    }

    #[test]
    fn test_magic_registry_hive() {
        assert_eq!(detect_bytes(b"regf\x00\x00\x00\x00"), Some(UniversalFormat::RegistryHive));
    }

    #[test]
    fn test_magic_plist() {
        assert_eq!(detect_bytes(b"bplist00\x00"), Some(UniversalFormat::Plist));
    }

    #[test]
    fn test_magic_mz_exe() {
        assert_eq!(detect_bytes(b"MZ\x90\x00\x03\x00"), Some(UniversalFormat::Exe));
    }

    #[test]
    fn test_magic_elf() {
        assert_eq!(
            detect_bytes(&[0x7F, 0x45, 0x4C, 0x46, 0x02, 0x01]),
            Some(UniversalFormat::Binary)
        );
    }

    #[test]
    fn test_magic_macho_64() {
        // Mach-O 64-bit: 0xFEEDFACF
        assert_eq!(
            detect_bytes(&[0xFE, 0xED, 0xFA, 0xCF, 0x00, 0x00]),
            Some(UniversalFormat::MachO)
        );
    }

    #[test]
    fn test_magic_macho_fat() {
        // Fat/Universal: 0xCAFEBABE
        assert_eq!(
            detect_bytes(&[0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00]),
            Some(UniversalFormat::MachO)
        );
    }

    #[test]
    fn test_magic_json_object() {
        assert_eq!(detect_bytes(b"{\"key\": \"value\"}"), Some(UniversalFormat::Json));
    }

    #[test]
    fn test_magic_json_array() {
        assert_eq!(detect_bytes(b"[1, 2, 3]"), Some(UniversalFormat::Json));
    }

    #[test]
    fn test_magic_json_with_whitespace() {
        assert_eq!(detect_bytes(b"  \n  {\"key\": 1}"), Some(UniversalFormat::Json));
    }

    #[test]
    fn test_magic_html() {
        assert_eq!(detect_bytes(b"<!DOCTYPE html>"), Some(UniversalFormat::Html));
        assert_eq!(detect_bytes(b"<html><body>"), Some(UniversalFormat::Html));
    }

    #[test]
    fn test_magic_xml() {
        assert_eq!(detect_bytes(b"<?xml version=\"1.0\"?>"), Some(UniversalFormat::Xml));
    }

    #[test]
    fn test_magic_eml() {
        assert_eq!(detect_bytes(b"From: user@example.com\r\nTo: other@example.com"), Some(UniversalFormat::Eml));
        assert_eq!(detect_bytes(b"Received: from mail.example.com"), Some(UniversalFormat::Eml));
        assert_eq!(detect_bytes(b"MIME-Version: 1.0\r\n"), Some(UniversalFormat::Eml));
    }

    #[test]
    fn test_magic_mbox() {
        assert_eq!(detect_bytes(b"From user@example.com Mon Jan 1"), Some(UniversalFormat::Mbox));
    }

    #[test]
    fn test_magic_csv() {
        assert_eq!(detect_bytes(b"name,age,city\nJohn,30,NYC"), Some(UniversalFormat::Csv));
    }

    #[test]
    fn test_magic_plain_text() {
        assert_eq!(detect_bytes(b"Hello World"), Some(UniversalFormat::Text));
    }

    #[test]
    fn test_magic_binary_fallback() {
        // Non-text, non-recognized binary data
        assert_eq!(detect_bytes(&[0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE]), Some(UniversalFormat::Binary));
    }

    #[test]
    fn test_magic_empty_file() {
        let f = NamedTempFile::new().unwrap();
        assert_eq!(UniversalFormat::detect_by_magic(f.path()), None);
    }

    #[test]
    fn test_magic_nonexistent_file() {
        assert_eq!(UniversalFormat::detect_by_magic("/nonexistent/path/file.dat"), None);
    }

    #[test]
    fn test_magic_new_variants() {
        // Verify Sys and RegistryHive enum properties
        assert_eq!(UniversalFormat::Sys.viewer_type(), ViewerType::Binary);
        assert_eq!(UniversalFormat::RegistryHive.viewer_type(), ViewerType::Registry);
        assert_eq!(UniversalFormat::Sys.mime_type(), "application/x-windows-driver");
        assert_eq!(UniversalFormat::RegistryHive.mime_type(), "application/x-windows-registry");
        assert_eq!(UniversalFormat::Sys.description(), "Windows Driver");
        assert_eq!(UniversalFormat::RegistryHive.description(), "Windows Registry Hive");
        assert_eq!(UniversalFormat::from_extension("sys"), Some(UniversalFormat::Sys));
        assert_eq!(UniversalFormat::from_extension("drv"), Some(UniversalFormat::Sys));
    }

    #[test]
    fn test_rtf_magic_detection() {
        // RTF files start with {\rtf — should NOT be detected as JSON
        use std::io::Write;
        let dir = std::env::temp_dir().join("test_rtf_magic");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test.dat"); // no .rtf extension to force magic detection
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(br"{\rtf1\ansi Hello World}").unwrap();
        drop(file);
        
        let detected = UniversalFormat::detect_by_magic(&path);
        assert_eq!(detected, Some(UniversalFormat::Rtf), "RTF should be detected by magic bytes, not as JSON");
        
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_rtf_viewer_type() {
        assert_eq!(UniversalFormat::Rtf.viewer_type(), ViewerType::Office);
    }

    #[test]
    fn test_pptx_viewer_type() {
        assert_eq!(UniversalFormat::Pptx.viewer_type(), ViewerType::Office);
    }

    #[test]
    fn test_odt_from_extension() {
        assert_eq!(UniversalFormat::from_extension("odt"), Some(UniversalFormat::Odt));
    }

    // =========================================================================
    // New extension mapping tests
    // =========================================================================

    #[test]
    fn test_xlsm_xlsb_extension() {
        assert_eq!(UniversalFormat::from_extension("xlsm"), Some(UniversalFormat::Xlsx));
        assert_eq!(UniversalFormat::from_extension("xlsb"), Some(UniversalFormat::Xlsx));
    }

    #[test]
    fn test_numbers_extension() {
        assert_eq!(UniversalFormat::from_extension("numbers"), Some(UniversalFormat::Xls));
    }

    #[test]
    fn test_executable_extensions() {
        assert_eq!(UniversalFormat::from_extension("com"), Some(UniversalFormat::Exe));
        assert_eq!(UniversalFormat::from_extension("scr"), Some(UniversalFormat::Exe));
        assert_eq!(UniversalFormat::from_extension("ocx"), Some(UniversalFormat::Exe));
        assert_eq!(UniversalFormat::from_extension("cpl"), Some(UniversalFormat::Exe));
    }

    #[test]
    fn test_binary_extensions() {
        assert_eq!(UniversalFormat::from_extension("bin"), Some(UniversalFormat::Binary));
        assert_eq!(UniversalFormat::from_extension("elf"), Some(UniversalFormat::Binary));
    }

    // =========================================================================
    // New magic-byte detection tests
    // =========================================================================

    #[test]
    fn test_magic_ole_compound() {
        // OLE Compound Document (D0 CF 11 E0 A1 B1 1A E1)
        assert_eq!(
            detect_bytes(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]),
            Some(UniversalFormat::Doc)
        );
    }

    #[test]
    fn test_magic_ico() {
        // ICO file (00 00 01 00)
        assert_eq!(
            detect_bytes(&[0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x20, 0x20]),
            Some(UniversalFormat::Ico)
        );
    }

    #[test]
    fn test_magic_cursor() {
        // CUR file (00 00 02 00) — also detected as Ico
        assert_eq!(
            detect_bytes(&[0x00, 0x00, 0x02, 0x00, 0x01, 0x00, 0x20, 0x20]),
            Some(UniversalFormat::Ico)
        );
    }

    #[test]
    fn test_magic_heic() {
        // HEIC: ftypheic at offset 4
        let mut bytes = vec![0x00, 0x00, 0x00, 0x18]; // box size
        bytes.extend_from_slice(b"ftypheic");           // ftyp + brand
        bytes.extend_from_slice(&[0x00; 20]);           // padding
        assert_eq!(detect_bytes(&bytes), Some(UniversalFormat::Heic));
    }

    #[test]
    fn test_magic_heif_mif1() {
        // HEIF with mif1 brand
        let mut bytes = vec![0x00, 0x00, 0x00, 0x1C];
        bytes.extend_from_slice(b"ftypmif1");
        bytes.extend_from_slice(&[0x00; 20]);
        assert_eq!(detect_bytes(&bytes), Some(UniversalFormat::Heic));
    }

    #[test]
    fn test_magic_tar() {
        // TAR: "ustar" at offset 257
        let mut bytes = vec![0u8; 263]; // at least 262 bytes
        bytes[257] = b'u';
        bytes[258] = b's';
        bytes[259] = b't';
        bytes[260] = b'a';
        bytes[261] = b'r';
        assert_eq!(detect_bytes(&bytes), Some(UniversalFormat::Tar));
    }

    // =========================================================================
    // AVIF and RAW image tests
    // =========================================================================

    #[test]
    fn test_avif_from_extension() {
        assert_eq!(UniversalFormat::from_extension("avif"), Some(UniversalFormat::Avif));
    }

    #[test]
    fn test_avif_viewer_type() {
        assert_eq!(UniversalFormat::Avif.viewer_type(), ViewerType::Image);
    }

    #[test]
    fn test_avif_mime_type() {
        assert_eq!(UniversalFormat::Avif.mime_type(), "image/avif");
    }

    #[test]
    fn test_avif_description() {
        assert_eq!(UniversalFormat::Avif.description(), "AVIF Image");
    }

    #[test]
    fn test_magic_avif() {
        // AVIF: ftypavif at offset 4
        let mut bytes = vec![0x00, 0x00, 0x00, 0x20]; // box size
        bytes.extend_from_slice(b"ftypavif");           // ftyp + brand
        bytes.extend_from_slice(&[0x00; 20]);           // padding
        assert_eq!(detect_bytes(&bytes), Some(UniversalFormat::Avif));
    }

    #[test]
    fn test_magic_avif_sequence() {
        // AVIF sequence: ftypavis at offset 4
        let mut bytes = vec![0x00, 0x00, 0x00, 0x20];
        bytes.extend_from_slice(b"ftypavis");
        bytes.extend_from_slice(&[0x00; 20]);
        assert_eq!(detect_bytes(&bytes), Some(UniversalFormat::Avif));
    }

    #[test]
    fn test_raw_image_extensions() {
        assert_eq!(UniversalFormat::from_extension("raw"), Some(UniversalFormat::RawImage));
        assert_eq!(UniversalFormat::from_extension("cr2"), Some(UniversalFormat::RawImage));
        assert_eq!(UniversalFormat::from_extension("nef"), Some(UniversalFormat::RawImage));
        assert_eq!(UniversalFormat::from_extension("arw"), Some(UniversalFormat::RawImage));
        assert_eq!(UniversalFormat::from_extension("dng"), Some(UniversalFormat::RawImage));
        assert_eq!(UniversalFormat::from_extension("orf"), Some(UniversalFormat::RawImage));
        assert_eq!(UniversalFormat::from_extension("rw2"), Some(UniversalFormat::RawImage));
    }

    #[test]
    fn test_raw_image_viewer_type() {
        assert_eq!(UniversalFormat::RawImage.viewer_type(), ViewerType::Image);
    }

    #[test]
    fn test_raw_image_description() {
        assert_eq!(UniversalFormat::RawImage.description(), "RAW Camera Image");
    }

    #[test]
    fn test_db3_sqlitedb_extensions() {
        assert_eq!(UniversalFormat::from_extension("db3"), Some(UniversalFormat::Db));
        assert_eq!(UniversalFormat::from_extension("sqlitedb"), Some(UniversalFormat::Sqlite));
    }
}
