# AD1 Parser Module

Parser for AccessData AD1 logical evidence containers.

## Files

- `mod.rs` - Public API
- `types.rs` - AD1 structs and metadata types
- `parser.rs` - Binary parsing and decompression
- `operations.rs` - Verify, extract, list
- `utils.rs` - Segment helpers and validation

## Format Notes

- AD1 uses segmented files (`.ad1`, `.ad2`, ...)
- Segment header signature: `ADSEGMENTEDFILE`
- Logical header points to the first item record
- Items link via address chains (next item, first child, metadata)

## Capabilities

- `info` / `info_fast` for metadata
- Lazy tree listing via addresses (`get_tree`, `get_children`)
- Entry reads by path or direct address
- Hash verification and extraction with progress

## Key Types

- `TreeEntry` - Tree node with addresses for lazy loading
- `Ad1Info` - Container metadata
