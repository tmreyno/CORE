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

pub mod file_info;
pub mod viewer_hint;

// Re-export all public items so external paths remain unchanged
pub use file_info::{FileInfo, read_as_data_url, read_as_text, read_bytes};
pub use viewer_hint::{
    ImageDimensions, get_image_dimensions, create_thumbnail, create_thumbnail_data_url,
    ViewerHint, DisplayMode, ViewerConfig, get_viewer_hint,
};

use std::path::Path;
use serde::{Serialize, Deserialize};

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
// TESTS
// =============================================================================

#[cfg(test)]
mod tests;
