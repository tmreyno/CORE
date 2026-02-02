// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, on } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ask } from "@tauri-apps/plugin-dialog";
import type { DiscoveredFile, ContainerInfo } from "../types";
import { normalizeError, formatHashDate, formatBytes, getBasename } from "../utils";
import { logAuditAction } from "../utils/telemetry";
import type { FileManager } from "./useFileManager";
import { getPreference } from "../components/preferences";

// Import new hash utilities and types
import {
  hashContainer,
  findMatchingStoredHash,
  compareHashes,
} from "./hashUtils";
import {
  HASH_ALGORITHMS,
  HASH_ALGORITHM_MAP,
  type HashAlgorithmName,
  type StoredHashEntry,
  type HashHistoryEntry,
  type FileHashInfo,
} from "../types/hash";

// Re-export FileHashInfo for external use
export type { FileHashInfo } from "../types/hash";

// Legacy type compatibility
import type { StoredHash } from "../types";

// Get initial hash algorithm from preferences
function getInitialHashAlgorithm(): HashAlgorithmName {
  const pref = getPreference("defaultHashAlgorithm");
  return HASH_ALGORITHM_MAP[pref] || HASH_ALGORITHMS.SHA256;
}

export function useHashManager(fileManager: FileManager) {
  console.log("[DEBUG] HashManager: Hook initialized");
  
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
  } = fileManager;
  
  // Hash state - initialize from preferences
  const [selectedHashAlgorithm, setSelectedHashAlgorithm] = createSignal<HashAlgorithmName>(getInitialHashAlgorithm());
  const [fileHashMap, setFileHashMap] = createSignal<Map<string, FileHashInfo>>(new Map());
  
  // Hash history state (per file)
  const [hashHistory, setHashHistory] = createSignal<Map<string, HashHistoryEntry[]>>(new Map());

  // Import stored hashes from container info into hash history
  // Called when container info is loaded to populate history with acquisition-time hashes
  // Standardized across all container types: E01, L01, AD1, UFED, Raw with companion logs
  const importStoredHashesToHistory = (filePath: string, info: ContainerInfo) => {
    const history = new Map(hashHistory());
    const existingHistory = history.get(filePath) ?? [];
    
    // Get the best fallback date from the container (acquisition date, extraction date, etc.)
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
    
    // E01/Ex01 stored hashes
    if (info.e01?.stored_hashes) {
      const acquiryDate = info.e01.acquiry_date;
      allStoredHashes.push(...info.e01.stored_hashes.map(sh => ({
        algorithm: sh.algorithm,
        hash: sh.hash,
        timestamp: sh.timestamp ?? acquiryDate
      })));
    }
    
    // L01/Lx01 stored hashes
    if (info.l01?.stored_hashes) {
      const acquiryDate = info.l01.acquiry_date;
      allStoredHashes.push(...info.l01.stored_hashes.map(sh => ({
        algorithm: sh.algorithm,
        hash: sh.hash,
        timestamp: sh.timestamp ?? acquiryDate
      })));
    }
    
    // AD1 companion log hashes
    if (info.ad1?.companion_log) {
      const log = info.ad1.companion_log;
      const acquiryDate = log.acquisition_date;
      if (log.md5_hash) {
        allStoredHashes.push({ algorithm: 'MD5', hash: log.md5_hash, timestamp: acquiryDate });
      }
      if (log.sha1_hash) {
        allStoredHashes.push({ algorithm: 'SHA-1', hash: log.sha1_hash, timestamp: acquiryDate });
      }
      if (log.sha256_hash) {
        allStoredHashes.push({ algorithm: 'SHA-256', hash: log.sha256_hash, timestamp: acquiryDate });
      }
    }
    
    // Generic companion log hashes
    if (info.companion_log?.stored_hashes) {
      const logDate = info.companion_log.verification_finished ?? info.companion_log.acquisition_finished;
      allStoredHashes.push(...info.companion_log.stored_hashes.map(sh => ({
        algorithm: sh.algorithm,
        hash: sh.hash,
        timestamp: sh.timestamp ?? logDate
      })));
    }
    
    // UFED stored hashes
    if (info.ufed?.stored_hashes) {
      const extractionDate = info.ufed.extraction_info?.end_time ?? info.ufed.extraction_info?.start_time;
      allStoredHashes.push(...info.ufed.stored_hashes.map(sh => ({
        algorithm: sh.algorithm,
        hash: sh.hash,
        timestamp: (sh as { timestamp?: string }).timestamp ?? extractionDate
      })));
    }
    
    // Deduplicate and convert to history entries
    const newEntries: HashHistoryEntry[] = [];
    const seen = new Set<string>();
    
    for (const sh of allStoredHashes) {
      const normalizedHash = sh.hash.toLowerCase().trim();
      const normalizedAlgo = sh.algorithm.toUpperCase().replace(/-/g, '');
      const key = `${normalizedAlgo}:${normalizedHash}`;
      
      // Check if already in existing history or new entries
      const alreadyExists = 
        existingHistory.some((h: HashHistoryEntry) => 
          h.algorithm.toUpperCase().replace(/-/g, '') === normalizedAlgo && 
          h.hash.toLowerCase().trim() === normalizedHash
        ) || seen.has(key);
      
      if (!alreadyExists && normalizedHash.length >= 32) { // Minimum MD5 length
        seen.add(key);
        const effectiveTimestamp = sh.timestamp ?? containerFallbackDate;
        newEntries.push({
          algorithm: sh.algorithm.toUpperCase(),
          hash: normalizedHash,
          timestamp: effectiveTimestamp ? new Date(effectiveTimestamp) : new Date(0),
          source: "stored",
          verified: undefined,
          verified_against: undefined
        });
      }
    }
    
    if (newEntries.length > 0) {
      // Prepend stored hashes (they come first chronologically)
      history.set(filePath, [...newEntries, ...existingHistory]);
      setHashHistory(history);
      console.debug(`[HashManager] Imported ${newEntries.length} stored hashes for ${filePath}`);
    }
  };

  // Import pre-loaded stored hashes directly (from wizard step 2)
  // This is the fast path that doesn't require full ContainerInfo
  const importPreloadedStoredHashes = (hashMap: Map<string, StoredHash[]>) => {
    const history = new Map(hashHistory());
    let totalImported = 0;
    
    for (const [filePath, storedHashes] of hashMap.entries()) {
      if (importedFiles.has(filePath)) continue; // Already imported
      
      const existingHistory = history.get(filePath) ?? [];
      const newEntries: HashHistoryEntry[] = [];
      
      for (const sh of storedHashes) {
        const normalizedHash = sh.hash.toLowerCase().trim();
        const normalizedAlgo = sh.algorithm.toUpperCase().replace(/-/g, '');
        
        // Check if already exists
        const alreadyExists = existingHistory.some((h: HashHistoryEntry) => 
          h.algorithm.toUpperCase().replace(/-/g, '') === normalizedAlgo && 
          h.hash.toLowerCase().trim() === normalizedHash
        ) || newEntries.some((h: HashHistoryEntry) => 
          h.algorithm.toUpperCase().replace(/-/g, '') === normalizedAlgo && 
          h.hash.toLowerCase().trim() === normalizedHash
        );
        
        if (!alreadyExists && normalizedHash.length >= 32) {
          newEntries.push({
            algorithm: sh.algorithm.toUpperCase(),
            hash: normalizedHash,
            timestamp: sh.timestamp ? new Date(sh.timestamp) : new Date(0),
            source: "stored",
            verified: sh.verified ?? undefined,
            verified_against: undefined
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
      console.debug(`[HashManager] Imported ${totalImported} pre-loaded stored hashes from ${hashMap.size} files`);
    }
  };

  // Track which files have had their stored hashes imported
  const importedFiles = new Set<string>();
  
  // Watch for new file info and auto-import stored hashes
  createEffect(on(fileInfoMap, (infoMap) => {
    for (const [filePath, info] of infoMap.entries()) {
      if (!importedFiles.has(filePath)) {
        importStoredHashesToHistory(filePath, info);
        importedFiles.add(filePath);
      }
    }
  }));

  // Add hash to history when computed, also update stored hash entries if verified
  const recordHashToHistory = (file: DiscoveredFile, algorithm: string, hash: string, verified?: boolean, verifiedAgainst?: string) => {
    // Guard against empty/invalid hashes
    if (!hash || hash.trim().length === 0) {
      console.warn(`[HashManager] recordHashToHistory: Skipping entry with empty hash for ${file.path}`);
      return;
    }
    
    const history = new Map(hashHistory());
    const existingHistory = history.get(file.path) ?? [];
    
    // If this computed hash matches a stored hash, update the stored entry too
    const updatedHistory = existingHistory.map(entry => {
      // Check if this stored hash matches the computed one
      if (entry.source === "stored" && 
          entry.algorithm.toUpperCase() === algorithm.toUpperCase() &&
          entry.hash.toLowerCase() === hash.toLowerCase() &&
          verified === true) {
        // Update the stored entry to show it was verified
        return { ...entry, verified: true, verified_against: hash };
      }
      return entry;
    });
    
    // Create new array to ensure reactivity (don't mutate existing)
    const newEntry: HashHistoryEntry = {
      algorithm,
      hash,
      timestamp: new Date(),
      source: verified !== undefined ? "verified" : "computed",
      verified,
      verified_against: verifiedAgainst
    };
    history.set(file.path, [...updatedHistory, newEntry]);
    setHashHistory(history);
  };

  // Restore hash history from a loaded project
  const restoreHashHistory = (projectHashHistory: Record<string, Array<{ algorithm: string; hash_value: string; computed_at: string; source?: string }>>) => {
    const history = new Map<string, HashHistoryEntry[]>();
    
    for (const [filePath, hashes] of Object.entries(projectHashHistory)) {
      // Filter out entries with missing/invalid hash values
      const validHashes = hashes.filter(h => h.hash_value && h.hash_value.trim().length > 0);
      if (validHashes.length === 0) continue;
      
      const entries: HashHistoryEntry[] = validHashes.map(h => {
        // Safely parse date, handling invalid dates
        let timestamp: Date;
        if (h.computed_at && h.computed_at !== 'Invalid Date') {
          const parsed = new Date(h.computed_at);
          timestamp = !isNaN(parsed.getTime()) ? parsed : new Date();
        } else {
          timestamp = new Date(); // Fallback to current time
        }
        
        return {
          algorithm: h.algorithm || 'UNKNOWN',
          hash: h.hash_value,
          timestamp,
          source: (h.source as "computed" | "stored" | "verified") || "computed",
        };
      });
      history.set(filePath, entries);
    }
    
    setHashHistory(history);
    console.log("[HashManager] Restored hash history:", history.size, "files");
  };

  // Restore file hash map from a loaded project's evidence cache
  // This restores computed hashes to avoid re-hashing files on project load
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
    console.log("[HashManager] Restored file hash map:", map.size, "computed hashes");
  };

  // Clear all hash manager state (for new project or reset)
  const clearAll = () => {
    setFileHashMap(new Map());
    setHashHistory(new Map());
    setSelectedHashAlgorithm(HASH_ALGORITHMS.SHA256);
    console.log("[HashManager] Cleared all state");
  };

  // Add multiple hash entries from a transfer operation
  // These entries are keyed by source file path if available, or fall back to a generic "transfer" key
  const addTransferHashesToHistory = (entries: HashHistoryEntry[], sourcePath?: string) => {
    if (entries.length === 0) return;
    
    const history = new Map(hashHistory());
    const key = sourcePath || "__transfer__";
    const existingHistory = history.get(key) ?? [];
    
    // Avoid duplicates by checking algorithm + hash
    const newEntries = entries.filter(entry => {
      return !existingHistory.some(existing => 
        existing.algorithm.toUpperCase() === entry.algorithm.toUpperCase() &&
        existing.hash.toLowerCase() === entry.hash.toLowerCase()
      );
    });
    
    if (newEntries.length > 0) {
      history.set(key, [...existingHistory, ...newEntries]);
      setHashHistory(history);
    }
  };

  // Hash a single file
  const hashSingleFile = async (file: DiscoveredFile): Promise<string | undefined> => {
    console.log(`[DEBUG] HashManager: hashSingleFile called for ${file.filename}, path=${file.path}, size=${file.size}`);
    
    // Check if confirmation is required
    if (getPreference("confirmBeforeHash")) {
      console.log("[DEBUG] HashManager: Showing confirmation dialog");
      const confirmed = await ask(
        `Compute hash for "${file.filename}" (${formatBytes(file.size)})?\n\nThis may take some time for large files.`,
        { title: "Confirm Hash", kind: "info" }
      );
      if (!confirmed) {
        console.log("[DEBUG] HashManager: User cancelled hash operation");
        return;
      }
    }
    
    const algorithm = selectedHashAlgorithm();
    console.log(`[DEBUG] HashManager: Starting hash with algorithm=${algorithm}`);
    updateFileStatus(file.path, "hashing", 0);
    
    // Listen for progress events, filtering by the file path to avoid mixing progress from multiple concurrent hashes
    const unlisten = await listen<{ path: string; percent: number }>("verify-progress", (e) => {
      if (e.payload.path === file.path) {
        updateFileStatus(file.path, "hashing", e.payload.percent);
      }
    });
    
    try {
      // Get file extension for hash routing
      const extension = file.filename.split('.').pop()?.toLowerCase() || '';
      
      // Compute hash using unified hash utility
      let hash: string;
      try {
        // Try to use the new unified hash system
        // Note: hashContainer expects normalized algorithm names (e.g., "SHA-256" not "sha256")
        hash = await hashContainer(file.path, extension, algorithm);
      } catch (err) {
        // Fallback to legacy system for backwards compatibility
        const ctype = file.container_type.toLowerCase();
        if (ctype.includes("e01") || ctype.includes("encase") || ctype.includes("ex01")) {
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
      storedHashes.push(...(info?.e01?.stored_hashes?.map(sh => ({
        algorithm: sh.algorithm,
        hash: sh.hash,
        source: 'container' as const
      })) ?? []));
      storedHashes.push(...(info?.l01?.stored_hashes?.map(sh => ({
        algorithm: sh.algorithm,
        hash: sh.hash,
        source: 'container' as const
      })) ?? []));
      
      // Collect from companion logs
      storedHashes.push(...(info?.companion_log?.stored_hashes?.map(sh => ({
        algorithm: sh.algorithm,
        hash: sh.hash,
        source: 'companion' as const
      })) ?? []));
      
      // Collect from AD1 companion log
      if (info?.ad1?.companion_log) {
        const log = info.ad1.companion_log;
        if (log.md5_hash) storedHashes.push({ algorithm: 'MD5', hash: log.md5_hash, source: 'companion' as const });
        if (log.sha1_hash) storedHashes.push({ algorithm: 'SHA-1', hash: log.sha1_hash, source: 'companion' as const });
        if (log.sha256_hash) storedHashes.push({ algorithm: 'SHA-256', hash: log.sha256_hash, source: 'companion' as const });
      }
      
      // Collect from UFED (match by filename)
      const fileName = getBasename(file.path);
      const ufedMatch = info?.ufed?.stored_hashes?.find(sh => 
        sh.filename.toLowerCase() === fileName.toLowerCase()
      );
      if (ufedMatch) {
        storedHashes.push({
          algorithm: ufedMatch.algorithm,
          hash: ufedMatch.hash,
          source: 'container' as const,
          filename: ufedMatch.filename
        });
      }
      
      // Find matching stored hash using utility function
      const matchingStored = findMatchingStoredHash(hash, algorithm, storedHashes);
      
      // Check hash history for self-verification (previously computed matches)
      const history = hashHistory().get(file.path) ?? [];
      const matchingComputedHistory = history.find(h => 
        h.source === 'computed' && 
        compareHashes(h.hash, hash, h.algorithm, algorithm)
      );
      
      // Determine verification status
      let verified: boolean | null;
      let verifiedAgainst: string | undefined;
      
      if (matchingStored) {
        verified = compareHashes(hash, matchingStored.hash, algorithm, matchingStored.algorithm);
        verifiedAgainst = matchingStored.hash;
        console.log(`[DEBUG] HashManager: Hash VERIFIED against stored hash, verified=${verified}`);
      } else if (matchingComputedHistory) {
        verified = true;
        verifiedAgainst = matchingComputedHistory.hash;
        console.log(`[DEBUG] HashManager: Hash VERIFIED against previous computation`);
      } else {
        verified = null;
        console.log(`[DEBUG] HashManager: No stored hash found for verification`);
      }
      
      // Update state
      console.log(`[DEBUG] HashManager: Hash complete: ${algorithm}=${hash.substring(0, 16)}... verified=${verified}`);
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
      console.warn(`Hash computation failed: ${errMsg}`);
      updateFileStatus(file.path, "error", 0, errMsg);
      throw err;
    } finally {
      unlisten();
    }
  };

  // Hash selected files
  const hashSelectedFiles = async (): Promise<void> => {
    const files = discoveredFiles().filter(f => selectedFiles().has(f.path));
    if (!files.length) {
      setError("No files selected");
      return;
    }
    
    console.log(`[DEBUG] HashManager: hashSelectedFiles starting with ${files.length} files`);
    
    const numCores = navigator.hardwareConcurrency || 4;
    
    // Set all selected files to hashing status immediately
    files.forEach(f => updateFileStatus(f.path, "hashing", 0));
    setWorking(`# Hashing 0/${files.length} files (${numCores} cores)...`);
    
    // Load file info in parallel with hashing setup (non-blocking)
    // This allows the UI to show progress immediately
    const filesToLoad = files.filter(f => !fileInfoMap().has(f.path));
    if (filesToLoad.length > 0) {
      console.log(`[DEBUG] HashManager: Loading info for ${filesToLoad.length} files in background`);
      // Start loading in background - don't await
      Promise.all(filesToLoad.map(async (file) => {
        try {
          await loadFileInfo(file, false);
        } catch (err) {
          console.debug(`[HashManager] Failed to load info for ${file.path}:`, err);
        }
      })).catch(() => {}); // Ignore errors, non-critical
    }
    
    // Track completed files for immediate UI updates
    let completedCount = 0;
    let verifiedCount = 0;
    let failedCount = 0;
    
    // Listen for batch progress events
    const unlisten = await listen<{ 
      path: string; 
      status: string; 
      percent: number; 
      files_completed: number; 
      files_total: number;
      chunks_processed?: number;
      chunks_total?: number;
      hash?: string;
      algorithm?: string;
      error?: string;
    }>(
      "batch-progress",
      (e) => {
        const { path, status, percent, files_completed: _files_completed, files_total: _files_total, chunks_processed, chunks_total, hash, algorithm, error } = e.payload;
        
        if (status === "progress" || status === "started") {
          updateFileStatus(path, "hashing", percent, undefined, chunks_processed, chunks_total);
        } else if (status === "completed" && hash && algorithm) {
          // Immediately update hash map and verify when a file completes
          const file = files.find(f => f.path === path);
          const info = fileInfoMap().get(path);
          const storedHashes = [...(info?.e01?.stored_hashes ?? []), ...(info?.companion_log?.stored_hashes ?? [])];
          // Also check UFED stored hashes (from .ufd file) - match by algorithm and filename
          const fileName = getBasename(path);
          const ufedStoredHashes = info?.ufed?.stored_hashes ?? [];
          const matchingUfedStored = ufedStoredHashes.find(sh => 
            sh.algorithm.toLowerCase() === algorithm.toLowerCase() && 
            sh.filename.toLowerCase() === fileName.toLowerCase()
          );
          const matchingStored = storedHashes.find(sh => sh.algorithm.toLowerCase() === algorithm.toLowerCase())
            ?? (matchingUfedStored ? { algorithm: matchingUfedStored.algorithm, hash: matchingUfedStored.hash } : undefined);
          
          // Also check hash history for matching hash (self-verification)
          const history = hashHistory().get(path) ?? [];
          const matchingHistory = history.find(h => 
            h.algorithm.toLowerCase() === algorithm.toLowerCase() && 
            h.hash.toLowerCase() === hash.toLowerCase()
          );
          
          // Verified if matches stored OR matches previous hash in history
          const verified = matchingStored 
            ? hash.toLowerCase() === matchingStored.hash.toLowerCase() 
            : matchingHistory ? true : null;
          const verifiedAgainst = matchingStored?.hash ?? matchingHistory?.hash;
          
          if (verified === true) verifiedCount++;
          else if (verified === false) failedCount++;
          completedCount++;
          
          console.log(`[DEBUG] HashManager: File completed: ${path}, completedCount=${completedCount}/${files.length}`);
          
          // Update hash map immediately
          const hashMap = new Map(fileHashMap());
          hashMap.set(path, { algorithm, hash, verified });
          setFileHashMap(hashMap);
          
          updateFileStatus(path, "hashed", 100);
          
          if (file) {
            recordHashToHistory(file, algorithm, hash, verified ?? undefined, verifiedAgainst);
          }
          
          // Update status with local count (more reliable than backend events)
          setWorking(`# Hashing ${completedCount}/${files.length} files completed`);
        } else if (status === "error") {
          updateFileStatus(path, "error", 0, error || "Unknown error");
          completedCount++;
          console.log(`[DEBUG] HashManager: File error: ${path}, completedCount=${completedCount}/${files.length}`);
          setWorking(`# Hashing ${completedCount}/${files.length} files (1 error)`);
        }
        
        // Show decompression progress in status if available
        if (chunks_processed !== undefined && chunks_total !== undefined && chunks_total > 0) {
          setWorking(`# ${completedCount}/${files.length} files | ${chunks_processed.toLocaleString()}/${chunks_total.toLocaleString()} chunks`);
        } else if (status === "progress" || status === "started") {
          // Only update with backend count for progress events, not completed
          setWorking(`# Hashing ${completedCount}/${files.length} files`);
        }
      }
    );
    
    try {
      // Wait for batch_hash to complete (results already processed via events)
      await invoke<{ path: string; algorithm: string; hash?: string; error?: string }[]>(
        "batch_hash",
        { files: files.map(f => ({ path: f.path, containerType: f.container_type })), algorithm: selectedHashAlgorithm() }
      );
      
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
      files.forEach(f => updateFileStatus(f.path, "error", 0, normalizeError(err)));
    } finally {
      unlisten();
    }
  };

  // Hash all files
  const hashAllFiles = async (): Promise<void> => {
    const files = discoveredFiles();
    if (!files.length) {
      setError("No files discovered");
      return;
    }
    setSelectedFiles(new Set(files.map(f => f.path)));
    await hashSelectedFiles();
  };

  // Get all stored hashes sorted by source and timestamp
  const getAllStoredHashesSorted = (info: ContainerInfo | undefined): StoredHash[] => {
    if (!info) return [];
    
    // Deep clone the entire info to escape SolidJS proxy completely
    let plainInfo: ContainerInfo;
    try {
      plainInfo = JSON.parse(JSON.stringify(info));
    } catch (err) {
      // JSON serialization failed (possibly circular reference), return empty
      console.debug("[HashManager] Failed to clone container info:", err);
      return [];
    }
    
    const allHashes: StoredHash[] = [];
    
    // E01 container hashes
    if (plainInfo.e01?.stored_hashes) {
      const containerDate = plainInfo.e01.acquiry_date;
      for (const h of plainInfo.e01.stored_hashes) {
        allHashes.push({
          algorithm: h.algorithm || '',
          hash: h.hash || '',
          verified: h.verified ?? null,
          timestamp: h.timestamp || containerDate || null,
          source: h.source || 'container',
        });
      }
    }
    
    // UFED container hashes (include filename since UFED has per-file hashes)
    if (plainInfo.ufed?.stored_hashes) {
      const containerDate = plainInfo.ufed.extraction_info?.start_time;
      for (const h of plainInfo.ufed.stored_hashes) {
        allHashes.push({
          algorithm: h.algorithm || '',
          hash: h.hash || '',
          verified: null,
          timestamp: containerDate || null,
          source: 'container',
          filename: h.filename || null,
        });
      }
    }
    
    // Companion log hashes
    if (plainInfo.companion_log?.stored_hashes) {
      const logDate = plainInfo.companion_log.verification_finished || plainInfo.companion_log.acquisition_finished;
      for (const h of plainInfo.companion_log.stored_hashes) {
        allHashes.push({
          algorithm: h.algorithm || '',
          hash: h.hash || '',
          verified: h.verified ?? null,
          timestamp: h.timestamp || logDate || null,
          source: h.source || 'companion',
        });
      }
    }
    
    const sortedHashes = allHashes.sort((a, b) => {
      if (a.source === 'container' && b.source !== 'container') return -1;
      if (b.source === 'container' && a.source !== 'container') return 1;
      if (a.algorithm !== b.algorithm) return a.algorithm.localeCompare(b.algorithm);
      if (!a.timestamp && !b.timestamp) return 0;
      if (!a.timestamp) return 1;
      if (!b.timestamp) return -1;
      return new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime();
    });
    
    return sortedHashes;
  };

  return {
    // State
    selectedHashAlgorithm,
    setSelectedHashAlgorithm,
    fileHashMap,
    setFileHashMap,
    hashHistory,
    
    // Actions
    hashSingleFile,
    hashSelectedFiles,
    hashAllFiles,
    importStoredHashesToHistory,
    importPreloadedStoredHashes,
    addTransferHashesToHistory,
    restoreHashHistory,
    restoreFileHashMap,
    clearAll,
    
    // Helpers
    getAllStoredHashesSorted,
    formatHashDate,
    recordHashToHistory,
  };
}

export type HashManager = ReturnType<typeof useHashManager>;
