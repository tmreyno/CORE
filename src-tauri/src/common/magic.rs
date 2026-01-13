// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! File type detection via magic signatures
//!
//! Detects file types from header bytes without relying on extensions.
//! Essential for forensic analysis where file extensions may be incorrect or missing.

use serde::Serialize;

// =============================================================================
// File Type Structures
// =============================================================================

/// Detected file type information
#[derive(Debug, Clone, Serialize)]
pub struct FileType {
    /// MIME type (e.g., "application/pdf")
    pub mime: String,
    /// Human-readable description
    pub description: String,
    /// Common file extension(s)
    pub extensions: Vec<String>,
    /// Category for grouping
    pub category: FileCategory,
    /// Confidence level of detection
    pub confidence: Confidence,
}

/// File type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FileCategory {
    /// Images (JPEG, PNG, GIF, etc.)
    Image,
    /// Documents (PDF, Office, etc.)
    Document,
    /// Archives (ZIP, RAR, 7z, etc.)
    Archive,
    /// Executables (EXE, ELF, Mach-O, etc.)
    Executable,
    /// Audio files
    Audio,
    /// Video files
    Video,
    /// Database files
    Database,
    /// Forensic containers (E01, AD1, etc.)
    Forensic,
    /// System/configuration files
    System,
    /// Text-based files
    Text,
    /// Unknown/unrecognized
    Unknown,
}

/// Detection confidence level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Confidence {
    /// Strong magic signature match
    High,
    /// Partial or common signature
    Medium,
    /// Heuristic/guess based on patterns
    Low,
}

impl FileType {
    fn new(mime: &str, description: &str, extensions: &[&str], category: FileCategory) -> Self {
        Self {
            mime: mime.to_string(),
            description: description.to_string(),
            extensions: extensions.iter().map(|s| s.to_string()).collect(),
            category,
            confidence: Confidence::High,
        }
    }

    fn with_confidence(mut self, confidence: Confidence) -> Self {
        self.confidence = confidence;
        self
    }
}

// =============================================================================
// Magic Detection - Table-Driven Approach
// =============================================================================

/// Magic signature entry for table-driven detection
struct MagicSignature {
    /// Byte pattern to match
    pattern: &'static [u8],
    /// Offset to check pattern at
    offset: usize,
    /// Minimum header size required
    min_size: usize,
    /// MIME type
    mime: &'static str,
    /// Description
    description: &'static str,
    /// File extensions
    extensions: &'static [&'static str],
    /// Category
    category: FileCategory,
}

impl MagicSignature {
    const fn new(
        pattern: &'static [u8],
        offset: usize,
        min_size: usize,
        mime: &'static str,
        description: &'static str,
        extensions: &'static [&'static str],
        category: FileCategory,
    ) -> Self {
        Self { pattern, offset, min_size, mime, description, extensions, category }
    }
    
    fn matches(&self, header: &[u8]) -> bool {
        header.len() >= self.min_size && 
        header.len() > self.offset + self.pattern.len() - 1 &&
        header[self.offset..self.offset + self.pattern.len()] == *self.pattern
    }
    
    fn to_file_type(&self) -> FileType {
        FileType::new(self.mime, self.description, self.extensions, self.category)
    }
}

/// Static table of magic signatures for fast lookup
static MAGIC_SIGNATURES: &[MagicSignature] = &[
    // === Images ===
    MagicSignature::new(&[0xFF, 0xD8, 0xFF], 0, 3, "image/jpeg", "JPEG Image", &["jpg", "jpeg"], FileCategory::Image),
    MagicSignature::new(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A], 0, 8, "image/png", "PNG Image", &["png"], FileCategory::Image),
    MagicSignature::new(b"GIF87a", 0, 6, "image/gif", "GIF Image", &["gif"], FileCategory::Image),
    MagicSignature::new(b"GIF89a", 0, 6, "image/gif", "GIF Image", &["gif"], FileCategory::Image),
    MagicSignature::new(b"BM", 0, 2, "image/bmp", "BMP Image", &["bmp"], FileCategory::Image),
    MagicSignature::new(&[0x49, 0x49, 0x2A, 0x00], 0, 4, "image/tiff", "TIFF Image", &["tif", "tiff"], FileCategory::Image),
    MagicSignature::new(&[0x4D, 0x4D, 0x00, 0x2A], 0, 4, "image/tiff", "TIFF Image", &["tif", "tiff"], FileCategory::Image),
    MagicSignature::new(&[0x00, 0x00, 0x01, 0x00], 0, 4, "image/x-icon", "ICO Icon", &["ico"], FileCategory::Image),
    
    // === Documents ===
    MagicSignature::new(b"%PDF", 0, 4, "application/pdf", "PDF Document", &["pdf"], FileCategory::Document),
    MagicSignature::new(b"{\\rtf", 0, 5, "application/rtf", "RTF Document", &["rtf"], FileCategory::Document),
    MagicSignature::new(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1], 0, 8, "application/x-ole-storage", "Microsoft Office Document (OLE)", &["doc", "xls", "ppt", "msg"], FileCategory::Document),
    
    // === Archives ===
    MagicSignature::new(&[0x50, 0x4B, 0x03, 0x04], 0, 4, "application/zip", "ZIP Archive", &["zip", "docx", "xlsx", "pptx", "jar", "apk"], FileCategory::Archive),
    MagicSignature::new(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C], 0, 6, "application/x-7z-compressed", "7-Zip Archive", &["7z"], FileCategory::Archive),
    MagicSignature::new(&[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00], 0, 8, "application/x-rar-compressed", "RAR Archive (v5)", &["rar"], FileCategory::Archive),
    MagicSignature::new(&[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00], 0, 7, "application/x-rar-compressed", "RAR Archive (v4)", &["rar"], FileCategory::Archive),
    MagicSignature::new(&[0x1F, 0x8B], 0, 2, "application/gzip", "GZIP Compressed", &["gz", "tgz"], FileCategory::Archive),
    MagicSignature::new(&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00], 0, 6, "application/x-xz", "XZ Compressed", &["xz", "txz"], FileCategory::Archive),
    MagicSignature::new(&[0x28, 0xB5, 0x2F, 0xFD], 0, 4, "application/zstd", "Zstandard Compressed", &["zst", "zstd"], FileCategory::Archive),
    MagicSignature::new(&[0x04, 0x22, 0x4D, 0x18], 0, 4, "application/x-lz4", "LZ4 Compressed", &["lz4"], FileCategory::Archive),
    
    // === Executables ===
    MagicSignature::new(b"MZ", 0, 2, "application/x-dosexec", "Windows Executable", &["exe", "dll", "sys"], FileCategory::Executable),
    MagicSignature::new(&[0x7F, 0x45, 0x4C, 0x46], 0, 4, "application/x-executable", "ELF Executable", &["elf", "so", "o"], FileCategory::Executable),
    
    // === Audio ===
    MagicSignature::new(b"ID3", 0, 3, "audio/mpeg", "MP3 Audio", &["mp3"], FileCategory::Audio),
    MagicSignature::new(b"fLaC", 0, 4, "audio/flac", "FLAC Audio", &["flac"], FileCategory::Audio),
    MagicSignature::new(b"OggS", 0, 4, "audio/ogg", "OGG Audio/Video", &["ogg", "ogv", "oga"], FileCategory::Audio),
    
    // === Video ===
    MagicSignature::new(&[0x1A, 0x45, 0xDF, 0xA3], 0, 4, "video/x-matroska", "Matroska Video", &["mkv", "webm"], FileCategory::Video),
    MagicSignature::new(b"FLV", 0, 3, "video/x-flv", "Flash Video", &["flv"], FileCategory::Video),
    
    // === Forensic Containers ===
    MagicSignature::new(&[0x45, 0x56, 0x46, 0x09, 0x0D, 0x0A, 0xFF, 0x00], 0, 8, "application/x-ewf", "EnCase Evidence File", &["e01"], FileCategory::Forensic),
    MagicSignature::new(b"ADSEGMENTEDFILE\x00", 0, 16, "application/x-ad1", "AccessData AD1 Image", &["ad1"], FileCategory::Forensic),
    MagicSignature::new(&[0x4C, 0x56, 0x46, 0x09, 0x0D, 0x0A, 0xFF, 0x00], 0, 8, "application/x-l01", "EnCase Logical Evidence", &["l01"], FileCategory::Forensic),
    MagicSignature::new(b"AFF", 0, 3, "application/x-aff", "Advanced Forensic Format", &["aff"], FileCategory::Forensic),
    MagicSignature::new(b"KDMV", 0, 4, "application/x-vmdk", "VMware Virtual Disk", &["vmdk"], FileCategory::Forensic),
    MagicSignature::new(b"# Disk DescriptorFile", 0, 21, "application/x-vmdk", "VMware Virtual Disk (Descriptor)", &["vmdk"], FileCategory::Forensic),
    MagicSignature::new(b"vhdxfile", 0, 8, "application/x-vhdx", "Microsoft VHDx", &["vhdx"], FileCategory::Forensic),
    MagicSignature::new(&[0x51, 0x46, 0x49, 0xFB], 0, 4, "application/x-qcow2", "QEMU Copy-On-Write v2", &["qcow2", "qcow"], FileCategory::Forensic),
    MagicSignature::new(b"cxsparse", 0, 8, "application/x-vhd", "Microsoft VHD (Dynamic)", &["vhd"], FileCategory::Forensic),
    MagicSignature::new(b"EVF2", 0, 4, "application/x-ewf2", "EnCase Evidence File v2", &["ex01"], FileCategory::Forensic),
    MagicSignature::new(b"LVF2", 0, 4, "application/x-ewf2", "EnCase Logical Evidence v2", &["lx01"], FileCategory::Forensic),
    
    // === Database ===
    MagicSignature::new(b"SQLite format 3\x00", 0, 16, "application/x-sqlite3", "SQLite Database", &["db", "sqlite", "sqlite3"], FileCategory::Database),
    
    // === System Files ===
    MagicSignature::new(b"regf", 0, 4, "application/x-ms-registry", "Windows Registry Hive", &["dat"], FileCategory::System),
    MagicSignature::new(&[0x4D, 0x41, 0x4D, 0x04], 0, 4, "application/x-prefetch", "Windows Prefetch (Compressed)", &["pf"], FileCategory::System),
    
    // === Text ===
    MagicSignature::new(b"<?xml", 0, 5, "application/xml", "XML Document", &["xml"], FileCategory::Text),
];

/// Detect file type from header bytes
///
/// Requires at least 32 bytes for reliable detection, but can work with less.
/// Returns None if file type cannot be determined.
pub fn detect_file_type(header: &[u8]) -> Option<FileType> {
    if header.is_empty() {
        return None;
    }

    // First pass: check static signature table
    for sig in MAGIC_SIGNATURES {
        if sig.matches(header) {
            return Some(sig.to_file_type());
        }
    }
    
    // Second pass: handle special cases requiring additional logic
    
    // BZIP2: BZh (third byte is compression level 1-9)
    if header.len() >= 3 && header[..2] == [0x42, 0x5A] && header[2] == 0x68 {
        return Some(FileType::new(
            "application/x-bzip2", "BZIP2 Compressed", &["bz2", "tbz2"], FileCategory::Archive
        ));
    }
    
    // RIFF-based formats (WAV, AVI, WebP)
    if let Some(ft) = detect_riff_type(header) {
        return Some(ft);
    }
    
    // ftyp-based formats (MP4, MOV, HEIC)
    if let Some(ft) = detect_ftyp_type(header) {
        return Some(ft);
    }
    
    // Mach-O executables (multiple magic numbers)
    if let Some(ft) = detect_macho_type(header) {
        return Some(ft);
    }
    
    // MP3 without ID3 tag (sync word detection)
    if header.len() >= 2 && header[0] == 0xFF && (header[1] & 0xE0) == 0xE0 {
        return Some(FileType::new(
            "audio/mpeg", "MP3 Audio", &["mp3"], FileCategory::Audio
        ).with_confidence(Confidence::Medium));
    }
    
    // Windows Prefetch (SCCA at offset 4)
    if header.len() >= 8 && header[4..8] == *b"SCCA" {
        return Some(FileType::new(
            "application/x-prefetch", "Windows Prefetch", &["pf"], FileCategory::System
        ));
    }
    
    // VDI check (text-based header)
    if let Some(ft) = detect_vdi_type(header) {
        return Some(ft);
    }
    
    // HTML detection (heuristic)
    if let Some(ft) = detect_html_type(header) {
        return Some(ft);
    }
    
    // JSON detection (low confidence heuristic)
    if !header.is_empty() && (header[0] == b'{' || header[0] == b'[') {
        return Some(FileType::new(
            "application/json", "JSON Data", &["json"], FileCategory::Text
        ).with_confidence(Confidence::Low));
    }

    None
}

/// Detect RIFF-based formats (WAV, AVI, WebP)
fn detect_riff_type(header: &[u8]) -> Option<FileType> {
    if header.len() < 12 || header[..4] != *b"RIFF" {
        return None;
    }
    
    match &header[8..12] {
        b"WEBP" => Some(FileType::new("image/webp", "WebP Image", &["webp"], FileCategory::Image)),
        b"WAVE" => Some(FileType::new("audio/wav", "WAV Audio", &["wav"], FileCategory::Audio)),
        b"AVI " => Some(FileType::new("video/avi", "AVI Video", &["avi"], FileCategory::Video)),
        _ => None,
    }
}

/// Detect ftyp-based formats (MP4, MOV, HEIC)
fn detect_ftyp_type(header: &[u8]) -> Option<FileType> {
    if header.len() < 12 || header[4..8] != *b"ftyp" {
        return None;
    }
    
    let brand = &header[8..12];
    match brand {
        b"heic" | b"heix" | b"hevc" | b"mif1" => {
            Some(FileType::new("image/heic", "HEIC Image", &["heic", "heif"], FileCategory::Image))
        }
        b"isom" | b"iso2" | b"mp41" | b"mp42" => {
            Some(FileType::new("video/mp4", "MP4 Video", &["mp4", "m4v"], FileCategory::Video))
        }
        b"qt  " => {
            Some(FileType::new("video/quicktime", "QuickTime Video", &["mov"], FileCategory::Video))
        }
        _ => None,
    }
}

/// Detect Mach-O executable formats
fn detect_macho_type(header: &[u8]) -> Option<FileType> {
    if header.len() < 4 {
        return None;
    }
    
    let magic = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
    if magic == 0xFEEDFACE || magic == 0xFEEDFACF || magic == 0xCAFEBABE {
        return Some(FileType::new(
            "application/x-mach-binary", "Mach-O Executable", &["app", "dylib"], 
            FileCategory::Executable
        ));
    }
    None
}

/// Detect VirtualBox VDI format
fn detect_vdi_type(header: &[u8]) -> Option<FileType> {
    if header.len() < 64 {
        return None;
    }
    
    let header_str = String::from_utf8_lossy(&header[..64]);
    if header_str.contains("<<< Oracle VM VirtualBox") || header_str.contains("<<< Sun VirtualBox") {
        return Some(FileType::new(
            "application/x-vdi", "VirtualBox VDI", &["vdi"], FileCategory::Forensic
        ));
    }
    None
}

/// Detect HTML documents (heuristic)
fn detect_html_type(header: &[u8]) -> Option<FileType> {
    if header.len() < 15 {
        return None;
    }
    
    let start = String::from_utf8_lossy(&header[..header.len().min(100)]).to_lowercase();
    if start.contains("<!doctype html") || start.starts_with("<html") {
        return Some(FileType::new(
            "text/html", "HTML Document", &["html", "htm"], FileCategory::Text
        ).with_confidence(Confidence::Medium));
    }
    None
}

/// Quick check if data looks like a specific type
pub fn is_type(header: &[u8], category: FileCategory) -> bool {
    detect_file_type(header)
        .map(|ft| ft.category == category)
        .unwrap_or(false)
}

/// Check if header indicates an image file
pub fn is_image(header: &[u8]) -> bool {
    is_type(header, FileCategory::Image)
}

/// Check if header indicates an archive
pub fn is_archive(header: &[u8]) -> bool {
    is_type(header, FileCategory::Archive)
}

/// Check if header indicates an executable
pub fn is_executable(header: &[u8]) -> bool {
    is_type(header, FileCategory::Executable)
}

/// Check if header indicates a forensic container
pub fn is_forensic_container(header: &[u8]) -> bool {
    is_type(header, FileCategory::Forensic)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_jpeg() {
        let header = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        let ft = detect_file_type(&header).unwrap();
        assert_eq!(ft.mime, "image/jpeg");
        assert_eq!(ft.category, FileCategory::Image);
    }

    #[test]
    fn test_detect_png() {
        let header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let ft = detect_file_type(&header).unwrap();
        assert_eq!(ft.mime, "image/png");
    }

    #[test]
    fn test_detect_pdf() {
        let header = b"%PDF-1.4";
        let ft = detect_file_type(header).unwrap();
        assert_eq!(ft.mime, "application/pdf");
        assert_eq!(ft.category, FileCategory::Document);
    }

    #[test]
    fn test_detect_zip() {
        let header = [0x50, 0x4B, 0x03, 0x04, 0x00, 0x00];
        let ft = detect_file_type(&header).unwrap();
        assert_eq!(ft.mime, "application/zip");
        assert_eq!(ft.category, FileCategory::Archive);
    }

    #[test]
    fn test_detect_exe() {
        let header = b"MZ\x90\x00\x03\x00";
        let ft = detect_file_type(header).unwrap();
        assert_eq!(ft.category, FileCategory::Executable);
    }

    #[test]
    fn test_detect_sqlite() {
        let header = b"SQLite format 3\x00";
        let ft = detect_file_type(header).unwrap();
        assert_eq!(ft.mime, "application/x-sqlite3");
        assert_eq!(ft.category, FileCategory::Database);
    }

    #[test]
    fn test_detect_e01() {
        let header = [0x45, 0x56, 0x46, 0x09, 0x0D, 0x0A, 0xFF, 0x00];
        let ft = detect_file_type(&header).unwrap();
        assert_eq!(ft.category, FileCategory::Forensic);
    }

    #[test]
    fn test_unknown() {
        let header = [0x00, 0x00, 0x00, 0x00];
        assert!(detect_file_type(&header).is_none());
    }

    #[test]
    fn test_is_helpers() {
        let jpeg = [0xFF, 0xD8, 0xFF, 0xE0];
        assert!(is_image(&jpeg));
        assert!(!is_archive(&jpeg));
        
        let zip = [0x50, 0x4B, 0x03, 0x04];
        assert!(is_archive(&zip));
        assert!(!is_image(&zip));
    }
    
    // ==========================================================================
    // Additional forensic format tests
    // ==========================================================================
    
    #[test]
    fn test_detect_ad1() {
        let header = b"ADSEGMENTEDFILE\x00";
        let ft = detect_file_type(header).unwrap();
        assert_eq!(ft.mime, "application/x-ad1");
        assert_eq!(ft.category, FileCategory::Forensic);
        assert!(ft.description.contains("AD1"));
    }
    
    #[test]
    fn test_detect_l01() {
        let header = [0x4C, 0x56, 0x46, 0x09, 0x0D, 0x0A, 0xFF, 0x00];
        let ft = detect_file_type(&header).unwrap();
        assert_eq!(ft.mime, "application/x-l01");
        assert_eq!(ft.category, FileCategory::Forensic);
    }
    
    #[test]
    fn test_detect_7z() {
        let header = [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];
        let ft = detect_file_type(&header).unwrap();
        assert_eq!(ft.mime, "application/x-7z-compressed");
        assert_eq!(ft.category, FileCategory::Archive);
    }
    
    #[test]
    fn test_detect_vmdk() {
        let header = b"KDMV\x00\x00\x00\x01";
        let ft = detect_file_type(header).unwrap();
        assert_eq!(ft.mime, "application/x-vmdk");
        assert_eq!(ft.category, FileCategory::Forensic);
    }
    
    #[test]
    fn test_detect_vhdx() {
        let header = b"vhdxfile";
        let ft = detect_file_type(header).unwrap();
        assert_eq!(ft.mime, "application/x-vhdx");
        assert_eq!(ft.category, FileCategory::Forensic);
    }
    
    #[test]
    fn test_detect_qcow2() {
        let header = [0x51, 0x46, 0x49, 0xFB, 0x00, 0x00, 0x00, 0x03];
        let ft = detect_file_type(&header).unwrap();
        assert_eq!(ft.mime, "application/x-qcow2");
        assert_eq!(ft.category, FileCategory::Forensic);
    }
    
    #[test]
    fn test_detect_aff() {
        let header = b"AFF\x00";
        let ft = detect_file_type(header).unwrap();
        assert_eq!(ft.mime, "application/x-aff");
        assert_eq!(ft.category, FileCategory::Forensic);
    }
    
    #[test]
    fn test_detect_rar5() {
        let header = [0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00];
        let ft = detect_file_type(&header).unwrap();
        assert_eq!(ft.mime, "application/x-rar-compressed");
        assert!(ft.description.contains("RAR") && ft.description.contains("v5"));
    }
    
    #[test]
    fn test_detect_rar4() {
        let header = [0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00];
        let ft = detect_file_type(&header).unwrap();
        assert_eq!(ft.mime, "application/x-rar-compressed");
        assert!(ft.description.contains("RAR") && ft.description.contains("v4"));
    }
    
    #[test]
    fn test_detect_gzip() {
        let header = [0x1F, 0x8B, 0x08, 0x00];
        let ft = detect_file_type(&header).unwrap();
        assert_eq!(ft.mime, "application/gzip");
        assert_eq!(ft.category, FileCategory::Archive);
    }
    
    #[test]
    fn test_detect_registry() {
        let header = b"regf\x00\x00\x00\x00";
        let ft = detect_file_type(header).unwrap();
        assert_eq!(ft.mime, "application/x-ms-registry");
        assert_eq!(ft.category, FileCategory::System);
    }
    
    #[test]
    fn test_detect_dynamic_vhd() {
        let header = b"cxsparse";
        let ft = detect_file_type(header).unwrap();
        assert_eq!(ft.mime, "application/x-vhd");
        assert_eq!(ft.category, FileCategory::Forensic);
    }
    
    #[test]
    fn test_forensic_container_helper() {
        let e01 = [0x45, 0x56, 0x46, 0x09, 0x0D, 0x0A, 0xFF, 0x00];
        assert!(is_forensic_container(&e01));
        
        let ad1 = b"ADSEGMENTEDFILE\x00";
        assert!(is_forensic_container(ad1));
        
        let vmdk = b"KDMV\x00\x00\x00\x01";
        assert!(is_forensic_container(vmdk));
        
        let zip = [0x50, 0x4B, 0x03, 0x04];
        assert!(!is_forensic_container(&zip));
    }
}
