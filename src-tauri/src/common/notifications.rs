// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Desktop Notification System - Cross-platform native notifications
//!
//! Provides desktop notifications for operation completion, errors, and important
//! events using the notify-rust crate.

use notify_rust::{Notification, Timeout, Urgency};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error};

/// Notification type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// Informational notification
    Info,
    /// Success notification
    Success,
    /// Warning notification
    Warning,
    /// Error notification
    Error,
}

impl NotificationType {
    /// Get urgency level for notify-rust (reserved for future platform-specific features)
    #[allow(dead_code)]
    fn urgency(&self) -> Urgency {
        match self {
            Self::Info => Urgency::Normal,
            Self::Success => Urgency::Low,
            Self::Warning => Urgency::Normal,
            Self::Error => Urgency::Critical,
        }
    }
    
    /// Get default timeout (reserved for future customization)
    #[allow(dead_code)]
    fn timeout(&self) -> Timeout {
        match self {
            Self::Info => Timeout::Milliseconds(5000),
            Self::Success => Timeout::Milliseconds(4000),
            Self::Warning => Timeout::Milliseconds(8000),
            Self::Error => Timeout::Milliseconds(10000),
        }
    }
}

/// Notification manager
#[derive(Debug, Clone)]
pub struct NotificationManager {
    app_name: String,
    enabled: bool,
}

impl NotificationManager {
    /// Create new notification manager
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            enabled: true,
        }
    }
    
    /// Enable/disable notifications
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        debug!("Notifications {}", if enabled { "enabled" } else { "disabled" });
    }
    
    /// Show notification
    pub fn notify(
        &self,
        _notification_type: NotificationType,
        title: &str,
        message: &str,
    ) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }
        
        // Build notification (notify-rust v4 uses simpler API)
        let mut notification = Notification::new();
        notification.appname(&self.app_name);
        notification.summary(title);
        notification.body(message);
        
        // Platform-specific configuration
        #[cfg(target_os = "linux")]
        {
            notification.urgency(notification_type.urgency());
            notification.timeout(notification_type.timeout());
        }
        
        let result = notification.show();
        
        match result {
            Ok(_) => {
                debug!("Notification shown: {} - {}", title, message);
                Ok(())
            }
            Err(e) => {
                error!("Failed to show notification: {}", e);
                Err(format!("Notification error: {}", e))
            }
        }
    }
    
    /// Show info notification
    pub fn info(&self, title: &str, message: &str) -> Result<(), String> {
        self.notify(NotificationType::Info, title, message)
    }
    
    /// Show success notification
    pub fn success(&self, title: &str, message: &str) -> Result<(), String> {
        self.notify(NotificationType::Success, title, message)
    }
    
    /// Show warning notification
    pub fn warning(&self, title: &str, message: &str) -> Result<(), String> {
        self.notify(NotificationType::Warning, title, message)
    }
    
    /// Show error notification
    pub fn error(&self, title: &str, message: &str) -> Result<(), String> {
        self.notify(NotificationType::Error, title, message)
    }
    
    /// Notify operation completion
    pub fn operation_completed(&self, operation_name: &str, duration: Duration) -> Result<(), String> {
        let message = format!(
            "Operation completed in {:.2}s",
            duration.as_secs_f64()
        );
        self.success(operation_name, &message)
    }
    
    /// Notify operation failure
    pub fn operation_failed(&self, operation_name: &str, error: &str) -> Result<(), String> {
        let message = format!("Failed: {}", error);
        self.error(operation_name, &message)
    }
    
    /// Notify with progress information
    pub fn progress_milestone(
        &self,
        operation_name: &str,
        current: usize,
        total: usize,
    ) -> Result<(), String> {
        let percent = (current as f64 / total as f64 * 100.0) as u32;
        let message = format!("{}/{} completed ({}%)", current, total, percent);
        self.info(operation_name, &message)
    }
    
    /// Notify about interrupted operation available for recovery
    pub fn recovery_available(&self, operation_name: &str) -> Result<(), String> {
        self.warning(
            "Recovery Available",
            &format!("Interrupted operation '{}' can be resumed", operation_name),
        )
    }
}

/// Global notification manager (lazy static)
static NOTIFICATION_MANAGER: std::sync::LazyLock<parking_lot::RwLock<NotificationManager>> =
    std::sync::LazyLock::new(|| {
        parking_lot::RwLock::new(NotificationManager::new("CORE-FFX"))
    });

/// Get global notification manager
pub fn get_notification_manager() -> parking_lot::RwLockReadGuard<'static, NotificationManager> {
    NOTIFICATION_MANAGER.read()
}

/// Set notification enabled state globally
pub fn set_notifications_enabled(enabled: bool) {
    NOTIFICATION_MANAGER.write().set_enabled(enabled);
}

/// Show info notification (global)
pub fn notify_info(title: &str, message: &str) -> Result<(), String> {
    get_notification_manager().info(title, message)
}

/// Show success notification (global)
pub fn notify_success(title: &str, message: &str) -> Result<(), String> {
    get_notification_manager().success(title, message)
}

/// Show warning notification (global)
pub fn notify_warning(title: &str, message: &str) -> Result<(), String> {
    get_notification_manager().warning(title, message)
}

/// Show error notification (global)
pub fn notify_error(title: &str, message: &str) -> Result<(), String> {
    get_notification_manager().error(title, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_notification_manager_creation() {
        let manager = NotificationManager::new("TestApp");
        assert_eq!(manager.app_name, "TestApp");
        assert!(manager.enabled);
    }
    
    #[test]
    fn test_enable_disable() {
        let mut manager = NotificationManager::new("TestApp");
        manager.set_enabled(false);
        assert!(!manager.enabled);
        manager.set_enabled(true);
        assert!(manager.enabled);
    }
    
    #[test]
    fn test_disabled_notifications_ok() {
        let mut manager = NotificationManager::new("TestApp");
        manager.set_enabled(false);
        
        // Should return Ok even when disabled
        let result = manager.info("Test", "Message");
        assert!(result.is_ok());
    }
    
    // Note: Actual notification display tests would require a display server
    // and are better suited for integration tests
}
