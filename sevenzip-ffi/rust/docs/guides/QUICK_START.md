# Rust Bindings Quick Start Checklist

Use this checklist to quickly integrate the 7z FFI SDK Rust bindings into your project.

## Prerequisites

- [ ] Rust 1.70+ installed (`rustc --version`)
- [ ] CMake 3.15+ installed (`cmake --version`)
- [ ] C compiler available (GCC/Clang/MSVC)

## Build C Library (One-Time Setup)

```bash
# From project root (sevenzip-ffi/)
cd /path/to/sevenzip-ffi

# Configure
cmake -B build -DCMAKE_BUILD_TYPE=Release

# Build
cmake --build build --config Release

# Verify library exists
ls build/src/lib7z_ffi.*
# macOS: lib7z_ffi.dylib
# Linux: lib7z_ffi.so
# Windows: 7z_ffi.dll
```

## Set Up Your Rust Project

### Option A: Use as Path Dependency (Recommended for Development)

```toml
# Your Cargo.toml
[dependencies]
seven-zip = { path = "/path/to/sevenzip-ffi/rust" }
```

### Option B: Configure Library Path

Create `.cargo/config.toml` in your project:

```toml
[build]
rustflags = ["-L", "/path/to/sevenzip-ffi/build/src"]
```

**OR** set environment variable:

```bash
# macOS
export DYLD_LIBRARY_PATH=/path/to/sevenzip-ffi/build/src:$DYLD_LIBRARY_PATH

# Linux
export LD_LIBRARY_PATH=/path/to/sevenzip-ffi/build/src:$LD_LIBRARY_PATH

# Windows
set PATH=C:\path\to\sevenzip-ffi\build\src\Release;%PATH%
```

## Test Installation

Create `test_integration.rs`:

```rust
use seven_zip::SevenZip;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing 7z FFI SDK integration...");
    
    // Initialize library
    let sz = SevenZip::new()?;
    println!("Library initialized successfully!");
    
    // Test pure Rust encryption (no OpenSSL required)
    use seven_zip::encryption_native::EncryptionContext;
    let ctx = EncryptionContext::new("test_password")?;
    let plaintext = b"Hello, World!";
    let ciphertext = ctx.encrypt(plaintext)?;
    let decrypted = ctx.decrypt(&ciphertext)?;
    assert_eq!(plaintext, decrypted.as_slice());
    println!("Encryption working!");
    
    println!("\n All tests passed! Integration successful!");
    Ok(())
}
```

Run it:

```bash
cargo run --bin test_integration
```

## Common Use Cases

### 1. Extract Archive

```rust
use seven_zip::SevenZip;

let sz = SevenZip::new()?;
sz.extract("archive.7z", "output_directory")?;
println!("Extraction complete!");
```

### 2. Create Archive

```rust
use seven_zip::{SevenZip, CompressionLevel};

let sz = SevenZip::new()?;
sz.create_archive(
    "archive.7z",
    &["file1.txt", "file2.txt", "directory"],
    CompressionLevel::Normal,
    None
)?;
println!("Archive created!");
```

### 3. Create Encrypted Archive

```rust
use seven_zip::{SevenZip, CompressionLevel, CompressOptions};

let sz = SevenZip::new()?;
let mut opts = CompressOptions::default();
opts.password = Some("your_password".to_string());
opts.num_threads = 8;

sz.create_archive(
    "encrypted.7z",
    &["sensitive_data"],
    CompressionLevel::Maximum,
    Some(&opts)
)?;
println!("Encrypted archive created!");
```

### 4. Extract with Progress

```rust
use seven_zip::SevenZip;

let sz = SevenZip::new()?;
sz.extract_with_password(
    "large.7z",
    "output",
    Some("password"),  // None if not encrypted
    Some(Box::new(|completed, total| {
        let pct = (completed as f64 / total as f64) * 100.0;
        print!("\rProgress: {:.1}%", pct);
        std::io::Write::flush(&mut std::io::stdout()).ok();
    }))
)?;
println!("\nExtraction complete!");
```

### 5. List Archive Contents

```rust
use seven_zip::SevenZip;

let sz = SevenZip::new()?;
let entries = sz.list("archive.7z", None)?;

for entry in entries {
    println!(
        "{:<40} {:>12} bytes (compressed: {:>12}) ratio: {:.1}%",
        entry.name,
        entry.size,
        entry.compressed_size,
        entry.compression_ratio()
    );
}
```

## Troubleshooting

### Problem: `error: linking with 'cc' failed`

**Solution**: Library not found by linker.

```bash
# Set library path (pick one method above)
# Then rebuild:
cargo clean
cargo build
```

### Problem: `undefined reference to 'AES_encrypt'`

**Solution**: Missing OpenSSL.

```bash
# macOS
brew install openssl

# Linux
sudo apt-get install libssl-dev
```

### Problem: Tests fail with "Archive not found"

**Solution**: Run from correct directory.

```bash
cd rust
cargo test
```

### Problem: Slow performance

**Solution**: Build in release mode.

```bash
cargo build --release
cargo run --release
```

## Examples to Try

### Run Complete Demo

```bash
cd /path/to/sevenzip-ffi/rust
cargo run --example complete_demo
```

This demonstrates:
- Library initialization
- Archive creation
- Archive listing
- Archive extraction
- Encrypted archives
- Progress tracking
- Integrity testing
- Direct encryption
- Single file compression

### Run Archive Tool

```bash
# Compress with encryption
cargo run --example archive_tool -- compress test.7z /path/to/data password

# Extract
cargo run --example archive_tool -- extract test.7z ./output password

# Test integrity
cargo run --example archive_tool -- test test.7z password
```

### Run Encryption Example

```bash
cargo run --example encryption_example
```

## Documentation

- **API Documentation**: `cargo doc --open`
- **User Guide**: [README_RUST_BINDINGS.md](README_RUST_BINDINGS.md)
- **Build Guide**: [BUILD_GUIDE.md](BUILD_GUIDE.md)
- **Status**: [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md)

## Next Steps

1. **Read the API docs**: `cargo doc --open`
2. **Study the examples**: Check `rust/examples/`
3. **Write your application**: Use the patterns above
4. **Add error handling**: Use `?` operator or `match`
5. **Consider progress UI**: Add progress bars for long operations

## Pre-Deployment Checklist

Before deployment:

- [ ] Build in release mode (`cargo build --release`)
- [ ] Test with real data
- [ ] Verify encryption with strong passwords
- [ ] Test error handling (wrong password, corrupt archive)
- [ ] Validate on target platform
- [ ] Run integration tests (`cargo test`)
- [ ] Check memory usage (Valgrind/profiling)
- [ ] Verify hardware acceleration (`sysctl -a | grep aes` or `grep aes /proc/cpuinfo`)
- [ ] Document your password policy
- [ ] Set up backup/recovery procedures

## Performance Tips

1. **Use multiple threads**:
   ```rust
   opts.num_threads = 8;  // or num_cpus::get()
   ```

2. **Choose appropriate compression level**:
   - `Store` - No compression (fastest)
   - `Fast` - Quick compression
   - `Normal` - Balanced (recommended)
   - `Maximum` - Best ratio
   - `Ultra` - Maximum compression (slowest)

3. **Use solid archives for many small files**:
   ```rust
   opts.solid = true;
   ```

4. **Monitor progress for long operations**:
   ```rust
   Some(Box::new(|completed, total| {
       // Update UI
   }))
   ```

## Security Best Practices

1. **Use strong passwords** (12+ characters, mixed case, numbers, symbols)
2. **Store passwords securely** (use keychain/vault, not plaintext)
3. **Verify integrity** after creation (`test_archive()`)
4. **Handle wrong password gracefully** (catch `Error::Extract`)
5. **Use hardware acceleration** (verify AES-NI available)
6. **Zero sensitive data** (library does this automatically)
7. **Verify decryption** (check output makes sense)

## Support

- **GitHub Issues**: Report bugs or request features
- **Documentation**: Check docs.rs for API reference
- **Examples**: Study the example programs
- **Build Guide**: Detailed platform-specific instructions

##  Success!

If you've completed this checklist, you're ready to use the 7z FFI SDK in your Rust project!

Happy coding! 🦀
