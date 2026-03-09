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
import { normalizeError, formatBytes, getBasename } from "../utils";
import { logAuditAction } from "../utils/telemetry";
import { getPreference } from "../components/preferences";
import { hashContainer, findMatchingStoredHash, compareHashes } from "./hashUtils";
import type { HashAlgorithmName, StoredHashEntry, HashHistoryEntry, FileHashInfo } from "../types/hash";
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
      let hash: string;
      try {
        hash = await hashContainer(file.path, extension, algorithm);
      } catch (_err) {
        // Fallback to legacy system for backwards compatibility
        const ctype = file.container_type.toLowerCase();
        if (ctype.includes("e01") || ctype.includes("encase") || ctype.includes("ex01") || ctype.includes("l01") || ctype.includes("lx01")) {
          hash = await invoke<string>("e01_v3_verify", { inputPath: file.path, algorithm });
        } else if (ctype.includes("ad1")) {
          hash = await invoke<string>("ad1_hash_segments", { inputPath: file.path, algorithm });
        } else if (ctype.includes("raw") || ctype.includes("dd")) {
          hash = await invoke<string>("raw_verify", { inputPath: file.path, algorithm });
        } else if (ctype.includes("ufed") || ctype.includes("zip") || ctype.includes("archive") || ctype.includes("tar") || ctype.includes("7z")) {
          hash = await invoke<string>("raw_verify", { inputPath: file.path, algorithm });
        } else {
          hash = await invoke<string>("raw_verify", { inputPath: file.path, algorithm });
        }
      }

      // Gather all stored hashes from container info
      const info = fileInfoMap().get(file.path);
      const storedHashes: StoredHashEntry[] = [];

      // Collect from E01/L01
      storedHashes.push(
        ...(info?.e01?.stored_hashes?.map((sh) => ({
          algorithm: sh.algorithm,
          hash: sh.hash,
          source: "container" as const,
        })) ?? []),
      );
      storedHashes.push(
        ...(info?.l01?.stored_hashes?.map((sh) => ({
          algorithm: sh.algorithm,
          hash: sh.hash,
          source: "container" as const,
        })) ?? []),
      );

      // Collect from companion logs
      storedHashes.push(
        ...(info?.companion_log?.stored_hashes?.map((sh) => ({
          algorithm: sh.algorithm,
          hash: sh.hash,
          source: "companion" as const,
        })) ?? []),
      );

      // Collect from AD1 companion log
      if (info?.ad1?.companion_log) {
        const adLog = info.ad1.companion_log;
        if (adLog.md5_hash) storedHashes.push({ algorithm: "MD5", hash: adLog.md5_hash, source: "companion" as const });
        if (adLog.sha1_hash) storedHashes.push({ algorithm: "SHA-1", hash: adLog.sha1_hash, source: "companion" as const });
        if (adLog.sha256_hash) storedHashes.push({ algorithm: "SHA-256", hash: adLog.sha256_hash, source: "companion" as const });
      }

      // Collect from UFED (match by filename)
      const fileName = getBasename(file.path);
      const ufedMatch = info?.ufed?.stored_hashes?.find((sh) => sh.filename.toLowerCase() === fileName.toLowerCase());
      if (ufedMatch) {
        storedHashes.push({
          algorithm: ufedMatch.algorithm,
          hash: ufedMatch.hash,
          source: "container" as const,
          filename: ufedMatch.filename,
        });
      }

      // Find matching stored hash using utility function
      const matchingStored = findMatchingStoredHash(hash, algorithm, storedHashes);

      // Check hash history for self-verification (previously computed matches)
      const history = hashHistory().get(file.path) ?? [];
      const matchingComputedHistory = history.find(
        (h) => h.source === "computed" && compareHashes(h.hash, hash, h.algorithm, algorithm),
      );

      // Determine verification status
      let verified: boolean | null;
      let verifiedAgainst: string | undefined;

      if (matchingStored) {
        verified = compareHashes(hash, matchingStored.hash, algorithm, matchingStored.algorithm);
        verifiedAgainst = matchingStored.hash;
        log.debug(`Hash VERIFIED against stored hash, verified=${verified}`);
      } else if (matchingComputedHistory) {
        verified = true;
        verifiedAgainst = matchingComputedHistory.hash;
        log.debug(`Hash VERIFIED against previous computation`);
      } else {
        verified = null;
        log.debug(`No stored hash found for verification`);
      }

      // Update state
      log.debug(`Hash complete: ${algorithm}=${hash.substring(0, 16)}... verified=${verified}`);
      const m = new Map(fileHashMap());
      m.set(file.path, { algorithm: algorithm.toUpperCase(), hash, verified });
      setFileHashMap(m);
      updateFileStatus(file.path, "hashed", 100);

      recordHashToHistory(file, algorithm.toUpperCase(), hash, verified ?? undefined, verifiedAgainst);

      // Audit log
      logAuditAction("hash_computed", {
        file: file.path,
        filename: file.filename,
        algorithm: algorithm.toUpperCase(),
        hash,
        verified,
        verifiedAgainst,
      });

      // Write-through: record hash in .ffxdb (awaitable for forensic integrity)
      const hashRecordId = generateId();
      try {
        await invoke("project_db_insert_hash", {
          hash: {
            id: hashRecordId,
            fileId: file.path,
            algorithm: algorithm.toUpperCase(),
            hashValue: hash,
            computedAt: new Date().toISOString(),
            source: "computed",
          },
        });

        // If verified, also record the verification result
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

    const computeOverallPercent = () => {
      let activeSum = 0;
      for (const p of activeFilePercents.values()) activeSum += p;
      return Math.min(100, Math.round(((completedCount * 100 + activeSum) / (files.length * 100)) * 100));
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
        updateFileStatus(path, "hashing", percent, undefined, chunksProcessed, chunksTotal);
        activeFilePercents.set(path, percent);
        updateBatch(batchId, { percent: computeOverallPercent() });
      } else if (status === "completed" && hash && algorithm) {
        // Immediately update hash map and verify when a file completes
        const file = files.find((f) => f.path === path);
        const info = fileInfoMap().get(path);
        const storedHashes = [...(info?.e01?.stored_hashes ?? []), ...(info?.companion_log?.stored_hashes ?? [])];
        // Also check UFED stored hashes
        const fileName = getBasename(path);
        const ufedStoredHashes = info?.ufed?.stored_hashes ?? [];
        const matchingUfedStored = ufedStoredHashes.find(
          (sh) => sh.algorithm.toLowerCase() === algorithm.toLowerCase() && sh.filename.toLowerCase() === fileName.toLowerCase(),
        );
        const matchingStored =
          storedHashes.find((sh) => sh.algorithm.toLowerCase() === algorithm.toLowerCase()) ??
          (matchingUfedStored ? { algorithm: matchingUfedStored.algorithm, hash: matchingUfedStored.hash } : undefined);

        // Check hash history for matching hash (self-verification)
        const history = hashHistory().get(path) ?? [];
        const matchingHistory = history.find(
          (h) => h.algorithm.toLowerCase() === algorithm.toLowerCase() && h.hash.toLowerCase() === hash.toLowerCase(),
        );

        // Verified if matches stored OR matches previous hash in history
        const verified = matchingStored
          ? hash.toLowerCase() === matchingStored.hash.toLowerCase()
          : matchingHistory
            ? true
            : null;
        const verifiedAgainst = matchingStored?.hash ?? matchingHistory?.hash;

        if (verified === true) verifiedCount++;
        else if (verified === false) failedCount++;
        completedCount++;
        activeFilePercents.delete(path);

        log.debug(`File completed: ${path}, completedCount=${completedCount}/${files.length}`);

        // Update hash map immediately
        const hashMap = new Map(fileHashMap());
        hashMap.set(path, { algorithm, hash, verified });
        setFileHashMap(hashMap);

        updateFileStatus(path, "hashed", 100);

        if (file) {
          recordHashToHistory(file, algorithm, hash, verified ?? undefined, verifiedAgainst);
        }

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
