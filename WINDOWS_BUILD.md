# Windows Build Requirements

## System Requirements

### Required Software

1. **Visual Studio Build Tools 2022** (or Visual Studio 2022)
   - Required components:
     - MSVC v143 - VS 2022 C++ x64/x86 build tools
     - Windows 10/11 SDK
   - Download: https://visualstudio.microsoft.com/downloads/

2. **Rust Toolchain**
   - Install via rustup: https://rustup.rs/
   - Ensure `stable-x86_64-pc-windows-msvc` toolchain is installed
   ```powershell
   rustup default stable-x86_64-pc-windows-msvc
   ```

3. **Node.js** (v18 or later)
   - Download: https://nodejs.org/
   - Includes npm package manager

4. **WebView2 Runtime** (usually pre-installed on Windows 11)
   - Download: https://developer.microsoft.com/en-us/microsoft-edge/webview2/

5. **CMake** (required by `libarchive2-sys` vendored C build)
   - Download: https://cmake.org/download/
   - Or via chocolatey: `choco install cmake`

### Optional But Recommended

1. **Ninja Build** (faster CMake builds)
   - Download: https://github.com/ninja-build/ninja/releases
   - Or via chocolatey: `choco install ninja`

2. **vcpkg** (enables full archive codec support — zlib, bz2, lzma, zstd, lz4 — in libarchive)
   - Without vcpkg, libarchive builds in minimal mode (deflate-only)
   - https://vcpkg.io/en/getting-started

## Native Library Dependencies

### Bundled (No External Dependencies Required)

These are statically linked and require no system libraries:

- ✅ **SQLite** - `rusqlite` with `bundled` feature
- ✅ **zlib** - `flate2` with `zlib-rs` feature (pure Rust, no CMake)
- ✅ **bzip2** - `bzip2` with `static` feature
- ✅ **xz/lzma** - `xz2` with `static` feature
- ✅ **zstd** - `zstd` with multi-threading support
- ✅ **sevenz-rust** - Pure Rust implementation

### Compiled From Vendored Source (Automatic)

- ✅ **UnRAR** - `unrar_sys` vendors the C++ source and compiles it via `cc` crate
  - Requires MSVC C++ build tools (already in prerequisite #1)
  - No external DLL or download needed
- ✅ **libarchive** - `libarchive2-sys` vendors libarchive v3.8.1 C source, builds via CMake
  - Full codec support requires vcpkg (see Optional section above)
  - Without vcpkg: builds in minimal mode (uncompressed + deflate only)

## Known Issues and Workarounds

### 1. SHA2-ASM Compilation Error
**Status**: ✅ FIXED
- **Issue**: `sha2-asm` v0.6.4 fails to compile on Windows MSVC
- **Solution**: Removed `asm` feature from `sha1` and `sha2` dependencies
- **Impact**: Slightly slower hashing, but still fast

### 2. TryFromIntError in WebView2
**Status**: ✅ FIXED
- **Issue**: Large file responses cause `TryFromIntError` in Windows API
- **Solution**: Updated to Tauri 2.1+ which includes fixes
- **Impact**: Can now handle large forensic files correctly

### 3. File Lock on Build Directory
**Issue**: Cargo build can leave file locks
**Solution**: 
```powershell
# Kill any lingering processes
Get-Process | Where-Object {$_.ProcessName -like "*cargo*" -or $_.ProcessName -like "*core-ffx*"} | Stop-Process -Force

# Remove lock file
Remove-Item -Recurse -Force src-tauri\target\.cargo-lock -ErrorAction SilentlyContinue
```

## Building for Release

### Development Build
```powershell
npm install
npm run tauri dev
```

### Production Build
```powershell
# Clean build
npm run build
npm run tauri build

# Output location:
# src-tauri\target\release\core-ffx.exe
# src-tauri\target\release\bundle\msi\CORE-FFX_0.1.0_x64_en-US.msi
```

### Build Size Optimization

Current configuration in `Cargo.toml`:
```toml
[profile.release]
opt-level = 3           # Maximum optimizations
lto = "thin"            # Link-time optimization
codegen-units = 1       # Better optimization
strip = true            # Strip debug symbols
panic = "abort"         # Smaller binary
```

Expected release build size: ~15-25 MB (stripped)

## Performance Considerations

### Windows-Specific Features

1. **File I/O Optimization**
   - Uses `memmap2` for memory-mapped file access
   - Multi-threaded hashing with `rayon`
   - Windows API integration via `windows` crate

2. **Compression Performance**
   - `zlib-rs` (pure Rust zlib) for fast decompression without CMake dependency
   - `blake3` with `rayon` feature for parallel hashing
   - Multi-threaded zstd decompression

3. **Heap Allocator**
   - Uses system allocator (Windows Heap API)
   - Consider `mimalloc` for better performance in future:
   ```toml
   [target.'cfg(windows)'.dependencies]
   mimalloc = "0.1"
   ```

## Troubleshooting

### Error: "error LNK1181: cannot open input file"
**Cause**: MSVC build tools not found
**Solution**: 
1. Install Visual Studio Build Tools
2. Restart terminal to pick up environment variables
3. Or run from "x64 Native Tools Command Prompt"

### Error: "failed to run custom build command for `sha2-asm`"
**Cause**: Assembly compilation not working with MSVC
**Solution**: Already fixed - `asm` features are disabled

### Error: "Port 1420 is already in use"
**Cause**: Previous dev server still running
**Solution**:
```powershell
Get-NetTCPConnection -LocalPort 1420 | Select-Object -ExpandProperty OwningProcess | ForEach-Object { Stop-Process -Id $_ -Force }
```

### Warning: "unused import: `Duration`"
**Cause**: Cleanup needed in `src\ad1\extract_v2.rs:277`
**Impact**: Harmless warning, doesn't affect functionality

## Deployment Checklist

- [ ] Visual Studio Build Tools installed (MSVC v143 + Windows SDK)
- [ ] Rust MSVC toolchain installed (`stable-x86_64-pc-windows-msvc`)
- [ ] Node.js (v18+) and npm installed
- [ ] CMake installed and on PATH
- [ ] WebView2 Runtime installed
- [ ] Dependencies installed: `npm install`
- [ ] Cargo dependencies updated: `cargo update`
- [ ] Clean build successful: `npm run tauri build`
- [ ] Application launches without errors
- [ ] Large file handling tested (no crashes)
- [ ] Archive support verified (ZIP, 7z, TAR.*)

## Automated Build Script

```powershell
# build.ps1 - Automated Windows build script

# Check prerequisites
$requiredTools = @("rustc", "cargo", "node", "npm", "cmake")
foreach ($tool in $requiredTools) {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        Write-Error "$tool not found in PATH. Please install required tools."
        exit 1
    }
}

# Clean previous build
Write-Host "Cleaning previous build..."
Remove-Item -Recurse -Force node_modules, dist, src-tauri/target -ErrorAction SilentlyContinue

# Install dependencies
Write-Host "Installing npm dependencies..."
npm install
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Update Cargo dependencies
Write-Host "Updating Cargo dependencies..."
Set-Location src-tauri
cargo update
Set-Location ..
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Build release
Write-Host "Building release..."
npm run tauri build
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`nBuild complete!"
Write-Host "Executable: src-tauri\target\release\core-ffx.exe"
Write-Host "Installer: src-tauri\target\release\bundle\msi\CORE-FFX_0.1.0_x64_en-US.msi"
```

## Cross-Compiling from macOS

You can build Windows `.exe` files directly from macOS using the provided build script.

### Quick Start

```bash
# 1. Install toolchain (one-time)
./scripts/build-windows.sh --setup

# 2. Verify everything is ready
./scripts/build-windows.sh --check

# 3. Build Windows .exe locally
./scripts/build-windows.sh
```

### Build Modes

| Mode | Command | Output | Features |
|------|---------|--------|----------|
| **Local** | `./scripts/build-windows.sh --local` | `.exe` binary | All Rust features; C FFI stubs (7z/EWF creation disabled) |
| **CI** | `./scripts/build-windows.sh --ci` | MSI + NSIS + `.exe` | Full features (pre-built native libs) |
| **CI + Wait** | `./scripts/build-windows.sh --ci --wait` | Downloads artifacts | Full features + auto-download |
| **Docker** | `./scripts/build-windows.sh --docker` | `.exe` binary | All Rust features; C FFI stubs |

### Local Cross-Compilation (cargo-xwin)

Uses [cargo-xwin](https://github.com/rust-cross/cargo-xwin) to cross-compile the Rust backend from macOS to Windows x64 MSVC. The tool automatically downloads the Windows SDK and CRT headers.

**Requirements:** Rust, cargo-xwin, clang (Xcode CLT), Node.js

**Limitations:** The C FFI libraries (sevenzip-ffi, libewf-ffi) compile with stub fallbacks, so 7z archive creation and EWF image creation return errors at runtime. All other features (container parsing, file viewing, hashing, searching, etc.) work normally. To get full C FFI support, run the pre-build workflow first:

```bash
# Pre-build Windows native libs in CI → creates a PR with .lib files
./scripts/build-windows.sh --prebuild
```

### GitHub Actions CI Build

Triggers the existing release workflow on GitHub Actions, which builds on a real Windows runner with full native library support (libarchive, libewf, sevenzip-ffi). Produces MSI and NSIS installers.

**Requirements:** [GitHub CLI](https://cli.github.com/) (`brew install gh`), authenticated (`gh auth login`)

### Docker Cross-Compilation

Runs the cross-compilation inside a Docker container for reproducibility.

**Requirements:** [Docker Desktop](https://docker.com/products/docker-desktop)

```bash
# Build and run
./scripts/build-windows.sh --docker

# Output: build-artifacts/windows-docker/core-ffx.exe
```

### Output Locations

| Mode | Path |
|------|------|
| Local | `src-tauri/target/x86_64-pc-windows-msvc/release/core-ffx.exe` |
| CI | `build-artifacts/windows/` (after download) |
| Docker | `build-artifacts/windows-docker/core-ffx.exe` |

## References

- [Tauri v2 Prerequisites](https://v2.tauri.app/start/prerequisites/#windows)
- [Rust Windows MSVC Guide](https://rust-lang.github.io/rustup/installation/windows-msvc.html)
- [Windows Build Tools](https://learn.microsoft.com/en-us/windows/dev-environment/rust/setup)
- [cargo-xwin](https://github.com/rust-cross/cargo-xwin) — Cross-compile Rust for Windows on macOS/Linux
