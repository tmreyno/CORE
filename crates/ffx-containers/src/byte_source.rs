// =============================================================================
// ffx-containers — Unified Container Abstraction Layer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Evidence byte-source adapters for container entries.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{ad1, archive, ewf, raw, ufed};
use ffx_common::{
    bounded_read_size, EvidenceByteSource, EvidenceSourceError, EvidenceSourceRef,
    EvidenceSourceResult, VfsEntryByteSource,
};

/// Open a readable byte source for a container entry.
///
/// This is the format-dispatch bridge used by higher-level engines. Callers
/// should not need to know whether the entry is coming from AD1 metadata,
/// L01 logical data offsets, libarchive, or a disk-image VFS.
pub fn open_container_entry_source(
    container_path: impl Into<String>,
    entry_path: impl Into<String>,
    container_type: impl Into<String>,
    known_size: Option<u64>,
) -> EvidenceSourceResult<Box<dyn EvidenceByteSource>> {
    let container_path = container_path.into();
    let entry_path = entry_path.into();
    let container_type = container_type.into();
    let kind = container_type.to_lowercase();

    if is_ad1_type(&kind) {
        return Ok(Box::new(Ad1EntryByteSource::new(
            container_path,
            entry_path,
            known_size,
        )));
    }

    if is_l01_type(&kind) {
        return Ok(Box::new(L01EntryByteSource::new(
            container_path,
            entry_path,
            known_size,
        )));
    }

    if is_archive_type(&kind) {
        let vfs = archive::ArchiveVfs::open(&container_path).map_err(|e| {
            source_error(
                &container_path,
                &entry_path,
                &container_type,
                format!("Archive VFS open failed: {e}"),
            )
        })?;
        return Ok(Box::new(VfsEntryByteSource::new(
            Arc::new(vfs),
            container_path,
            entry_path,
            Some(container_type),
        )));
    }

    if is_ufed_type(&kind) {
        return Ok(Box::new(UfedEntryByteSource::new(
            container_path,
            entry_path,
            known_size,
        )));
    }

    if is_ewf_type(&kind) {
        let vfs = ewf::vfs::EwfVfs::open(&container_path).map_err(|e| {
            source_error(
                &container_path,
                &entry_path,
                &container_type,
                format!("EWF VFS open failed: {e:?}"),
            )
        })?;
        return Ok(Box::new(VfsEntryByteSource::new(
            Arc::new(vfs),
            container_path,
            entry_path,
            Some(container_type),
        )));
    }

    if is_raw_type(&kind) {
        let vfs = raw::vfs::RawVfs::open_filesystem(&container_path)
            .or_else(|_| raw::vfs::RawVfs::open(&container_path))
            .map_err(|e| {
                source_error(
                    &container_path,
                    &entry_path,
                    &container_type,
                    format!("Raw VFS open failed: {e:?}"),
                )
            })?;
        return Ok(Box::new(VfsEntryByteSource::new(
            Arc::new(vfs),
            container_path,
            entry_path,
            Some(container_type),
        )));
    }

    Err(source_error(
        &container_path,
        &entry_path,
        &container_type,
        format!("Unsupported container entry source type: {container_type}"),
    ))
}

#[derive(Clone, Debug)]
pub struct Ad1EntryByteSource {
    container_path: String,
    entry_path: String,
    known_size: Option<u64>,
}

impl Ad1EntryByteSource {
    pub fn new(container_path: String, entry_path: String, known_size: Option<u64>) -> Self {
        Self {
            container_path,
            entry_path,
            known_size,
        }
    }
}

impl EvidenceByteSource for Ad1EntryByteSource {
    fn source_ref(&self) -> EvidenceSourceRef {
        container_entry_ref(&self.container_path, &self.entry_path, "ad1")
    }

    fn len(&self) -> EvidenceSourceResult<u64> {
        if let Some(size) = self.known_size {
            return Ok(size);
        }

        ad1::get_entry_info(&self.container_path, &self.entry_path)
            .map(|entry| entry.size)
            .map_err(|e| source_error(&self.container_path, &self.entry_path, "ad1", e.to_string()))
    }

    fn read_range(&self, offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>> {
        let total_size = self.len()?;
        let read_size = bounded_read_size(&self.source_ref(), total_size, offset, size)?;
        if read_size == 0 {
            return Ok(Vec::new());
        }

        ad1::read_entry_chunk(&self.container_path, &self.entry_path, offset, read_size)
            .map_err(|e| source_error(&self.container_path, &self.entry_path, "ad1", e.to_string()))
    }
}

#[derive(Clone, Debug)]
pub struct L01EntryByteSource {
    container_path: String,
    entry_path: String,
    known_size: Option<u64>,
}

impl L01EntryByteSource {
    pub fn new(container_path: String, entry_path: String, known_size: Option<u64>) -> Self {
        Self {
            container_path,
            entry_path,
            known_size,
        }
    }

    fn entry(&self) -> EvidenceSourceResult<ewf::L01Entry> {
        let tree = ewf::parse_l01_file_tree(&self.container_path).map_err(|e| {
            source_error(&self.container_path, &self.entry_path, "l01", e.to_string())
        })?;
        tree.entry_at_path(&self.entry_path)
            .cloned()
            .ok_or_else(|| {
                source_error(
                    &self.container_path,
                    &self.entry_path,
                    "l01",
                    format!("L01 entry not found: {}", self.entry_path),
                )
            })
    }
}

impl EvidenceByteSource for L01EntryByteSource {
    fn source_ref(&self) -> EvidenceSourceRef {
        container_entry_ref(&self.container_path, &self.entry_path, "l01")
    }

    fn len(&self) -> EvidenceSourceResult<u64> {
        if let Some(size) = self.known_size {
            return Ok(size);
        }

        let entry = self.entry()?;
        Ok(if entry.size > 0 {
            entry.size
        } else {
            entry.data_size
        })
    }

    fn read_range(&self, offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>> {
        let entry = self.entry()?;
        let total_size = if entry.size > 0 {
            entry.size
        } else {
            entry.data_size
        };
        let read_size = bounded_read_size(&self.source_ref(), total_size, offset, size)?;
        if read_size == 0 {
            return Ok(Vec::new());
        }

        let mut handle = ewf::EwfHandle::open(&self.container_path).map_err(|e| {
            source_error(&self.container_path, &self.entry_path, "l01", e.to_string())
        })?;
        let read_offset = checked_entry_data_offset(
            &self.container_path,
            &self.entry_path,
            "l01",
            entry.data_offset,
            offset,
        )?;
        handle
            .read_at(read_offset, read_size)
            .map_err(|e| source_error(&self.container_path, &self.entry_path, "l01", e.to_string()))
    }
}

#[derive(Clone, Debug)]
pub struct UfedEntryByteSource {
    container_path: String,
    entry_path: String,
    known_size: Option<u64>,
}

impl UfedEntryByteSource {
    pub fn new(container_path: String, entry_path: String, known_size: Option<u64>) -> Self {
        Self {
            container_path,
            entry_path,
            known_size,
        }
    }

    fn source_id(&self) -> EvidenceSourceRef {
        container_entry_ref(&self.container_path, &self.entry_path, "ufed")
    }

    fn is_zip_backed(&self) -> bool {
        matches!(
            ufed::detect_format(&self.container_path),
            Some(ufed::UfedFormat::UfedZip)
        )
    }

    fn local_entry_path(&self) -> PathBuf {
        local_container_entry_path(&self.container_path, &self.entry_path)
    }

    fn zip_entry_len(&self) -> EvidenceSourceResult<u64> {
        let expected = self.entry_path.trim_start_matches('/').replace('\\', "/");
        ufed::archive_ops::list_zip_entries(&self.container_path)
            .map_err(|e| {
                source_error(
                    &self.container_path,
                    &self.entry_path,
                    "ufed",
                    e.to_string(),
                )
            })?
            .into_iter()
            .find(|entry| {
                let path = entry.path.trim_start_matches('/').replace('\\', "/");
                path == expected
            })
            .map(|entry| entry.size)
            .ok_or_else(|| {
                source_error(
                    &self.container_path,
                    &self.entry_path,
                    "ufed",
                    format!("UFED ZIP entry not found: {}", self.entry_path),
                )
            })
    }

    fn read_zip_range(&self, offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>> {
        let total_size = self.len()?;
        let read_size = bounded_read_size(&self.source_ref(), total_size, offset, size)?;
        if read_size == 0 {
            return Ok(Vec::new());
        }

        ufed::archive_ops::read_archive_file_range(
            &self.container_path,
            self.entry_path.trim_start_matches('/'),
            offset,
            read_size,
        )
        .map_err(|e| {
            source_error(
                &self.container_path,
                &self.entry_path,
                "ufed",
                e.to_string(),
            )
        })
    }

    fn read_local_range(&self, offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>> {
        let total_size = std::fs::metadata(self.local_entry_path())
            .map(|metadata| metadata.len())
            .map_err(|e| {
                source_error(
                    &self.container_path,
                    &self.entry_path,
                    "ufed",
                    format!("UFED local entry metadata failed: {e}"),
                )
            })?;
        let read_size = bounded_read_size(&self.source_ref(), total_size, offset, size)?;
        if read_size == 0 {
            return Ok(Vec::new());
        }

        let mut file = File::open(self.local_entry_path()).map_err(|e| {
            source_error(
                &self.container_path,
                &self.entry_path,
                "ufed",
                format!("UFED local entry open failed: {e}"),
            )
        })?;
        file.seek(SeekFrom::Start(offset)).map_err(|e| {
            source_error(
                &self.container_path,
                &self.entry_path,
                "ufed",
                format!("UFED local entry seek failed: {e}"),
            )
        })?;

        let mut data = vec![0u8; read_size];
        file.read_exact(&mut data).map_err(|e| {
            source_error(
                &self.container_path,
                &self.entry_path,
                "ufed",
                format!("UFED local entry range read failed: {e}"),
            )
        })?;
        Ok(data)
    }
}

fn local_container_entry_path(container_path: &str, entry_path: &str) -> PathBuf {
    let entry = Path::new(entry_path);
    if entry.is_absolute() {
        return entry.to_path_buf();
    }

    let mut resolved = Path::new(container_path)
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .to_path_buf();
    for component in entry_path
        .split(['/', '\\'])
        .filter(|part| !part.is_empty())
    {
        resolved.push(component);
    }
    resolved
}

fn checked_entry_data_offset(
    container_path: &str,
    entry_path: &str,
    container_type: &str,
    data_offset: u64,
    relative_offset: u64,
) -> EvidenceSourceResult<u64> {
    data_offset.checked_add(relative_offset).ok_or_else(|| {
        source_error(
            container_path,
            entry_path,
            container_type,
            format!(
                "Container entry read offset overflow: data offset {data_offset} + requested offset {relative_offset}"
            ),
        )
    })
}

impl EvidenceByteSource for UfedEntryByteSource {
    fn source_ref(&self) -> EvidenceSourceRef {
        self.source_id()
    }

    fn len(&self) -> EvidenceSourceResult<u64> {
        if let Some(size) = self.known_size {
            return Ok(size);
        }

        if self.is_zip_backed() {
            return self.zip_entry_len();
        }

        std::fs::metadata(self.local_entry_path())
            .map(|metadata| metadata.len())
            .map_err(|e| {
                source_error(
                    &self.container_path,
                    &self.entry_path,
                    "ufed",
                    format!("UFED local entry metadata failed: {e}"),
                )
            })
    }

    fn read_range(&self, offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>> {
        if self.is_zip_backed() {
            return self.read_zip_range(offset, size);
        }

        self.read_local_range(offset, size)
    }
}

fn container_entry_ref(
    container_path: &str,
    entry_path: &str,
    container_type: &str,
) -> EvidenceSourceRef {
    EvidenceSourceRef::ContainerEntry {
        container_path: container_path.to_string(),
        entry_path: entry_path.to_string(),
        container_type: container_type.to_string(),
    }
}

fn source_error(
    container_path: &str,
    entry_path: &str,
    container_type: &str,
    message: String,
) -> EvidenceSourceError {
    EvidenceSourceError::Container {
        source_id: container_entry_ref(container_path, entry_path, container_type).display_id(),
        message,
    }
}

fn is_ad1_type(kind: &str) -> bool {
    kind.contains("ad1")
}

fn is_l01_type(kind: &str) -> bool {
    kind.contains("l01") || kind.contains("lx01")
}

fn is_ufed_type(kind: &str) -> bool {
    kind.contains("ufed") || kind == "ufd" || kind == "ufdr" || kind == "ufdx"
}

fn is_ewf_type(kind: &str) -> bool {
    kind.contains("e01") || kind.contains("ex01") || kind.contains("ewf")
}

fn is_raw_type(kind: &str) -> bool {
    kind == "raw" || kind == "dd" || kind == "img" || kind == "001"
}

fn is_archive_type(kind: &str) -> bool {
    matches!(
        kind,
        "archive"
            | "zip"
            | "zip64"
            | "7z"
            | "7-zip"
            | "rar"
            | "rar4"
            | "rar5"
            | "tar"
            | "gz"
            | "gzip"
            | "bz2"
            | "bzip2"
            | "xz"
            | "zst"
            | "zstd"
            | "lz4"
            | "tar.gz"
            | "tgz"
            | "tar.xz"
            | "txz"
            | "tar.bz2"
            | "tbz2"
            | "tar.zst"
            | "tar.lz4"
            | "dmg"
            | "iso"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn ad1_source_ref_identifies_container_entry() {
        let source = Ad1EntryByteSource::new(
            "/cases/evidence.ad1".to_string(),
            "/Documents/file.txt".to_string(),
            Some(128),
        );

        assert_eq!(
            source.source_ref(),
            EvidenceSourceRef::ContainerEntry {
                container_path: "/cases/evidence.ad1".to_string(),
                entry_path: "/Documents/file.txt".to_string(),
                container_type: "ad1".to_string(),
            }
        );
        assert_eq!(source.len().unwrap(), 128);
    }

    #[test]
    fn l01_source_ref_identifies_container_entry() {
        let source = L01EntryByteSource::new(
            "/cases/logical.L01".to_string(),
            "/Users/test/report.pdf".to_string(),
            Some(256),
        );

        assert_eq!(
            source.source_ref(),
            EvidenceSourceRef::ContainerEntry {
                container_path: "/cases/logical.L01".to_string(),
                entry_path: "/Users/test/report.pdf".to_string(),
                container_type: "l01".to_string(),
            }
        );
        assert_eq!(source.len().unwrap(), 256);
    }

    #[test]
    fn checked_entry_data_offset_rejects_overflow() {
        let err =
            checked_entry_data_offset("/cases/logical.L01", "/entry.bin", "l01", u64::MAX - 1, 2)
                .unwrap_err();

        assert!(matches!(err, EvidenceSourceError::Container { .. }));
        assert!(err.to_string().contains("read offset overflow"));
    }

    #[test]
    fn local_container_entry_path_accepts_windows_relative_components() {
        let resolved = local_container_entry_path("/cases/ufed/case.ufd", r"files\media\photo.jpg");

        assert_eq!(
            resolved,
            Path::new("/cases/ufed")
                .join("files")
                .join("media")
                .join("photo.jpg")
        );
    }

    #[test]
    fn unsupported_container_type_returns_container_error() {
        match open_container_entry_source("/cases/item.bin", "/entry", "unknown-format", None) {
            Ok(_) => panic!("unsupported source type should fail"),
            Err(err) => assert!(matches!(err, EvidenceSourceError::Container { .. })),
        }
    }

    #[test]
    fn archive_type_gate_matches_browsable_archive_types() {
        for kind in [
            "archive", "zip", "zip64", "7z", "7-zip", "rar", "rar4", "rar5", "tar", "gz", "gzip",
            "bz2", "bzip2", "xz", "zst", "zstd", "lz4", "tar.gz", "tgz", "tar.xz", "txz",
            "tar.bz2", "tbz2", "tar.zst", "tar.lz4", "dmg", "iso",
        ] {
            assert!(is_archive_type(kind), "{kind} should route to ArchiveVfs");
        }
    }

    #[test]
    fn ufed_source_reads_local_entry_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let container_path = temp_dir.path().join("case.ufd");
        let entry_path = temp_dir.path().join("Media/photo.txt");
        std::fs::create_dir_all(entry_path.parent().unwrap()).unwrap();
        std::fs::write(&container_path, b"[case]\n").unwrap();
        std::fs::write(&entry_path, b"abcdef").unwrap();

        let source = UfedEntryByteSource::new(
            container_path.to_string_lossy().to_string(),
            entry_path.to_string_lossy().to_string(),
            None,
        );

        assert_eq!(source.len().unwrap(), 6);
        assert_eq!(source.read_range(2, 3).unwrap(), b"cde");
        assert_eq!(source.read_range(6, 10).unwrap(), b"");
        let err = source.read_range(7, 1).unwrap_err();
        assert!(matches!(
            err,
            EvidenceSourceError::InvalidRange {
                offset: 7,
                size: 6,
                ..
            }
        ));
        assert_eq!(
            source.source_ref(),
            EvidenceSourceRef::ContainerEntry {
                container_path: container_path.to_string_lossy().to_string(),
                entry_path: entry_path.to_string_lossy().to_string(),
                container_type: "ufed".to_string(),
            }
        );
    }

    #[test]
    fn ufed_source_reads_sparse_local_entry_range_without_full_materialization() {
        let temp_dir = tempfile::tempdir().unwrap();
        let container_path = temp_dir.path().join("case.ufd");
        let entry_path = temp_dir.path().join("files/large.bin");
        std::fs::create_dir_all(entry_path.parent().unwrap()).unwrap();
        std::fs::write(&container_path, b"[case]\n").unwrap();

        let mut file = File::create(&entry_path).unwrap();
        file.write_all(b"head").unwrap();
        file.set_len(2 * 1024 * 1024 * 1024).unwrap();

        let source = UfedEntryByteSource::new(
            container_path.to_string_lossy().to_string(),
            entry_path.to_string_lossy().to_string(),
            None,
        );

        assert_eq!(source.len().unwrap(), 2 * 1024 * 1024 * 1024);
        assert_eq!(source.read_range(0, 4).unwrap(), b"head");
    }

    #[test]
    fn ufed_source_resolves_relative_local_entry_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let container_path = temp_dir.path().join("case.ufd");
        let entry_path = temp_dir.path().join("files/report.txt");
        std::fs::create_dir_all(entry_path.parent().unwrap()).unwrap();
        std::fs::write(&container_path, b"[case]\n").unwrap();
        std::fs::write(&entry_path, b"report").unwrap();

        let source = UfedEntryByteSource::new(
            container_path.to_string_lossy().to_string(),
            "files/report.txt".to_string(),
            Some(6),
        );

        assert_eq!(source.len().unwrap(), 6);
        assert_eq!(source.read_range(0, 64).unwrap(), b"report");
    }

    #[test]
    fn ufed_source_resolves_windows_relative_local_entry_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let container_path = temp_dir.path().join("case.ufd");
        let entry_path = temp_dir.path().join("files").join("report.txt");
        std::fs::create_dir_all(entry_path.parent().unwrap()).unwrap();
        std::fs::write(&container_path, b"[case]\n").unwrap();
        std::fs::write(&entry_path, b"report").unwrap();

        let source = UfedEntryByteSource::new(
            container_path.to_string_lossy().to_string(),
            r"files\report.txt".to_string(),
            Some(6),
        );

        assert_eq!(source.len().unwrap(), 6);
        assert_eq!(source.read_range(0, 64).unwrap(), b"report");
    }
}
