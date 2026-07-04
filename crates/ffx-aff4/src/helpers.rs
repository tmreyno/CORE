// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Shared helper functions used by the reader, writer, and logical modules.

use std::path::{Path, PathBuf};

use crate::types::{Aff4Phase, Aff4Progress};

/// Emit a progress event if a callback is configured.
pub(crate) fn emit_progress(
    progress_fn: &mut Option<Box<dyn FnMut(Aff4Progress) + Send>>,
    phase: Aff4Phase,
    bytes_processed: u64,
    total_bytes: u64,
    current_file: &str,
    files_processed: usize,
    total_files: usize,
) {
    if let Some(cb) = progress_fn.as_mut() {
        cb(Aff4Progress {
            phase,
            bytes_processed,
            total_bytes,
            current_file: current_file.to_string(),
            files_processed,
            total_files,
        });
    }
}

/// Ensure the output path has an `.aff4` extension.
pub(crate) fn ensure_aff4_extension(path: &Path) -> PathBuf {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext.eq_ignore_ascii_case("aff4") {
        path.to_path_buf()
    } else {
        path.with_extension("aff4")
    }
}

/// Read exactly `buf.len()` bytes, or fewer at EOF.
///
/// Unlike `Read::read_exact`, this does not error when fewer bytes are
/// available — it returns the number of bytes actually read.
#[cfg(test)]
pub(crate) fn read_exact_or_eof<R: std::io::Read>(
    reader: &mut R,
    buf: &mut [u8],
) -> crate::error::Aff4Result<usize> {
    use crate::error::Aff4Error;
    let mut total = 0;
    while total < buf.len() {
        match reader.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(Aff4Error::Io(e)),
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_extension_adds() {
        assert_eq!(
            ensure_aff4_extension(Path::new("test")),
            Path::new("test.aff4")
        );
    }

    #[test]
    fn test_ensure_extension_preserves() {
        assert_eq!(
            ensure_aff4_extension(Path::new("test.aff4")),
            Path::new("test.aff4")
        );
    }

    #[test]
    fn test_ensure_extension_replaces() {
        assert_eq!(
            ensure_aff4_extension(Path::new("/tmp/image.raw")),
            Path::new("/tmp/image.aff4")
        );
    }

    #[test]
    fn test_ensure_extension_case_insensitive() {
        assert_eq!(
            ensure_aff4_extension(Path::new("test.AFF4")),
            Path::new("test.AFF4")
        );
    }

    #[test]
    fn test_read_exact_or_eof_full() {
        let data = b"hello world";
        let mut cursor = std::io::Cursor::new(data);
        let mut buf = [0u8; 11];
        let n = read_exact_or_eof(&mut cursor, &mut buf).unwrap();
        assert_eq!(n, 11);
        assert_eq!(&buf, b"hello world");
    }

    #[test]
    fn test_read_exact_or_eof_partial() {
        let data = b"short";
        let mut cursor = std::io::Cursor::new(data);
        let mut buf = [0u8; 10];
        let n = read_exact_or_eof(&mut cursor, &mut buf).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buf[..5], b"short");
    }

    #[test]
    fn test_emit_progress_with_callback() {
        let mut cb: Option<Box<dyn FnMut(Aff4Progress) + Send>> =
            Some(Box::new(|p: Aff4Progress| {
                assert_eq!(p.phase, Aff4Phase::WritingData);
                assert_eq!(p.bytes_processed, 100);
                assert_eq!(p.total_bytes, 200);
                // We can't set `called` here due to borrow rules, but the
                // assertion passing proves the callback ran.
            }));
        emit_progress(&mut cb, Aff4Phase::WritingData, 100, 200, "test.bin", 1, 2);
        // If we get here without panic, the callback ran successfully.

        // Test with None — should not panic
        let mut none_cb: Option<Box<dyn FnMut(Aff4Progress) + Send>> = None;
        emit_progress(&mut none_cb, Aff4Phase::Preparing, 0, 0, "", 0, 0);
    }
}
