# Project Save/Load V2 Upgrade

## Summary

Updated the Rust backend to fully support the v2 project schema, fixing the issue where project data was not being properly saved and loaded.

## Problem

The frontend was sending a comprehensive v2 project structure with 20+ fields (sessions, activity logs, bookmarks, notes, reports, processed databases, etc.), but the Rust backend only supported v1 with ~10 basic fields. This caused:

- **Data loss**: All v2 fields were silently dropped on save
- **Incomplete restoration**: Loading a project only restored basic v1 fields
- **Lost work**: Users lost bookmarks, activity logs, session history, and more between sessions

## Solution

Upgraded the Rust backend (`src-tauri/src/project.rs`) to match the TypeScript schema (`src/types/project.ts`):

### Changes Made

1. **Updated PROJECT_VERSION from 1 to 2**
2. **Expanded FFXProject struct** with all v2 fields:
   - Metadata (project_id, description, version tracking)
   - Users & Sessions (full session management)
   - Activity Log (complete audit trail)
   - Evidence State (open directories, file selection, locations)
   - Processed Databases (full state with integrity tracking)
   - Bookmarks & Notes (arrays of structured objects)
   - Tags (structured tag definitions)
   - Reports (report generation history)
   - Searches (saved and recent searches)
   - Filter State (active filters and sorting)
   - UI State (comprehensive panel/view state)
   - Settings (project-specific configuration)
   - Custom Data (extensibility)

3. **Added supporting types** (30+ new structs):
   - ProjectUser, ProjectSession, ActivityLogEntry
   - OpenDirectory, RecentDirectory, ProjectLocations
   - FileSelectionState
   - ProcessedDatabaseState, ProcessedDbIntegrity, ProcessedDbWorkMetrics
   - ProjectBookmark, ProjectNote, ProjectTag
   - ProjectReportRecord
   - SavedSearch, RecentSearch, FilterState
   - TreeNodeState, WindowDimensions, UIPreferences
   - ProjectSettings

4. **Implemented proper defaults**:
   - Custom Default impl for ProjectUIState
   - Helper functions for default values
   - `#[serde(default)]` attributes for optional fields

5. **Backward compatibility**:
   - Preserved legacy `app_version` field (maps to `saved_by_version`)
   - All v1 projects automatically upgrade on load
   - Missing v2 fields get sensible defaults

6. **Updated tests**:
   - Fixed all failing tests
   - Updated version checks
   - Enhanced serialization roundtrip tests
   - All 12 tests passing

7. **Updated documentation**:
   - Revised FFX_PROJECT_FORMAT.md
   - Added comprehensive v2 schema documentation

## Files Modified

- `src-tauri/src/project.rs` - Complete v2 schema implementation (~500 lines added)
- `src-tauri/FFX_PROJECT_FORMAT.md` - Updated documentation
- `PROJECT_V2_UPGRADE.md` - This file

## Testing

```bash
# Run project tests
cd src-tauri
cargo test --lib project::tests
# Result: ok. 12 passed; 0 failed

# Build application
npm run tauri build -- --debug
# Result: Build successful
```

## Impact

### Before

- ❌ Only ~10 basic fields saved (tabs, hash history, simple UI state)
- ❌ Lost bookmarks, notes, activity logs, sessions
- ❌ Lost processed database state
- ❌ Lost search history and filters
- ❌ Incomplete UI restoration

### After

- ✅ Full v2 schema with 20+ major fields
- ✅ Complete session and activity tracking
- ✅ Bookmarks and notes preserved
- ✅ Processed database state maintained
- ✅ Search history and filters restored
- ✅ Complete UI state restoration
- ✅ Backward compatible with v1 files

## Migration Path

No manual migration needed:

1. v1 projects automatically upgrade on first load
2. Missing v2 fields initialized with defaults
3. All v1 data preserved
4. New v2 fields start empty and populate during use

## Future Enhancements

Potential improvements for future versions:

- Add migration warnings for significant schema changes
- Implement schema versioning for field-level migrations
- Add data validation on load
- Implement automatic backups before major version upgrades
- Add telemetry for tracking which fields are actually used

## Date

January 15, 2026
