// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
//
// libewf-ffi build script
//
// Discovery order:
//   1. LIBEWF_DIR env var → link directly
//   2. pkg-config → system-installed libewf (Homebrew, apt, etc.)
//   3. Common library paths (/opt/homebrew/lib, /usr/local/lib)
//   4. Stub fallback → compiles stub.c providing all FFI symbols
//      (EWF C-library features return errors at runtime; pure-Rust
//       EWF reader and L01 writer are unaffected)

fn main() {
    println!("cargo:rerun-if-changed=src/stub.c");
    println!("cargo:rerun-if-env-changed=LIBEWF_DIR");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");

    // --- Step 1: Explicit LIBEWF_DIR ---
    if let Ok(libewf_dir) = std::env::var("LIBEWF_DIR") {
        println!("cargo:warning=Using LIBEWF_DIR={}", libewf_dir);
        println!("cargo:rustc-link-search=native={}", libewf_dir);
        println!("cargo:rustc-link-lib=ewf");
        link_system_deps();
        return;
    }

    // --- Step 2: pkg-config ---
    if pkg_config::Config::new().probe("libewf").is_ok() {
        println!("cargo:warning=Found libewf via pkg-config");
        link_system_deps();
        return;
    }

    // --- Step 3a: Pre-built libraries in repo (Windows CI) ---
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let prebuilt_dir = std::path::PathBuf::from(&manifest_dir)
            .join("prebuilt")
            .join("windows-x64-msvc");
        if prebuilt_dir.join("ewf.lib").exists() {
            println!(
                "cargo:warning=Found pre-built libewf at: {}",
                prebuilt_dir.display()
            );
            println!(
                "cargo:rustc-link-search=native={}",
                prebuilt_dir.display()
            );
            println!("cargo:rustc-link-lib=ewf");
            link_system_deps_prebuilt(&prebuilt_dir);
            return;
        }
    }

    // --- Step 3b: Common library paths ---
    let search_paths: &[&str] = if cfg!(target_os = "macos") {
        &[
            "/opt/homebrew/lib",  // Apple Silicon Homebrew
            "/usr/local/lib",     // Intel Homebrew / manual install
        ]
    } else {
        &[
            "/usr/lib",
            "/usr/local/lib",
            "/usr/lib/x86_64-linux-gnu",
        ]
    };

    for dir in search_paths {
        let p = std::path::Path::new(dir);
        let has_lib = if cfg!(target_os = "macos") {
            p.join("libewf.dylib").exists() || p.join("libewf.a").exists()
        } else if cfg!(target_os = "windows") {
            p.join("ewf.lib").exists() || p.join("ewf.dll").exists()
        } else {
            p.join("libewf.so").exists() || p.join("libewf.a").exists()
        };
        if has_lib {
            println!("cargo:warning=Found libewf at: {}", dir);
            println!("cargo:rustc-link-search=native={}", dir);
            println!("cargo:rustc-link-lib=ewf");
            link_system_deps();
            return;
        }
    }

    // --- Step 4: Stub fallback (all platforms) ---
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let stub_path = std::path::PathBuf::from(&manifest_dir)
        .join("src")
        .join("stub.c");

    if stub_path.exists() {
        println!(
            "cargo:warning=libewf not found — building stub \
             (EWF C-library features disabled; pure-Rust EWF reader \
             and L01 writer still work)"
        );
        cc::Build::new()
            .file(&stub_path)
            .warnings(false)
            .compile("ewf");
    } else {
        println!("cargo:warning=libewf not found and stub.c missing!");
        println!("cargo:rustc-link-lib=ewf");
    }
}

/// Link transitive system dependencies required by the real libewf.
/// Only called when a real libewf library was found (not for stubs).
fn link_system_deps() {
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=z");
        println!("cargo:rustc-link-lib=bz2");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=z");
    } else if cfg!(target_os = "windows") {
        // When using LIBEWF_DIR (e.g. from CI), zlib/bz2 must already
        // be on the system link path (vcpkg or manual).
        println!("cargo:rustc-link-lib=zlib");
        println!("cargo:rustc-link-lib=bz2");
        println!("cargo:rustc-link-lib=ws2_32");
        println!("cargo:rustc-link-lib=advapi32");
    }
}

/// Link deps from the prebuilt directory (Windows CI).
/// The prebuilt dir contains ewf.lib, zlib.lib, and bz2.lib together.
fn link_system_deps_prebuilt(prebuilt_dir: &std::path::Path) {
    // zlib and bz2 are co-located in the same prebuilt directory
    if prebuilt_dir.join("zlib.lib").exists() {
        println!("cargo:rustc-link-lib=zlib");
    }
    if prebuilt_dir.join("bz2.lib").exists() {
        println!("cargo:rustc-link-lib=bz2");
    }
    // Windows system libraries needed by libewf
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=ws2_32");
        println!("cargo:rustc-link-lib=advapi32");
    }
}
