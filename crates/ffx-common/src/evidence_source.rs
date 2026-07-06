// =============================================================================
// ffx-common — Common Utilities for Forensic Container Parsers
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Read-only byte sources for evidence data.
//!
//! This module gives viewers, hashers, indexers, and report/artifact extractors
//! a common way to read bytes from local files or virtual filesystem entries.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::vfs::VirtualFileSystem;

const READ_ALL_CHUNK_BYTES: u64 = 1024 * 1024;

/// Stable reference to evidence bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum EvidenceSourceRef {
    /// A file directly accessible on the host filesystem.
    LocalFile { path: String },
    /// A file entry inside a logical/container format.
    ContainerEntry {
        container_path: String,
        entry_path: String,
        container_type: String,
    },
    /// A file entry inside a container that is itself nested inside another container.
    NestedContainerEntry {
        container_path: String,
        nested_container_path: String,
        entry_path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        container_type: Option<String>,
    },
    /// A file inside a mounted/readable evidence container.
    VfsEntry {
        container_path: String,
        entry_path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        container_type: Option<String>,
    },
}

impl EvidenceSourceRef {
    /// Human-readable identifier suitable for diagnostics.
    pub fn display_id(&self) -> String {
        match self {
            Self::LocalFile { path } => path.clone(),
            Self::ContainerEntry {
                container_path,
                entry_path,
                ..
            } => format!("{container_path}:{entry_path}"),
            Self::NestedContainerEntry {
                container_path,
                nested_container_path,
                entry_path,
                ..
            } => format!("{container_path}:{nested_container_path}::{entry_path}"),
            Self::VfsEntry {
                container_path,
                entry_path,
                ..
            } => format!("{container_path}:{entry_path}"),
        }
    }
}

/// Errors returned by [`EvidenceByteSource`] implementations.
#[derive(Debug, thiserror::Error)]
pub enum EvidenceSourceError {
    #[error("I/O error reading {source_id}: {message}")]
    Io { source_id: String, message: String },
    #[error("VFS error reading {source_id}: {message}")]
    Vfs { source_id: String, message: String },
    #[error("Container error reading {source_id}: {message}")]
    Container { source_id: String, message: String },
    #[error("Invalid read range for {source_id}: offset {offset} is beyond size {size}")]
    InvalidRange {
        source_id: String,
        offset: u64,
        size: u64,
    },
    #[error("Evidence source is too large for this operation: {size} bytes > {max_size} bytes")]
    TooLarge { size: u64, max_size: u64 },
    #[error("Short read for {source_id}: expected {expected} bytes but read {actual} bytes")]
    ShortRead {
        source_id: String,
        expected: u64,
        actual: u64,
    },
    #[error("Oversized read for {source_id}: requested {requested} bytes but source returned {actual} bytes")]
    OversizedRead {
        source_id: String,
        requested: u64,
        actual: u64,
    },
    #[error("Read offset overflow for {source_id}: offset {offset} + {bytes_read} bytes")]
    ReadOffsetOverflow {
        source_id: String,
        offset: u64,
        bytes_read: u64,
    },
}

/// Result alias for evidence byte-source operations.
pub type EvidenceSourceResult<T> = Result<T, EvidenceSourceError>;

/// Read-only byte source for evidence data.
pub trait EvidenceByteSource: Send + Sync {
    /// Stable source reference.
    fn source_ref(&self) -> EvidenceSourceRef;

    /// Total byte length of this source.
    fn len(&self) -> EvidenceSourceResult<u64>;

    /// Whether this source is empty.
    fn is_empty(&self) -> EvidenceSourceResult<bool> {
        Ok(self.len()? == 0)
    }

    /// Read up to `size` bytes at `offset`.
    fn read_range(&self, offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>>;
}

/// [`Read`] + [`Seek`] adapter over an [`EvidenceByteSource`].
///
/// This lets parser engines consume evidence entries directly without copying
/// the full entry to a temporary file or adding container-specific code paths.
pub struct EvidenceSourceReader<'a> {
    source: &'a dyn EvidenceByteSource,
    position: u64,
}

impl<'a> EvidenceSourceReader<'a> {
    pub fn new(source: &'a dyn EvidenceByteSource) -> Self {
        Self {
            source,
            position: 0,
        }
    }

    pub fn position(&self) -> u64 {
        self.position
    }

    fn io_error(error: EvidenceSourceError) -> std::io::Error {
        std::io::Error::other(error.to_string())
    }
}

impl Read for EvidenceSourceReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let data = self
            .source
            .read_range(self.position, buf.len())
            .map_err(Self::io_error)?;
        let bytes_read = data.len();
        if bytes_read > buf.len() {
            return Err(Self::io_error(EvidenceSourceError::OversizedRead {
                source_id: self.source.source_ref().display_id(),
                requested: buf.len() as u64,
                actual: bytes_read as u64,
            }));
        }
        if bytes_read == 0 {
            let total_size = self.source.len().map_err(Self::io_error)?;
            if self.position < total_size {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    EvidenceSourceError::ShortRead {
                        source_id: self.source.source_ref().display_id(),
                        expected: total_size,
                        actual: self.position,
                    }
                    .to_string(),
                ));
            }
        }
        let next_position =
            checked_read_offset_add(&self.source.source_ref(), self.position, bytes_read)
                .map_err(Self::io_error)?;
        buf[..bytes_read].copy_from_slice(&data);
        self.position = next_position;
        Ok(bytes_read)
    }
}

impl Seek for EvidenceSourceReader<'_> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let next = match pos {
            SeekFrom::Start(offset) => offset as i128,
            SeekFrom::End(offset) => {
                let len = self.source.len().map_err(Self::io_error)?;
                len as i128 + offset as i128
            }
            SeekFrom::Current(offset) => self.position as i128 + offset as i128,
        };

        if next < 0 || next > u64::MAX as i128 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid evidence source seek",
            ));
        }

        self.position = next as u64;
        Ok(self.position)
    }
}

/// Read-only byte source backed by a local file.
#[derive(Clone, Debug)]
pub struct LocalFileByteSource {
    path: PathBuf,
}

impl LocalFileByteSource {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn source_id(&self) -> String {
        self.path.to_string_lossy().to_string()
    }
}

impl EvidenceByteSource for LocalFileByteSource {
    fn source_ref(&self) -> EvidenceSourceRef {
        EvidenceSourceRef::LocalFile {
            path: self.source_id(),
        }
    }

    fn len(&self) -> EvidenceSourceResult<u64> {
        std::fs::metadata(&self.path)
            .map(|m| m.len())
            .map_err(|e| EvidenceSourceError::Io {
                source_id: self.source_id(),
                message: e.to_string(),
            })
    }

    fn read_range(&self, offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>> {
        let total_size = self.len()?;
        let read_size = bounded_read_size(&self.source_ref(), total_size, offset, size)?;
        if read_size == 0 {
            return Ok(Vec::new());
        }

        let mut file = File::open(&self.path).map_err(|e| EvidenceSourceError::Io {
            source_id: self.source_id(),
            message: e.to_string(),
        })?;
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| EvidenceSourceError::Io {
                source_id: self.source_id(),
                message: e.to_string(),
            })?;

        let mut data = vec![0u8; read_size];
        file.read_exact(&mut data)
            .map_err(|e| EvidenceSourceError::Io {
                source_id: self.source_id(),
                message: e.to_string(),
            })?;
        Ok(data)
    }
}

/// Read-only byte source backed by a [`VirtualFileSystem`] entry.
pub struct VfsEntryByteSource {
    vfs: Arc<dyn VirtualFileSystem>,
    source_ref: EvidenceSourceRef,
}

impl VfsEntryByteSource {
    pub fn new(
        vfs: Arc<dyn VirtualFileSystem>,
        container_path: impl Into<String>,
        entry_path: impl Into<String>,
        container_type: Option<String>,
    ) -> Self {
        Self {
            vfs,
            source_ref: EvidenceSourceRef::VfsEntry {
                container_path: container_path.into(),
                entry_path: entry_path.into(),
                container_type,
            },
        }
    }

    fn entry_path(&self) -> &str {
        match &self.source_ref {
            EvidenceSourceRef::VfsEntry { entry_path, .. } => entry_path,
            EvidenceSourceRef::LocalFile { .. }
            | EvidenceSourceRef::ContainerEntry { .. }
            | EvidenceSourceRef::NestedContainerEntry { .. } => {
                unreachable!("VfsEntryByteSource source kind")
            }
        }
    }
}

impl EvidenceByteSource for VfsEntryByteSource {
    fn source_ref(&self) -> EvidenceSourceRef {
        self.source_ref.clone()
    }

    fn len(&self) -> EvidenceSourceResult<u64> {
        self.vfs
            .file_size(self.entry_path())
            .map_err(|e| EvidenceSourceError::Vfs {
                source_id: self.source_ref.display_id(),
                message: e.to_string(),
            })
    }

    fn read_range(&self, offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>> {
        let total_size = self.len()?;
        let target_size = bounded_read_size(&self.source_ref, total_size, offset, size)?;
        if target_size == 0 {
            return Ok(Vec::new());
        }

        let mut data = Vec::with_capacity(target_size);
        let mut current_offset = offset;
        let mut remaining = target_size as u64;

        while remaining > 0 {
            let read_size = remaining.min(READ_ALL_CHUNK_BYTES) as usize;
            let chunk = self
                .vfs
                .read(self.entry_path(), current_offset, read_size)
                .map_err(|e| EvidenceSourceError::Vfs {
                    source_id: self.source_ref.display_id(),
                    message: e.to_string(),
                })?;
            if chunk.len() > read_size {
                return Err(EvidenceSourceError::OversizedRead {
                    source_id: self.source_ref.display_id(),
                    requested: read_size as u64,
                    actual: chunk.len() as u64,
                });
            }
            if chunk.is_empty() {
                return Err(EvidenceSourceError::ShortRead {
                    source_id: self.source_ref.display_id(),
                    expected: target_size as u64,
                    actual: data.len() as u64,
                });
            }

            current_offset =
                checked_read_offset_add(&self.source_ref, current_offset, chunk.len())?;
            remaining -= chunk.len() as u64;
            data.extend_from_slice(&chunk);
        }

        Ok(data)
    }
}

/// Read an entire source, enforcing an operation-specific size cap.
pub fn read_all_with_limit(
    source: &dyn EvidenceByteSource,
    max_size: u64,
) -> EvidenceSourceResult<Vec<u8>> {
    let size = source.len()?;
    if size > max_size {
        return Err(EvidenceSourceError::TooLarge { size, max_size });
    }

    let size = usize::try_from(size).map_err(|_| EvidenceSourceError::TooLarge {
        size,
        max_size: usize::MAX as u64,
    })?;
    read_range_fully(source, 0, size)
}

/// Read exactly `size` bytes from a source range, assembling smaller chunks
/// returned by [`EvidenceByteSource::read_range`].
pub fn read_range_fully(
    source: &dyn EvidenceByteSource,
    offset: u64,
    size: usize,
) -> EvidenceSourceResult<Vec<u8>> {
    if size == 0 {
        return Ok(Vec::new());
    }

    let mut data = Vec::with_capacity(size);
    let mut current_offset = offset;
    let mut remaining = size as u64;

    while remaining > 0 {
        let read_size = remaining.min(READ_ALL_CHUNK_BYTES) as usize;
        let chunk = source.read_range(current_offset, read_size)?;
        if chunk.len() > read_size {
            return Err(EvidenceSourceError::OversizedRead {
                source_id: source.source_ref().display_id(),
                requested: read_size as u64,
                actual: chunk.len() as u64,
            });
        }
        if chunk.is_empty() {
            return Err(EvidenceSourceError::ShortRead {
                source_id: source.source_ref().display_id(),
                expected: size as u64,
                actual: data.len() as u64,
            });
        }

        if chunk.len() as u64 > remaining {
            return Err(EvidenceSourceError::ShortRead {
                source_id: source.source_ref().display_id(),
                expected: size as u64,
                actual: data.len() as u64 + chunk.len() as u64,
            });
        }

        current_offset =
            checked_read_offset_add(&source.source_ref(), current_offset, chunk.len())?;
        remaining -= chunk.len() as u64;
        data.extend_from_slice(&chunk);
    }

    if data.len() != size {
        return Err(EvidenceSourceError::ShortRead {
            source_id: source.source_ref().display_id(),
            expected: size as u64,
            actual: data.len() as u64,
        });
    }
    Ok(data)
}

fn checked_read_offset_add(
    source_ref: &EvidenceSourceRef,
    offset: u64,
    bytes_read: usize,
) -> EvidenceSourceResult<u64> {
    let bytes_read = u64::try_from(bytes_read).map_err(|_| EvidenceSourceError::TooLarge {
        size: u64::MAX,
        max_size: usize::MAX as u64,
    })?;
    offset
        .checked_add(bytes_read)
        .ok_or_else(|| EvidenceSourceError::ReadOffsetOverflow {
            source_id: source_ref.display_id(),
            offset,
            bytes_read,
        })
}

/// Return a read size clamped to source length or an error for offsets past EOF.
pub fn bounded_read_size(
    source_ref: &EvidenceSourceRef,
    total_size: u64,
    offset: u64,
    requested: usize,
) -> EvidenceSourceResult<usize> {
    if offset > total_size {
        return Err(EvidenceSourceError::InvalidRange {
            source_id: source_ref.display_id(),
            offset,
            size: total_size,
        });
    }

    let remaining = total_size - offset;
    Ok((requested as u64).min(remaining) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::{DirEntry, FileAttr, VfsError};
    use std::io::Write;

    struct ShortReadSource {
        source_ref: EvidenceSourceRef,
        declared_len: u64,
        data: Vec<u8>,
        max_chunk: Option<usize>,
    }

    impl ShortReadSource {
        fn new(declared_len: u64, data: &[u8]) -> Self {
            Self {
                source_ref: EvidenceSourceRef::LocalFile {
                    path: "short-read-source.bin".to_string(),
                },
                declared_len,
                data: data.to_vec(),
                max_chunk: None,
            }
        }

        fn chunked(declared_len: u64, data: &[u8], max_chunk: usize) -> Self {
            Self {
                source_ref: EvidenceSourceRef::LocalFile {
                    path: "chunked-source.bin".to_string(),
                },
                declared_len,
                data: data.to_vec(),
                max_chunk: Some(max_chunk),
            }
        }
    }

    impl EvidenceByteSource for ShortReadSource {
        fn source_ref(&self) -> EvidenceSourceRef {
            self.source_ref.clone()
        }

        fn len(&self) -> EvidenceSourceResult<u64> {
            Ok(self.declared_len)
        }

        fn read_range(&self, offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>> {
            if offset > self.declared_len {
                return Err(EvidenceSourceError::InvalidRange {
                    source_id: self.source_ref.display_id(),
                    offset,
                    size: self.declared_len,
                });
            }

            let start = offset as usize;
            if start >= self.data.len() {
                return Ok(Vec::new());
            }
            let requested = self.max_chunk.map(|max| size.min(max)).unwrap_or(size);
            let end = start.saturating_add(requested).min(self.data.len());
            Ok(self.data[start..end].to_vec())
        }
    }

    struct OversizedReadSource {
        source_ref: EvidenceSourceRef,
        declared_len: u64,
        data: Vec<u8>,
    }

    impl OversizedReadSource {
        fn new(data: &[u8]) -> Self {
            Self {
                source_ref: EvidenceSourceRef::LocalFile {
                    path: "oversized-read-source.bin".to_string(),
                },
                declared_len: data.len() as u64,
                data: data.to_vec(),
            }
        }
    }

    impl EvidenceByteSource for OversizedReadSource {
        fn source_ref(&self) -> EvidenceSourceRef {
            self.source_ref.clone()
        }

        fn len(&self) -> EvidenceSourceResult<u64> {
            Ok(self.declared_len)
        }

        fn read_range(&self, offset: u64, _size: usize) -> EvidenceSourceResult<Vec<u8>> {
            let start = usize::try_from(offset).unwrap_or(usize::MAX);
            if start >= self.data.len() {
                return Ok(Vec::new());
            }
            Ok(self.data[start..].to_vec())
        }
    }

    struct OverflowingOffsetSource {
        source_ref: EvidenceSourceRef,
    }

    impl OverflowingOffsetSource {
        fn new() -> Self {
            Self {
                source_ref: EvidenceSourceRef::LocalFile {
                    path: "overflowing-offset-source.bin".to_string(),
                },
            }
        }
    }

    impl EvidenceByteSource for OverflowingOffsetSource {
        fn source_ref(&self) -> EvidenceSourceRef {
            self.source_ref.clone()
        }

        fn len(&self) -> EvidenceSourceResult<u64> {
            Ok(u64::MAX)
        }

        fn read_range(&self, _offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>> {
            if size == 0 {
                Ok(Vec::new())
            } else {
                Ok(vec![0xAA])
            }
        }
    }

    struct ChunkedVfs {
        data: Vec<u8>,
        max_chunk: usize,
        empty_at: Option<u64>,
        oversized: bool,
    }

    impl ChunkedVfs {
        fn new(data: &[u8], max_chunk: usize) -> Self {
            Self {
                data: data.to_vec(),
                max_chunk,
                empty_at: None,
                oversized: false,
            }
        }

        fn with_empty_at(mut self, offset: u64) -> Self {
            self.empty_at = Some(offset);
            self
        }

        fn oversized(data: &[u8]) -> Self {
            Self {
                data: data.to_vec(),
                max_chunk: data.len(),
                empty_at: None,
                oversized: true,
            }
        }
    }

    impl VirtualFileSystem for ChunkedVfs {
        fn getattr(&self, path: &str) -> Result<FileAttr, VfsError> {
            if path != "/entry.bin" {
                return Err(VfsError::NotFound(path.to_string()));
            }
            Ok(FileAttr::file(self.data.len() as u64))
        }

        fn readdir(&self, _path: &str) -> Result<Vec<DirEntry>, VfsError> {
            Ok(Vec::new())
        }

        fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
            if path != "/entry.bin" {
                return Err(VfsError::NotFound(path.to_string()));
            }
            if self.empty_at == Some(offset) {
                return Ok(Vec::new());
            }

            let start =
                usize::try_from(offset).map_err(|_| VfsError::OutOfBounds { offset, size })?;
            if start >= self.data.len() {
                return Ok(Vec::new());
            }
            if self.oversized {
                let end = start.saturating_add(size + 1).min(self.data.len());
                return Ok(self.data[start..end].to_vec());
            }

            let read_size = size.min(self.max_chunk);
            let end = start.saturating_add(read_size).min(self.data.len());
            Ok(self.data[start..end].to_vec())
        }
    }

    #[test]
    fn local_file_source_reads_requested_range() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample.bin");
        std::fs::write(&path, b"0123456789").unwrap();

        let source = LocalFileByteSource::new(&path);

        assert_eq!(source.len().unwrap(), 10);
        assert_eq!(source.read_range(2, 4).unwrap(), b"2345");
    }

    #[test]
    fn local_file_source_clamps_at_eof() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample.bin");
        std::fs::write(&path, b"abcdef").unwrap();

        let source = LocalFileByteSource::new(&path);

        assert_eq!(source.read_range(4, 8).unwrap(), b"ef");
        assert!(source.read_range(6, 8).unwrap().is_empty());
    }

    #[test]
    fn local_file_source_rejects_offset_past_eof() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample.bin");
        std::fs::write(&path, b"abcdef").unwrap();

        let source = LocalFileByteSource::new(&path);
        let err = source.read_range(7, 1).unwrap_err();

        assert!(matches!(err, EvidenceSourceError::InvalidRange { .. }));
    }

    #[test]
    fn vfs_entry_source_assembles_partial_chunks() {
        let source = VfsEntryByteSource::new(
            Arc::new(ChunkedVfs::new(b"0123456789", 2)),
            "/cases/disk.E01",
            "/entry.bin",
            Some("e01".to_string()),
        );

        let data = source.read_range(2, 5).unwrap();

        assert_eq!(data, b"23456");
    }

    #[test]
    fn vfs_entry_source_rejects_empty_chunk_before_requested_range_complete() {
        let source = VfsEntryByteSource::new(
            Arc::new(ChunkedVfs::new(b"0123456789", 3).with_empty_at(5)),
            "/cases/disk.E01",
            "/entry.bin",
            Some("e01".to_string()),
        );

        let err = source.read_range(2, 6).unwrap_err();

        assert!(matches!(
            err,
            EvidenceSourceError::ShortRead {
                source_id,
                expected: 6,
                actual: 3,
            } if source_id == "/cases/disk.E01:/entry.bin"
        ));
    }

    #[test]
    fn vfs_entry_source_rejects_oversized_vfs_chunk() {
        let source = VfsEntryByteSource::new(
            Arc::new(ChunkedVfs::oversized(b"0123456789")),
            "/cases/disk.E01",
            "/entry.bin",
            Some("e01".to_string()),
        );

        let err = source.read_range(0, 5).unwrap_err();

        assert!(matches!(
            err,
            EvidenceSourceError::OversizedRead {
                source_id,
                requested: 5,
                actual: 6,
            } if source_id == "/cases/disk.E01:/entry.bin"
        ));
    }

    #[test]
    fn read_all_with_limit_rejects_oversize_source() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample.bin");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"abcdef").unwrap();

        let source = LocalFileByteSource::new(&path);
        let err = read_all_with_limit(&source, 5).unwrap_err();

        assert!(matches!(err, EvidenceSourceError::TooLarge { .. }));
    }

    #[test]
    fn read_all_with_limit_rejects_short_source() {
        let source = ShortReadSource::new(8, b"abc");

        let err = read_all_with_limit(&source, 16).unwrap_err();

        assert!(matches!(
            err,
            EvidenceSourceError::ShortRead {
                source_id,
                expected: 8,
                actual: 3,
            } if source_id == "short-read-source.bin"
        ));
    }

    #[test]
    fn read_all_with_limit_accepts_chunked_source() {
        let source = ShortReadSource::chunked(10, b"0123456789", 3);

        let data = read_all_with_limit(&source, 16).unwrap();

        assert_eq!(data, b"0123456789");
    }

    #[test]
    fn read_range_fully_accepts_chunked_source_with_offset() {
        let source = ShortReadSource::chunked(10, b"0123456789", 2);

        let data = read_range_fully(&source, 2, 5).unwrap();

        assert_eq!(data, b"23456");
    }

    #[test]
    fn read_range_fully_rejects_oversized_source_chunk() {
        let source = OversizedReadSource::new(b"abcdef");

        let err = read_range_fully(&source, 0, 3).unwrap_err();

        assert!(matches!(
            err,
            EvidenceSourceError::OversizedRead {
                source_id,
                requested: 3,
                actual: 6,
            } if source_id == "oversized-read-source.bin"
        ));
    }

    #[test]
    fn read_range_fully_rejects_offset_overflow() {
        let source = OverflowingOffsetSource::new();

        let err = read_range_fully(&source, u64::MAX, 2).unwrap_err();

        assert!(matches!(
            err,
            EvidenceSourceError::ReadOffsetOverflow {
                source_id,
                offset: u64::MAX,
                bytes_read: 1,
            } if source_id == "overflowing-offset-source.bin"
        ));
    }

    #[test]
    fn evidence_source_reader_reads_and_seeks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample.bin");
        std::fs::write(&path, b"0123456789").unwrap();
        let source = LocalFileByteSource::new(&path);
        let mut reader = EvidenceSourceReader::new(&source);

        let mut first = [0u8; 4];
        assert_eq!(reader.read(&mut first).unwrap(), 4);
        assert_eq!(&first, b"0123");
        assert_eq!(reader.position(), 4);

        assert_eq!(reader.seek(SeekFrom::Current(2)).unwrap(), 6);
        let mut second = [0u8; 3];
        assert_eq!(reader.read(&mut second).unwrap(), 3);
        assert_eq!(&second, b"678");

        assert_eq!(reader.seek(SeekFrom::End(-2)).unwrap(), 8);
        let mut tail = Vec::new();
        assert_eq!(reader.read_to_end(&mut tail).unwrap(), 2);
        assert_eq!(tail, b"89");
    }

    #[test]
    fn evidence_source_reader_rejects_negative_seek() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample.bin");
        std::fs::write(&path, b"abc").unwrap();
        let source = LocalFileByteSource::new(&path);
        let mut reader = EvidenceSourceReader::new(&source);

        let err = reader.seek(SeekFrom::Current(-1)).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn evidence_source_reader_rejects_premature_eof() {
        let source = ShortReadSource::new(8, b"abc");
        let mut reader = EvidenceSourceReader::new(&source);
        let mut data = Vec::new();

        let err = reader.read_to_end(&mut data).unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
        assert_eq!(data, b"abc");
        assert!(err
            .to_string()
            .contains("Short read for short-read-source.bin"));
    }

    #[test]
    fn evidence_source_reader_rejects_oversized_source_chunk() {
        let source = OversizedReadSource::new(b"abcdef");
        let mut reader = EvidenceSourceReader::new(&source);
        let mut data = [0u8; 3];

        let err = reader.read(&mut data).unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::Other);
        assert!(err
            .to_string()
            .contains("Oversized read for oversized-read-source.bin"));
    }

    #[test]
    fn evidence_source_reader_rejects_offset_overflow() {
        let source = OverflowingOffsetSource::new();
        let mut reader = EvidenceSourceReader::new(&source);
        reader.seek(SeekFrom::Start(u64::MAX)).unwrap();
        let mut byte = [0u8; 1];

        let err = reader.read(&mut byte).unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::Other);
        assert!(err
            .to_string()
            .contains("Read offset overflow for overflowing-offset-source.bin"));
        assert_eq!(reader.position(), u64::MAX);
    }
}
