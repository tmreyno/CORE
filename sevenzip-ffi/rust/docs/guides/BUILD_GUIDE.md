# Rust Bindings Build Guide

Complete guide to building and using the 7z FFI SDK Rust bindings.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Building from Source](#building-from-source)
3. [Integration Methods](#integration-methods)
4. [Features](#features)
5. [Testing](#testing)
6. [Platform-Specific Notes](#platform-specific-notes)
7. [Troubleshooting](#troubleshooting)

## Prerequisites

### Required Software

- **Rust**: 1.70 or later
  ```bash
  # Check version
  rustc --version
  
  # Install/update via rustup
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **CMake**: 3.15 or later
  ```bash
  # macOS
  brew install cmake
  
  # Linux (Debian/Ubuntu)
  sudo apt-get install cmake
  
  # Check version
  cmake --version
  ```

- **C Compiler**:
  - macOS: Xcode Command Line Tools (`xcode-select --install`)
  - Linux: GCC or Clang (`sudo apt-get install build-essential`)
  - Windows: Visual Studio 2019+ or MinGW-w64

### Optional Tools

- **Valgrind** (memory leak detection, Linux/macOS):
  ```bash
  # macOS
  brew install valgrind
  
  # Linux
  sudo apt-get install valgrind
  ```

- **Miri** (Rust UB detection):
  ```bash
  rustup +nightly component add miri
  ```

## Building from Source

### Step 1: Clone Repository

```bash
git clone https://github.com/yourusername/sevenzip-ffi.git
cd sevenzip-ffi
```

### Step 2: Build C Library

The Rust bindings link against the C library, so build it first:

```bash
# Create build directory
cmake -B build -DCMAKE_BUILD_TYPE=Release

# Build
cmake --build build --config Release

# Verify library exists
ls build/src/lib7z_ffi.* # macOS: .dylib, Linux: .so, Windows: .dll
```

### Step 3: Build Rust Bindings

```bash
cd rust

# Build in debug mode
cargo build

# Build in release mode (optimized)
cargo build --release

# Build and run tests
cargo test

# Build documentation
cargo doc --open
```

### Step 4: Run Examples

```bash
# Complete demo (shows all features)
cargo run --example complete_demo

# Archive tool CLI
cargo run --example archive_tool -- --help

# Encryption example
cargo run --example encryption_example
```

## Integration Methods

### Method 1: Direct Dependency (Recommended for Development)

If you're developing within the sevenzip-ffi repository:

```toml
# Your Cargo.toml
[dependencies]
seven-zip = { path = "../rust" }
```

### Method 2: Git Dependency

```toml
[dependencies]
seven-zip = { git = "https://github.com/yourusername/sevenzip-ffi", branch = "main" }
```

### Method 3: Published Crate (Future)

Once published to crates.io:

```toml
[dependencies]
seven-zip = "1.2.0"
```

### Method 4: Local Path

```toml
[dependencies]
seven-zip = { path = "/path/to/sevenzip-ffi/rust" }
```

## Linking Configuration

### Option A: System Library (Preferred)

Install the C library system-wide:

```bash
# Build with install target
cmake -B build -DCMAKE_INSTALL_PREFIX=/usr/local
cmake --build build
sudo cmake --install build

# Update library cache (Linux)
sudo ldconfig
```

Then create `rust/.cargo/config.toml`:

```toml
[build]
rustflags = ["-L", "/usr/local/lib"]
```

### Option B: Local Build

Set environment variables before building:

```bash
# macOS/Linux
export DYLD_LIBRARY_PATH=/path/to/sevenzip-ffi/build/src:$DYLD_LIBRARY_PATH  # macOS
export LD_LIBRARY_PATH=/path/to/sevenzip-ffi/build/src:$LD_LIBRARY_PATH      # Linux

# Build
cd rust
cargo build
```

### Option C: Build Script Integration

Create `rust/build.rs`:

```rust
use std::env;
use std::path::PathBuf;

fn main() {
    // Get project root
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_root = PathBuf::from(&manifest_dir).parent().unwrap().to_path_buf();
    
    // Tell cargo where to find the library
    let lib_dir = project_root.join("build").join("src");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=7z_ffi");
    
    // Re-run if C library changes
    println!("cargo:rerun-if-changed=../src/");
    println!("cargo:rerun-if-changed=../include/");
}
```

Then add to `Cargo.toml`:

```toml
[build-dependencies]
cc = "1.0"
```

## Features

### Pure Rust AES-256 Encryption

The Rust crate includes a pure Rust encryption module that requires **no OpenSSL dependency**. This makes cross-compilation easy and eliminates external library requirements.

#### Encryption Module (`encryption_native`)

```rust
use seven_zip::encryption_native::{EncryptionContext, DecryptionContext};

// Encrypt data
let ctx = EncryptionContext::new("my_password")?;
let ciphertext = ctx.encrypt(b"Sensitive data")?;

// Decrypt data
let ctx = DecryptionContext::new("my_password")?;
let plaintext = ctx.decrypt(&ciphertext)?;

// Verify password
let valid = seven_zip::encryption_native::verify_password(&ciphertext, "my_password")?;
```

#### Security Features

- **AES-256-CBC** encryption (NSA TOP SECRET approved)
- **PBKDF2-SHA256** key derivation with 262,144 iterations (7-Zip compatible)
- **Automatic memory zeroization** using the `zeroize` crate
- **Secure random** salt and IV generation

#### Dependencies (all pure Rust)

The encryption module uses RustCrypto crates:

| Crate | Version | Purpose |
|-------|---------|---------|
| `aes` | 0.8 | AES block cipher |
| `cbc` | 0.1 | CBC mode operation |
| `pbkdf2` | 0.12 | Key derivation |
| `sha2` | 0.10 | SHA-256 for PBKDF2 |
| `rand` | 0.8 | Secure random generation |
| `zeroize` | 1.7 | Memory wiping |

### Feature Flags

```toml
[dependencies]
seven-zip = { version = "1.2", features = ["native-crypto"] }
```

Available features:

- **`default`** - Core archive operations
- **`native-crypto`** - Pure Rust AES-256 encryption (recommended)
- **`c-crypto`** - C library encryption (requires OpenSSL)

## Testing

### Unit Tests

```bash
cd rust
cargo test
```

### Integration Tests

Create `rust/tests/integration_test.rs`:

```rust
use seven_zip::{SevenZip, CompressionLevel};
use tempfile::TempDir;

#[test]
fn test_create_and_extract() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("test.7z");
    
    // Create
    let sz = SevenZip::new().unwrap();
    std::fs::write(temp.path().join("test.txt"), b"Hello").unwrap();
    
    sz.create_archive(
        archive.to_str().unwrap(),
        &[temp.path().join("test.txt").to_str().unwrap()],
        CompressionLevel::Normal,
        None
    ).unwrap();
    
    // Extract
    let extract_dir = temp.path().join("extracted");
    std::fs::create_dir(&extract_dir).unwrap();
    sz.extract(archive.to_str().unwrap(), extract_dir.to_str().unwrap()).unwrap();
    
    // Verify
    let content = std::fs::read_to_string(extract_dir.join("test.txt")).unwrap();
    assert_eq!(content, "Hello");
}

#[test]
fn test_encryption_roundtrip() {
    // Using pure Rust encryption (no OpenSSL required)
    use seven_zip::encryption_native::EncryptionContext;
    
    let ctx = EncryptionContext::new("password").unwrap();
    let plaintext = b"Secret data";
    
    let ciphertext = ctx.encrypt(plaintext).unwrap();
    let decrypted = ctx.decrypt(&ciphertext).unwrap();
    
    assert_eq!(plaintext, decrypted.as_slice());
}

#[test]
fn test_password_verification() {
    use seven_zip::encryption_native::{EncryptionContext, verify_password};
    
    let ctx = EncryptionContext::new("correct_password").unwrap();
    let ciphertext = ctx.encrypt(b"test data").unwrap();
    
    // Correct password
    assert!(verify_password(&ciphertext, "correct_password").unwrap());
    
    // Wrong password
    assert!(!verify_password(&ciphertext, "wrong_password").unwrap());
}
```

Run integration tests:

```bash
cargo test --test integration_test
```

### Memory Safety Testing

#### With Valgrind (Linux/macOS)

```bash
cargo build --example complete_demo
valgrind --leak-check=full --show-leak-kinds=all \
    ./target/debug/examples/complete_demo
```

#### With Miri (Rust UB detection):

```bash
MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test
```

### Benchmark Tests

Create `rust/benches/benchmark.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use seven_zip::encryption_native::EncryptionContext;

fn encryption_benchmark(c: &mut Criterion) {
    c.bench_function("aes256_encrypt_1mb", |b| {
        let ctx = EncryptionContext::new("password").unwrap();
        let data = vec![0u8; 1024 * 1024]; // 1MB
        
        b.iter(|| {
            ctx.encrypt(black_box(&data)).unwrap()
        });
    });
    
    c.bench_function("pbkdf2_key_derivation", |b| {
        b.iter(|| {
            EncryptionContext::new(black_box("password")).unwrap()
        });
    });
}

criterion_group!(benches, encryption_benchmark);
criterion_main!(benches);
```

Add to `Cargo.toml`:

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "benchmark"
harness = false
```

Run benchmarks:

```bash
cargo bench
```

## Platform-Specific Notes

### macOS (ARM64 - M1/M2)

- AES-NI hardware acceleration: Supported
- Build flags: No special flags needed
- Library location: `build/src/lib7z_ffi.dylib`

```bash
# Set library path for testing
export DYLD_LIBRARY_PATH=$PWD/build/src:$DYLD_LIBRARY_PATH
```

### Linux (x86_64)

- AES-NI hardware acceleration: Supported
- Build flags: May need `-lcrypto` if using system OpenSSL
- Library location: `build/src/lib7z_ffi.so`

```bash
# Set library path for testing
export LD_LIBRARY_PATH=$PWD/build/src:$LD_LIBRARY_PATH

# Check for hardware acceleration
grep aes /proc/cpuinfo
```

### Windows (x86_64)

- AES-NI hardware acceleration: Supported
- Build with Visual Studio 2019+
- Library location: `build\src\Release\7z_ffi.dll`

```powershell
# Build
cmake -B build -G "Visual Studio 16 2019"
cmake --build build --config Release

# Set PATH for testing
$env:PATH += ";$PWD\build\src\Release"
```

## Troubleshooting

### Issue: `error: linking with 'cc' failed`

**Cause**: C library not found by linker.

**Solutions**:
1. Set library path:
   ```bash
   export LD_LIBRARY_PATH=/path/to/build/src:$LD_LIBRARY_PATH
   ```

2. Or create `.cargo/config.toml`:
   ```toml
   [build]
   rustflags = ["-L", "/path/to/build/src"]
   ```

3. Or install system-wide:
   ```bash
   sudo cmake --install build
   ```

### Issue: `undefined reference to 'AES_encrypt'`

**Cause**: Missing OpenSSL dependency.

**Solution**: Link OpenSSL:
```bash
# macOS
brew install openssl
export LDFLAGS="-L/opt/homebrew/opt/openssl/lib"

# Linux
sudo apt-get install libssl-dev
```

### Issue: `error: failed to run custom build command`

**Cause**: Build script (`build.rs`) can't find C library.

**Solution**: Build C library first:
```bash
cmake -B build
cmake --build build
```

### Issue: Archive extraction fails with garbage output

**Cause**: ABI mismatch or wrong library version.

**Solution**: Rebuild both C and Rust:
```bash
# Clean everything
rm -rf build target
cmake -B build
cmake --build build
cd rust && cargo clean && cargo build
```

### Issue: Tests fail with "Archive not found"

**Cause**: Tests run from wrong directory.

**Solution**: Run from rust directory:
```bash
cd rust
cargo test
```

### Issue: Slow encryption performance

**Cause**: Hardware acceleration not enabled or debug build.

**Checks**:
1. Build in release mode: `cargo build --release`
2. Verify AES-NI:
   ```bash
   # macOS
   sysctl -a | grep aes
   
   # Linux
   grep aes /proc/cpuinfo
   ```
3. Check CPU flags in output (should show "Hardware AES: Yes")

## Development Tips

### Fast Iteration

Use `cargo watch` for automatic rebuilding:

```bash
cargo install cargo-watch
cargo watch -x build -x test
```

### Documentation Development

Preview docs locally:

```bash
cargo doc --no-deps --open
```

### Clippy Linting

Run Clippy for code quality:

```bash
cargo clippy -- -D warnings
```

### Format Code

```bash
cargo fmt
```

## CI/CD Integration

### GitHub Actions Example

Create `.github/workflows/rust.yml`:

```yaml
name: Rust CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        rust: [stable, nightly]
    
    steps:
    - uses: actions/checkout@v2
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
        override: true
    
    - name: Build C Library
      run: |
        cmake -B build -DCMAKE_BUILD_TYPE=Release
        cmake --build build
    
    - name: Build Rust
      run: cd rust && cargo build --verbose
    
    - name: Run Tests
      run: cd rust && cargo test --verbose
    
    - name: Run Clippy
      run: cd rust && cargo clippy -- -D warnings
```

## Next Steps

1. **Try the examples**: `cargo run --example complete_demo`
2. **Read the API docs**: `cargo doc --open`
3. **Write your first app**: See [README_RUST_BINDINGS.md](README_RUST_BINDINGS.md)
4. **Contribute**: See [CONTRIBUTING.md](../CONTRIBUTING.md)

## Support

- GitHub Issues: https://github.com/yourusername/sevenzip-ffi/issues
- Documentation: https://docs.rs/seven-zip
- Examples: `/rust/examples/`

Happy coding! 🦀
