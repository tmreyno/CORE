# sevenzip-ffi C Library Analysis

**Library:** `/Users/terryreynolds/GitHub/sevenzip-ffi/build/lib7z_ffi.a`  
**Date:** February 6, 2026

---

## Summary: ⚠️ 6 out of 8 Features Available

The compiled C library (`lib7z_ffi.a`) contains **6 of the 8** target features. LZMA compression functions are **NOT implemented** in the C library.

---

## ✅ Available Features (6)

### 1. **Archive Testing** ✅
**C Function:** `sevenzip_test_archive`  
**Status:** PRESENT  
**Signature:**
```c
SevenZipErrorCode sevenzip_test_archive(
    const char* archive_path,
    const char* password,
    SevenZipBytesProgressCallback progress_callback,
    void* user_data
);
```

### 2. **Archive Repair** ✅
**C Function:** `sevenzip_repair_archive`  
**Status:** PRESENT  
**Signature:**
```c
SevenZipErrorCode sevenzip_repair_archive(
    const char* corrupted_path,
    const char* repaired_path,
    SevenZipProgressCallback progress_callback,
    void* user_data
);
```

### 3. **Archive Validation** ✅
**C Function:** `sevenzip_validate_archive`  
**Status:** PRESENT  
**Signature:**
```c
SevenZipErrorCode sevenzip_validate_archive(
    const char* archive_path,
    SevenZipErrorInfo* error_info
);
```

### 4. **LZMA Decompression** ✅
**C Function:** `sevenzip_decompress_lzma`  
**Status:** PRESENT  
**Signature:**
```c
SevenZipErrorCode sevenzip_decompress_lzma(
    const char* lzma_path,
    const char* output_path,
    SevenZipProgressCallback progress_callback,
    void* user_data
);
```

### 5. **LZMA2 Decompression** ✅
**C Function:** `sevenzip_decompress_lzma2`  
**Status:** PRESENT  
**Signature:**
```c
SevenZipErrorCode sevenzip_decompress_lzma2(
    const char* lzma2_path,
    const char* output_path,
    SevenZipProgressCallback progress_callback,
    void* user_data
);
```

### 6. **Split Archive Extraction** ✅
**C Function:** `sevenzip_extract_split_archive`  
**Status:** PRESENT  
**Signature:**
```c
SevenZipErrorCode sevenzip_extract_split_archive(
    const char* archive_path,
    const char* output_dir,
    const char* password,
    SevenZipProgressCallback progress_callback,
    void* user_data
);
```

---

## ❌ Missing Features (2)

### 7. **LZMA Compression** ❌
**C Function:** `sevenzip_compress_lzma`  
**Status:** **NOT IMPLEMENTED**  
**FFI Declaration:** Present in `ffi.rs` but function doesn't exist in C library

### 8. **LZMA2 Compression** ❌
**C Function:** `sevenzip_compress_lzma2`  
**Status:** **NOT IMPLEMENTED**  
**FFI Declaration:** Present in `ffi.rs` but function doesn't exist in C library

---

## Alternative Functions Found

The library DOES contain these related compression functions:

### XZ Compression (LZMA2-based)
```c
SevenZipErrorCode sevenzip_compress_xz(...)     // Create .xz files (LZMA2)
SevenZipErrorCode sevenzip_decompress_xz(...)   // Extract .xz files
```

**Note:** XZ format uses LZMA2 compression, so `compress_xz` could potentially be used instead of `compress_lzma2`.

---

## All Exported Functions (48 total)

```
sevenzip_apply_filter
sevenzip_cleanup
sevenzip_clear_last_error
sevenzip_compress
sevenzip_compress_bound
sevenzip_compress_buffer
sevenzip_compress_xz                    ← Alternative to compress_lzma2
sevenzip_create_7z
sevenzip_create_7z_streaming
sevenzip_create_7z_true_streaming
sevenzip_create_archive
sevenzip_create_multivolume_7z_complete
sevenzip_decompress_buffer
sevenzip_decompress_lzma                ✅
sevenzip_decompress_lzma2               ✅
sevenzip_decompress_xz
sevenzip_decrypt_data
sevenzip_detect_filter
sevenzip_encrypt_data
sevenzip_error_compression
sevenzip_error_extraction
sevenzip_error_file_open
sevenzip_error_invalid_archive
sevenzip_error_memory
sevenzip_error_wrong_password
sevenzip_extract
sevenzip_extract_archive
sevenzip_extract_files
sevenzip_extract_split_archive          ✅
sevenzip_extract_streaming
sevenzip_filter_name
sevenzip_free_list
sevenzip_get_error_message
sevenzip_get_error_string
sevenzip_get_last_error
sevenzip_get_version
sevenzip_init
sevenzip_init_decryption
sevenzip_init_encryption
sevenzip_list
sevenzip_lzma_version
sevenzip_repair_archive                 ✅
sevenzip_reverse_filter
sevenzip_set_error_internal
sevenzip_stream_options_init
sevenzip_test_archive                   ✅
sevenzip_validate_archive               ✅
sevenzip_verify_password
sevenzip_version
```

---

## Rust Bindings Status

### External Repository (`/Users/terryreynolds/GitHub/sevenzip-ffi/rust/`)

**Updated with:**
- ✅ FFI declarations for all 8 functions (in `ffi.rs`)
- ✅ Rust wrappers for `test_archive`, `repair_archive`, `validate_archive`
- ❌ No wrappers for `compress_lzma` / `compress_lzma2` (functions don't exist in C)

### CORE-1 Embedded Copy (`/Users/terryreynolds/GitHub/CORE-1/sevenzip-ffi/rust/`)

**Status:** OUTDATED
- ❌ Missing `test_archive` with progress callback
- ❌ Missing `repair_archive`
- ❌ Missing `validate_archive`
- ✅ Has `decompress_lzma`, `decompress_lzma2` (basic versions)
- ✅ Has `extract_split_archive` (basic version)

---

## Recommendations

### Option 1: Use What's Available (Current Approach)
**Implement 6 features:**
1. ✅ Archive testing
2. ✅ Archive repair
3. ✅ Archive validation
4. ✅ LZMA decompression
5. ✅ LZMA2 decompression
6. ✅ Split archive extraction

**Skip:**
- ❌ LZMA compression
- ❌ LZMA2 compression

### Option 2: Implement Missing C Functions
Add to C library source:
```c
// In 7z_ffi.c or new lzma_compress.c
SevenZipErrorCode sevenzip_compress_lzma(...) {
    // Use LZMA SDK to implement standalone LZMA compression
}

SevenZipErrorCode sevenzip_compress_lzma2(...) {
    // Use LZMA SDK to implement standalone LZMA2 compression
}
```

Then rebuild `lib7z_ffi.a`.

### Option 3: Use XZ Format (Workaround)
Use existing `sevenzip_compress_xz` for LZMA2 compression:
- XZ format = LZMA2 + simple container
- Widely compatible (.xz files)
- Already implemented in C library

---

## Next Steps for CORE-1 Integration

### Immediate (6 Features)
1. Copy updated Rust wrappers from external repo to CORE-1
2. Uncomment `test_7z_archive`, `repair_7z_archive`, `validate_7z_archive` 
3. Enable `decompress_lzma` command
4. Test all 6 working features

### Future (Add LZMA Compression)
1. Implement `sevenzip_compress_lzma` in C library source
2. Rebuild `lib7z_ffi.a`
3. Add Rust wrapper
4. Uncomment `compress_to_lzma` command

---

## Verification Commands

```bash
# Check for specific functions
nm /Users/terryreynolds/GitHub/sevenzip-ffi/build/lib7z_ffi.a | grep "sevenzip_test_archive"
nm /Users/terryreynolds/GitHub/sevenzip-ffi/build/lib7z_ffi.a | grep "sevenzip_repair_archive"
nm /Users/terryreynolds/GitHub/sevenzip-ffi/build/lib7z_ffi.a | grep "sevenzip_validate_archive"
nm /Users/terryreynolds/GitHub/sevenzip-ffi/build/lib7z_ffi.a | grep "sevenzip_compress_lzma"

# List all exported functions
nm /Users/terryreynolds/GitHub/sevenzip-ffi/build/lib7z_ffi.a | grep " T _sevenzip"
```

---

## Conclusion

The library has **6 out of 8** target features. LZMA compression functions (`compress_lzma`, `compress_lzma2`) are declared in FFI but **not implemented in the C library**.

**Recommendation:** Proceed with implementing the 6 available features now, and add LZMA compression later if needed (or use XZ compression as alternative).
