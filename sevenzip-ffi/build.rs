// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
//
// sevenzip-ffi build script
// Links against pre-built lib7z_ffi.a static library (LZMA SDK 24.09)
//
// The C library is built separately in the standalone sevenzip-ffi repo:
//   cd ~/GitHub/sevenzip-ffi && cmake -B build -DCMAKE_BUILD_TYPE=Release && cmake --build build
// Then copied here:
//   cp build/lib7z_ffi.a ~/GitHub/CORE-1/sevenzip-ffi/build/lib7z_ffi.a

use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = PathBuf::from(&manifest_dir);

    // Pre-built static library lives in build/
    let build_dir = manifest_path.join("build");
    let lib_path = if cfg!(target_os = "windows") {
        build_dir.join("Release").join("7z_ffi.lib")
    } else {
        build_dir.join("lib7z_ffi.a")
    };

    if lib_path.exists() {
        let lib_dir = if cfg!(target_os = "windows") {
            build_dir.join("Release")
        } else {
            build_dir.clone()
        };

        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=7z_ffi");

        // Link pthread on Unix (needed for thread-safe error reporting in C library)
        #[cfg(target_os = "macos")]
        println!("cargo:rustc-link-lib=dylib=pthread");

        #[cfg(target_os = "linux")]
        println!("cargo:rustc-link-lib=dylib=pthread");

        #[cfg(target_os = "windows")]
        println!("cargo:rustc-link-lib=bcrypt");
    } else {
        println!(
            "cargo:warning=lib7z_ffi.a not found at: {}",
            lib_path.display()
        );
        println!("cargo:warning=Build the C library in the standalone sevenzip-ffi repo:");
        println!("cargo:warning=  cd ~/GitHub/sevenzip-ffi && rm -rf build && mkdir build && cd build");
        println!("cargo:warning=  cmake -DBUILD_SHARED_LIBS=OFF -DCMAKE_BUILD_TYPE=Release ..");
        println!("cargo:warning=  make -j$(sysctl -n hw.ncpu)");
        println!("cargo:warning=Then copy: cp build/lib7z_ffi.a ~/GitHub/CORE-1/sevenzip-ffi/build/");

        // Still emit the link directive so the error is clear
        println!("cargo:rustc-link-lib=static=7z_ffi");
    }

    // Re-run only if the pre-built library changes
    println!("cargo:rerun-if-changed=build/lib7z_ffi.a");
}
