// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Notification System Hook
 * 
 * Provides access to desktop notifications for operation completion, errors,
 * and important events.
 * 
 * @example
 * ```tsx
 * const notifications = useNotifications();
 * 
 * // Show success notification
 * await notifications.success("Hash Complete", "All files hashed successfully");
 * 
 * // Show error notification
 * await notifications.error("Extraction Failed", "Could not extract file");
 * 
 * // Notify operation completion with duration
 * await notifications.operationCompleted("Hash Batch", 5432); // 5.4 seconds
 * ```
 */

import { invoke } from "@tauri-apps/api/core";
import type { NotificationType, NotificationSystem } from "../types/recovery";

export function useNotifications(): NotificationSystem {
  const show = async (
    type: NotificationType,
    title: string,
    message: string
  ): Promise<void> => {
    await invoke("notification_show", { notificationType: type, title, message });
  };

  const info = async (title: string, message: string): Promise<void> => {
    await invoke("notification_info", { title, message });
  };

  const success = async (title: string, message: string): Promise<void> => {
    await invoke("notification_success", { title, message });
  };

  const warning = async (title: string, message: string): Promise<void> => {
    await invoke("notification_warning", { title, message });
  };

  const error = async (title: string, message: string): Promise<void> => {
    await invoke("notification_error", { title, message });
  };

  const setEnabled = async (enabled: boolean): Promise<void> => {
    await invoke("notification_set_enabled", { enabled });
  };

  const operationCompleted = async (
    operationName: string,
    durationMs: number
  ): Promise<void> => {
    await invoke("notification_operation_completed", {
      operationName,
      durationMs,
    });
  };

  const operationFailed = async (
    operationName: string,
    error: string
  ): Promise<void> => {
    await invoke("notification_operation_failed", {
      operationName,
      error,
    });
  };

  const progressMilestone = async (
    operationName: string,
    current: number,
    total: number
  ): Promise<void> => {
    await invoke("notification_progress_milestone", {
      operationName,
      current,
      total,
    });
  };

  const recoveryAvailable = async (operationName: string): Promise<void> => {
    await invoke("notification_recovery_available", { operationName });
  };

  return {
    show,
    info,
    success,
    warning,
    error,
    setEnabled,
    operationCompleted,
    operationFailed,
    progressMilestone,
    recoveryAvailable,
  };
}
