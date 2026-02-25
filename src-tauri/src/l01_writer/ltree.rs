// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Ltree section builder for L01 files.
//!
//! The ltree section contains a UTF-16LE encoded, tab-separated text tree
//! describing the file hierarchy. It has 5 categories:
//!
//! - `rec`   — record summary (total bytes acquired, cluster info)
//! - `perm`  — permission groups (SIDs, ACL bitmasks)
//! - `srce`  — acquisition sources (evidence origin metadata)
//! - `sub`   — subjects (user accounts/profiles)
//! - `entry` — file entry tree (files and directories with metadata)
//!
//! Each category is a tree of tab-separated key-value fields.
//! Tree depth is indicated by tab indentation level.

use super::types::{
    LefFileEntry, LefPermissionGroup, LefRecordType, LefSource, LefSubject,
};

/// Line ending for ltree text (CR+LF)
const CRLF: &str = "\r\n";

/// Tab character for field separation / indentation
const TAB: &str = "\t";

/// Build the complete ltree text content.
///
/// Returns a UTF-8 string that will be converted to UTF-16LE before
/// being written into the ltree section.
pub fn build_ltree_text(
    entries: &[LefFileEntry],
    sources: &[LefSource],
    permission_groups: &[LefPermissionGroup],
    subjects: &[LefSubject],
    total_data_bytes: u64,
) -> String {
    let mut text = String::new();

    // Record category
    build_rec_category(&mut text, entries, total_data_bytes);

    // Permission groups category
    build_perm_category(&mut text, permission_groups);

    // Source category
    build_srce_category(&mut text, sources);

    // Subject category
    build_sub_category(&mut text, subjects);

    // Entry category (file tree)
    build_entry_category(&mut text, entries);

    text
}

/// Build the `rec` (record) category.
///
/// Format:
/// ```text
/// rec\t<total_bytes>\t<cluster_count>\t<cluster_size>
/// ```
fn build_rec_category(text: &mut String, entries: &[LefFileEntry], total_data_bytes: u64) {
    let file_count = entries
        .iter()
        .filter(|e| e.record_type == LefRecordType::File)
        .count();

    // rec category: total data bytes, number of files, block size
    text.push_str("rec");
    text.push_str(TAB);
    text.push_str(&total_data_bytes.to_string());
    text.push_str(TAB);
    text.push_str(&file_count.to_string());
    text.push_str(TAB);
    text.push('0'); // cluster size (0 for logical)
    text.push_str(CRLF);
}

/// Build the `perm` (permission groups) category.
///
/// Format:
/// ```text
/// perm
/// \tp\tn\t<group_name>
/// \t\tn\t<perm_name>\ts\t<sid>\tpr\t<bitmask>
/// ```
fn build_perm_category(text: &mut String, groups: &[LefPermissionGroup]) {
    text.push_str("perm");
    text.push_str(CRLF);

    for group in groups {
        // Parent group node
        text.push_str(TAB);
        text.push_str("p");
        text.push_str(TAB);
        text.push('1'); // is_parent = true
        text.push_str(TAB);
        text.push_str("n");
        text.push_str(TAB);
        text.push_str(&escape_ltree_value(&group.name));
        text.push_str(CRLF);

        // Child permission entries
        for perm in &group.permissions {
            text.push_str(TAB);
            text.push_str(TAB);
            text.push_str("n");
            text.push_str(TAB);
            text.push_str(&escape_ltree_value(&perm.name));
            text.push_str(TAB);
            text.push_str("s");
            text.push_str(TAB);
            text.push_str(&escape_ltree_value(&perm.sid));
            text.push_str(TAB);
            text.push_str("pr");
            text.push_str(TAB);
            text.push_str(&perm.permissions_bitmask.to_string());
            text.push_str(CRLF);
        }
    }
}

/// Build the `srce` (source) category.
///
/// Format:
/// ```text
/// srce
/// \tp\t1\tn\t<source_name>\tid\t<id>\tev\t<evidence_number>\t...
/// ```
fn build_srce_category(text: &mut String, sources: &[LefSource]) {
    text.push_str("srce");
    text.push_str(CRLF);

    for source in sources {
        text.push_str(TAB);
        text.push_str("p");
        text.push_str(TAB);
        text.push('1'); // is_parent
        text.push_str(TAB);
        text.push_str("n");
        text.push_str(TAB);
        text.push_str(&escape_ltree_value(&source.name));
        text.push_str(TAB);
        text.push_str("id");
        text.push_str(TAB);
        text.push_str(&source.identifier.to_string());

        // Optional fields — only include if non-empty
        if !source.evidence_number.is_empty() {
            text.push_str(TAB);
            text.push_str("ev");
            text.push_str(TAB);
            text.push_str(&escape_ltree_value(&source.evidence_number));
        }
        if !source.location.is_empty() {
            text.push_str(TAB);
            text.push_str("lo");
            text.push_str(TAB);
            text.push_str(&escape_ltree_value(&source.location));
        }
        if !source.device_guid.is_empty() {
            text.push_str(TAB);
            text.push_str("gu");
            text.push_str(TAB);
            text.push_str(&escape_ltree_value(&source.device_guid));
        }
        if !source.drive_type.is_empty() {
            text.push_str(TAB);
            text.push_str("dt");
            text.push_str(TAB);
            text.push_str(&escape_ltree_value(&source.drive_type));
        }
        if !source.manufacturer.is_empty() {
            text.push_str(TAB);
            text.push_str("ma");
            text.push_str(TAB);
            text.push_str(&escape_ltree_value(&source.manufacturer));
        }
        if !source.model.is_empty() {
            text.push_str(TAB);
            text.push_str("mo");
            text.push_str(TAB);
            text.push_str(&escape_ltree_value(&source.model));
        }
        if !source.serial_number.is_empty() {
            text.push_str(TAB);
            text.push_str("sn");
            text.push_str(TAB);
            text.push_str(&escape_ltree_value(&source.serial_number));
        }

        text.push_str(CRLF);
    }
}

/// Build the `sub` (subject) category.
///
/// Format:
/// ```text
/// sub
/// \tp\t1\tn\t<subject_name>\tid\t<id>
/// ```
fn build_sub_category(text: &mut String, subjects: &[LefSubject]) {
    text.push_str("sub");
    text.push_str(CRLF);

    for subject in subjects {
        text.push_str(TAB);
        text.push_str("p");
        text.push_str(TAB);
        text.push('1');
        text.push_str(TAB);
        text.push_str("n");
        text.push_str(TAB);
        text.push_str(&escape_ltree_value(&subject.name));
        text.push_str(TAB);
        text.push_str("id");
        text.push_str(TAB);
        text.push_str(&subject.identifier.to_string());
        text.push_str(CRLF);
    }
}

/// Build the `entry` (file tree) category.
///
/// Files and directories are arranged hierarchically.
/// The indentation depth reflects parent-child relationships.
///
/// Format for a directory:
/// ```text
/// \tp\t1\tn\t<name>\tid\t<id>\tcid\t1
/// ```
///
/// Format for a file:
/// ```text
/// \t\tn\t<name>\tid\t<id>\tls\t<size>\tbe\t<offset> <size>\tcr\t<creation_time>\t...
/// ```
fn build_entry_category(text: &mut String, entries: &[LefFileEntry]) {
    text.push_str("entry");
    text.push_str(CRLF);

    // Build a simple tree structure: group entries by parent_identifier
    // Root entries have parent_identifier == 0
    write_entry_children(text, entries, 0, 1);
}

/// Recursively write child entries at the given indentation depth.
fn write_entry_children(
    text: &mut String,
    entries: &[LefFileEntry],
    parent_id: u64,
    depth: usize,
) {
    // Collect children of this parent, preserving order
    let children: Vec<&LefFileEntry> = entries
        .iter()
        .filter(|e| e.parent_identifier == parent_id)
        .collect();

    for entry in children {
        // Write indentation (tabs)
        for _ in 0..depth {
            text.push_str(TAB);
        }

        if entry.is_parent || entry.record_type == LefRecordType::Directory {
            // Directory entry
            write_directory_entry(text, entry);
            text.push_str(CRLF);

            // Recurse into children
            write_entry_children(text, entries, entry.identifier, depth + 1);
        } else {
            // File entry
            write_file_entry(text, entry);
            text.push_str(CRLF);
        }
    }
}

/// Write a single directory entry line (without leading tabs or trailing CRLF).
fn write_directory_entry(text: &mut String, entry: &LefFileEntry) {
    // p=1 (is parent), n=name, id=identifier, cid=1 (directory)
    text.push_str("p");
    text.push_str(TAB);
    text.push('1');
    text.push_str(TAB);
    text.push_str("n");
    text.push_str(TAB);
    text.push_str(&escape_ltree_value(&entry.name));
    text.push_str(TAB);
    text.push_str("mid");
    text.push_str(TAB);
    text.push_str(&escape_ltree_value(&entry.guid));
    text.push_str(TAB);
    text.push_str("id");
    text.push_str(TAB);
    text.push_str(&entry.identifier.to_string());
    text.push_str(TAB);
    text.push_str("cid");
    text.push_str(TAB);
    text.push('1'); // directory record type

    // Timestamps
    write_timestamps(text, entry);

    // Source/subject
    write_source_fields(text, entry);
}

/// Write a single file entry line (without leading tabs or trailing CRLF).
fn write_file_entry(text: &mut String, entry: &LefFileEntry) {
    // n=name
    text.push_str("n");
    text.push_str(TAB);
    text.push_str(&escape_ltree_value(&entry.name));
    text.push_str(TAB);

    // mid=GUID
    text.push_str("mid");
    text.push_str(TAB);
    text.push_str(&escape_ltree_value(&entry.guid));
    text.push_str(TAB);

    // id=identifier
    text.push_str("id");
    text.push_str(TAB);
    text.push_str(&entry.identifier.to_string());
    text.push_str(TAB);

    // ls=logical size
    text.push_str("ls");
    text.push_str(TAB);
    text.push_str(&entry.size.to_string());

    // be=binary extents (data offset + data size)
    let extents = entry.binary_extents();
    if !extents.is_empty() {
        text.push_str(TAB);
        text.push_str("be");
        text.push_str(TAB);
        text.push_str(&extents);
    }

    // cid=0 (file record type)
    text.push_str(TAB);
    text.push_str("cid");
    text.push_str(TAB);
    text.push('0');

    // Timestamps
    write_timestamps(text, entry);

    // Hashes
    if let Some(ref md5) = entry.md5_hash {
        text.push_str(TAB);
        text.push_str("ha");
        text.push_str(TAB);
        text.push_str(md5);
    }
    if let Some(ref sha1) = entry.sha1_hash {
        text.push_str(TAB);
        text.push_str("sha");
        text.push_str(TAB);
        text.push_str(sha1);
    }

    // Short name
    if let Some(ref short_name) = entry.short_name {
        text.push_str(TAB);
        text.push_str("snh");
        text.push_str(TAB);
        text.push_str(&escape_ltree_value(short_name));
    }

    // Duplicate offset
    if let Some(dup_offset) = entry.duplicate_data_offset {
        text.push_str(TAB);
        text.push_str("du");
        text.push_str(TAB);
        text.push_str(&dup_offset.to_string());
    }

    // Logical/physical offsets (only if non-zero)
    if entry.logical_offset != 0 {
        text.push_str(TAB);
        text.push_str("lo");
        text.push_str(TAB);
        text.push_str(&entry.logical_offset.to_string());
    }
    if entry.physical_offset != 0 {
        text.push_str(TAB);
        text.push_str("po");
        text.push_str(TAB);
        text.push_str(&entry.physical_offset.to_string());
    }

    // Source/subject
    write_source_fields(text, entry);

    // Permission group
    if entry.permission_group_index >= 0 {
        text.push_str(TAB);
        text.push_str("pm");
        text.push_str(TAB);
        text.push_str(&entry.permission_group_index.to_string());
    }

    // Extended attributes
    if let Some(ref ea) = entry.extended_attributes {
        text.push_str(TAB);
        text.push_str("ea");
        text.push_str(TAB);
        text.push_str(&escape_ltree_value(ea));
    }
}

/// Write timestamp fields for an entry.
fn write_timestamps(text: &mut String, entry: &LefFileEntry) {
    if entry.creation_time != 0 {
        text.push_str(TAB);
        text.push_str("cr");
        text.push_str(TAB);
        text.push_str(&entry.creation_time.to_string());
    }
    if entry.access_time != 0 {
        text.push_str(TAB);
        text.push_str("ac");
        text.push_str(TAB);
        text.push_str(&entry.access_time.to_string());
    }
    if entry.modification_time != 0 {
        text.push_str(TAB);
        text.push_str("wr");
        text.push_str(TAB);
        text.push_str(&entry.modification_time.to_string());
    }
    if entry.entry_modification_time != 0 {
        text.push_str(TAB);
        text.push_str("mo");
        text.push_str(TAB);
        text.push_str(&entry.entry_modification_time.to_string());
    }
    if entry.deletion_time != 0 {
        text.push_str(TAB);
        text.push_str("dl");
        text.push_str(TAB);
        text.push_str(&entry.deletion_time.to_string());
    }
}

/// Write source/subject identifier fields.
fn write_source_fields(text: &mut String, entry: &LefFileEntry) {
    if entry.source_identifier != 0 {
        text.push_str(TAB);
        text.push_str("src");
        text.push_str(TAB);
        text.push_str(&entry.source_identifier.to_string());
    }
    if entry.subject_identifier != 0 {
        text.push_str(TAB);
        text.push_str("sub");
        text.push_str(TAB);
        text.push_str(&entry.subject_identifier.to_string());
    }
}

/// Escape special characters in ltree values.
///
/// Tabs, carriage returns, and line feeds in values must be escaped
/// to avoid breaking the tab-separated format.
fn escape_ltree_value(value: &str) -> String {
    // In EnCase's ltree format, special characters in values are
    // typically handled by the application layer. For safety, we
    // replace tabs and newlines with spaces.
    value
        .replace('\t', " ")
        .replace('\r', "")
        .replace('\n', " ")
}

/// Convert a UTF-8 string to UTF-16LE bytes (for ltree section data).
pub fn utf8_to_utf16le(text: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(text.len() * 2 + 2);

    // BOM (Byte Order Mark) for UTF-16LE
    bytes.push(0xFF);
    bytes.push(0xFE);

    for code_unit in text.encode_utf16() {
        bytes.push((code_unit & 0xFF) as u8);
        bytes.push((code_unit >> 8) as u8);
    }

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l01_writer::types::*;

    #[test]
    fn test_utf8_to_utf16le_empty() {
        let result = utf8_to_utf16le("");
        // Just the BOM
        assert_eq!(result, vec![0xFF, 0xFE]);
    }

    #[test]
    fn test_utf8_to_utf16le_ascii() {
        let result = utf8_to_utf16le("AB");
        // BOM + 'A'(0x0041) + 'B'(0x0042) in LE
        assert_eq!(result, vec![0xFF, 0xFE, 0x41, 0x00, 0x42, 0x00]);
    }

    #[test]
    fn test_utf8_to_utf16le_unicode() {
        let result = utf8_to_utf16le("日");
        // BOM + U+65E5 in LE = 0xE5, 0x65
        assert_eq!(result, vec![0xFF, 0xFE, 0xE5, 0x65]);
    }

    #[test]
    fn test_escape_ltree_value() {
        assert_eq!(escape_ltree_value("hello"), "hello");
        assert_eq!(escape_ltree_value("hello\tworld"), "hello world");
        assert_eq!(escape_ltree_value("line1\nline2"), "line1 line2");
        assert_eq!(escape_ltree_value("cr\rlf\n"), "crlf ");
    }

    #[test]
    fn test_build_rec_category() {
        let entries = vec![
            LefFileEntry::new_file(1, "a.txt".into(), 100),
            LefFileEntry::new_file(2, "b.txt".into(), 200),
            LefFileEntry::new_directory(3, "dir".into()),
        ];
        let mut text = String::new();
        build_rec_category(&mut text, &entries, 300);
        assert!(text.starts_with("rec\t300\t2\t0\r\n"));
    }

    #[test]
    fn test_build_entry_category_simple() {
        let entries = vec![
            LefFileEntry::new_file(1, "test.txt".into(), 1024),
        ];
        let mut text = String::new();
        build_entry_category(&mut text, &entries);

        assert!(text.starts_with("entry\r\n"));
        assert!(text.contains("n\ttest.txt"));
        assert!(text.contains("ls\t1024"));
        assert!(text.contains("cid\t0"));
    }

    #[test]
    fn test_build_entry_category_hierarchy() {
        let dir = LefFileEntry::new_directory(1, "Documents".into());
        let file = LefFileEntry::new_file(2, "readme.txt".into(), 512)
            .with_parent(1);

        let entries = vec![dir, file];
        let mut text = String::new();
        build_entry_category(&mut text, &entries);

        // Directory should appear before file
        let dir_pos = text.find("Documents").unwrap();
        let file_pos = text.find("readme.txt").unwrap();
        assert!(dir_pos < file_pos);

        // File should have deeper indentation (2 tabs vs 1 tab)
        assert!(text.contains("\t\tn\treadme.txt"));
    }

    #[test]
    fn test_build_srce_category() {
        let sources = vec![LefSource::new(1, "Evidence Drive".into())];
        let mut text = String::new();
        build_srce_category(&mut text, &sources);

        assert!(text.starts_with("srce\r\n"));
        assert!(text.contains("n\tEvidence Drive"));
        assert!(text.contains("id\t1"));
    }

    #[test]
    fn test_build_sub_category() {
        let subjects = vec![LefSubject {
            identifier: 1,
            name: "User Account".into(),
        }];
        let mut text = String::new();
        build_sub_category(&mut text, &subjects);

        assert!(text.starts_with("sub\r\n"));
        assert!(text.contains("n\tUser Account"));
    }

    #[test]
    fn test_build_complete_ltree() {
        let entries = vec![
            LefFileEntry::new_directory(1, "Root".into()),
            LefFileEntry::new_file(2, "file.txt".into(), 100).with_parent(1),
        ];
        let sources = vec![LefSource::new(1, "Source".into())];
        let groups = vec![];
        let subjects = vec![];

        let text = build_ltree_text(&entries, &sources, &groups, &subjects, 100);

        // Should contain all 5 categories
        assert!(text.contains("rec\t"));
        assert!(text.contains("perm\r\n"));
        assert!(text.contains("srce\r\n"));
        assert!(text.contains("sub\r\n"));
        assert!(text.contains("entry\r\n"));
    }

    #[test]
    fn test_file_entry_with_hashes() {
        let mut entry = LefFileEntry::new_file(1, "hashed.bin".into(), 256);
        entry.md5_hash = Some("d41d8cd98f00b204e9800998ecf8427e".into());
        entry.sha1_hash = Some("da39a3ee5e6b4b0d3255bfef95601890afd80709".into());

        let mut text = String::new();
        write_file_entry(&mut text, &entry);

        assert!(text.contains("ha\td41d8cd98f00b204e9800998ecf8427e"));
        assert!(text.contains("sha\tda39a3ee5e6b4b0d3255bfef95601890afd80709"));
    }
}
