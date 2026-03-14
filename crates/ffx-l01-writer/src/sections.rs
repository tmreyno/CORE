// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! EWF v1 section writers for L01 files.
//!
//! Each section in an EWF v1 file has a 76-byte header followed by section-specific
//! data. The header contains the section type, the offset of the next section,
//! and an Adler-32 checksum of the header itself.
//!
//! L01 section order: header → header2 → volume → sectors → table → table2 → ltypes → ltree → data → hash → digest → done

use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{self, Write};

use super::ltree;
use super::types::*;

// ─── Section Header Writer ──────────────────────────────────────────────────

/// Write a 76-byte EWF v1 section header.
///
/// Layout:
/// - [0..16]   Section type (null-padded ASCII)
/// - [16..24]  Next section offset (u64 LE)
/// - [24..32]  Section size including header (u64 LE)
/// - [32..72]  Padding (zeros)
/// - [72..76]  Adler-32 checksum of bytes [0..72]
pub fn write_section_header<W: Write>(
    writer: &mut W,
    section_type: &[u8; SECTION_TYPE_LEN],
    next_offset: u64,
    section_size: u64,
) -> Result<(), L01WriteError> {
    let mut header = [0u8; SECTION_HEADER_SIZE];

    // Section type
    header[0..SECTION_TYPE_LEN].copy_from_slice(section_type);

    // Next section offset
    header[16..24].copy_from_slice(&next_offset.to_le_bytes());

    // Section size (including this 76-byte header)
    header[24..32].copy_from_slice(&section_size.to_le_bytes());

    // Padding [32..72] is already zeros

    // Adler-32 checksum of first 72 bytes
    let checksum = adler32(&header[0..72]);
    header[72..76].copy_from_slice(&checksum.to_le_bytes());

    writer.write_all(&header)?;
    Ok(())
}

// ─── File Header ────────────────────────────────────────────────────────────

/// Write the 13-byte L01 file header (LVF signature + segment info).
///
/// Layout:
/// - [0..8]    LVF signature
/// - [8]       Fields start marker (0x01)
/// - [9..11]   Segment number (u16 LE)
/// - [11..13]  Fields end marker (0x0000)
pub fn write_file_header<W: Write>(
    writer: &mut W,
    segment_number: u16,
) -> Result<(), L01WriteError> {
    let mut header = [0u8; FILE_HEADER_SIZE];

    // LVF signature
    header[0..8].copy_from_slice(LVF_SIGNATURE);

    // Fields start
    header[8] = 0x01;

    // Segment number
    header[9..11].copy_from_slice(&segment_number.to_le_bytes());

    // Fields end
    header[11..13].copy_from_slice(&0u16.to_le_bytes());

    writer.write_all(&header)?;
    Ok(())
}

// ─── Header Section ─────────────────────────────────────────────────────────

/// Build the header section content string (EWF v1 format).
///
/// The header is a UTF-8 text block with key-value pairs,
/// then zlib-compressed.
///
/// Format:
/// ```text
/// 1
/// main
/// c\tn\ta\te\tt\tav\tov\tm\tu\tp\r\n
/// <case>\t<evidence>\t<description>\t<examiner>\t<notes>\t<sw_version>\t<os>\t<acq_date>\t<system_date>\t<password>\r\n
/// \r\n
/// ```
fn build_header_text(case_info: &L01CaseInfo) -> String {
    let now = chrono::Local::now();
    let acq_date = now.format("%Y %m %d %H %M %S").to_string();
    let system_date = acq_date.clone();

    let mut text = String::new();
    text.push_str("1\r\n");
    text.push_str("main\r\n");
    text.push_str("c\tn\ta\te\tt\tav\tov\tm\tu\tp\r\n");
    text.push_str(&case_info.case_number);
    text.push('\t');
    text.push_str(&case_info.evidence_number);
    text.push('\t');
    text.push_str(&case_info.description);
    text.push('\t');
    text.push_str(&case_info.examiner);
    text.push('\t');
    text.push_str(&case_info.notes);
    text.push('\t');
    text.push_str("CORE-FFX 1.0"); // acquiry software version
    text.push('\t');
    text.push_str(std::env::consts::OS); // acquiry OS
    text.push('\t');
    text.push_str(&acq_date);
    text.push('\t');
    text.push_str(&system_date);
    text.push('\t');
    text.push('\t'); // password (empty)
    text.push_str("\r\n");
    text.push_str("\r\n");

    text
}

/// Write the "header" section (UTF-8, zlib-compressed).
///
/// The header section is written twice in EnCase format — once as "header" and
/// once as "header2". We write both for compatibility.
pub fn write_header_section<W: Write + io::Seek>(
    writer: &mut W,
    case_info: &L01CaseInfo,
    next_offset: u64,
) -> Result<u64, L01WriteError> {
    let text = build_header_text(case_info);
    let text_bytes = text.as_bytes();

    // Compress with zlib
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(text_bytes)
        .map_err(|e| L01WriteError::CompressionError(e.to_string()))?;
    let compressed = encoder
        .finish()
        .map_err(|e| L01WriteError::CompressionError(e.to_string()))?;

    let section_size = (SECTION_HEADER_SIZE + compressed.len()) as u64;
    write_section_header(writer, SECTION_TYPE_HEADER, next_offset, section_size)?;
    writer.write_all(&compressed)?;

    Ok(section_size)
}

/// Write the "header2" section (UTF-16LE, zlib-compressed).
///
/// Same content as header but encoded in UTF-16LE for EnCase compatibility.
pub fn write_header2_section<W: Write + io::Seek>(
    writer: &mut W,
    case_info: &L01CaseInfo,
    next_offset: u64,
) -> Result<u64, L01WriteError> {
    let text = build_header_text(case_info);
    let utf16_bytes = ltree::utf8_to_utf16le(&text);

    // Compress with zlib
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&utf16_bytes)
        .map_err(|e| L01WriteError::CompressionError(e.to_string()))?;
    let compressed = encoder
        .finish()
        .map_err(|e| L01WriteError::CompressionError(e.to_string()))?;

    let section_size = (SECTION_HEADER_SIZE + compressed.len()) as u64;
    write_section_header(writer, SECTION_TYPE_HEADER2, next_offset, section_size)?;
    writer.write_all(&compressed)?;

    Ok(section_size)
}

// ─── Volume Section ─────────────────────────────────────────────────────────

/// Write the "volume" section.
///
/// For L01 files, the volume section describes the logical image:
/// - media_type = 0x0e (logical)
/// - chunk_count = total number of data chunks
/// - bytes_per_sector = block_size (usually 512)
/// - sectors_per_chunk = 64
///
/// The volume data is 94 bytes after the 76-byte section header.
/// Layout (from EWF v1 spec):
/// - [0..4]    Reserved/media_type_header (1)
/// - [4..8]    Chunk count (u32 LE)
/// - [8..12]   Sectors per chunk (u32 LE)
/// - [12..16]  Bytes per sector (u32 LE)
/// - [16..24]  Sector count (u64 LE)
/// - [24..28]  CHS cylinders (u32 LE) - 0 for logical
/// - [28..32]  CHS heads (u32 LE) - 0 for logical
/// - [32..36]  CHS sectors (u32 LE) - 0 for logical
/// - [36]      Media type (0x0e = logical)
/// - [37..56]  Padding
/// - [56]      Compression level
/// - [57..60]  Padding
/// - [60..76]  GUID (16 bytes)
/// - [76..90]  Padding/reserved
/// - [90..94]  Adler-32 checksum of volume data
pub fn write_volume_section<W: Write + io::Seek>(
    writer: &mut W,
    chunk_count: u32,
    block_size: u32,
    sectors_per_chunk: u32,
    compression_level: CompressionLevel,
    next_offset: u64,
) -> Result<u64, L01WriteError> {
    let section_size = (SECTION_HEADER_SIZE + VOLUME_DATA_SIZE) as u64;
    write_section_header(writer, SECTION_TYPE_VOLUME, next_offset, section_size)?;

    let mut data = [0u8; VOLUME_DATA_SIZE];

    // Reserved / header value
    data[0..4].copy_from_slice(&1u32.to_le_bytes());

    // Chunk count
    data[4..8].copy_from_slice(&chunk_count.to_le_bytes());

    // Sectors per chunk
    data[8..12].copy_from_slice(&sectors_per_chunk.to_le_bytes());

    // Bytes per sector (block size)
    data[12..16].copy_from_slice(&block_size.to_le_bytes());

    // Sector count (total logical sectors = chunk_count * sectors_per_chunk)
    let sector_count = (chunk_count as u64) * (sectors_per_chunk as u64);
    data[16..24].copy_from_slice(&sector_count.to_le_bytes());

    // CHS values: all zero for logical images
    // [24..36] already zeros

    // Media type
    data[36] = MEDIA_TYPE_LOGICAL;

    // Compression level
    data[56] = compression_level as u8;

    // GUID (16 bytes at offset 60)
    let guid = uuid::Uuid::new_v4();
    data[60..76].copy_from_slice(guid.as_bytes());

    // Adler-32 checksum of volume data (bytes 0..90)
    let checksum = adler32(&data[0..90]);
    data[90..94].copy_from_slice(&checksum.to_le_bytes());

    writer.write_all(&data)?;

    Ok(section_size)
}

// ─── Sectors Section ────────────────────────────────────────────────────────

/// Write the "sectors" section containing compressed chunk data.
///
/// The section header is followed immediately by the compressed chunk data.
/// Each chunk is either zlib-compressed or stored raw (indicated by the
/// MSB of the offset in the table section).
pub fn write_sectors_section<W: Write + io::Seek>(
    writer: &mut W,
    compressed_data: &[u8],
    next_offset: u64,
) -> Result<u64, L01WriteError> {
    let section_size = (SECTION_HEADER_SIZE + compressed_data.len()) as u64;
    write_section_header(writer, SECTION_TYPE_SECTORS, next_offset, section_size)?;
    writer.write_all(compressed_data)?;

    Ok(section_size)
}

// ─── Table Section ──────────────────────────────────────────────────────────

/// Table entry size: 4 bytes (u32) per chunk offset
const TABLE_ENTRY_SIZE: usize = 4;

/// Table header size: 4 (chunk_count) + 4 (padding) + 8 (base_offset) + 4 (padding) + 4 (Adler-32) = 24 bytes
const TABLE_HEADER_SIZE: usize = 24;

/// Write the "table" section with chunk offset entries.
///
/// Layout after section header:
/// - [0..4]    Chunk count (u32 LE)
/// - [4..8]    Padding (zeros)
/// - [8..16]   Base offset (u64 LE) — offset of sectors section data start
/// - [16..20]  Padding (zeros)
/// - [20..24]  Adler-32 checksum of table header (bytes 0..20)
/// - [24..]    Chunk offset entries (4 bytes each, u32 LE)
///   MSB (bit 31) = 1 if chunk is compressed
///
/// Followed by 4-byte Adler-32 checksum of the entries.
pub fn write_table_section<W: Write + io::Seek>(
    writer: &mut W,
    table: &ChunkTable,
    next_offset: u64,
) -> Result<u64, L01WriteError> {
    let entries_size = table.chunk_count() as usize * TABLE_ENTRY_SIZE;
    let entries_checksum_size = 4;
    let data_size = TABLE_HEADER_SIZE + entries_size + entries_checksum_size;
    let section_size = (SECTION_HEADER_SIZE + data_size) as u64;

    write_section_header(writer, SECTION_TYPE_TABLE, next_offset, section_size)?;

    // Table header
    let mut header = [0u8; TABLE_HEADER_SIZE];
    header[0..4].copy_from_slice(&table.chunk_count().to_le_bytes());
    // [4..8] padding
    header[8..16].copy_from_slice(&table.base_offset.to_le_bytes());
    // [16..20] padding
    let header_checksum = adler32(&header[0..20]);
    header[20..24].copy_from_slice(&header_checksum.to_le_bytes());
    writer.write_all(&header)?;

    // Chunk offset entries
    let mut entries_data = Vec::with_capacity(entries_size);
    for chunk in &table.chunks {
        let mut offset_value = chunk.offset as u32;
        if chunk.is_compressed {
            offset_value |= 0x8000_0000; // Set MSB for compressed chunks
        }
        entries_data.extend_from_slice(&offset_value.to_le_bytes());
    }
    writer.write_all(&entries_data)?;

    // Entries checksum
    let entries_checksum = adler32(&entries_data);
    writer.write_all(&entries_checksum.to_le_bytes())?;

    Ok(section_size)
}

/// Write the "table2" section (identical to "table" in L01).
pub fn write_table2_section<W: Write + io::Seek>(
    writer: &mut W,
    table: &ChunkTable,
    next_offset: u64,
) -> Result<u64, L01WriteError> {
    let entries_size = table.chunk_count() as usize * TABLE_ENTRY_SIZE;
    let entries_checksum_size = 4;
    let data_size = TABLE_HEADER_SIZE + entries_size + entries_checksum_size;
    let section_size = (SECTION_HEADER_SIZE + data_size) as u64;

    write_section_header(writer, SECTION_TYPE_TABLE2, next_offset, section_size)?;

    // Table header (same as table)
    let mut header = [0u8; TABLE_HEADER_SIZE];
    header[0..4].copy_from_slice(&table.chunk_count().to_le_bytes());
    header[8..16].copy_from_slice(&table.base_offset.to_le_bytes());
    let header_checksum = adler32(&header[0..20]);
    header[20..24].copy_from_slice(&header_checksum.to_le_bytes());
    writer.write_all(&header)?;

    // Chunk offset entries (same as table)
    let mut entries_data = Vec::with_capacity(entries_size);
    for chunk in &table.chunks {
        let mut offset_value = chunk.offset as u32;
        if chunk.is_compressed {
            offset_value |= 0x8000_0000;
        }
        entries_data.extend_from_slice(&offset_value.to_le_bytes());
    }
    writer.write_all(&entries_data)?;

    let entries_checksum = adler32(&entries_data);
    writer.write_all(&entries_checksum.to_le_bytes())?;

    Ok(section_size)
}

// ─── Ltypes Section ─────────────────────────────────────────────────────────

/// Write the "ltypes" section (EnCase 7+ logical type metadata).
///
/// This is a minimal section for L01 compatibility. The ltypes section
/// describes the types of entries in the ltree. For basic L01 files,
/// we write a minimal placeholder.
///
/// Format: section header + 4-byte type count + type entries
pub fn write_ltypes_section<W: Write + io::Seek>(
    writer: &mut W,
    next_offset: u64,
) -> Result<u64, L01WriteError> {
    // Minimal ltypes data: just a type count of 0
    let data = [0u8; 4]; // type count = 0
    let section_size = (SECTION_HEADER_SIZE + data.len()) as u64;

    write_section_header(writer, SECTION_TYPE_LTYPES, next_offset, section_size)?;
    writer.write_all(&data)?;

    Ok(section_size)
}

// ─── Ltree Section ──────────────────────────────────────────────────────────

/// Write the "ltree" section containing the file tree metadata.
///
/// The ltree section has a special header followed by the UTF-16LE tree data:
///
/// Section header (76 bytes) + ltree header (48 bytes) + data:
/// - [0..16]   MD5 hash of uncompressed ltree data
/// - [16..24]  Size of uncompressed ltree data (u64 LE)
/// - [24..28]  Adler-32 checksum of ltree header (bytes 0..24)
/// - [28..48]  Reserved (zeros)
/// - [48..]    zlib-compressed UTF-16LE ltree text
pub fn write_ltree_section<W: Write + io::Seek>(
    writer: &mut W,
    ltree_utf16_data: &[u8],
    next_offset: u64,
) -> Result<u64, L01WriteError> {
    // Compress the UTF-16LE data
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(ltree_utf16_data)
        .map_err(|e| L01WriteError::CompressionError(e.to_string()))?;
    let compressed = encoder
        .finish()
        .map_err(|e| L01WriteError::CompressionError(e.to_string()))?;

    // Compute MD5 of uncompressed data
    use md5::Digest;
    let md5_hash = md5::Md5::digest(ltree_utf16_data);

    // Build ltree header (48 bytes)
    let ltree_header_size = 48;
    let mut ltree_header = [0u8; 48];

    // MD5 hash of uncompressed data
    ltree_header[0..16].copy_from_slice(&md5_hash);

    // Uncompressed data size
    let data_size = ltree_utf16_data.len() as u64;
    ltree_header[16..24].copy_from_slice(&data_size.to_le_bytes());

    // Adler-32 of ltree header bytes [0..24]
    let header_checksum = adler32(&ltree_header[0..24]);
    ltree_header[24..28].copy_from_slice(&header_checksum.to_le_bytes());

    // [28..48] reserved, already zeros

    let section_size = (SECTION_HEADER_SIZE + ltree_header_size + compressed.len()) as u64;
    write_section_header(writer, SECTION_TYPE_LTREE, next_offset, section_size)?;
    writer.write_all(&ltree_header)?;
    writer.write_all(&compressed)?;

    Ok(section_size)
}

// ─── Data Section ───────────────────────────────────────────────────────────

/// Write the "data" section (metadata about the logical image).
///
/// For L01, the data section contains minimal metadata:
/// - [0..4]    Media type (u32 LE) — 0x0e for logical
/// - [4..8]    Reserved
/// - [8..16]   Chunk count (u64 LE)
/// - [16..20]  Sectors per chunk (u32 LE)
/// - [20..24]  Bytes per sector (u32 LE)
/// - [24..32]  Sector count (u64 LE)
/// - [32..52]  Padding
/// - [52..68]  GUID (16 bytes)
/// - [68..72]  Adler-32 checksum
const DATA_SECTION_DATA_SIZE: usize = 72;

pub fn write_data_section<W: Write + io::Seek>(
    writer: &mut W,
    chunk_count: u32,
    block_size: u32,
    sectors_per_chunk: u32,
    next_offset: u64,
) -> Result<u64, L01WriteError> {
    let section_size = (SECTION_HEADER_SIZE + DATA_SECTION_DATA_SIZE) as u64;
    write_section_header(writer, SECTION_TYPE_DATA, next_offset, section_size)?;

    let mut data = [0u8; DATA_SECTION_DATA_SIZE];

    // Media type
    data[0..4].copy_from_slice(&(MEDIA_TYPE_LOGICAL as u32).to_le_bytes());

    // Chunk count (as u64 at offset 8)
    data[8..16].copy_from_slice(&(chunk_count as u64).to_le_bytes());

    // Sectors per chunk
    data[16..20].copy_from_slice(&sectors_per_chunk.to_le_bytes());

    // Bytes per sector
    data[20..24].copy_from_slice(&block_size.to_le_bytes());

    // Sector count
    let sector_count = (chunk_count as u64) * (sectors_per_chunk as u64);
    data[24..32].copy_from_slice(&sector_count.to_le_bytes());

    // GUID
    let guid = uuid::Uuid::new_v4();
    data[52..68].copy_from_slice(guid.as_bytes());

    // Adler-32 checksum
    let checksum = adler32(&data[0..68]);
    data[68..72].copy_from_slice(&checksum.to_le_bytes());

    writer.write_all(&data)?;

    Ok(section_size)
}

// ─── Hash Section ───────────────────────────────────────────────────────────

/// Write the "hash" section with the MD5 hash of all sectors data.
///
/// Layout:
/// - [0..16]  MD5 hash
/// - [16..20] Adler-32 checksum of hash data
const HASH_SECTION_DATA_SIZE: usize = 20;

pub fn write_hash_section<W: Write + io::Seek>(
    writer: &mut W,
    md5_hash: &[u8; 16],
    next_offset: u64,
) -> Result<u64, L01WriteError> {
    let section_size = (SECTION_HEADER_SIZE + HASH_SECTION_DATA_SIZE) as u64;
    write_section_header(writer, SECTION_TYPE_HASH, next_offset, section_size)?;

    let mut data = [0u8; HASH_SECTION_DATA_SIZE];
    data[0..16].copy_from_slice(md5_hash);

    let checksum = adler32(&data[0..16]);
    data[16..20].copy_from_slice(&checksum.to_le_bytes());

    writer.write_all(&data)?;

    Ok(section_size)
}

// ─── Digest Section ─────────────────────────────────────────────────────────

/// Write the "digest" section with the SHA-1 hash of all sectors data.
///
/// Layout:
/// - [0..20]  SHA-1 hash
/// - [20..24] Adler-32 checksum
const DIGEST_SECTION_DATA_SIZE: usize = 24;

pub fn write_digest_section<W: Write + io::Seek>(
    writer: &mut W,
    sha1_hash: &[u8; 20],
    next_offset: u64,
) -> Result<u64, L01WriteError> {
    let section_size = (SECTION_HEADER_SIZE + DIGEST_SECTION_DATA_SIZE) as u64;
    write_section_header(writer, SECTION_TYPE_DIGEST, next_offset, section_size)?;

    let mut data = [0u8; DIGEST_SECTION_DATA_SIZE];
    data[0..20].copy_from_slice(sha1_hash);

    let checksum = adler32(&data[0..20]);
    data[20..24].copy_from_slice(&checksum.to_le_bytes());

    writer.write_all(&data)?;

    Ok(section_size)
}

// ─── Done Section ───────────────────────────────────────────────────────────

/// Write the "done" section (final section, no data).
///
/// The done section is just a section header with next_offset = 0
/// (indicating no more sections).
pub fn write_done_section<W: Write + io::Seek>(writer: &mut W) -> Result<u64, L01WriteError> {
    let section_size = SECTION_HEADER_SIZE as u64;
    write_section_header(writer, SECTION_TYPE_DONE, 0, section_size)?;

    Ok(section_size)
}

// ─── Next Section (for multi-segment) ───────────────────────────────────────

/// Write the "next" section indicating continuation in the next segment file.
///
/// The next section header's next_offset is 0 (no more sections in this file).
pub fn write_next_section<W: Write + io::Seek>(writer: &mut W) -> Result<u64, L01WriteError> {
    let section_size = SECTION_HEADER_SIZE as u64;
    write_section_header(writer, SECTION_TYPE_NEXT, 0, section_size)?;

    Ok(section_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write_section_header() {
        let mut buf = Cursor::new(Vec::new());
        write_section_header(&mut buf, SECTION_TYPE_HEADER, 1000, 500).unwrap();

        let data = buf.into_inner();
        assert_eq!(data.len(), SECTION_HEADER_SIZE);

        // Check section type
        assert_eq!(&data[0..6], b"header");

        // Check next offset
        let next_offset = u64::from_le_bytes(data[16..24].try_into().unwrap());
        assert_eq!(next_offset, 1000);

        // Check section size
        let section_size = u64::from_le_bytes(data[24..32].try_into().unwrap());
        assert_eq!(section_size, 500);

        // Check that checksum is non-zero
        let checksum = u32::from_le_bytes(data[72..76].try_into().unwrap());
        assert_ne!(checksum, 0);

        // Verify checksum
        let expected = adler32(&data[0..72]);
        assert_eq!(checksum, expected);
    }

    #[test]
    fn test_write_file_header() {
        let mut buf = Cursor::new(Vec::new());
        write_file_header(&mut buf, 1).unwrap();

        let data = buf.into_inner();
        assert_eq!(data.len(), FILE_HEADER_SIZE);

        // Check LVF signature
        assert_eq!(&data[0..8], LVF_SIGNATURE);

        // Check segment number
        let seg = u16::from_le_bytes(data[9..11].try_into().unwrap());
        assert_eq!(seg, 1);
    }

    #[test]
    fn test_write_volume_section() {
        let mut buf = Cursor::new(Vec::new());
        write_volume_section(
            &mut buf,
            10,  // chunk_count
            512, // block_size
            64,  // sectors_per_chunk
            CompressionLevel::Fast,
            1000, // next_offset
        )
        .unwrap();

        let data = buf.into_inner();
        let expected_size = SECTION_HEADER_SIZE + VOLUME_DATA_SIZE;
        assert_eq!(data.len(), expected_size);

        // Verify section type
        assert_eq!(&data[0..6], b"volume");

        // Verify volume data
        let vol_data = &data[SECTION_HEADER_SIZE..];

        // Chunk count at offset 4
        let chunk_count = u32::from_le_bytes(vol_data[4..8].try_into().unwrap());
        assert_eq!(chunk_count, 10);

        // Media type at offset 36
        assert_eq!(vol_data[36], MEDIA_TYPE_LOGICAL);

        // Compression level at offset 56
        assert_eq!(vol_data[56], CompressionLevel::Fast as u8);
    }

    #[test]
    fn test_write_hash_section() {
        let mut buf = Cursor::new(Vec::new());
        let md5 = [0xABu8; 16];
        write_hash_section(&mut buf, &md5, 0).unwrap();

        let data = buf.into_inner();
        let expected_size = SECTION_HEADER_SIZE + HASH_SECTION_DATA_SIZE;
        assert_eq!(data.len(), expected_size);

        // Verify MD5 hash
        let hash_data = &data[SECTION_HEADER_SIZE..];
        assert_eq!(&hash_data[0..16], &md5);

        // Verify checksum
        let stored_checksum = u32::from_le_bytes(hash_data[16..20].try_into().unwrap());
        let expected_checksum = adler32(&hash_data[0..16]);
        assert_eq!(stored_checksum, expected_checksum);
    }

    #[test]
    fn test_write_digest_section() {
        let mut buf = Cursor::new(Vec::new());
        let sha1 = [0xCDu8; 20];
        write_digest_section(&mut buf, &sha1, 0).unwrap();

        let data = buf.into_inner();
        assert_eq!(data.len(), SECTION_HEADER_SIZE + DIGEST_SECTION_DATA_SIZE);
    }

    #[test]
    fn test_write_done_section() {
        let mut buf = Cursor::new(Vec::new());
        write_done_section(&mut buf).unwrap();

        let data = buf.into_inner();
        assert_eq!(data.len(), SECTION_HEADER_SIZE);

        // Check section type
        assert_eq!(&data[0..4], b"done");

        // Check next_offset is 0
        let next_offset = u64::from_le_bytes(data[16..24].try_into().unwrap());
        assert_eq!(next_offset, 0);
    }

    #[test]
    fn test_write_table_section() {
        let mut buf = Cursor::new(Vec::new());
        let mut table = ChunkTable::new(1000);
        table.add_chunk(0, 500, true); // Compressed chunk
        table.add_chunk(500, 32768, false); // Uncompressed chunk

        write_table_section(&mut buf, &table, 0).unwrap();

        let data = buf.into_inner();

        // Verify section type
        assert_eq!(&data[0..5], b"table");

        // Verify table header
        let th = &data[SECTION_HEADER_SIZE..];
        let chunk_count = u32::from_le_bytes(th[0..4].try_into().unwrap());
        assert_eq!(chunk_count, 2);

        let base_offset = u64::from_le_bytes(th[8..16].try_into().unwrap());
        assert_eq!(base_offset, 1000);

        // Verify chunk entries
        let entries_start = TABLE_HEADER_SIZE;
        let entry0 = u32::from_le_bytes(th[entries_start..entries_start + 4].try_into().unwrap());
        // First chunk: offset=0, compressed → MSB set
        assert_eq!(entry0, 0x8000_0000);

        let entry1 =
            u32::from_le_bytes(th[entries_start + 4..entries_start + 8].try_into().unwrap());
        // Second chunk: offset=500, not compressed → MSB clear
        assert_eq!(entry1, 500);
    }

    #[test]
    fn test_write_header_section() {
        let mut buf = Cursor::new(Vec::new());
        let case_info = L01CaseInfo {
            case_number: "2024-001".into(),
            evidence_number: "E001".into(),
            description: "Test case".into(),
            examiner: "Forensic Expert".into(),
            notes: "Test notes".into(),
        };

        let size = write_header_section(&mut buf, &case_info, 0).unwrap();
        assert!(size > SECTION_HEADER_SIZE as u64);

        let data = buf.into_inner();
        assert_eq!(&data[0..6], b"header");
    }

    #[test]
    fn test_write_ltree_section() {
        let mut buf = Cursor::new(Vec::new());
        let ltree_data = ltree::utf8_to_utf16le("entry\r\n\tn\ttest.txt\r\n");

        let size = write_ltree_section(&mut buf, &ltree_data, 0).unwrap();
        assert!(size > SECTION_HEADER_SIZE as u64);

        let data = buf.into_inner();
        assert_eq!(&data[0..5], b"ltree");
    }

    #[test]
    fn test_write_next_section() {
        let mut buf = Cursor::new(Vec::new());
        write_next_section(&mut buf).unwrap();

        let data = buf.into_inner();
        assert_eq!(data.len(), SECTION_HEADER_SIZE);
        assert_eq!(&data[0..4], b"next");
    }
}
