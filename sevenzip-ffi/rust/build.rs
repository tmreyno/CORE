use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Get the manifest directory (rust/)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = PathBuf::from(&manifest_dir);
    
    // Project root is parent of rust/
    let project_root = manifest_path.parent().unwrap();
    
    // Check if C library is already built
    let build_dir = project_root.join("build");
    let lib_path = if cfg!(target_os = "windows") {
        build_dir.join("Release").join("7z_ffi.lib")
    } else {
        // On Unix-like systems, look for static library
        build_dir.join("lib7z_ffi.a")
    };
    
    // Build C library if it doesn't exist
    if !lib_path.exists() {
        println!("cargo:warning=Building C library...");
        
        // Run cmake configuration
        let cmake_status = Command::new("cmake")
            .args(&["-B", "build", "-DCMAKE_BUILD_TYPE=Release"])
            .current_dir(project_root)
            .status();
        
        match cmake_status {
            Ok(status) if status.success() => {
                println!("cargo:warning=CMake configuration successful");
            }
            Ok(status) => {
                println!("cargo:warning=CMake configuration failed with status: {}", status);
                println!("cargo:warning=Please build the C library manually:");
                println!("cargo:warning=  cd .. && cmake -B build && cmake --build build");
            }
            Err(e) => {
                println!("cargo:warning=CMake not found: {}", e);
                println!("cargo:warning=Please install CMake and build the C library manually:");
                println!("cargo:warning=  cd .. && cmake -B build && cmake --build build");
            }
        }
        
        // Run cmake build
        if build_dir.exists() {
            let build_status = Command::new("cmake")
                .args(&["--build", "build", "--config", "Release"])
                .current_dir(project_root)
                .status();
            
            match build_status {
                Ok(status) if status.success() => {
                    println!("cargo:warning=C library build successful");
                }
                Ok(status) => {
                    println!("cargo:warning=C library build failed with status: {}", status);
                }
                Err(e) => {
                    println!("cargo:warning=Build failed: {}", e);
                }
            }
        }
    } else {
        println!("cargo:warning=C library already built at: {}", lib_path.display());
    }
    
    // Tell cargo where to find the library
    let lib_dir = if cfg!(target_os = "windows") {
        build_dir.join("Release")
    } else {
        build_dir.clone()
    };
    
    if lib_dir.exists() && lib_path.exists() {
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=7z_ffi");
        
        // Link system libraries (no OpenSSL needed - using pure Rust crypto)
        #[cfg(not(target_os = "windows"))]
        {
            // Link pthread for thread-safe error reporting
            #[cfg(target_os = "macos")]
            println!("cargo:rustc-link-lib=dylib=pthread");
            
            #[cfg(target_os = "linux")]
            println!("cargo:rustc-link-lib=dylib=pthread");
        }
        
        // On Windows, link against bcrypt for system crypto (if C library needs it)
        #[cfg(target_os = "windows")]
        {
            println!("cargo:rustc-link-lib=bcrypt");
        }
    } else {
        println!("cargo:warning=Library directory not found: {}", lib_dir.display());
        println!("cargo:warning=Please build the C library manually:");
        println!("cargo:warning=  cd .. && cmake -B build && cmake --build build");
        
        // Still try to link, might be in a custom location
        println!("cargo:rustc-link-lib=static=7z_ffi");
    }
    
    // Re-run if C sources change
    println!("cargo:rerun-if-changed=../src/");
    println!("cargo:rerun-if-changed=../include/");
    println!("cargo:rerun-if-changed=../CMakeLists.txt");
    println!("cargo:rerun-if-changed=../build/");
    
    // Check for required system dependencies
    check_system_dependencies();
}

fn check_system_dependencies() {
    // Check for CMake
    let cmake_check = Command::new("cmake")
        .arg("--version")
        .output();
    
    match cmake_check {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            let version_line = version.lines().next().unwrap_or("unknown");
            println!("cargo:warning=CMake found: {}", version_line);
        }
        _ => {
            println!("cargo:warning=CMake not found. Please install CMake 3.15 or later");
            println!("cargo:warning=On macOS: brew install cmake");
            println!("cargo:warning=On Linux: sudo apt-get install cmake");
        }
    }
    
    // Note: OpenSSL is no longer required - using pure Rust crypto (RustCrypto crates)
    println!("cargo:warning=Using pure Rust AES encryption (no OpenSSL required)");
}
