// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AFF4 URI and URN handling.
//!
//! AFF4 uses URNs (Uniform Resource Names) as unique identifiers for all
//! objects: volumes, images, streams, bevies. URNs use the `aff4://` scheme
//! with UUIDs.
//!
//! The mapping between URNs and ZIP member paths follows these rules:
//! 1. If the URN starts with the Volume URN → strip that prefix, use remainder
//! 2. Otherwise → URI-encode `://` as `%3A%2F%2F`, rest forms folder/file path
//! 3. Escaping uses uppercase hex encoding

/// Generate a new random AFF4 URN.
pub fn new_urn() -> String {
    let id = uuid::Uuid::new_v4();
    format!("aff4://{}", id.as_hyphenated())
}

/// Generate a volume URN.
pub fn new_volume_urn() -> String {
    new_urn()
}

/// Generate an image URN (child of a volume).
pub fn new_image_urn() -> String {
    new_urn()
}

/// Convert an AFF4 URN to a ZIP member path, relative to a volume URN.
///
/// Per the AFF4 spec §5:
/// - If `urn` starts with `volume_urn`, strip the prefix and use the remainder
/// - Otherwise, URL-encode the `://` portion and use the full path
pub fn urn_to_zip_path(urn: &str, volume_urn: &str) -> String {
    // Ensure volume URN ends with "/" for prefix matching
    let prefix = if volume_urn.ends_with('/') {
        volume_urn.to_string()
    } else {
        format!("{}/", volume_urn)
    };

    if urn.starts_with(&prefix) {
        // Strip volume URN prefix
        urn[prefix.len()..].to_string()
    } else if urn == volume_urn {
        // The volume itself — container.description
        String::new()
    } else {
        // Full URI encoding: replace "://" with "%3A%2F%2F"
        escape_uri_to_path(urn)
    }
}

/// Convert a ZIP member path back to an AFF4 URN, given the volume URN.
pub fn zip_path_to_urn(path: &str, volume_urn: &str) -> String {
    if path.contains("%3A%2F%2F") || path.contains("%3a%2f%2f") {
        // The path is a fully-escaped external URN
        unescape_path_to_uri(path)
    } else {
        // Local path under the volume
        let prefix = if volume_urn.ends_with('/') {
            volume_urn.to_string()
        } else {
            format!("{}/", volume_urn)
        };
        format!("{}{}", prefix, path)
    }
}

/// Escape a URI for use as a ZIP member path.
/// Replaces `://` with `%3A%2F%2F` and uses uppercase hex for special chars.
fn escape_uri_to_path(uri: &str) -> String {
    uri.replace("://", "%3A%2F%2F")
}

/// Unescape a ZIP member path back to a URI.
fn unescape_path_to_uri(path: &str) -> String {
    path.replace("%3A%2F%2F", "://").replace("%3a%2f%2f", "://")
}

/// Build the ZIP path for a bevy data file.
///
/// Format: `<urn-path>/XXXXXXXX` (8-digit zero-padded hex bevy index)
pub fn bevy_data_path(stream_urn: &str, volume_urn: &str, bevy_index: u32) -> String {
    let base = urn_to_zip_path(stream_urn, volume_urn);
    format!("{}/{:08x}", base, bevy_index)
}

/// Build the ZIP path for a bevy index file.
///
/// Format: `<urn-path>/XXXXXXXX.index`
pub fn bevy_index_path(stream_urn: &str, volume_urn: &str, bevy_index: u32) -> String {
    let base = urn_to_zip_path(stream_urn, volume_urn);
    format!("{}/{:08x}.index", base, bevy_index)
}

/// Build the ZIP path for a bevy block hash file.
///
/// Format: `<urn-path>/XXXXXXXX.<hash-ext>` (e.g., `00000000.sha256`)
pub fn bevy_block_hash_path(
    stream_urn: &str,
    volume_urn: &str,
    bevy_index: u32,
    hash_ext: &str,
) -> String {
    let base = urn_to_zip_path(stream_urn, volume_urn);
    format!("{}/{:08x}.{}", base, bevy_index, hash_ext)
}

/// Build the ZIP path for a map data file.
pub fn map_data_path(map_urn: &str, volume_urn: &str) -> String {
    let base = urn_to_zip_path(map_urn, volume_urn);
    format!("{}/map", base)
}

/// Build the ZIP path for a map index file.
pub fn map_idx_path(map_urn: &str, volume_urn: &str) -> String {
    let base = urn_to_zip_path(map_urn, volume_urn);
    format!("{}/idx", base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_urn_format() {
        let urn = new_urn();
        assert!(urn.starts_with("aff4://"));
        assert_eq!(urn.len(), 7 + 36); // "aff4://" + UUID hyphenated
    }

    #[test]
    fn test_urn_to_zip_path_local() {
        let volume = "aff4://abc-123";
        let urn = "aff4://abc-123/stream001";
        assert_eq!(urn_to_zip_path(urn, volume), "stream001");
    }

    #[test]
    fn test_urn_to_zip_path_external() {
        let volume = "aff4://abc-123";
        let urn = "aff4://def-456/other";
        assert_eq!(urn_to_zip_path(urn, volume), "aff4%3A%2F%2Fdef-456/other");
    }

    #[test]
    fn test_zip_path_to_urn_local() {
        let volume = "aff4://abc-123";
        assert_eq!(
            zip_path_to_urn("stream001", volume),
            "aff4://abc-123/stream001"
        );
    }

    #[test]
    fn test_zip_path_to_urn_external() {
        let volume = "aff4://abc-123";
        assert_eq!(
            zip_path_to_urn("aff4%3A%2F%2Fdef-456/other", volume),
            "aff4://def-456/other"
        );
    }

    #[test]
    fn test_bevy_paths() {
        let volume = "aff4://vol-1";
        let stream = "aff4://vol-1/image";
        assert_eq!(bevy_data_path(stream, volume, 0), "image/00000000");
        assert_eq!(bevy_index_path(stream, volume, 0), "image/00000000.index");
        assert_eq!(
            bevy_block_hash_path(stream, volume, 3, "sha256"),
            "image/00000003.sha256"
        );
    }

    #[test]
    fn test_map_paths() {
        let volume = "aff4://vol-1";
        let map = "aff4://vol-1/map-stream";
        assert_eq!(map_data_path(map, volume), "map-stream/map");
        assert_eq!(map_idx_path(map, volume), "map-stream/idx");
    }

    #[test]
    fn test_roundtrip() {
        let volume = "aff4://abc-123";
        let urn = "aff4://abc-123/data/stream";
        let path = urn_to_zip_path(urn, volume);
        let recovered = zip_path_to_urn(&path, volume);
        assert_eq!(recovered, urn);
    }
}
