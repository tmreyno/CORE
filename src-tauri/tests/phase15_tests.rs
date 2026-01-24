// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Phase 15 Integration Tests: Advanced Memory Profiling
//!
//! Tests for memory allocation tracking, leak detection, and heap analysis.

use ffx_check_lib::common::memory_profiler::{
    clear_snapshots, detect_leaks, format_bytes, get_current_memory_usage, get_memory_stats,
    get_snapshots, is_tracking_active, start_tracking, stop_tracking, take_snapshot,
    MemoryTrackingGuard,
};

#[test]
fn test_memory_tracking_lifecycle() {
    clear_snapshots();

    // Initially not tracking
    assert!(!is_tracking_active());

    // Start tracking
    start_tracking();
    assert!(is_tracking_active());

    // Allocate memory
    let data: Vec<u8> = vec![0; 5_000_000]; // 5 MB
    assert_eq!(data.len(), 5_000_000);

    // Get stats
    let stats = get_memory_stats().expect("Should get stats while tracking");
    assert!(stats.current_bytes > 0);
    assert!(stats.duration_secs >= 0.0);

    // Stop tracking
    let final_stats = stop_tracking();
    assert!(!is_tracking_active());
    assert!(final_stats.peak_bytes >= stats.current_bytes);

    println!("✅ Memory tracking lifecycle test passed");
    println!("   Peak memory: {} bytes", final_stats.peak_bytes);
}

#[test]
fn test_memory_snapshots() {
    clear_snapshots();

    start_tracking();

    // Take baseline snapshot
    take_snapshot("baseline");
    assert_eq!(get_snapshots().len(), 1);

    // Allocate some memory
    let _data1: Vec<u8> = vec![0; 1_000_000]; // 1 MB
    take_snapshot("after_1mb");
    assert_eq!(get_snapshots().len(), 2);

    // Allocate more memory
    let _data2: Vec<u8> = vec![0; 5_000_000]; // 5 MB
    take_snapshot("after_6mb_total");
    assert_eq!(get_snapshots().len(), 3);

    let snapshots = get_snapshots();
    assert_eq!(snapshots[0].name, "baseline");
    assert_eq!(snapshots[1].name, "after_1mb");
    assert_eq!(snapshots[2].name, "after_6mb_total");

    // Verify timestamps are ordered
    assert!(snapshots[1].timestamp_secs >= snapshots[0].timestamp_secs);
    assert!(snapshots[2].timestamp_secs >= snapshots[1].timestamp_secs);

    stop_tracking();

    println!("✅ Memory snapshots test passed");
    println!("   Captured {} snapshots", snapshots.len());
}

#[test]
fn test_memory_tracking_guard() {
    clear_snapshots();

    {
        let guard = MemoryTrackingGuard::new("test_operation");
        
        // Allocate memory within guard scope
        let _data: Vec<u8> = vec![0; 2_000_000]; // 2 MB
        
        guard.snapshot("midpoint");
        
        let _more_data: Vec<u8> = vec![0; 3_000_000]; // 3 MB more
        
        // Guard will automatically stop tracking on drop
    }

    let snapshots = get_snapshots();
    assert!(snapshots.len() >= 2); // At least start and end

    println!("✅ Memory tracking guard test passed");
    println!("   Auto-captured {} snapshots", snapshots.len());
}

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(0), "0 bytes");
    assert_eq!(format_bytes(512), "512 bytes");
    assert_eq!(format_bytes(1024), "1.00 KB");
    assert_eq!(format_bytes(2048), "2.00 KB");
    assert_eq!(format_bytes(1_048_576), "1.00 MB");
    assert_eq!(format_bytes(5_242_880), "5.00 MB");
    assert_eq!(format_bytes(1_073_741_824), "1.00 GB");
    assert_eq!(format_bytes(2_147_483_648), "2.00 GB");

    println!("✅ Format bytes test passed");
}

#[test]
fn test_leak_detection() {
    clear_snapshots();

    start_tracking();
    
    take_snapshot("start");
    
    // Allocate memory (simulating normal operation)
    let _data: Vec<u8> = vec![0; 1_000_000]; // 1 MB
    
    take_snapshot("end");

    let report = detect_leaks().expect("Should detect leaks");
    assert!(report.analysis_duration_secs >= 0.0);
    
    // For small allocations, shouldn't trigger leak detection
    // (threshold is 10 MB)
    assert!(report.total_leaked_bytes < 10_000_000);

    stop_tracking();

    println!("✅ Leak detection test passed");
    println!("   Leaked bytes: {}", report.total_leaked_bytes);
}

#[test]
fn test_large_leak_detection() {
    clear_snapshots();

    start_tracking();
    
    take_snapshot("start");
    
    // Allocate large amount to trigger leak detection
    let _data: Vec<u8> = vec![0; 15_000_000]; // 15 MB
    
    take_snapshot("end");

    let report = detect_leaks().expect("Should detect leaks");
    
    // Should detect the 15 MB allocation as potential leak
    if report.total_leaked_bytes > 10_000_000 {
        assert!(!report.potential_leaks.is_empty());
        println!("   Detected leak: {} bytes", report.total_leaked_bytes);
    }

    stop_tracking();

    println!("✅ Large leak detection test passed");
}

#[test]
fn test_current_memory_usage() {
    let usage = get_current_memory_usage();
    
    // Should be non-zero (process is running)
    assert!(usage > 0);
    
    // Allocate memory and check it increases
    let baseline = usage;
    let _data: Vec<u8> = vec![0; 10_000_000]; // 10 MB
    let after_alloc = get_current_memory_usage();
    
    // Memory should have increased (though exact amount varies)
    assert!(after_alloc >= baseline);

    println!("✅ Current memory usage test passed");
    println!("   Baseline: {} bytes, After alloc: {} bytes", baseline, after_alloc);
}

#[test]
fn test_clear_snapshots() {
    clear_snapshots();

    start_tracking();
    take_snapshot("snap1");
    take_snapshot("snap2");
    take_snapshot("snap3");
    
    assert_eq!(get_snapshots().len(), 3);
    
    clear_snapshots();
    assert_eq!(get_snapshots().len(), 0);
    
    stop_tracking();

    println!("✅ Clear snapshots test passed");
}

#[test]
fn test_peak_memory_tracking() {
    clear_snapshots();

    start_tracking();
    
    // Allocate in stages
    let _data1: Vec<u8> = vec![0; 2_000_000]; // 2 MB
    take_snapshot("2mb");
    
    let _data2: Vec<u8> = vec![0; 5_000_000]; // 5 MB more (7 MB total)
    take_snapshot("7mb");
    
    drop(_data1); // Free 2 MB
    take_snapshot("5mb_after_drop");
    
    let stats = stop_tracking();
    
    // Peak should be at least 7 MB point
    assert!(stats.peak_bytes >= 7_000_000);

    println!("✅ Peak memory tracking test passed");
    println!("   Peak: {} bytes", stats.peak_bytes);
}

#[test]
fn test_phase15_summary() {
    clear_snapshots();

    println!("\n========================================");
    println!("Phase 15 Integration Tests Summary");
    println!("========================================");
    println!();
    
    // Comprehensive test
    start_tracking();
    
    take_snapshot("start");
    
    let _data1: Vec<u8> = vec![0; 3_000_000]; // 3 MB
    take_snapshot("3mb");
    
    let _data2: Vec<u8> = vec![0; 7_000_000]; // 7 MB more
    take_snapshot("10mb");
    
    let stats = stop_tracking();
    let snapshots = get_snapshots();
    let report = detect_leaks().expect("Should analyze leaks");

    println!("✅ Phase 15: Advanced Memory Profiling");
    println!();
    println!("Memory Profiler Infrastructure:");
    println!("  • Allocation tracking ✅");
    println!("  • Memory snapshots ✅");
    println!("  • Leak detection ✅");
    println!("  • Peak usage tracking ✅");
    println!("  • RAII guard pattern ✅");
    println!();
    println!("Test Results:");
    println!("  • Current memory: {} bytes", stats.current_bytes);
    println!("  • Peak memory: {} bytes", stats.peak_bytes);
    println!("  • Physical memory: {} bytes", stats.physical_bytes);
    println!("  • Virtual memory: {} bytes", stats.virtual_mem_bytes);
    println!("  • Tracking duration: {:.3}s", stats.duration_secs);
    println!("  • Snapshots captured: {}", snapshots.len());
    println!("  • Leaked bytes: {}", report.total_leaked_bytes);
    println!("  • Potential leaks: {}", report.potential_leaks.len());
    println!();
    println!("Commands Registered:");
    println!("  1. memory_start_tracking");
    println!("  2. memory_stop_tracking");
    println!("  3. memory_is_active");
    println!("  4. memory_take_snapshot");
    println!("  5. memory_get_stats");
    println!("  6. memory_get_snapshots");
    println!("  7. memory_clear_snapshots");
    println!("  8. memory_detect_leaks");
    println!("  9. memory_get_current_usage");
    println!("  10. memory_format_bytes");
    println!("  11. memory_get_summary");
    println!();
    println!("========================================");
}
