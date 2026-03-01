// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Pure-Rust L01 (Logical Evidence File) writer.
//!
//! Creates EnCase-compatible L01 logical evidence containers from
//! source files on disk. Supports:
//!
//! - Hierarchical file/directory structures
//! - zlib chunk compression (none / fast / best)
//! - MD5 and SHA-1 image integrity hashes
//! - Per-file MD5/SHA-1 hashes
//! - Multi-segment output (.L01, .L02, ... .L99, .LAA, ...)
//! - Case metadata (case number, evidence number, examiner, etc.)
//! - Source tracking and permission groups
//! - Cancellation support via `AtomicBool`
//!
//! # Architecture
//!
//! ```text
//! L01Writer
//!   ├── types.rs     — Data types, constants, Adler-32
//!   ├── ltree.rs     — UTF-16LE ltree text builder (5 categories)
//!   ├── chunks.rs    — zlib chunk compression
//!   ├── sections.rs  — EWF v1 section writers
//!   └── segment.rs   — Multi-segment file support
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use ffx_check_lib::l01_writer::{L01Writer, L01WriterConfig, L01CaseInfo};
//! use std::path::PathBuf;
//!
//! let config = L01WriterConfig {
//!     output_path: PathBuf::from("/output/evidence.L01"),
//!     case_info: L01CaseInfo {
//!         case_number: "2024-001".into(),
//!         evidence_number: "E001".into(),
//!         description: "Logical acquisition".into(),
//!         examiner: "Forensic Examiner".into(),
//!         notes: String::new(),
//!     },
//!     ..Default::default()
//! };
//!
//! let mut writer = L01Writer::new(config);
//! writer.add_source_directory(&PathBuf::from("/evidence/files")).unwrap();
//! let result = writer.write(None, None).unwrap();
//! ```

pub mod chunks;
pub mod ltree;
pub mod sections;
pub mod segment;
pub mod types;
mod writer;

pub use types::*;

use std::path::{Path, PathBuf};

/// Pure-Rust L01 logical evidence file writer.
///
/// Build a file tree with `add_file()` / `add_directory()` / `add_source_directory()`,
/// then call `write()` to produce the L01 output.
pub struct L01Writer {
    pub(super) config: L01WriterConfig,
    pub(super) entries: Vec<LefFileEntry>,
    pub(super) sources: Vec<LefSource>,
    pub(super) permission_groups: Vec<LefPermissionGroup>,
    pub(super) subjects: Vec<LefSubject>,
    next_identifier: u64,
}

impl L01Writer {
    /// Create a new L01 writer with the given configuration.
    pub fn new(config: L01WriterConfig) -> Self {
        Self {
            config,
            entries: Vec::new(),
            sources: Vec::new(),
            permission_groups: Vec::new(),
            subjects: Vec::new(),
            next_identifier: 1,
        }
    }

    /// Allocate and return the next unique entry identifier.
    fn alloc_id(&mut self) -> u64 {
        let id = self.next_identifier;
        self.next_identifier += 1;
        id
    }

    // ── File/Directory Registration ─────────────────────────────────────

    /// Add a single file entry.
    ///
    /// Returns the assigned identifier.
    pub fn add_file(
        &mut self,
        name: String,
        size: u64,
        source_path: PathBuf,
        parent_id: u64,
    ) -> u64 {
        let id = self.alloc_id();
        let entry = LefFileEntry::new_file(id, name, size)
            .with_parent(parent_id)
            .with_source_path(source_path);
        self.entries.push(entry);
        id
    }

    /// Add a directory entry.
    ///
    /// Returns the assigned identifier.
    pub fn add_directory(&mut self, name: String, parent_id: u64) -> u64 {
        let id = self.alloc_id();
        let entry = LefFileEntry::new_directory(id, name).with_parent(parent_id);
        self.entries.push(entry);
        id
    }

    /// Add a pre-built file entry directly.
    pub fn add_entry(&mut self, mut entry: LefFileEntry) {
        if entry.identifier == 0 {
            entry.identifier = self.alloc_id();
        } else if entry.identifier >= self.next_identifier {
            self.next_identifier = entry.identifier + 1;
        }
        self.entries.push(entry);
    }

    /// Add an acquisition source.
    pub fn add_source(&mut self, source: LefSource) {
        self.sources.push(source);
    }

    /// Add a permission group.
    pub fn add_permission_group(&mut self, group: LefPermissionGroup) {
        self.permission_groups.push(group);
    }

    /// Add a subject.
    pub fn add_subject(&mut self, subject: LefSubject) {
        self.subjects.push(subject);
    }

    /// Recursively add all files and directories from a source directory.
    ///
    /// Preserves the directory hierarchy. Sets timestamps from filesystem
    /// metadata. Returns the number of entries added.
    pub fn add_source_directory(&mut self, dir_path: &Path) -> Result<usize, L01WriteError> {
        if !dir_path.exists() {
            return Err(L01WriteError::FileNotFound(
                dir_path.to_string_lossy().to_string(),
            ));
        }

        // Add a source for this directory
        let source_id = if self.sources.is_empty() {
            let sid = self.sources.len() as u64 + 1;
            self.sources.push(LefSource::from_path(sid, dir_path));
            sid
        } else {
            self.sources[0].identifier
        };

        let count = self.walk_directory(dir_path, 0, source_id)?;
        Ok(count)
    }

    /// Recursively walk a directory, adding entries.
    fn walk_directory(
        &mut self,
        dir_path: &Path,
        parent_id: u64,
        source_id: u64,
    ) -> Result<usize, L01WriteError> {
        use writer::set_timestamps_from_metadata;

        let mut count = 0;

        let mut dir_entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir_path)
            .map_err(|e| L01WriteError::SourceReadError {
                path: dir_path.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?
            .filter_map(|e| e.ok())
            .collect();

        // Sort for deterministic output
        dir_entries.sort_by_key(|e| e.file_name());

        for dir_entry in dir_entries {
            let path = dir_entry.path();
            let file_name = dir_entry.file_name().to_string_lossy().to_string();

            // Skip hidden files/directories (starting with .)
            if file_name.starts_with('.') {
                continue;
            }

            let metadata = dir_entry
                .metadata()
                .map_err(|e| L01WriteError::SourceReadError {
                    path: path.to_string_lossy().to_string(),
                    reason: e.to_string(),
                })?;

            if metadata.is_dir() {
                let dir_id = self.alloc_id();
                let mut entry = LefFileEntry::new_directory(dir_id, file_name)
                    .with_parent(parent_id)
                    .with_source(source_id)
                    .with_source_path(path.clone());

                // Set timestamps
                set_timestamps_from_metadata(&mut entry, &metadata);

                self.entries.push(entry);
                count += 1;

                // Recurse
                count += self.walk_directory(&path, dir_id, source_id)?;
            } else if metadata.is_file() {
                let file_id = self.alloc_id();
                let size = metadata.len();
                let mut entry = LefFileEntry::new_file(file_id, file_name, size)
                    .with_parent(parent_id)
                    .with_source(source_id)
                    .with_source_path(path);

                set_timestamps_from_metadata(&mut entry, &metadata);

                self.entries.push(entry);
                count += 1;
            }
            // Skip symlinks, special files
        }

        Ok(count)
    }

    /// Get the total number of entries (files + directories).
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Get the total size of all file entries.
    pub fn total_file_size(&self) -> u64 {
        self.entries
            .iter()
            .filter(|e| e.record_type == LefRecordType::File)
            .map(|e| e.size)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tempfile::TempDir;

    fn create_test_files(dir: &Path) {
        std::fs::create_dir_all(dir.join("subdir")).unwrap();
        std::fs::write(dir.join("file1.txt"), b"Hello World").unwrap();
        std::fs::write(dir.join("file2.bin"), vec![0xABu8; 1024]).unwrap();
        std::fs::write(dir.join("subdir/nested.txt"), b"Nested content").unwrap();
    }

    #[test]
    fn test_writer_new() {
        let config = L01WriterConfig::default();
        let writer = L01Writer::new(config);
        assert_eq!(writer.entry_count(), 0);
        assert_eq!(writer.total_file_size(), 0);
    }

    #[test]
    fn test_writer_add_file() {
        let mut writer = L01Writer::new(L01WriterConfig::default());
        let id = writer.add_file("test.txt".into(), 100, PathBuf::from("/tmp/test.txt"), 0);
        assert_eq!(id, 1);
        assert_eq!(writer.entry_count(), 1);
        assert_eq!(writer.total_file_size(), 100);
    }

    #[test]
    fn test_writer_add_directory() {
        let mut writer = L01Writer::new(L01WriterConfig::default());
        let dir_id = writer.add_directory("docs".into(), 0);
        let file_id = writer.add_file(
            "readme.txt".into(),
            50,
            PathBuf::from("/tmp/readme.txt"),
            dir_id,
        );

        assert_eq!(dir_id, 1);
        assert_eq!(file_id, 2);
        assert_eq!(writer.entry_count(), 2);
        assert_eq!(writer.total_file_size(), 50);
    }

    #[test]
    fn test_writer_add_source_directory() {
        let tmp = TempDir::new().unwrap();
        create_test_files(tmp.path());

        let mut writer = L01Writer::new(L01WriterConfig::default());
        let count = writer.add_source_directory(tmp.path()).unwrap();

        // Should find: subdir, file1.txt, file2.bin, subdir/nested.txt
        assert_eq!(count, 4);
        assert_eq!(writer.entry_count(), 4);
    }

    #[test]
    fn test_writer_add_source_directory_not_found() {
        let mut writer = L01Writer::new(L01WriterConfig::default());
        let result = writer.add_source_directory(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_writer_validation_no_output() {
        let mut writer = L01Writer::new(L01WriterConfig::default());
        writer.add_file("test.txt".into(), 100, PathBuf::from("/tmp/test.txt"), 0);

        let result = writer.write(None, None);
        assert!(matches!(result, Err(L01WriteError::NoOutputPath)));
    }

    #[test]
    fn test_writer_validation_no_files() {
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: PathBuf::from("/tmp/test.L01"),
            ..Default::default()
        });

        let result = writer.write(None, None);
        assert!(matches!(result, Err(L01WriteError::NoFiles)));
    }

    #[test]
    fn test_writer_validation_segment_too_small() {
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: PathBuf::from("/tmp/test.L01"),
            segment_size: 100, // Too small
            ..Default::default()
        });
        writer.add_file("test.txt".into(), 100, PathBuf::from("/tmp/test.txt"), 0);

        let result = writer.write(None, None);
        assert!(matches!(result, Err(L01WriteError::SegmentTooSmall(_))));
    }

    #[test]
    fn test_write_complete_l01() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");
        create_test_files(&source_dir);

        let output_path = tmp.path().join("output.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            case_info: L01CaseInfo {
                case_number: "TEST-001".into(),
                evidence_number: "E001".into(),
                description: "Test L01 output".into(),
                examiner: "Test Examiner".into(),
                notes: "Integration test".into(),
            },
            ..Default::default()
        });

        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();

        // Verify result
        assert_eq!(result.output_paths.len(), 1);
        assert_eq!(result.total_files, 3); // file1.txt, file2.bin, nested.txt
        assert_eq!(result.total_directories, 1); // subdir
        assert!(result.total_data_bytes > 0);
        assert!(result.md5_hash.is_some());
        assert!(result.sha1_hash.is_some());
        assert_eq!(result.segment_count, 1);
        assert!(result.chunk_count > 0);

        // Verify the file exists and has LVF signature
        assert!(output_path.exists());
        let written = std::fs::read(&output_path).unwrap();
        assert_eq!(&written[0..3], b"LVF");
        assert_eq!(&written[0..8], LVF_SIGNATURE);

        // File should be non-trivially sized
        assert!(written.len() > FILE_HEADER_SIZE + SECTION_HEADER_SIZE);
    }

    #[test]
    fn test_write_with_cancellation() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");
        create_test_files(&source_dir);

        let output_path = tmp.path().join("output.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let cancel = AtomicBool::new(true); // Cancel immediately
        let result = writer.write(Some(&cancel), None);
        assert!(matches!(result, Err(L01WriteError::Cancelled)));
    }

    #[test]
    fn test_write_with_progress() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");
        create_test_files(&source_dir);

        let output_path = tmp.path().join("output.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            case_info: L01CaseInfo {
                case_number: "PROG-001".into(),
                ..Default::default()
            },
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let progress_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let pc = progress_count.clone();

        let result = writer.write(
            None,
            Some(Box::new(move |_progress| {
                pc.fetch_add(1, Ordering::Relaxed);
            })),
        );

        assert!(result.is_ok());
        assert!(progress_count.load(Ordering::Relaxed) > 0);
    }

    #[test]
    fn test_write_no_compression() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");
        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::write(source_dir.join("data.bin"), vec![0x42u8; 512]).unwrap();

        let output_path = tmp.path().join("nocomp.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            compression_level: CompressionLevel::None,
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();
        assert!(result.total_compressed_bytes > 0);
        assert!(output_path.exists());
    }

    #[test]
    fn test_segment_extension() {
        assert_eq!(segment::segment_extension(1), "L01");
        assert_eq!(segment::segment_extension(50), "L50");
        assert_eq!(segment::segment_extension(100), "LAA");
    }

    // ── Multi-segment write tests ──

    /// Helper to create source files with pseudorandom data that resists
    /// zlib compression. Uses a simple PRNG to produce near-uniform bytes.
    fn create_large_test_files(dir: &Path, count: usize, each_size: usize) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        std::fs::create_dir_all(dir).unwrap();
        for i in 0..count {
            let mut data = vec![0u8; each_size];
            // Simple LCG PRNG seeded by file index
            let mut state: u64 = {
                let mut h = DefaultHasher::new();
                i.hash(&mut h);
                h.finish()
            };
            for byte in data.iter_mut() {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                *byte = (state >> 33) as u8;
            }
            std::fs::write(dir.join(format!("file{}.bin", i)), &data).unwrap();
        }
    }

    /// Minimum segment size constant for tests (mirrors production value)
    const MIN_SEGMENT: u64 = 1_048_576;

    #[test]
    fn test_write_multi_segment_creates_multiple_files() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");

        // 4 files × 768 KB = 3 MB of incompressible data → at min segment (1 MB) → ≥2 segments
        create_large_test_files(&source_dir, 4, 768 * 1024);

        let output_path = tmp.path().join("output.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            segment_size: MIN_SEGMENT,
            case_info: L01CaseInfo {
                case_number: "SPLIT-001".into(),
                ..Default::default()
            },
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();

        // Should produce multiple segments
        assert!(
            result.segment_count > 1,
            "Expected multiple segments, got {}",
            result.segment_count
        );
        assert_eq!(result.output_paths.len(), result.segment_count as usize);

        // All segment files should exist
        for path in &result.output_paths {
            assert!(
                std::path::Path::new(path).exists(),
                "Segment file {} doesn't exist",
                path
            );
        }

        // First segment should have L01 extension
        assert!(
            result.output_paths[0].ends_with(".L01"),
            "First segment should have .L01 extension: {}",
            result.output_paths[0]
        );

        // All segments should have LVF signature
        for path in &result.output_paths {
            let data = std::fs::read(path).unwrap();
            assert_eq!(
                &data[0..8],
                LVF_SIGNATURE,
                "Segment {} missing LVF signature",
                path
            );
        }
    }

    #[test]
    fn test_write_multi_segment_sequential_extensions() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");

        // 8 files × 768 KB = 6 MB → at min segment → ≥3 segments
        create_large_test_files(&source_dir, 8, 768 * 1024);

        let output_path = tmp.path().join("multi.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            segment_size: MIN_SEGMENT,
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();

        assert!(
            result.segment_count >= 3,
            "Expected at least 3 segments, got {}",
            result.segment_count
        );

        // Verify sequential extension naming
        for (idx, path) in result.output_paths.iter().enumerate() {
            let expected_ext = segment::segment_extension((idx + 1) as u16);
            assert!(
                path.ends_with(&format!(".{}", expected_ext)),
                "Segment {} should end with .{}, got {}",
                idx,
                expected_ext,
                path
            );
        }
    }

    #[test]
    fn test_write_multi_segment_preserves_hashes() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");

        // 4 files × 768 KB = 3 MB
        create_large_test_files(&source_dir, 4, 768 * 1024);

        // Single-segment write
        let single_output = tmp.path().join("single.L01");
        let mut single_writer = L01Writer::new(L01WriterConfig {
            output_path: single_output.clone(),
            ..Default::default()
        });
        single_writer.add_source_directory(&source_dir).unwrap();
        let single_result = single_writer.write(None, None).unwrap();

        // Multi-segment write of the same data
        let multi_output = tmp.path().join("multi.L01");
        let mut multi_writer = L01Writer::new(L01WriterConfig {
            output_path: multi_output.clone(),
            segment_size: MIN_SEGMENT,
            ..Default::default()
        });
        multi_writer.add_source_directory(&source_dir).unwrap();
        let multi_result = multi_writer.write(None, None).unwrap();

        // Both should have the same data hashes (content is identical)
        assert_eq!(single_result.md5_hash, multi_result.md5_hash);
        assert_eq!(single_result.sha1_hash, multi_result.sha1_hash);
        assert_eq!(single_result.total_files, multi_result.total_files);
        assert_eq!(
            single_result.total_data_bytes,
            multi_result.total_data_bytes
        );
    }

    #[test]
    fn test_write_segment_size_zero_means_single() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");
        create_test_files(&source_dir);

        let output_path = tmp.path().join("single.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            segment_size: 0, // Zero means no splitting
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();

        assert_eq!(result.segment_count, 1);
        assert_eq!(result.output_paths.len(), 1);
    }

    #[test]
    fn test_write_multi_segment_segment_number_in_header() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");

        // 4 files × 768 KB = 3 MB → ≥2 segments at 1 MB
        create_large_test_files(&source_dir, 4, 768 * 1024);

        let output_path = tmp.path().join("segtest.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            segment_size: MIN_SEGMENT,
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();

        // Each segment file header should contain the correct segment number
        // LVF file header: bytes 0-7 = LVF signature, byte 8 = 0x01,
        // bytes 9-10 = segment number (u16 LE), bytes 11-12 = fields end
        for (idx, path) in result.output_paths.iter().enumerate() {
            let data = std::fs::read(path).unwrap();
            assert!(data.len() > 13, "Segment {} too small", idx);

            // Check segment number at offset 9 (2 bytes LE)
            let seg_num = u16::from_le_bytes([data[9], data[10]]);
            assert_eq!(
                seg_num,
                (idx + 1) as u16,
                "Segment {} has wrong segment number in header: expected {}, got {}",
                idx,
                idx + 1,
                seg_num
            );
        }
    }

    #[test]
    fn test_write_multi_segment_last_has_done_section() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");

        // 4 files × 768 KB = 3 MB → ≥2 segments at 1 MB
        create_large_test_files(&source_dir, 4, 768 * 1024);

        let output_path = tmp.path().join("done.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            segment_size: MIN_SEGMENT,
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();
        assert!(result.segment_count > 1);

        // Last segment should contain "done" section type somewhere
        let last_path = result.output_paths.last().unwrap();
        let data = std::fs::read(last_path).unwrap();

        // Search for "done" section type in section headers (16 bytes type field, null-padded)
        let done_marker = b"done\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let found_done = data.windows(16).any(|w| w == done_marker);
        assert!(found_done, "Last segment should contain 'done' section");

        // Non-last segments should contain "next" section
        if result.segment_count > 1 {
            let first_path = &result.output_paths[0];
            let first_data = std::fs::read(first_path).unwrap();
            let next_marker = b"next\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
            let found_next = first_data.windows(16).any(|w| w == next_marker);
            assert!(found_next, "Non-last segment should contain 'next' section");
        }
    }

    #[test]
    fn test_write_multi_segment_total_data_correct() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");

        let file_sizes: Vec<usize> = vec![200_000, 400_000, 300_000, 150_000, 250_000];
        std::fs::create_dir_all(&source_dir).unwrap();
        for (i, &size) in file_sizes.iter().enumerate() {
            // Pseudorandom data for each file
            let data: Vec<u8> = (0..size)
                .map(|j| ((j as u64 * 31 + i as u64 * 97) % 256) as u8)
                .collect();
            std::fs::write(source_dir.join(format!("f{}.bin", i)), &data).unwrap();
        }

        let output_path = tmp.path().join("totalcheck.L01");
        let mut writer = L01Writer::new(L01WriterConfig {
            output_path: output_path.clone(),
            segment_size: MIN_SEGMENT,
            ..Default::default()
        });
        writer.add_source_directory(&source_dir).unwrap();

        let result = writer.write(None, None).unwrap();

        let expected_total: u64 = file_sizes.iter().map(|&s| s as u64).sum();
        assert_eq!(
            result.total_data_bytes, expected_total,
            "Total data bytes mismatch"
        );
        assert_eq!(result.total_files, 5);
        assert!(result.chunk_count > 0);
    }
}
