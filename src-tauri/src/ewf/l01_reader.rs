// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! L01 logical evidence file reader.
//!
//! Parses the ltree section from L01 files to reconstruct the file/directory
//! hierarchy. Enables browsing and extracting individual files from L01
//! containers without needing to extract the entire data stream.
//!
//! ## Ltree Format
//!
//! The ltree section is stored as zlib-compressed UTF-16LE text with a 48-byte
//! header (16-byte MD5 + 8-byte data size + 4-byte checksum + 20 reserved).
//! The text is tab-separated with 5 categories:
//!
//! - `rec`   — record summary
//! - `perm`  — permission groups
//! - `srce`  — acquisition sources
//! - `sub`   — subjects
//! - `entry` — file/directory tree
//!
//! Tree depth is indicated by leading tab count.
//! Each field is a key-value pair separated by tabs.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use tracing::{debug, trace, warn};

use crate::common::segments::discover_l01_segments;
use crate::containers::ContainerError;

// ─── Types ──────────────────────────────────────────────────────────────────

/// A parsed file/directory entry from the ltree section
#[derive(Debug, Clone)]
pub struct L01Entry {
    /// Unique identifier within the L01 image
    pub identifier: u64,
    /// Entry name (file or directory name)
    pub name: String,
    /// GUID for this entry
    pub guid: String,
    /// Whether this is a directory (cid=1) or file (cid=0)
    pub is_directory: bool,
    /// Logical file size in bytes (ls field)
    pub size: u64,
    /// Data offset in the decompressed data stream (from be field)
    pub data_offset: u64,
    /// Data size in the compressed stream (from be field)
    pub data_size: u64,
    /// Parent identifier (0 = root)
    pub parent_id: u64,
    /// Full path within the L01 (computed)
    pub path: String,
    /// MD5 hash if stored (ha field)
    pub md5_hash: Option<String>,
    /// SHA-1 hash if stored (sha field)
    pub sha1_hash: Option<String>,
    /// Creation time (POSIX seconds)
    pub creation_time: i64,
    /// Modification time (POSIX seconds)
    pub modification_time: i64,
    /// Access time (POSIX seconds)
    pub access_time: i64,
    /// Entry modification time (POSIX seconds)
    pub entry_modification_time: i64,
}

impl L01Entry {
    fn new() -> Self {
        Self {
            identifier: 0,
            name: String::new(),
            guid: String::new(),
            is_directory: false,
            size: 0,
            data_offset: 0,
            data_size: 0,
            parent_id: 0,
            path: String::new(),
            md5_hash: None,
            sha1_hash: None,
            creation_time: 0,
            modification_time: 0,
            access_time: 0,
            entry_modification_time: 0,
        }
    }
}

/// Record summary from the `rec` category
#[derive(Debug, Clone)]
pub struct L01RecordSummary {
    /// Total data bytes acquired
    pub total_bytes: u64,
    /// Number of files
    pub file_count: u64,
    /// Cluster size (0 for logical)
    pub cluster_size: u64,
}

/// Source info from the `srce` category
#[derive(Debug, Clone)]
pub struct L01SourceInfo {
    /// Source name
    pub name: String,
    /// Source identifier
    pub identifier: u64,
    /// Evidence number
    pub evidence_number: String,
}

/// Parsed L01 file tree
#[derive(Debug)]
pub struct L01FileTree {
    /// All entries (files and directories)
    pub entries: Vec<L01Entry>,
    /// Record summary
    pub record_summary: Option<L01RecordSummary>,
    /// Source information
    pub sources: Vec<L01SourceInfo>,
    /// Map from identifier to entry index for fast lookup
    id_to_index: HashMap<u64, usize>,
}

impl L01FileTree {
    /// Get root entries (entries whose parent_id is 0)
    pub fn root_entries(&self) -> Vec<&L01Entry> {
        self.entries.iter().filter(|e| e.parent_id == 0).collect()
    }

    /// Get children of a specific entry by parent identifier
    pub fn children_of(&self, parent_id: u64) -> Vec<&L01Entry> {
        self.entries
            .iter()
            .filter(|e| e.parent_id == parent_id)
            .collect()
    }

    /// Get children of an entry by its path
    pub fn children_at_path(&self, path: &str) -> Vec<&L01Entry> {
        // Find the entry with this path
        if path == "/" || path.is_empty() {
            return self.root_entries();
        }

        // Find entry matching this path
        if let Some(entry) = self.entries.iter().find(|e| e.path == path) {
            self.children_of(entry.identifier)
        } else {
            Vec::new()
        }
    }

    /// Find an entry by its path
    pub fn entry_at_path(&self, path: &str) -> Option<&L01Entry> {
        self.entries.iter().find(|e| e.path == path)
    }

    /// Find an entry by its identifier
    pub fn entry_by_id(&self, id: u64) -> Option<&L01Entry> {
        self.id_to_index.get(&id).map(|&idx| &self.entries[idx])
    }

    /// Get total number of files
    pub fn file_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.is_directory).count()
    }

    /// Get total number of directories
    pub fn directory_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_directory).count()
    }
}

// ─── Parsing ────────────────────────────────────────────────────────────────

/// Parse the ltree section from an L01 file.
///
/// This reads the file, locates the ltree section, decompresses it,
/// and parses the UTF-16LE text into an `L01FileTree`.
pub fn parse_l01_file_tree(path: &str) -> Result<L01FileTree, ContainerError> {
    debug!("Parsing L01 file tree from: {}", path);

    // Discover all segment files (.L01, .L02, .L03, ...)
    let segment_paths =
        discover_l01_segments(path).unwrap_or_else(|_| vec![Path::new(path).to_path_buf()]);
    debug!("L01 segments to scan for ltree: {}", segment_paths.len());

    // Try each segment file — the ltree section is often in the last segment
    // Try in reverse order (last segment first) since that's where ltree typically lives
    let mut last_error = None;
    for segment_path in segment_paths.iter().rev() {
        let segment_str = segment_path.to_string_lossy();
        debug!("Scanning segment for ltree: {}", segment_str);

        let mut file = match File::open(segment_path) {
            Ok(f) => f,
            Err(e) => {
                warn!("Failed to open L01 segment {}: {}", segment_str, e);
                continue;
            }
        };

        // Verify LVF signature
        let mut sig = [0u8; 8];
        if file.read_exact(&mut sig).is_err() {
            continue;
        }

        let is_l01 = &sig == b"LVF\x09\x0d\x0a\xff\x00";
        let is_lx01 = &sig == b"LVF2\x0d\x0a\x81\x00";

        if !is_l01 && !is_lx01 {
            continue;
        }

        let file_size = match file.metadata() {
            Ok(m) => m.len(),
            Err(_) => continue,
        };

        // Scan sections to find ltree
        match find_and_read_ltree_section(&mut file, 13, file_size) {
            Ok(ltree_data) => {
                debug!("Found ltree in segment: {}", segment_str);
                // Decode UTF-16LE to string
                let text = decode_utf16le(&ltree_data)?;
                // Parse the ltree text into entries
                return parse_ltree_text(&text);
            }
            Err(e) => {
                debug!("No ltree in segment {}: {}", segment_str, e);
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        ContainerError::ParseError("No ltree section found in any L01 segment file".to_string())
    }))
}

/// Scan section headers to find the ltree section, read and decompress it.
fn find_and_read_ltree_section(
    file: &mut File,
    start_offset: u64,
    file_size: u64,
) -> Result<Vec<u8>, ContainerError> {
    let mut offset = start_offset;
    let max_sections = 1000;
    let mut section_count = 0;

    while offset < file_size && section_count < max_sections {
        if offset + 76 > file_size {
            break;
        }

        file.seek(SeekFrom::Start(offset))?;

        let mut header = [0u8; 76];
        if file.read_exact(&mut header).is_err() {
            break;
        }

        // Parse section type (first 16 bytes, null-terminated)
        let section_type: String = header[0..16]
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect();

        // Parse next_offset and section_size
        let next_offset = u64::from_le_bytes([
            header[16], header[17], header[18], header[19], header[20], header[21], header[22],
            header[23],
        ]);

        let section_size = u64::from_le_bytes([
            header[24], header[25], header[26], header[27], header[28], header[29], header[30],
            header[31],
        ]);

        trace!(
            "L01 section '{}' at offset {}, size {}, next {}",
            section_type,
            offset,
            section_size,
            next_offset
        );

        if section_type == "ltree" {
            // Found the ltree section — read its data
            let data_offset = offset + 76; // Skip section header

            // Ltree has a 48-byte sub-header:
            // 0..16: MD5 hash of uncompressed data
            // 16..24: uncompressed data size (u64 LE)
            // 24..28: Adler32 checksum of first 24 bytes
            // 28..48: reserved/padding
            file.seek(SeekFrom::Start(data_offset))?;
            let mut ltree_header = [0u8; 48];
            file.read_exact(&mut ltree_header).map_err(|e| {
                ContainerError::ParseError(format!("Failed to read ltree header: {}", e))
            })?;

            let uncompressed_size = u64::from_le_bytes([
                ltree_header[16],
                ltree_header[17],
                ltree_header[18],
                ltree_header[19],
                ltree_header[20],
                ltree_header[21],
                ltree_header[22],
                ltree_header[23],
            ]);

            // The compressed data follows the 48-byte sub-header
            let compressed_data_offset = data_offset + 48;

            // Calculate compressed data size. Some EWF writers set section_size=0
            // and encode the data extent via next_offset instead.
            let compressed_size = if section_size > 124 {
                // section_size includes the 76-byte header + 48-byte sub-header
                section_size - 76 - 48
            } else if next_offset > compressed_data_offset {
                // Fallback: use next_offset to determine data end
                next_offset - compressed_data_offset
            } else {
                // Last resort: read from current position to end of file
                file_size.saturating_sub(compressed_data_offset)
            };

            debug!(
                "Ltree section: compressed={}, uncompressed={}",
                compressed_size, uncompressed_size
            );

            file.seek(SeekFrom::Start(compressed_data_offset))?;
            let mut raw_data = vec![0u8; compressed_size as usize];
            file.read_exact(&mut raw_data).map_err(|e| {
                ContainerError::ParseError(format!("Failed to read ltree data: {}", e))
            })?;

            // If compressed == uncompressed size, data is stored uncompressed.
            // Also try detecting zlib header (0x78) to confirm compression.
            let decompressed = if compressed_size == uncompressed_size
                || (compressed_size > 0 && raw_data[0] != 0x78)
            {
                debug!("Ltree data is uncompressed ({} bytes)", raw_data.len());
                raw_data
            } else {
                decompress_zlib(&raw_data, uncompressed_size)?
            };

            debug!("Ltree data ready: {} bytes", decompressed.len());

            return Ok(decompressed);
        }

        // Check for terminal sections
        if section_type == "done" {
            break;
        }

        // Move to next section
        if next_offset > 0 && next_offset > offset {
            offset = next_offset;
        } else if section_size > 0 {
            offset += section_size;
        } else {
            break;
        }

        section_count += 1;
    }

    Err(ContainerError::ParseError(
        "No ltree section found in L01 file".to_string(),
    ))
}

/// Decompress zlib-compressed data
fn decompress_zlib(data: &[u8], expected_size: u64) -> Result<Vec<u8>, ContainerError> {
    use flate2::read::ZlibDecoder;

    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::with_capacity(expected_size as usize);
    decoder.read_to_end(&mut decompressed).map_err(|e| {
        ContainerError::ParseError(format!("Failed to decompress ltree data: {}", e))
    })?;

    Ok(decompressed)
}

/// Decode UTF-16LE bytes to a String.
/// Handles optional BOM (0xFF 0xFE).
fn decode_utf16le(data: &[u8]) -> Result<String, ContainerError> {
    let mut start = 0;

    // Check for BOM
    if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xFE {
        start = 2;
    }

    let remaining = &data[start..];
    if !remaining.len().is_multiple_of(2) {
        warn!("UTF-16LE data has odd length, truncating last byte");
    }

    let code_units: Vec<u16> = remaining
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();

    String::from_utf16(&code_units)
        .map_err(|e| ContainerError::ParseError(format!("Failed to decode UTF-16LE: {}", e)))
}

/// Parse the decoded ltree text into an L01FileTree.
fn parse_ltree_text(text: &str) -> Result<L01FileTree, ContainerError> {
    // Detect ltree format version.
    // V3 format starts with "3\n" as the first line (version indicator).
    // V1 format has no version line — categories start immediately.
    let first_line = text.lines().next().unwrap_or("");
    if first_line.trim() == "3" {
        debug!("Detected ltree V3 format");
        return parse_ltree_text_v3(text);
    }

    debug!("Detected ltree V1 format");
    parse_ltree_text_v1(text)
}

/// Parse ltree V3 columnar format.
///
/// V3 structure:
///   - Line 0: "3" (version)
///   - Category blocks: category_name, then count\tflag, then column_headers,
///     then pairs of (type_line, data_line) for each entry.
///   - Entry type_line: "<field_count>\t<child_count>"
///     child_count > 0 means directory; 0 means file.
///   - Entry data_line: tab-separated values matching column headers.
///   - Hierarchy is implicit via child_count stack.
fn parse_ltree_text_v3(text: &str) -> Result<L01FileTree, ContainerError> {
    let mut entries = Vec::new();
    let mut record_summary = None;
    let mut sources = Vec::new();

    let lines: Vec<&str> = text.lines().collect();
    let mut line_idx = 0;

    // Skip version line
    if line_idx < lines.len() && lines[line_idx].trim() == "3" {
        line_idx += 1;
    }

    // parent_stack: Vec<(identifier, remaining_children)>
    let mut parent_stack: Vec<(u64, u64)> = Vec::new();

    while line_idx < lines.len() {
        let line = lines[line_idx].trim_end_matches('\r');
        line_idx += 1;

        if line.is_empty() {
            continue;
        }

        // Detect category header (no tabs, simple word)
        if !line.contains('\t') && line.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            let category = line.to_string();

            match category.as_str() {
                "rec" => {
                    // rec category: count\tflag line, then column_headers, then data lines
                    // But rec is special — it's just one data record
                    if line_idx < lines.len() {
                        let count_line = lines[line_idx].trim_end_matches('\r');
                        line_idx += 1;
                        // Column headers
                        if line_idx < lines.len() {
                            let cols: Vec<&str> =
                                lines[line_idx].trim_end_matches('\r').split('\t').collect();
                            line_idx += 1;
                            // Data line(s)
                            if line_idx < lines.len() {
                                let vals: Vec<&str> =
                                    lines[line_idx].trim_end_matches('\r').split('\t').collect();
                                line_idx += 1;
                                // Map columns to values
                                let col_map = build_column_map(&cols, &vals);
                                record_summary = Some(L01RecordSummary {
                                    total_bytes: col_map
                                        .get("tb")
                                        .and_then(|v| v.parse().ok())
                                        .unwrap_or_else(|| {
                                            // Fallback: first value in count_line
                                            count_line
                                                .split('\t')
                                                .next()
                                                .and_then(|v| v.parse().ok())
                                                .unwrap_or(0)
                                        }),
                                    file_count: col_map
                                        .get("cl")
                                        .and_then(|v| v.parse().ok())
                                        .unwrap_or(0),
                                    cluster_size: col_map
                                        .get("n")
                                        .and_then(|v| v.parse().ok())
                                        .unwrap_or(0),
                                });
                            }
                        }
                    }
                }
                "perm" | "sub" => {
                    // Skip perm and sub categories — consume their lines
                    // Count/flag line
                    if line_idx < lines.len() {
                        let count_line = lines[line_idx].trim_end_matches('\r');
                        line_idx += 1;
                        // Column headers
                        if line_idx < lines.len() {
                            line_idx += 1; // skip headers
                                           // Parse count from count_line to know how many records
                            let entry_count: usize = count_line
                                .split('\t')
                                .next()
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(0);
                            // Each record is 2 lines (type + data)
                            let lines_to_skip = entry_count * 2;
                            line_idx += lines_to_skip.min(lines.len().saturating_sub(line_idx));
                        }
                    }
                }
                "srce" => {
                    // Source entries
                    if line_idx < lines.len() {
                        let count_line = lines[line_idx].trim_end_matches('\r');
                        line_idx += 1;
                        if line_idx < lines.len() {
                            let cols: Vec<&str> =
                                lines[line_idx].trim_end_matches('\r').split('\t').collect();
                            line_idx += 1;
                            let entry_count: usize = count_line
                                .split('\t')
                                .next()
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(0);
                            for _ in 0..entry_count {
                                if line_idx + 1 >= lines.len() {
                                    break;
                                }
                                line_idx += 1; // skip type line
                                let vals: Vec<&str> =
                                    lines[line_idx].trim_end_matches('\r').split('\t').collect();
                                line_idx += 1;
                                let col_map = build_column_map(&cols, &vals);
                                if let Some(name) = col_map.get("n") {
                                    if !name.is_empty() {
                                        sources.push(L01SourceInfo {
                                            name: name.clone(),
                                            identifier: col_map
                                                .get("id")
                                                .and_then(|v| v.parse().ok())
                                                .unwrap_or(0),
                                            evidence_number: col_map
                                                .get("ev")
                                                .cloned()
                                                .unwrap_or_default(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                "entry" => {
                    // Entry records — the main file/directory tree
                    if line_idx < lines.len() {
                        let _count_line = lines[line_idx].trim_end_matches('\r');
                        line_idx += 1;
                        if line_idx < lines.len() {
                            let cols: Vec<&str> =
                                lines[line_idx].trim_end_matches('\r').split('\t').collect();
                            line_idx += 1;
                            parent_stack.clear();

                            // Pre-compute column indices for O(1) field access
                            // instead of building a HashMap per entry
                            let col_n = cols.iter().position(|&c| c == "n");
                            let col_mid = cols.iter().position(|&c| c == "mid");
                            let col_id = cols.iter().position(|&c| c == "id");
                            let col_ls = cols.iter().position(|&c| c == "ls");
                            let col_p = cols.iter().position(|&c| c == "p");
                            let col_be = cols.iter().position(|&c| c == "be");
                            let col_cr = cols.iter().position(|&c| c == "cr");
                            let col_wr = cols.iter().position(|&c| c == "wr");
                            let col_ac = cols.iter().position(|&c| c == "ac");
                            let col_mo = cols.iter().position(|&c| c == "mo");
                            let col_ha = cols.iter().position(|&c| c == "ha");
                            let col_sha = cols.iter().position(|&c| c == "sha");

                            // id → path map for O(1) parent path lookup (replaces O(n) linear scan)
                            let mut id_to_path: HashMap<u64, String> = HashMap::new();

                            // Parse entries until we hit an empty line or new category
                            while line_idx + 1 < lines.len() {
                                let type_line = lines[line_idx].trim_end_matches('\r');

                                // Check if this is still a type indicator line (digits\tdigits)
                                if type_line.is_empty() {
                                    line_idx += 1;
                                    continue;
                                }

                                // Type indicator: <num>\t<child_count>
                                let type_parts: Vec<&str> = type_line.split('\t').collect();
                                if type_parts.len() < 2 || type_parts[0].parse::<u32>().is_err() {
                                    // Not a type indicator — probably a new category
                                    break;
                                }

                                let child_count: u64 = type_parts[1].parse().unwrap_or(0);
                                line_idx += 1;

                                if line_idx >= lines.len() {
                                    break;
                                }
                                let data_line = lines[line_idx].trim_end_matches('\r');
                                line_idx += 1;

                                // Split data by tabs (including leading empty fields)
                                let vals: Vec<&str> = data_line.split('\t').collect();

                                // Helper: get non-empty field by pre-computed column index
                                let get_val = |idx: Option<usize>| -> Option<&str> {
                                    idx.and_then(|i| vals.get(i))
                                        .filter(|v| !v.is_empty())
                                        .copied()
                                };

                                let mut entry = L01Entry::new();

                                // Name
                                if let Some(name) = get_val(col_n) {
                                    entry.name = name.to_string();
                                }

                                // GUID
                                if let Some(guid) = get_val(col_mid) {
                                    entry.guid = guid.to_string();
                                }

                                // Identifier
                                if let Some(id_str) = get_val(col_id) {
                                    entry.identifier = id_str.parse().unwrap_or(0);
                                }

                                // Logical size
                                if let Some(ls) = get_val(col_ls) {
                                    entry.size = ls.parse().unwrap_or(0);
                                }

                                // Is directory: child_count > 0 OR p field starts with "1"
                                let p_flag =
                                    get_val(col_p).map(|v| v.starts_with('1')).unwrap_or(false);
                                entry.is_directory = child_count > 0 || p_flag;

                                // Binary extents
                                if let Some(be) = get_val(col_be) {
                                    let parts: Vec<&str> = be.split_whitespace().collect();
                                    if parts.len() >= 2 {
                                        entry.data_offset = parts[0].parse().unwrap_or(0);
                                        entry.data_size = parts[1].parse().unwrap_or(0);
                                    }
                                }

                                // Timestamps
                                if let Some(cr) = get_val(col_cr) {
                                    entry.creation_time = cr.parse().unwrap_or(0);
                                }
                                if let Some(wr) = get_val(col_wr) {
                                    entry.modification_time = wr.parse().unwrap_or(0);
                                }
                                if let Some(ac) = get_val(col_ac) {
                                    entry.access_time = ac.parse().unwrap_or(0);
                                }
                                if let Some(mo) = get_val(col_mo) {
                                    entry.entry_modification_time = mo.parse().unwrap_or(0);
                                }

                                // Hashes
                                if let Some(ha) = get_val(col_ha) {
                                    entry.md5_hash = Some(ha.to_string());
                                }
                                if let Some(sha) = get_val(col_sha) {
                                    entry.sha1_hash = Some(sha.to_string());
                                }

                                // Determine parent from child_count stack
                                // Pop parents whose child counts are exhausted
                                while let Some(last) = parent_stack.last_mut() {
                                    if last.1 == 0 {
                                        parent_stack.pop();
                                    } else {
                                        break;
                                    }
                                }

                                // Parent is top of stack (or root=0)
                                entry.parent_id =
                                    parent_stack.last().map(|&(id, _)| id).unwrap_or(0);

                                // Decrement parent's remaining child count
                                if let Some(last) = parent_stack.last_mut() {
                                    last.1 = last.1.saturating_sub(1);
                                }

                                // If this is a directory, push onto parent stack
                                if entry.is_directory && child_count > 0 {
                                    parent_stack.push((entry.identifier, child_count));
                                }

                                // Build path using O(1) HashMap lookup instead of O(n) linear scan
                                entry.path = if entry.parent_id == 0 {
                                    format!("/{}", entry.name)
                                } else if let Some(parent_path) = id_to_path.get(&entry.parent_id) {
                                    format!("{}/{}", parent_path, entry.name)
                                } else {
                                    format!("/{}", entry.name)
                                };

                                // Register this entry's path for future children
                                if entry.is_directory {
                                    id_to_path.insert(entry.identifier, entry.path.clone());
                                }

                                entries.push(entry);
                            }
                        }
                    }
                }
                _ => {
                    // Unknown category — skip lines until next category
                    while line_idx < lines.len() {
                        let peek = lines[line_idx].trim_end_matches('\r');
                        if !peek.is_empty()
                            && !peek.contains('\t')
                            && peek.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                        {
                            break; // New category found
                        }
                        line_idx += 1;
                    }
                }
            }
            continue;
        }

        // Lines outside any category — skip
    }

    // Build identifier -> index map
    let mut id_to_index = HashMap::new();
    for (idx, entry) in entries.iter().enumerate() {
        id_to_index.insert(entry.identifier, idx);
    }

    debug!(
        "Parsed L01 V3 file tree: {} entries ({} files, {} directories)",
        entries.len(),
        entries.iter().filter(|e| !e.is_directory).count(),
        entries.iter().filter(|e| e.is_directory).count(),
    );

    Ok(L01FileTree {
        entries,
        record_summary,
        sources,
        id_to_index,
    })
}

/// Build a column-name to value map from column headers and tab-separated values.
fn build_column_map(cols: &[&str], vals: &[&str]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (i, col) in cols.iter().enumerate() {
        if let Some(val) = vals.get(i) {
            if !col.is_empty() && !val.is_empty() {
                map.insert(col.to_string(), val.to_string());
            }
        }
    }
    map
}

/// Parse ltree V1 key=value format (original parser).
fn parse_ltree_text_v1(text: &str) -> Result<L01FileTree, ContainerError> {
    let mut entries = Vec::new();
    let mut record_summary = None;
    let mut sources = Vec::new();

    // Split into lines (CR+LF or LF)
    let lines: Vec<&str> = text.lines().collect();

    let mut current_category = String::new();
    // Stack of (identifier, depth) for building parent hierarchy
    let mut parent_stack: Vec<(u64, usize)> = Vec::new();
    // id → path map for O(1) parent path lookup (replaces O(n) linear scan)
    let mut id_to_path: HashMap<u64, String> = HashMap::new();
    let mut line_idx = 0;

    while line_idx < lines.len() {
        let line = lines[line_idx];
        line_idx += 1;

        if line.is_empty() {
            continue;
        }

        // Count leading tabs to determine depth
        let depth = line.chars().take_while(|&c| c == '\t').count();
        let trimmed = &line[depth..];

        if trimmed.is_empty() {
            continue;
        }

        // Category headers are at depth 0 with no tab prefix
        if depth == 0 && !trimmed.contains('\t') {
            current_category = trimmed.to_string();
            // Reset parent stack for entry category
            if current_category == "entry" {
                parent_stack.clear();
            }
            continue;
        }

        match current_category.as_str() {
            "rec" => {
                // Already parsed on the category line itself
                // rec\t<total_bytes>\t<file_count>\t<cluster_size>
                if depth == 0 {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 4 && parts[0] == "rec" {
                        record_summary = Some(L01RecordSummary {
                            total_bytes: parts[1].parse().unwrap_or(0),
                            file_count: parts[2].parse().unwrap_or(0),
                            cluster_size: parts[3].parse().unwrap_or(0),
                        });
                    }
                }
            }
            "srce" => {
                // Source entries
                let fields = parse_tab_fields(trimmed);
                if let Some(name) = fields.get("n") {
                    let source = L01SourceInfo {
                        name: name.clone(),
                        identifier: fields.get("id").and_then(|v| v.parse().ok()).unwrap_or(0),
                        evidence_number: fields.get("ev").cloned().unwrap_or_default(),
                    };
                    sources.push(source);
                }
            }
            "entry" => {
                // Parse file/directory entry
                let fields = parse_tab_fields(trimmed);

                let mut entry = L01Entry::new();

                // Determine if directory (has p=1) or file
                let is_parent = fields.get("p").map(|v| v == "1").unwrap_or(false);
                let cid = fields
                    .get("cid")
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(0);
                entry.is_directory = is_parent || cid == 1;

                // Name
                if let Some(name) = fields.get("n") {
                    entry.name = name.clone();
                }

                // GUID
                if let Some(guid) = fields.get("mid") {
                    entry.guid = guid.clone();
                }

                // Identifier
                if let Some(id_str) = fields.get("id") {
                    entry.identifier = id_str.parse().unwrap_or(0);
                }

                // Logical size
                if let Some(ls) = fields.get("ls") {
                    entry.size = ls.parse().unwrap_or(0);
                }

                // Binary extents (be field): "offset size" or "offset size offset2 size2 ..."
                if let Some(be) = fields.get("be") {
                    let parts: Vec<&str> = be.split_whitespace().collect();
                    if parts.len() >= 2 {
                        entry.data_offset = parts[0].parse().unwrap_or(0);
                        entry.data_size = parts[1].parse().unwrap_or(0);
                    }
                }

                // Timestamps
                if let Some(cr) = fields.get("cr") {
                    entry.creation_time = cr.parse().unwrap_or(0);
                }
                if let Some(wr) = fields.get("wr") {
                    entry.modification_time = wr.parse().unwrap_or(0);
                }
                if let Some(ac) = fields.get("ac") {
                    entry.access_time = ac.parse().unwrap_or(0);
                }
                if let Some(mo) = fields.get("mo") {
                    entry.entry_modification_time = mo.parse().unwrap_or(0);
                }

                // Hashes
                if let Some(ha) = fields.get("ha") {
                    entry.md5_hash = Some(ha.clone());
                }
                if let Some(sha) = fields.get("sha") {
                    entry.sha1_hash = Some(sha.clone());
                }

                // Determine parent from depth using parent stack
                // depth=1 means root entry (child of virtual root 0)
                // depth=2 means child of the last directory at depth=1
                // etc.

                // Pop entries from stack until we find one at depth < current
                while let Some(&(_, d)) = parent_stack.last() {
                    if d >= depth {
                        parent_stack.pop();
                    } else {
                        break;
                    }
                }

                // Parent is the top of stack, or 0 (root) if stack is empty
                entry.parent_id = parent_stack.last().map(|&(id, _)| id).unwrap_or(0);

                // If this is a directory, push it onto the stack
                if entry.is_directory {
                    parent_stack.push((entry.identifier, depth));
                }

                // Build path using O(1) HashMap lookup instead of O(n) linear scan
                entry.path = if entry.parent_id == 0 {
                    format!("/{}", entry.name)
                } else if let Some(parent_path) = id_to_path.get(&entry.parent_id) {
                    format!("{}/{}", parent_path, entry.name)
                } else {
                    format!("/{}", entry.name)
                };

                // Register this entry's path for future children
                if entry.is_directory {
                    id_to_path.insert(entry.identifier, entry.path.clone());
                }

                entries.push(entry);
            }
            _ => {
                // Skip perm, sub, and other categories
            }
        }
    }

    // Handle the rec category if it was on the same line as the category name
    // (rec\ttotal\tcount\tcluster on line 0)
    if record_summary.is_none() {
        // Try parsing from the original text
        for line in text.lines() {
            if line.starts_with("rec\t") {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 4 {
                    record_summary = Some(L01RecordSummary {
                        total_bytes: parts[1].parse().unwrap_or(0),
                        file_count: parts[2].parse().unwrap_or(0),
                        cluster_size: parts[3].parse().unwrap_or(0),
                    });
                }
                break;
            }
        }
    }

    // Build identifier -> index map
    let mut id_to_index = HashMap::new();
    for (idx, entry) in entries.iter().enumerate() {
        id_to_index.insert(entry.identifier, idx);
    }

    debug!(
        "Parsed L01 file tree: {} entries ({} files, {} directories)",
        entries.len(),
        entries.iter().filter(|e| !e.is_directory).count(),
        entries.iter().filter(|e| e.is_directory).count(),
    );

    Ok(L01FileTree {
        entries,
        record_summary,
        sources,
        id_to_index,
    })
}

/// Parse tab-separated key-value fields from a line.
///
/// The format uses alternating key\tvalue pairs:
/// `n\tfilename\tid\t42\tls\t1024`
/// becomes: {"n": "filename", "id": "42", "ls": "1024"}
///
/// Special case: "p\t1" at the start marks a directory.
fn parse_tab_fields(line: &str) -> HashMap<String, String> {
    let mut fields = HashMap::new();
    let parts: Vec<&str> = line.split('\t').collect();

    let mut i = 0;
    while i + 1 < parts.len() {
        let key = parts[i];
        let value = parts[i + 1];
        fields.insert(key.to_string(), value.to_string());
        i += 2;
    }

    fields
}

/// Build the full path for an entry based on its parents.
/// Used as fallback for edge cases — main parsers use O(1) id_to_path HashMap.
#[allow(dead_code)]
fn build_entry_path(existing_entries: &[L01Entry], entry: &L01Entry) -> String {
    if entry.parent_id == 0 {
        // Root-level entry
        return format!("/{}", entry.name);
    }

    // Find parent path
    if let Some(parent) = existing_entries
        .iter()
        .find(|e| e.identifier == entry.parent_id)
    {
        let parent_path = if parent.path.ends_with('/') {
            parent.path.clone()
        } else {
            format!("{}/", parent.path)
        };
        format!("{}{}", parent_path, entry.name)
    } else {
        // Parent not found (shouldn't happen in well-formed ltree)
        format!("/{}", entry.name)
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tab_fields() {
        let fields = parse_tab_fields("n\ttest.txt\tid\t42\tls\t1024");
        assert_eq!(fields.get("n").unwrap(), "test.txt");
        assert_eq!(fields.get("id").unwrap(), "42");
        assert_eq!(fields.get("ls").unwrap(), "1024");
    }

    #[test]
    fn test_parse_tab_fields_directory() {
        let fields = parse_tab_fields("p\t1\tn\tDocuments\tid\t1\tcid\t1");
        assert_eq!(fields.get("p").unwrap(), "1");
        assert_eq!(fields.get("n").unwrap(), "Documents");
        assert_eq!(fields.get("cid").unwrap(), "1");
    }

    #[test]
    fn test_decode_utf16le_with_bom() {
        // "Hello" in UTF-16LE with BOM
        let data: Vec<u8> = vec![
            0xFF, 0xFE, // BOM
            b'H', 0, b'e', 0, b'l', 0, b'l', 0, b'o', 0,
        ];
        let result = decode_utf16le(&data).unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_decode_utf16le_without_bom() {
        // "Hi" in UTF-16LE without BOM
        let data: Vec<u8> = vec![b'H', 0, b'i', 0];
        let result = decode_utf16le(&data).unwrap();
        assert_eq!(result, "Hi");
    }

    #[test]
    fn test_parse_ltree_text_basic() {
        let text = "rec\t1024\t2\t0\r\n\
                     perm\r\n\
                     srce\r\n\
                     sub\r\n\
                     entry\r\n\
                     \tp\t1\tn\tDocuments\tmid\tguid1\tid\t1\tcid\t1\r\n\
                     \t\tn\tfile1.txt\tmid\tguid2\tid\t2\tls\t512\tbe\t0 512\tcid\t0\r\n\
                     \t\tn\tfile2.txt\tmid\tguid3\tid\t3\tls\t512\tbe\t512 512\tcid\t0\r\n";

        let tree = parse_ltree_text(text).unwrap();

        assert_eq!(tree.entries.len(), 3);

        // Directory
        let docs = &tree.entries[0];
        assert_eq!(docs.name, "Documents");
        assert!(docs.is_directory);
        assert_eq!(docs.parent_id, 0);
        assert_eq!(docs.path, "/Documents");

        // Files
        let file1 = &tree.entries[1];
        assert_eq!(file1.name, "file1.txt");
        assert!(!file1.is_directory);
        assert_eq!(file1.size, 512);
        assert_eq!(file1.data_offset, 0);
        assert_eq!(file1.data_size, 512);
        assert_eq!(file1.parent_id, 1);
        assert_eq!(file1.path, "/Documents/file1.txt");

        let file2 = &tree.entries[2];
        assert_eq!(file2.name, "file2.txt");
        assert_eq!(file2.data_offset, 512);
        assert_eq!(file2.path, "/Documents/file2.txt");

        // Record summary
        let rec = tree.record_summary.as_ref().unwrap();
        assert_eq!(rec.total_bytes, 1024);
        assert_eq!(rec.file_count, 2);

        // Root entries
        let roots = tree.root_entries();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].name, "Documents");

        // Children
        let children = tree.children_of(1);
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_parse_ltree_text_nested_dirs() {
        let text = "rec\t0\t0\t0\r\n\
                     perm\r\n\
                     srce\r\n\
                     sub\r\n\
                     entry\r\n\
                     \tp\t1\tn\tRoot\tmid\tg1\tid\t1\tcid\t1\r\n\
                     \t\tp\t1\tn\tSub\tmid\tg2\tid\t2\tcid\t1\r\n\
                     \t\t\tn\tdeep.txt\tmid\tg3\tid\t3\tls\t100\tbe\t0 100\tcid\t0\r\n";

        let tree = parse_ltree_text(text).unwrap();
        assert_eq!(tree.entries.len(), 3);

        let root = &tree.entries[0];
        assert_eq!(root.path, "/Root");
        assert_eq!(root.parent_id, 0);

        let sub = &tree.entries[1];
        assert_eq!(sub.path, "/Root/Sub");
        assert_eq!(sub.parent_id, 1);

        let deep = &tree.entries[2];
        assert_eq!(deep.path, "/Root/Sub/deep.txt");
        assert_eq!(deep.parent_id, 2);
    }

    #[test]
    fn test_parse_ltree_text_with_hashes() {
        let text = "rec\t512\t1\t0\r\n\
                     perm\r\n\
                     srce\r\n\
                     sub\r\n\
                     entry\r\n\
                     \tn\ttest.bin\tmid\tg1\tid\t1\tls\t512\tbe\t0 512\tcid\t0\tha\tmd5hash\tsha\tsha1hash\r\n";

        let tree = parse_ltree_text(text).unwrap();
        let entry = &tree.entries[0];
        assert_eq!(entry.md5_hash.as_deref(), Some("md5hash"));
        assert_eq!(entry.sha1_hash.as_deref(), Some("sha1hash"));
    }

    #[test]
    fn test_parse_ltree_text_with_timestamps() {
        let text = "rec\t100\t1\t0\r\n\
                     perm\r\n\
                     srce\r\n\
                     sub\r\n\
                     entry\r\n\
                     \tn\ttest.txt\tmid\tg1\tid\t1\tls\t100\tbe\t0 100\tcid\t0\tcr\t1700000000\twr\t1700000100\tac\t1700000200\r\n";

        let tree = parse_ltree_text(text).unwrap();
        let entry = &tree.entries[0];
        assert_eq!(entry.creation_time, 1700000000);
        assert_eq!(entry.modification_time, 1700000100);
        assert_eq!(entry.access_time, 1700000200);
    }

    #[test]
    fn test_children_at_path() {
        let text = "rec\t0\t0\t0\r\n\
                     perm\r\n\
                     srce\r\n\
                     sub\r\n\
                     entry\r\n\
                     \tp\t1\tn\tFolder\tmid\tg1\tid\t1\tcid\t1\r\n\
                     \t\tn\ta.txt\tmid\tg2\tid\t2\tls\t10\tbe\t0 10\tcid\t0\r\n\
                     \t\tn\tb.txt\tmid\tg3\tid\t3\tls\t20\tbe\t10 20\tcid\t0\r\n";

        let tree = parse_ltree_text(text).unwrap();

        let root_children = tree.children_at_path("/");
        assert_eq!(root_children.len(), 1);
        assert_eq!(root_children[0].name, "Folder");

        let folder_children = tree.children_at_path("/Folder");
        assert_eq!(folder_children.len(), 2);
        assert_eq!(folder_children[0].name, "a.txt");
        assert_eq!(folder_children[1].name, "b.txt");
    }

    #[test]
    fn test_parse_srce_category() {
        let text = "rec\t0\t0\t0\r\n\
                     perm\r\n\
                     srce\r\n\
                     \tp\t1\tn\tMyDrive\tid\t1\tev\tEV001\r\n\
                     sub\r\n\
                     entry\r\n";

        let tree = parse_ltree_text(text).unwrap();
        assert_eq!(tree.sources.len(), 1);
        assert_eq!(tree.sources[0].name, "MyDrive");
        assert_eq!(tree.sources[0].identifier, 1);
        assert_eq!(tree.sources[0].evidence_number, "EV001");
    }

    #[test]
    fn test_file_and_directory_counts() {
        let text = "rec\t0\t0\t0\r\n\
                     perm\r\n\
                     srce\r\n\
                     sub\r\n\
                     entry\r\n\
                     \tp\t1\tn\tDir1\tmid\tg1\tid\t1\tcid\t1\r\n\
                     \t\tn\tf1.txt\tmid\tg2\tid\t2\tls\t10\tcid\t0\r\n\
                     \t\tp\t1\tn\tDir2\tmid\tg3\tid\t3\tcid\t1\r\n\
                     \t\t\tn\tf2.txt\tmid\tg4\tid\t4\tls\t20\tcid\t0\r\n\
                     \t\t\tn\tf3.txt\tmid\tg5\tid\t5\tls\t30\tcid\t0\r\n";

        let tree = parse_ltree_text(text).unwrap();
        assert_eq!(tree.file_count(), 3);
        assert_eq!(tree.directory_count(), 2);
    }
}
