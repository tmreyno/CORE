// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Recovery & Notification Commands - Tauri IPC endpoints

use crate::common::{
    RecoveryManager, RecoverableOperation, OperationType, OperationState,
    RecoveryStats, create_operation, NotificationType,
    get_notification_manager, set_notifications_enabled,
};
use serde_json::Value;
use tauri::command;
use tracing::debug;
use std::sync::LazyLock;
use parking_lot::RwLock;

/// Global recovery manager
static RECOVERY_MANAGER: LazyLock<RwLock<Option<RecoveryManager>>> =
    LazyLock::new(|| RwLock::new(None));

/// Initialize recovery manager
fn get_recovery_manager() -> Result<RecoveryManager, String> {
    let manager_lock = RECOVERY_MANAGER.read();
    if let Some(manager) = manager_lock.as_ref() {
        Ok(manager.clone())
    } else {
        drop(manager_lock);
        
        // Initialize on first access
        let db_path = dirs::data_local_dir()
            .ok_or("Could not find data directory")?
            .join("CORE-FFX")
            .join("recovery.db");
        
        let manager = RecoveryManager::new(&db_path)
            .map_err(|e| format!("Failed to initialize recovery manager: {}", e))?;
        
        *RECOVERY_MANAGER.write() = Some(manager.clone());
        Ok(manager)
    }
}

/// Save operation state for recovery
#[command]
pub async fn recovery_save_operation(operation: RecoverableOperation) -> Result<(), String> {
    let manager = get_recovery_manager()?;
    manager.save_operation(&operation)
        .map_err(|e| format!("Failed to save operation: {}", e))?;
    
    debug!("Saved operation: {}", operation.id);
    Ok(())
}

/// Load operation by ID
#[command]
pub async fn recovery_load_operation(id: String) -> Result<RecoverableOperation, String> {
    let manager = get_recovery_manager()?;
    manager.load_operation(&id)
        .map_err(|e| format!("Failed to load operation: {}", e))
}

/// Get interrupted operations (Running or Paused)
#[command]
pub async fn recovery_get_interrupted() -> Result<Vec<RecoverableOperation>, String> {
    let manager = get_recovery_manager()?;
    manager.get_interrupted_operations()
        .map_err(|e| format!("Failed to get interrupted operations: {}", e))
}

/// Get operations by state
#[command]
pub async fn recovery_get_by_state(state: OperationState) -> Result<Vec<RecoverableOperation>, String> {
    let manager = get_recovery_manager()?;
    manager.get_operations_by_state(state)
        .map_err(|e| format!("Failed to get operations: {}", e))
}

/// Update operation progress
#[command]
pub async fn recovery_update_progress(id: String, progress: f64) -> Result<(), String> {
    let manager = get_recovery_manager()?;
    manager.update_progress(&id, progress)
        .map_err(|e| format!("Failed to update progress: {}", e))
}

/// Update operation state
#[command]
pub async fn recovery_update_state(id: String, state: OperationState) -> Result<(), String> {
    let manager = get_recovery_manager()?;
    manager.update_state(&id, state)
        .map_err(|e| format!("Failed to update state: {}", e))
}

/// Mark operation as failed
#[command]
pub async fn recovery_mark_failed(id: String, error_message: String) -> Result<(), String> {
    let manager = get_recovery_manager()?;
    manager.mark_failed(&id, &error_message)
        .map_err(|e| format!("Failed to mark as failed: {}", e))
}

/// Delete operation
#[command]
pub async fn recovery_delete_operation(id: String) -> Result<(), String> {
    let manager = get_recovery_manager()?;
    manager.delete_operation(&id)
        .map_err(|e| format!("Failed to delete operation: {}", e))
}

/// Clean up old operations
#[command]
pub async fn recovery_cleanup_old(days: u32) -> Result<usize, String> {
    let manager = get_recovery_manager()?;
    manager.cleanup_old_operations(days)
        .map_err(|e| format!("Failed to cleanup: {}", e))
}

/// Get recovery statistics
#[command]
pub async fn recovery_get_stats() -> Result<RecoveryStats, String> {
    let manager = get_recovery_manager()?;
    manager.get_stats()
        .map_err(|e| format!("Failed to get stats: {}", e))
}

/// Create new operation
#[command]
pub async fn recovery_create_operation(
    id: String,
    operation_type: OperationType,
    data: Value,
) -> Result<RecoverableOperation, String> {
    let operation = create_operation(id, operation_type, data);
    Ok(operation)
}

// =============================================================================
// Notification Commands
// =============================================================================

/// Show desktop notification
#[command]
pub async fn notification_show(
    notification_type: NotificationType,
    title: String,
    message: String,
) -> Result<(), String> {
    let manager = get_notification_manager();
    manager.notify(notification_type, &title, &message)
}

/// Show info notification
#[command]
pub async fn notification_info(title: String, message: String) -> Result<(), String> {
    let manager = get_notification_manager();
    manager.info(&title, &message)
}

/// Show success notification
#[command]
pub async fn notification_success(title: String, message: String) -> Result<(), String> {
    let manager = get_notification_manager();
    manager.success(&title, &message)
}

/// Show warning notification
#[command]
pub async fn notification_warning(title: String, message: String) -> Result<(), String> {
    let manager = get_notification_manager();
    manager.warning(&title, &message)
}

/// Show error notification
#[command]
pub async fn notification_error(title: String, message: String) -> Result<(), String> {
    let manager = get_notification_manager();
    manager.error(&title, &message)
}

/// Enable/disable notifications
#[command]
pub async fn notification_set_enabled(enabled: bool) -> Result<(), String> {
    set_notifications_enabled(enabled);
    debug!("Notifications {}", if enabled { "enabled" } else { "disabled" });
    Ok(())
}

/// Notify operation completed
#[command]
pub async fn notification_operation_completed(
    operation_name: String,
    duration_ms: u64,
) -> Result<(), String> {
    let manager = get_notification_manager();
    let duration = std::time::Duration::from_millis(duration_ms);
    manager.operation_completed(&operation_name, duration)
}

/// Notify operation failed
#[command]
pub async fn notification_operation_failed(
    operation_name: String,
    error: String,
) -> Result<(), String> {
    let manager = get_notification_manager();
    manager.operation_failed(&operation_name, &error)
}

/// Notify progress milestone
#[command]
pub async fn notification_progress_milestone(
    operation_name: String,
    current: usize,
    total: usize,
) -> Result<(), String> {
    let manager = get_notification_manager();
    manager.progress_milestone(&operation_name, current, total)
}

/// Notify recovery available
#[command]
pub async fn notification_recovery_available(operation_name: String) -> Result<(), String> {
    let manager = get_notification_manager();
    manager.recovery_available(&operation_name)
}
