# Format Unification Refactoring Notes

## Overview

This document tracks efforts to unify format handling across the backend. The current codebase uses a single `containers` abstraction that dispatches to format-specific parsers.

## Current State

- All container operations are routed through `src-tauri/src/containers/`
- E01/Ex01/L01/Lx01 are handled by the `ewf/` module
- AD1 parsing and tree operations live in `ad1/`
- Archive metadata and ZIP extraction live in `archive/`
- Raw images live in `raw.rs`
- UFED parsing lives in `ufed/`

## Consistency Notes

- Hash verification is normalized in `containers::verify()`
- Companion log handling is format-specific (AD1 uses its own log model)
- Format detection is centralized in `formats.rs` and `containers::scanning.rs`

## Module Summary

```
src-tauri/src/
|-- ad1/        # AD1 parsing and extraction
|-- archive/    # Archive detection + ZIP extraction
|-- containers/ # Unified container API
|-- ewf/        # E01/Ex01/L01 parsing
|-- raw.rs      # Raw image handling
|-- ufed/       # UFED extraction parsing
|-- common/     # Shared utilities
```

*Last updated: 2026-01-01*
