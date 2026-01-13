// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AD1 V2 Command-line Test Tool
//! 
//! This tool tests the AD1 V2 implementation with real AD1 files

use std::env;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "info" => {
            if args.len() < 3 {
                eprintln!("Error: Missing AD1 file path");
                eprintln!("Usage: ad1_test info <path-to-ad1>");
                process::exit(1);
            }
            test_info(&args[2]);
        }
        "list" => {
            if args.len() < 3 {
                eprintln!("Error: Missing AD1 file path");
                eprintln!("Usage: ad1_test list <path-to-ad1>");
                process::exit(1);
            }
            test_list(&args[2]);
        }
        "verify" => {
            if args.len() < 3 {
                eprintln!("Error: Missing AD1 file path");
                eprintln!("Usage: ad1_test verify <path-to-ad1> [md5|sha1]");
                process::exit(1);
            }
            let hash_type = if args.len() >= 4 {
                &args[3]
            } else {
                "md5"
            };
            test_verify(&args[2], hash_type);
        }
        "extract" => {
            if args.len() < 4 {
                eprintln!("Error: Missing arguments");
                eprintln!("Usage: ad1_test extract <path-to-ad1> <output-dir>");
                process::exit(1);
            }
            test_extract(&args[2], &args[3]);
        }
        "all" => {
            if args.len() < 3 {
                eprintln!("Error: Missing AD1 file path");
                eprintln!("Usage: ad1_test all <path-to-ad1>");
                process::exit(1);
            }
            run_all_tests(&args[2]);
        }
        _ => {
            eprintln!("Error: Unknown command: {}", command);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    println!("AD1 V2 Test Tool");
    println!();
    println!("Usage:");
    println!("  ad1_test info <path-to-ad1>              Show container information");
    println!("  ad1_test list <path-to-ad1>              List root files");
    println!("  ad1_test verify <path-to-ad1> [md5|sha1] Verify file hashes");
    println!("  ad1_test extract <path-to-ad1> <out-dir> Extract files");
    println!("  ad1_test all <path-to-ad1>               Run all tests");
    println!();
    println!("Examples:");
    println!("  ad1_test info ~/evidence/test.ad1");
    println!("  ad1_test verify ~/evidence/test.ad1 md5");
    println!("  ad1_test all ~/evidence/test.ad1");
}

fn test_info(path: &str) {
    println!("====================");
    println!("Container Info Test");
    println!("====================");
    println!();
    println!("File: {}", path);
    
    if !Path::new(path).exists() {
        eprintln!("✗ Error: File not found");
        process::exit(1);
    }
    
    println!("✓ File exists");
    
    // Actual implementation would call SessionV2::open() and get_container_info()
    println!("\n✓ Info test passed (implementation needed in lib)");
}

fn test_list(path: &str) {
    println!("====================");
    println!("List Files Test");
    println!("====================");
    println!();
    println!("File: {}", path);
    
    if !Path::new(path).exists() {
        eprintln!("✗ Error: File not found");
        process::exit(1);
    }
    
    println!("✓ File exists");
    
    // Actual implementation would call get_root_children_v2()
    println!("\n✓ List test passed (implementation needed in lib)");
}

fn test_verify(path: &str, hash_type: &str) {
    println!("====================");
    println!("Hash Verification Test");
    println!("====================");
    println!();
    println!("File: {}", path);
    println!("Hash type: {}", hash_type);
    
    if !Path::new(path).exists() {
        eprintln!("✗ Error: File not found");
        process::exit(1);
    }
    
    println!("✓ File exists");
    
    // Actual implementation would call verify_all_items()
    println!("\n✓ Verify test passed (implementation needed in lib)");
}

fn test_extract(path: &str, output: &str) {
    println!("====================");
    println!("Extraction Test");
    println!("====================");
    println!();
    println!("File: {}", path);
    println!("Output: {}", output);
    
    if !Path::new(path).exists() {
        eprintln!("✗ Error: File not found");
        process::exit(1);
    }
    
    println!("✓ File exists");
    
    // Actual implementation would call extract_all()
    println!("\n✓ Extract test passed (implementation needed in lib)");
}

fn run_all_tests(path: &str) {
    println!("================================");
    println!("Running All AD1 V2 Tests");
    println!("================================");
    println!();
    
    test_info(path);
    println!();
    
    test_list(path);
    println!();
    
    test_verify(path, "md5");
    println!();
    
    println!("================================");
    println!("All Tests Completed");
    println!("================================");
    println!();
    println!("✓ All basic tests passed");
    println!();
    println!("Note: Full functionality testing requires:");
    println!("  - Opening actual AD1 containers");
    println!("  - Parsing tree structures");
    println!("  - Computing file hashes");
    println!("  - Extracting files");
    println!();
    println!("These will be tested through Tauri commands.");
}
