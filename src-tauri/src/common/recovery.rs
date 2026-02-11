// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Error Recovery System - State persistence and graceful failure handling
//!
//! Provides automatic state persistence for long-running operations with the ability
//! to resume interrupted tasks. Uses SQLite for persistent storage and serde for
//! serialization.

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::{debug, info, warn};

/// Error types for recovery operations
#[derive(Error, Debug)]
pub enum RecoveryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Operation not found: {0}")]
    NotFound(String),
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

/// Result type for recovery operations
pub type RecoveryResult<T> = Result<T, RecoveryError>;

/// Operation types that support recovery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    /// Hashing operation (single file or batch)
    Hashing,
    /// Container extraction
    Extraction,
    /// File deduplication scan
    Deduplication,
    /// Index building
    Indexing,
    /// Report generation
    ReportGeneration,
    /// Archive extraction
    ArchiveExtraction,
}

/// Operation state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationState {
    /// Operation is queued but not started
    Pending,
    /// Operation is currently running
    Running,
    /// Operation paused by user
    Paused,
    /// Operation completed successfully
    Completed,
    /// Operation failed with error
    Failed,
    /// Operation cancelled by user
    Cancelled,
}

/// Recoverable operation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoverableOperation {
    /// Unique operation ID
    pub id: String,
    
    /// Operation type
    pub operation_type: OperationType,
    
    /// Current state
    pub state: OperationState,
    
    /// Operation-specific data (serialized JSON)
    pub data: serde_json::Value,
    
    /// Progress (0.0 to 1.0)
    pub progress: f64,
    
    /// Error message if failed
    pub error_message: Option<String>,
    
    /// Created timestamp (Unix epoch seconds)
    pub created_at: i64,
    
    /// Updated timestamp (Unix epoch seconds)
    pub updated_at: i64,
    
    /// Number of retry attempts
    pub retry_count: u32,
}

/// Recovery manager - persists and restores operation state
#[derive(Clone, Debug)]
pub struct RecoveryManager {
    db_path: PathBuf,
}

impl RecoveryManager {
    /// Create new recovery manager
    pub fn new(db_path: impl AsRef<Path>) -> RecoveryResult<Self> {
        let db_path = db_path.as_ref().to_path_buf();
        
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let manager = Self { db_path };
        manager.initialize_db()?;
        
        info!("Recovery manager initialized at {:?}", manager.db_path);
        Ok(manager)
    }
    
    /// Initialize database schema
    fn initialize_db(&self) -> RecoveryResult<()> {
        let conn = Connection::open(&self.db_path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS recoverable_operations (
                id TEXT PRIMARY KEY,
                operation_type TEXT NOT NULL,
                state TEXT NOT NULL,
                data TEXT NOT NULL,
                progress REAL NOT NULL DEFAULT 0.0,
                error_message TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                retry_count INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;
        
        // Index for quick lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_state ON recoverable_operations(state)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_type ON recoverable_operations(operation_type)",
            [],
        )?;
        
        debug!("Recovery database schema initialized");
        Ok(())
    }
    
    /// Save or update operation state
    pub fn save_operation(&self, operation: &RecoverableOperation) -> RecoveryResult<()> {
        let conn = Connection::open(&self.db_path)?;
        
        let data_json = serde_json::to_string(&operation.data)?;
        let op_type_str = serde_json::to_string(&operation.operation_type)?;
        let state_str = serde_json::to_string(&operation.state)?;
        
        conn.execute(
            "INSERT OR REPLACE INTO recoverable_operations 
             (id, operation_type, state, data, progress, error_message, created_at, updated_at, retry_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &operation.id,
                op_type_str,
                state_str,
                data_json,
                operation.progress,
                operation.error_message,
                operation.created_at,
                operation.updated_at,
                operation.retry_count,
            ],
        )?;
        
        debug!("Saved operation {} (state: {:?})", operation.id, operation.state);
        Ok(())
    }
    
    /// Load operation by ID
    pub fn load_operation(&self, id: &str) -> RecoveryResult<RecoverableOperation> {
        let conn = Connection::open(&self.db_path)?;
        
        let mut stmt = conn.prepare(
            "SELECT id, operation_type, state, data, progress, error_message, 
                    created_at, updated_at, retry_count
             FROM recoverable_operations WHERE id = ?1"
        )?;
        
        let operation = stmt.query_row(params![id], |row| {
            let op_type_str: String = row.get(1)?;
            let state_str: String = row.get(2)?;
            let data_str: String = row.get(3)?;
            
            Ok(RecoverableOperation {
                id: row.get(0)?,
                operation_type: serde_json::from_str(&op_type_str).expect("operation_type was serialized by us"),
                state: serde_json::from_str(&state_str).expect("state was serialized by us"),
                data: serde_json::from_str(&data_str).expect("data was serialized by us"),
                progress: row.get(4)?,
                error_message: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                retry_count: row.get(8)?,
            })
        })?;
        
        debug!("Loaded operation {}", id);
        Ok(operation)
    }
    
    /// Get all operations with specified state
    pub fn get_operations_by_state(&self, state: OperationState) -> RecoveryResult<Vec<RecoverableOperation>> {
        let conn = Connection::open(&self.db_path)?;
        let state_str = serde_json::to_string(&state)?;
        
        let mut stmt = conn.prepare(
            "SELECT id, operation_type, state, data, progress, error_message,
                    created_at, updated_at, retry_count
             FROM recoverable_operations WHERE state = ?1
             ORDER BY updated_at DESC"
        )?;
        
        let operations = stmt.query_map(params![state_str], |row| {
            let op_type_str: String = row.get(1)?;
            let state_str: String = row.get(2)?;
            let data_str: String = row.get(3)?;
            
            Ok(RecoverableOperation {
                id: row.get(0)?,
                operation_type: serde_json::from_str(&op_type_str).expect("operation_type was serialized by us"),
                state: serde_json::from_str(&state_str).expect("state was serialized by us"),
                data: serde_json::from_str(&data_str).expect("data was serialized by us"),
                progress: row.get(4)?,
                error_message: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                retry_count: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        debug!("Found {} operations with state {:?}", operations.len(), state);
        Ok(operations)
    }
    
    /// Get all interrupted operations (Running or Paused)
    pub fn get_interrupted_operations(&self) -> RecoveryResult<Vec<RecoverableOperation>> {
        let mut running = self.get_operations_by_state(OperationState::Running)?;
        let paused = self.get_operations_by_state(OperationState::Paused)?;
        running.extend(paused);
        
        info!("Found {} interrupted operations", running.len());
        Ok(running)
    }
    
    /// Update operation progress
    pub fn update_progress(&self, id: &str, progress: f64) -> RecoveryResult<()> {
        let conn = Connection::open(&self.db_path)?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("system clock after UNIX_EPOCH").as_secs() as i64;
        
        let rows = conn.execute(
            "UPDATE recoverable_operations SET progress = ?1, updated_at = ?2 WHERE id = ?3",
            params![progress, now, id],
        )?;
        
        if rows == 0 {
            return Err(RecoveryError::NotFound(id.to_string()));
        }
        
        Ok(())
    }
    
    /// Update operation state
    pub fn update_state(&self, id: &str, state: OperationState) -> RecoveryResult<()> {
        let conn = Connection::open(&self.db_path)?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("system clock after UNIX_EPOCH").as_secs() as i64;
        let state_str = serde_json::to_string(&state)?;
        
        let rows = conn.execute(
            "UPDATE recoverable_operations SET state = ?1, updated_at = ?2 WHERE id = ?3",
            params![state_str, now, id],
        )?;
        
        if rows == 0 {
            return Err(RecoveryError::NotFound(id.to_string()));
        }
        
        debug!("Updated operation {} state to {:?}", id, state);
        Ok(())
    }
    
    /// Mark operation as failed with error message
    pub fn mark_failed(&self, id: &str, error_msg: &str) -> RecoveryResult<()> {
        let conn = Connection::open(&self.db_path)?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("system clock after UNIX_EPOCH").as_secs() as i64;
        let state_str = serde_json::to_string(&OperationState::Failed)?;
        
        // Increment retry count
        conn.execute(
            "UPDATE recoverable_operations 
             SET state = ?1, error_message = ?2, updated_at = ?3, retry_count = retry_count + 1
             WHERE id = ?4",
            params![state_str, error_msg, now, id],
        )?;
        
        warn!("Operation {} failed: {}", id, error_msg);
        Ok(())
    }
    
    /// Delete operation from database
    pub fn delete_operation(&self, id: &str) -> RecoveryResult<()> {
        let conn = Connection::open(&self.db_path)?;
        
        let rows = conn.execute(
            "DELETE FROM recoverable_operations WHERE id = ?1",
            params![id],
        )?;
        
        if rows == 0 {
            return Err(RecoveryError::NotFound(id.to_string()));
        }
        
        debug!("Deleted operation {}", id);
        Ok(())
    }
    
    /// Clean up old completed/failed operations (older than days)
    pub fn cleanup_old_operations(&self, days: u32) -> RecoveryResult<usize> {
        let conn = Connection::open(&self.db_path)?;
        let cutoff = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after UNIX_EPOCH")
            .as_secs() as i64
            - (days as i64 * 86400);
        
        let completed_str = serde_json::to_string(&OperationState::Completed)?;
        let failed_str = serde_json::to_string(&OperationState::Failed)?;
        let cancelled_str = serde_json::to_string(&OperationState::Cancelled)?;
        
        let rows = conn.execute(
            "DELETE FROM recoverable_operations 
             WHERE (state = ?1 OR state = ?2 OR state = ?3) AND updated_at < ?4",
            params![completed_str, failed_str, cancelled_str, cutoff],
        )?;
        
        info!("Cleaned up {} old operations (older than {} days)", rows, days);
        Ok(rows)
    }
    
    /// Get recovery statistics
    pub fn get_stats(&self) -> RecoveryResult<RecoveryStats> {
        let conn = Connection::open(&self.db_path)?;
        
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM recoverable_operations",
            [],
            |row| row.get(0)
        )?;
        
        let pending_str = serde_json::to_string(&OperationState::Pending)?;
        let pending: i64 = conn.query_row(
            "SELECT COUNT(*) FROM recoverable_operations WHERE state = ?1",
            params![pending_str],
            |row| row.get(0)
        )?;
        
        let running_str = serde_json::to_string(&OperationState::Running)?;
        let running: i64 = conn.query_row(
            "SELECT COUNT(*) FROM recoverable_operations WHERE state = ?1",
            params![running_str],
            |row| row.get(0)
        )?;
        
        let completed_str = serde_json::to_string(&OperationState::Completed)?;
        let completed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM recoverable_operations WHERE state = ?1",
            params![completed_str],
            |row| row.get(0)
        )?;
        
        let failed_str = serde_json::to_string(&OperationState::Failed)?;
        let failed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM recoverable_operations WHERE state = ?1",
            params![failed_str],
            |row| row.get(0)
        )?;
        
        Ok(RecoveryStats {
            total_operations: total as usize,
            pending: pending as usize,
            running: running as usize,
            completed: completed as usize,
            failed: failed as usize,
        })
    }
}

/// Recovery statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryStats {
    pub total_operations: usize,
    pub pending: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
}

/// Helper to create a new operation
pub fn create_operation(
    id: String,
    operation_type: OperationType,
    data: serde_json::Value,
) -> RecoverableOperation {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after UNIX_EPOCH")
        .as_secs() as i64;
    
    RecoverableOperation {
        id,
        operation_type,
        state: OperationState::Pending,
        data,
        progress: 0.0,
        error_message: None,
        created_at: now,
        updated_at: now,
        retry_count: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_recovery_manager_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("recovery.db");
        
        let manager = RecoveryManager::new(&db_path).unwrap();
        
        // Create operation
        let op = create_operation(
            "test-op-1".to_string(),
            OperationType::Hashing,
            serde_json::json!({"file": "/path/to/file.bin"}),
        );
        
        // Save operation
        manager.save_operation(&op).unwrap();
        
        // Load operation
        let loaded = manager.load_operation("test-op-1").unwrap();
        assert_eq!(loaded.id, "test-op-1");
        assert_eq!(loaded.state, OperationState::Pending);
        
        // Update progress
        manager.update_progress("test-op-1", 0.5).unwrap();
        let loaded = manager.load_operation("test-op-1").unwrap();
        assert_eq!(loaded.progress, 0.5);
        
        // Update state
        manager.update_state("test-op-1", OperationState::Running).unwrap();
        let loaded = manager.load_operation("test-op-1").unwrap();
        assert_eq!(loaded.state, OperationState::Running);
        
        // Mark as completed
        manager.update_state("test-op-1", OperationState::Completed).unwrap();
        
        // Get stats
        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.completed, 1);
    }
    
    #[test]
    fn test_get_interrupted_operations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("recovery.db");
        let manager = RecoveryManager::new(&db_path).unwrap();
        
        // Create multiple operations
        let mut op1 = create_operation(
            "op1".to_string(),
            OperationType::Extraction,
            serde_json::json!({}),
        );
        op1.state = OperationState::Running;
        manager.save_operation(&op1).unwrap();
        
        let mut op2 = create_operation(
            "op2".to_string(),
            OperationType::Hashing,
            serde_json::json!({}),
        );
        op2.state = OperationState::Paused;
        manager.save_operation(&op2).unwrap();
        
        let mut op3 = create_operation(
            "op3".to_string(),
            OperationType::Indexing,
            serde_json::json!({}),
        );
        op3.state = OperationState::Completed;
        manager.save_operation(&op3).unwrap();
        
        // Get interrupted operations
        let interrupted = manager.get_interrupted_operations().unwrap();
        assert_eq!(interrupted.len(), 2);
    }
}
