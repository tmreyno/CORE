use seven_zip::{SevenZip, CompressionLevel};
use std::fs::{self, File};
use std::io::{Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Large File Archiving Demo (100GB+ Support) ===\n");
    
    let sz = SevenZip::new().expect("Failed to initialize");
    
    fs::create_dir_all("large_file_test").ok();
    
    // Create 100MB test file (simulating large data)
    println!("Creating 100MB test file...");
    let test_file = "large_file_test/large_file.bin";
    create_test_file(test_file, 100 * 1024 * 1024);
    
    // Compress to single archive
    println!("\nCompressing to 7z archive...");
    match sz.create_archive(
        "large_file_test/compressed.7z",
        &[test_file],
        CompressionLevel::Fast,
        None,
    ) {
        Ok(_) => {
            println!("✓ Archive created successfully!");
            
            let archive_size = fs::metadata("large_file_test/compressed.7z")
                .unwrap()
                .len();
            println!("  Archive size: {:.2} MB", archive_size as f64 / (1024.0 * 1024.0));
            
            // Now split the archive into chunks
            println!("\nSplitting archive into 25MB chunks...");
            let chunk_size = 25 * 1024 * 1024;
            split_file("large_file_test/compressed.7z", chunk_size)?;
            
            println!("\n✅ SUCCESS!");
            println!("\nFor 100GB+ files:");
            println!("1. Compress: sz.create_archive(path, files, level, None)");
            println!("2. Split: Use split_file() to create manageable chunks");
            println!("3. Transfer: Move .part files separately");
            println!("4. Rejoin: cat compressed.7z.part* > compressed.7z");
            println!("5. Extract: sz.extract('compressed.7z', output_dir)");
            
            println!("\nGenerated files:");
            for entry in fs::read_dir("large_file_test").unwrap() {
                let entry = entry.unwrap();
                if entry.path().extension().and_then(|s| s.to_str()) == Some("part") {
                    let size = entry.metadata().unwrap().len();
                    println!("  {} - {:.2} MB", 
                        entry.file_name().to_string_lossy(),
                        size as f64 / (1024.0 * 1024.0)
                    );
                }
            }
        }
        Err(e) => println!("❌ Error: {:?}", e),
    }
    
    Ok(())
}

fn create_test_file(path: &str, size: usize) {
    let mut file = File::create(path).unwrap();
    let mut rng_state: u64 = 0x123456789ABCDEF0;
    
    let chunk_size = 1024 * 1024;
    let mut remaining = size;
    
    while remaining > 0 {
        let to_write = remaining.min(chunk_size);
        let mut chunk = Vec::with_capacity(to_write);
        
        for _ in 0..to_write {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            chunk.push((rng_state >> 32) as u8);
        }
        
        file.write_all(&chunk).unwrap();
        remaining -= to_write;
        
        let progress = ((size - remaining) as f64 / size as f64) * 100.0;
        print!("\r  Progress: {:.1}%", progress);
        std::io::stdout().flush().unwrap();
    }
    println!();
}

fn split_file(path: &str, chunk_size: usize) -> std::io::Result<()> {
    let mut input = File::open(path)?;
    let mut buffer = vec![0u8; chunk_size];
    let mut part_num = 1;
    
    loop {
        let bytes_read = input.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        let part_path = format!("{}.part{:03}", path, part_num);
        let mut output = File::create(&part_path)?;
        output.write_all(&buffer[..bytes_read])?;
        
        println!("  Created: {} ({:.2} MB)", 
            part_path,
            bytes_read as f64 / (1024.0 * 1024.0)
        );
        
        part_num += 1;
    }
    
    Ok(())
}
