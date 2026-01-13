# Archive Module

Archive detection and metadata parsing (ZIP/7z/RAR and common compressed formats).

## Files

- `mod.rs` - Public API
- `types.rs` - Archive types and metadata
- `detection.rs` - Magic signatures
- `zip.rs` - ZIP/ZIP64 metadata parsing
- `sevenz.rs` - 7-Zip header parsing
- `segments.rs` - Multipart archive discovery
- `extraction.rs` - ZIP extraction (other formats are metadata-only)

## Supported Operations

- Format detection by signature
- Metadata extraction (entry counts, encryption flags, segment info)
- ZIP entry verification (CRC32)
- ZIP extraction to a safe output directory

Other archive formats (RAR, TAR, etc.) are detected for metadata but not extracted in-core.
