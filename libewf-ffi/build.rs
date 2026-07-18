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
//   2. Pre-built libraries in repo → static libewf
//   3. pkg-config → system-installed libewf (Homebrew, apt, etc.)
//   4. Common library paths (/opt/homebrew/lib, /usr/local/lib)
//   5. Stub fallback → compiles stub.c providing all FFI symbols
//      (EWF C-library features return errors at runtime; pure-Rust
//       EWF reader and L01 writer are unaffected)

fn main() -> Result<(), String> {
    println!("cargo:rerun-if-changed=src/stub.c");
    println!("cargo:rerun-if-changed=prebuilt/linux-x64/libewf.a");
    println!("cargo:rerun-if-changed=prebuilt/macos-arm64/libewf.a");
    println!("cargo:rerun-if-changed=prebuilt/windows-x64-msvc/ewf.lib");
    println!("cargo:rerun-if-env-changed=LIBEWF_DIR");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");

    let target = std::env::var("TARGET").unwrap_or_default();
    let host = std::env::var("HOST").unwrap_or_default();
    let is_cross = target != host;
    let is_windows_target = target.contains("windows");
    let is_macos_target = target.contains("apple");
    let _is_linux_target = target.contains("linux") && !target.contains("android");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|e| format!("CARGO_MANIFEST_DIR is not set for libewf-ffi build: {e}"))?;
    let manifest_path = std::path::PathBuf::from(&manifest_dir);

    // --- Step 1: Explicit LIBEWF_DIR ---
    if let Ok(libewf_dir) = std::env::var("LIBEWF_DIR") {
        let libewf_path = std::path::Path::new(&libewf_dir);
        println!("cargo:warning=Using LIBEWF_DIR={}", libewf_dir);
        println!("cargo:rustc-link-search=native={}", libewf_dir);
        if is_windows_target || libewf_path.join("libewf.a").exists() {
            println!("cargo:rustc-link-lib=static=ewf");
        } else {
            println!("cargo:rustc-link-lib=ewf");
        }
        link_system_deps_for_target(&target);
        return Ok(());
    }

    // --- Step 2: Pre-built libraries in repo (CI + release/local packaging) ---
    {
        // Determine platform-specific prebuilt directory and library name
        let (prebuilt_subdir, lib_name) = if is_windows_target && target.contains("msvc") {
            ("windows-x64-msvc", "ewf.lib")
        } else if is_macos_target && target.contains("aarch64") {
            ("macos-arm64", "libewf.a")
        } else if _is_linux_target && target.contains("x86_64") {
            ("linux-x64", "libewf.a")
        } else {
            ("", "")
        };

        if !prebuilt_subdir.is_empty() {
            let prebuilt_dir = manifest_path.join("prebuilt").join(prebuilt_subdir);
            if prebuilt_dir.join(lib_name).exists() {
                println!(
                    "cargo:warning=Found pre-built libewf at: {}",
                    prebuilt_dir.display()
                );
                println!("cargo:rustc-link-search=native={}", prebuilt_dir.display());
                println!("cargo:rustc-link-lib=static=ewf");
                link_system_deps_prebuilt(&prebuilt_dir, &target);
                return Ok(());
            }
        }
    }

    // --- Step 3: pkg-config (only when not cross-compiling) ---
    // When cross-compiling, pkg-config would find host libraries, not target ones.
    if !is_cross && pkg_config::Config::new().probe("libewf").is_ok() {
        println!("cargo:warning=Found libewf via pkg-config");
        link_system_deps_for_target(&target);
        return Ok(());
    }

    // --- Step 4: Common library paths (only for native builds) ---
    if !is_cross {
        let search_paths: &[&str] = if is_macos_target {
            &[
                "/opt/homebrew/lib", // Apple Silicon Homebrew
                "/usr/local/lib",    // Intel Homebrew / manual install
            ]
        } else {
            &["/usr/lib", "/usr/local/lib", "/usr/lib/x86_64-linux-gnu"]
        };

        for dir in search_paths {
            let p = std::path::Path::new(dir);
            let has_lib = if is_macos_target {
                p.join("libewf.dylib").exists() || p.join("libewf.a").exists()
            } else if is_windows_target {
                p.join("ewf.lib").exists() || p.join("ewf.dll").exists()
            } else {
                p.join("libewf.so").exists() || p.join("libewf.a").exists()
            };
            if has_lib {
                println!("cargo:warning=Found libewf at: {}", dir);
                println!("cargo:rustc-link-search=native={}", dir);
                println!("cargo:rustc-link-lib=ewf");
                link_system_deps_for_target(&target);
                return Ok(());
            }
        }
    }

    // --- Step 5: Stub fallback (all platforms) ---
    let stub_path = manifest_path.join("src").join("stub.c");

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
        return Err(format!(
            "libewf was not found and fallback stub.c is missing at {}",
            stub_path.display()
        ));
    }

    Ok(())
}

/// Link transitive system dependencies based on TARGET (not host).
fn link_system_deps_for_target(target: &str) {
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=z");
        println!("cargo:rustc-link-lib=bz2");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=z");
    } else if target.contains("windows") {
        println!("cargo:rustc-link-lib=zlib");
        println!("cargo:rustc-link-lib=bz2");
        println!("cargo:rustc-link-lib=ws2_32");
        println!("cargo:rustc-link-lib=advapi32");
    }
}

/// Link deps from the prebuilt directory.
fn link_system_deps_prebuilt(prebuilt_dir: &std::path::Path, target: &str) {
    if target.contains("windows") {
        if prebuilt_dir.join("zlib.lib").exists() {
            println!("cargo:rustc-link-lib=zlib");
        }
        if prebuilt_dir.join("bz2.lib").exists() {
            println!("cargo:rustc-link-lib=bz2");
        }
        println!("cargo:rustc-link-lib=ws2_32");
        println!("cargo:rustc-link-lib=advapi32");
    } else if target.contains("apple") {
        // macOS prebuilt — link system zlib + bzip2
        println!("cargo:rustc-link-lib=z");
        println!("cargo:rustc-link-lib=bz2");
    } else if target.contains("linux") {
        // Linux prebuilt — link system zlib
        println!("cargo:rustc-link-lib=z");
    }
}
