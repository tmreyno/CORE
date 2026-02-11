# Containers Module

Evidence container lifecycle and management.

## Architecture

This module provides a unified API over format-specific parsers (ad1, ewf, raw, ufed, archive) and exposes container discovery, info, verification, and extraction.

```
containers/
|-- mod.rs            # Public API
|-- types.rs          # ContainerInfo, ContainerKind, errors
|-- operations.rs     # info/verify/extract dispatch
|-- scanning.rs       # Directory scanning (extension-based)
|-- traits.rs         # Evidence container traits
|-- impls.rs          # Trait implementations
|-- segments.rs       # Multi-segment helpers
|-- companion.rs      # Companion log detection
|-- case_documents.rs # Case document scanning
|-- unified.rs        # Unified container operations
```

## Supported Container Kinds

- AD1
- E01/Ex01
- L01/Lx01
- Raw
- Archive
- UFED

## Lifecycle

1) Discovery -> `scan_directory*`
2) Detection -> `detect_container`
3) Info -> `info()` / `info_fast()`
4) Verify -> `verify()`
5) Extract -> `extract()` (ZIP extraction for archives)

## Notes

- Scans use extension-based detection for speed
- `detect_container` performs format-specific checks and may be stricter
- Additional formats are surfaced during scanning but may not have full parsing yet
