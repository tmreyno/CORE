# Updating CORE-FFX to Use Latest sevenzip-ffi Features

## Overview

CORE-FFX currently uses `sevenzip-ffi` for archive creation but doesn't leverage all the latest features. This guide shows how to update to use:

- ✅ **Already Implemented:**
  - Basic 7z archive creation with streaming
  - Split/multi-volume archives  
  - AES-256 encryption
  - Progress callbacks
  - Multi-threading

- 🆕 **New Features to Add:**
  - **Archive testing/verification** - Test archive integrity
  - **Archive repair** - Recover corrupted archives
  - **Raw LZMA compression** - Create .lzma/.xz files
  - **Enhanced error reporting** - Detailed error context
  - **Native Rust encryption** - Pure Rust AES-256 (faster)
  - **Split archive extraction** - Extract multi-volume archives

---

## Current Implementation Status

### ✅ Currently Used Features

**File:** `src-tauri/src/commands/archive_create.rs`

```rust
use seven_zip::{SevenZip, CompressionLevel, CompressOptions, StreamOptions};

// ✅ Standard compression
sz.create_archive("archive.7z", &input_paths, compression_level, password)?;

// ✅ Streaming compression (for large files)
sz.create_archive_streaming(&archive_path, &input_paths, level, &stream_opts, progress_cb)?;

// ✅ Split archives
stream_opts.split_size = Some(2048 * 1024 * 1024); // 2GB volumes
```

---

## Feature 1: Archive Testing & Verification

### Why It's Needed
Forensic integrity verification - ensure archives are not corrupted before long-term storage or court presentation.

### Implementation

**Add to `src-tauri/src/commands/archive.rs`:**

```rust
use seven_zip::SevenZip;

/// Test archive integrity without extracting
#[tauri::command]
pub async fn test_7z_archive(
    archive_path: String,
    password: Option<String>,
    window: Window,
) -> Result<bool, String> {
    info!("Testing archive integrity: {}", archive_path);
    
    let window_clone = window.clone();
    let archive_path_clone = archive_path.clone();
    
    tauri::async_runtime::spawn_blocking(move || {
        let sz = SevenZip::new()
            .map_err(|e| format!("Failed to initialize 7z library: {}", e))?;
        
        // Progress callback
        let progress_cb = Box::new(move |bytes_processed: u64, bytes_total: u64, 
                                         _: u64, _: u64, current_file: &str, _: *mut std::ffi::c_void| {
            let percent = if bytes_total > 0 {
                (bytes_processed as f64 / bytes_total as f64) * 100.0
            } else {
                0.0
            };
            
            let _ = window_clone.emit("archive-test-progress", serde_json::json!({
                "archive_path": archive_path_clone,
                "current_file": current_file,
                "bytes_processed": bytes_processed,
                "bytes_total": bytes_total,
                "percent": percent,
            }));
        });
        
        // Test archive
        sz.test_archive(
            &archive_path_clone,
            password.as_deref(),
            Some(progress_cb)
        ).map_err(|e| format!("Archive test failed: {}", e))?;
        
        info!("Archive test passed: {}", archive_path);
        Ok(true)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
```

**Register command in `src-tauri/src/lib.rs`:**

```rust
tauri::generate_handler![
    // ... existing commands ...
    test_7z_archive,
]
```

**Frontend API (`src/api/archiveCreate.ts`):**

```typescript
export async function testArchive(
  archivePath: string,
  password?: string,
  onProgress?: (progress: ArchiveTestProgress) => void
): Promise<boolean> {
  const unlisten = onProgress 
    ? await listen<ArchiveTestProgress>("archive-test-progress", (event) => {
        onProgress(event.payload);
      })
    : null;
  
  try {
    const result = await invoke<boolean>("test_7z_archive", {
      archivePath,
      password: password || null,
    });
    return result;
  } finally {
    unlisten?.();
  }
}
```

---

## Feature 2: Archive Repair

### Why It's Needed
Recover data from partially corrupted archives (e.g., damaged USB drives, interrupted transfers).

### Implementation

**Add to `src-tauri/src/commands/archive.rs`:**

```rust
/// Repair corrupted archive
#[tauri::command]
pub async fn repair_7z_archive(
    corrupted_path: String,
    repaired_path: String,
    window: Window,
) -> Result<String, String> {
    info!("Repairing archive: {} -> {}", corrupted_path, repaired_path);
    
    let window_clone = window.clone();
    
    tauri::async_runtime::spawn_blocking(move || {
        let sz = SevenZip::new()
            .map_err(|e| format!("Failed to initialize 7z library: {}", e))?;
        
        // Progress callback
        let progress_cb = Box::new(move |completed: u64, total: u64, _: *mut std::ffi::c_void| {
            let percent = if total > 0 {
                (completed as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            
            let _ = window_clone.emit("archive-repair-progress", serde_json::json!({
                "percent": percent,
                "status": "Repairing archive...",
            }));
        });
        
        // Repair archive
        sz.repair_archive(
            &corrupted_path,
            &repaired_path,
            Some(progress_cb)
        ).map_err(|e| format!("Archive repair failed: {}", e))?;
        
        info!("Archive repaired successfully: {}", repaired_path);
        Ok(repaired_path)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
```

---

## Feature 3: Enhanced Error Reporting

### Why It's Needed
Better debugging and user feedback when archive operations fail.

### Implementation

**Add to `src-tauri/src/commands/archive.rs`:**

```rust
use seven_zip::advanced::{DetailedError, get_error_string};

/// Get detailed information about the last archive error
#[tauri::command]
pub fn get_last_archive_error() -> Result<serde_json::Value, String> {
    DetailedError::get_last()
        .map(|err| serde_json::json!({
            "code": err.code,
            "message": err.message,
            "file_context": err.file_context,
            "position": err.position,
            "suggestion": err.suggestion,
        }))
        .map_err(|e| format!("Failed to get error details: {}", e))
}

/// Clear last error
#[tauri::command]
pub fn clear_last_archive_error() {
    DetailedError::clear();
}
```

**Integrate into error handling:**

```rust
// In archive_create.rs
match sz.create_archive(...) {
    Ok(_) => Ok(archive_path),
    Err(e) => {
        // Try to get detailed error info
        if let Ok(details) = DetailedError::get_last() {
            warn!("Archive creation failed with details: {:?}", details);
            Err(format!(
                "Archive creation failed: {}\nFile: {}\nSuggestion: {}",
                details.message,
                details.file_context,
                details.suggestion
            ))
        } else {
            Err(format!("Archive creation failed: {}", e))
        }
    }
}
```

---

## Feature 4: Raw LZMA Compression

### Why It's Needed
Create standalone compressed files (.lzma/.xz) for single large files like disk images.

### Implementation

**Add to `src-tauri/src/commands/archive.rs`:**

```rust
use seven_zip::advanced;

/// Compress a single file to .lzma format
#[tauri::command]
pub async fn compress_to_lzma(
    input_path: String,
    output_path: String,
    compression_level: u8,
    window: Window,
) -> Result<String, String> {
    info!("Compressing to LZMA: {} -> {}", input_path, output_path);
    
    let window_clone = window.clone();
    let output_clone = output_path.clone();
    
    tauri::async_runtime::spawn_blocking(move || {
        let level = match compression_level {
            0 => CompressionLevel::Store,
            1 => CompressionLevel::Fastest,
            2..=3 => CompressionLevel::Fast,
            4..=6 => CompressionLevel::Normal,
            7..=8 => CompressionLevel::Maximum,
            9 => CompressionLevel::Ultra,
            _ => CompressionLevel::Normal,
        };
        
        let progress_cb = Box::new(move |completed: u64, total: u64, _: *mut std::ffi::c_void| {
            let percent = if total > 0 {
                (completed as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            
            let _ = window_clone.emit("lzma-compress-progress", serde_json::json!({
                "percent": percent,
                "bytes_processed": completed,
                "bytes_total": total,
            }));
        });
        
        advanced::compress_lzma(
            &input_path,
            &output_path,
            level,
            Some(progress_cb)
        ).map_err(|e| format!("LZMA compression failed: {}", e))?;
        
        Ok(output_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Decompress a .lzma file
#[tauri::command]
pub async fn decompress_lzma(
    lzma_path: String,
    output_path: String,
    window: Window,
) -> Result<String, String> {
    info!("Decompressing LZMA: {} -> {}", lzma_path, output_path);
    
    let window_clone = window.clone();
    let output_clone = output_path.clone();
    
    tauri::async_runtime::spawn_blocking(move || {
        let progress_cb = Box::new(move |completed: u64, total: u64, _: *mut std::ffi::c_void| {
            let percent = if total > 0 {
                (completed as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            
            let _ = window_clone.emit("lzma-decompress-progress", serde_json::json!({
                "percent": percent,
            }));
        });
        
        advanced::decompress_lzma(
            &lzma_path,
            &output_path,
            Some(progress_cb)
        ).map_err(|e| format!("LZMA decompression failed: {}", e))?;
        
        Ok(output_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
```

---

## Feature 5: Native Rust Encryption (Recommended)

### Why It's Needed
Pure Rust AES-256 implementation is faster and safer than FFI calls.

### Implementation

**Update `src-tauri/src/commands/archive_create.rs`:**

```rust
// Add to imports
use seven_zip::encryption_native::{NativeEncryptionContext, generate_salt, generate_iv};

// When user wants to test password strength or pre-encrypt data
#[tauri::command]
pub fn encrypt_data_native(
    data: Vec<u8>,
    password: String,
) -> Result<Vec<u8>, String> {
    let mut ctx = NativeEncryptionContext::new(&password)
        .map_err(|e| format!("Failed to initialize encryption: {}", e))?;
    
    ctx.encrypt(&data)
        .map_err(|e| format!("Encryption failed: {}", e))
}

#[tauri::command]
pub fn decrypt_data_native(
    encrypted_data: Vec<u8>,
    password: String,
) -> Result<Vec<u8>, String> {
    let mut ctx = NativeEncryptionContext::new(&password)
        .map_err(|e| format!("Failed to initialize decryption: {}", e))?;
    
    ctx.decrypt(&encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))
}
```

---

## Feature 6: Split Archive Extraction

### Why It's Needed
Extract multi-volume archives that were created with split support.

### Implementation

**Add to `src-tauri/src/commands/archive.rs`:**

```rust
use seven_zip::advanced;

/// Extract split/multi-volume archive
#[tauri::command]
pub async fn extract_split_7z_archive(
    first_volume_path: String,
    output_dir: String,
    password: Option<String>,
    window: Window,
) -> Result<String, String> {
    info!("Extracting split archive: {} to {}", first_volume_path, output_dir);
    
    let window_clone = window.clone();
    let output_clone = output_dir.clone();
    
    tauri::async_runtime::spawn_blocking(move || {
        let progress_cb = Box::new(move |bytes_processed: u64, bytes_total: u64,
                                         _: u64, _: u64, current_file: &str, _: *mut std::ffi::c_void| {
            let percent = if bytes_total > 0 {
                (bytes_processed as f64 / bytes_total as f64) * 100.0
            } else {
                0.0
            };
            
            let _ = window_clone.emit("split-extract-progress", serde_json::json!({
                "current_file": current_file,
                "bytes_processed": bytes_processed,
                "bytes_total": bytes_total,
                "percent": percent,
            }));
        });
        
        advanced::extract_split_archive(
            &first_volume_path,
            &output_dir,
            password.as_deref(),
            Some(progress_cb)
        ).map_err(|e| format!("Split archive extraction failed: {}", e))?;
        
        Ok(output_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
```

---

## Register All New Commands

**Update `src-tauri/src/lib.rs`:**

```rust
tauri::generate_handler![
    // ... existing commands ...
    
    // New archive features
    test_7z_archive,
    repair_7z_archive,
    get_last_archive_error,
    clear_last_archive_error,
    compress_to_lzma,
    decompress_lzma,
    encrypt_data_native,
    decrypt_data_native,
    extract_split_7z_archive,
]
```

---

## Frontend Integration Examples

### Archive Testing UI

**Add to `src/components/ArchiveVerifyPanel.tsx`:**

```tsx
import { createSignal } from "solid-js";
import { testArchive } from "../api/archiveCreate";
import { useToast } from "./Toast";

export function ArchiveVerifyPanel() {
  const [archivePath, setArchivePath] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [testing, setTesting] = createSignal(false);
  const [progress, setProgress] = createSignal(0);
  const toast = useToast();
  
  const handleTest = async () => {
    setTesting(true);
    try {
      const valid = await testArchive(
        archivePath(),
        password() || undefined,
        (prog) => setProgress(prog.percent)
      );
      
      if (valid) {
        toast.success("Archive Verified", "Archive integrity check passed");
      }
    } catch (error: any) {
      toast.error("Verification Failed", error.message || String(error));
    } finally {
      setTesting(false);
      setProgress(0);
    }
  };
  
  return (
    <div class="card p-4">
      <h3 class="text-lg font-semibold mb-4">Verify Archive Integrity</h3>
      
      <input
        class="input mb-2"
        type="text"
        placeholder="Archive path..."
        value={archivePath()}
        onInput={(e) => setArchivePath(e.currentTarget.value)}
      />
      
      <input
        class="input mb-4"
        type="password"
        placeholder="Password (if encrypted)"
        value={password()}
        onInput={(e) => setPassword(e.currentTarget.value)}
      />
      
      {testing() && (
        <div class="mb-4">
          <div class="w-full bg-bg-secondary rounded-full h-2">
            <div
              class="bg-accent h-2 rounded-full transition-all"
              style={{ width: `${progress()}%` }}
            />
          </div>
          <p class="text-xs text-txt-muted mt-1">{progress().toFixed(1)}% verified</p>
        </div>
      )}
      
      <button
        class="btn btn-primary"
        onClick={handleTest}
        disabled={testing() || !archivePath()}
      >
        {testing() ? "Testing..." : "Test Archive"}
      </button>
    </div>
  );
}
```

---

## Testing the Updates

### 1. Test Archive Creation with Split

```bash
# In src-tauri
cargo test
```

### 2. Test Archive Verification

```bash
# Create test archive
cargo run --example create_archive

# Verify it
cargo run --example test_archive
```

### 3. Test Error Reporting

```bash
# Try to open corrupted archive
cargo run --example test_error_reporting
```

---

## Performance Improvements

### Before Update
- ❌ No integrity verification
- ❌ No repair capability
- ❌ Generic error messages
- ❌ FFI encryption overhead

### After Update
- ✅ Built-in integrity testing
- ✅ Archive repair for corrupted files
- ✅ Detailed error context with suggestions
- ✅ Native Rust encryption (faster)
- ✅ Raw LZMA support
- ✅ Split archive extraction

---

## Migration Checklist

- [ ] Add new commands to `archive.rs`
- [ ] Register commands in `lib.rs`
- [ ] Update frontend API in `archiveCreate.ts`
- [ ] Add UI components for new features
- [ ] Test archive verification
- [ ] Test archive repair
- [ ] Test error reporting
- [ ] Update documentation
- [ ] Add user notifications for long operations

---

## Additional Resources

- **sevenzip-ffi Documentation:** `/Users/terryreynolds/GitHub/sevenzip-ffi/DOCUMENTATION.md`
- **Tauri Guide:** `/Users/terryreynolds/GitHub/sevenzip-ffi/TAURI_SOLIDJS_GUIDE.md`
- **7z SDK Reference:** `/Users/terryreynolds/GitHub/sevenzip-ffi/include/7z_ffi.h`

---

## Summary

This update adds **6 major features** to CORE-FFX:

1. ✅ **Archive Testing** - Verify integrity without extraction
2. ✅ **Archive Repair** - Recover corrupted archives
3. ✅ **Enhanced Errors** - Detailed error context
4. ✅ **Raw LZMA** - Compress individual files
5. ✅ **Native Encryption** - Faster pure Rust AES-256
6. ✅ **Split Extraction** - Extract multi-volume archives

All features are **already implemented in sevenzip-ffi** - just need to add the Tauri command wrappers and frontend UI!
