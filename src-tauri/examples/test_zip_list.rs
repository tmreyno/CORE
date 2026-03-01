// Test ZIP listing with libarchive fallback
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <zip_file>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
    println!("Testing ZIP: {}", path);

    // Try listing entries
    match ffx_check_lib::archive::list_zip_entries(path) {
        Ok(entries) => {
            println!("Successfully listed {} entries:", entries.len());
            for (i, entry) in entries.iter().take(20).enumerate() {
                println!(
                    "  {:3}. {} {} ({})",
                    i + 1,
                    if entry.is_directory { "[DIR]" } else { "     " },
                    entry.path,
                    entry.compression_method
                );
            }
            if entries.len() > 20 {
                println!("  ... and {} more entries", entries.len() - 20);
            }
        }
        Err(e) => {
            eprintln!("Failed to list entries: {}", e);
            std::process::exit(1);
        }
    }
}
