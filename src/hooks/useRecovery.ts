// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Recovery System Hook
 * 
 * Provides access to the error recovery system for persisting and restoring
 * interrupted operations.
 * 
 * @example
 * ```tsx
 * const recovery = useRecovery();
 * 
 * // Save operation state
 * await recovery.saveOperation({
 *   id: "hash-batch-1",
 *   operationType: "hashing",
 *   state: "running",
 *   data: { files: [...] },
 *   progress: 0.5,
 *   // ...
 * });
 * 
 * // Check for interrupted operations on startup
 * const interrupted = await recovery.getInterruptedOperations();
 * for (const op of interrupted) {
 *   console.log(`Resume operation: ${op.id}`);
 * }
 * ```
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  RecoverableOperation,
  RecoveryStats,
  OperationType,
  OperationState,
  RecoverySystem,
} from "../types/recovery";

export function useRecovery(): RecoverySystem {
  const saveOperation = async (operation: RecoverableOperation): Promise<void> => {
    await invoke("recovery_save_operation", { operation });
  };

  const loadOperation = async (id: string): Promise<RecoverableOperation> => {
    return await invoke("recovery_load_operation", { id });
  };

  const getInterruptedOperations = async (): Promise<RecoverableOperation[]> => {
    return await invoke("recovery_get_interrupted");
  };

  const getOperationsByState = async (state: OperationState): Promise<RecoverableOperation[]> => {
    return await invoke("recovery_get_by_state", { state });
  };

  const updateProgress = async (id: string, progress: number): Promise<void> => {
    await invoke("recovery_update_progress", { id, progress });
  };

  const updateState = async (id: string, state: OperationState): Promise<void> => {
    await invoke("recovery_update_state", { id, state });
  };

  const markFailed = async (id: string, errorMessage: string): Promise<void> => {
    await invoke("recovery_mark_failed", { id, errorMessage });
  };

  const deleteOperation = async (id: string): Promise<void> => {
    await invoke("recovery_delete_operation", { id });
  };

  const cleanupOld = async (days: number): Promise<number> => {
    return await invoke("recovery_cleanup_old", { days });
  };

  const getStats = async (): Promise<RecoveryStats> => {
    return await invoke("recovery_get_stats");
  };

  const createOperation = async (
    id: string,
    operationType: OperationType,
    data: Record<string, unknown>
  ): Promise<RecoverableOperation> => {
    return await invoke("recovery_create_operation", {
      id,
      operationType,
      data,
    });
  };

  return {
    saveOperation,
    loadOperation,
    getInterruptedOperations,
    getOperationsByState,
    updateProgress,
    updateState,
    markFailed,
    deleteOperation,
    cleanupOld,
    getStats,
    createOperation,
  };
}
