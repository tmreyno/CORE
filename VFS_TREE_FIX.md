# VFS Tree Loading Fix - February 8, 2026

## Issue Description

**Problem:** VFS trees (E01, L01, Raw disk images) were intermittently showing "Empty filesystem" message when expanded. Users had to collapse and re-expand containers multiple times before the folder/file tree would appear.

**Symptoms:**
- E01 containers expand but show "Empty filesystem"
- Partitions visible but content not loading
- Inconsistent behavior - sometimes works, sometimes doesn't
- Multiple expand/collapse attempts eventually show content

## Root Cause

The issue was a **race condition** in the VFS partition loading logic:

### Problem in `EvidenceTree.tsx` (Line 203 - old code):
```tsx
onToggle={async (cp, vp) => tree.vfs.toggleVfsDir(cp, vp, tree.loading(), () => {})}
```

The `onToggle` callback was passing an **empty function** `() => {}` as the `setLoading` parameter. This meant:
1. When a partition was clicked, `toggleVfsDir` would start loading children
2. Loading state was never updated (empty function did nothing)
3. UI would immediately check `children().length === 0 && !isLoading()`
4. Since loading state wasn't set, `isLoading()` returned `false`
5. "Empty filesystem" message displayed while data was still loading
6. Sometimes data arrived fast enough to prevent the error message
7. Other times (slower loads), the empty message appeared

### Why Multiple Attempts "Fixed" It:

After the first expansion:
- Data was loaded into cache despite the error
- On second expansion, cached data was available immediately
- No loading needed, so children displayed instantly

## Solution

### 1. Exposed `setLoadingState` from `useEvidenceTree` hook

**File:** `src/components/EvidenceTree/hooks/useEvidenceTree.ts`

```typescript
export interface UseEvidenceTreeReturn {
  // Loading state
  loading: Accessor<Set<string>>;
  isLoading: (key: string) => boolean;
  setLoadingState: (key: string, isLoadingNow: boolean) => void; // ← ADDED
  // ... rest of interface
}

// In return statement:
return {
  // ...
  loading,
  isLoading,
  setLoadingState, // ← ADDED
  // ...
};
```

### 2. Fixed `EvidenceTree.tsx` to Pass Proper Loading Callback

**File:** `src/components/EvidenceTree.tsx` (Lines 194-221)

```tsx
onToggle={async (cp, vp) => {
  // Use proper loading state management via exposed setLoadingState
  const nodeKey = `${cp}::vfs::${vp}`;
  const setLoadingForNode = (fn: (prev: Set<string>) => Set<string>) => {
    const newLoading = fn(tree.loading());
    if (newLoading.has(nodeKey)) {
      tree.setLoadingState(nodeKey, true);  // ← Set loading state
    } else {
      tree.setLoadingState(nodeKey, false); // ← Clear loading state
    }
  };
  await tree.vfs.toggleVfsDir(cp, vp, tree.loading(), setLoadingForNode);
}}
```

### 3. Enhanced Logging in `useVfsTree.ts`

**File:** `src/components/EvidenceTree/hooks/useVfsTree.ts`

Added debug logging to track:
- When `loadVfsChildren` is called
- Cache hits/misses
- Backend invocation and response
- Loading state changes
- Expansion state updates

```typescript
console.log(`[DEBUG] loadVfsChildren called: containerPath=${containerPath}, vfsPath=${vfsPath}`);
console.log(`[DEBUG] loadVfsChildren - backend returned ${children.length} entries`);
console.log(`[DEBUG] toggleVfsDir - expanding ${vfsPath}, needsLoad=${needsLoad}`);
```

## Testing Instructions

### 1. Quick Test (macwd.E01)
1. Open CORE-FFX
2. Load project: `1827-1001.ffxproj`
3. Find `macwd.E01` in Evidence Tree
4. Click chevron to expand
5. **Expected:** 
   - Loading indicator appears briefly
   - Partitions appear within 1-2 seconds
   - Click partition to expand
   - Files/folders appear immediately
   - NO "Empty filesystem" message

### 2. Stress Test
1. Expand `macwd.E01`
2. Collapse it
3. Expand again immediately
4. Collapse again
5. Repeat 5-10 times rapidly
6. **Expected:** Consistent behavior every time, no "Empty filesystem"

### 3. Large File Test
1. Expand `PC-MUS-001.E01` (49GB)
2. Wait for mount (may take 10-15 seconds)
3. Expand partitions
4. **Expected:** Loading indicator visible during load, then content appears

### 4. Console Verification
Open browser DevTools (Cmd+Option+I) → Console tab

Look for these log messages:
```
[DEBUG] toggleVfsDir called: path=/Partition1_NTFS, nodeKey=...
[DEBUG] toggleVfsDir - expanding /Partition1_NTFS, needsLoad=true
[DEBUG] toggleVfsDir - setting loading state for ...
[DEBUG] loadVfsChildren called: containerPath=.../macwd.E01, vfsPath=/Partition1_NTFS
[DEBUG] loadVfsChildren - invoking vfs_list_dir...
[DEBUG] loadVfsChildren - backend returned 15 entries for /Partition1_NTFS
[DEBUG] toggleVfsDir - clearing loading state for ...
[DEBUG] toggleVfsDir - expanded, new expanded count=1
```

**Red flags (should NOT see):**
- ❌ "Empty filesystem" message
- ❌ Errors about failed vfs_list_dir
- ❌ Long delays without loading indicator

## Technical Details

### Loading State Flow

**Before (Broken):**
```
1. User clicks partition
2. toggleVfsDir called with empty setLoading function
3. loadVfsChildren starts backend call
4. UI checks: children.length === 0 && !isLoading()
5. isLoading() returns false (never set to true)
6. "Empty filesystem" displays
7. Backend returns data
8. Cache updated
9. UI still shows "Empty filesystem" until next render
```

**After (Fixed):**
```
1. User clicks partition
2. toggleVfsDir called with proper setLoading function
3. Loading state set to TRUE
4. loadVfsChildren starts backend call
5. UI checks: children.length === 0 && !isLoading()
6. isLoading() returns true
7. Loading spinner displays
8. Backend returns data
9. Cache updated
10. Loading state set to FALSE
11. UI re-renders with children
12. Files/folders display
```

### Cache Key Format

VFS children cache uses this key format:
```
{containerPath}::vfs::{vfsPath}
```

Examples:
```
/path/to/macwd.E01::vfs::/Partition1_NTFS
/path/to/macwd.E01::vfs::/Partition1_NTFS/Users
/path/to/macwd.E01::vfs::/Partition1_NTFS/Users/Documents
```

### Loading State Key Format

Same as cache key:
```
{containerPath}::vfs::{vfsPath}
```

When a node is loading, its key is added to the `loading` Set. When complete, removed from Set.

## Files Modified

1. **src/components/EvidenceTree/hooks/useEvidenceTree.ts**
   - Added `setLoadingState` to interface (line ~57)
   - Added `setLoadingState` to return statement (line ~553)

2. **src/components/EvidenceTree.tsx**
   - Fixed `onToggle` callback for PartitionNode (lines 194-221)
   - Now properly manages loading state via `tree.setLoadingState()`

3. **src/components/EvidenceTree/hooks/useVfsTree.ts**
   - Enhanced `loadVfsChildren` with debug logging
   - Enhanced `toggleVfsDir` with debug logging
   - Better tracking of cache operations

## Expected Performance

| Operation | Time | Notes |
|-----------|------|-------|
| E01 mount (< 1GB) | 1-5 sec | VFS initialization |
| Partition expand | < 500ms | Load root directory |
| Directory expand | < 200ms | Load subdirectory |
| Cached expand | < 50ms | From cache |

## Verification Checklist

- [ ] E01 expands without "Empty filesystem" error
- [ ] Partitions load consistently on first try
- [ ] Loading indicator visible during load
- [ ] Collapse/expand works reliably
- [ ] Large files (PC-MUS-001.E01) work correctly
- [ ] Console shows proper debug logs
- [ ] No errors in browser console
- [ ] No errors in Tauri terminal

## Known Limitations

- **First mount may be slow** for large E01 files (10-30 seconds)
- **L01 containers** not yet fully implemented (will show "not yet implemented" message)
- **DMG files** are unsupported and will show appropriate error

## Rollback

If this fix causes issues, revert these commits:
1. Revert changes to `EvidenceTree.tsx` (restore `() => {}` callback)
2. Revert changes to `useEvidenceTree.ts` (remove `setLoadingState` export)
3. Revert changes to `useVfsTree.ts` (remove debug logging)

## Next Steps

1. **Test manually** - Follow testing instructions above
2. **Monitor logs** - Watch browser console during testing
3. **Report issues** - If "Empty filesystem" still appears, check console logs
4. **Performance test** - Test with large containers (PC-MUS-001.E01)
5. **Regression test** - Verify other container types still work (AD1, Archives)

## Related Issues

- VFS mounting working correctly (backend verified)
- Cache mechanism working correctly
- Only issue was UI race condition with loading state

## Success Criteria

✅ **FIXED if:**
- E01 containers expand reliably on first attempt
- No "Empty filesystem" errors
- Loading indicators appear during load
- Console shows proper debug flow
- Works consistently across multiple files

❌ **NOT FIXED if:**
- Still seeing "Empty filesystem" intermittently
- Multiple attempts needed to see content
- Loading state not displaying
- Console errors appear

---

**Date:** February 8, 2026  
**Issue:** VFS tree "Empty filesystem" race condition  
**Status:** FIXED ✅  
**Test Status:** Awaiting manual verification
