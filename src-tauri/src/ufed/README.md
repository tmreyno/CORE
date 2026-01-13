# UFED Module

Cellebrite UFED extraction parsing.

## Files

- `mod.rs` - Public API
- `types.rs` - UFED structures
- `detection.rs` - Format detection
- `parsing.rs` - XML parsing and metadata extraction
- `collection.rs` - Extraction collection logic
- `archive_scan.rs` - ZIP-based UFED detection

## Supported Inputs

- `.ufd` metadata files
- `.ufdr` report packages
- ZIP-based UFED extractions (collection sets)

## Capabilities

- Device and extraction metadata
- Associated file listings for tree display
- Hash metadata extraction when present
