// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Recovery & Notification System Types
 * 
 * Type definitions for error recovery and desktop notification features.
 */

/** Operation types that support recovery */
export type OperationType =
  | "hashing"
  | "extraction"
  | "deduplication"
  | "indexing"
  | "report_generation"
  | "archive_extraction";

/** Operation states */
export type OperationState =
  | "pending"
  | "running"
  | "paused"
  | "completed"
  | "failed"
  | "cancelled";

/** Recoverable operation metadata */
export interface RecoverableOperation {
  /** Unique operation ID */
  id: string;
  
  /** Operation type */
  operationType: OperationType;
  
  /** Current state */
  state: OperationState;
  
  /** Operation-specific data (JSON) */
  data: Record<string, unknown>;
  
  /** Progress (0.0 to 1.0) */
  progress: number;
  
  /** Error message if failed */
  errorMessage?: string;
  
  /** Created timestamp (Unix epoch seconds) */
  createdAt: number;
  
  /** Updated timestamp (Unix epoch seconds) */
  updatedAt: number;
  
  /** Number of retry attempts */
  retryCount: number;
}

/** Recovery statistics */
export interface RecoveryStats {
  totalOperations: number;
  pending: number;
  running: number;
  completed: number;
  failed: number;
}

/** Notification types */
export type NotificationType = "info" | "success" | "warning" | "error";

/** Recovery system hook return type */
export interface RecoverySystem {
  /** Save operation state */
  saveOperation: (operation: RecoverableOperation) => Promise<void>;
  
  /** Load operation by ID */
  loadOperation: (id: string) => Promise<RecoverableOperation>;
  
  /** Get interrupted operations */
  getInterruptedOperations: () => Promise<RecoverableOperation[]>;
  
  /** Get operations by state */
  getOperationsByState: (state: OperationState) => Promise<RecoverableOperation[]>;
  
  /** Update operation progress */
  updateProgress: (id: string, progress: number) => Promise<void>;
  
  /** Update operation state */
  updateState: (id: string, state: OperationState) => Promise<void>;
  
  /** Mark operation as failed */
  markFailed: (id: string, errorMessage: string) => Promise<void>;
  
  /** Delete operation */
  deleteOperation: (id: string) => Promise<void>;
  
  /** Clean up old operations */
  cleanupOld: (days: number) => Promise<number>;
  
  /** Get recovery statistics */
  getStats: () => Promise<RecoveryStats>;
  
  /** Create new operation */
  createOperation: (id: string, operationType: OperationType, data: Record<string, unknown>) => Promise<RecoverableOperation>;
}

/** Notification system hook return type */
export interface NotificationSystem {
  /** Show desktop notification */
  show: (type: NotificationType, title: string, message: string) => Promise<void>;
  
  /** Show info notification */
  info: (title: string, message: string) => Promise<void>;
  
  /** Show success notification */
  success: (title: string, message: string) => Promise<void>;
  
  /** Show warning notification */
  warning: (title: string, message: string) => Promise<void>;
  
  /** Show error notification */
  error: (title: string, message: string) => Promise<void>;
  
  /** Enable/disable notifications */
  setEnabled: (enabled: boolean) => Promise<void>;
  
  /** Notify operation completion */
  operationCompleted: (operationName: string, durationMs: number) => Promise<void>;
  
  /** Notify operation failure */
  operationFailed: (operationName: string, error: string) => Promise<void>;
  
  /** Notify progress milestone */
  progressMilestone: (operationName: string, current: number, total: number) => Promise<void>;
  
  /** Notify recovery available */
  recoveryAvailable: (operationName: string) => Promise<void>;
}
