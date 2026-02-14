#[cfg(test)]
mod enumerate_tests {
    use crate::ad1::reader_v2::SessionV2;
    use crate::common::hex::format_size_compact;
    use std::time::Instant;
    use std::collections::VecDeque;

    const TEST_AD1: &str = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/02606-0900_1E_GRCDH2/02606-0900_1E_GRCDH2_IMG1.ad1";

    #[test]
    fn enumerate_all_items() {
        if !std::path::Path::new(TEST_AD1).exists() {
            eprintln!("SKIPPED: test fixture not found: {}", TEST_AD1);
            return;
        }
        println!("\n================================================================================");
        println!("=== COMPLETE AD1 CONTAINER ENUMERATION ===");
        println!("================================================================================");
        
        let start = Instant::now();
        let session = SessionV2::open(TEST_AD1).expect("Failed to open AD1");
        
        println!("\n📦 AD1 CONTAINER INFO:");
        println!("────────────────────────────────────────");
        println!("   File: {}", TEST_AD1.split('/').next_back().unwrap_or(""));
        println!("   Segments: {}", session.segments.len());
        let total: u64 = session.segments.iter().filter_map(|s| s.as_ref()).map(|s| s.size).sum();
        println!("   Total Size: {:.2} GB", total as f64 / 1024.0 / 1024.0 / 1024.0);
        println!("   Version: {}", session.logical_header.image_version);
        
        let first_addr = session.logical_header.first_item_addr;
        
        println!("\n================================================================================");
        println!("📂 COMPLETE FILE/FOLDER TREE (All 43,842 items):");
        println!("================================================================================\n");
        
        let mut total_files = 0u64;
        let mut total_folders = 0u64;
        let mut total_bytes = 0u64;
        let mut max_depth = 0usize;
        
        let mut queue: VecDeque<(u64, usize, bool)> = VecDeque::new();
        queue.push_back((first_addr, 0, true));
        
        let mut item_count = 0u64;
        let max_items = 200000;
        
        // Track all items for output
        let mut all_items: Vec<(String, bool, u64, usize)> = Vec::new(); // (name, is_dir, size, depth)
        
        while let Some((addr, depth, is_sibling_chain)) = queue.pop_front() {
            if addr == 0 || item_count >= max_items {
                continue;
            }
            
            item_count += 1;
            
            if let Ok(item) = session.read_item_at(addr) {
                    let is_dir = item.item_type == 0x05;
                    
                    all_items.push((item.name.clone(), is_dir, item.decompressed_size, depth));
                    
                    if is_dir {
                        total_folders += 1;
                        if item.first_child_addr != 0 {
                            queue.push_back((item.first_child_addr, depth + 1, true));
                        }
                    } else {
                        total_files += 1;
                        total_bytes += item.decompressed_size;
                    }
                    
                    if depth > max_depth {
                        max_depth = depth;
                    }
                    
                    if is_sibling_chain && item.next_item_addr != 0 {
                        queue.push_back((item.next_item_addr, depth, true));
                    }
            }
        }
        
        // Now print everything organized by depth
        println!("📁 ROOT LEVEL (2 items):");
        println!("════════════════════════════════════════════════════════════════════════════════");
        for (name, is_dir, size, depth) in &all_items {
            if *depth == 0 {
                let icon = if *is_dir { "📂" } else { "📄" };
                let size_str = if *is_dir { "".to_string() } else { format!(" ({})", format_size_compact(*size)) };
                println!("{} {}{}", icon, name, size_str);
            }
        }
        
        println!("\n📁 LEVEL 1 - Direct children of roots:");
        println!("════════════════════════════════════════════════════════════════════════════════");
        let mut count = 0;
        for (name, is_dir, size, depth) in &all_items {
            if *depth == 1 && count < 100 {
                let icon = if *is_dir { "📂" } else { "📄" };
                let size_str = if *is_dir { "".to_string() } else { format!(" ({})", format_size_compact(*size)) };
                println!("  {} {}{}", icon, name, size_str);
                count += 1;
            }
        }
        if count == 100 {
            let total_at_level = all_items.iter().filter(|(_, _, _, d)| *d == 1).count();
            println!("  ... and {} more at this level", total_at_level - 100);
        }
        
        println!("\n📁 LEVEL 2:");
        println!("════════════════════════════════════════════════════════════════════════════════");
        count = 0;
        for (name, is_dir, size, depth) in &all_items {
            if *depth == 2 && count < 100 {
                let icon = if *is_dir { "📂" } else { "📄" };
                let size_str = if *is_dir { "".to_string() } else { format!(" ({})", format_size_compact(*size)) };
                println!("    {} {}{}", icon, name, size_str);
                count += 1;
            }
        }
        if count == 100 {
            let total_at_level = all_items.iter().filter(|(_, _, _, d)| *d == 2).count();
            println!("    ... and {} more at this level", total_at_level - 100);
        }
        
        println!("\n📁 LEVEL 3:");
        println!("════════════════════════════════════════════════════════════════════════════════");
        count = 0;
        for (name, is_dir, size, depth) in &all_items {
            if *depth == 3 && count < 100 {
                let icon = if *is_dir { "📂" } else { "📄" };
                let size_str = if *is_dir { "".to_string() } else { format!(" ({})", format_size_compact(*size)) };
                println!("      {} {}{}", icon, name, size_str);
                count += 1;
            }
        }
        if count == 100 {
            let total_at_level = all_items.iter().filter(|(_, _, _, d)| *d == 3).count();
            println!("      ... and {} more at this level", total_at_level - 100);
        }
        
        // Level statistics
        println!("\n📊 ITEMS PER DEPTH LEVEL:");
        println!("════════════════════════════════════════════════════════════════════════════════");
        for level in 0..=max_depth {
            let (folders, files, bytes): (usize, usize, u64) = all_items.iter()
                .filter(|(_, _, _, d)| *d == level)
                .fold((0, 0, 0u64), |(f, fi, b), (_, is_dir, size, _)| {
                    if *is_dir { (f + 1, fi, b) } else { (f, fi + 1, b + size) }
                });
            if folders > 0 || files > 0 {
                println!("   Level {:2}: {:5} folders, {:6} files ({:>12})", 
                    level, folders, files, format_size_compact(bytes));
            }
        }
        
        let elapsed = start.elapsed();
        
        println!("\n================================================================================");
        println!("📊 GRAND TOTAL SUMMARY");
        println!("================================================================================");
        println!("   Total Folders:  {:>10}", total_folders);
        println!("   Total Files:    {:>10}", total_files);
        println!("   Total Items:    {:>10}", total_files + total_folders);
        println!("   Total Size:     {} ({} bytes)", format_size_compact(total_bytes), total_bytes);
        println!("   Max Depth:      {}", max_depth);
        println!("   Enum Time:      {:?}", elapsed);
        println!("================================================================================");
        println!("✅ ENUMERATION COMPLETE");
    }
}
