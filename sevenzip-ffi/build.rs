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

    // Pre-built static library lives in prebuilt/<platform>/ (CI) or build/ (local macOS dev)
    let build_dir = manifest_path.join("build");

    // Platform-specific prebuilt directory (CI-built libs)
    let prebuilt_subdir = if is_windows_target {
        Some("windows-x64-msvc")
    } else if target.contains("linux") && !target.contains("android") {
        Some("linux-x64")
    } else if target.contains("apple") {
        Some("macos-arm64")
    } else {
        None
    };
    let prebuilt_lib = prebuilt_subdir.map(|sub| {
        if is_windows_target {
            manifest_path.join("prebuilt").join(sub).join("7z_ffi.lib")
        } else {
            manifest_path.join("prebuilt").join(sub).join("lib7z_ffi.a")
        }
    });

    // Local build/ directory — ONLY valid for the host platform (macOS dev build).
    // On CI, the build/ dir may contain macOS objects that can't link on Linux.
    let host = env::var("HOST").unwrap_or_default();
    let local_lib = if is_windows_target {
        build_dir.join("Release").join("7z_ffi.lib")
    } else {
        build_dir.join("lib7z_ffi.a")
    };
    // Only use local build/ lib when target matches host (prevents macOS .a on Linux)
    let local_lib_valid = local_lib.exists() && target == host;

    // Check prebuilt dir first, then local build/ (only if target == host)
    let effective_path = if let Some(ref pb) = prebuilt_lib {
        if pb.exists() { pb.clone() } else if local_lib_valid { local_lib.clone() } else { PathBuf::new() }
    } else if local_lib_valid {
        local_lib.clone()
    } else {
        PathBuf::new() // Will fall through to stub below since exists() will fail
    };

    if effective_path.exists() {
        let lib_dir = effective_path.parent().unwrap();

        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=7z_ffi");

        // Link platform-specific dependencies based on TARGET
        if is_windows_target {
            println!("cargo:rustc-link-lib=bcrypt");
        } else if target.contains("apple") || target.contains("linux") {
            println!("cargo:rustc-link-lib=dylib=pthread");
        }
    } else {
        println!(
            "cargo:warning=Pre-built library not found at: {}",
            effective_path.display()
        );

        // Compile a stub C file that provides all FFI symbols
        // so the project can link. Functions return error codes at runtime.
        let stub_path = manifest_path.join("src").join("stub.c");
        if stub_path.exists() {
            println!("cargo:warning=Building stub library (7z features will return errors at runtime)");
            cc::Build::new()
                .file(&stub_path)
                .warnings(false)
                .compile("7z_ffi");
            // cc::Build emits the correct rustc-link-lib and rustc-link-search
        } else {
            println!("cargo:warning=stub.c not found at: {}", stub_path.display());
            println!("cargo:rustc-link-lib=static=7z_ffi");
        }
    }

    // Re-run only if the pre-built library or stub changes
    println!("cargo:rerun-if-changed=build/lib7z_ffi.a");
    println!("cargo:rerun-if-changed=build/Release/7z_ffi.lib");
    println!("cargo:rerun-if-changed=prebuilt/linux-x64/lib7z_ffi.a");
    println!("cargo:rerun-if-changed=prebuilt/macos-arm64/lib7z_ffi.a");
    println!("cargo:rerun-if-changed=prebuilt/windows-x64-msvc/7z_ffi.lib");
    println!("cargo:rerun-if-changed=src/stub.c");
}
