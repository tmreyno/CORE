# How C Source Files are Used in Rust

This document explains how the C source files in `/src` are integrated with the Rust library.

## Overview

The Rust library (`sevenzip-ffi`) wraps a C library that provides 7-Zip compression functionality. The C files in `/src` implement the core compression, extraction, and encryption features using the LZMA SDK.

## Architecture Flow

```
Rust Application Code
        ↓
Rust Safe API (archive.rs, compression.rs, encryption.rs, extract.rs)
        ↓
Rust FFI Bindings (ffi.rs)
        ↓
C Library Functions (src/*.c)
        ↓
LZMA SDK (lzma/C/*.c)
```

## Build Process

### 1. CMake Builds the C Library

**File: `CMakeLists.txt`** (project root)

```cmake
# Defines all C source files
set(FFI_SOURCES
    src/ffi_interface.c          # Core API initialization
    src/error_reporting.c        # Error handling
    src/archive_create.c         # Archive creation
    src/archive_extract.c        # Archive extraction
    src/archive_list.c           # List archive contents
    src/encryption_aes.c         # AES encryption
    src/lzma_compress.c          # LZMA compression
    src/lzma_decompress.c        # LZMA decompression
    # ... and more
)

# Creates static library: lib7z_ffi.a (macOS/Linux) or 7z_ffi.lib (Windows)
add_library(7z_ffi ${LZMA_SOURCES} ${FFI_SOURCES})
```

This produces: **`build/lib7z_ffi.a`** - a static library containing all C code.

### 2. Rust Build Script Links the C Library

**File: `rust/build.rs`**

```rust
fn main() {
    // 1. Checks if C library exists at build/lib7z_ffi.a
    let lib_path = build_dir.join("lib7z_ffi.a");
    
    // 2. If not found, runs CMake to build it:
    Command::new("cmake")
        .args(&["-B", "build", "-DCMAKE_BUILD_TYPE=Release"])
        .current_dir(project_root)
        .status();
    
    Command::new("cmake")
        .args(&["--build", "build", "--config", "Release"])
        .current_dir(project_root)
        .status();
    
    // 3. Tells Cargo where to find the library:
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=7z_ffi");  // Links lib7z_ffi.a
}
```

At compile time, `build.rs`:
- Ensures the C library is built
- Links `lib7z_ffi.a` into the Rust binary
- All C code becomes part of the final Rust application

### 3. Rust FFI Bindings Declare C Functions

**File: `rust/src/ffi.rs`**

```rust
#[link(name = "7z_ffi", kind = "static")]
extern "C" {
    // Declares C functions from src/*.c files
    pub fn sevenzip_init() -> SevenZipErrorCode;
    pub fn sevenzip_compress(...) -> SevenZipErrorCode;
    pub fn sevenzip_extract(...) -> SevenZipErrorCode;
    pub fn sevenzip_list(...) -> SevenZipErrorCode;
    // ... etc
}
```

These declarations match the C function signatures in the `.c` files.

### 4. Safe Rust Wrappers Call FFI Functions

**File: `rust/src/archive.rs`**

```rust
pub fn create_archive(...) -> Result<(), Error> {
    unsafe {
        // Calls C function from ffi.rs
        let result = ffi::sevenzip_create_archive(
            c_archive_path,
            c_input_paths.as_ptr(),
            level,
            c_password,
            callback,
            callback_data,
        );
        
        // Converts C error code to Rust Result
        Error::from_code(result)?;
    }
    Ok(())
}
```

## C Source Files Mapping

### Core Library Files

| C File | Purpose | Used By Rust |
|--------|---------|--------------|
| `ffi_interface.c` | Library initialization, version info | `sevenzip_init()`, `sevenzip_cleanup()` |
| `error_reporting.c` | Error message translation | `sevenzip_get_error_message()` |

### Archive Creation Files

| C File | Purpose | Rust FFI Function |
|--------|---------|-------------------|
| `archive_create.c` | Basic archive creation with LZMA2 | `sevenzip_create_archive()` |
| `archive_create_custom.c` | Custom compression options | `sevenzip_create_7z()` |
| `archive_create_multivolume.c` | Split archives | `sevenzip_compress_stream()` |
| `archive_create_true_streaming.c` | Large file streaming | `sevenzip_create_7z_true_streaming()` |

### Archive Extraction Files

| C File | Purpose | Rust FFI Function |
|--------|---------|-------------------|
| `archive_extract.c` | Basic extraction | `sevenzip_extract()` |
| `archive_extract_custom.c` | Extract specific files | `sevenzip_extract_files()` |
| `archive_extract_split.c` | Extract multi-volume archives | `sevenzip_extract_archive()` |

### Other Operations

| C File | Purpose | Rust FFI Function |
|--------|---------|-------------------|
| `archive_list.c` | List archive contents | `sevenzip_list()` |
| `archive_test.c` | Test archive integrity | `sevenzip_test()` |
| `lzma_compress.c` | Raw LZMA compression | `sevenzip_compress()` |
| `lzma_decompress.c` | Raw LZMA decompression | `sevenzip_decompress()` |
| `encryption_aes.c` | AES-256 encryption support | Used internally by archive_create.c |

## Example: How a Compression Call Works

```rust
// 1. User calls Rust API
use seven_zip::archive::create_archive;

create_archive(
    "output.7z",
    &["file1.txt", "file2.txt"],
    CompressionLevel::Normal,
    None, // no password
    None, // no callback
)?;
```

**What happens internally:**

1. **Rust API** (`archive.rs:create_archive()`)
   - Validates inputs
   - Converts Rust strings to C strings (null-terminated)
   - Converts enums to C types

2. **FFI Call** (`ffi.rs`)
   ```rust
   unsafe {
       ffi::sevenzip_create_archive(
           c_archive_path,     // *const c_char
           c_input_paths.as_ptr(), // *const *const c_char
           level,              // SevenZipCompressionLevel
           null(),             // no password
           None,               // no callback
           null_mut(),         // no user data
       )
   }
   ```

3. **C Library** (`src/archive_create.c:sevenzip_create_archive()`)
   - Opens input files
   - Compresses with LZMA2 (`lzma/C/Lzma2Enc.c`)
   - Writes 7z format headers
   - Returns error code

4. **Error Handling** (back in Rust)
   - C error code converted to `Result<(), Error>`
   - Returns to user

## Memory Management

### C Side Allocations
- C code allocates memory for: archive buffers, compression contexts, file lists
- Must be freed by C code

### Rust Ownership
- Rust creates temporary C strings (CString)
- Automatically freed when Rust objects drop
- Example:
  ```rust
  let c_path = CString::new(archive_path)?; // Allocates
  ffi::sevenzip_create_archive(c_path.as_ptr(), ...);
  // c_path automatically freed when out of scope
  ```

### FFI Safety Rules
1. Rust never frees C-allocated memory (except via C functions like `sevenzip_free_list`)
2. C never frees Rust-allocated memory
3. Pointers passed to C must remain valid for the duration of the call

## Encryption Integration

The `encryption_aes.c` file provides AES-256-CBC encryption for archives:

```c
// C implementation (encryption_aes.c)
SevenZipErrorCode sevenzip_init_encryption(
    const char* password,
    uint8_t* key,
    uint8_t* iv,
    uint32_t* aes_context
) {
    // Derives 256-bit key using PBKDF2-SHA256
    derive_key_from_password(password, salt, 16, 262144, key, 32);
    // ... initializes AES context
}
```

**However**, the Rust library uses **pure Rust crypto** instead:

```rust
// Rust implementation (rust/src/encryption_native.rs)
use aes::Aes256;
use cbc::Cipher;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

pub fn encrypt_data(data: &[u8], password: &[u8]) -> Result<Vec<u8>, Error> {
    // Pure Rust crypto - doesn't call C encryption functions
    let key = pbkdf2_hmac::<Sha256>(password, &salt, 262_144, &mut key_bytes);
    let cipher = Aes256CbcEnc::new(&key, &iv);
    // ...
}
```

**Why?** Pure Rust crypto is:
- Memory-safe (no C vulnerabilities)
- Easier to audit and maintain
- Better integration with Rust's type system

So `encryption_aes.c` is **not used** by the Rust bindings - it's available for C-only users.

## Build Dependencies

For the build to work, you need:

1. **CMake** (3.15+) - Builds the C library
2. **C Compiler** - gcc/clang (Unix) or MSVC (Windows)
3. **Rust** (1.70+) - Builds the Rust library
4. **LZMA SDK** - In `lzma/C/` directory

## Key Takeaways

1. **C files in `/src`** implement core compression logic
2. **CMake** builds these into `lib7z_ffi.a` static library
3. **`build.rs`** ensures C library is built and links it to Rust
4. **`ffi.rs`** declares C functions for Rust to call
5. **Safe wrappers** in `archive.rs` provide ergonomic Rust API
6. **All C code is statically linked** - no runtime dependencies
7. **Encryption uses pure Rust** - C encryption code not used by Rust

## Verifying the Integration

```bash
# 1. Build C library
cd /Users/terryreynolds/GitHub/sevenzip-ffi
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build

# Verify library exists
ls -lh build/lib7z_ffi.a

# 2. Build Rust library (automatically links C library)
cd rust
cargo build --release

# Verify Rust binary contains C symbols
nm target/release/libseven_zip.rlib | grep sevenzip_

# 3. Run tests to verify integration
cargo test

# 4. Use in an application
cargo run --example demo
```

## Summary

The C source files in `/src` are:
- **Compiled** by CMake into a static library (`lib7z_ffi.a`)
- **Linked** into the Rust binary by `build.rs` at compile time
- **Called** through FFI bindings in `ffi.rs`
- **Wrapped** with safe Rust APIs in `archive.rs` and other modules
- **Deployed** as a single binary with no external C dependencies

This creates a fully integrated library where Rust code can leverage battle-tested C compression code through a safe, ergonomic API.
