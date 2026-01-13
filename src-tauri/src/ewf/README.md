# EWF Parser Module

Expert Witness Format (E01/Ex01) and EnCase Logical (L01/Lx01) parsing.

## Files

- `mod.rs` - Public API
- `types.rs` - EWF structs
- `parser.rs` - Binary parsing
- `operations.rs` - Read/seek/verify
- `handle.rs` - File handle management
- `cache.rs` - Chunk cache

## Capabilities

- Segment discovery and metadata
- Hash verification (stored vs computed)
- Chunked read operations
- L01/Lx01 logical evidence support

## Notes

E01 and L01 share the EVF container family. L01 is logical (file-based) while E01 is physical (sector-based). Detection distinguishes them by signature and section semantics.
