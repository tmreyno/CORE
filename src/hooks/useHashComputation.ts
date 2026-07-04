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
import type { HashSourceResult, ProjectDbEvidenceFile, ProjectDbHashSourceResult } from "../api/commands";
import type { SelectedEntry } from "../components/EvidenceTree/types";
import { normalizeError, formatBytes } from "../utils";
import { logAuditAction } from "../utils/telemetry";
import { getBasename } from "../utils/pathUtils";
import { getPreference } from "../components/preferences";
import { hashContainer, collectStoredHashes, determineVerification } from "./hashUtils";
import type { HashAlgorithmName, HashHistoryEntry, FileHashInfo } from "../types/hash";
import { logger } from "../utils/logger";
import { generateId } from "../types/project";
import { dbSync } from "./project/useProjectDbSync";
import { buildLocalFileHashSourceFields } from "../utils/hashSourceIdentity";
import { buildEvidenceSourceInput } from "../components/evidenceSourceInput";
import { isTauri } from "../utils/platform";

const log = logger.scope("HashComputation");

function evidenceFileRecordForHashTarget(
  file: DiscoveredFile | null | undefined,
  entry?: SelectedEntry,
): ProjectDbEvidenceFile | undefined {
  if (file) {
    return {
      id: file.path,
      path: file.path,
      filename: file.filename,
      containerType: file.container_type,
      totalSize: file.size,
      segmentCount: file.segment_count ?? 1,
      discoveredAt: new Date().toISOString(),
      created: file.created ?? null,
      modified: file.modified ?? null,
    };
  }

  if (!entry) return undefined;

  const containerPath = entry.isDiskFile ? entry.entryPath : entry.containerPath;
  return {
    id: containerPath,
    path: containerPath,
    filename: getBasename(containerPath) ?? containerPath,
    containerType: entry.containerType ?? "container",
    totalSize: entry.isDiskFile ? entry.size : 0,
    segmentCount: 1,
    discoveredAt: new Date().toISOString(),
    created: null,
    modified: null,
  };
}

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
  /** Throttled variant for high-frequency progress events (batched per animation frame). */
  updateFileStatusThrottled: (path: string, status: string, progress: number, error?: string, chunksProcessed?: number, chunksTotal?: number) => void;
  loadFileInfo: (file: DiscoveredFile, includeTree?: boolean) => Promise<ContainerInfo>;

  // From HashHistory
  selectedHashAlgorithm: Accessor<HashAlgorithmName>;
  fileHashMap: Accessor<Map<string, FileHashInfo>>;
  setFileHashMap: Setter<Map<string, FileHashInfo>>;
  hashHistory: Accessor<Map<string, HashHistoryEntry[]>>;
  recordHashToHistory: (
    file: DiscoveredFile,
    algorithm: string,
    hash: string,
    options?: {
      computedAt?: string;
      verified?: boolean | null;
      verifiedAgainst?: string;
      comparisonSource?: "stored" | "history";
    },
  ) => void;
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

interface HashSourceProgressEvent {
  sourceId: string;
  current: number;
  total: number;
  percent: number;
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
    updateFileStatusThrottled,
    // loadFileInfo intentionally not used — see note in hashSelectedFiles
    selectedHashAlgorithm,
    fileHashMap,
    setFileHashMap,
    hashHistory,
    recordHashToHistory,
  } = deps;

  // ── Batch progress tracking ───────────────────────────────────────────
  const [activeBatches, setActiveBatches] = createSignal<HashBatchProgress[]>([]);
  let batchIdCounter = 0;
  let hashingInProgress = false;

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
    if (!isTauri) {
      setActiveBatches((prev) => prev.map((b) => ({ ...b, paused: true })));
      return;
    }

    try {
      await invoke("hash_queue_pause");
      setActiveBatches((prev) => prev.map((b) => ({ ...b, paused: true })));
    } catch (err) {
      log.warn(`Failed to pause hash queue: ${normalizeError(err)}`);
    }
  };

  /** Resume hash queue — new jobs begin processing */
  const resumeHashQueue = async () => {
    if (!isTauri) {
      setActiveBatches((prev) => prev.map((b) => ({ ...b, paused: false })));
      return;
    }

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
    computedAt: string,
    verified: boolean | null,
    verifiedAgainst: string | undefined,
    comparisonSource: "stored" | "history" | undefined,
  ): Promise<void> => {
    if (!isTauri) {
      return;
    }

    const hashRecordId = generateId();
    try {
      await invoke("project_db_insert_hash", {
        hash: {
          id: hashRecordId,
          fileId: filePath,
          ...buildLocalFileHashSourceFields(filePath),
          algorithm,
          hashValue: hash,
          computedAt,
          source: "computed",
        },
      });

      if (comparisonSource === "stored" && verified !== null && verifiedAgainst) {
        await invoke("project_db_insert_verification", {
          v: {
            id: generateId(),
            hashId: hashRecordId,
            verifiedAt: computedAt,
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
  ): {
    computedAt: string;
    verified: boolean | null;
    verifiedAgainst: string | undefined;
    comparisonSource: "stored" | "history" | undefined;
  } => {
    const info = fileInfoMap().get(filePath);
    const storedHashes = collectStoredHashes(filePath, info);
    const history = hashHistory().get(filePath) ?? [];
    const computedAt = new Date().toISOString();
    const { verified, verifiedAgainst, comparisonSource } = determineVerification(hash, algorithm, storedHashes, history);

    // Update hash map
    const hashMap = new Map(fileHashMap());
    hashMap.set(filePath, {
      algorithm,
      hash,
      verified,
      computedAt,
      verifiedAgainst,
      comparisonSource,
    });
    setFileHashMap(hashMap);

    updateFileStatus(filePath, "hashed", 100);

    if (file) {
      recordHashToHistory(file, algorithm, hash, {
        computedAt,
        verified,
        verifiedAgainst,
        comparisonSource,
      });
    }

    logAuditAction("hash_computed", {
      file: filePath,
      filename: file?.filename ?? getBasename(filePath) ?? filePath,
      algorithm,
      hash,
      verified,
      verifiedAgainst,
      comparisonSource,
    });

    return {
      computedAt,
      verified,
      verifiedAgainst,
      comparisonSource,
    };
  };

  // ── hashSingleFile ────────────────────────────────────────────────────

  const hashSingleFile = async (file: DiscoveredFile): Promise<string | undefined> => {
    log.debug(`hashSingleFile called for ${file.filename}, path=${file.path}, size=${file.size}`);
    console.warn(`[HASH-DIAG] hashSingleFile: file=${file.filename}, container_type=${file.container_type}, path=${file.path}, size=${file.size}`);

    // Check if confirmation is required
    if (getPreference("confirmBeforeHash")) {
      if (!isTauri) {
        log.info("Skipping native hash confirmation outside Tauri runtime");
      } else {
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
    }

    const algorithm = selectedHashAlgorithm();
    log.debug(`Starting hash with algorithm=${algorithm}`);
    updateFileStatus(file.path, "hashing", 0);

    // Listen for progress events
    const unlisten = isTauri
      ? await listen<{ path: string; percent: number }>("verify-progress", (e) => {
          if (e.payload.path === file.path) {
            console.warn(`[HASH-DIAG] verify-progress: path=${e.payload.path}, percent=${e.payload.percent}`);
            updateFileStatus(file.path, "hashing", e.payload.percent);
          }
        })
      : () => {};

    try {
      // Get file extension for hash routing
      const extension = file.filename.split(".").pop()?.toLowerCase() || "";

      // Compute hash using unified hash utility
      console.warn(`[HASH-DIAG] calling hashContainer: path=${file.path}, ext=${extension}, algo=${algorithm}`);
      const hash = await hashContainer(file.path, extension, algorithm);
      console.warn(`[HASH-DIAG] hashContainer returned: hash=${hash?.substring(0, 16)}...`);

      // Verify, persist, and record using shared handler
      const { computedAt, verified, verifiedAgainst, comparisonSource } = handleHashCompleted(
        file.path,
        hash,
        algorithm.toUpperCase(),
        file,
      );
      console.warn(`[HASH-DIAG] handleHashCompleted done: verified=${verified}, verifiedAgainst=${verifiedAgainst}, comparisonSource=${comparisonSource}`);

      log.debug(`Hash complete: ${algorithm}=${hash.substring(0, 16)}... verified=${verified}`);
      setOk(
        `Hash computed: ${algorithm.toUpperCase()} ${hash.substring(0, 16)}…${
          verified === true
            ? " ✓ Verified"
            : verified === false
              ? " ✗ MISMATCH"
              : comparisonSource === "history"
                ? " Repeat match"
                : ""
        }`,
      );

      // Write-through: await DB persistence for single-file (forensic integrity)
      await persistHashToDb(
        file.path,
        algorithm.toUpperCase(),
        hash,
        computedAt,
        verified,
        verifiedAgainst,
        comparisonSource,
      );

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
      console.warn(`[HASH-DIAG] hashSingleFile ERROR: ${errMsg}`);
      log.warn(`Hash computation failed: ${errMsg}`);
      updateFileStatus(file.path, "error", 0, errMsg);
      throw err;
    } finally {
      unlisten();
    }
  };

  // ── hashEntry ─────────────────────────────────────────────────────────

  const hashEntry = async (
    entry: SelectedEntry,
    parentFile?: DiscoveredFile | null,
  ): Promise<string | undefined> => {
    if (entry.isDir) {
      setError("Cannot hash a directory entry");
      return;
    }

    const source = buildEvidenceSourceInput(parentFile ?? null, entry);
    if (!source) {
      setError("No hashable source selected");
      return;
    }

    if (!isTauri && import.meta.env.MODE !== "test") {
      setError("Source entry hashing is available in the desktop app.");
      return;
    }

    if (getPreference("confirmBeforeHash")) {
      if (!isTauri) {
        log.info("Skipping native source-entry hash confirmation outside Tauri runtime");
      } else {
        const confirmed = await ask(
          `Compute hash for "${entry.name}" (${formatBytes(entry.size)})?\n\nThis hashes the selected source entry, not just the parent container file.`,
          { title: "Confirm Hash", kind: "info" },
        );
        if (!confirmed) return;
      }
    }

    const algorithm = selectedHashAlgorithm();
    const initialStatusKey = `${entry.containerPath}:${entry.entryPath}`;
    updateFileStatus(initialStatusKey, "hashing", 0);
    setWorking(`# Hashing ${entry.name || entry.entryPath}...`);

    const unlisten = await listen<HashSourceProgressEvent>("hash-source-progress", (event) => {
      updateFileStatus(event.payload.sourceId, "hashing", event.payload.percent);
    });

    try {
      const evidenceFile = evidenceFileRecordForHashTarget(parentFile, entry);
      const projectDbOpen = await invoke<boolean>("project_db_is_open").catch(() => false);
      let hashResult: HashSourceResult;

      if (projectDbOpen) {
        const persisted = await invoke<ProjectDbHashSourceResult>(
          "project_db_hash_source_and_insert",
          {
            request: {
              source,
              algorithm,
              evidenceFile,
              hashRecordSource: "computed",
            },
          },
        );
        hashResult = persisted.hashResult;
      } else {
        hashResult = await invoke<HashSourceResult>("hash_source", {
          source,
          algorithm,
        });
      }

      const computedAt = new Date().toISOString();
      const sourceId = hashResult.sourceId;
      const hashMap = new Map(fileHashMap());
      hashMap.set(sourceId, {
        algorithm: hashResult.algorithm,
        hash: hashResult.hash,
        verified: null,
        computedAt,
      });
      setFileHashMap(hashMap);

      updateFileStatus(initialStatusKey, "hashed", 100);
      updateFileStatus(sourceId, "hashed", 100);
      setOk(`Hash computed: ${hashResult.algorithm} ${hashResult.hash.substring(0, 16)}...`);

      logAuditAction("hash_computed", {
        file: sourceId,
        filename: entry.name || getBasename(entry.entryPath) || entry.entryPath,
        algorithm: hashResult.algorithm,
        hash: hashResult.hash,
        verified: null,
        sourceId,
        sourceRef: hashResult.sourceRef,
      });

      if (getPreference("copyHashToClipboard")) {
        try {
          await navigator.clipboard.writeText(hashResult.hash);
        } catch {
          // Ignore clipboard failures
        }
      }

      return hashResult.hash;
    } catch (err) {
      const errMsg = normalizeError(err);
      log.warn(`Entry hash computation failed: ${errMsg}`);
      updateFileStatus(initialStatusKey, "error", 0, errMsg);
      setError(errMsg);
      throw err;
    } finally {
      unlisten();
    }
  };

  // ── hashSelectedFiles ─────────────────────────────────────────────────

  const hashSelectedFiles = async (): Promise<void> => {
    // Prevent concurrent invocations — two listeners on the same events
    // causes duplicate completion handling, double-counting, and race conditions.
    if (hashingInProgress) {
      log.warn("hashSelectedFiles already in progress — ignoring duplicate call");
      return;
    }

    const files = discoveredFiles().filter((f) => selectedFiles().has(f.path));
    if (!files.length) {
      setError("No files selected");
      return;
    }

    if (!isTauri) {
      setError("Batch hashing is available in the desktop app.");
      return;
    }

    hashingInProgress = true;
    log.debug(`hashSelectedFiles starting with ${files.length} files`);

    // Create a batch progress entry
    const batchId = `batch-${++batchIdCounter}-${Date.now()}`;
    setActiveBatches((prev) => [
      ...prev,
      { id: batchId, totalFiles: files.length, completedFiles: 0, percent: 0, paused: false, done: false },
    ]);

    // Set all selected files to hashing status immediately
    files.forEach((f) => updateFileStatus(f.path, "hashing", 0));
    setWorking(`# Hashing 0/${files.length} files...`);

    // Listen for drive detection results (emitted before hashing starts)
    let driveInfoSummary = "";
    const unlistenDrive = await listen<{
      drives: Array<{ mountPoint: string; storageClass: string; concurrency: number; fileCount: number }>;
      totalFiles: number;
    }>("batch-drive-info", (e) => {
      const { drives } = e.payload;
      const parts = drives.map((d) => `${d.storageClass} @ ${d.mountPoint} (${d.concurrency} concurrent, ${d.fileCount} files)`);
      driveInfoSummary = parts.join("; ");
      log.info(`Drive detection: ${driveInfoSummary}`);
      setWorking(`# Hashing 0/${files.length} files — ${parts.map((p) => p.split(" @ ")[0]).join(", ")}`);
    });

    // Ensure all files have evidence_file records in .ffxdb before hashing.
    // This prevents FOREIGN KEY constraint failures when persisting hash results.
    // Uses fire-and-forget dbSync (lightweight IPC, no heavy I/O).
    const now = new Date().toISOString();
    for (const file of files) {
      dbSync.upsertEvidenceFile({
        id: file.path,
        path: file.path,
        filename: file.filename,
        containerType: file.container_type,
        totalSize: file.size,
        segmentCount: file.segment_count ?? 1,
        discoveredAt: now,
      });
    }

    // NOTE: We intentionally do NOT fire parallel loadFileInfo calls here.
    // Each loadFileInfo invokes "logical_info" which opens and parses the full
    // container (E01 segment discovery, header parsing, etc.). Firing 14 of
    // these in parallel saturates Tauri's thread pool and USB I/O, blocking
    // batch_hash from starting for minutes. Container info is loaded on-demand
    // when users click files, or can be loaded after hashing completes.
    // The stored-hash verification in handleHashCompleted uses whatever info
    // is already cached in fileInfoMap() — if not cached, verification is
    // deferred (hash is recorded as "no stored hash" rather than blocking).

    // Track completed files for immediate UI updates
    let completedCount = 0;

    // Throttle status bar updates — at most once per 250ms to avoid UI jank
    let _lastStatusUpdate = 0;
    const throttledSetWorking = (msg: string) => {
      const now = Date.now();
      if (now - _lastStatusUpdate >= 250) {
        _lastStatusUpdate = now;
        setWorking(msg);
      }
    };

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
        updateFileStatusThrottled(p, "hashing", info.percent, undefined, info.chunksProcessed, info.chunksTotal);
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
        console.warn(`[HASH-DIAG] batch completed: path=${path}, hash=${hash.substring(0, 16)}..., algo=${algorithm}, fileFound=${!!file}, container_type=${file?.container_type}`);

        // Use shared completion handler (verify + audit)
        const { computedAt, verified, verifiedAgainst, comparisonSource } = handleHashCompleted(path, hash, algorithm, file);

        void persistHashToDb(
          path,
          algorithm,
          hash,
          computedAt,
          verified,
          verifiedAgainst,
          comparisonSource,
        );

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
        console.warn(`[HASH-DIAG] batch error: path=${path}, error=${error}`);
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

      // Show decompression progress in status if available — throttled to avoid flooding
      if (chunksProcessed !== undefined && chunksTotal !== undefined && chunksTotal > 0) {
        throttledSetWorking(`# ${completedCount}/${files.length} files | ${chunksProcessed.toLocaleString()}/${chunksTotal.toLocaleString()} chunks`);
      } else if (status === "progress" || status === "started") {
        throttledSetWorking(`# Hashing ${completedCount}/${files.length} files`);
      }
    });

    try {
      // Build per-storage-class concurrency overrides from user preferences.
      // Keys match StorageClass::key() in hash.rs. Value 0 = auto (backend default).
      const concurrencyOverrides: Record<string, number> = {
        nvme: getPreference("hashConcurrencyNvme"),
        ssd: getPreference("hashConcurrencySsd"),
        raid: getPreference("hashConcurrencyRaid"),
        hdd: getPreference("hashConcurrencyHdd"),
        removable: getPreference("hashConcurrencyRemovable"),
        network: getPreference("hashConcurrencyNetwork"),
      };
      // Only send overrides if at least one is non-zero (user customized)
      const hasCustomOverrides = Object.values(concurrencyOverrides).some((v) => v > 0);

      console.warn(`[HASH-DIAG] invoking batch_hash with ${files.length} files:`, files.map(f => ({ path: f.path, ct: f.container_type })));
      await invoke<{ path: string; algorithm: string; hash?: string; error?: string; driveKind?: string }[]>("batch_hash", {
        files: files.map((f) => ({ path: f.path, containerType: f.container_type })),
        algorithm: selectedHashAlgorithm(),
        concurrencyOverrides: hasCustomOverrides ? concurrencyOverrides : null,
      });
      console.warn(`[HASH-DIAG] batch_hash invoke returned, terminatedFiles: ${terminatedFiles.size}/${files.length}`);

      // Count results from current state (already updated via events)
      const hashMap = fileHashMap();
      let completed = 0;
      let verifiedCountFinal = 0;
      let failedCountFinal = 0;
      let noStoredCount = 0;
      let repeatMatchCountFinal = 0;

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
          else if (hash.comparisonSource === "history") repeatMatchCountFinal++;
          else noStoredCount++;
        }
      }

      let statusMsg = `Hashed ${completed}/${files.length} files`;
      if (verifiedCountFinal > 0 || failedCountFinal > 0 || repeatMatchCountFinal > 0) {
        const parts: string[] = [];
        if (verifiedCountFinal > 0) parts.push(`✓ ${verifiedCountFinal} verified`);
        if (failedCountFinal > 0) parts.push(`✗ ${failedCountFinal} FAILED`);
        if (repeatMatchCountFinal > 0) parts.push(`${repeatMatchCountFinal} repeat match`);
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
      hashingInProgress = false;
      unlisten();
      unlistenDrive();
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
    hashEntry,
    hashSelectedFiles,
    hashAllFiles,
    // Batch progress
    activeBatches,
    pauseHashQueue,
    resumeHashQueue,
  };
}
