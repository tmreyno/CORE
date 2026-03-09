// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * @fileoverview Hash management hook — thin orchestrator
 *
 * Composes two focused sub-modules:
 *   - useHashHistory    — state, import, restore, recording
 *   - useHashComputation — IPC-heavy single/batch/all hash operations
 *
 * The public API surface is unchanged from the monolithic version.
 */

import { createSignal } from "solid-js";
import type { FileManager } from "./useFileManager";
import { getPreference } from "../components/preferences";
import { HASH_ALGORITHMS, HASH_ALGORITHM_MAP, type HashAlgorithmName } from "../types/hash";
import { useHashHistory } from "./useHashHistory";
import { useHashComputation } from "./useHashComputation";
import { logger } from "../utils/logger";

const log = logger.scope("HashManager");

// Re-export FileHashInfo for external consumers
export type { FileHashInfo } from "../types/hash";

// ─── Helpers ────────────────────────────────────────────────────────────────

/** Read the default hash algorithm from user preferences. */
function getInitialHashAlgorithm(): HashAlgorithmName {
  const pref = getPreference("defaultHashAlgorithm");
  return HASH_ALGORITHM_MAP[pref] || HASH_ALGORITHMS.SHA256;
}

// ─── Hook ───────────────────────────────────────────────────────────────────

export function useHashManager(fileManager: FileManager) {
  log.debug("Hook initialized");

  // Algorithm signal — owned here, shared with both sub-modules
  const [selectedHashAlgorithm, setSelectedHashAlgorithm] = createSignal<HashAlgorithmName>(getInitialHashAlgorithm());

  // ── Sub-module: Hash History (state + import + restore) ────────────────
  const history = useHashHistory({
    fileInfoMap: fileManager.fileInfoMap,
  });

  // ── Sub-module: Hash Computation (single / batch / all) ────────────────
  const computation = useHashComputation({
    // FileManager deps
    discoveredFiles: fileManager.discoveredFiles,
    selectedFiles: fileManager.selectedFiles,
    setSelectedFiles: fileManager.setSelectedFiles,
    fileInfoMap: fileManager.fileInfoMap,
    setWorking: fileManager.setWorking,
    setOk: fileManager.setOk,
    setError: fileManager.setError,
    updateFileStatus: fileManager.updateFileStatus,
    loadFileInfo: fileManager.loadFileInfo,

    // Hash history deps
    selectedHashAlgorithm,
    fileHashMap: history.fileHashMap,
    setFileHashMap: history.setFileHashMap,
    hashHistory: history.hashHistory,
    recordHashToHistory: history.recordHashToHistory,
  });

  // ── clearAll (needs algorithm reset + history clear) ───────────────────
  const clearAll = () => {
    history.clearAll();
    setSelectedHashAlgorithm(HASH_ALGORITHMS.SHA256);
    log.debug(" Cleared all state");
  };

  // ── Public API (unchanged surface) ────────────────────────────────────

  return {
    // State
    selectedHashAlgorithm,
    setSelectedHashAlgorithm,
    fileHashMap: history.fileHashMap,
    setFileHashMap: history.setFileHashMap,
    hashHistory: history.hashHistory,

    // Actions — computation
    hashSingleFile: computation.hashSingleFile,
    hashSelectedFiles: computation.hashSelectedFiles,
    hashAllFiles: computation.hashAllFiles,

    // Batch progress & queue control
    activeBatches: computation.activeBatches,
    pauseHashQueue: computation.pauseHashQueue,
    resumeHashQueue: computation.resumeHashQueue,

    // Actions — history
    importStoredHashesToHistory: history.importStoredHashesToHistory,
    importPreloadedStoredHashes: history.importPreloadedStoredHashes,
    addTransferHashesToHistory: history.addTransferHashesToHistory,
    restoreHashHistory: history.restoreHashHistory,
    restoreFileHashMap: history.restoreFileHashMap,
    clearAll,

    // Helpers
    getAllStoredHashesSorted: history.getAllStoredHashesSorted,
    formatHashDate: history.formatHashDate,
    recordHashToHistory: history.recordHashToHistory,
  };
}

export type HashManager = ReturnType<typeof useHashManager>;
