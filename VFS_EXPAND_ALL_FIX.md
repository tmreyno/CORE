# VFS "Expand All" Fix - February 8, 2026

## Issue Description

**Problem:** When clicking "Expand All" button in Evidence Tree toolbar, all mounted disk containers (E01, L01, Raw) show "Empty filesystem" message instead of displaying their contents.

**Symptoms:**
- Individual expansion works fine (after previous fix)
- "Expand All" causes all VFS containers to expand but show empty
- Partitions visible but no files/folders
- Manual collapse/expand after "Expand All" shows content correctly

## Root Cause

The `expandAllVfsDirs` function in `useVfsTree.ts` was **only setting expanded state** without **loading the actual children data**.

### Problem Code (Lines 189-210)

```typescript
const expandAllVfsDirs = async (containerPath: string): Promise<void> => {
  const mountInfo = vfsMountCache().get(containerPath);
  if (!mountInfo) return;
  
  // For VFS containers, expand the root children of each partition
  const keysToExpand: string[] = [];
  for (let i = 0; i < (mountInfo.partitions || []).length; i++) {
    const partition = mountInfo.partitions[i];
    const mountName = partition.mountName ?? `Partition${partition.number ?? i + 1}`;
    const rootKey = `${containerPath}::vfs::/${mountName}`;
    keysToExpand.push(rootKey);
  }
  
  // ❌ PROBLEM: Only setting expanded state, NOT loading children
  if (keysToExpand.length > 0) {
    setExpandedVfsPaths(prev => {
      const next = new Set(prev);
      keysToExpand.forEach(key => next.add(key));
      return next;
    });
  }
};
```

**What was missing:**
- No call to `loadVfsChildren()` to fetch data from backend
- No call to `invoke("vfs_list_dir", ...)` to populate cache
- Only UI state changed, not the actual data

**Comparison with AD1:**
AD1's `expandAllAd1Dirs` function properly loads children:
```typescript
await Promise.all(rootChildren.map(child => loadAd1Children(containerPath, child.address)));
```

## Solution

Modified `expandAllVfsDirs` to:
1. **Load children data first** using `loadVfsChildren()`
2. Load all partitions in **parallel** for speed
3. **Then** mark paths as expanded
4. Add comprehensive **debug logging**

### Fixed Code (Lines 189-234)

```typescript
const expandAllVfsDirs = async (containerPath: string): Promise<void> => {
  console.log(`[DEBUG] expandAllVfsDirs called for ${containerPath}`);
  const mountInfo = vfsMountCache().get(containerPath);
  if (!mountInfo) {
    console.log(`[DEBUG] expandAllVfsDirs - no mount info found for ${containerPath}`);
    return;
  }
  
  // For VFS containers, expand the root children of each partition
  const partitionsToLoad: { mountName: string; rootPath: string }[] = [];
  for (let i = 0; i < (mountInfo.partitions || []).length; i++) {
    const partition = mountInfo.partitions[i];
    const mountName = partition.mountName ?? `Partition${partition.number ?? i + 1}`;
    const rootPath = `/${mountName}`;
    partitionsToLoad.push({ mountName, rootPath });
  }
  
  console.log(`[DEBUG] expandAllVfsDirs - found ${partitionsToLoad.length} partitions to load`);
  
  // ✅ FIX: Load children for each partition in parallel
  if (partitionsToLoad.length > 0) {
    await Promise.all(
      partitionsToLoad.map(async ({ rootPath }) => {
        try {
          console.log(`[DEBUG] expandAllVfsDirs - loading children for ${rootPath}`);
          await loadVfsChildren(containerPath, rootPath);
          console.log(`[DEBUG] expandAllVfsDirs - loaded children for ${rootPath}`);
        } catch (error) {
          console.error(`[ERROR] expandAllVfsDirs - failed to load ${rootPath}:`, error);
        }
      })
    );
    
    // After loading, mark as expanded
    setExpandedVfsPaths(prev => {
      const next = new Set(prev);
      partitionsToLoad.forEach(({ rootPath }) => {
        const key = `${containerPath}::vfs::${rootPath}`;
        next.add(key);
      });
      return next;
    });
    
    console.log(`[DEBUG] expandAllVfsDirs - completed for ${containerPath}`);
  }
};
```

## Key Changes

1. **Parallel Loading:**
   ```typescript
   await Promise.all(
     partitionsToLoad.map(async ({ rootPath }) => {
       await loadVfsChildren(containerPath, rootPath);
     })
   );
   ```

2. **Error Handling:**
   - Wrapped in try/catch per partition
   - Logs errors but continues with other partitions
   - One failed partition doesn't block others

3. **Debug Logging:**
   - Logs function entry with container path
   - Logs partition count found
   - Logs each partition load start/completion
   - Logs errors with full context
   - Logs final completion

4. **Proper Ordering:**
   - **First:** Load all children data (parallel)
   - **Then:** Set expanded state
   - **Previously:** Only set expanded state (bug)

## Testing Instructions

### 1. Test Expand All Button

1. Open CORE-FFX
2. Load project: `1827-1001.ffxproj`
3. Find Evidence Tree toolbar (top of tree panel)
4. Click **"Expand All"** button
5. **Expected:**
   - All containers expand
   - E01 containers show partitions
   - Each partition shows files/folders
   - **NO "Empty filesystem" messages**
   - Loading indicators appear briefly during load
6. **Check console** - should see:
   ```
   [DEBUG] expandAllVfsDirs called for .../macwd.E01
   [DEBUG] expandAllVfsDirs - found 2 partitions to load
   [DEBUG] expandAllVfsDirs - loading children for /Partition1_NTFS
   [DEBUG] expandAllVfsDirs - loaded children for /Partition1_NTFS
   [DEBUG] expandAllVfsDirs - completed for .../macwd.E01
   ```

### 2. Test Multiple Containers

1. Ensure project has multiple E01 files
2. Click "Expand All"
3. Verify ALL E01 containers show content
4. Check console - should see parallel loading:
   ```
   [DEBUG] expandAllVfsDirs called for .../macwd.E01
   [DEBUG] expandAllVfsDirs called for .../PC-MUS-001.E01
   [DEBUG] expandAllVfsDirs - loading children for /Partition1_NTFS
   [DEBUG] expandAllVfsDirs - loading children for /OS_NTFS
   ```

### 3. Test Individual vs Expand All

1. Collapse all containers (click "Collapse All")
2. Manually expand `macwd.E01` - should work
3. Collapse it
4. Click "Expand All" - should also work
5. Both methods should show identical content

### 4. Test Performance

With 3 E01 files (macwd.E01, 2020JimmyWilson.E01, PC-MUS-001.E01):
- **Expected load time:** 2-10 seconds depending on size
- **Parallel loading:** All mount simultaneously
- **Progress:** Loading indicators on each container

## Files Modified

**src/components/EvidenceTree/hooks/useVfsTree.ts**
- Lines 188-234: Rewrote `expandAllVfsDirs` to load children before expanding
- Added parallel loading with `Promise.all()`
- Added comprehensive error handling and debug logging

## Expected Behavior

| Action | Old Behavior | New Behavior |
|--------|-------------|--------------|
| Click single container | ❌ Empty filesystem (fixed in previous commit) | ✅ Loads and displays content |
| Click "Expand All" | ❌ All VFS show empty filesystem | ✅ All VFS load and display content |
| Collapse/Re-expand | ✅ Shows content (from cache) | ✅ Shows content (from cache) |
| Multiple E01 files | ❌ All empty on expand all | ✅ All load in parallel |

## Related Fixes

This fix complements the earlier loading state fix:

1. **Previous fix (VFS_TREE_FIX.md):** Fixed individual expansion loading state management
2. **This fix:** Fixed "Expand All" to actually load data before expanding

Both fixes address the same symptom ("Empty filesystem") but different code paths:
- Individual expansion: `toggleVfsDir` → `loadVfsChildren`
- Expand all: `expandAllVfsDirs` → **now calls** `loadVfsChildren`

## Verification Checklist

- [ ] "Expand All" loads all E01 partitions
- [ ] No "Empty filesystem" on any VFS container
- [ ] Loading indicators appear during load
- [ ] Console shows debug logs for all partitions
- [ ] Parallel loading works (multiple containers at once)
- [ ] Error handling works (one failed partition doesn't block others)
- [ ] Works with large containers (PC-MUS-001.E01 49GB)
- [ ] Works with small containers (macwd.E01 295MB)

## Console Log Examples

**Success Case:**
```
[DEBUG] expandAllVfsDirs called for /path/to/macwd.E01
[DEBUG] expandAllVfsDirs - found 2 partitions to load
[DEBUG] expandAllVfsDirs - loading children for /Partition1_NTFS
[DEBUG] expandAllVfsDirs - loading children for /Partition2_FAT32
[DEBUG] loadVfsChildren called: containerPath=/path/to/macwd.E01, vfsPath=/Partition1_NTFS
[DEBUG] loadVfsChildren - invoking vfs_list_dir...
[DEBUG] loadVfsChildren - backend returned 15 entries for /Partition1_NTFS
[DEBUG] expandAllVfsDirs - loaded children for /Partition1_NTFS
[DEBUG] expandAllVfsDirs - loaded children for /Partition2_FAT32
[DEBUG] expandAllVfsDirs - completed for /path/to/macwd.E01
```

**Error Case (one partition fails):**
```
[DEBUG] expandAllVfsDirs called for /path/to/macwd.E01
[DEBUG] expandAllVfsDirs - found 2 partitions to load
[DEBUG] expandAllVfsDirs - loading children for /Partition1_NTFS
[DEBUG] expandAllVfsDirs - loading children for /Partition2_CORRUPTED
[DEBUG] loadVfsChildren - backend returned 15 entries for /Partition1_NTFS
[ERROR] expandAllVfsDirs - failed to load /Partition2_CORRUPTED: Failed to mount partition
[DEBUG] expandAllVfsDirs - loaded children for /Partition1_NTFS
[DEBUG] expandAllVfsDirs - completed for /path/to/macwd.E01
```

## Known Limitations

- **First mount may be slow** for large E01 files
- **Parallel loading** may spike CPU/memory temporarily
- **DMG files** are unsupported and will log errors (expected)

---

**Date:** February 8, 2026  
**Issue:** VFS "Expand All" showing empty filesystem  
**Status:** FIXED ✅  
**Test Status:** Awaiting manual verification  
**Related:** VFS_TREE_FIX.md (individual expansion fix)
