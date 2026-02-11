# Code Refactoring Plan - CORE-FFX

## Overview
This document outlines a systematic plan to refactor large files into smaller, more maintainable modules.

---

## Priority 1: Backend - `commands/archive.rs` (1,529 lines)

### Current Issues
- Single 1,500+ line file handling multiple responsibilities
- Hard to navigate and maintain
- Difficult to test individual components

### Target Structure
```
src-tauri/src/commands/archive/
├── mod.rs              # Module exports and re-exports
├── metadata.rs         # Quick metadata & full tree listing (lines 1-588)
├── extraction.rs       # Entry extraction & chunk reading (lines 589-758)
├── nested.rs           # Nested container operations (lines 759-1214)
└── tools.rs            # Test/repair/validate/extract tools (lines 1215-1529)
```

### Implementation Steps

#### Step 1: Create `metadata.rs`
**Lines to move:** 1-588 from archive.rs

**Contents:**
- `ArchiveTreeEntry` struct
- `ArchiveQuickMetadata` struct
- `archive_get_metadata()` command
- `archive_get_tree()` command

**Dependencies:**
```rust
use tracing::debug;
use crate::archive;
use crate::common::filesystem::FilesystemDriver;
```

#### Step 2: Create `extraction.rs`
**Lines to move:** 589-758 from archive.rs

**Contents:**
- `archive_extract_entry()` command
- `archive_read_entry_chunk()` command

**Dependencies:**
```rust
use tracing::debug;
use crate::archive;
use std::sync::Arc;
use parking_lot::RwLock;
```

#### Step 3: Create `nested.rs`
**Lines to move:** 759-1214 from archive.rs

**Contents:**
- `NestedContainerEntry` struct
- `NestedContainerInfo` struct
- `get_or_create_nested_temp()` helper
- `detect_nested_container_type()` helper
- `is_container_filename()` helper
- `nested_archive_read_entry_chunk()` command
- `nested_container_get_tree()` command
- `nested_container_get_info()` command
- `nested_container_clear_cache()` command

**Dependencies:**
```rust
use tracing::debug;
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::collections::HashMap;
```

#### Step 4: Create `tools.rs`
**Lines to move:** 1215-1529 from archive.rs

**Contents:**
- `test_7z_archive()` command
- `repair_7z_archive()` command
- `validate_7z_archive()` command
- `extract_split_7z_archive()` command
- `get_last_archive_error()` command
- `clear_last_archive_error()` command
- `compress_to_lzma()` command
- `decompress_lzma()` command

**Dependencies:**
```rust
use tracing::{debug, info};
use tauri::{Window, Emitter};
use seven_zip::SevenZip;
```

#### Step 5: Update `commands/mod.rs`
Change:
```rust
pub mod archive;
```
To:
```rust
pub mod archive;
// All exports are handled in archive/mod.rs
```

#### Step 6: Update `lib.rs` command registration
No changes needed - all commands are re-exported from `archive/mod.rs`

### Testing After Refactoring
```bash
cd src-tauri
cargo check              # Verify compilation
cargo test               # Run tests
npm run tauri dev        # Test in UI
```

---

## Priority 2: Backend - `project.rs` (1,904 lines)

### Target Structure
```
src-tauri/src/project/
├── mod.rs              # Main save/load functions
├── types.rs            # ProjectV2 core struct
├── evidence.rs         # Evidence cache types
├── ui_state.rs         # UI state types (tabs, layout, filters)
├── hash_history.rs     # Hash computation history
├── migrations.rs       # Version migration logic
└── cache.rs            # Preview/file caching logic
```

### Key Splits

**types.rs** (lines 1-200):
- `ProjectV2` struct
- Core metadata types

**evidence.rs** (lines 225-320):
- `EvidenceCache` struct
- `CachedFile` struct
- `CachedHash` struct

**ui_state.rs** (lines 286-370):
- `CenterPaneState` struct
- `TabInfo` struct
- Layout preferences

**hash_history.rs** (lines 416-440):
- Hash computation history types

**cache.rs** (lines 320-416):
- `PreviewCache` struct
- `PreviewEntry` struct
- `CaseDocumentsCache` struct

**migrations.rs** (lines 1200-1904):
- Version migration functions
- Legacy format converters

---

## Priority 3: Frontend - `ReportWizard.tsx` (2,042 lines)

### Target Structure
```
src/components/report/
├── ReportWizard.tsx              # Main wizard container
├── wizard/
│   ├── WizardContext.tsx         # Shared state (already exists)
│   ├── steps/
│   │   ├── SelectionStep.tsx     # Evidence selection
│   │   ├── OptionsStep.tsx       # Report options
│   │   ├── TemplateStep.tsx      # Template selection
│   │   └── GenerationStep.tsx    # Generation & progress
│   └── components/
│       ├── ProgressTracker.tsx   # Progress bar
│       └── ReportPreview.tsx     # Preview panel
```

### Benefits
- Each step is self-contained
- Easier to test individual steps
- Better code organization
- Simpler state management

---

## Priority 4: Frontend - `ExportPanel.tsx` (1,163 lines)

### Target Structure
```
src/components/export/
├── ExportPanel.tsx                # Main panel & mode switcher
├── modes/
│   ├── ExportMode.tsx             # File export UI
│   ├── ArchiveMode.tsx            # Archive creation UI
│   └── ToolsMode.tsx              # Archive tools UI
├── handlers/
│   ├── exportHandlers.ts          # Export logic
│   ├── archiveHandlers.ts         # Archive logic
│   └── toolHandlers.ts            # Tool logic
└── components/
    ├── SourceSelector.tsx         # File/folder picker
    └── DestinationPicker.tsx      # Destination selector
```

### Implementation Steps

**ExportMode.tsx** (300 lines):
- Source/destination pickers
- Export options checkboxes
- Export handler integration

**ArchiveMode.tsx** (350 lines):
- Source/destination pickers
- Compression options
- Password field
- Advanced options
- Archive handler integration

**ToolsMode.tsx** (250 lines):
- Tab selector (test/repair/validate/extract)
- Tool-specific UI for each tab
- Tool handler integration

**handlers/** (150 lines each):
- Pure business logic
- No UI components
- Easy to test
- Reusable

---

## Priority 5: Frontend - `SettingsPanel.tsx` (1,074 lines)

### Target Structure
```
src/components/settings/
├── SettingsPanel.tsx              # Main container & tab switcher
└── tabs/
    ├── GeneralTab.tsx             # General settings
    ├── InterfaceTab.tsx           # UI preferences
    ├── StorageTab.tsx             # Storage paths
    ├── ProcessedTab.tsx           # Processed DB settings
    └── AdvancedTab.tsx            # Advanced options
```

### Each Tab (150-200 lines):
- Self-contained settings group
- Clear responsibilities
- Easy to maintain

---

## Additional Candidates for Refactoring

### Backend
- `ad1/operations.rs` (1,675 lines) → split into tree.rs, extraction.rs, metadata.rs
- `ewf/parser.rs` (1,426 lines) → split by section type parsing
- `containers/operations.rs` (1,160 lines) → split by operation type

### Frontend
- `DetailPanelContent.tsx` (682 lines) → split by content type
- `MetadataPanel.tsx` (648 lines) → split by metadata category

---

## General Refactoring Principles

### Before Refactoring
1. ✅ Ensure all tests pass
2. ✅ Commit current working state
3. ✅ Create feature branch: `git checkout -b refactor/module-name`

### During Refactoring
1. ✅ Move one module at a time
2. ✅ Keep git commits small and focused
3. ✅ Test after each module
4. ✅ Update imports immediately

### After Refactoring
1. ✅ Run full test suite
2. ✅ Test in UI manually
3. ✅ Update documentation
4. ✅ Merge when stable

---

## Benefits of Refactoring

### Maintainability
- **Easier Navigation**: Find code quickly by logical grouping
- **Clear Responsibilities**: Each module has one purpose
- **Reduced Complexity**: Smaller files are easier to understand

### Testing
- **Unit Tests**: Test modules independently
- **Mocking**: Easier to mock dependencies
- **Coverage**: Better test coverage tracking

### Development
- **Parallel Work**: Multiple developers can work without conflicts
- **Code Review**: Smaller PRs are easier to review
- **Onboarding**: New developers understand structure faster

### Performance
- **Compilation**: Rust incremental compilation works better
- **Hot Reload**: Frontend HMR is faster with smaller files

---

## Estimated Timeline

| Task | Estimated Time | Complexity |
|------|---------------|------------|
| archive.rs refactoring | 2-3 hours | Medium |
| project.rs refactoring | 3-4 hours | High |
| ReportWizard.tsx | 2-3 hours | Medium |
| ExportPanel.tsx | 2-3 hours | Medium |
| SettingsPanel.tsx | 1-2 hours | Low |
| **Total** | **10-15 hours** | **Mixed** |

---

## Implementation Order

1. **Start with `archive.rs`** - Clear boundaries, medium risk
2. **Then `ExportPanel.tsx`** - Just refactored, fresh in memory
3. **Then `SettingsPanel.tsx`** - Simple split, low risk
4. **Then `ReportWizard.tsx`** - More complex but high value
5. **Finally `project.rs`** - Most complex, save for last

---

## Notes

- Create backup branch before starting
- Test thoroughly after each module
- Update CODE_BIBLE.md with new structure
- Consider adding README.md in each new directory
- Keep original files until refactoring is complete and tested

---

## Status Tracking

- [x] **archive.rs → archive/ (4 modules)** ✅ COMPLETED
  - Successfully refactored 1,530-line file into modular structure
  - `metadata.rs` (591 lines) - Metadata extraction and tree listing
  - `extraction.rs` (134 lines) - Entry extraction and chunk reading
  - `nested.rs` (515 lines) - Nested container support
  - `tools.rs` (327 lines) - Archive tools (test/repair/validate)
  - `mod.rs` (43 lines) - Module entry point with re-exports
  - **Solution:** Use fully-qualified paths in lib.rs (e.g., `commands::archive::metadata::archive_get_tree`)
  - **Build Status:** ✅ Compiles successfully with `cargo build`
- [ ] project.rs → project/ (6 modules)
- [ ] ReportWizard.tsx → report/ (6 files)
- [ ] ExportPanel.tsx → export/ (7 files)
- [ ] SettingsPanel.tsx → settings/ (6 files)
