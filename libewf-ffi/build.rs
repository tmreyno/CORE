// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
//
// libewf-ffi build script
// Links against system-installed libewf via pkg-config (Homebrew on macOS)
// Requires libewf 20251220 (modern libyal/libewf from source)

fn main() {
    // On Windows, check if libewf is installed; if not, compile a stub
    #[cfg(target_os = "windows")]
    {
        // Check for pre-built libewf on Windows
        let has_libewf = std::env::var("LIBEWF_DIR").is_ok();
        
        if has_libewf {
            let libewf_dir = std::env::var("LIBEWF_DIR").unwrap();
            println!("cargo:rustc-link-search=native={}", libewf_dir);
            println!("cargo:rustc-link-lib=ewf");
        } else {
            // Compile stub C file that provides all FFI symbols
            let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            let stub_path = std::path::PathBuf::from(&manifest_dir).join("src").join("stub.c");
            if stub_path.exists() {
                println!("cargo:warning=libewf not found on Windows - building stub (EWF write features will return errors at runtime)");
                cc::Build::new()
                    .file(&stub_path)
                    .warnings(false)
                    .compile("ewf");
            } else {
                println!("cargo:warning=libewf stub.c not found at: {}", stub_path.display());
                println!("cargo:rustc-link-lib=ewf");
            }
        }
        println!("cargo:rerun-if-changed=src/stub.c");
        println!("cargo:rerun-if-env-changed=LIBEWF_DIR");
        return;
    }

    // Try pkg-config first (works on macOS with Homebrew, Linux with apt)
    match pkg_config::Config::new()
        .atleast_version("20251220")
        .probe("libewf")
    {
        Ok(lib) => {
            println!("cargo:warning=Found libewf via pkg-config:");
            for path in &lib.include_paths {
                println!("cargo:warning=  include: {}", path.display());
            }
            for path in &lib.link_paths {
                println!("cargo:warning=  lib: {}", path.display());
            }
            // pkg-config handles rustc-link-search and rustc-link-lib
        }
        Err(e) => {
            println!("cargo:warning=pkg-config failed for libewf: {}", e);
            println!("cargo:warning=Falling back to manual search...");

            // Fallback: try common Homebrew paths on macOS
            let homebrew_paths = [
                "/opt/homebrew/lib",           // Apple Silicon
                "/usr/local/lib",              // Intel Mac
                "/opt/homebrew/Cellar/libewf", // Version-specific
            ];

            let mut found = false;
            for base in &homebrew_paths {
                let path = std::path::Path::new(base);
                if path.exists() {
                    // If it's the Cellar path, find the version directory
                    if base.contains("Cellar") {
                        if let Ok(entries) = std::fs::read_dir(path) {
                            for entry in entries.flatten() {
                                let lib_dir = entry.path().join("lib");
                                let include_dir = entry.path().join("include");
                                if lib_dir.join("libewf.a").exists()
                                    || lib_dir.join("libewf.dylib").exists()
                                {
                                    println!(
                                        "cargo:rustc-link-search=native={}",
                                        lib_dir.display()
                                    );
                                    println!("cargo:rustc-link-lib=ewf");
                                    println!("cargo:rustc-link-lib=z");
                                    if include_dir.exists() {
                                        println!(
                                            "cargo:include={}",
                                            include_dir.display()
                                        );
                                    }
                                    found = true;
                                    println!(
                                        "cargo:warning=Found libewf at: {}",
                                        lib_dir.display()
                                    );
                                    break;
                                }
                            }
                        }
                    } else if path.join("libewf.a").exists()
                        || path.join("libewf.dylib").exists()
                    {
                        println!("cargo:rustc-link-search=native={}", base);
                        println!("cargo:rustc-link-lib=ewf");
                        println!("cargo:rustc-link-lib=z");
                        found = true;
                        println!("cargo:warning=Found libewf at: {}", base);
                    }
                }
                if found {
                    break;
                }
            }

            if !found {
                println!("cargo:warning=libewf not found! Install with:");
                println!("cargo:warning=  macOS: brew install libewf");
                println!("cargo:warning=  Linux: sudo apt-get install libewf-dev");
                // Still emit the link directive — the linker will give a clear error
                println!("cargo:rustc-link-lib=ewf");
            }
        }
    }

    // Always need zlib on macOS (libewf depends on it)
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=z");
        // Modern libewf 20251220+ supports BZIP2 compression
        println!("cargo:rustc-link-lib=bz2");
    }

    println!("cargo:rerun-if-env-changed=LIBEWF_DIR");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
}
