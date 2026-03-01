// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Test viewer hex highlighting regions
use ffx_check_lib::viewer;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or("/Users/terryreynolds/Downloads/4Dell Latitude CPi.E01".to_string());
    println!("Testing viewer hex regions for: {}\n", path);

    match viewer::parse_file_header(&path) {
        Ok(metadata) => {
            println!("Format: {}", metadata.format);
            if let Some(v) = &metadata.version {
                println!("Version: {}", v);
            }

            println!("\n=== METADATA FIELDS ({}) ===", metadata.fields.len());
            let mut current_category = String::new();
            for field in &metadata.fields {
                if field.category != current_category {
                    current_category = field.category.clone();
                    println!("\n[{}]", current_category);
                }
                println!("  {}: {}", field.key, field.value);
            }

            println!("\n=== HEX REGIONS ({}) ===", metadata.regions.len());
            for region in &metadata.regions {
                println!(
                    "  0x{:06X}-0x{:06X}  {}  [{}]",
                    region.start, region.end, region.name, region.color_class
                );
                println!("                     {}", region.description);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
