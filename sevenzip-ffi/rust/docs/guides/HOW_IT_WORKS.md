# 🦀 How sevenzip-ffi Library is Created and Used in Rust

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Rust Application                        │
│              (Your code using the library)                  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ Uses safe API
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  Rust Safe Wrappers                         │
│  • SevenZip struct (rust/src/archive.rs)                   │
│  • CompressionLevel, CompressOptions                        │
│  • Error handling with Result<T, Error>                     │
│  • Smart optimizations (entropy detection, threading)       │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ FFI calls
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    FFI Layer                                │
│  • rust/src/ffi.rs - C function declarations                │
│  • Unsafe extern "C" bindings                               │
│  • Type conversions (Rust ↔ C)                              │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ Links to static library
                         ▼
┌─────────────────────────────────────────────────────────────┐
│               C Library (lib7z_ffi.a)                       │
│  • src/archive_create.c                                     │
│  • src/archive_extract.c                                    │
│  • src/lzma_compress.c                                      │
│  • Built with CMake from LZMA SDK                           │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ Uses
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   LZMA SDK (C++)                            │
│  • lzma/C/ - LZMA2 compression codec                        │
│  • Industry-standard 7z implementation                      │
└─────────────────────────────────────────────────────────────┘
```

---

## 📦 How the Library is Created

### 1. Build System (build.rs)

**File:** `rust/build.rs`

The build script runs at compile time and:

1. **Finds the C library:**
   ```rust
   let lib_path = build_dir.join("lib7z_ffi.a");
   ```

2. **Builds C library if needed:**
   ```rust
   Command::new("cmake")
       .args(&["-B", "build", "-DCMAKE_BUILD_TYPE=Release"])
       .current_dir(project_root)
       .status();
   ```

3. **Links the library:**
   ```rust
   println!("cargo:rustc-link-search=native={}", lib_dir.display());
   println!("cargo:rustc-link-lib=static=7z_ffi");
   ```

### 2. FFI Layer (ffi.rs)

**File:** `rust/src/ffi.rs`

Declares C functions and types:

```rust
#[repr(C)]
pub enum SevenZipErrorCode {
    SEVENZIP_OK = 0,
    SEVENZIP_ERROR_OPEN_FILE = 1,
    // ...
}

extern "C" {
    pub fn sevenzip_init() -> SevenZipErrorCode;
    pub fn sevenzip_extract(
        archive_path: *const c_char,
        output_dir: *const c_char,
        password: *const c_char,
        callback: SevenZipProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;
}
```

### 3. Safe Rust Wrappers (archive.rs)

**File:** `rust/src/archive.rs`

Provides safe, idiomatic Rust API:

```rust
pub struct SevenZip {
    _initialized: bool,
}

impl SevenZip {
    pub fn new() -> Result<Self> {
        unsafe {
            let result = ffi::sevenzip_init();
            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }
        Ok(Self { _initialized: true })
    }
    
    pub fn extract(&self, archive_path: impl AsRef<Path>, 
                   output_dir: impl AsRef<Path>) -> Result<()> {
        // Convert Rust types to C types
        let archive_path_c = CString::new(archive_path.as_ref().to_str().unwrap())?;
        
        // Call FFI
        unsafe {
            let result = ffi::sevenzip_extract(
                archive_path_c.as_ptr(),
                output_dir_c.as_ptr(),
                ptr::null(),
                None,
                ptr::null_mut(),
            );
            
            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
        }
        Ok(())
    }
}
```

### 4. Package Definition (Cargo.toml)

**File:** `rust/Cargo.toml`

```toml
[package]
name = "seven-zip"
version = "1.2.0"
edition = "2021"

[lib]
name = "seven_zip"
path = "src/lib.rs"

[dependencies]
# Pure Rust crypto (no OpenSSL)
aes = "0.8"
cbc = "0.1"
pbkdf2 = "0.12"
sha2 = "0.10"
rand = "0.8"
zeroize = "1.7"

[build-dependencies]
cc = "1.0"  # For linking C library
```

---

## 🚀 How to Use the Library

### Method 1: As a Cargo Dependency

**In your `Cargo.toml`:**
```toml
[dependencies]
seven-zip = { path = "../sevenzip-ffi/rust" }
# or from crates.io once published:
# seven-zip = "1.2"
```

**In your code:**
```rust
use seven_zip::{SevenZip, CompressionLevel};

fn main() -> Result<(), seven_zip::Error> {
    let sz = SevenZip::new()?;
    
    // Extract archive
    sz.extract("archive.7z", "output/")?;
    
    // Create archive
    sz.create_archive(
        "backup.7z",
        &["file1.txt", "file2.txt"],
        CompressionLevel::Normal,
        None,
    )?;
    
    Ok(())
}
```

### Method 2: Run Examples

```bash
cd rust/

# Basic demo
cargo run --example demo

# Forensic archiver
cargo run --example forensic_archiver

# Encryption example
cargo run --example encryption
```

### Method 3: Include in Your Project

```bash
# Copy the entire rust/ directory to your project
cp -r sevenzip-ffi/rust /path/to/your/project/seven-zip

# In your Cargo.toml
[dependencies]
seven-zip = { path = "./seven-zip" }
```

---

## 📚 Complete Usage Examples

### Basic Archive Operations

```rust
use seven_zip::{SevenZip, CompressionLevel, CompressOptions};

// Initialize
let sz = SevenZip::new()?;

// Extract
sz.extract("archive.7z", "output_dir")?;

// Create
sz.create_archive(
    "backup.7z",
    &["file1.txt", "dir/"],
    CompressionLevel::Normal,
    None,
)?;

// List contents
let entries = sz.list("archive.7z", None)?;
for entry in entries {
    println!("{}: {} bytes", entry.name, entry.size);
}
```

### With Encryption

```rust
// Create encrypted archive
let mut opts = CompressOptions::default();
opts.password = Some("MyPassword123".to_string());

sz.create_archive(
    "secure.7z",
    &["sensitive.txt"],
    CompressionLevel::Normal,
    Some(&opts),
)?;

// Extract with password
sz.extract_with_password(
    "secure.7z",
    "output/",
    Some("MyPassword123"),
    None,
)?;
```

### Using Smart Features (NEW!)

```rust
// Auto-tuned performance
sz.create_smart_archive(
    "optimized.7z",
    &["large_file.dat"],
    CompressionLevel::Normal,
)?;

// Encrypted with convenience method
sz.create_encrypted_archive(
    "secure.7z",
    &["data/"],
    "password",
    CompressionLevel::Normal,
)?;

// Builder pattern
let opts = CompressOptions::default()
    .with_threads(4)
    .with_password("pass".to_string())
    .with_auto_detect(true);
```

### With Progress Callbacks

```rust
sz.extract_with_password(
    "large.7z",
    "output/",
    None,
    Some(Box::new(|completed, total| {
        let percent = (completed * 100) / total;
        println!("Progress: {}%", percent);
    })),
)?;
```

---

## 🏗️ Project Structure

```
sevenzip-ffi/
├── rust/                          # Rust bindings
│   ├── Cargo.toml                # Package manifest
│   ├── build.rs                  # Build script (links C lib)
│   ├── src/
│   │   ├── lib.rs               # Main library entry point
│   │   ├── ffi.rs               # FFI declarations
│   │   ├── archive.rs           # Safe archive operations
│   │   ├── error.rs             # Error handling
│   │   ├── encryption.rs        # Encryption wrapper
│   │   └── encryption_native.rs # Pure Rust AES
│   ├── examples/                # Usage examples
│   │   ├── demo.rs
│   │   ├── forensic_archiver.rs
│   │   └── encryption_example.rs
│   ├── tests/                   # Integration tests
│   │   └── integration_tests.rs
│   └── benches/                 # Performance benchmarks
│       └── compression_benchmarks.rs
│
├── src/                          # C library source
│   ├── archive_create.c
│   ├── archive_extract.c
│   ├── lzma_compress.c
│   └── ...
│
├── include/                      # C headers
│   └── 7z_ffi.h
│
├── lzma/                         # LZMA SDK
│   └── C/
│
└── build/                        # CMake build output
    └── lib7z_ffi.a              # Static library
```

---

## 🔧 Build Process

### Step 1: Build C Library

```bash
# From project root
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

Creates: `build/lib7z_ffi.a`

### Step 2: Build Rust Library

```bash
cd rust/
cargo build --release
```

Process:
1. `build.rs` runs first
2. Checks for `lib7z_ffi.a`
3. Builds it if missing (using CMake)
4. Links Rust code to C library
5. Compiles Rust wrapper

### Step 3: Run Tests

```bash
cargo test
```

### Step 4: Run Benchmarks

```bash
cargo bench
```

---

## 🎯 Key Features

### 1. Type Safety

Rust types wrap C types safely:
```rust
// C: char* (can be NULL, unsafe)
// Rust: &str or String (guaranteed valid UTF-8)

// C: int error_code
// Rust: Result<T, Error> (forced error handling)
```

### 2. Memory Safety

```rust
// No manual malloc/free
// RAII ensures cleanup
// No use-after-free bugs
// No buffer overflows
```

### 3. Ergonomic API

```rust
// Before (C):
SevenZipCompressOptions opts;
opts.num_threads = 4;
opts.password = "pass";
sevenzip_create_7z(path, files, LEVEL_NORMAL, &opts, NULL, NULL);

// After (Rust):
sz.create_smart_archive("out.7z", &["file.txt"], CompressionLevel::Normal)?;
```

### 4. Performance Optimizations

- **Incompressible detection:** Auto-skips compression for random data
- **Smart threading:** Auto-selects thread count based on file size
- **Zero-copy:** Efficient data transfer between Rust and C

---

## 📖 Documentation

### Generate Docs

```bash
cd rust/
cargo doc --open
```

### View Tests

```bash
cargo test -- --nocapture
```

### Run Examples

```bash
cargo run --example demo
cargo run --example forensic_archiver
cargo run --example encryption
```

---

## 🔍 How It Works Internally

### Example: Creating an Archive

1. **User calls Rust API:**
   ```rust
   sz.create_archive("out.7z", &["file.txt"], CompressionLevel::Normal, None)?;
   ```

2. **Rust wrapper:**
   - Validates inputs
   - Converts Rust strings to CString
   - Checks file sizes (smart threading)
   - Analyzes entropy (incompressible detection)

3. **FFI call:**
   ```rust
   unsafe {
       ffi::sevenzip_create_7z(
           archive_path_c.as_ptr(),
           input_ptrs.as_ptr(),
           level.into(),
           opts_ptr,
           None,
           ptr::null_mut(),
       )
   }
   ```

4. **C library:**
   - Opens files
   - Compresses with LZMA2
   - Writes 7z archive format
   - Returns error code

5. **Back to Rust:**
   - Checks error code
   - Converts to Result<(), Error>
   - Cleans up C strings (automatic via Drop)

---

## ✅ Summary

**The library is a three-layer system:**

1. **C Layer:** LZMA SDK + custom wrappers (high performance)
2. **FFI Layer:** Unsafe bindings between Rust and C
3. **Rust Layer:** Safe, ergonomic API with smart optimizations

**To use it:**
- Add as dependency in `Cargo.toml`
- Import `use seven_zip::*`
- Call safe Rust methods
- Enjoy automatic memory management and error handling

**Key advantages:**
- ✅ Memory safe (Rust's guarantees)
- ✅ Type safe (compile-time checking)
- ✅ Ergonomic (idiomatic Rust API)
- ✅ Fast (zero-cost abstractions, smart optimizations)
- ✅ Reliable (comprehensive tests and benchmarks)

This is a **production-ready** library combining C performance with Rust safety! 🚀
