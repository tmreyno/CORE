# Archive Tools Quick Reference

## 🚀 Quick Access

**From Export Panel:** Click "Archive Tools" button in header

---

## 📖 API Reference

### Test Archive
```typescript
import { testArchive } from "@/api/archiveCreate";

const isValid = await testArchive(
  "/path/to/archive.7z",
  "password123" // optional
);
```

### Repair Archive
```typescript
import { repairArchive, listenToRepairProgress } from "@/api/archiveCreate";

const unlisten = await listenToRepairProgress((progress) => {
  console.log(`${progress.percent}% - ${progress.status}`);
});

try {
  const repairedPath = await repairArchive(
    "/path/to/corrupted.7z",
    "/path/to/repaired.7z"
  );
  console.log("Repaired:", repairedPath);
} finally {
  unlisten();
}
```

### Validate Archive
```typescript
import { validateArchive } from "@/api/archiveCreate";

const result = await validateArchive("/path/to/archive.7z");
if (!result.isValid) {
  console.error(result.errorMessage);
  console.log("Suggestion:", result.suggestion);
}
```

### Extract Split Archive
```typescript
import { extractSplitArchive, listenToSplitExtractProgress } from "@/api/archiveCreate";

const unlisten = await listenToSplitExtractProgress((progress) => {
  console.log(`${progress.percent}% - ${progress.status}`);
});

try {
  const outputDir = await extractSplitArchive(
    "/path/to/archive.7z.001",
    "/path/to/output/",
    "password" // optional
  );
  console.log("Extracted to:", outputDir);
} finally {
  unlisten();
}
```

### Get Error Details
```typescript
import { getLastArchiveError, clearLastArchiveError } from "@/api/archiveCreate";

const error = await getLastArchiveError();
if (error) {
  console.log(`Error ${error.code}: ${error.message}`);
  console.log(`Context: ${error.fileContext}`);
  console.log(`Suggestion: ${error.suggestion}`);
  await clearLastArchiveError();
}
```

### Encrypt/Decrypt Data
```typescript
import { encryptDataNative, decryptDataNative } from "@/api/archiveCreate";

// Encrypt
const plaintext = new TextEncoder().encode("sensitive data");
const encrypted = await encryptDataNative(plaintext, "strong-password");

// Decrypt
const decrypted = await decryptDataNative(encrypted, "strong-password");
const text = new TextDecoder().decode(decrypted);
```

---

## 🎨 UI Component

### Open Archive Tools Modal
```tsx
import { ArchiveToolsPanel } from "@/components";

function MyComponent() {
  const [showTools, setShowTools] = createSignal(false);
  
  return (
    <>
      <button onClick={() => setShowTools(true)}>
        Open Archive Tools
      </button>
      
      <Show when={showTools()}>
        <ArchiveToolsPanel onClose={() => setShowTools(false)} />
      </Show>
    </>
  );
}
```

---

## 🔥 Common Patterns

### Test → Repair → Test Workflow
```typescript
async function recoverArchive(path: string) {
  // 1. Test original
  const valid = await testArchive(path);
  if (valid) return path; // Already good
  
  // 2. Attempt repair
  const repairedPath = path.replace(".7z", "_repaired.7z");
  await repairArchive(path, repairedPath);
  
  // 3. Test repaired
  const repairedValid = await testArchive(repairedPath);
  if (!repairedValid) {
    throw new Error("Repair failed - archive too damaged");
  }
  
  return repairedPath;
}
```

### Validate with Error Details
```typescript
async function validateWithDetails(path: string) {
  const result = await validateArchive(path);
  
  if (!result.isValid) {
    const error = await getLastArchiveError();
    return {
      ...result,
      details: error,
    };
  }
  
  return result;
}
```

### Progress with Toast
```typescript
import { useToast } from "@/components/Toast";

async function repairWithProgress(corruptedPath: string, repairedPath: string) {
  const toast = useToast();
  
  const unlisten = await listenToRepairProgress((progress) => {
    if (progress.percent === 100) {
      toast.success("Repair Complete", repairedPath);
    }
  });
  
  try {
    return await repairArchive(corruptedPath, repairedPath);
  } catch (error) {
    toast.error("Repair Failed", error.message);
    throw error;
  } finally {
    unlisten();
  }
}
```

---

## 🎯 Backend Commands

All commands registered in `src-tauri/src/lib.rs`:

```rust
commands::test_7z_archive,
commands::repair_7z_archive,
commands::validate_7z_archive,
commands::extract_split_7z_archive,
commands::encrypt_data_native,
commands::decrypt_data_native,
commands::get_last_archive_error,
commands::clear_last_archive_error,
```

---

## 📊 Progress Event Types

### Archive Test Progress
```typescript
interface ArchiveTestProgress {
  archivePath: string;
  status: string;
  percent: number;
}
// Event: "archive-test-progress"
```

### Archive Repair Progress
```typescript
interface ArchiveRepairProgress {
  percent: number;
  status: string;
}
// Event: "archive-repair-progress"
```

### Split Extract Progress
```typescript
interface SplitExtractProgress {
  status: string;
  percent: number;
}
// Event: "split-extract-progress"
```

---

## ⚡ Performance Tips

1. **Testing:** Very fast (~100ms-10s), use liberally
2. **Repair:** Slow (~1-5 min), show progress bar
3. **Validate:** Medium (~2x test time), good for debugging
4. **Extract:** Fast with streaming, constant memory

---

## 🔒 Security Best Practices

✅ Clear passwords from memory after use  
✅ Use `getLastArchiveError()` for debugging only (may contain sensitive paths)  
✅ Always validate user input paths before passing to backend  
✅ Don't log passwords or encrypted data  
✅ Clear error state with `clearLastArchiveError()` after handling  

---

## 🐛 Error Handling

```typescript
try {
  await testArchive(path);
} catch (error) {
  // Network/IPC error
  console.error("Communication error:", error);
}

// Archive-specific errors
const result = await validateArchive(path);
if (!result.isValid) {
  // Archive validation failed
  const details = await getLastArchiveError();
  console.error("Archive error:", details);
}
```

---

## 📚 Documentation Links

- **User Guide:** `docs/ARCHIVE_TOOLS_USER_GUIDE.md`
- **API Details:** `src/api/archiveCreate.ts`
- **Component Code:** `src/components/ArchiveToolsPanel.tsx`
- **Backend Commands:** `src-tauri/src/commands/archive.rs`
- **Integration Summary:** `docs/SEVENZIP_INTEGRATION_COMPLETE.md`

---

## 🎯 Use Cases by Feature

| Feature | Use Case |
|---------|----------|
| **Test** | Quick integrity check before extraction |
| **Repair** | Recover data from damaged evidence |
| **Validate** | Deep analysis with error context |
| **Extract Split** | Process multi-volume disk images |
| **Encrypt** | Secure sensitive analysis results |
| **Error Details** | Troubleshooting failed operations |

---

## ✨ Integration Points

### From File Manager
```typescript
// Right-click menu on .7z file
contextMenu.addItem({
  label: "Test Archive",
  action: async () => {
    const valid = await testArchive(filePath);
    toast.info(valid ? "Valid" : "Invalid");
  }
});
```

### From Activity Panel
```typescript
// After archive creation completes
onArchiveComplete: async (archivePath) => {
  const valid = await testArchive(archivePath);
  if (!valid) {
    toast.warning("Archive may be corrupted", "Consider re-creating");
  }
}
```

### From Report Generator
```typescript
// Include archive validation in report
const validation = await validateArchive(evidencePath);
report.addSection({
  title: "Archive Integrity",
  status: validation.isValid ? "PASS" : "FAIL",
  details: validation.errorMessage,
});
```

---

**Last Updated:** February 6, 2026  
**Version:** CORE-FFX v0.1.0
