// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * @fileoverview Hash history & import management
 *
 * Manages the reactive hash-history and file-hash-map state.
 * Handles importing stored hashes from container metadata, recording
 * newly computed hashes, restoring history from saved projects, and
 * getting sorted stored-hash views.
 *
 * Extracted from useHashManager.ts to keep computation and state
 * management concerns separate.
 */

import { createSignal, createEffect, on, type Accessor } from "solid-js";
import type { ContainerInfo, StoredHash, DiscoveredFile } from "../types";
import { formatHashDate } from "../utils";
import {
  type HashHistoryEntry,
  type FileHashInfo,
} from "../types/hash";
import { logger } from "../utils/logger";

const log = logger.scope("HashHistory");

// Re-export for external consumers
export type { FileHashInfo } from "../types/hash";

// ─── Types ──────────────────────────────────────────────────────────────────

export interface UseHashHistoryDeps {
  /** Reactive map of file path → ContainerInfo */
  fileInfoMap: Accessor<Map<string, ContainerInfo>>;
}

// ─── Hook ───────────────────────────────────────────────────────────────────

export function useHashHistory(deps: UseHashHistoryDeps) {
  const { fileInfoMap } = deps;

  // ── State ─────────────────────────────────────────────────────────────

  const [fileHashMap, setFileHashMap] = createSignal<Map<string, FileHashInfo>>(new Map());
  const [hashHistory, setHashHistory] = createSignal<Map<string, HashHistoryEntry[]>>(new Map());

  /** Tracks which files have already had stored hashes imported */
  const importedFiles = new Set<string>();

  // ── Auto-import stored hashes when file info arrives ───────────────────

  createEffect(on(fileInfoMap, (infoMap) => {
    for (const [filePath, info] of infoMap.entries()) {
      if (!importedFiles.has(filePath)) {
        importStoredHashesToHistory(filePath, info);
        importedFiles.add(filePath);
      }
    }
  }));

  // ── importStoredHashesToHistory ────────────────────────────────────────

  /**
   * Import stored hashes from container info into the hash history.
   * Called when container info is loaded to populate history with
   * acquisition-time hashes. Standardized across all container types:
   * E01, L01, AD1, UFED, Raw with companion logs.
   */
  const importStoredHashesToHistory = (filePath: string, info: ContainerInfo) => {
    const history = new Map(hashHistory());
    const existingHistory = history.get(filePath) ?? [];

    // Best fallback date from the container
    const containerFallbackDate =
      info.e01?.acquiry_date ||
      info.l01?.acquiry_date ||
      info.ad1?.companion_log?.acquisition_date ||
      info.ufed?.extraction_info?.start_time ||
      info.companion_log?.acquisition_finished ||
      info.companion_log?.verification_finished ||
      null;

    // Collect all stored hashes from various sources
    const allStoredHashes: Array<{ algorithm: string; hash: string; timestamp?: string | null }> = [];

    // E01/Ex01
    if (info.e01?.stored_hashes) {
      const acquiryDate = info.e01.acquiry_date;
      allStoredHashes.push(
        ...info.e01.stored_hashes.map((sh) => ({
          algorithm: sh.algorithm,
          hash: sh.hash,
          timestamp: sh.timestamp ?? acquiryDate,
        })),
      );
    }

    // L01/Lx01
    if (info.l01?.stored_hashes) {
      const acquiryDate = info.l01.acquiry_date;
      allStoredHashes.push(
        ...info.l01.stored_hashes.map((sh) => ({
          algorithm: sh.algorithm,
          hash: sh.hash,
          timestamp: sh.timestamp ?? acquiryDate,
        })),
      );
    }

    // AD1 companion log
    if (info.ad1?.companion_log) {
      const adLog = info.ad1.companion_log;
      const acquiryDate = adLog.acquisition_date;
      if (adLog.md5_hash) allStoredHashes.push({ algorithm: "MD5", hash: adLog.md5_hash, timestamp: acquiryDate });
      if (adLog.sha1_hash) allStoredHashes.push({ algorithm: "SHA-1", hash: adLog.sha1_hash, timestamp: acquiryDate });
      if (adLog.sha256_hash) allStoredHashes.push({ algorithm: "SHA-256", hash: adLog.sha256_hash, timestamp: acquiryDate });
    }

    // Generic companion log
    if (info.companion_log?.stored_hashes) {
      const logDate = info.companion_log.verification_finished ?? info.companion_log.acquisition_finished;
      allStoredHashes.push(
        ...info.companion_log.stored_hashes.map((sh) => ({
          algorithm: sh.algorithm,
          hash: sh.hash,
          timestamp: sh.timestamp ?? logDate,
        })),
      );
    }

    // UFED
    if (info.ufed?.stored_hashes) {
      const extractionDate = info.ufed.extraction_info?.end_time ?? info.ufed.extraction_info?.start_time;
      allStoredHashes.push(
        ...info.ufed.stored_hashes.map((sh) => ({
          algorithm: sh.algorithm,
          hash: sh.hash,
          timestamp: (sh as { timestamp?: string }).timestamp ?? extractionDate,
        })),
      );
    }

    // Deduplicate and convert to history entries
    const newEntries: HashHistoryEntry[] = [];
    const seen = new Set<string>();

    for (const sh of allStoredHashes) {
      const normalizedHash = sh.hash.toLowerCase().trim();
      const normalizedAlgo = sh.algorithm.toUpperCase().replace(/-/g, "");
      const key = `${normalizedAlgo}:${normalizedHash}`;

      const alreadyExists =
        existingHistory.some(
          (h: HashHistoryEntry) =>
            h.algorithm.toUpperCase().replace(/-/g, "") === normalizedAlgo &&
            h.hash.toLowerCase().trim() === normalizedHash,
        ) || seen.has(key);

      if (!alreadyExists && normalizedHash.length >= 32) {
        seen.add(key);
        const effectiveTimestamp = sh.timestamp ?? containerFallbackDate;
        newEntries.push({
          algorithm: sh.algorithm.toUpperCase(),
          hash: normalizedHash,
          timestamp: effectiveTimestamp ? new Date(effectiveTimestamp) : new Date(0),
          source: "stored",
          verified: undefined,
          verified_against: undefined,
        });
      }
    }

    if (newEntries.length > 0) {
      history.set(filePath, [...newEntries, ...existingHistory]);
      setHashHistory(history);
      log.debug(` Imported ${newEntries.length} stored hashes for ${filePath}`);
    }
  };

  // ── importPreloadedStoredHashes ───────────────────────────────────────

  /**
   * Import pre-loaded stored hashes directly (from wizard step 2).
   * Fast path that doesn't require full ContainerInfo.
   */
  const importPreloadedStoredHashes = (hashMap: Map<string, StoredHash[]>) => {
    const history = new Map(hashHistory());
    let totalImported = 0;

    for (const [filePath, storedHashes] of hashMap.entries()) {
      if (importedFiles.has(filePath)) continue;

      const existingHistory = history.get(filePath) ?? [];
      const newEntries: HashHistoryEntry[] = [];

      for (const sh of storedHashes) {
        const normalizedHash = sh.hash.toLowerCase().trim();
        const normalizedAlgo = sh.algorithm.toUpperCase().replace(/-/g, "");

        const alreadyExists =
          existingHistory.some(
            (h: HashHistoryEntry) =>
              h.algorithm.toUpperCase().replace(/-/g, "") === normalizedAlgo &&
              h.hash.toLowerCase().trim() === normalizedHash,
          ) ||
          newEntries.some(
            (h: HashHistoryEntry) =>
              h.algorithm.toUpperCase().replace(/-/g, "") === normalizedAlgo &&
              h.hash.toLowerCase().trim() === normalizedHash,
          );

        if (!alreadyExists && normalizedHash.length >= 32) {
          newEntries.push({
            algorithm: sh.algorithm.toUpperCase(),
            hash: normalizedHash,
            timestamp: sh.timestamp ? new Date(sh.timestamp) : new Date(0),
            source: "stored",
            verified: sh.verified ?? undefined,
            verified_against: undefined,
          });
        }
      }

      if (newEntries.length > 0) {
        history.set(filePath, [...newEntries, ...existingHistory]);
        importedFiles.add(filePath);
        totalImported += newEntries.length;
      }
    }

    if (totalImported > 0) {
      setHashHistory(history);
      log.debug(` Imported ${totalImported} pre-loaded stored hashes from ${hashMap.size} files`);
    }
  };

  // ── recordHashToHistory ───────────────────────────────────────────────

  /**
   * Record a computed or verified hash into the per-file history.
   * If the computed hash matches a stored hash, updates the stored entry.
   */
  const recordHashToHistory = (
    file: DiscoveredFile,
    algorithm: string,
    hash: string,
    verified?: boolean,
    verifiedAgainst?: string,
  ) => {
    if (!hash || hash.trim().length === 0) {
      log.warn(`recordHashToHistory: Skipping entry with empty hash for ${file.path}`);
      return;
    }

    const history = new Map(hashHistory());
    const existingHistory = history.get(file.path) ?? [];

    // If this computed hash matches a stored hash, update the stored entry
    const updatedHistory = existingHistory.map((entry) => {
      if (
        entry.source === "stored" &&
        entry.algorithm.toUpperCase() === algorithm.toUpperCase() &&
        entry.hash.toLowerCase() === hash.toLowerCase() &&
        verified === true
      ) {
        return { ...entry, verified: true, verified_against: hash };
      }
      return entry;
    });

    const newEntry: HashHistoryEntry = {
      algorithm,
      hash,
      timestamp: new Date(),
      source: verified !== undefined ? "verified" : "computed",
      verified,
      verified_against: verifiedAgainst,
    };
    history.set(file.path, [...updatedHistory, newEntry]);
    setHashHistory(history);
  };

  // ── restoreHashHistory ────────────────────────────────────────────────

  /** Restore hash history from a loaded project. */
  const restoreHashHistory = (
    projectHashHistory: Record<string, Array<{ algorithm: string; hash_value: string; computed_at: string; source?: string }>>,
  ) => {
    const history = new Map<string, HashHistoryEntry[]>();

    for (const [filePath, hashes] of Object.entries(projectHashHistory)) {
      const validHashes = hashes.filter((h) => h.hash_value && h.hash_value.trim().length > 0);
      if (validHashes.length === 0) continue;

      const entries: HashHistoryEntry[] = validHashes.map((h) => {
        let timestamp: Date;
        if (h.computed_at && h.computed_at !== "Invalid Date") {
          const parsed = new Date(h.computed_at);
          timestamp = !isNaN(parsed.getTime()) ? parsed : new Date();
        } else {
          timestamp = new Date();
        }

        return {
          algorithm: h.algorithm || "UNKNOWN",
          hash: h.hash_value,
          timestamp,
          source: (h.source as "computed" | "stored" | "verified") || "computed",
        };
      });
      history.set(filePath, entries);
    }

    setHashHistory(history);
    log.debug(" Restored hash history:", history.size, "files");
  };

  // ── restoreFileHashMap ────────────────────────────────────────────────

  /** Restore computed hashes from a loaded project's evidence cache. */
  const restoreFileHashMap = (cachedHashes: Record<string, { algorithm: string; hash: string; verified?: boolean | null }>) => {
    if (!cachedHashes || Object.keys(cachedHashes).length === 0) return;

    const map = new Map<string, FileHashInfo>();
    for (const [filePath, hashInfo] of Object.entries(cachedHashes)) {
      map.set(filePath, {
        algorithm: hashInfo.algorithm,
        hash: hashInfo.hash,
        verified: hashInfo.verified,
      });
    }
    setFileHashMap(map);
    log.debug(" Restored file hash map:", map.size, "computed hashes");
  };

  // ── clearAll ──────────────────────────────────────────────────────────

  /** Clear all hash manager state (for new project or reset). */
  const clearAll = () => {
    setFileHashMap(new Map());
    setHashHistory(new Map());
    log.debug(" Cleared all state");
  };

  // ── addTransferHashesToHistory ─────────────────────────────────────────

  /** Add multiple hash entries from a transfer/export operation. */
  const addTransferHashesToHistory = (entries: HashHistoryEntry[], sourcePath?: string) => {
    if (entries.length === 0) return;

    const history = new Map(hashHistory());
    const key = sourcePath || "__transfer__";
    const existingHistory = history.get(key) ?? [];

    const newEntries = entries.filter(
      (entry) =>
        !existingHistory.some(
          (existing) =>
            existing.algorithm.toUpperCase() === entry.algorithm.toUpperCase() &&
            existing.hash.toLowerCase() === entry.hash.toLowerCase(),
        ),
    );

    if (newEntries.length > 0) {
      history.set(key, [...existingHistory, ...newEntries]);
      setHashHistory(history);
    }
  };

  // ── getAllStoredHashesSorted ───────────────────────────────────────────

  /** Get all stored hashes from container info, sorted by source then algorithm. */
  const getAllStoredHashesSorted = (info: ContainerInfo | undefined): StoredHash[] => {
    if (!info) return [];

    // Deep clone to escape SolidJS proxy
    let plainInfo: ContainerInfo;
    try {
      plainInfo = JSON.parse(JSON.stringify(info));
    } catch (err) {
      log.debug(" Failed to clone container info:", err);
      return [];
    }

    const allHashes: StoredHash[] = [];

    // E01 container hashes
    if (plainInfo.e01?.stored_hashes) {
      const containerDate = plainInfo.e01.acquiry_date;
      for (const h of plainInfo.e01.stored_hashes) {
        allHashes.push({
          algorithm: h.algorithm || "",
          hash: h.hash || "",
          verified: h.verified ?? null,
          timestamp: h.timestamp || containerDate || null,
          source: h.source || "container",
        });
      }
    }

    // UFED container hashes
    if (plainInfo.ufed?.stored_hashes) {
      const containerDate = plainInfo.ufed.extraction_info?.start_time;
      for (const h of plainInfo.ufed.stored_hashes) {
        allHashes.push({
          algorithm: h.algorithm || "",
          hash: h.hash || "",
          verified: null,
          timestamp: containerDate || null,
          source: "container",
          filename: h.filename || null,
        });
      }
    }

    // Companion log hashes
    if (plainInfo.companion_log?.stored_hashes) {
      const logDate = plainInfo.companion_log.verification_finished || plainInfo.companion_log.acquisition_finished;
      for (const h of plainInfo.companion_log.stored_hashes) {
        allHashes.push({
          algorithm: h.algorithm || "",
          hash: h.hash || "",
          verified: h.verified ?? null,
          timestamp: h.timestamp || logDate || null,
          source: h.source || "companion",
        });
      }
    }

    return allHashes.sort((a, b) => {
      if (a.source === "container" && b.source !== "container") return -1;
      if (b.source === "container" && a.source !== "container") return 1;
      if (a.algorithm !== b.algorithm) return a.algorithm.localeCompare(b.algorithm);
      if (!a.timestamp && !b.timestamp) return 0;
      if (!a.timestamp) return 1;
      if (!b.timestamp) return -1;
      return new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime();
    });
  };

  // ── Public API ────────────────────────────────────────────────────────

  return {
    // State
    fileHashMap,
    setFileHashMap,
    hashHistory,

    // Import / Restore
    importStoredHashesToHistory,
    importPreloadedStoredHashes,
    restoreHashHistory,
    restoreFileHashMap,

    // Record / Transfer
    recordHashToHistory,
    addTransferHashesToHistory,

    // Sorted view
    getAllStoredHashesSorted,

    // Reset
    clearAll,

    // Utility re-export
    formatHashDate,
  };
}

export type HashHistory = ReturnType<typeof useHashHistory>;
