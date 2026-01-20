use ffx_check_lib::raw::vfs::RawVfs;
use ffx_check_lib::common::vfs::VirtualFileSystem;

fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();
}

/// Find raw disk images recursively
#[allow(dead_code)]
fn find_raw_images(base: &std::path::Path, images: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_raw_images(&path, images);
            } else {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if matches!(ext.to_lowercase().as_str(), "dd" | "raw" | "001") {
                    images.push(path);
                }
            }
        }
    }
}

#[test]
fn test_raw_rhinousb() {
    init_logging();
    
    let path = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/DFRWS2005-RODEO/RHINOUSB.dd";
    
    if !std::path::Path::new(path).exists() {
        println!("Skipping: RHINOUSB.dd not found");
        return;
    }
    
    println!("\n=== Testing RHINOUSB.dd ===");
    
    match RawVfs::open_filesystem(path) {
        Ok(vfs) => {
            println!("VFS opened! Partitions: {}", vfs.partition_count());
            
            match vfs.readdir("/") {
                Ok(entries) => {
                    println!("Root entries: {}", entries.len());
                    for entry in &entries {
                        let type_str = if entry.is_directory { "<DIR>" } else { "<FILE>" };
                        println!("  {} {}", type_str, entry.name);
                        
                        if entry.is_directory {
                            let part_path = format!("/{}", entry.name);
                            if let Ok(part_entries) = vfs.readdir(&part_path) {
                                println!("    ({} items)", part_entries.len());
                                for pe in part_entries.iter().take(15) {
                                    let pt = if pe.is_directory { "DIR" } else { "FILE" };
                                    println!("      [{}] {}", pt, pe.name);
                                }
                            }
                        }
                    }
                }
                Err(e) => println!("Failed to read root: {:?}", e),
            }
        }
        Err(e) => {
            println!("Filesystem mode failed: {:?}", e);
            
            // Fallback to physical mode
            match RawVfs::open(path) {
                Ok(vfs) => {
                    println!("Physical mode opened");
                    if let Ok(entries) = vfs.readdir("/") {
                        println!("Root: {} entries", entries.len());
                        for e in &entries {
                            if let Ok(attr) = vfs.getattr(&format!("/{}", e.name)) {
                                println!("  {} ({} bytes)", e.name, attr.size);
                            }
                        }
                    }
                }
                Err(e) => println!("Physical mode failed: {:?}", e),
            }
        }
    }
}

#[test]
fn test_raw_schardt() {
    init_logging();
    
    let path = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/SCHARDT/SCHARDT.001";
    
    if !std::path::Path::new(path).exists() {
        println!("Skipping: SCHARDT.001 not found");
        return;
    }
    
    println!("\n=== Testing SCHARDT.001 ===");
    
    match RawVfs::open_filesystem(path) {
        Ok(vfs) => {
            println!("VFS opened! Partitions: {}", vfs.partition_count());
            
            match vfs.readdir("/") {
                Ok(entries) => {
                    println!("Root entries: {}", entries.len());
                    for entry in &entries {
                        let type_str = if entry.is_directory { "<DIR>" } else { "<FILE>" };
                        println!("  {} {}", type_str, entry.name);
                        
                        if entry.is_directory {
                            let part_path = format!("/{}", entry.name);
                            if let Ok(part_entries) = vfs.readdir(&part_path) {
                                println!("    ({} items)", part_entries.len());
                                for pe in part_entries.iter().take(15) {
                                    let pt = if pe.is_directory { "DIR" } else { "FILE" };
                                    println!("      [{}] {}", pt, pe.name);
                                }
                            }
                        }
                    }
                }
                Err(e) => println!("Failed to read root: {:?}", e),
            }
        }
        Err(e) => {
            println!("Filesystem mode failed: {:?}", e);
            
            // Fallback to physical mode
            match RawVfs::open(path) {
                Ok(vfs) => {
                    println!("Physical mode opened");
                    if let Ok(entries) = vfs.readdir("/") {
                        println!("Root: {} entries", entries.len());
                        for e in &entries {
                            if let Ok(attr) = vfs.getattr(&format!("/{}", e.name)) {
                                println!("  {} ({} bytes)", e.name, attr.size);
                            }
                        }
                    }
                }
                Err(e) => println!("Physical mode failed: {:?}", e),
            }
        }
    }
}
