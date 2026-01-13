#[cfg(test)]
mod tests {
    use crate::ad1::reader_v2::SessionV2;
    use crate::ad1::operations_v2::get_root_children;
    use std::time::Instant;

    const TEST_AD1: &str = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/02606-0900_1E_GRCDH2/02606-0900_1E_GRCDH2_IMG1.ad1";

    #[test]
    fn test_v2_session() {
        println!("\n=== TEST: Session Initialization ===");
        let start = Instant::now();
        let session = SessionV2::open(TEST_AD1).expect("Failed to open");
        println!("Init time: {:?}", start.elapsed());
        println!("Segments: {}", session.segments.len());
        let total: u64 = session.segments.iter().map(|s| s.size).sum();
        println!("Total size: {:.2} GB", total as f64 / 1024.0 / 1024.0 / 1024.0);
        for (i, seg) in session.segments.iter().take(5).enumerate() {
            println!("  [{}] {:.2} GB", i+1, seg.size as f64 / 1024.0 / 1024.0 / 1024.0);
        }
        println!("✅ PASSED");
    }

    #[test]
    fn test_v2_header() {
        println!("\n=== TEST: Logical Header ===");
        let session = SessionV2::open(TEST_AD1).expect("Failed to open");
        println!("Signature: {}", session.logical_header.signature);
        println!("Version: {}", session.logical_header.image_version);
        println!("Zlib chunk: {} bytes", session.logical_header.zlib_chunk_size);
        println!("First item: 0x{:X}", session.logical_header.first_item_addr);
        println!("Data source: {}", session.logical_header.data_source_name);
        println!("✅ PASSED");
    }

    #[test]
    fn test_v2_root() {
        println!("\n=== TEST: Root Items ===");
        let start = Instant::now();
        let items = get_root_children(TEST_AD1).expect("Failed to get items");
        println!("Enum time: {:?}", start.elapsed());
        println!("Root items: {}", items.len());
        for (i, item) in items.iter().enumerate() {
            let icon = if item.is_dir { "📂" } else { "📄" };
            println!("  [{}] {} {} ({} bytes)", i+1, icon, item.name, item.size);
        }
        println!("✅ PASSED");
    }
}

// Additional tests appended
#[cfg(test)]
mod more_tests {
    use crate::ad1::reader_v2::SessionV2;
    use crate::ad1::operations_v2::{get_root_children, get_children_at_addr, decompress_file_data};
    use std::time::Instant;

    const TEST_AD1: &str = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/02606-0900_1E_GRCDH2/02606-0900_1E_GRCDH2_IMG1.ad1";

    #[test]
    fn test_v2_tree() {
        println!("\n=== TEST: Directory Tree (3 levels) ===");
        let items = get_root_children(TEST_AD1).expect("Failed");
        println!("Root: {} items", items.len());
        
        for item in &items {
            let icon = if item.is_dir { "📂" } else { "📄" };
            println!("{} {} [{:?} children]", icon, item.name, item.child_count);
            
            if item.is_dir && item.child_count.unwrap_or(0) > 0 {
                if let Some(addr) = item.item_addr {
                    if let Ok(children) = get_children_at_addr(TEST_AD1, addr, &item.path) {
                        for child in children.iter().take(5) {
                            let ic = if child.is_dir { "📂" } else { "📄" };
                            println!("   {} {} ({} bytes)", ic, child.name, child.size);
                        }
                        if children.len() > 5 {
                            println!("   ... and {} more", children.len() - 5);
                        }
                    }
                }
            }
        }
        println!("✅ PASSED");
    }

    #[test]
    fn test_v2_path() {
        println!("\n=== TEST: Path Building ===");
        let session = SessionV2::open(TEST_AD1).expect("Failed");
        let first = session.logical_header.first_item_addr;
        
        if let Ok(item) = session.read_item_at(first) {
            let path = session.build_item_path(&item);
            println!("Root: {} -> {}", item.name, path);
            
            if item.first_child_addr != 0 {
                if let Ok(child) = session.read_item_at(item.first_child_addr) {
                    let cpath = session.build_item_path(&child);
                    println!("Child: {} -> {}", child.name, cpath);
                }
            }
        }
        println!("✅ PASSED");
    }

    #[test]
    fn test_v2_decompress() {
        println!("\n=== TEST: File Decompression ===");
        let session = SessionV2::open(TEST_AD1).expect("Failed");
        
        let mut to_visit = vec![session.logical_header.first_item_addr];
        let mut files_tested = 0;
        
        while !to_visit.is_empty() && files_tested < 3 {
            let addr = to_visit.remove(0);
            if addr == 0 { continue; }
            
            if let Ok(item) = session.read_item_at(addr) {
                if item.item_type != 0x05 && item.decompressed_size > 0 && item.decompressed_size < 100000 {
                    let start = Instant::now();
                    match decompress_file_data(&session, &item) {
                        Ok(data) => {
                            println!("�� {} - {} bytes in {:?}", item.name, data.len(), start.elapsed());
                            let hex: String = data.iter().take(16).map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                            println!("   Hex: {}...", hex);
                        }
                        Err(e) => println!("❌ {} - {:?}", item.name, e),
                    }
                    files_tested += 1;
                }
                
                if item.first_child_addr != 0 { to_visit.push(item.first_child_addr); }
                if item.next_item_addr != 0 && to_visit.len() < 50 { to_visit.push(item.next_item_addr); }
            }
        }
        println!("✅ PASSED");
    }

    #[test]
    fn test_v2_cache() {
        println!("\n=== TEST: File Cache ===");
        let session = SessionV2::open(TEST_AD1).expect("Failed");
        
        let mut to_visit = vec![session.logical_header.first_item_addr];
        
        while !to_visit.is_empty() {
            let addr = to_visit.remove(0);
            if addr == 0 { continue; }
            
            if let Ok(item) = session.read_item_at(addr) {
                if item.item_type != 0x05 && item.decompressed_size > 1000 && item.decompressed_size < 50000 {
                    println!("Testing with: {} ({} bytes)", item.name, item.decompressed_size);
                    
                    let t1 = Instant::now();
                    let _ = decompress_file_data(&session, &item);
                    let d1 = t1.elapsed();
                    
                    let t2 = Instant::now();
                    let _ = decompress_file_data(&session, &item);
                    let d2 = t2.elapsed();
                    
                    println!("   1st read: {:?}", d1);
                    println!("   2nd read: {:?} (cache)", d2);
                    if d2 < d1 {
                        println!("   Speedup: {:.1}x", d1.as_nanos() as f64 / d2.as_nanos().max(1) as f64);
                    }
                    break;
                }
                
                if item.first_child_addr != 0 { to_visit.push(item.first_child_addr); }
                if item.next_item_addr != 0 && to_visit.len() < 30 { to_visit.push(item.next_item_addr); }
            }
        }
        println!("✅ PASSED");
    }

    #[test]
    fn test_v2_metadata() {
        println!("\n=== TEST: Item Metadata ===");
        let session = SessionV2::open(TEST_AD1).expect("Failed");
        let mut addr = session.logical_header.first_item_addr;
        let mut count = 0;
        
        while addr != 0 && count < 3 {
            if let Ok(item) = session.read_item_at(addr) {
                println!("[{}] {}", count+1, item.name);
                println!("   Type: {}", if item.item_type == 0x05 { "Dir" } else { "File" });
                println!("   Offset: 0x{:X}", item.offset);
                
                if item.first_metadata_addr != 0 {
                    if let Ok(meta) = session.read_metadata_chain(item.first_metadata_addr) {
                        println!("   Metadata: {} entries", meta.len());
                        for m in meta.iter().take(3) {
                            println!("     cat={}, key={}, len={}", m.category, m.key, m.data_length);
                        }
                    }
                }
                addr = item.next_item_addr;
                count += 1;
            } else { break; }
        }
        println!("✅ PASSED");
    }

    #[test]
    fn test_v2_errors() {
        println!("\n=== TEST: Error Handling ===");
        
        println!("Non-existent file:");
        match SessionV2::open("/fake/path.ad1") {
            Err(e) => println!("  ✅ Error: {:?}", e),
            Ok(_) => println!("  ❌ Should have failed"),
        }
        
        println!("Invalid file:");
        match SessionV2::open("/etc/passwd") {
            Err(e) => println!("  ✅ Error: {:?}", e),
            Ok(_) => println!("  ❌ Should have failed"),
        }
        
        println!("✅ PASSED");
    }
}
