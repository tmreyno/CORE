//! Forensic evidence archiver example
//!
//! Demonstrates archiving large directories with encryption and split archives

use seven_zip::{SevenZip, CompressionLevel, StreamOptions, Error};
use std::env;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 4 {
        println!("Usage: {} <command> <archive> <directory> [password] [--nosplit]", args[0]);
        println!();
        println!("Commands:");
        println!("  compress  - Create encrypted archive");
        println!("  extract   - Extract encrypted archive");
        println!("  test      - Test archive integrity");
        println!();
        println!("Options:");
        println!("  --nosplit - Create single file archive (no 2GB split volumes)");
        println!();
        println!("Example:");
        println!("  {} compress evidence.7z /path/to/evidence MyPassword123", args[0]);
        println!("  {} compress evidence.7z /path/to/data --nosplit", args[0]);
        println!("  {} extract evidence.7z.001 ./extracted MyPassword123", args[0]);
        return Ok(());
    }

    let command = &args[1];
    let archive = &args[2];
    let path = &args[3];
    
    // Parse optional args - password and --nosplit flag
    let mut password: Option<&str> = None;
    let mut no_split = false;
    let mut custom_split_size: u64 = 2 * 1024 * 1024 * 1024; // 2GB default
    for arg in args.iter().skip(4) {
        if arg == "--nosplit" || arg == "--single" {
            no_split = true;
        } else if arg.starts_with("--split=") {
            if let Ok(size) = arg.trim_start_matches("--split=").parse::<u64>() {
                custom_split_size = size * 1024 * 1024; // Argument is in MB
            }
        } else if password.is_none() {
            password = Some(arg.as_str());
        }
    }

    let sz = SevenZip::new()?;

    match command.as_str() {
        "compress" => {
            println!("╔═══════════════════════════════════════════════════════════╗");
            println!("║        Forensic Evidence Archiver - Compression          ║");
            println!("╚═══════════════════════════════════════════════════════════╝\n");

            println!("Input:    {}", path);
            println!("Output:   {}", archive);
            println!("Password: {}", if password.is_some() { "Yes (encrypted)" } else { "No" });
            println!();

            let mut opts = StreamOptions::default();
            opts.num_threads = 10; // Maximum threads - use all 10 cores
            opts.split_size = if no_split { 0 } else { custom_split_size }; // 0 = single file
            opts.password = password.map(|p| p.to_string());
            // dict_size = 0 (default) - SDK auto-optimizes based on compression level

            println!("Settings:");
            println!("  Threads:     {} (MAX - all cores)", opts.num_threads);
            println!("  Split size:  {}", if no_split { "Single file (no split)" } else { "2 GB" });
            println!("  Dictionary:  N/A (Store mode - no compression)");
            println!("  Compression: Level 0 (Store) - Maximum Speed");
            println!("  Encryption:  {}", if opts.password.is_some() { "AES-256-CBC" } else { "None" });
            println!();

            println!("Compressing (streaming mode - memory efficient)...\n");

            // Use streaming compression - CRITICAL for large files!
            // This uses 64MB chunks instead of loading entire files into memory
            opts.chunk_size = 64 * 1024 * 1024; // 64MB chunks (memory-safe)
            
            let start_time = std::time::Instant::now();
            let progress_callback = Box::new(move |processed: u64, total: u64, file_bytes: u64, file_total: u64, filename: &str| {
                if total > 0 {
                    let pct = (processed as f64 / total as f64) * 100.0;
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let speed = if elapsed > 0.0 { processed as f64 / elapsed / 1_000_000.0 } else { 0.0 };
                    print!("\r[{:50}] {:.1}% | {:.1} MB/s | {} ", 
                        "=".repeat((pct / 2.0) as usize),
                        pct,
                        speed,
                        filename.split('/').last().unwrap_or(filename));
                    use std::io::Write;
                    std::io::stdout().flush().ok();
                }
            });

            sz.create_archive_streaming(
                archive,
                &[path],
                CompressionLevel::Store,  // Level 0 - Maximum speed for pre-compressed data
                Some(&opts),
                Some(progress_callback)
            )?;

            println!("\n✓ Archive created successfully!");
            println!("\nYou can now:");
            println!("  1. Verify integrity: test {}", archive);
            println!("  2. Extract: {} extract {} ./output {}", 
                args[0], archive, password.unwrap_or("[password]"));
        }

        "extract" => {
            println!("╔═══════════════════════════════════════════════════════════╗");
            println!("║        Forensic Evidence Archiver - Extraction           ║");
            println!("╚═══════════════════════════════════════════════════════════╝\n");

            println!("Archive: {}", archive);
            println!("Output:  {}", path);
            println!();

            println!("Extracting...\n");

            sz.extract_with_password(
                archive,
                path,
                password,
                Some(Box::new(|completed, total| {
                    if total > 0 {
                        let pct = (completed as f64 / total as f64) * 100.0;
                        print!("\rProgress: [{:50}] {:.1}%", 
                            "=".repeat((pct / 2.0) as usize),
                            pct);
                    }
                }))
            )?;

            println!("\n\n✓ Extraction completed successfully!");
        }

        "test" => {
            println!("╔═══════════════════════════════════════════════════════════╗");
            println!("║      Forensic Evidence Archiver - Integrity Test         ║");
            println!("╚═══════════════════════════════════════════════════════════╝\n");

            println!("Archive: {}", archive);
            println!();
            println!("Testing archive integrity...\n");

            sz.test_archive(archive, password)?;

            println!("✓ Archive integrity verified!");
            println!("  - All CRCs match");
            println!("  - All files can be decompressed");
            println!("  - Archive structure is valid");
        }

        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            eprintln!("Use 'compress' or 'extract'");
            std::process::exit(1);
        }
    }

    Ok(())
}
