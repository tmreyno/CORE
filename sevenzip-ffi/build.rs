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
    let target = env::var("TARGET").unwrap_or_default();
    let is_windows_target = target.contains("windows");

    // Pre-built static library lives in build/
    let build_dir = manifest_path.join("build");
    // Use TARGET env var (not cfg!) so cross-compilation picks the right file
    let lib_path = if is_windows_target {
        build_dir.join("Release").join("7z_ffi.lib")
    } else {
        build_dir.join("lib7z_ffi.a")
    };

    if lib_path.exists() {
        let lib_dir = if is_windows_target {
            build_dir.join("Release")
        } else {
            build_dir.clone()
        };

        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=7z_ffi");

        // Link platform-specific dependencies based on TARGET
        if is_windows_target {
            println!("cargo:rustc-link-lib=bcrypt");
        } else if target.contains("apple") {
            println!("cargo:rustc-link-lib=dylib=pthread");
        } else if target.contains("linux") {
            println!("cargo:rustc-link-lib=dylib=pthread");
        }
    } else {
        println!(
            "cargo:warning=Pre-built library not found at: {}",
            lib_path.display()
        );

        // Compile a stub C file that provides all FFI symbols
        // so the project can link. Functions return error codes at runtime.
        if is_windows_target {
            let stub_path = manifest_path.join("src").join("stub.c");
            if stub_path.exists() {
                println!("cargo:warning=Building stub library for Windows (7z features will return errors at runtime)");
                cc::Build::new()
                    .file(&stub_path)
                    .warnings(false)
                    .compile("7z_ffi");
                // cc::Build emits the correct rustc-link-lib and rustc-link-search
            } else {
                println!("cargo:warning=stub.c not found at: {}", stub_path.display());
                println!("cargo:rustc-link-lib=static=7z_ffi");
            }
        } else {
            println!("cargo:warning=Build the C library in the standalone sevenzip-ffi repo:");
            println!("cargo:warning=  cd ~/GitHub/sevenzip-ffi && rm -rf build && mkdir build && cd build");
            println!("cargo:warning=  cmake -DBUILD_SHARED_LIBS=OFF -DCMAKE_BUILD_TYPE=Release ..");
            println!("cargo:warning=  make -j$(sysctl -n hw.ncpu)");
            println!("cargo:warning=Then copy: cp build/lib7z_ffi.a ~/GitHub/CORE-1/sevenzip-ffi/build/");
            // Still emit the link directive so the error is clear
            println!("cargo:rustc-link-lib=static=7z_ffi");
        }
    }

    // Re-run only if the pre-built library or stub changes
    println!("cargo:rerun-if-changed=build/lib7z_ffi.a");
    println!("cargo:rerun-if-changed=build/Release/7z_ffi.lib");
    println!("cargo:rerun-if-changed=src/stub.c");
}
