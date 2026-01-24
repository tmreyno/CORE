// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { createSignal, createEffect } from "solid-js";

/**
 * Backup file metadata from backend
 */
export interface BackupFile {
  path: string;
  metadata: BackupMetadata;
}

export interface BackupMetadata {
  backup_type: "auto" | "manual" | "pre_operation";
  original_project_path: string;
  created_at: string;
  project_version: number;
  project_name: string;
  file_size: number;
  retained: boolean;
}

/**
 * Project health status from backend
 */
export interface ProjectHealth {
  status: "healthy" | "warning" | "critical";
  score: number;
  checks: HealthCheck[];
  recommendations: string[];
  last_checked: string;
}

export interface HealthCheck {
  name: string;
  status: "pass" | "warning" | "fail";
  message: string;
  severity: "info" | "warning" | "error";
}

/**
 * Version entry for project history
 */
export interface VersionEntry {
  version: number;
  timestamp: string;
  backup_path: string;
  description: string;
}

/**
 * Hook for project recovery features (backup, restore, health monitoring)
 */
export function useProjectRecovery() {
  const [backups, setBackups] = createSignal<BackupFile[]>([]);
  const [health, setHealth] = createSignal<ProjectHealth | null>(null);
  const [versions, setVersions] = createSignal<VersionEntry[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  /**
   * List all backups for a project
   */
  const listBackups = async (projectPath: string) => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<BackupFile[]>("project_list_backups", {
        projectPath,
      });
      setBackups(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to list backups:", err);
      return [];
    } finally {
      setLoading(false);
    }
  };

  /**
   * Restore project from a backup
   */
  const restoreBackup = async (
    backupPath: string,
    targetPath: string
  ): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("project_restore_backup", {
        backupPath,
        targetPath,
      });
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to restore backup:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Create manual backup
   */
  const createBackup = async (projectPath: string): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("project_create_backup", {
        projectPath,
        backupType: "manual",
      });
      // Refresh backup list
      await listBackups(projectPath);
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to create backup:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Check project health
   */
  const checkHealth = async (projectPath: string) => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<ProjectHealth>("project_check_health", {
        projectPath,
      });
      setHealth(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to check project health:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * List version history
   */
  const listVersions = async (projectPath: string) => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<VersionEntry[]>("project_list_versions", {
        projectPath,
      });
      setVersions(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to list versions:", err);
      return [];
    } finally {
      setLoading(false);
    }
  };

  /**
   * Clean old backups
   */
  const cleanBackups = async (
    projectPath: string,
    keepCount: number = 10
  ): Promise<number> => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<number>("project_clean_backups", {
        projectPath,
        keepCount,
      });
      // Refresh backup list
      await listBackups(projectPath);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to clean backups:", err);
      return 0;
    } finally {
      setLoading(false);
    }
  };

  return {
    // State
    backups,
    health,
    versions,
    loading,
    error,
    // Actions
    listBackups,
    restoreBackup,
    createBackup,
    checkHealth,
    listVersions,
    cleanBackups,
  };
}
