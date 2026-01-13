// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Test AD1 verification with debug output
//! 
//! Usage: cargo run --example test_ad1_verify -- /path/to/file.ad1

use ffx_check_lib::ad1::{self, VerifyStatus};
use std::env;
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), String> {
    // Initialize tracing with env filter
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        &args[1]
    } else {
        eprintln!("Usage: {} <ad1-file>", args[0]);
        std::process::exit(1);
    };
    
    println!("Testing AD1 verification for: {}", path);
    println!("{}", "=".repeat(60));
    
    // First, get info about the AD1
    println!("\n--- AD1 Info ---");
    match ad1::info(path, false) {
        Ok(info) => {
            println!("Segment: {:?}", info.segment.signature);
            println!("Item count: {}", info.item_count);
            if let Some(vol) = &info.volume {
                println!("Volume label: {:?}", vol.volume_label);
                println!("Filesystem: {:?}", vol.filesystem);
            }
            if let Some(log) = &info.companion_log {
                println!("Case #: {:?}", log.case_number);
                println!("Evidence #: {:?}", log.evidence_number);
                println!("MD5 (from log): {:?}", log.md5_hash);
                println!("SHA1 (from log): {:?}", log.sha1_hash);
            }
        }
        Err(e) => {
            eprintln!("Error getting info: {}", e);
            return Err(e.to_string());
        }
    }
    
    // Verify with progress
    println!("\n--- Verification ---");
    let mut last_percent = 0;
    match ad1::verify_with_progress(path, "md5", |current, total| {
        if total > 0 {
            let percent = (current * 100 / total) as i32;
            if percent > last_percent && percent % 10 == 0 {
                println!("Progress: {}% ({}/{})", percent, current, total);
                last_percent = percent;
            }
        }
    }) {
        Ok(results) => {
            println!("\nVerification complete. {} items checked.", results.len());
            
            let passed: Vec<_> = results.iter().filter(|r| r.status == VerifyStatus::Ok).collect();
            let failed: Vec<_> = results.iter().filter(|r| r.status == VerifyStatus::Nok).collect();
            let computed: Vec<_> = results.iter().filter(|r| r.status == VerifyStatus::Computed).collect();
            let skipped: Vec<_> = results.iter().filter(|r| r.status == VerifyStatus::Skipped).collect();
            
            println!("\nSummary:");
            println!("  Passed: {}", passed.len());
            println!("  Failed: {}", failed.len());
            println!("  Computed (no stored hash): {}", computed.len());
            println!("  Skipped: {}", skipped.len());
            
            if !failed.is_empty() {
                println!("\nFailed items:");
                for r in &failed {
                    println!("  - {}", r.path);
                    println!("    Algorithm: {:?}", r.algorithm);
                    println!("    Stored:   {:?}", r.stored);
                    println!("    Computed: {:?}", r.computed);
                    println!("    Size: {:?} bytes", r.size);
                }
            }
            
            // Show some passed items for debugging
            if !passed.is_empty() && std::env::var("SHOW_PASSED").is_ok() {
                println!("\nPassed items (first 5):");
                for r in passed.iter().take(5) {
                    println!("  - {}", r.path);
                    println!("    Computed: {:?}", r.computed);
                }
            }
        }
        Err(e) => {
            eprintln!("Verification error: {}", e);
            return Err(e.to_string());
        }
    }
    
    Ok(())
}
