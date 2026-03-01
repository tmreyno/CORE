// =============================================================================
// CORE-FFX - Forensic File Explorer
// Filesystem Integration Test with Real Disk Images
// =============================================================================

//! Test the filesystem module against real disk images.
//!
//! Usage:
//!   cargo run --example test_filesystem -- /path/to/disk.E01

use ffx_check_lib::common::format_size_compact;
use ffx_check_lib::common::vfs::VirtualFileSystem;
use ffx_check_lib::ewf::vfs::EwfVfs;

fn explore_directory(vfs: &EwfVfs, path: &str, depth: usize, max_depth: usize) {
    if depth > max_depth {
        return;
    }

    let indent = "  ".repeat(depth);

    match vfs.readdir(path) {
        Ok(entries) => {
            for entry in entries.iter().take(15) {
                let full_path = if path == "/" {
                    format!("/{}", entry.name)
                } else {
                    format!("{}/{}", path, entry.name)
                };

                let attr = vfs.getattr(&full_path).ok();
                let size = attr.as_ref().map(|a| a.size).unwrap_or(0);
                let is_dir = entry.is_directory;

                if is_dir {
                    println!("{}📁 {}/", indent, entry.name);
                    explore_directory(vfs, &full_path, depth + 1, max_depth);
                } else {
                    println!(
                        "{}📄 {} ({})",
                        indent,
                        entry.name,
                        format_size_compact(size)
                    );
                }
            }

            if entries.len() > 15 {
                println!("{}... and {} more entries", indent, entries.len() - 15);
            }
        }
        Err(e) => {
            println!("{}[Error: {:?}]", indent, e);
        }
    }
}

fn main() {
    // Get path from command line or use default
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() > 1 {
        &args[1]
    } else {
        "/Users/terryreynolds/Downloads/2020JimmyWilson.E01"
    };

    println!("=== Filesystem Integration Test ===\n");
    println!("Testing image: {}\n", path);

    // Test 1: Open in Physical mode
    println!("--- Test 1: Physical Mode ---");
    match EwfVfs::open_physical(path) {
        Ok(vfs) => {
            println!("✓ Opened in physical mode");

            // List root
            match vfs.readdir("/") {
                Ok(entries) => {
                    println!("  Root entries:");
                    for entry in &entries {
                        let attr = vfs.getattr(&format!("/{}", entry.name)).ok();
                        let size = attr.map(|a| a.size).unwrap_or(0);
                        println!("    {} ({} bytes)", entry.name, size);
                    }
                }
                Err(e) => println!("  Error listing root: {:?}", e),
            }
        }
        Err(e) => println!("✗ Failed to open: {:?}", e),
    }

    println!();

    // Test 2: Open in Filesystem mode
    println!("--- Test 2: Filesystem Mode ---");
    match EwfVfs::open_filesystem(path) {
        Ok(vfs) => {
            println!("✓ Opened in filesystem mode");
            println!("  Partition count: {}", vfs.partition_count());

            // List root (partitions)
            match vfs.readdir("/") {
                Ok(entries) => {
                    println!("  Mounted partitions:");
                    for entry in &entries {
                        println!("    {} (directory: {})", entry.name, entry.is_directory);

                        // Try to list files in this partition
                        let partition_path = format!("/{}", entry.name);
                        match vfs.readdir(&partition_path) {
                            Ok(files) => {
                                println!("      Files ({} entries):", files.len());
                                for file in files.iter().take(10) {
                                    let file_path = format!("{}/{}", partition_path, file.name);
                                    let attr = vfs.getattr(&file_path).ok();
                                    let size = attr.as_ref().map(|a| a.size).unwrap_or(0);
                                    let is_dir =
                                        attr.as_ref().map(|a| a.is_directory).unwrap_or(false);
                                    let type_indicator = if is_dir { "📁" } else { "📄" };
                                    println!(
                                        "        {} {} ({} bytes)",
                                        type_indicator, file.name, size
                                    );
                                }
                                if files.len() > 10 {
                                    println!("        ... and {} more entries", files.len() - 10);
                                }
                            }
                            Err(e) => println!("      Error listing partition: {:?}", e),
                        }
                    }
                }
                Err(e) => println!("  No partitions found or error: {:?}", e),
            }
        }
        Err(e) => println!("✗ Failed to open in filesystem mode: {:?}", e),
    }

    println!();

    // Test 3: Deep Directory Exploration
    println!("--- Test 3: Directory Tree (depth 2) ---");
    match EwfVfs::open(path) {
        Ok(vfs) => {
            let mode = if vfs.partition_count() > 0 {
                "Filesystem"
            } else {
                "Physical"
            };
            println!("✓ Opened in {} mode (auto-detected)\n", mode);

            explore_directory(&vfs, "/", 0, 2);
        }
        Err(e) => println!("✗ Failed: {:?}", e),
    }

    println!();

    // Test 4: Read file contents
    println!("--- Test 4: File Content Reading ---");
    match EwfVfs::open(path) {
        Ok(vfs) => {
            // Try to find and read a text file
            let test_files = [
                "/Partition1_NTFS/AUTOEXEC.BAT",
                "/Partition1_NTFS/boot.ini",
                "/Partition1_NTFS/CONFIG.SYS",
                "/Partition2_NTFS/boot.ini",
                "/Partition1_FAT32/EFI/Microsoft/Boot/bootmgfw.efi",
            ];

            for file_path in test_files {
                println!("\nTrying to read: {}", file_path);
                match vfs.getattr(file_path) {
                    Ok(attr) => {
                        println!("  File exists, size: {}", format_size_compact(attr.size));
                        // Read first 256 bytes
                        match vfs.read(file_path, 0, 256) {
                            Ok(data) => {
                                println!("  Read {} bytes", data.len());
                                // Show as hex + ASCII preview
                                print!("  Hex: ");
                                for (i, b) in data.iter().take(32).enumerate() {
                                    if i > 0 && i % 16 == 0 {
                                        print!("\n       ");
                                    }
                                    print!("{:02x} ", b);
                                }
                                println!();

                                // Try to show as text if printable
                                let text: String = data
                                    .iter()
                                    .take(64)
                                    .map(|&b| {
                                        if (32..127).contains(&b) {
                                            b as char
                                        } else {
                                            '.'
                                        }
                                    })
                                    .collect();
                                println!("  ASCII: {}", text);
                            }
                            Err(e) => println!("  Read error: {:?}", e),
                        }
                    }
                    Err(_) => {
                        // File doesn't exist, skip silently
                    }
                }
            }
        }
        Err(e) => println!("✗ Failed: {:?}", e),
    }

    println!("\n=== Test Complete ===");
}
