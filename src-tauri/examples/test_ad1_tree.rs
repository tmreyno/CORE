//! Test AD1 tree loading for the GRCDH2 evidence file

use ffx_check_lib::ad1;
use ffx_check_lib::ad1::TreeEntry;

fn main() {
    let path = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/02606-0900_1E_GRCDH2/02606-0900_1E_GRCDH2_IMG1.ad1";
    
    println!("Testing AD1 tree loading...");
    println!("Path: {}", path);
    
    // Check if file exists
    if !std::path::Path::new(path).exists() {
        println!("ERROR: File does not exist!");
        println!("Checking for mounted volumes...");
        if let Ok(entries) = std::fs::read_dir("/Volumes") {
            for entry in entries.flatten() {
                println!("  Volume: {:?}", entry.path());
            }
        }
        return;
    }
    
    // Check if it's AD1
    match ad1::is_ad1(path) {
        Ok(true) => println!("✓ Confirmed AD1 format"),
        Ok(false) => {
            println!("✗ Not detected as AD1");
            return;
        }
        Err(e) => {
            println!("✗ Error checking format: {}", e);
            return;
        }
    }
    
    // Try to get fast info first
    println!("\n--- Getting fast info (headers only) ---");
    match ad1::info_fast(path) {
        Ok(info) => {
            println!("✓ info_fast succeeded!");
            println!("  Segment count: {}", info.segment.segment_number);
            println!("  First item addr: 0x{:X}", info.logical.first_item_addr);
            println!("  Data source: {}", info.logical.data_source_name);
            println!("  Item count: {}", info.item_count);
        }
        Err(e) => {
            println!("✗ info_fast failed: {}", e);
        }
    }
    
    // Try to get tree
    println!("\n--- Getting tree entries ---");
    match ad1::get_tree(path) {
        Ok(entries) => {
            let entries: Vec<TreeEntry> = entries;
            println!("✓ get_tree succeeded! {} entries", entries.len());
            // Show first 20 entries
            for (i, entry) in entries.iter().take(20).enumerate() {
                let type_str = if entry.is_dir { "DIR " } else { "FILE" };
                println!("  {:3}. [{}] {} (size: {})", i+1, type_str, entry.path, entry.size);
            }
            if entries.len() > 20 {
                println!("  ... and {} more entries", entries.len() - 20);
            }
        }
        Err(e) => {
            println!("✗ get_tree failed: {}", e);
        }
    }
    
    // Try full info with tree
    println!("\n--- Getting full info with tree ---");
    match ad1::info(path, true) {
        Ok(info) => {
            println!("✓ info(include_tree=true) succeeded!");
            if let Some(tree) = &info.tree {
                let tree: &Vec<TreeEntry> = tree;
                println!("  Tree has {} entries", tree.len());
            } else {
                println!("  Tree is None!");
            }
        }
        Err(e) => {
            println!("✗ info failed: {}", e);
        }
    }
}
