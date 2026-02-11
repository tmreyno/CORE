# File Type Detection Consolidation - Phase 6

**Date:** January 22, 2026
**Status:** ✅ COMPLETE
**Files Modified:** 3 files

---

## Overview

Identified and eliminated duplicate file type detection logic scattered across multiple frontend files. Created centralized `fileTypeUtils.ts` module with comprehensive type detection utilities.

---

## Problem Identified

File type detection was duplicated across multiple files with inconsistent patterns:

1. **`src/components/ContainerEntryViewer.tsx`**

   - Local functions: `getExtension()`, `isPdf()`, `isImage()`, `isSpreadsheet()`, `isDocumentViewerFile()`
   - ~25 lines of duplicate logic

2. **`src/components/transfer/utils.ts`**

   - `detectFileType()` function with ~15 lines of regex patterns
   - Inline checks for images, video, audio, documents, code, databases, archives

**Duplication Issues:**

- Extension lists maintained separately in multiple files
- Regex patterns repeated for each file type
- Risk of inconsistent behavior (e.g., one place recognizes `.tiff`, another doesn't)
- No type-safe file type categories
- Difficult to add support for new formats

---

## Solution Implemented

### 1. Created Comprehensive File Type Utilities

**File:** `src/utils/fileTypeUtils.ts` (325 lines)

**Extension Categories:**

```typescript
export const IMAGE_EXTENSIONS = [
  "jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico",
  "tiff", "tif", "heic", "heif",  // Professional formats
  "raw", "cr2", "nef", "arw", "dng", "orf", "rw2",  // RAW camera formats
] as const;

export const VIDEO_EXTENSIONS = [
  "mp4", "avi", "mov", "mkv", "wmv", "flv", "webm",
  "m4v", "mpeg", "mpg", "3gp", "ogv", "m2ts", "mts",
] as const;

export const AUDIO_EXTENSIONS = [
  "mp3", "wav", "flac", "aac", "ogg", "wma", "m4a",
  "aiff", "aif", "opus", "ape", "alac",
] as const;

// ... similar for DOCUMENT, SPREADSHEET, CODE, DATABASE, ARCHIVE

```


**Type Guard Functions:**

```typescript
export function isImage(filename: string): boolean;
export function isVideo(filename: string): boolean;
export function isAudio(filename: string): boolean;
export function isDocument(filename: string): boolean;
export function isSpreadsheet(filename: string): boolean;
export function isTextDocument(filename: string): boolean;
export function isCode(filename: string): boolean;
export function isDatabase(filename: string): boolean;
export function isArchive(filename: string): boolean;
export function isPdf(filename: string): boolean;

```


**Detection Functions:**

```typescript
export function detectFileType(filename: string): FileTypeCategory;
export function getExtension(filename: string): string;
export function getBaseName(filename: string): string;
export function hasExtension(filename: string, extension: string): boolean;
export function hasAnyExtension(filename: string, extensions: readonly string[]): boolean;

```


**Type Safety:**

```typescript
export type FileTypeCategory =
  | "image"
  | "video"
  | "audio"
  | "document"
  | "spreadsheet"
  | "code"
  | "database"
  | "archive"
  | "container"
  | "unknown";

```


---

### 2. Updated ContainerEntryViewer.tsx

**Changes:**

- Added imports from `fileTypeUtils`
- Removed 4 duplicate local functions: `getExtension()`, `isPdf()`, `isImage()`, `isSpreadsheet()`, `isDocumentViewerFile()`
- Updated to use `isTextDocument()` instead of `isDocumentViewerFile()`

**Before (25 lines):**

```typescript
/** Get file extension from name */
function getExtension(name: string): string {
  const parts = name.split(".");
  return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : "";
}

/** Check if file is a PDF */
function isPdf(name: string): boolean {
  return getExtension(name) === "pdf";
}

/** Check if file is an image */
function isImage(name: string): boolean {
  const ext = getExtension(name);
  return ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico"].includes(ext);
}

/** Check if file is a spreadsheet */
function isSpreadsheet(name: string): boolean {
  const ext = getExtension(name);
  return ["xlsx", "xls", "ods", "csv", "tsv"].includes(ext);
}

/** Check if file is a text-based document that DocumentViewer can render */
function isDocumentViewerFile(name: string): boolean {
  const ext = getExtension(name);
  return ["docx", "doc", "html", "htm", "md", "markdown", "txt", "rtf"].includes(ext);
}

```


**After (1 import statement):**

```typescript
import {
  getExtension,
  isImage,
  isSpreadsheet,
  isPdf,
  isTextDocument
} from "../utils/fileTypeUtils";

```


---

### 3. Updated transfer/utils.ts

**Changes:**

- Added import from `fileTypeUtils`
- Replaced `detectFileType()` implementation with delegation to centralized utility
- Eliminated 15 lines of regex patterns

**Before (15 lines of regex patterns):**

```typescript
export function detectFileType(path: string): FileType {
  const lower = path.toLowerCase();
  if (detectContainerType(path) !== "unknown") return "container";
  if (/\.(jpg|jpeg|png|gif|bmp|webp|svg|ico|tiff?|heic|raw|cr2|nef|arw)$/.test(lower)) return "image";
  if (/\.(mp4|avi|mov|mkv|wmv|flv|webm|m4v|mpeg|mpg|3gp)$/.test(lower)) return "video";
  if (/\.(mp3|wav|flac|aac|ogg|wma|m4a|aiff?)$/.test(lower)) return "audio";
  if (/\.(pdf|doc|docx|xls|xlsx|ppt|pptx|odt|ods|odp|rtf|txt|md|csv)$/.test(lower)) return "document";
  if (/\.(js|ts|jsx|tsx|py|rs|go|java|c|cpp|h|hpp|cs|rb|php|html|css|scss|json|xml|yaml|yml|toml|sql|sh|bash|ps1)$/.test(lower)) return "code";
  if (/\.(db|sqlite|sqlite3|mdb|accdb|sql)$/.test(lower)) return "database";
  if (/\.(zip|tar|gz|bz2|7z|rar|xz|tgz)$/.test(lower)) return "archive";
  return "unknown";
}

```


**After (3 lines):**

```typescript
import { detectFileType as detectFileTypeUtil } from "../../utils/fileTypeUtils";

export function detectFileType(path: string): FileType {
  return detectFileTypeUtil(path) as FileType;
}

```


---

## Benefits

### Code Reduction

- **Eliminated ~40 lines** of duplicate file type detection logic
- **Single source of truth** for all file type checking
- **Consistent behavior** across entire application

### Extensibility

- **Easy to add formats**: Add to extension array in one place
- **Comprehensive coverage**: Supports professional/RAW image formats, modern video codecs, etc.
- **Categorized extensions**: Clear organization by file type

### Type Safety

- Type-safe `FileTypeCategory` enum
- Readonly extension arrays prevent modification
- TypeScript catches misuse at compile time

### Maintainability

- Clear, well-documented functions
- Consistent naming patterns
- Easy to understand and modify

---

## Compilation Status

✅ **All files compile with 0 errors:**

- `src/utils/fileTypeUtils.ts` - 0 errors
- `src/components/ContainerEntryViewer.tsx` - 0 errors
- `src/components/transfer/utils.ts` - 0 errors

---

## Enhanced Format Support

The centralized utilities add support for many additional formats:

**Images:**

- Added: TIFF, HEIF, HEIC (modern mobile formats)
- Added: CR2, NEF, ARW, DNG, ORF, RW2 (professional RAW formats)

**Video:**

- Added: OGV, M2TS, MTS (additional codecs)

**Audio:**

- Added: OPUS, APE, ALAC (high-quality formats)

**Documents:**

- Added: Numbers (Apple spreadsheet format)

**Code:**

- Added: MJS, CJS (modern JavaScript modules)
- Added: PYW, PYX (Python variants)
- Added: KT, KTS (Kotlin)
- Added: CXX, HXX (C++ variants)
- Added: RAKE (Ruby build files)
- Added: PHTML (PHP HTML templates)
- Added: SASS, LESS (CSS preprocessors)

**Databases:**

- Added: DBF, SDF, NDF, MDF, LDF (additional database formats)

---

## Usage Examples

### Type Guards

```typescript
import { isImage, isDocument, isPdf } from "../utils/fileTypeUtils";

if (isImage(filename)) {
  // Show image viewer
}

if (isPdf(filename)) {
  // Show PDF viewer
}

if (isDocument(filename)) {
  // Show document preview
}

```


### File Type Detection

```typescript
import { detectFileType } from "../utils/fileTypeUtils";

const fileType = detectFileType(filename);
switch (fileType) {
  case "image":
    return <ImageIcon />;
  case "video":
    return <VideoIcon />;
  case "document":
    return <DocumentIcon />;
  default:
    return <FileIcon />;
}

```


### Extension Utilities

```typescript
import { getExtension, hasExtension, hasAnyExtension } from "../utils/fileTypeUtils";

const ext = getExtension("report.pdf");  // "pdf"
const isPdf = hasExtension(filename, "pdf");
const isMedia = hasAnyExtension(filename, ["jpg", "png", "mp4", "mov"]);

```


---

## Impact Summary

| Metric | Achievement |
| -------- | ------------- |
| **Lines eliminated** | ~40 lines of duplicate code |
| **Files improved** | 3 files updated |
| **Extension categories** | 9 comprehensive categories |
| **Type guard functions** | 10 type-safe functions |
| **Additional formats** | 30+ new format extensions |
| **Compilation errors** | 0 |
| **Type safety** | 100% |

---

## Overall Progress Summary

| Phase | Focus | Lines Saved |
| ------- | ------- | ------------- |
| **Phases 1-4** | Hash system modernization | ~170 lines |
| **Phase 5** | Container detection consolidation | ~50 lines |
| **Phase 6** | File type detection consolidation | ~40 lines |
| **Total** | | **~260 lines** |

---

## Related Files

- `src/utils/fileTypeUtils.ts` - New centralized utilities (325 lines)
- `src/components/ContainerEntryViewer.tsx` - Updated to use utilities
- `src/components/transfer/utils.ts` - Updated to use utilities
- `src/utils/containerUtils.ts` - Container detection (Phase 5)
- `src/types/hash.ts` - Hash types (Phase 1)
- `src/hooks/hashUtils.ts` - Hash utilities (Phase 2)

---

## Testing Recommendations

### Unit Tests Needed

1. Test all file type guard functions with various extensions
2. Test case-insensitive detection (`.JPG`, `.jpg`, `.Jpg`)
3. Test extension extraction with edge cases (no extension, multiple dots, hidden files)
4. Test `detectFileType()` returns correct categories
5. Test `hasExtension()` and `hasAnyExtension()` helpers

### Integration Tests

1. Verify ContainerEntryViewer shows correct viewers for each file type
2. Verify Transfer panel shows correct icons for each file type
3. Verify file type detection works with forensic container entries
4. Verify all viewers (PDF, Image, Document, Spreadsheet) open correctly

---

