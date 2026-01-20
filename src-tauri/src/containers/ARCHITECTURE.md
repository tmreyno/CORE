# Container Module Architecture

This document describes the architecture of the containers module and which components to use for different operations.

## Overview

The containers module has evolved to have multiple systems. This document clarifies which to use.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       CONTAINER OPERATIONS                               │
├─────────────────────────────────────────────────────────────────────────┤
│  operations.rs  │  Primary API for container info/verify/extract         │
│  unified.rs     │  Tree navigation and lazy loading for UI               │
│  impls.rs       │  DEPRECATED: Trait-based parsers (not used)            │
│  traits.rs      │  ContainerError (active), parser traits (deprecated)   │
└─────────────────────────────────────────────────────────────────────────┘
```

## Active Components

### 1. operations.rs - Container Operations

**Purpose**: Main entry points for container operations (info, verify, extract)

**Used by**: `commands/container.rs`

**Key Functions**:
- `info(path)` - Get full container metadata with file tree
- `info_fast(path)` - Get metadata without parsing full tree (faster)
- `verify(path)` - Verify container integrity with hash checking
- `extract(path, output)` - Extract contents to destination

**Example**:
```rust
use crate::containers::{info, verify, extract};

// Get container info
let container = info("/path/to/evidence.ad1")?;

// Verify integrity
let result = verify("/path/to/evidence.e01")?;

// Extract contents
extract("/path/to/evidence.zip", "/output/dir")?;
```

### 2. unified.rs - Tree Navigation

**Purpose**: Unified handler for browsing container file trees in the UI

**Used by**: `commands/unified.rs`, frontend via `useUnifiedContainer.ts` hook

**Key Functions**:
- `get_summary(path)` - Get container summary (type, size, entry count)
- `get_root_children(path, offset, limit)` - Get paginated root entries
- `get_children(path, parent_path, offset, limit)` - Get children of a specific entry
- `get_handler_for_path(path)` - Get appropriate handler by file extension

**Example**:
```rust
use crate::containers::unified::{get_summary, get_children};

// Get container summary
let summary = get_summary("/path/to/evidence.7z")?;

// Get root-level entries (first 100)
let root = get_root_children("/path/to/evidence.7z", 0, 100)?;

// Get children of a specific folder
let children = get_children("/path/to/evidence.7z", "Documents/", 0, 100)?;
```

### 3. traits.rs - Error Types

**Active Parts**:
- `ContainerError` - Unified error type used throughout the codebase

**Deprecated Parts**:
- `EvidenceContainer` trait - Not used in production
- `SegmentedContainer`, `TreeContainer`, `HashableContainer` traits - Not used
- Other metadata types (`FormatInfo`, `ContainerMetadata`, etc.) - Not used

## Deprecated Components

### impls.rs - Trait-Based Parsers (DEPRECATED)

This module contains ~1100 lines of trait implementations that were designed for a plugin system but never integrated into the production code.

**Status**: Marked as deprecated with `#![allow(dead_code, deprecated)]`

**Why Deprecated**:
1. `operations.rs` calls format modules directly instead of using traits
2. The trait-based approach adds indirection without benefit
3. All functionality is duplicated in `operations.rs`

**If You Need Plugins**:
The trait system could be revived for a future plugin architecture. The implementations are preserved but not compiled into the main binary paths.

## Format-Specific Modules

Each format has its own module with direct implementations:

| Format | Module | Functions |
|--------|--------|-----------|
| AD1 | `crate::ad1` | `info()`, `verify()`, `extract()`, `read_file()` |
| E01/L01 | `crate::ewf` | `info()`, `verify()`, `extract()`, `read_file()` |
| RAW | `crate::raw` | `info()`, `verify()` |
| UFED | `crate::ufed` | `info()`, `verify()`, `extract()` |
| ZIP/7z/RAR | `crate::archive` | `info()`, `list_entries()`, `extract()` |

## Best Practices

1. **For Commands**: Use `containers::*` functions from `operations.rs`
2. **For UI Tree**: Use `unified::get_children()` with pagination
3. **For Errors**: Use `ContainerError` from `traits.rs`
4. **For Format Detection**: Use `operations::detect_container()`
5. **Never**: Use parsers from `impls.rs` (they're deprecated)

## File Structure

```
containers/
├── mod.rs           # Module exports and re-exports
├── operations.rs    # ACTIVE: Main container operations
├── unified.rs       # ACTIVE: Tree navigation for UI
├── traits.rs        # MIXED: ContainerError (active), traits (deprecated)
├── impls.rs         # DEPRECATED: Trait implementations
├── types.rs         # Container info types
├── scanning.rs      # Directory scanning
├── segments.rs      # Multi-segment file handling
├── companion.rs     # Companion log file handling
├── case_documents.rs # Case document discovery
└── ARCHITECTURE.md  # This file
```
