# sevenzip-ffi C API Reference

Complete documentation of C functions available in `lib7z_ffi.a` for Rust FFI usage.

## Table of Contents

- [Library Management](#library-management)
- [Archive Creation](#archive-creation)
- [Archive Extraction](#archive-extraction)
- [Archive Information](#archive-information)
- [LZMA Compression/Decompression](#lzma-compressiondecompression)
- [AES-256 Encryption](#aes-256-encryption)
- [Error Handling](#error-handling)
- [Progress Callbacks](#progress-callbacks)
- [Data Structures](#data-structures)

---

## Library Management

### `sevenzip_init()`

Initialize the 7z library. **Must be called before any other functions.**

```c
SevenZipErrorCode sevenzip_init(void);
```

**Returns:** `SEVENZIP_OK` on success, error code otherwise.

**Critical:** This function initializes the CRC lookup table (`CrcGenerateTable()`) which is required for all compression operations. Failure to call this will result in NULL pointer crashes.

### `sevenzip_cleanup()`

Cleanup the 7z library resources.

```c
void sevenzip_cleanup(void);
```

Call this when done using the library to free internal resources.

---

## Archive Creation

### `sevenzip_compress()`

Create a 7z archive with basic options.

```c
SevenZipErrorCode sevenzip_compress(
    const char* archive_path,
    const char** input_paths,
    SevenZipCompressionLevel level,
    const char* password,
    SevenZipProgressCallback progress_callback,
    void* user_data
);
```

**Parameters:**
- `archive_path` - Path for the new archive file
- `input_paths` - NULL-terminated array of file/directory paths to compress
- `level` - Compression level (0-9, see `SevenZipCompressionLevel`)
- `password` - Optional password (NULL for no encryption)
- `progress_callback` - Optional progress callback (NULL to disable)
- `user_data` - User data passed to progress callback

**Returns:** `SEVENZIP_OK` on success, error code otherwise.

### `sevenzip_create_archive()`

Create a multi-file archive with LZMA2 compression.

```c
SevenZipErrorCode sevenzip_create_archive(
    const char* archive_path,
    const char** input_paths,
    SevenZipCompressionLevel level,
    const char* password,
    SevenZipProgressCallback progress_callback,
    void* user_data
);
```

**Parameters:** Same as `sevenzip_compress()`

**Note:** Password encryption not yet implemented for this function.

### `sevenzip_create_7z()`

Create a standard .7z archive compatible with official 7-Zip.

```c
SevenZipErrorCode sevenzip_create_7z(
    const char* archive_path,
    const char** input_paths,
    SevenZipCompressionLevel level,
    const SevenZipCompressOptions* options,
    SevenZipProgressCallback progress_callback,
    void* user_data
);
```

**Parameters:**
- `archive_path` - Path for the output .7z file
- `input_paths` - NULL-terminated array of file/directory paths
- `level` - Compression level
- `options` - Advanced options (NULL for defaults: 2 threads, solid, no password)
- `progress_callback` - Optional progress callback
- `user_data` - User data for callback

**Features:**
- Supports directories and large files (>4GB)
- Multi-threading support
- Solid compression
- Archives readable by official 7-Zip

### `sevenzip_create_7z_streaming()`

Create a 7z archive with streaming compression for large files and split archives.

```c
SevenZipErrorCode sevenzip_create_7z_streaming(
    const char* archive_path,
    const char** input_paths,
    SevenZipCompressionLevel level,
    const SevenZipStreamOptions* options,
    SevenZipBytesProgressCallback progress_callback,
    void* user_data
);
```

**Parameters:**
- `archive_path` - Base path for archive (e.g., "archive.7z")
  - For split archives, creates archive.7z.001, archive.7z.002, etc.
- `input_paths` - NULL-terminated array of file/directory paths
- `level` - Compression level (0-9)
- `options` - Streaming options (NULL for defaults)
- `progress_callback` - Optional byte-level progress callback
- `user_data` - User data for callback

**Features:**
- Processes files in chunks to avoid loading entire files into RAM
- Supports split/multi-volume archives
- Configurable chunk size (default: 64MB)
- Configurable split size (e.g., 4GB segments)

**Example:**
```c
SevenZipStreamOptions opts;
sevenzip_stream_options_init(&opts);
opts.split_size = 4294967296;  // 4GB segments
opts.chunk_size = 67108864;     // 64MB chunks
opts.num_threads = 8;

const char* files[] = {"/path/to/large/file.img", NULL};
sevenzip_create_7z_streaming("archive.7z", files, 
                              SEVENZIP_LEVEL_NORMAL, &opts, 
                              my_progress_callback, NULL);
```

### `sevenzip_create_7z_true_streaming()`

Create a 7z archive with TRUE streaming compression (ultra-low memory usage).

```c
SevenZipErrorCode sevenzip_create_7z_true_streaming(
    const char* archive_path,
    const char** input_paths,
    SevenZipCompressionLevel level,
    const SevenZipStreamOptions* options,
    SevenZipBytesProgressCallback progress_callback,
    void* user_data
);
```

**Parameters:** Same as `sevenzip_create_7z_streaming()`

**Features:**
- Processes files in 64MB chunks WITHOUT loading all data into RAM first
- Essential for large archives (10GB+) that would cause out-of-memory crashes
- Memory usage: ~250MB peak regardless of archive size

**⚠️ IMPORTANT:** Use this function for archives larger than available RAM to prevent crashes.

---

## Archive Extraction

### `sevenzip_extract()`

Extract a 7z archive.

```c
SevenZipErrorCode sevenzip_extract(
    const char* archive_path,
    const char* output_dir,
    const char* password,
    SevenZipProgressCallback progress_callback,
    void* user_data
);
```

**Parameters:**
- `archive_path` - Path to the archive file
- `output_dir` - Directory to extract to
- `password` - Optional password (NULL if not encrypted)
- `progress_callback` - Optional progress callback (NULL to disable)
- `user_data` - User data passed to callback

**Returns:** `SEVENZIP_OK` on success, error code otherwise.

### `sevenzip_extract_files()`

Extract specific files from a 7z archive.

```c
SevenZipErrorCode sevenzip_extract_files(
    const char* archive_path,
    const char* output_dir,
    const char** files,
    const char* password,
    SevenZipProgressCallback progress_callback,
    void* user_data
);
```

**Parameters:**
- `archive_path` - Path to the archive file
- `output_dir` - Directory to extract to
- `files` - NULL-terminated array of file names to extract
- `password` - Optional password (NULL if not encrypted)
- `progress_callback` - Optional progress callback
- `user_data` - User data for callback

**Returns:** `SEVENZIP_OK` on success, error code otherwise.

### `sevenzip_extract_archive()`

Extract a multi-file archive created with `sevenzip_create_archive()`.

```c
SevenZipErrorCode sevenzip_extract_archive(
    const char* archive_path,
    const char* output_dir,
    const char* password,
    SevenZipProgressCallback progress_callback,
    void* user_data
);
```

**Parameters:** Same as `sevenzip_extract()`

### `sevenzip_extract_streaming()`

Extract a 7z archive with streaming decompression and byte-level progress.

```c
SevenZipErrorCode sevenzip_extract_streaming(
    const char* archive_path,
    const char* output_dir,
    const char* password,
    SevenZipBytesProgressCallback progress_callback,
    void* user_data
);
```

**Parameters:**
- `archive_path` - Path to archive (for splits, use base name like "archive.7z.001")
- `output_dir` - Directory to extract to
- `password` - Optional password (NULL if not encrypted)
- `progress_callback` - Optional byte-level progress callback
- `user_data` - User data for callback

**Features:**
- Handles split/multi-volume archives automatically
- Byte-level progress reporting

---

## Archive Information

### `sevenzip_list()`

List contents of a 7z archive.

```c
SevenZipErrorCode sevenzip_list(
    const char* archive_path,
    const char* password,
    SevenZipList** list
);
```

**Parameters:**
- `archive_path` - Path to the archive file
- `password` - Optional password (NULL if not encrypted)
- `list` - Pointer to receive the list result (must be freed with `sevenzip_free_list()`)

**Returns:** `SEVENZIP_OK` on success, error code otherwise.

**Example:**
```c
SevenZipList* list = NULL;
SevenZipErrorCode result = sevenzip_list("archive.7z", NULL, &list);
if (result == SEVENZIP_OK) {
    for (size_t i = 0; i < list->count; i++) {
        printf("File: %s, Size: %llu bytes\n", 
               list->entries[i].name, 
               list->entries[i].size);
    }
    sevenzip_free_list(list);
}
```

### `sevenzip_free_list()`

Free memory allocated by `sevenzip_list()`.

```c
void sevenzip_free_list(SevenZipList* list);
```

**Parameters:**
- `list` - List to free

### `sevenzip_test_archive()`

Test archive integrity without extracting.

```c
SevenZipErrorCode sevenzip_test_archive(
    const char* archive_path,
    const char* password,
    SevenZipBytesProgressCallback progress_callback,
    void* user_data
);
```

**Parameters:**
- `archive_path` - Path to archive (supports split volumes)
- `password` - Optional password (NULL if not encrypted)
- `progress_callback` - Optional progress callback
- `user_data` - User data for callback

**Features:**
- Validates CRCs
- Tests decompression
- Verifies structure
- Does not write files to disk

**Returns:** `SEVENZIP_OK` if archive is valid, error code otherwise.

---

## LZMA Compression/Decompression

### `sevenzip_decompress_lzma()`

Decompress a standalone LZMA file (.lzma).

```c
SevenZipErrorCode sevenzip_decompress_lzma(
    const char* lzma_path,
    const char* output_path,
    SevenZipProgressCallback progress_callback,
    void* user_data
);
```

**Parameters:**
- `lzma_path` - Path to the .lzma file
- `output_path` - Path for the decompressed output file
- `progress_callback` - Optional progress callback
- `user_data` - User data for callback

**Returns:** `SEVENZIP_OK` on success, error code otherwise.

### `sevenzip_decompress_lzma2()`

Decompress a standalone LZMA2 file (.xz or custom format).

```c
SevenZipErrorCode sevenzip_decompress_lzma2(
    const char* lzma2_path,
    const char* output_path,
    SevenZipProgressCallback progress_callback,
    void* user_data
);
```

**Parameters:** Same as `sevenzip_decompress_lzma()`

---

## AES-256 Encryption

### Constants

```c
#define AES_KEY_SIZE 32          /* 256 bits */
#define AES_BLOCK_SIZE 16        /* 128 bits */
#define AES_NUM_IVMRK_WORDS ((1 + 1 + 15) * 4)  /* IV + keyMode + roundKeys */
```

### `sevenzip_init_encryption()`

Initialize encryption context with password.

```c
SevenZipErrorCode sevenzip_init_encryption(
    const char* password,
    uint8_t* key,
    uint8_t* iv,
    uint32_t* aes_context
);
```

**Parameters:**
- `password` - Password string (UTF-8)
- `key` - Output buffer for derived key (32 bytes)
- `iv` - Output buffer for initialization vector (16 bytes, randomly generated)
- `aes_context` - Output buffer for AES context (must be 16-byte aligned)

**Features:**
- Derives 256-bit AES key from password using PBKDF2-SHA256
- Uses 262,144 iterations (7-Zip standard)

**Returns:** `SEVENZIP_OK` on success, error code otherwise.

### `sevenzip_init_decryption()`

Initialize decryption context with password and salt.

```c
SevenZipErrorCode sevenzip_init_decryption(
    const char* password,
    const uint8_t* salt,
    size_t salt_len,
    uint8_t* key,
    uint32_t* aes_context
);
```

**Parameters:**
- `password` - Password string (UTF-8)
- `salt` - Salt used for key derivation (from archive header)
- `salt_len` - Length of salt in bytes (typically 16)
- `key` - Output buffer for derived key (32 bytes)
- `aes_context` - Output buffer for AES context (must be 16-byte aligned)

**Returns:** `SEVENZIP_OK` on success, error code otherwise.

### `sevenzip_encrypt_data()`

Encrypt data using AES-256-CBC with PKCS#7 padding.

```c
SevenZipErrorCode sevenzip_encrypt_data(
    uint32_t* aes_context,
    const uint8_t* iv,
    const uint8_t* plaintext,
    size_t plaintext_len,
    uint8_t* ciphertext,
    size_t* ciphertext_len
);
```

**Parameters:**
- `aes_context` - AES context from `sevenzip_init_encryption()`
- `iv` - Initialization vector (16 bytes)
- `plaintext` - Data to encrypt
- `plaintext_len` - Length of plaintext in bytes
- `ciphertext` - Output buffer for encrypted data (must be 16-byte aligned)
- `ciphertext_len` - In: buffer size, Out: actual encrypted length

**Returns:** `SEVENZIP_OK` on success, error code otherwise.

### `sevenzip_decrypt_data()`

Decrypt data using AES-256-CBC and verify PKCS#7 padding.

```c
SevenZipErrorCode sevenzip_decrypt_data(
    uint32_t* aes_context,
    const uint8_t* iv,
    const uint8_t* ciphertext,
    size_t ciphertext_len,
    uint8_t* plaintext,
    size_t* plaintext_len
);
```

**Parameters:**
- `aes_context` - AES context from `sevenzip_init_decryption()`
- `iv` - Initialization vector (16 bytes, from archive header)
- `ciphertext` - Encrypted data
- `ciphertext_len` - Length of ciphertext in bytes (must be multiple of 16)
- `plaintext` - Output buffer for decrypted data (must be 16-byte aligned)
- `plaintext_len` - In: buffer size, Out: actual decrypted length

**Returns:** `SEVENZIP_OK` on success, `SEVENZIP_ERROR_EXTRACT` if wrong password.

### `sevenzip_verify_password()`

Verify password correctness by decrypting test block.

```c
SevenZipErrorCode sevenzip_verify_password(
    const char* password,
    const uint8_t* encrypted_test_block,
    size_t test_block_len,
    const uint8_t* salt,
    size_t salt_len,
    const uint8_t* iv
);
```

**Parameters:**
- `password` - Password to verify
- `encrypted_test_block` - Encrypted test data
- `test_block_len` - Length of test block
- `salt` - Salt used for key derivation
- `salt_len` - Length of salt
- `iv` - Initialization vector

**Returns:** `SEVENZIP_OK` if password correct, error code otherwise.

---

## Error Handling

### Error Codes

```c
typedef enum {
    SEVENZIP_OK = 0,
    SEVENZIP_ERROR_OPEN_FILE = 1,
    SEVENZIP_ERROR_INVALID_ARCHIVE = 2,
    SEVENZIP_ERROR_MEMORY = 3,
    SEVENZIP_ERROR_EXTRACT = 4,
    SEVENZIP_ERROR_COMPRESS = 5,
    SEVENZIP_ERROR_INVALID_PARAM = 6,
    SEVENZIP_ERROR_NOT_IMPLEMENTED = 7,
    SEVENZIP_ERROR_UNKNOWN = 99
} SevenZipErrorCode;
```

### `sevenzip_get_error_message()`

Get error message for error code.

```c
const char* sevenzip_get_error_message(SevenZipErrorCode error_code);
```

**Parameters:**
- `error_code` - Error code

**Returns:** Human-readable error message (static string, never NULL).

### `sevenzip_get_error_string()`

Alias for `sevenzip_get_error_message()`.

```c
const char* sevenzip_get_error_string(SevenZipErrorCode code);
```

### `sevenzip_get_last_error()`

Get detailed information about the last error.

```c
SevenZipErrorCode sevenzip_get_last_error(SevenZipErrorInfo* error_info);
```

**Parameters:**
- `error_info` - Output structure to fill with error details

**Returns:** `SEVENZIP_OK` if error info retrieved, `SEVENZIP_ERROR_INVALID_PARAM` if NULL.

**Features:**
- Thread-safe: each thread has its own error context
- Provides error code, message, file context, position, and suggestions

### `sevenzip_clear_last_error()`

Clear the last error information.

```c
void sevenzip_clear_last_error(void);
```

Useful for debugging to ensure you're seeing fresh errors.

### `sevenzip_get_version()`

Get library version.

```c
const char* sevenzip_get_version(void);
```

**Returns:** Version string (e.g., "1.2.0").

---

## Progress Callbacks

### `SevenZipProgressCallback`

Simple progress callback for file-level operations.

```c
typedef void (*SevenZipProgressCallback)(
    uint64_t completed, 
    uint64_t total, 
    void* user_data
);
```

**Parameters:**
- `completed` - Number of files/bytes completed
- `total` - Total number of files/bytes
- `user_data` - User data passed to original function

### `SevenZipBytesProgressCallback`

Detailed byte-level progress callback for streaming operations.

```c
typedef void (*SevenZipBytesProgressCallback)(
    uint64_t bytes_processed,       /* Total bytes processed so far */
    uint64_t bytes_total,            /* Total bytes to process (0 if unknown) */
    uint64_t current_file_bytes,     /* Bytes processed in current file */
    uint64_t current_file_total,     /* Total bytes in current file */
    const char* current_file_name,   /* Name of file being processed */
    void* user_data
);
```

**Parameters:**
- `bytes_processed` - Total bytes processed so far
- `bytes_total` - Total bytes to process (0 if unknown)
- `current_file_bytes` - Bytes processed in current file
- `current_file_total` - Total bytes in current file
- `current_file_name` - Name of file being processed
- `user_data` - User data passed to original function

---

## Data Structures

### `SevenZipEntry`

Archive entry information.

```c
typedef struct {
    char* name;              /* File name (UTF-8) */
    uint64_t size;           /* Uncompressed size */
    uint64_t packed_size;    /* Compressed size */
    uint64_t modified_time;  /* Unix timestamp */
    uint32_t attributes;     /* File attributes */
    int is_directory;        /* 1 if directory, 0 if file */
} SevenZipEntry;
```

### `SevenZipList`

Archive list result.

```c
typedef struct {
    SevenZipEntry* entries;  /* Array of entries */
    size_t count;            /* Number of entries */
} SevenZipList;
```

### `SevenZipCompressionLevel`

Compression level enumeration.

```c
typedef enum {
    SEVENZIP_LEVEL_STORE = 0,      /* No compression */
    SEVENZIP_LEVEL_FASTEST = 1,    /* Fastest compression */
    SEVENZIP_LEVEL_FAST = 3,       /* Fast compression */
    SEVENZIP_LEVEL_NORMAL = 5,     /* Normal compression */
    SEVENZIP_LEVEL_MAXIMUM = 7,    /* Maximum compression */
    SEVENZIP_LEVEL_ULTRA = 9       /* Ultra compression */
} SevenZipCompressionLevel;
```

### `SevenZipCompressOptions`

Advanced compression options.

```c
typedef struct {
    int num_threads;           /* Number of threads (0 = auto, default: 2) */
    uint64_t dict_size;        /* Dictionary size in bytes (0 = auto) */
    int solid;                 /* Solid archive (1 = yes, 0 = no, default: 1) */
    const char* password;      /* Password for encryption (NULL = no encryption) */
} SevenZipCompressOptions;
```

### `SevenZipStreamOptions`

Streaming compression options for large files and split archives.

```c
typedef struct {
    int num_threads;           /* Number of threads (0 = auto, default: 2) */
    uint64_t dict_size;        /* Dictionary size in bytes (0 = auto, default: 32MB) */
    int solid;                 /* Solid archive (1 = yes, 0 = no, default: 1) */
    const char* password;      /* Password for encryption (NULL = no encryption) */
    uint64_t split_size;       /* Split archive size in bytes (0 = no split, e.g., 4GB = 4294967296) */
    uint64_t chunk_size;       /* Chunk size for streaming (0 = auto, default: 64MB) */
    const char* temp_dir;      /* Temporary directory (NULL = system default) */
    int delete_temp_on_error;  /* Delete temp files on error (1 = yes, 0 = no, default: 1) */
} SevenZipStreamOptions;
```

**Initialize with defaults:**
```c
SevenZipStreamOptions opts;
sevenzip_stream_options_init(&opts);
```

### `SevenZipErrorInfo`

Detailed error information structure.

```c
typedef struct {
    SevenZipErrorCode code;           /* Error code */
    char message[512];                 /* Error message */
    char file_context[256];            /* File being processed when error occurred */
    int64_t position;                  /* Position in file/archive (-1 if N/A) */
    char suggestion[256];              /* Actionable suggestion to fix the error */
} SevenZipErrorInfo;
```

---

## Rust FFI Integration

All functions are exported with C linkage and can be called from Rust using FFI:

```rust
use std::ffi::{CString, c_void};
use std::os::raw::{c_char, c_int};

#[link(name = "7z_ffi", kind = "static")]
extern "C" {
    fn sevenzip_init() -> c_int;
    fn sevenzip_cleanup();
    fn sevenzip_extract(
        archive_path: *const c_char,
        output_dir: *const c_char,
        password: *const c_char,
        progress_callback: Option<extern "C" fn(u64, u64, *mut c_void)>,
        user_data: *mut c_void
    ) -> c_int;
    // ... other functions
}
```

The `sevenzip-ffi/rust` directory provides safe Rust wrappers around these C functions.

---

## Complete Function List (Exported from lib7z_ffi.a)

### Core Library Functions
- `sevenzip_init()` - Initialize library (CRITICAL: must call first)
- `sevenzip_cleanup()` - Cleanup resources

### Archive Creation
- `sevenzip_compress()` - Basic compression
- `sevenzip_create_archive()` - Multi-file LZMA2 archive
- `sevenzip_create_7z()` - Standard 7z archive
- `sevenzip_create_7z_streaming()` - Streaming compression with splits
- `sevenzip_create_7z_true_streaming()` - Ultra-low memory streaming
- `sevenzip_create_multivolume_7z_complete()` - Multi-volume archive (internal)

### Archive Extraction
- `sevenzip_extract()` - Basic extraction
- `sevenzip_extract_files()` - Extract specific files
- `sevenzip_extract_archive()` - Extract multi-file archive
- `sevenzip_extract_streaming()` - Streaming extraction with byte-level progress

### Archive Information
- `sevenzip_list()` - List archive contents
- `sevenzip_free_list()` - Free list result
- `sevenzip_test_archive()` - Test archive integrity

### LZMA Operations
- `sevenzip_decompress_lzma()` - Decompress .lzma file
- `sevenzip_decompress_lzma2()` - Decompress .xz file

### Encryption
- `sevenzip_init_encryption()` - Initialize encryption context
- `sevenzip_init_decryption()` - Initialize decryption context
- `sevenzip_encrypt_data()` - Encrypt data (AES-256-CBC)
- `sevenzip_decrypt_data()` - Decrypt data (AES-256-CBC)
- `sevenzip_verify_password()` - Verify password correctness

### Error Handling
- `sevenzip_get_error_message()` - Get error message for code
- `sevenzip_get_error_string()` - Alias for get_error_message
- `sevenzip_get_last_error()` - Get detailed error info (thread-safe)
- `sevenzip_clear_last_error()` - Clear error context
- `sevenzip_set_error_internal()` - Internal error setter
- `sevenzip_error_*()` - Various error helpers

### Utility
- `sevenzip_get_version()` - Get library version
- `sevenzip_stream_options_init()` - Initialize streaming options

### Low-Level 7-Zip SDK Functions

The library also exports low-level 7-Zip SDK functions for advanced usage:

**CRC Operations:**
- `CrcGenerateTable()` - Initialize CRC lookup table (called by sevenzip_init)
- `CrcCalc()` - Calculate CRC32
- `CrcUpdate()` - Update CRC32
- `Crc64GenerateTable()` - Initialize CRC64 table
- `Crc64Calc()` - Calculate CRC64

**AES Operations:**
- `AesCbc_Init()` - Initialize AES-CBC
- `AesCbc_Encode()` - Encrypt with AES-CBC
- `AesCbc_Decode()` - Decrypt with AES-CBC
- `AesCtr_Code()` - AES-CTR mode encryption/decryption
- `AesGenTables()` - Generate AES tables
- `Aes_SetKey_Enc()` - Set encryption key
- `Aes_SetKey_Dec()` - Set decryption key

**LZMA Encoding/Decoding:**
- `LzmaEnc_*()` - LZMA encoder functions
- `LzmaDec_*()` - LZMA decoder functions
- `Lzma2Enc_*()` - LZMA2 encoder functions
- `Lzma2Dec_*()` - LZMA2 decoder functions

**Threading:**
- `CriticalSection_*()` - Mutex operations
- `Event_*()` - Event synchronization
- `AutoResetEvent_*()` - Auto-reset events

**Memory:**
- `Buf_*()` - Buffer operations
- `DynBuf_*()` - Dynamic buffer operations

**CPU Features:**
- `CPU_IsSupported_AES()` - Check AES-NI support
- `CPU_IsSupported_CRC32()` - Check CRC32 acceleration
- `CPU_IsSupported_SHA1()` - Check SHA1 acceleration
- `CPU_IsSupported_SHA2()` - Check SHA2 acceleration

---

## Usage Notes

1. **Always call `sevenzip_init()` first** - This initializes CRC tables and other global state.

2. **Thread Safety** - Most functions are thread-safe for different archives, but avoid concurrent operations on the same archive.

3. **Memory Management**:
   - Free `SevenZipList*` results with `sevenzip_free_list()`
   - Buffers for AES operations must be 16-byte aligned
   - Use streaming functions for large archives to avoid OOM

4. **Password Encryption**:
   - Uses AES-256-CBC with PKCS#7 padding
   - Key derivation: PBKDF2-SHA256 with 262,144 iterations (7-Zip standard)

5. **Split Archives**:
   - Use `.7z.001`, `.7z.002`, etc. naming convention
   - Pass base name (e.g., "archive.7z.001") to extraction functions

6. **Progress Callbacks**:
   - Use `SevenZipProgressCallback` for simple file counts
   - Use `SevenZipBytesProgressCallback` for detailed byte-level progress

7. **Error Handling**:
   - Check return codes from all functions
   - Use `sevenzip_get_last_error()` for detailed diagnostics

---

## Build Information

**Library:** `lib7z_ffi.a` (static library)  
**Version:** 1.2.0  
**Build System:** CMake 4.1.2  
**Source:** `/Users/terryreynolds/GitHub/sevenzip-ffi/`  
**Encryption:** Pure Rust AES (no OpenSSL dependency)

**Compile with:**
```bash
cd /Users/terryreynolds/GitHub/sevenzip-ffi/build
cmake -DBUILD_SHARED_LIBS=OFF ..
make -j8
```

**Link in Rust:**
```toml
[dependencies]
seven-zip = { path = "/Users/terryreynolds/GitHub/sevenzip-ffi/rust" }
```

---

## Related Documentation

- **7-Zip SDK Documentation:** `lzma_temp/DOC/`
- **Rust Bindings:** `rust/src/lib.rs`
- **Example Programs:** `build/examples/forensic_archiver`
- **Test Suite:** `build/tests/`

---

*Last Updated: January 31, 2025*  
*Library Version: 1.2.0*
