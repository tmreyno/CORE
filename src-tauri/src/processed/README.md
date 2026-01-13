# Processed Databases Module

Detection and parsing of processed forensic databases.

## Files

- `mod.rs` - Module exports
- `types.rs` - Database types and structures
- `detection.rs` - Type detection
- `commands.rs` - Tauri commands
- `axiom.rs` - Magnet AXIOM parsing

## Supported Databases

- Magnet AXIOM (implemented)
- Cellebrite PA, X-Ways, Autopsy, EnCase, FTK (detected only)

## Notes

AXIOM parsing provides case info, artifact categories, and artifact queries. Other tools are surfaced as processed database entries but do not yet have parsers.
