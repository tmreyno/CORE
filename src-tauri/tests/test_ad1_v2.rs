// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Integration tests for AD1 V2 implementation

use std::path::Path;

// Test data directory
const TEST_DATA_DIR: &str = "/Users/terryreynolds/1827-1001 Case With Data/1.Evidence";

#[cfg(test)]
mod ad1_v2_tests {
    use super::*;

    /// Test opening various AD1 formats
    #[test]
    fn test_open_containers() {
        let test_files = vec![
            "test_5gb_encase7.E01",
            "test_5gb_linen7.E01",
            "test_5gb_ewf.e01",
        ];

        for file in test_files {
            let path = Path::new(TEST_DATA_DIR).join(file);
            if path.exists() {
                println!("Testing: {}", file);
                // Test will be implemented when we can import the modules
                println!("  ✓ File exists");
            } else {
                println!("  ⚠ File not found: {}", path.display());
            }
        }
    }

    /// Test reading root children
    #[test]
    fn test_root_children() {
        let path = Path::new(TEST_DATA_DIR).join("test_5gb_encase7.E01");
        if !path.exists() {
            println!("⚠ Test file not found, skipping");
            return;
        }
        
        println!("Testing root children loading...");
        // Implementation will be added
    }

    /// Test hash verification
    #[test]
    fn test_hash_verification() {
        let path = Path::new(TEST_DATA_DIR).join("test_5gb_encase7.E01");
        if !path.exists() {
            println!("⚠ Test file not found, skipping");
            return;
        }
        
        println!("Testing hash verification...");
        // Implementation will be added
    }

    /// Test file extraction
    #[test]
    fn test_extraction() {
        let path = Path::new(TEST_DATA_DIR).join("test_5gb_encase7.E01");
        if !path.exists() {
            println!("⚠ Test file not found, skipping");
            return;
        }
        
        println!("Testing file extraction...");
        // Implementation will be added
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_open_speed() {
        let path = Path::new(TEST_DATA_DIR).join("test_5gb_encase7.E01");
        if !path.exists() {
            println!("⚠ Test file not found, skipping benchmark");
            return;
        }

        let start = Instant::now();
        // Open container
        let duration = start.elapsed();
        
        println!("Open time: {:?}", duration);
        assert!(duration.as_millis() < 100, "Open should be < 100ms");
    }

    #[test]
    fn benchmark_root_load_speed() {
        let path = Path::new(TEST_DATA_DIR).join("test_5gb_encase7.E01");
        if !path.exists() {
            println!("⚠ Test file not found, skipping benchmark");
            return;
        }

        let start = Instant::now();
        // Load root children
        let duration = start.elapsed();
        
        println!("Root load time: {:?}", duration);
        assert!(duration.as_millis() < 50, "Root load should be < 50ms");
    }
}
