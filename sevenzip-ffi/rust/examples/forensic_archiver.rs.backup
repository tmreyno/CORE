//! Forensic evidence archiver example
//!
//! Demonstrates archiving large directories with encryption and split archives

use seven_zip::{SevenZip, CompressionLevel, StreamOptions, Error};
use std::env;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 4 {
        println!("Usage: {} <command> <archive> <directory> [password]", args[0]);
        println!();
        println!("Commands:");
        println!("  compress  - Create encrypted archive");
        println!("  extract   - Extract encrypted archive");
        println!();
        println!("Example:");
        println!("  {} compress evidence.7z /path/to/evidence MyPassword123", args[0]);
        println!("  {} extract evidence.7z.001 ./extracted MyPassword123", args[0]);
        return Ok(());
    }

    let command = &args[1];
    let archive = &args[2];
    let path = &args[3];
    let password = args.get(4).map(|s| s.as_str());

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
            opts.num_threads = 10; // Use all 10 cores (original setting)
            opts.split_size = 8 * 1024 * 1024 * 1024; // 8GB splits
            opts.password = password.map(|p| p.to_string());
            // Leave dict_size at 0 to use defaults (256KB for fastest mode)

            println!("Settings:");
            println!("  Threads:     {}", opts.num_threads);
            println!("  Split size:  8 GB");
            println!("  Dictionary:  Default (64KB for fastest)");
            println!("  Compression: LZMA2 (Fastest - Original Settings)");
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
                CompressionLevel::Fastest,  // Maximum speed
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
