// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Phase 11 Integration Tests: Error Recovery & Notifications
//!
//! Tests:
//! 1. State persistence (create/save/load operations)
//! 2. Retry logic (exponential backoff with jitter)
//! 3. Operation lifecycle (pending -> running -> completed/failed)

use tempfile::TempDir;
use std::time::Duration;

// Import Phase 11 modules
use ffx_check_lib::common::recovery::{
    RecoveryManager, OperationState, OperationType, create_operation
};
use ffx_check_lib::common::retry::{RetryConfig, retry_async};

/// Helper: Create test recovery manager
fn create_recovery_manager() -> (RecoveryManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let state_file = temp_dir.path().join("recovery.db");
    let manager = RecoveryManager::new(&state_file).unwrap();
    (manager, temp_dir)
}

// ============================================================================
// Test 1: Create and Persist Operations
// ============================================================================

#[tokio::test]
async fn test_create_and_persist_operations() {
    println!("\n=== Phase 11: Create & Persist Operations Test ===");
    
    let (manager, _temp_dir) = create_recovery_manager();
    
    // Create 3 operations
    let op1 = create_operation(
        "hash-001".to_string(),
        OperationType::Hashing,
        serde_json::json!({"file": "test.ad1", "algorithm": "SHA-256"}),
    );
    
    let op2 = create_operation(
        "extract-001".to_string(),
        OperationType::Extraction,
        serde_json::json!({"path": "/data/output", "format": "AD1"}),
    );
    
    let op3 = create_operation(
        "dedup-001".to_string(),
        OperationType::Deduplication,
        serde_json::json!({"files": 500, "threshold": 0.95}),
    );
    
    println!("📝 Saving 3 operations...");
    
    manager.save_operation(&op1).unwrap();
    manager.save_operation(&op2).unwrap();
    manager.save_operation(&op3).unwrap();
    
    println!("✅ Saved 3 operations");
    
    // Load operations
    let loaded1 = manager.load_operation("hash-001").unwrap();
    let loaded2 = manager.load_operation("extract-001").unwrap();
    let loaded3 = manager.load_operation("dedup-001").unwrap();
    
    println!("📖 Loaded 3 operations successfully");
    
    // Verify types
    assert_eq!(loaded1.operation_type, OperationType::Hashing);
    assert_eq!(loaded2.operation_type, OperationType::Extraction);
    assert_eq!(loaded3.operation_type, OperationType::Deduplication);
    
    // Verify initial state
    assert_eq!(loaded1.state, OperationState::Pending);
    assert_eq!(loaded2.state, OperationState::Pending);
    assert_eq!(loaded3.state, OperationState::Pending);
    
    println!("✅ Content verification passed");
    println!("✅ Create & persist test PASSED\n");
}

// ============================================================================
// Test 2: Update Operation Progress
// ============================================================================

#[tokio::test]
async fn test_update_operation_progress() {
    println!("\n=== Phase 11: Update Progress Test ===");
    
    let (manager, _temp_dir) = create_recovery_manager();
    
    let op = create_operation(
        "progress-test".to_string(),
        OperationType::Hashing,
        serde_json::json!({"file": "large_file.ad1"}),
    );
    
    manager.save_operation(&op).unwrap();
    
    println!("📊 Simulating progress updates...");
    
    // Simulate progress: 0% -> 25% -> 50% -> 75% -> 100%
    for progress in [0.0, 0.25, 0.5, 0.75, 1.0] {
        manager.update_progress("progress-test", progress).unwrap();
        println!("   Progress: {}%", (progress * 100.0) as u32);
    }
    
    let final_op = manager.load_operation("progress-test").unwrap();
    assert_eq!(final_op.progress, 1.0);
    
    println!("✅ Final progress: 100%");
    println!("✅ Update progress test PASSED\n");
}

// ============================================================================
// Test 3: Operation State Transitions
// ============================================================================

#[tokio::test]
async fn test_operation_state_transitions() {
    println!("\n=== Phase 11: State Transitions Test ===");
    
    let (manager, _temp_dir) = create_recovery_manager();
    
    let op = create_operation(
        "state-test".to_string(),
        OperationType::Extraction,
        serde_json::json!({}),
    );
    
    manager.save_operation(&op).unwrap();
    
    println!("🔄 Testing state transitions:");
    
    // Pending -> Running
    manager.update_state("state-test", OperationState::Running).unwrap();
    let op1 = manager.load_operation("state-test").unwrap();
    assert_eq!(op1.state, OperationState::Running);
    println!("   ✓ Pending → Running");
    
    // Running -> Paused
    manager.update_state("state-test", OperationState::Paused).unwrap();
    let op2 = manager.load_operation("state-test").unwrap();
    assert_eq!(op2.state, OperationState::Paused);
    println!("   ✓ Running → Paused");
    
    // Paused -> Running
    manager.update_state("state-test", OperationState::Running).unwrap();
    let op3 = manager.load_operation("state-test").unwrap();
    assert_eq!(op3.state, OperationState::Running);
    println!("   ✓ Paused → Running");
    
    // Running -> Completed
    manager.update_state("state-test", OperationState::Completed).unwrap();
    let op4 = manager.load_operation("state-test").unwrap();
    assert_eq!(op4.state, OperationState::Completed);
    println!("   ✓ Running → Completed");
    
    println!("✅ State transitions test PASSED\n");
}

// ============================================================================
// Test 4: Query Operations by State
// ============================================================================

#[tokio::test]
async fn test_query_operations_by_state() {
    println!("\n=== Phase 11: Query by State Test ===");
    
    let (manager, _temp_dir) = create_recovery_manager();
    
    // Create operations in different states
    for i in 0..3 {
        let op = create_operation(
            format!("running-{}", i),
            OperationType::Hashing,
            serde_json::json!({}),
        );
        manager.save_operation(&op).unwrap();
        manager.update_state(&format!("running-{}", i), OperationState::Running).unwrap();
    }
    
    for i in 0..2 {
        let op = create_operation(
            format!("completed-{}", i),
            OperationType::Extraction,
            serde_json::json!({}),
        );
        manager.save_operation(&op).unwrap();
        manager.update_state(&format!("completed-{}", i), OperationState::Completed).unwrap();
    }
    
    println!("📝 Created 3 running + 2 completed operations");
    
    // Query by state
    let running_ops = manager.get_operations_by_state(OperationState::Running).unwrap();
    let completed_ops = manager.get_operations_by_state(OperationState::Completed).unwrap();
    
    println!("📊 Query results:");
    println!("   Running: {} operations", running_ops.len());
    println!("   Completed: {} operations", completed_ops.len());
    
    assert_eq!(running_ops.len(), 3);
    assert_eq!(completed_ops.len(), 2);
    
    println!("✅ Query by state test PASSED\n");
}

// ============================================================================
// Test 5: Retry Logic - Exponential Backoff
// ============================================================================

#[tokio::test]
async fn test_retry_exponential_backoff() {
    println!("\n=== Phase 11: Retry Exponential Backoff Test ===");
    
    let config = RetryConfig {
        max_attempts: 4,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        backoff_multiplier: 2.0,
        jitter_factor: 0.1,
    };
    
    println!("🔄 Testing retry with config:");
    println!("   Max attempts: {}", config.max_attempts);
    println!("   Initial delay: {:?}", config.initial_delay);
    println!("   Backoff multiplier: {}", config.backoff_multiplier);
    println!("   Jitter factor: {}", config.jitter_factor);
    
    // Use a shared counter with Arc
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    
    let attempt_count = Arc::new(AtomicU32::new(0));
    let attempt_clone = attempt_count.clone();
    
    let result: Result<String, String> = retry_async(config, "test_operation", move || {
        let attempt_clone = attempt_clone.clone();
        async move {
            let current = attempt_clone.fetch_add(1, Ordering::SeqCst) + 1;
            println!("   Attempt {}", current);
            
            // Fail first 3 attempts, succeed on 4th
            if current < 4 {
                Err("Simulated transient error".to_string())
            } else {
                Ok("Success!".to_string())
            }
        }
    }).await;
    
    assert!(result.is_ok(), "Should succeed after retries");
    assert_eq!(attempt_count.load(Ordering::SeqCst), 4, "Should make exactly 4 attempts");
    
    println!("✅ Exponential backoff test PASSED\n");
}

// ============================================================================
// Test 6: Retry Logic - Max Attempts Exhausted
// ============================================================================

#[tokio::test]
async fn test_retry_max_attempts_exhausted() {
    println!("\n=== Phase 11: Retry Max Attempts Test ===");
    
    let config = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(5),
        max_delay: Duration::from_millis(50),
        backoff_multiplier: 2.0,
        jitter_factor: 0.0, // No jitter for predictability
    };
    
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    
    let attempt_count = Arc::new(AtomicU32::new(0));
    let attempt_clone = attempt_count.clone();
    
    println!("🔄 Testing retry exhaustion (max {} attempts)...", config.max_attempts);
    
    let result: Result<String, String> = retry_async(config, "persistent_failure", move || {
        let attempt_clone = attempt_clone.clone();
        async move {
            let current = attempt_clone.fetch_add(1, Ordering::SeqCst) + 1;
            println!("   Attempt {}", current);
            // Always fail
            Err("Persistent failure".to_string())
        }
    }).await;
    
    assert!(result.is_err(), "Should fail after max attempts");
    // max_attempts=3 means: initial attempt + 3 retries = 4 total attempts
    assert_eq!(attempt_count.load(Ordering::SeqCst), 4, "Should make exactly 4 attempts (1 initial + 3 retries)");
    
    println!("❌ Failed after {} attempts (expected)", attempt_count.load(Ordering::SeqCst));
    println!("✅ Max attempts test PASSED\n");
}

// ============================================================================
// Test 7: Mark Operation as Failed
// ============================================================================

#[tokio::test]
async fn test_mark_operation_failed() {
    println!("\n=== Phase 11: Mark Failed Test ===");
    
    let (manager, _temp_dir) = create_recovery_manager();
    
    let op = create_operation(
        "fail-test".to_string(),
        OperationType::Hashing,
        serde_json::json!({}),
    );
    
    manager.save_operation(&op).unwrap();
    manager.update_state("fail-test", OperationState::Running).unwrap();
    
    println!("❌ Marking operation as failed...");
    
    manager.mark_failed("fail-test", "File not found").unwrap();
    
    let failed_op = manager.load_operation("fail-test").unwrap();
    
    println!("📊 Failed operation:");
    println!("   State: {:?}", failed_op.state);
    println!("   Error: {}", failed_op.error_message.as_ref().unwrap());
    println!("   Retry count: {}", failed_op.retry_count);
    
    assert_eq!(failed_op.state, OperationState::Failed);
    assert_eq!(failed_op.error_message.as_deref(), Some("File not found"));
    assert_eq!(failed_op.retry_count, 1);
    
    println!("✅ Mark failed test PASSED\n");
}

// ============================================================================
// Test 8: Get Recovery Stats
// ============================================================================

#[tokio::test]
async fn test_get_recovery_stats() {
    println!("\n=== Phase 11: Recovery Stats Test ===");
    
    let (manager, _temp_dir) = create_recovery_manager();
    
    // Create operations in various states
    let states = vec![
        (OperationState::Pending, 2),
        (OperationState::Running, 3),
        (OperationState::Completed, 4),
        (OperationState::Failed, 1),
    ];
    
    let mut op_counter = 0;
    for (state, count) in &states {
        for _ in 0..*count {
            let op = create_operation(
                format!("op-{}", op_counter),
                OperationType::Hashing,
                serde_json::json!({}),
            );
            manager.save_operation(&op).unwrap();
            manager.update_state(&format!("op-{}", op_counter), state.clone()).unwrap();
            op_counter += 1;
        }
    }
    
    println!("📝 Created 10 operations in various states");
    
    let stats = manager.get_stats().unwrap();
    
    println!("\n📊 Recovery Statistics:");
    println!("   Total operations: {}", stats.total_operations);
    println!("   Pending: {}", stats.pending);
    println!("   Running: {}", stats.running);
    println!("   Completed: {}", stats.completed);
    println!("   Failed: {}", stats.failed);
    
    assert_eq!(stats.total_operations, 10);
    assert_eq!(stats.pending, 2);
    assert_eq!(stats.running, 3);
    assert_eq!(stats.completed, 4);
    assert_eq!(stats.failed, 1);
    
    println!("✅ Recovery stats test PASSED\n");
}

// ============================================================================
// Test 9: File Persistence Across Restarts
// ============================================================================

#[tokio::test]
async fn test_file_persistence_across_restarts() {
    println!("\n=== Phase 11: File Persistence Test ===");
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("recovery.db");
    
    // Phase 1: Create and save operations
    {
        println!("🔧 Phase 1: Creating manager and saving operations...");
        let manager = RecoveryManager::new(&db_path).unwrap();
        
        let op1 = create_operation(
            "persist-1".to_string(),
            OperationType::Hashing,
            serde_json::json!({"progress": 65.5}),
        );
        
        let op2 = create_operation(
            "persist-2".to_string(),
            OperationType::Extraction,
            serde_json::json!({"progress": 42.0}),
        );
        
        manager.save_operation(&op1).unwrap();
        manager.save_operation(&op2).unwrap();
        manager.update_progress("persist-1", 0.655).unwrap();
        manager.update_progress("persist-2", 0.420).unwrap();
        
        println!("💾 Saved 2 operations with progress");
        println!("🛑 Dropping manager (simulates shutdown)...");
    } // Manager dropped here
    
    assert!(db_path.exists(), "Database should persist");
    println!("✅ Database file persisted");
    
    // Phase 2: Restore from file
    {
        println!("\n🔧 Phase 2: Creating new manager from persisted database...");
        let manager = RecoveryManager::new(&db_path).unwrap();
        
        let loaded1 = manager.load_operation("persist-1").unwrap();
        let loaded2 = manager.load_operation("persist-2").unwrap();
        
        println!("📖 Loaded 2 operations:");
        println!("   Op1 progress: {}%", (loaded1.progress * 100.0) as u32);
        println!("   Op2 progress: {}%", (loaded2.progress * 100.0) as u32);
        
        assert_eq!((loaded1.progress * 1000.0) as u32, 655);
        assert_eq!((loaded2.progress * 1000.0) as u32, 420);
        
        println!("✅ All operations restored correctly");
    }
    
    println!("✅ File persistence test PASSED\n");
}

// ============================================================================
// Test 10: Summary - Phase 11 Complete
// ============================================================================

#[tokio::test]
async fn test_phase11_summary() {
    println!("\n=== Phase 11: Summary Test ===");
    
    let (manager, _temp_dir) = create_recovery_manager();
    
    println!("📊 Testing complete recovery workflow:");
    
    // 1. Create operation
    let op = create_operation(
        "workflow-test".to_string(),
        OperationType::Hashing,
        serde_json::json!({"file": "test.ad1", "size": 1073741824}),
    );
    manager.save_operation(&op).unwrap();
    println!("   ✓ Created operation");
    
    // 2. Start operation
    manager.update_state("workflow-test", OperationState::Running).unwrap();
    println!("   ✓ Started operation");
    
    // 3. Update progress
    for progress in [0.2, 0.4, 0.6, 0.8, 1.0] {
        manager.update_progress("workflow-test", progress).unwrap();
    }
    println!("   ✓ Updated progress (100%)");
    
    // 4. Mark complete
    manager.update_state("workflow-test", OperationState::Completed).unwrap();
    println!("   ✓ Marked completed");
    
    // 5. Verify final state
    let final_op = manager.load_operation("workflow-test").unwrap();
    assert_eq!(final_op.state, OperationState::Completed);
    assert_eq!(final_op.progress, 1.0);
    
    println!("\n✅ Complete workflow test PASSED");
    println!("✅ Phase 11 integration tests complete!\n");
}
