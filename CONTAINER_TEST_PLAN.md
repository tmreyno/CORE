# CORE-FFX Container & Archive Testing Plan

**Date:** February 7, 2026  
**Purpose:** Comprehensive testing of all container and archive format support in Evidence Tree

---

## Test Objectives

1. ✅ Verify all container types can be discovered and loaded
2. ✅ Verify tree expansion and navigation works correctly
3. ✅ Verify file contents can be viewed in the tree
4. ✅ Verify nested containers can be opened (archives within containers)
5. ✅ Test performance with large containers
6. ✅ Verify error handling for corrupted/unsupported formats

---

## Supported Container Formats

### 1. **AD1 (AccessData Logical Evidence)**
- **Extensions:** `.ad1`
- **Features:**
  - Tree-based file system structure
  - Partition support
  - Metadata preservation
  - Hash verification
  - Segment support (multi-file AD1)
- **Test Files:** Look for `*.ad1` in evidence directory

### 2. **E01/EWF (EnCase Evidence Format)**
- **Extensions:** `.e01`, `.ex01`, `.ewf`
- **Features:**
  - VFS mounting (disk image)
  - Partition detection
  - File system browsing
  - Hash verification
  - Split segment support (`.e01`, `.e02`, etc.)
- **Test Files:** Look for `*.E01`, `*.e01` in evidence directory

### 3. **L01 (Logical Evidence File)**
- **Extensions:** `.l01`, `.lx01`, `.lvf`
- **Features:**
  - VFS mounting (logical collection)
  - File system structure
  - Metadata preservation
- **Test Files:** Look for `*.l01` in evidence directory

### 4. **Archives (ZIP, 7z, TAR, RAR)**
- **Extensions:** `.zip`, `.7z`, `.rar`, `.tar`, `.gz`, `.tgz`, `.tar.gz`, `.tar.bz2`, `.tar.xz`
- **Features:**
  - Fast metadata loading (header-only reads)
  - Full tree browsing
  - Nested archive support (archives within archives)
  - Directory expansion
  - Extract on demand
- **Test Files:** Look for archives in evidence directory

### 5. **UFED (Cellebrite Mobile Extraction)**
- **Extensions:** `.ufd`, `.ufdr`, `.ufdx`
- **Features:**
  - Lazy loading (large datasets)
  - Paginated tree navigation
  - "Load More" functionality
  - Mobile artifact detection
- **Test Files:** Look for `*.ufd`, `*.ufdr` in evidence directory

### 6. **Raw Disk Images**
- **Extensions:** `.raw`, `.dd`, `.img`, `.001`
- **Features:**
  - VFS mounting
  - Partition detection
  - File system browsing
- **Test Files:** Look for raw images in evidence directory

### 7. **ISO Images**
- **Extensions:** `.iso`
- **Features:**
  - VFS mounting (ISO 9660)
  - Read-only file system browsing
- **Test Files:** Look for `*.iso` in evidence directory

---

## Test Procedure

### Phase 1: Container Discovery

**Steps:**
1. Launch CORE-FFX application
2. Navigate to "1827-1001 Case With Data" workspace
3. Click "Add Evidence" button
4. Browse to `1.Evidence/` directory
5. Observe discovered containers

**Expected Results:**
- All supported container types are discovered
- Container types are correctly identified (AD1, E01, ZIP, UFED, etc.)
- File sizes are displayed correctly
- Type filter badges show correct counts

**Test Data:**
```bash
# Run this to see available containers:
cd "/Users/terryreynolds/1827-1001 Case With Data/1.Evidence"
ls -lh *.{E01,e01,ad1,AD1,zip,ZIP,7z,tar,gz,ufd,UFD,ufdr,dmg,iso} 2>/dev/null
```

---

### Phase 2: Container Expansion & Tree Navigation

#### Test 2.1: AD1 Container
**Container:** `*.ad1` files  
**Steps:**
1. Click the chevron next to an AD1 container
2. Wait for tree to load
3. Observe root children (partitions/folders)
4. Expand nested directories
5. Verify folder/file icons are correct

**Expected Results:**
- ✅ Container expands smoothly
- ✅ Partition structure is displayed (if multi-partition)
- ✅ Folders show folder icon, files show document icon
- ✅ File sizes displayed correctly
- ✅ Expand/collapse animation works

**Performance:** Should load in < 2 seconds for containers with < 10K entries

---

#### Test 2.2: E01/EWF Container
**Container:** `*.E01` files  
**Steps:**
1. Click the chevron next to an E01 container
2. Wait for VFS mount
3. Observe partition detection
4. Navigate through file system (C:\, /Users, etc.)
5. Expand deep directory structures

**Expected Results:**
- ✅ VFS mounts successfully (check console for "Mounted successfully")
- ✅ Partitions are detected and displayed
- ✅ File system structure is navigable
- ✅ NTFS/FAT32/EXT4/APFS file systems supported
- ✅ Deleted files shown (if supported)

**Performance:** VFS mount should complete in < 5 seconds

---

#### Test 2.3: Archive Container (ZIP/7z)
**Container:** `*.zip`, `*.7z` files  
**Steps:**
1. Click the chevron next to an archive
2. Observe fast metadata load (nearly instant)
3. Wait for full tree load
4. Expand nested directories within archive
5. Test deeply nested archives (archive within archive)

**Expected Results:**
- ✅ Metadata loads instantly (< 100ms)
- ✅ Container shows "X files, Y folders" summary
- ✅ Full tree loads in background
- ✅ Directory structure is correct
- ✅ Compressed vs uncompressed size displayed

**Performance:** 
- Metadata: < 100ms
- Full tree (< 1000 entries): < 500ms
- Full tree (< 10K entries): < 2 seconds

---

#### Test 2.4: UFED Container
**Container:** `*.ufd`, `*.ufdr` files  
**Steps:**
1. Click the chevron next to UFED container
2. Observe lazy loading (first 100 entries)
3. Scroll to bottom and click "Load More"
4. Expand mobile data folders (Contacts, Messages, Media)
5. Test performance with large UFED extractions

**Expected Results:**
- ✅ First 100 entries load quickly
- ✅ "Load More" button appears when more entries available
- ✅ Subsequent pages load on demand
- ✅ Mobile artifacts organized by category
- ✅ No performance degradation with large datasets (100K+ entries)

**Performance:** Each page (100 entries) should load in < 1 second

---

### Phase 3: File Content Viewing

#### Test 3.1: File Selection in Tree
**Steps:**
1. Expand any container (AD1, E01, Archive, UFED)
2. Click on a file entry (not folder)
3. Observe Content Viewer panel on the right

**Expected Results:**
- ✅ File is selected (highlighted in blue)
- ✅ Content Viewer shows file details (name, size, path, hash)
- ✅ File content is displayed (text, hex, image, document)
- ✅ Appropriate viewer is selected based on file type

---

#### Test 3.2: Different File Types
Test viewing these file types within containers:

| File Type | Expected Viewer | Test Files |
|-----------|----------------|------------|
| **Text Files** | Text Viewer | `.txt`, `.log`, `.csv` |
| **Images** | Image Viewer | `.jpg`, `.png`, `.gif`, `.bmp` |
| **Documents** | Document Viewer | `.pdf`, `.docx`, `.xlsx` |
| **Executables** | Binary Viewer | `.exe`, `.dll`, `.so`, `.dylib` |
| **Email** | Email Viewer | `.eml`, `.msg`, `.mbox` |
| **Plist** | Plist Viewer | `.plist` |
| **SQLite** | Database Viewer | `.db`, `.sqlite` |
| **Hex/Binary** | Hex Viewer | All binary files |

**Steps for each type:**
1. Navigate to a file of this type within any container
2. Click the file entry
3. Verify appropriate viewer loads
4. Test viewer-specific features (zoom for images, syntax highlighting for text, etc.)

---

### Phase 4: Nested Container Support

#### Test 4.1: Archive Within AD1
**Scenario:** AD1 container contains a ZIP file  
**Steps:**
1. Expand AD1 container
2. Navigate to a ZIP file within the AD1 tree
3. Click the ZIP file entry
4. Look for "Open as Container" button or inline expansion

**Expected Results:**
- ✅ ZIP file is recognized as nested container
- ✅ Can expand ZIP inline or open in new tree view
- ✅ ZIP contents are viewable
- ✅ Can navigate back to parent AD1

---

#### Test 4.2: Archive Within Archive
**Scenario:** ZIP file contains another ZIP or 7z  
**Steps:**
1. Expand outer archive
2. Navigate to nested archive file
3. Attempt to open/expand nested archive

**Expected Results:**
- ✅ Nested archive is recognized
- ✅ Can be extracted and opened
- ✅ Full tree navigation works
- ✅ Multiple nesting levels supported (3+ deep)

---

### Phase 5: Performance Testing

#### Test 5.1: Large AD1 Container
**Test File:** AD1 with > 50,000 entries  
**Metrics:**
- Initial load time: _________ seconds
- Tree expansion time: _________ seconds
- Scroll performance: _________ FPS
- Memory usage: _________ MB

**Expected:**
- Load < 5 seconds
- Smooth 60 FPS scrolling
- Memory < 500 MB

---

#### Test 5.2: Multiple Containers Open
**Scenario:** 5+ containers expanded simultaneously  
**Steps:**
1. Add 5 different containers to evidence
2. Expand all 5 containers
3. Navigate between containers
4. Monitor performance

**Expected Results:**
- ✅ All containers remain responsive
- ✅ No memory leaks (check dev tools)
- ✅ Tree state preserved when switching
- ✅ Scroll position remembered

---

#### Test 5.3: Archive with Many Small Files
**Test File:** ZIP/7z with 10,000+ small files  
**Metrics:**
- Metadata load: _________ ms (should be < 100ms)
- Full tree load: _________ seconds
- Directory expansion: _________ ms

---

### Phase 6: Error Handling

#### Test 6.1: Corrupted Container
**Steps:**
1. Add a corrupted/incomplete container file
2. Attempt to expand the container
3. Observe error message

**Expected Results:**
- ✅ Error message is displayed clearly
- ✅ "Retry" button available
- ✅ Other containers remain functional
- ✅ Application does not crash

---

#### Test 6.2: Unsupported Format
**Steps:**
1. Add a DMG file (currently unsupported for VFS)
2. Attempt to expand the container

**Expected Results:**
- ✅ Shows "Format 'dmg' not supported" message
- ✅ Provides helpful hint
- ✅ Does not attempt to mount/crash

---

#### Test 6.3: Missing Segments
**Scenario:** E01 with missing .e02 segment  
**Steps:**
1. Add E01 file with missing segments
2. Attempt to expand

**Expected Results:**
- ✅ Error message indicates missing segments
- ✅ Shows which segments are missing
- ✅ Retry option available

---

### Phase 7: Integration Tests

#### Test 7.1: Export from Tree
**Steps:**
1. Expand any container
2. Select multiple files in tree (Cmd+Click)
3. Click "Export" button
4. Verify files are exported correctly

**Expected Results:**
- ✅ Multiple files can be selected
- ✅ Export preserves directory structure
- ✅ Exported files match originals (hash verification)

---

#### Test 7.2: Hash Verification in Tree
**Steps:**
1. Expand container with known hashes
2. Right-click file in tree
3. Select "Verify Hash"

**Expected Results:**
- ✅ Hash is displayed
- ✅ Verification status shown (✓ or ✗)
- ✅ Hash history tracked

---

#### Test 7.3: Search Within Containers
**Steps:**
1. Expand multiple containers
2. Use search/filter functionality
3. Search for specific filename or extension

**Expected Results:**
- ✅ Search works across all expanded containers
- ✅ Results are highlighted in tree
- ✅ Click result navigates to file

---

## Manual Testing Checklist

### Pre-Test Setup
- [ ] Application built and running (`npm run tauri dev`)
- [ ] Test workspace loaded: `1827-1001 Case With Data`
- [ ] Evidence directory available: `1.Evidence/`
- [ ] At least one of each container type available for testing

### Discovery & Expansion
- [ ] AD1 containers discovered and expandable
- [ ] E01/EWF containers discovered and mountable
- [ ] L01 containers discovered and mountable
- [ ] ZIP/7z archives discovered and expandable
- [ ] UFED containers discovered and lazy-loadable
- [ ] RAW/DD images discovered and mountable
- [ ] ISO images discovered and mountable

### Navigation & Viewing
- [ ] Tree navigation smooth (expand/collapse)
- [ ] File selection works (click highlights)
- [ ] Content viewer loads for selected files
- [ ] Correct viewer selected for each file type
- [ ] Scroll performance is smooth (60 FPS)

### Nested Containers
- [ ] Archives within AD1 can be opened
- [ ] Archives within E01 can be opened
- [ ] Archives within archives work (nested)
- [ ] Navigation between nested containers works

### Performance
- [ ] Large containers (50K+ entries) load in < 5 seconds
- [ ] Archives load metadata instantly (< 100ms)
- [ ] UFED lazy loading works (paginated)
- [ ] Multiple containers open simultaneously without lag
- [ ] Memory usage stays under 500 MB with 5 containers open

### Error Handling
- [ ] Corrupted containers show clear error messages
- [ ] Unsupported formats show "not supported" message
- [ ] Missing segments detected and reported
- [ ] Application remains stable after errors
- [ ] Retry functionality works

### Integration
- [ ] Export from tree works
- [ ] Hash verification works for tree entries
- [ ] Search across containers works
- [ ] Type filtering works (filter by container type)
- [ ] Project save/load preserves tree state

---

## Known Issues / Limitations

### Currently Unsupported
- **DMG files:** Apple Disk Images require `hdiutil` or custom implementation
- **VMDK/VHD/VHDX:** Virtual disk formats (planned for future)
- **AFF/AFF4:** Advanced Forensic Format (planned for future)

### Partial Support
- **Memory dumps (.mem, .raw):** Detected but no specialized viewer
- **Large UFED (> 1M entries):** May experience slowdowns

---

## Test Results Template

| Test ID | Container Type | File Name | Status | Notes | Performance |
|---------|---------------|-----------|--------|-------|-------------|
| 2.1 | AD1 | __________ | ⬜ Pass ⬜ Fail | | _____ sec |
| 2.2 | E01 | __________ | ⬜ Pass ⬜ Fail | | _____ sec |
| 2.3 | ZIP/7z | __________ | ⬜ Pass ⬜ Fail | | _____ ms |
| 2.4 | UFED | __________ | ⬜ Pass ⬜ Fail | | _____ sec |
| 3.1 | Various | __________ | ⬜ Pass ⬜ Fail | | |
| 4.1 | AD1→ZIP | __________ | ⬜ Pass ⬜ Fail | | |
| 4.2 | ZIP→ZIP | __________ | ⬜ Pass ⬜ Fail | | |
| 5.1 | Large AD1 | __________ | ⬜ Pass ⬜ Fail | | _____ sec |
| 5.2 | Multiple | __________ | ⬜ Pass ⬜ Fail | | |
| 6.1 | Corrupted | __________ | ⬜ Pass ⬜ Fail | | |
| 6.2 | Unsupported | __________ | ⬜ Pass ⬜ Fail | | |

---

## Automated Testing (Future)

```typescript
// Example automated test for container expansion
describe("EvidenceTree - Container Expansion", () => {
  it("should expand AD1 container and load root children", async () => {
    const file = { path: "/path/to/test.ad1", container_type: "AD1" };
    await tree.toggleContainer(file);
    expect(tree.isContainerExpanded(file.path)).toBe(true);
    const children = tree.getAd1RootChildren(file.path);
    expect(children.length).toBeGreaterThan(0);
  });
  
  it("should load archive metadata instantly", async () => {
    const start = performance.now();
    await tree.archive.loadArchiveMetadata("/path/to/test.zip");
    const elapsed = performance.now() - start;
    expect(elapsed).toBeLessThan(100); // < 100ms
  });
});
```

---

## Regression Testing

When making changes to Evidence Tree or container parsers:

1. ✅ Run all Phase 2 tests (expansion for each container type)
2. ✅ Verify Phase 3 (file viewing still works)
3. ✅ Check Phase 5 (performance hasn't degraded)
4. ✅ Test with largest available container files

---

## Contact / Issues

If any test fails or unexpected behavior occurs:
1. Check browser console for errors
2. Check Tauri logs: `~/Library/Logs/ffx/`
3. Document exact steps to reproduce
4. Include container file details (type, size, format)
5. Report in GitHub Issues: `CORE-FFX/issues`

---

**Test Completed By:** __________________  
**Date:** __________________  
**Application Version:** v2.0.0  
**Platform:** macOS / Windows / Linux

