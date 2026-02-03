//! Real-World Export Performance Test
//! Tests actual forensic export scenarios to identify performance issues

use seven_zip::{SevenZip, CompressionLevel, CompressOptions, StreamOptions};
use std::fs::{self, File};
use std::io::Write;
use std::time::Instant;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_forensic_evidence(dir: &std::path::Path) -> Vec<String> {
    let mut files = Vec::new();
    
    // Simulate forensic evidence files
    
    // 1. Log files (highly compressible) - 5MB each
    for i in 0..3 {
        let log_file = dir.join(format!("system_log_{}.txt", i));
        let mut f = File::create(&log_file).unwrap();
        for j in 0..100000 {
            writeln!(f, "[2026-02-02 {:02}:{:02}:{:02}.{:03}] [INFO] Event {} - Process activity recorded for forensic analysis", 
                     (j / 3600) % 24, (j / 60) % 60, j % 60, j % 1000, j).unwrap();
        }
        files.push(log_file.to_string_lossy().to_string());
    }
    
    // 2. Database files (moderately compressible) - 2MB each
    for i in 0..2 {
        let db_file = dir.join(format!("evidence_db_{}.sqlite", i));
        let mut f = File::create(&db_file).unwrap();
        // Simulate SQLite-like structure
        let mut seed = (i as u64 + 1) * 12345;
        for _ in 0..2_000_000 {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let byte = if seed % 10 < 7 {
                // 70% structured data
                (b'A' + (seed % 26) as u8) as u8
            } else {
                // 30% binary
                ((seed >> 16) & 0xFF) as u8
            };
            f.write_all(&[byte]).unwrap();
        }
        files.push(db_file.to_string_lossy().to_string());
    }
    
    // 3. Image files (incompressible) - 1MB each
    for i in 0..2 {
        let img_file = dir.join(format!("screenshot_{}.raw", i));
        let mut f = File::create(&img_file).unwrap();
        let mut seed = (i as u64 + 100) * 54321;
        for _ in 0..1_000_000 {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            f.write_all(&[((seed >> 16) & 0xFF) as u8]).unwrap();
        }
        files.push(img_file.to_string_lossy().to_string());
    }
    
    files
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     Real-World Forensic Export Performance Analysis           ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let temp_dir = TempDir::new().unwrap();
    let evidence_dir = temp_dir.path().join("evidence");
    fs::create_dir_all(&evidence_dir).unwrap();
    
    println!("Creating simulated forensic evidence...");
    let start = Instant::now();
    let files = create_forensic_evidence(&evidence_dir);
    println!("  Evidence created in {:?}\n", start.elapsed());
    
    let total_size: u64 = files.iter()
        .map(|f| fs::metadata(f).unwrap().len())
        .sum();
    
    println!("Evidence files ({} total, {:.1} MB):", files.len(), total_size as f64 / 1_000_000.0);
    for f in &files {
        let size = fs::metadata(f).unwrap().len();
        let name = std::path::Path::new(f).file_name().unwrap().to_string_lossy();
        println!("  {} - {:.1} MB", name, size as f64 / 1_000_000.0);
    }
    
    let sz = SevenZip::new().unwrap();
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  SCENARIO 1: Quick Export (Level 1 - Fastest)");
    println!("═══════════════════════════════════════════════════════════════");
    
    let archive = temp_dir.path().join("quick_export.7z");
    let start = Instant::now();
    sz.create_archive(&archive, &file_refs, CompressionLevel::Fastest, None).unwrap();
    let elapsed = start.elapsed();
    let archive_size = fs::metadata(&archive).unwrap().len();
    
    println!("  Time: {:?}", elapsed);
    println!("  Speed: {:.1} MB/s", (total_size as f64 / 1_000_000.0) / elapsed.as_secs_f64());
    println!("  Size: {:.1} MB ({:.1}% ratio)", archive_size as f64 / 1_000_000.0, 
             (archive_size as f64 / total_size as f64) * 100.0);
    
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  SCENARIO 2: Balanced Export (Level 5 - Normal)");
    println!("═══════════════════════════════════════════════════════════════");
    
    let archive = temp_dir.path().join("balanced_export.7z");
    let start = Instant::now();
    sz.create_archive(&archive, &file_refs, CompressionLevel::Normal, None).unwrap();
    let elapsed = start.elapsed();
    let archive_size = fs::metadata(&archive).unwrap().len();
    
    println!("  Time: {:?}", elapsed);
    println!("  Speed: {:.1} MB/s", (total_size as f64 / 1_000_000.0) / elapsed.as_secs_f64());
    println!("  Size: {:.1} MB ({:.1}% ratio)", archive_size as f64 / 1_000_000.0,
             (archive_size as f64 / total_size as f64) * 100.0);
    
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  SCENARIO 3: Maximum Export (Level 9 - Ultra)");
    println!("═══════════════════════════════════════════════════════════════");
    
    let archive = temp_dir.path().join("max_export.7z");
    let start = Instant::now();
    sz.create_archive(&archive, &file_refs, CompressionLevel::Ultra, None).unwrap();
    let elapsed = start.elapsed();
    let archive_size = fs::metadata(&archive).unwrap().len();
    
    println!("  Time: {:?}", elapsed);
    println!("  Speed: {:.1} MB/s", (total_size as f64 / 1_000_000.0) / elapsed.as_secs_f64());
    println!("  Size: {:.1} MB ({:.1}% ratio)", archive_size as f64 / 1_000_000.0,
             (archive_size as f64 / total_size as f64) * 100.0);
    
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  SCENARIO 4: Encrypted Export (AES-256)");
    println!("═══════════════════════════════════════════════════════════════");
    
    let archive = temp_dir.path().join("encrypted_export.7z");
    let opts = CompressOptions {
        password: Some("ForensicPassword2026!".to_string()),
        ..Default::default()
    };
    let start = Instant::now();
    sz.create_archive(&archive, &file_refs, CompressionLevel::Normal, Some(&opts)).unwrap();
    let elapsed = start.elapsed();
    let archive_size = fs::metadata(&archive).unwrap().len();
    
    println!("  Time: {:?}", elapsed);
    println!("  Speed: {:.1} MB/s", (total_size as f64 / 1_000_000.0) / elapsed.as_secs_f64());
    println!("  Size: {:.1} MB (encrypted)", archive_size as f64 / 1_000_000.0);
    
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  SCENARIO 5: Streaming with Progress (App-like)");
    println!("═══════════════════════════════════════════════════════════════");
    
    let archive = temp_dir.path().join("streaming_export.7z");
    let progress_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let progress_count_clone = progress_count.clone();
    
    let opts = StreamOptions::default();
    let start = Instant::now();
    sz.create_archive_streaming(
        &archive, 
        &file_refs, 
        CompressionLevel::Normal, 
        Some(&opts),
        Some(Box::new(move |_p, _t, _fb, _ft, _fn| {
            progress_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        })),
    ).unwrap();
    let elapsed = start.elapsed();
    let archive_size = fs::metadata(&archive).unwrap().len();
    
    println!("  Time: {:?}", elapsed);
    println!("  Speed: {:.1} MB/s", (total_size as f64 / 1_000_000.0) / elapsed.as_secs_f64());
    println!("  Progress callbacks: {}", progress_count.load(std::sync::atomic::Ordering::SeqCst));
    println!("  Size: {:.1} MB", archive_size as f64 / 1_000_000.0);
    
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  PERFORMANCE RECOMMENDATIONS");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    println!("  For interactive exports (user waiting):");
    println!("    → Use Level 1 (Fastest): ~700+ MB/s, good compression");
    println!("");
    println!("  For background exports (batched):");
    println!("    → Use Level 5 (Normal): ~30 MB/s, excellent compression");
    println!("");
    println!("  Avoid Level 9 (Ultra) unless storage is critical:");
    println!("    → Very slow (~9 MB/s), minimal size improvement");
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                  ANALYSIS COMPLETE                             ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
}
