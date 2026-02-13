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
// Recovery Commands
// =============================================================================

export const recoveryCommands = {
  /**
   * Save an operation
   */
  save: (operation: RecoveryOperation): Promise<void> =>
    invoke("recovery_save_operation", { operation }),

  /**
   * Load an operation by ID
   */
  load: (id: string): Promise<RecoveryOperation | null> =>
    invoke("recovery_load_operation", { id }),

  /**
   * Get all interrupted operations
   */
  getInterrupted: (): Promise<RecoveryOperation[]> =>
    invoke("recovery_get_interrupted"),

  /**
   * Get operations by state
   */
  getByState: (state: RecoveryState): Promise<RecoveryOperation[]> =>
    invoke("recovery_get_by_state", { state }),

  /**
   * Update operation progress
   */
  updateProgress: (id: string, progress: number): Promise<void> =>
    invoke("recovery_update_progress", { id, progress }),

  /**
   * Update operation state
   */
  updateState: (id: string, state: RecoveryState): Promise<void> =>
    invoke("recovery_update_state", { id, state }),

  /**
   * Mark operation as failed
   */
  markFailed: (id: string, errorMessage: string): Promise<void> =>
    invoke("recovery_mark_failed", { id, errorMessage }),

  /**
   * Delete an operation
   */
  delete: (id: string): Promise<void> =>
    invoke("recovery_delete_operation", { id }),

  /**
   * Cleanup old operations
   */
  cleanupOld: (days: number): Promise<number> =>
    invoke("recovery_cleanup_old", { days }),

  /**
   * Get recovery stats
   */
  getStats: (): Promise<RecoveryStats> =>
    invoke("recovery_get_stats"),

  /**
   * Create a new recovery operation
   */
  create: (
    operationType: OperationType,
    metadata?: Record<string, unknown>
  ): Promise<string> =>
    invoke("recovery_create_operation", { operationType, metadata }),
};

// =============================================================================
// Notification Commands
// =============================================================================

export const notificationCommands = {
  /**
   * Show a notification
   */
  show: (type: NotificationType, title: string, message?: string): Promise<void> =>
    invoke("notification_show", { notificationType: type, title, message }),

  /**
   * Show info notification
   */
  info: (title: string, message?: string): Promise<void> =>
    invoke("notification_info", { title, message }),

  /**
   * Show success notification
   */
  success: (title: string, message?: string): Promise<void> =>
    invoke("notification_success", { title, message }),

  /**
   * Show warning notification
   */
  warning: (title: string, message?: string): Promise<void> =>
    invoke("notification_warning", { title, message }),

  /**
   * Show error notification
   */
  error: (title: string, message?: string): Promise<void> =>
    invoke("notification_error", { title, message }),

  /**
   * Enable/disable notifications
   */
  setEnabled: (enabled: boolean): Promise<void> =>
    invoke("notification_set_enabled", { enabled }),

  /**
   * Notify operation completed
   */
  operationCompleted: (
    operationName: string,
    summary?: string,
    durationMs?: number
  ): Promise<void> =>
    invoke("notification_operation_completed", { operationName, summary, durationMs }),

  /**
   * Notify operation failed
   */
  operationFailed: (
    operationName: string,
    errorMessage: string,
    recoverable?: boolean
  ): Promise<void> =>
    invoke("notification_operation_failed", { operationName, errorMessage, recoverable }),

  /**
   * Notify progress milestone
   */
  progressMilestone: (
    operationName: string,
    milestone: string,
    progress: number
  ): Promise<void> =>
    invoke("notification_progress_milestone", { operationName, milestone, progress }),

  /**
   * Notify recovery available
   */
  recoveryAvailable: (operationName: string): Promise<void> =>
    invoke("notification_recovery_available", { operationName }),
};

// =============================================================================
// Index Cache Commands
// =============================================================================

export const indexCacheCommands = {
  /**
   * Initialize the index cache
   */
  init: (dbPath: string): Promise<void> =>
    invoke("index_cache_init", { dbPath }),

  /**
   * Start indexing worker
   */
  startWorker: (containerPath: string, containerType: string): Promise<void> =>
    invoke("index_worker_start", { containerPath, containerType }),

  /**
   * Cancel indexing worker
   */
  cancelWorker: (containerPath: string): Promise<void> =>
    invoke("index_worker_cancel", { containerPath }),

  /**
   * Invalidate cache for container
   */
  invalidate: (containerPath: string): Promise<void> =>
    invoke("index_cache_invalidate", { containerPath }),

  /**
   * Clear all cache
   */
  clear: (): Promise<void> =>
    invoke("index_cache_clear"),
};

// =============================================================================
// Extraction Commands
// =============================================================================

export const extractionCommands = {
  /**
   * Parallel extraction
   */
  parallel: {
    init: (): Promise<void> => invoke("parallel_extract_init"),
    
    batch: (
      containerPath: string,
      items: ExtractionItem[],
      batchId: string
    ): Promise<void> =>
      invoke("parallel_extract_batch", { containerPath, items, batchId }),
    
    cancel: (batchId: string): Promise<void> =>
      invoke("parallel_extract_cancel", { batchId }),
  },

  /**
   * Streaming extraction
   */
  streaming: {
    init: (): Promise<void> => invoke("stream_extract_init"),
    
    start: (
      containerPath: string,
      entryPath: string,
      destinationPath: string,
      streamId: string
    ): Promise<void> =>
      invoke("stream_extract_start", { 
        containerPath, 
        entryPath, 
        destinationPath, 
        streamId 
      }),
    
    cancel: (streamId: string): Promise<void> =>
      invoke("stream_extract_cancel", { streamId }),
  },
};

// =============================================================================
// Deduplication Commands
// =============================================================================

export const dedupCommands = {
  /**
   * Initialize deduplication
   */
  init: (): Promise<void> => invoke("dedup_init"),

  /**
   * Scan files for duplicates
   */
  scanFiles: (filePaths: string[]): Promise<void> =>
    invoke("dedup_scan_files", { filePaths }),

  /**
   * Clear deduplication data
   */
  clear: (): Promise<void> => invoke("dedup_clear"),
};

// =============================================================================
// Observability Commands
// =============================================================================

export const observabilityCommands = {
  /**
   * Initialize tracing
   */
  initTracing: (level: string, logDir: string): Promise<void> =>
    invoke("init_tracing", { level, logDir }),

  /**
   * Metrics
   */
  metrics: {
    incrementCounter: (name: string, amount: number): Promise<void> =>
      invoke("increment_counter", { name, amount }),
    
    setGauge: (name: string, value: number): Promise<void> =>
      invoke("set_gauge", { name, value }),
    
    recordHistogram: (name: string, value: number): Promise<void> =>
      invoke("record_histogram", { name, value }),
    
    reset: (): Promise<void> => invoke("reset_metrics"),
  },
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
  recovery: recoveryCommands,
  notification: notificationCommands,
  indexCache: indexCacheCommands,
  extraction: extractionCommands,
  dedup: dedupCommands,
  observability: observabilityCommands,
  system: systemCommands,
} as const;

export default commands;
