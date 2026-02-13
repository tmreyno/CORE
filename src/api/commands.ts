// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Tauri Commands API
 * 
 * Type-safe wrappers for Tauri invoke calls. Use these instead of raw invoke()
 * calls for better type safety and centralized command management.
 * 
 * @example
 * ```tsx
 * import { commands } from '../api/commands';
 * 
 * // Container operations
 * const info = await commands.container.getInfo(path);
 * const children = await commands.container.getChildren(path, parentId);
 * 
 * // Hash operations
 * await commands.hash.compute(path, 'SHA-256');
 * 
 * // Database operations
 * await commands.database.upsertFile(fileData);
 * ```
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  LazyLoadResult,
  ContainerSummary,
  LazyLoadConfig,
} from "../types/lazy-loading";

// =============================================================================
// Type Definitions
// =============================================================================

/** Operation types for recovery */
export type OperationType = 
  | 'hash_verification'
  | 'extraction'
  | 'indexing'
  | 'scan';

/** Recovery operation states */
export type RecoveryState = 
  | 'pending'
  | 'in_progress'
  | 'paused'
  | 'completed'
  | 'failed'
  | 'cancelled';

/** Notification types */
export type NotificationType = 'info' | 'success' | 'warning' | 'error';

/** Hash algorithm types */
export type HashAlgorithm = 'MD5' | 'SHA-1' | 'SHA-256' | 'SHA-512';

/** Recovery operation interface */
export interface RecoveryOperation {
  id: string;
  operation_type: OperationType;
  state: RecoveryState;
  progress: number;
  created_at: string;
  updated_at: string;
  error_message?: string;
  metadata?: Record<string, unknown>;
}

/** Recovery stats interface */
export interface RecoveryStats {
  total: number;
  pending: number;
  in_progress: number;
  completed: number;
  failed: number;
}

/** File record for database */
export interface FileRecord {
  path: string;
  name: string;
  size: number;
  modified?: string;
  created?: string;
  file_type?: string;
}

/** Hash record for database */
export interface HashRecord {
  file_path: string;
  algorithm: HashAlgorithm;
  hash_value: string;
  computed_at: string;
}

/** Verification record for database */
export interface VerificationRecord {
  file_path: string;
  algorithm: HashAlgorithm;
  expected_hash: string;
  actual_hash: string;
  verified_at: string;
  is_match: boolean;
}

/** Extraction item */
export interface ExtractionItem {
  source_path: string;
  entry_path: string;
  destination_path: string;
}

/** Health check result */
export interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  uptime_seconds: number;
  version: string;
  platform: string;
  memory_used_mb: number;
}

// =============================================================================
// Container Commands
// =============================================================================

export const containerCommands = {
  /**
   * Get container summary (entry count, type, lazy loading recommendation)
   */
  getSummary: (path: string): Promise<ContainerSummary> =>
    invoke<ContainerSummary>("container_get_summary", { path }),

  /**
   * Get root-level children of a container (V2 API - fast, cached)
   */
  getRootChildren: (
    path: string
  ): Promise<LazyLoadResult> =>
    invoke<LazyLoadResult>("container_get_root_children_v2", { 
      containerPath: path
    }),

  /**
   * Get children at a specific address (V2 API - fast, cached)
   * Use entry.first_child_addr to get children of a directory
   */
  getChildrenAtAddr: (
    path: string,
    addr: number,
    parentPath: string
  ): Promise<LazyLoadResult> =>
    invoke<LazyLoadResult>("container_get_children_at_addr_v2", { 
      containerPath: path, 
      addr,
      parentPath
    }),

  /**
   * Get lazy load settings
   */
  getSettings: (): Promise<LazyLoadConfig> =>
    invoke<LazyLoadConfig>("get_lazy_load_settings"),

  /**
   * Update lazy load settings
   */
  updateSettings: (config: Partial<LazyLoadConfig>): Promise<void> =>
    invoke("update_lazy_load_settings", { config }),
};

// =============================================================================
// Hash Commands
// =============================================================================

export const hashCommands = {
  /**
   * Queue operations
   */
  queue: {
    resume: (): Promise<void> => invoke("hash_queue_resume"),
    pause: (): Promise<void> => invoke("hash_queue_pause"),
    clearCompleted: (): Promise<void> => invoke("hash_queue_clear_completed"),
  },
};

// =============================================================================
// Database Commands
// =============================================================================

export const databaseCommands = {
  /**
   * Insert or update a file record
   */
  upsertFile: (file: FileRecord): Promise<void> =>
    invoke("db_upsert_file", { file }),

  /**
   * Insert a hash record
   */
  insertHash: (hash: HashRecord): Promise<void> =>
    invoke("db_insert_hash", { hash }),

  /**
   * Insert a verification record
   */
  insertVerification: (verification: VerificationRecord): Promise<void> =>
    invoke("db_insert_verification", { verification }),

  /**
   * Save open tabs to session
   */
  saveOpenTabs: (sessionId: string, tabs: string[]): Promise<void> =>
    invoke("db_save_open_tabs", { sessionId, tabs }),

  /**
   * Set a setting value
   */
  setSetting: (key: string, value: string): Promise<void> =>
    invoke("db_set_setting", { key, value }),
};

// =============================================================================
// System Commands
// =============================================================================

export const systemCommands = {
  /**
   * Open path in system file manager
   */
  openPath: (path: string): Promise<void> =>
    invoke("plugin:opener|open_path", { path }),
};

// =============================================================================
// Unified Commands Export
// =============================================================================

/**
 * Unified commands API for all Tauri backend operations.
 * Provides type-safe wrappers with organized namespaces.
 */
export const commands = {
  container: containerCommands,
  hash: hashCommands,
  database: databaseCommands,
  system: systemCommands,
} as const;

export default commands;
