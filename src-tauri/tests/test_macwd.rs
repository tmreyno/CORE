use ffx_check_lib::ewf::vfs::EwfVfs;
use ffx_check_lib::common::vfs::VirtualFileSystem;

fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();
}

#[test]
fn test_macwd_e01_contents() {
    init_logging();
    
    let path = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/macwd/macwd.E01";
    
    if !std::path::Path::new(path).exists() {
        println!("Skipping test: macwd.E01 not found");
        return;
    }
    
    println!("\n=== Testing macwd.E01 VFS Contents ===");
    
    let vfs = EwfVfs::open_filesystem(path).expect("VFS open failed");
    println!("VFS opened with {} partitions", vfs.partition_count());
    
    // List root
    let root_entries = vfs.readdir("/").expect("Failed to read root");
    println!("\nRoot entries ({}):", root_entries.len());
    for entry in &root_entries {
        println!("  /{} (dir: {})", entry.name, entry.is_directory);
    }
    
    // Explore each partition
    for entry in &root_entries {
        let partition_path = format!("/{}", entry.name);
        println!("\n--- {} ---", partition_path);
        
        match vfs.readdir(&partition_path) {
            Ok(entries) => {
                println!("  Contains {} items:", entries.len());
                for item in entries.iter().take(25) {
                    let type_str = if item.is_directory { "<DIR>" } else { "<FILE>" };
                    println!("    {} {}", type_str, item.name);
                }
                if entries.len() > 25 {
                    println!("    ... and {} more", entries.len() - 25);
                }
            }
            Err(e) => {
                println!("  Error reading: {:?}", e);
            }
        }
    }
}
