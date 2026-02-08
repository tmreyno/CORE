# 🧪 CORE-FFX Evidence Tree - Quick Test Guide

**Test Date:** February 7, 2026  
**Purpose:** Verify all container and archive features work correctly

---

## 📦 Available Test Files

Based on scan of `1827-1001 Case With Data /1.Evidence/`:

| File | Size | Type | Priority |
|------|------|------|----------|
| `PC-MUS-001.E01` | 49GB | E01 (EnCase) | 🔴 HIGH - Large E01, test VFS mounting |
| `Google Pixel 3a XL Logical Image - Data.tar` | 9.1GB | TAR Archive | 🔴 HIGH - Large archive, test lazy loading |
| `2020JimmyWilson.E01` | 295MB | E01 (EnCase) | 🟢 MEDIUM - Good size for testing |
| `ChatGPT.dmg` | 50MB | DMG (Apple) | 🟡 MEDIUM - Test unsupported format |
| `PST.7z` | 33MB | 7z Archive | 🟢 MEDIUM - Test 7z support |
| `macwd.E01` | 24MB | E01 (EnCase) | 🟢 LOW - Quick test |

---

## ✅ Quick Test Checklist (5 Minutes)

### 1. **E01 Container Test** (2 min)
**File:** `macwd.E01` (24MB - fast)

- [ ] Open CORE-FFX application
- [ ] Load project: `1827-1001.ffxproj`
- [ ] Find `macwd.E01` in Evidence Tree
- [ ] Click chevron to expand
- [ ] **Expected:** VFS mounts, partitions shown, < 5 seconds
- [ ] Navigate into partition folders
- [ ] Click a file to view content
- [ ] **Expected:** File content loads in Content Viewer

**✅ PASS if:** Tree expands, files are clickable, content viewer works  
**❌ FAIL if:** Error message, no partitions, or crash

---

### 2. **Archive Container Test** (1 min)
**File:** `PST.7z` (33MB)

- [ ] Find `PST.7z` in Evidence Tree
- [ ] Click chevron to expand
- [ ] **Expected:** Metadata loads instantly (< 100ms)
- [ ] **Expected:** Shows "X files, Y folders" info bar
- [ ] **Expected:** Full tree loads within 1-2 seconds
- [ ] Expand nested directories
- [ ] Click a file inside archive
- [ ] **Expected:** File content displays

**✅ PASS if:** Fast metadata, tree loads, files viewable  
**❌ FAIL if:** Slow (> 3 sec), error, or no tree structure

---

### 3. **Large Archive Test** (2 min)
**File:** `Google Pixel 3a XL Logical Image - Data.tar` (9.1GB)

- [ ] Find tar file in Evidence Tree
- [ ] Click chevron to expand
- [ ] **Expected:** Metadata loads quickly
- [ ] **Expected:** Tree loads progressively (may take 10-30 seconds for 9GB)
- [ ] Scroll through entries - should be smooth
- [ ] Expand a folder with many files
- [ ] **Expected:** No lag, smooth scrolling

**✅ PASS if:** Loads without crashing, scrolling smooth  
**❌ FAIL if:** Hangs, crashes, or extremely slow (> 60 sec)

---

### 4. **Unsupported Format Test** (30 sec)
**File:** `ChatGPT.dmg` (50MB - Apple Disk Image)

- [ ] Find `ChatGPT.dmg` in Evidence Tree
- [ ] Click chevron to expand
- [ ] **Expected:** Error message: "Format 'dmg' not supported"
- [ ] **Expected:** Helpful hint displayed
- [ ] **Expected:** Application remains stable

**✅ PASS if:** Clear error message, no crash  
**❌ FAIL if:** Crash, hang, or misleading error

---

### 5. **Large E01 Test** (Optional - takes longer)
**File:** `PC-MUS-001.E01` (49GB)

⚠️ **Warning:** This is a large file. VFS mount may take 10-30 seconds.

- [ ] Find `PC-MUS-001.E01` in Evidence Tree
- [ ] Click chevron to expand
- [ ] Wait for VFS mount (watch console logs)
- [ ] **Expected:** Mounts successfully, partitions detected
- [ ] Navigate file system
- [ ] **Expected:** Smooth navigation, no lag

**✅ PASS if:** Mounts and navigates smoothly  
**❌ FAIL if:** Timeout, crash, or extreme slowness

---

## 🔍 Detailed Feature Tests

### Test A: Multiple Containers Open
**Goal:** Verify app handles multiple expanded containers

1. Expand `macwd.E01`
2. Expand `PST.7z`
3. Expand `2020JimmyWilson.E01`
4. Navigate between all three
5. **Expected:** All remain expanded, no performance issues

### Test B: File Content Viewing
**Goal:** Verify different file types display correctly

1. Open any E01 or archive
2. Find and click these file types (if available):
   - `.txt` or `.log` → Text Viewer
   - `.jpg` or `.png` → Image Viewer
   - `.pdf` → Document Viewer
   - `.exe` or `.dll` → Binary Viewer
   - `.db` or `.sqlite` → Database Viewer

3. **Expected:** Appropriate viewer loads for each type

### Test C: Nested Containers
**Goal:** Verify archives within containers can be opened

1. Expand any E01 container
2. Navigate to find a ZIP/TAR/7z file inside
3. Look for nested container indicator
4. Attempt to expand the nested archive
5. **Expected:** Can open archive within E01

### Test D: Search & Filter
**Goal:** Verify search works across containers

1. Expand 2-3 containers
2. Use search bar (if available)
3. Search for common extension like `.txt` or `.log`
4. **Expected:** Results from all containers
5. Click result → navigates to file

### Test E: Export from Tree
**Goal:** Verify export functionality works

1. Expand any container
2. Select a file (or multiple with Cmd+Click)
3. Click "Export" button
4. Choose output directory
5. **Expected:** Files exported successfully
6. Verify exported files match originals (check size/hash)

---

## 🐛 Common Issues to Watch For

| Issue | Symptom | Expected Behavior |
|-------|---------|-------------------|
| **VFS Mount Timeout** | E01 doesn't expand after 30+ sec | Should mount in < 10 sec for most files |
| **Archive Load Hang** | Archive expansion freezes UI | Should load progressively, never freeze |
| **Memory Leak** | App slows down over time | Memory should stay < 500 MB with 5 containers |
| **Crash on Expand** | App crashes when opening container | Should show error message, not crash |
| **Content Not Loading** | File clicks don't show content | Content Viewer should always respond |
| **Nested Container Fail** | Can't open ZIP inside E01 | Should detect and allow extraction |

---

## 📊 Performance Benchmarks

| Operation | Expected Time | Acceptable | Slow/Issue |
|-----------|---------------|------------|------------|
| E01 VFS Mount (< 1GB) | < 5 sec | < 10 sec | > 15 sec |
| Archive Metadata | < 100 ms | < 500 ms | > 1 sec |
| Archive Full Tree (< 10K entries) | < 2 sec | < 5 sec | > 10 sec |
| File Click → Content Load | < 500 ms | < 1 sec | > 2 sec |
| Scroll Performance | 60 FPS | 30 FPS | < 20 FPS |

---

## 🎯 Test Results

### Quick Test Results (5 min tests)

| Test | File | Status | Time | Notes |
|------|------|--------|------|-------|
| E01 Small | macwd.E01 | ⬜ | ____ sec | |
| Archive 7z | PST.7z | ⬜ | ____ sec | |
| Archive Large | Google...tar | ⬜ | ____ sec | |
| Unsupported | ChatGPT.dmg | ⬜ | N/A | |
| E01 Large | PC-MUS-001.E01 | ⬜ | ____ sec | |

**Legend:** ✅ Pass | ❌ Fail | ⚠️ Warning | ⏭️ Skipped

### Detailed Feature Test Results

| Test | Status | Notes |
|------|--------|-------|
| Multiple Containers | ⬜ | |
| File Content Viewing | ⬜ | |
| Nested Containers | ⬜ | |
| Search & Filter | ⬜ | |
| Export from Tree | ⬜ | |

---

## 🔧 Troubleshooting

### If E01 won't mount:
1. Check console for libewf errors
2. Verify file isn't corrupted: `file macwd.E01`
3. Check if segments are missing (E01, E02, etc.)
4. Try smaller E01 file first

### If archive won't load:
1. Check if file is actually an archive: `file PST.7z`
2. Verify 7z is installed (backend dependency)
3. Try different archive (ZIP instead of 7z)
4. Check console for extraction errors

### If app is slow:
1. Check memory usage (Activity Monitor / Task Manager)
2. Close other containers to free resources
3. Restart application
4. Try smaller test files first

### If content viewer doesn't work:
1. Check file is actually selected (highlighted blue)
2. Verify file has content (not 0 bytes)
3. Check console for viewer errors
4. Try different file type

---

## 📞 Report Issues

If tests fail, collect this information:

1. **Test that failed:** (e.g., "E01 Small - macwd.E01")
2. **Error message:** (screenshot or copy from console)
3. **Console logs:** (Browser DevTools → Console)
4. **File details:** (size, type, source)
5. **Steps to reproduce:** (exact clicks/actions)
6. **System info:** (OS, RAM, CORE-FFX version)

**Submit to:** GitHub Issues or project team

---

## ✨ Success Criteria

**All tests PASS if:**
- ✅ All container types can be expanded
- ✅ Tree navigation is smooth (no lag)
- ✅ Files can be viewed in Content Viewer
- ✅ Performance meets benchmarks
- ✅ Errors are handled gracefully (no crashes)
- ✅ Multiple containers work simultaneously

**READY FOR PRODUCTION** ✨

---

**Tester:** ____________________  
**Date:** ____________________  
**Version:** v2.0.0  
**Platform:** macOS

