// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { createSignal } from "solid-js";
import type { FFXProject } from "./useActivityTimeline";
import { logger } from "../utils/logger";
const log = logger.scope("ProjectRecovery");

/**
 * Backup type matching backend BackupType enum
 */
export type BackupType = "ManualSave" | "AutoSave" | "ManualBackup" | "PreOperation";

/**
 * Backup metadata from backend
 */
export interface BackupMetadata {
  original_path: string;
  created_at: string;
  app_version: string;
  file_size: number;
  backup_type: BackupType;
  user: string | null;
}

/**
 * Backup file with metadata from backend
 */
export interface BackupFile {
  path: string;
  metadata: BackupMetadata;
}

/**
 * Health status enum matching backend
 */
export type ProjectHealthStatus = "Healthy" | "Warning" | "Critical";

/**
 * Issue severity enum matching backend
 */
export type IssueSeverity = "Info" | "Warning" | "Error" | "Critical";

/**
 * Issue category enum matching backend
 */
export type IssueCategory = 
  | "FileSize" 
  | "ActivityLog" 
  | "MissingFiles" 
  | "Corruption" 
  | "Performance" 
  | "Security";

/**
 * Health issue from backend
 */
export interface HealthIssue {
  severity: IssueSeverity;
  category: IssueCategory;
  message: string;
  recommendation: string | null;
}

/**
 * Project health status from backend
 */
export interface ProjectHealth {
  status: ProjectHealthStatus;
  issues: HealthIssue[];
  checked_at: string;
  file_size: number;
  activity_log_size: number;
  tab_count: number;
  session_count: number;
  has_backup: boolean;
  version_count: number;
}

/**
 * Recovery info from backend
 */
export interface RecoveryInfo {
  has_autosave: boolean;
  autosave_path: string | null;
  autosave_age_seconds: number | null;
  autosave_is_newer: boolean;
  has_backup: boolean;
  backup_path: string | null;
}

/**
 * Hook for project recovery features (backup, restore, health monitoring)
 */
export function useProjectRecovery() {
  const [backups, setBackups] = createSignal<BackupFile[]>([]);
  const [health, setHealth] = createSignal<ProjectHealth | null>(null);
  const [versions, setVersions] = createSignal<BackupFile[]>([]);
  const [recoveryInfo, setRecoveryInfo] = createSignal<RecoveryInfo | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  /**
   * Create a backup of the project
   */
  const createBackup = async (
    projectPath: string,
    backupType: BackupType = "ManualBackup",
    user?: string
  ): Promise<string | null> => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<string>("project_create_backup", {
        project_path: projectPath,
        backup_type: backupType,
        user: user ?? null,
      });
      // Refresh version list after backup
      await listVersions(projectPath);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to create backup:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Create a versioned backup
   */
  const createVersionBackup = async (projectPath: string): Promise<string | null> => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<string>("project_create_version", {
        project_path: projectPath,
      });
      // Refresh version list
      await listVersions(projectPath);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to create version backup:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * List all version backups for a project
   */
  const listVersions = async (projectPath: string): Promise<BackupFile[]> => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<BackupFile[]>("project_list_versions", {
        project_path: projectPath,
      });
      setVersions(result);
      setBackups(result); // Also set backups for compatibility
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to list versions:", err);
      return [];
    } finally {
      setLoading(false);
    }
  };

  /**
   * Check recovery availability
   */
  const checkRecovery = async (projectPath: string): Promise<RecoveryInfo | null> => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<RecoveryInfo>("project_check_recovery", {
        project_path: projectPath,
      });
      setRecoveryInfo(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to check recovery:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Recover project from autosave
   */
  const recoverFromAutosave = async (projectPath: string): Promise<FFXProject | null> => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<FFXProject>("project_recover_autosave", {
        project_path: projectPath,
      });
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to recover from autosave:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Clear autosave file
   */
  const clearAutosave = async (projectPath: string): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("project_clear_autosave", {
        project_path: projectPath,
      });
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to clear autosave:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Check project health
   */
  const checkHealth = async (projectPath: string): Promise<ProjectHealth | null> => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<ProjectHealth>("project_check_health", {
        project_path: projectPath,
      });
      setHealth(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to check project health:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Helper to get health status color
   */
  const getHealthStatusColor = (status: ProjectHealthStatus): string => {
    switch (status) {
      case "Healthy":
        return "text-success";
      case "Warning":
        return "text-warning";
      case "Critical":
        return "text-error";
      default:
        return "text-txt-muted";
    }
  };

  /**
   * Helper to get severity color
   */
  const getSeverityColor = (severity: IssueSeverity): string => {
    switch (severity) {
      case "Info":
        return "text-info";
      case "Warning":
        return "text-warning";
      case "Error":
        return "text-error";
      case "Critical":
        return "text-error";
      default:
        return "text-txt-muted";
    }
  };

  return {
    // State
    backups,
    health,
    versions,
    recoveryInfo,
    loading,
    error,
    // Actions
    createBackup,
    createVersionBackup,
    listVersions,
    checkRecovery,
    recoverFromAutosave,
    clearAutosave,
    checkHealth,
    // Helpers
    getHealthStatusColor,
    getSeverityColor,
  };
}
