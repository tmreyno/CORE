# Evidence Tree Container Test - Quick Reference

## Current Test Files Available

1. **macwd.E01** (24MB) - BEST FOR QUICK TESTING
2. **PST.7z** (33MB) - Test archive support
3. **Google Pixel TAR** (9.1GB) - Large archive test
4. **ChatGPT.dmg** (50MB) - Unsupported format test

## Quick Test Procedure (5 minutes)

### Setup
1. Application is already running (detected at PID 32520)
2. Open CORE-FFX window
3. Verify Evidence Tree panel is visible on left

### Test 1: E01 Container (2 min)
**File:** macwd.E01

Steps:
1. Look for "macwd.E01" in Evidence Tree
2. Click the chevron (▶) icon next to it
3. **WATCH FOR:**
   - Loading indicator appears briefly
   - Tree expands showing partitions/folders
   - Should complete in < 5 seconds
4. Click on a folder to expand it
5. Click on a file to view content
6. **EXPECTED:** Content viewer shows file details on right

**✅ PASS if:** Tree expands, folders/files visible, content viewable
**❌ FAIL if:** Error message, hangs, or crash

### Test 2: 7z Archive (1 min)
**File:** PST.7z

Steps:
1. Find "PST.7z" in Evidence Tree
2. Click chevron to expand
3. **WATCH FOR:**
   - Metadata loads almost instantly (< 100ms)
   - Info bar shows "X files, Y folders"
   - Full tree appears within 1-2 seconds
4. Expand a folder and click a file

**✅ PASS if:** Fast load, tree structure correct, files viewable
**❌ FAIL if:** Slow (> 3 sec), error, or no structure

### Test 3: Unsupported Format (30 sec)
**File:** ChatGPT.dmg

Steps:
1. Find "ChatGPT.dmg" in Evidence Tree
2. Click chevron to expand
3. **EXPECTED:** Error message like:
   - "Format 'dmg' not supported"
   - "VFS mounting failed"
   - Or similar clear message
4. **VERIFY:** Application doesn't crash, other containers still work

**✅ PASS if:** Clear error, app stable
**❌ FAIL if:** Crash or confusing error

## Browser Console Monitoring

Open DevTools (Cmd+Option+I) and watch Console for:
- **Green/Blue messages:** Normal operation
- **Orange warnings:** Non-critical issues
- **Red errors:** Problems that need fixing

Look for these messages:
- `[DEBUG] EvidenceTree: loadArchiveTree - got X entries` (good)
- `VFS mounted successfully` (good for E01)
- `Failed to mount` (bad - report this)
- Any stack traces (bad - screenshot and report)

## Quick Diagnosis

### If E01 won't expand:
- Check console for: "libewf" or "mount" errors
- Try different E01 file (2020JimmyWilson.E01)
- Check Terminal for Rust errors

### If 7z won't expand:
- Check console for: "7z" or "archive" errors
- Verify file isn't corrupted: `7z l PST.7z` in Terminal

### If app crashes:
- Check Terminal for panic messages
- Note which file caused crash
- Try restarting app and testing other files

## Expected Performance

| Operation | Expected Time |
|-----------|---------------|
| E01 mount (< 1GB) | < 5 sec |
| 7z metadata | < 100 ms |
| 7z full tree | < 2 sec |
| File click → content | < 500 ms |

## Report Results

After testing, note:
- ✅ Which tests passed
- ❌ Which tests failed
- ⚠️ Any warnings or unusual behavior
- Screenshot any errors

