// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * @fileoverview Hash computation operations
 *
 * Contains the IPC-heavy functions that actually compute hashes:
 *   - hashSingleFile  — hash one evidence file
 *   - hashSelectedFiles — batch-hash all selected files (parallel, with progress)
 *   - hashAllFiles — select-all + hash
 *
 * Extracted from useHashManager.ts to isolate computation logic
 * from state management (useHashHistory.ts).
 */

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ask } from "@tauri-apps/plugin-dialog";
import { createSignal, type Accessor, type Setter } from "solid-js";
import type { ContainerInfo, DiscoveredFile } from "../types";
import { normalizeError, formatBytes } from "../utils";
import { logAuditAction } from "../utils/telemetry";
import { getPreference } from "../components/preferences";
import { hashContainer, collectStoredHashes, determineVerification } from "./hashUtils";
import type { HashAlgorithmName, HashHistoryEntry, FileHashInfo } from "../types/hash";
import { logger } from "../utils/logger";
import { generateId } from "../types/project";

const log = logger.scope("HashComputation");

// ─── Types ──────────────────────────────────────────────────────────────────

export interface UseHashComputationDeps {
  // From FileManager
  discoveredFiles: Accessor<DiscoveredFile[]>;
  selectedFiles: Accessor<Set<string>>;
  setSelectedFiles: Setter<Set<string>>;
  fileInfoMap: Accessor<Map<string, ContainerInfo>>;
  setWorking: (msg: string) => void;
  setOk: (msg: string) => void;
  setError: (msg: string) => void;
  updateFileStatus: (path: string, status: string, progress: number, error?: string, chunksProcessed?: number, chunksTotal?: number) => void;
  loadFileInfo: (file: DiscoveredFile, includeTree?: boolean) => Promise<ContainerInfo>;

  // From HashHistory
  selectedHashAlgorithm: Accessor<HashAlgorithmName>;
  fileHashMap: Accessor<Map<string, FileHashInfo>>;
  setFileHashMap: Setter<Map<string, FileHashInfo>>;
  hashHistory: Accessor<Map<string, HashHistoryEntry[]>>;
  recordHashToHistory: (file: DiscoveredFile, algorithm: string, hash: string, verified?: boolean, verifiedAgainst?: string) => void;
}

// ─── Batch Progress Types ───────────────────────────────────────────────────

export interface HashBatchProgress {
  /** Unique batch ID */
  id: string;
  /** Total files in this batch */
  totalFiles: number;
  /** Number of files completed so far */
  completedFiles: number;
  /** Overall percent (0-100) */
  percent: number;
  /** Whether this batch is paused */
  paused: boolean;
  /** Whether this batch has finished */
  done: boolean;
}

// ─── Hook ───────────────────────────────────────────────────────────────────

export function useHashComputation(deps: UseHashComputationDeps) {
  const {
    discoveredFiles,
    selectedFiles,
    setSelectedFiles,
    fileInfoMap,
    setWorking,
    setOk,
    setError,
    updateFileStatus,
    loadFileInfo,
    selectedHashAlgorithm,
    fileHashMap,
    setFileHashMap,
    hashHistory,
    recordHashToHistory,
  } = deps;

  // ── Batch progress tracking ───────────────────────────────────────────
  const [activeBatches, setActiveBatches] = createSignal<HashBatchProgress[]>([]);
  let batchIdCounter = 0;

  const updateBatch = (id: string, update: Partial<HashBatchProgress>) => {
    setActiveBatches((prev) =>
      prev.map((b) => (b.id === id ? { ...b, ...update } : b)),
    );
  };

  const removeBatch = (id: string) => {
    setActiveBatches((prev) => prev.filter((b) => b.id !== id));
  };

  /** Pause hash queue — jobs in progress continue, no new jobs start */
  const pauseHashQueue = async () => {
    try {
      await invoke("hash_queue_pause");
      setActiveBatches((prev) => prev.map((b) => ({ ...b, paused: true })));
    } catch (err) {
      log.warn(`Failed to pause hash queue: ${normalizeError(err)}`);
    }
  };

  /** Resume hash queue — new jobs begin processing */
  const resumeHashQueue = async () => {
    try {
      await invoke("hash_queue_resume");
      setActiveBatches((prev) => prev.map((b) => ({ ...b, paused: false })));
    } catch (err) {
      log.warn(`Failed to resume hash queue: ${normalizeError(err)}`);
    }
  };

  // ── Shared DB persistence ─────────────────────────────────────────────

  /**
   * Persist a hash result (and optional verification) to the .ffxdb database.
   * Fire-and-forget — errors are logged but don't break the hash flow.
   */
  const persistHashToDb = async (
    filePath: string,
    algorithm: string,
    hash: string,
    verified: boolean | null,
    verifiedAgainst: string | undefined,
  ): Promise<void> => {
    const hashRecordId = generateId();
    try {
      await invoke("project_db_insert_hash", {
        hash: {
          id: hashRecordId,
          fileId: filePath,
          algorithm,
          hashValue: hash,
          computedAt: new Date().toISOString(),
          source: "computed",
        },
      });

      if (verified !== null && verifiedAgainst) {
        await invoke("project_db_insert_verification", {
          v: {
            id: generateId(),
            hashId: hashRecordId,
            verifiedAt: new Date().toISOString(),
            result: verified ? "match" : "mismatch",
            expectedHash: verifiedAgainst,
            actualHash: hash,
          },
        });
      }
    } catch (dbErr) {
      log.warn(`Failed to persist hash record to .ffxdb: ${normalizeError(dbErr)}`);
    }
  };

  // ── Shared completion handler ─────────────────────────────────────────

  /**
   * Process a completed hash result: verify against stored hashes,
   * update state, record to history, persist to DB, and audit log.
   *
   * Used by both hashSingleFile and the batch event handler.
   */
  const handleHashCompleted = (
    filePath: string,
    hash: string,
    algorithm: string,
    file: DiscoveredFile | undefined,
  ): { verified: boolean | null; verifiedAgainst: string | undefined } => {
    const info = fileInfoMap().get(filePath);
    const storedHashes = collectStoredHashes(filePath, info);
    const history = hashHistory().get(filePath) ?? [];
    const { verified, verifiedAgainst } = determineVerification(hash, algorithm, storedHashes, history);

    // Update hash map
    const hashMap = new Map(fileHashMap());
    hashMap.set(filePath, { algorithm, hash, verified });
    setFileHashMap(hashMap);

    updateFileStatus(filePath, "hashed", 100);

    if (file) {
      recordHashToHistory(file, algorithm, hash, verified ?? undefined, verifiedAgainst);
    }

    // Audit log + DB persistence (fire-and-forget for batch)
    logAuditAction("hash_computed", {
      file: filePath,
      filename: file?.filename ?? filePath.split("/").pop() ?? filePath,
      algorithm,
      hash,
      verified,
      verifiedAgainst,
    });
    persistHashToDb(filePath, algorithm, hash, verified, verifiedAgainst);

    return { verified, verifiedAgainst };
  };

  // ── hashSingleFile ────────────────────────────────────────────────────

  const hashSingleFile = async (file: DiscoveredFile): Promise<string | undefined> => {
    log.debug(`hashSingleFile called for ${file.filename}, path=${file.path}, size=${file.size}`);

    // Check if confirmation is required
    if (getPreference("confirmBeforeHash")) {
      log.debug("Showing confirmation dialog");
      const confirmed = await ask(
        `Compute hash for "${file.filename}" (${formatBytes(file.size)})?\n\nThis may take some time for large files.`,
        { title: "Confirm Hash", kind: "info" },
      );
      if (!confirmed) {
        log.debug("User cancelled hash operation");
        return;
      }
    }

    const algorithm = selectedHashAlgorithm();
    log.debug(`Starting hash with algorithm=${algorithm}`);
    updateFileStatus(file.path, "hashing", 0);

    // Listen for progress events
    const unlisten = await listen<{ path: string; percent: number }>("verify-progress", (e) => {
      if (e.payload.path === file.path) {
        updateFileStatus(file.path, "hashing", e.payload.percent);
      }
    });

    try {
      // Get file extension for hash routing
      const extension = file.filename.split(".").pop()?.toLowerCase() || "";

      // Compute hash using unified hash utility
      const hash = await hashContainer(file.path, extension, algorithm);

      // Verify, persist, and record using shared handler
      const { verified, verifiedAgainst } = handleHashCompleted(file.path, hash, algorithm.toUpperCase(), file);

      log.debug(`Hash complete: ${algorithm}=${hash.substring(0, 16)}... verified=${verified}`);
      setOk(`Hash computed: ${algorithm.toUpperCase()} ${hash.substring(0, 16)}…${verified === true ? " ✓ Verified" : verified === false ? " ✗ MISMATCH" : ""}`);

      // Write-through: await DB persistence for single-file (forensic integrity)
      await persistHashToDb(file.path, algorithm.toUpperCase(), hash, verified, verifiedAgainst);

      // Copy to clipboard if preference enabled
      if (getPreference("copyHashToClipboard")) {
        try {
          await navigator.clipboard.writeText(hash);
        } catch {
          // Ignore clipboard failures
        }
      }

      return hash;
    } catch (err) {
      const errMsg = normalizeError(err);
      log.warn(`Hash computation failed: ${errMsg}`);
      updateFileStatus(file.path, "error", 0, errMsg);
      throw err;
    } finally {
      unlisten();
    }
  };

  // ── hashSelectedFiles ─────────────────────────────────────────────────

  const hashSelectedFiles = async (): Promise<void> => {
    const files = discoveredFiles().filter((f) => selectedFiles().has(f.path));
    if (!files.length) {
      setError("No files selected");
      return;
    }

    log.debug(`hashSelectedFiles starting with ${files.length} files`);

    const numCores = navigator.hardwareConcurrency || 4;

    // Create a batch progress entry
    const batchId = `batch-${++batchIdCounter}-${Date.now()}`;
    setActiveBatches((prev) => [
      ...prev,
      { id: batchId, totalFiles: files.length, completedFiles: 0, percent: 0, paused: false, done: false },
    ]);

    // Set all selected files to hashing status immediately
    files.forEach((f) => updateFileStatus(f.path, "hashing", 0));
    setWorking(`# Hashing 0/${files.length} files (${numCores} cores)...`);

    // Load file info in parallel with hashing setup (non-blocking)
    const filesToLoad = files.filter((f) => !fileInfoMap().has(f.path));
    if (filesToLoad.length > 0) {
      log.debug(`Loading info for ${filesToLoad.length} files in background`);
      Promise.all(
        filesToLoad.map(async (file) => {
          try {
            await loadFileInfo(file, false);
          } catch (err) {
            log.debug(` Failed to load info for ${file.path}:`, err);
          }
        }),
      ).catch(() => {});
    }

    // Track completed files for immediate UI updates
    let completedCount = 0;
    let verifiedCount = 0;
    let failedCount = 0;

    // Track per-file chunk progress for smooth overall progress.
    // With parallel hashing (num_cpus files at once), multiple files
    // emit independent progress events. This map captures each file's
    // chunk-level percent so the overall bar reflects real progress
    // even when hashing a single large file.
    const activeFilePercents = new Map<string, number>();
    // Track which files have received a terminal event (completed or error)
    const terminatedFiles = new Set<string>();

    const computeOverallPercent = () => {
      let activeSum = 0;
      for (const p of activeFilePercents.values()) activeSum += p;
      return Math.min(100, Math.round(((completedCount * 100 + activeSum) / (files.length * 100)) * 100));
    };

    // ── Throttled progress UI updates ──────────────────────────────────
    // Buffer per-file progress events and flush to the UI at most every 200ms.
    // This avoids creating a new Map per event when many files hash in parallel.
    const pendingProgress = new Map<string, { percent: number; chunksProcessed?: number; chunksTotal?: number }>();
    let progressFlushTimer: ReturnType<typeof setTimeout> | null = null;

    const flushProgress = () => {
      progressFlushTimer = null;
      if (pendingProgress.size === 0) return;
      for (const [p, info] of pendingProgress) {
        updateFileStatus(p, "hashing", info.percent, undefined, info.chunksProcessed, info.chunksTotal);
      }
      pendingProgress.clear();
    };

    const scheduleProgressFlush = () => {
      if (!progressFlushTimer) {
        progressFlushTimer = setTimeout(flushProgress, 200);
      }
    };

    // Listen for batch progress events
    const unlisten = await listen<{
      path: string;
      status: string;
      percent: number;
      filesCompleted: number;
      filesTotal: number;
      chunksProcessed?: number;
      chunksTotal?: number;
      hash?: string;
      algorithm?: string;
      error?: string;
    }>("batch-progress", (e) => {
      const {
        path,
        status,
        percent,
        filesCompleted: _fc,
        filesTotal: _ft,
        chunksProcessed,
        chunksTotal,
        hash,
        algorithm,
        error,
      } = e.payload;

      if (status === "progress" || status === "started") {
        // Buffer progress events for batched UI update
        pendingProgress.set(path, { percent, chunksProcessed, chunksTotal });
        activeFilePercents.set(path, percent);
        scheduleProgressFlush();
        updateBatch(batchId, { percent: computeOverallPercent() });
      } else if (status === "completed" && hash && algorithm) {
        const file = files.find((f) => f.path === path);

        // Use shared completion handler (verify + persist + audit)
        const { verified } = handleHashCompleted(path, hash, algorithm, file);

        if (verified === true) verifiedCount++;
        else if (verified === false) failedCount++;
        completedCount++;
        activeFilePercents.delete(path);
        terminatedFiles.add(path);

        log.debug(`File completed: ${path}, completedCount=${completedCount}/${files.length}`);

        // Update status with local count
        setWorking(`# Hashing ${completedCount}/${files.length} files completed`);
        updateBatch(batchId, {
          completedFiles: completedCount,
          percent: computeOverallPercent(),
        });
      } else if (status === "error") {
        updateFileStatus(path, "error", 0, error || "Unknown error");
        completedCount++;
        activeFilePercents.delete(path);
        terminatedFiles.add(path);
        log.debug(`File error: ${path}, completedCount=${completedCount}/${files.length}`);
        setWorking(`# Hashing ${completedCount}/${files.length} files (1 error)`);
        updateBatch(batchId, {
          completedFiles: completedCount,
          percent: computeOverallPercent(),
        });
      }

      // Show decompression progress in status if available
      if (chunksProcessed !== undefined && chunksTotal !== undefined && chunksTotal > 0) {
        setWorking(`# ${completedCount}/${files.length} files | ${chunksProcessed.toLocaleString()}/${chunksTotal.toLocaleString()} chunks`);
      } else if (status === "progress" || status === "started") {
        setWorking(`# Hashing ${completedCount}/${files.length} files`);
      }
    });

    try {
      await invoke<{ path: string; algorithm: string; hash?: string; error?: string }[]>("batch_hash", {
        files: files.map((f) => ({ path: f.path, containerType: f.container_type })),
        algorithm: selectedHashAlgorithm(),
      });

      // Count results from current state (already updated via events)
      const hashMap = fileHashMap();
      let completed = 0;
      let verifiedCountFinal = 0;
      let failedCountFinal = 0;
      let noStoredCount = 0;

      // Safety net: mark any files that never received a terminal event
      // as errors. This handles spawn_blocking panics, JoinErrors, and
      // any other backend failure that didn't emit an event.
      for (const file of files) {
        if (!terminatedFiles.has(file.path)) {
          log.warn(`File never completed/errored: ${file.path} — marking as error`);
          updateFileStatus(file.path, "error", 0, "Hash operation did not complete");
          failedCountFinal++;
          completed++;
        }
      }

      for (const file of files) {
        const hash = hashMap.get(file.path);
        if (hash) {
          completed++;
          if (hash.verified === true) verifiedCountFinal++;
          else if (hash.verified === false) failedCountFinal++;
          else noStoredCount++;
        }
      }

      let statusMsg = `Hashed ${completed}/${files.length} files`;
      if (verifiedCountFinal > 0 || failedCountFinal > 0) {
        const parts: string[] = [];
        if (verifiedCountFinal > 0) parts.push(`✓ ${verifiedCountFinal} verified`);
        if (failedCountFinal > 0) parts.push(`✗ ${failedCountFinal} FAILED`);
        if (noStoredCount > 0) parts.push(`${noStoredCount} no stored hash`);
        statusMsg += ` • ${parts.join(", ")}`;
      }

      if (failedCountFinal > 0) {
        setError(statusMsg);
      } else {
        setOk(statusMsg);
      }
    } catch (err) {
      setError(normalizeError(err));
      files.forEach((f) => updateFileStatus(f.path, "error", 0, normalizeError(err)));
    } finally {
      unlisten();
      // Flush any remaining buffered progress events
      if (progressFlushTimer) {
        clearTimeout(progressFlushTimer);
        progressFlushTimer = null;
      }
      flushProgress();
      // Remove completed batch after a short delay so the user sees 100%
      updateBatch(batchId, { done: true, percent: 100, completedFiles: files.length });
      setTimeout(() => removeBatch(batchId), 3000);
    }
  };

  // ── hashAllFiles ──────────────────────────────────────────────────────

  const hashAllFiles = async (): Promise<void> => {
    const files = discoveredFiles();
    if (!files.length) {
      setError("No files discovered");
      return;
    }
    setSelectedFiles(new Set(files.map((f) => f.path)));
    await hashSelectedFiles();
  };

  // ── Public API ────────────────────────────────────────────────────────

  return {
    hashSingleFile,
    hashSelectedFiles,
    hashAllFiles,
    // Batch progress
    activeBatches,
    pauseHashQueue,
    resumeHashQueue,
  };
}
