# Testing Project Save/Load V2

## Quick Test Procedure

### 1. Basic Save/Load Test

1. **Launch the application**

   ```bash
   npm run tauri dev
   ```

2. **Create some project state**:
   - Open an evidence directory (Browse or scan for files)
   - Open one or more evidence containers (E01, AD1, etc.)
   - Compute hashes on some files
   - Add a bookmark (if feature is available)
   - Create a note (if feature is available)
   - Adjust panel widths and expand/collapse panels

3. **Save the project**:
   - Click the Save button in toolbar (💾 icon)
   - Project should save to `<evidence-dir>/../<dirname>.cffx`
   - Look for success toast notification

4. **Close and reopen the application**

5. **Load the project**:
   - Click the Load button in toolbar (📂 icon)
   - Select the `.cffx` file you just saved
   - Or browse to the parent directory of your evidence folder

6. **Verify restoration**:
   - ✅ Evidence directory is restored
   - ✅ Open tabs are restored (with same order)
   - ✅ Hash history is shown for previously hashed files
   - ✅ Panel widths are restored
   - ✅ Panel collapsed states are restored
   - ✅ Bookmarks are restored (if any were created)
   - ✅ Notes are restored (if any were created)
   - ✅ Last active tab is selected

### 2. Verify V2 Fields in File

Open the saved `.cffx` file in a text editor and verify these v2 fields are present:

```json
{
  "version": 2,
  "project_id": "proj_...",
  "created_by_version": "0.1.0",
  "saved_by_version": "0.1.0",
  "users": [],
  "sessions": [],
  "activity_log": [],
  "open_directories": [],
  "file_selection": {},
  "processed_databases": {},
  "bookmarks": [],
  "notes": [],
  "tags": [],
  "reports": [],
  "saved_searches": [],
  "recent_searches": [],
  "filter_state": {},
  "ui_state": {
    "left_panel_width": 320,
    "right_panel_width": 280,
    "left_panel_collapsed": false,
    "right_panel_collapsed": true,
    "left_panel_tab": "evidence",
    "detail_view_mode": "info"
  },
  "settings": {}
}
```

### 3. Backward Compatibility Test

1. **Create a v1 project file** (for testing):
   - Create a `.cffx` file with only v1 fields:

   ```json
   {
     "version": 1,
     "name": "TestCase",
     "root_path": "/path/to/evidence",
     "created_at": "2026-01-15T10:00:00Z",
     "saved_at": "2026-01-15T10:00:00Z",
     "app_version": "0.1.0",
     "tabs": [
       {"file_path": "/path/to/evidence/test.E01", "name": "test.E01", "order": 0}
     ],
     "active_tab_path": "/path/to/evidence/test.E01",
     "hash_history": {"files": {}},
     "ui_state": {},
     "notes": [],
     "tags": []
   }
   ```

2. **Load the v1 file**
   - Should load without errors
   - Should automatically upgrade to v2
   - Missing v2 fields should have defaults

3. **Save the project**
   - Should save as v2
   - Should include all new v2 fields

### 4. Data Persistence Test

Test that v2 fields persist across multiple save/load cycles:

1. Open project
2. Add a bookmark → Save → Close
3. Load project → Verify bookmark exists
4. Add a note → Save → Close
5. Load project → Verify both bookmark and note exist
6. Compute a hash → Save → Close
7. Load project → Verify bookmark, note, and hash all exist

### 5. Console Verification

Check the browser console (for frontend) and terminal (for backend) for:

**On Save:**

- `Project saved to: /path/to/file.cffx`
- `Project saved: <name> (<count> tabs)`

**On Load:**

- `Loading project from: /path/to/file.cffx`
- `Project loaded: <name> (<count> tabs)`
- No errors about missing fields
- No warnings about unknown fields

### 6. Expected File Size

A typical v2 project file with moderate usage should be:

- Empty project: ~1-2 KB
- With open tabs and hashes: ~5-10 KB
- With bookmarks, notes, activity: ~20-50 KB
- Heavy usage (lots of history): ~100-500 KB

If the file is significantly smaller than expected, v2 fields might not be serializing.

## Troubleshooting

### Problem: Save button does nothing

- Check console for errors
- Verify evidence directory is opened
- Check file permissions in parent directory

### Problem: Load restores only basic fields

- Open the `.cffx` file and check if v2 fields are present
- If missing, backend isn't serializing properly
- Check Rust cargo build output for errors

### Problem: v1 file won't load

- Check if file format is valid JSON
- Ensure all required v1 fields are present
- Check backend logs for deserialization errors

### Problem: Project file is empty or corrupted

- Check if the file exists at expected location
- Verify write permissions
- Check if serialization is producing valid JSON

## Automated Testing

Run the Rust tests:

```bash
cd src-tauri
cargo test --lib project
```

All 12 tests should pass:

- ✅ test_project_new
- ✅ test_project_new_with_trailing_slash
- ✅ test_project_default_save_path
- ✅ test_get_default_project_path
- ✅ test_project_touch
- ✅ test_project_serialization_roundtrip
- ✅ test_project_hash_history_default
- ✅ test_project_ui_state_default
- ✅ test_project_load_result_success
- ✅ test_project_load_result_failure
- ✅ test_project_save_result_success
- ✅ test_project_constants

## Success Criteria

All manual and automated tests pass, demonstrating full v2 schema support.
