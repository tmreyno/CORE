# Dead Code Analysis

**Last Updated**: January 17, 2026

This document tracks dead code identified across the CORE-FFX codebase. Code is either:
- **Removed** - No longer present in the codebase
- **Deprecated** - Marked as deprecated, preserved for potential future use
- **Archived** - Moved to `_archive/` directory for reference
- **Consolidated** - Duplicate implementations merged into single source

---

## Summary

| Category | Files/Items | Status | Lines |
|----------|-------------|--------|-------|
| Backend deprecated modules | 1 | Deprecated | ~1100 |
| Backend #[allow(dead_code)] items | 73 | Reviewed | ~500 |
| Frontend test components | 4 | Archived | ~800 |
| Frontend dead components | 3 | Archived | ~2500 |
| Dead CSS files | 1 | Deleted | ~800 |
| Duplicate code consolidated | 1 | Consolidated | ~40 |

---

## Code Consolidation (January 17, 2026)

### Duplicate Settings Components - CONSOLIDATED

**Issue**: `SettingGroup`, `SettingRow`, and `SettingsSelect` were defined in two places:
1. Inline in `src/components/SettingsPanel.tsx` (lines 259-298)
2. As modules in `src/components/settings/` folder

**Resolution**: Removed inline definitions from `SettingsPanel.tsx`, now imports from `./settings` module.

**Files Changed**:
- `src/components/SettingsPanel.tsx` - Removed ~40 lines of duplicate component definitions

### New Standardized UI Components

Added to `src/components/ui/index.tsx`:
- `Button` - Standardized button with variants (primary, secondary, ghost, danger) and sizes
- `Spinner` - Consistent loading spinner
- `IconButton` - Icon-only button with tooltip
- `Modal` - Standardized modal overlay with backdrop, escape handling
- `ModalFooter` - Footer for modal dialogs

These replace inconsistent inline implementations found across:
- 6 modal overlays with different backdrops
- Multiple spinner implementations (icon vs border spinner)
- Inconsistent button styling

### Removed Unused Utility Function (January 17, 2026)

**`typeIcon` function removed from `src/utils.ts`**:
- **Purpose**: Returned emoji icons for container types (📦, 💿, 📱, etc.)
- **Status**: Had 6 tests but was never imported/used anywhere in actual UI code
- **Why Unused**: UI uses SVG icons from `src/components/icons/index.tsx` instead
- **Action**: Function and tests removed (~30 lines)
- **Related Used Function**: `typeClass` is still used for CSS styling

---

### Rust Code Standardization (January 17, 2026)

**Removed duplicate `format_size` function:**
- `src-tauri/examples/test_filesystem.rs` - Removed inline ~12-line `format_size()` function
- Now uses `ffx_check_lib::common::format_size_compact` from the canonical `common/hex.rs`

**Clippy auto-fixes applied:**
- Fixed `push_str()` with single-char string literals → use `push()`
- Fixed manual suffix stripping → use `strip_suffix()`
- Fixed `or_insert_with` for default values → use `or_default()`
- Fixed unnecessary reference on left operand
- Fixed redundant casts (`u32 as u32`)
- Fixed `clone` on Copy types → direct copy
- Collapsed nested `if` statements
- Fixed `std::io::Error` construction patterns
- Applied 21+ clippy suggestions

**Canonical implementations to use:**
| Function | Location | Use instead of |
|----------|----------|----------------|
| `format_size()` | `common/hex.rs` | Inline size formatters |
| `format_size_compact()` | `common/hex.rs` | Inline compact size formatters |
| `read_u32_le()` etc. | `common/binary.rs` | Format-specific duplicates |
| `bytes_to_string()` | `common/binary.rs` | Inline conversions |
| `compute_hash()` | `common/hash.rs` | Format-specific hash functions |
| `hash_segment_with_progress()` | `common/segment_hash.rs` | Format-specific segment hashing |

### Consolidated Segment Hashing (January 17, 2026)

**Issue**: `hash_single_segment` was implemented 4 times in separate modules:
- `ewf/operations.rs` (~100 lines)
- `raw.rs` (~70 lines)
- `ufed/mod.rs` (~35 lines)
- `ad1/operations.rs` (~30 lines)

**Resolution**: Created `common/segment_hash.rs` (451 lines) with unified implementation:
- `hash_segment_with_progress()` - Main hashing function
- `hash_segment()` - Convenience wrapper without progress
- `hash_segments_combined()` - Multi-segment combined hashing

**Features consolidated**:
- Memory-mapped I/O for files >64MB
- BLAKE3 parallel hashing with rayon
- Consistent progress callbacks
- Uniform error handling

**Line savings**: ~235 lines removed across 4 files, replaced with thin wrappers.

**Files changed**:
| File | Before | After | Saved |
|------|--------|-------|-------|
| `raw.rs` | 1370 | 1299 | 71 |
| `ewf/operations.rs` | 1156 | 1051 | 105 |
| `ufed/mod.rs` | 1261 | 1233 | 28 |
| `ad1/operations.rs` | 1700 | 1672 | 28 |
| **New**: `common/segment_hash.rs` | - | 451 | - |

---

## Frontend Dead Code

### Removed/Archived Files (January 17, 2026)

#### 7. `src/components/EvidenceTreeLazy.tsx` (~1800 lines) - ARCHIVED
- **Purpose**: Legacy lazy-loading evidence tree implementation
- **Why Dead**: Never imported, replaced by `EvidenceTree/` directory components
- **Action**: Moved to `_archive/frontend/`

#### 8. `src/components/Ad1OperationsV2.tsx` (~500 lines) - ARCHIVED
- **Purpose**: Alternative AD1 operations component
- **Why Dead**: Never imported by any component
- **Action**: Moved to `_archive/frontend/`

#### 9. `src/components/Responsive.tsx` (~500 lines) - ARCHIVED
- **Purpose**: Responsive design utilities (useBreakpoint, ResizablePanel, etc.)
- **Why Dead**: Exported but never actually imported/used
- **Action**: Moved to `_archive/frontend/`

#### 10. `src/styles/components.css` (~800 lines) - DELETED
- **Purpose**: Reusable component CSS classes
- **Why Dead**: File existed but was never imported - all styles duplicated in index.css
- **Action**: Deleted (not archived - fully duplicated elsewhere)

### Previously Removed Files

#### 1. `src/E01V3Test.tsx` (~376 lines) - ARCHIVED
- **Purpose**: Standalone EWF/E01 format test component
- **Why Dead**: Never imported by any other component
- **Action**: Moved to `_archive/frontend/`

#### 2. `src/RawTest.tsx` (~400 lines) - ARCHIVED  
- **Purpose**: Standalone RAW disk image test component
- **Why Dead**: Never imported by any other component
- **Action**: Moved to `_archive/frontend/`

#### 3. `src/E01V3Test.css` - ARCHIVED
- **Purpose**: Styles for E01V3Test component
- **Why Dead**: Only imported by removed test components
- **Action**: Moved to `_archive/frontend/`

#### 4. `src/components/ComponentPlayground.tsx` (~384 lines) - ARCHIVED
- **Purpose**: Development component showcase/testing
- **Why Dead**: Never imported, only exported from itself
- **Action**: Moved to `_archive/frontend/`

#### 5. `src/components/EvidenceTreeRouter.tsx` - ARCHIVED
- **Purpose**: Router for evidence tree types
- **Why Dead**: Never imported or referenced
- **Action**: Moved to `_archive/frontend/`

---

## Backend Dead Code

### Deprecated Modules (Preserved)

#### `src-tauri/src/containers/impls.rs` (~1100 lines) - DEPRECATED
- **Purpose**: Trait-based parser implementations for EvidenceContainer
- **Contains**:
  - `Ad1Parser` - AD1 container parser
  - `EwfParser` - EWF/E01 parser  
  - `RawParser` - Raw disk image parser
  - `UfedParser` - UFED container parser
  - `ArchiveParser` - Archive (ZIP/7z/RAR) parser
  - `get_parsers()` - Get all parser instances
  - `detect_parser()` - Auto-detect format and return parser
- **Why Deprecated**: 
  - Application uses direct module calls via `operations.rs` instead
  - Trait-based approach adds indirection without benefit
  - All functionality duplicated in `operations.rs`
- **Status**: Marked with `#![allow(dead_code)]` and `#[deprecated]`
- **Future Use**: Could be revived for plugin system architecture

### Items with #[allow(dead_code)]

The following items are marked as dead code but intentionally preserved:

#### `src-tauri/src/archive/sevenz.rs`
- `SevenZipEntry` struct fields - Reserved for future 7-zip features

#### `src-tauri/src/ad1/` modules
- Various utility functions in `utils.rs` for AD1 parsing diagnostics
- `operations_v2.rs` helper structs - Alternative implementation preserved

#### `src-tauri/src/ewf/` modules  
- `types.rs` - EWF header field definitions (may be needed for writes)
- `operations.rs` diagnostic functions
- `vfs.rs` partition mounting fields (future feature)

#### `src-tauri/src/containers/`
- `segments.rs` - Segment validation helpers
- `traits.rs` line 758 - ParserRegistry trait (plugin system)

#### `src-tauri/src/common/`
- `binary.rs` - Binary parsing utilities
- `progress.rs` - Progress callback types
- `filesystem/` - FAT/NTFS driver scaffolding

#### `src-tauri/src/ufed/vfs.rs`
- Virtual filesystem scaffolding for UFED

#### `src-tauri/src/raw.rs`
- Raw image parsing helpers

#### `src-tauri/src/report/`
- `mod.rs` template workflow functions
- `pdf.rs` PDF generation fields

---

## Cleanup Actions Taken

### 2026-01-15

1. **Created archive directories**:
   ```
   _archive/
   ├── frontend/
   │   ├── E01V3Test.tsx
   │   ├── E01V3Test.css
   │   ├── RawTest.tsx
   │   ├── ComponentPlayground.tsx
   │   ├── EvidenceTreeLazy.tsx
   │   └── EvidenceTreeRouter.tsx
   └── README.md
   ```

2. **Deprecated impls.rs**:
   - Added `#![allow(dead_code)]` and `#![allow(deprecated)]`
   - Added deprecation notices to all parser structs
   - Updated module documentation

3. **Removed dead LibarchiveHandler methods**:
   - `get_root_children()` - ~30 lines
   - `get_children()` - ~30 lines  
   - `entry_to_lazy_tree()` - ~35 lines

---

## How to Check for Dead Code

### Backend (Rust)
```bash
cd src-tauri
# Check for warnings
cargo check 2>&1 | grep -E "warning:|unused|dead"

# Use clippy for more thorough analysis
cargo clippy -- -W clippy::all 2>&1 | grep -E "unused|dead"

# Search for #[allow(dead_code)] markers
grep -r "#\[allow(dead_code)\]" src/ --include="*.rs"
```

### Frontend (TypeScript)
```bash
# Find components never imported
for f in src/components/*.tsx; do
  name=$(basename "$f" .tsx)
  if ! grep -rq "from.*$name\|import.*$name" src --include="*.ts" --include="*.tsx" | grep -v "^$f:"; then
    echo "Potentially unused: $f"
  fi
done

# Check ESLint for unused exports
npx eslint src --ext .ts,.tsx --rule "@typescript-eslint/no-unused-vars: error"
```

---

## Policy

1. **Test Components**: Move to `_archive/` rather than delete
2. **Deprecated Modules**: Mark with deprecation notices, preserve for future
3. **Intentional Dead Code**: Document reason with comment near `#[allow(dead_code)]`
4. **Regular Review**: Audit dead code quarterly
