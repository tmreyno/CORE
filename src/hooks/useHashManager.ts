// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, on } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { DiscoveredFile, ContainerInfo, SegmentHashResult, HashHistoryEntry, HashAlgorithm, StoredHash } from "../types";
import { normalizeError } from "../utils";
import type { FileManager } from "./useFileManager";

export interface FileHashInfo {
  algorithm: string;
  hash: string;
  verified?: boolean | null;
}

export function useHashManager(fileManager: FileManager) {
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
  
  // Hash state
  const [selectedHashAlgorithm, setSelectedHashAlgorithm] = createSignal<HashAlgorithm>("sha256");
  const [fileHashMap, setFileHashMap] = createSignal<Map<string, FileHashInfo>>(new Map());
  
  // Segment verification state
  const [segmentResults, setSegmentResults] = createSignal<Map<string, SegmentHashResult[]>>(new Map());
  const [segmentVerifyProgress, setSegmentVerifyProgress] = createSignal<{ segment: string; percent: number; completed: number; total: number } | null>(null);
  
  // Hash history state (per file)
  const [hashHistory, setHashHistory] = createSignal<Map<string, HashHistoryEntry[]>>(new Map());

  // Import stored hashes from container info into hash history
  // Called when container info is loaded to populate history with acquisition-time hashes
  // Standardized across all container types: E01, L01, AD1, UFED, Raw with companion logs
  const importStoredHashesToHistory = (filePath: string, info: ContainerInfo) => {
    const history = new Map(hashHistory());
    const existingHistory = history.get(filePath) ?? [];
    const newEntries: HashHistoryEntry[] = [];
    
    // Helper to add stored hash if not already in history
    const addStoredHash = (algo: string, hash: string, timestamp?: string | null, _source: string = "stored") => {
      // Normalize hash for comparison
      const normalizedHash = hash.toLowerCase().trim();
      const normalizedAlgo = algo.toUpperCase().replace(/-/g, '');
      
      // Check if this exact hash is already in history
      const alreadyExists = existingHistory.some(h => 
        h.algorithm.toUpperCase().replace(/-/g, '') === normalizedAlgo && 
        h.hash.toLowerCase().trim() === normalizedHash
      ) || newEntries.some(h => 
        h.algorithm.toUpperCase().replace(/-/g, '') === normalizedAlgo && 
        h.hash.toLowerCase().trim() === normalizedHash
      );
      
      if (!alreadyExists && normalizedHash.length >= 32) { // Minimum MD5 length
        newEntries.push({
          algorithm: algo.toUpperCase(),
          hash: normalizedHash,
          timestamp: timestamp ? new Date(timestamp) : new Date(),
          source: "stored",
          verified: undefined, // Will be set when verified
          verified_against: undefined
        });
      }
    };
    
    // === E01/Ex01 stored hashes (from EWF container) ===
    if (info.e01?.stored_hashes) {
      const acquiryDate = info.e01.acquiry_date;
      for (const sh of info.e01.stored_hashes) {
        addStoredHash(sh.algorithm, sh.hash, sh.timestamp ?? acquiryDate);
      }
    }
    
    // === L01/Lx01 stored hashes (logical EWF container) ===
    if (info.l01?.stored_hashes) {
      const acquiryDate = info.l01.acquiry_date;
      for (const sh of info.l01.stored_hashes) {
        addStoredHash(sh.algorithm, sh.hash, sh.timestamp ?? acquiryDate);
      }
    }
    
    // === AD1 companion log hashes ===
    if (info.ad1?.companion_log) {
      const log = info.ad1.companion_log;
      const acquiryDate = log.acquisition_date;
      
      // AD1 stores hashes as separate fields
      if (log.md5_hash) {
        addStoredHash('MD5', log.md5_hash, acquiryDate);
      }
      if (log.sha1_hash) {
        addStoredHash('SHA-1', log.sha1_hash, acquiryDate);
      }
      if (log.sha256_hash) {
        addStoredHash('SHA-256', log.sha256_hash, acquiryDate);
      }
    }
    
    // === Companion log stored hashes (generic - for raw images, etc.) ===
    if (info.companion_log?.stored_hashes) {
      const logDate = info.companion_log.verification_finished 
        ?? info.companion_log.acquisition_finished;
      for (const sh of info.companion_log.stored_hashes) {
        addStoredHash(sh.algorithm, sh.hash, sh.timestamp ?? logDate);
      }
    }
    
    // === UFED stored hashes (from .ufd file) ===
    if (info.ufed?.stored_hashes) {
      const extractionDate = info.ufed.extraction_info?.end_time 
        ?? info.ufed.extraction_info?.start_time;
      for (const sh of info.ufed.stored_hashes) {
        // Use extraction timestamp if available
        const timestamp = (sh as { timestamp?: string }).timestamp ?? extractionDate;
        addStoredHash(sh.algorithm, sh.hash, timestamp);
      }
    }
    
    if (newEntries.length > 0) {
      // Prepend stored hashes (they come first chronologically)
      history.set(filePath, [...newEntries, ...existingHistory]);
      setHashHistory(history);
      console.debug(`[HashManager] Imported ${newEntries.length} stored hashes for ${filePath}`);
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

  // Hash a single file
  const hashSingleFile = async (file: DiscoveredFile) => {
    const algorithm = selectedHashAlgorithm();
    updateFileStatus(file.path, "hashing", 0);
    // Listen for progress events, filtering by the file path to avoid mixing progress from multiple concurrent hashes
    const unlisten = await listen<{ path: string; percent: number }>("verify-progress", (e) => {
      if (e.payload.path === file.path) {
        updateFileStatus(file.path, "hashing", e.payload.percent);
      }
    });
    try {
      let hash: string;
      const ctype = file.container_type.toLowerCase();
      if (ctype.includes("e01") || ctype.includes("encase") || ctype.includes("ex01")) {
        hash = await invoke<string>("e01_v3_verify", { inputPath: file.path, algorithm });
      } else if (ctype.includes("ad1")) {
        // AD1 containers - hash the segment files directly (not internal files)
        try {
          hash = await invoke<string>("ad1_hash_segments", { inputPath: file.path, algorithm });
        } catch (ad1Err) {
          // AD1 may fail if segments are missing - report gracefully
          const errMsg = normalizeError(ad1Err);
          console.warn(`AD1 hash failed: ${errMsg}`);
          updateFileStatus(file.path, "error", 0, errMsg);
          unlisten();
          return;
        }
      } else if (ctype.includes("raw") || ctype.includes("dd")) {
        hash = await invoke<string>("raw_verify", { inputPath: file.path, algorithm });
      } else if (ctype.includes("ufed") || ctype.includes("zip") || ctype.includes("archive") || ctype.includes("tar") || ctype.includes("7z")) {
        // UFED and archive containers - hash the file directly
        hash = await invoke<string>("raw_verify", { inputPath: file.path, algorithm });
      } else {
        // Unknown container type - try raw verification
        try {
          hash = await invoke<string>("raw_verify", { inputPath: file.path, algorithm });
        } catch (rawErr) {
          const errMsg = normalizeError(rawErr);
          console.warn(`Verification failed for unknown type: ${errMsg}`);
          updateFileStatus(file.path, "error", 0, errMsg);
          unlisten();
          return;
        }
      }
      
      // Check if there's a stored hash to compare against
      const info = fileInfoMap().get(file.path);
      const storedHashes = [...(info?.e01?.stored_hashes ?? []), ...(info?.companion_log?.stored_hashes ?? [])];
      // Also check UFED stored hashes (from .ufd file) - match by algorithm and filename
      const fileName = file.path.split('/').pop() ?? '';
      const ufedStoredHashes = info?.ufed?.stored_hashes ?? [];
      const matchingUfedStored = ufedStoredHashes.find(sh => 
        sh.algorithm.toLowerCase() === algorithm.toLowerCase() && 
        sh.filename.toLowerCase() === fileName.toLowerCase()
      );
      const matchingStored = storedHashes.find(sh => sh.algorithm.toLowerCase() === algorithm.toLowerCase()) 
        ?? (matchingUfedStored ? { algorithm: matchingUfedStored.algorithm, hash: matchingUfedStored.hash } : undefined);
      
      // Also check hash history for matching hash (self-verification)
      const history = hashHistory().get(file.path) ?? [];
      const matchingHistory = history.find(h => 
        h.algorithm.toLowerCase() === algorithm.toLowerCase() && 
        h.hash.toLowerCase() === hash.toLowerCase()
      );
      
      // Verified if matches stored OR matches previous hash in history
      const verified = matchingStored 
        ? hash.toLowerCase() === matchingStored.hash.toLowerCase() 
        : matchingHistory ? true : null;
      const verifiedAgainst = matchingStored?.hash ?? matchingHistory?.hash;
      
      const m = new Map(fileHashMap());
      m.set(file.path, { algorithm: algorithm.toUpperCase(), hash, verified });
      setFileHashMap(m);
      updateFileStatus(file.path, "hashed", 100);
      
      recordHashToHistory(file, algorithm.toUpperCase(), hash, verified ?? undefined, verifiedAgainst);
      
      return hash;
    } catch (err) {
      updateFileStatus(file.path, "error", 0, normalizeError(err));
      throw err;
    } finally {
      unlisten();
    }
  };

  // Hash selected files
  const hashSelectedFiles = async () => {
    const files = discoveredFiles().filter(f => selectedFiles().has(f.path));
    if (!files.length) {
      setError("No files selected");
      return;
    }
    
    const numCores = navigator.hardwareConcurrency || 4;
    setWorking(`Loading file info for ${files.length} file(s)...`);
    
    // First, load file info for all files that don't have it yet
    const filesToLoad = files.filter(f => !fileInfoMap().has(f.path));
    for (const file of filesToLoad) {
      try {
        await loadFileInfo(file, false);
      } catch (err) {
        // Non-critical: file info loading failed, continue with hashing anyway
        console.debug(`[HashManager] Failed to load info for ${file.path}:`, err);
      }
    }
    
    setWorking(`# Hashing ${files.length} file(s) in parallel (${numCores} cores)...`);
    
    // Set all selected files to hashing status
    files.forEach(f => updateFileStatus(f.path, "hashing", 0));
    
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
        const { path, status, percent, files_completed, files_total, chunks_processed, chunks_total, hash, algorithm, error } = e.payload;
        
        if (status === "progress" || status === "started") {
          updateFileStatus(path, "hashing", percent, undefined, chunks_processed, chunks_total);
        } else if (status === "completed" && hash && algorithm) {
          // Immediately update hash map and verify when a file completes
          const file = files.find(f => f.path === path);
          const info = fileInfoMap().get(path);
          const storedHashes = [...(info?.e01?.stored_hashes ?? []), ...(info?.companion_log?.stored_hashes ?? [])];
          // Also check UFED stored hashes (from .ufd file) - match by algorithm and filename
          const fileName = path.split('/').pop() ?? '';
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
          
          // Update hash map immediately
          const hashMap = new Map(fileHashMap());
          hashMap.set(path, { algorithm, hash, verified });
          setFileHashMap(hashMap);
          
          updateFileStatus(path, "hashed", 100);
          
          if (file) {
            recordHashToHistory(file, algorithm, hash, verified ?? undefined, verifiedAgainst);
          }
        } else if (status === "error") {
          updateFileStatus(path, "error", 0, error || "Unknown error");
          completedCount++;
        }
        
        // Show decompression progress in status if available
        if (chunks_processed !== undefined && chunks_total !== undefined && chunks_total > 0) {
          setWorking(`# ${files_completed}/${files_total} files | ${chunks_processed.toLocaleString()}/${chunks_total.toLocaleString()} chunks`);
        } else {
          setWorking(`# Hashing ${files_completed}/${files_total} files`);
        }
      }
    );
    
    try {
      // Wait for batch_hash to complete (results already processed via events)
      await invoke<{ path: string; algorithm: string; hash?: string; error?: string }[]>(
        "batch_hash",
        { files: files.map(f => ({ path: f.path, container_type: f.container_type })), algorithm: selectedHashAlgorithm() }
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
  const hashAllFiles = async () => {
    const files = discoveredFiles();
    if (!files.length) {
      setError("No files discovered");
      return;
    }
    setSelectedFiles(new Set(files.map(f => f.path)));
    await hashSelectedFiles();
  };

  // Verify individual segments
  const verifySegments = async (file: DiscoveredFile) => {
    const info = fileInfoMap().get(file.path);
    const isE01 = file.container_type.toLowerCase().includes("e01") || file.container_type.toLowerCase().includes("encase");
    
    const expectedHashes = info?.companion_log?.segment_hashes ?? [];
    const algorithm = expectedHashes.length > 0 ? expectedHashes[0].algorithm.toLowerCase() : selectedHashAlgorithm();
    
    setWorking(`Verifying segments with ${algorithm.toUpperCase()}...`);
    updateFileStatus(file.path, "verifying-segments", 0);
    setSegmentVerifyProgress({ segment: "", percent: 0, completed: 0, total: 0 });
    
    const unlisten = await listen<{ segment_name: string; segment_number: number; percent: number; segments_completed: number; segments_total: number }>(
      "segment-verify-progress",
      (e) => {
        setSegmentVerifyProgress({
          segment: e.payload.segment_name,
          percent: e.payload.percent,
          completed: e.payload.segments_completed,
          total: e.payload.segments_total
        });
        setWorking(`Verifying segment ${e.payload.segment_name} (${e.payload.segments_completed}/${e.payload.segments_total})...`);
      }
    );
    
    try {
      const command = isE01 ? "e01_verify_segments" : "raw_verify_segments";
      const results = await invoke<SegmentHashResult[]>(command, {
        inputPath: file.path,
        algorithm,
        expectedHashes
      });
      
      const resultsMap = new Map(segmentResults());
      resultsMap.set(file.path, results);
      setSegmentResults(resultsMap);
      
      // Update hash history
      const history = new Map(hashHistory());
      const existingHistory = history.get(file.path) ?? [];
      const timestamp = new Date();
      
      // Create new array with all segment entries (don't mutate existing)
      const newEntries: HashHistoryEntry[] = results.map(seg => ({
        algorithm: seg.algorithm,
        hash: seg.computed_hash,
        timestamp,
        source: seg.expected_hash ? "verified" as const : "computed" as const,
        verified: seg.verified,
        verified_against: seg.expected_hash
      }));
      history.set(file.path, [...existingHistory, ...newEntries]);
      setHashHistory(history);
      
      const verified = results.filter(r => r.verified === true).length;
      const failed = results.filter(r => r.verified === false).length;
      const noExpected = results.filter(r => r.verified === null || r.verified === undefined).length;
      
      updateFileStatus(file.path, "segments-verified", 100);
      setSegmentVerifyProgress(null);
      
      if (failed > 0) {
        setError(`⚠️ ${failed} segment(s) FAILED verification!`);
      } else if (verified > 0) {
        setOk(`✓ All ${verified} segments verified • ${noExpected > 0 ? `${noExpected} no expected hash` : ""}`);
      } else {
        setOk(`Computed ${results.length} segment hashes (no stored hashes to verify against)`);
      }
      
    } catch (err) {
      updateFileStatus(file.path, "error", 0, normalizeError(err));
      setError(normalizeError(err));
      setSegmentVerifyProgress(null);
    } finally {
      unlisten();
    }
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

  // Parse various date formats (UFED uses DD/MM/YYYY HH:MM:SS (timezone))
  const parseTimestamp = (timestamp: string): Date | null => {
    // Try standard Date parsing first
    const standardDate = new Date(timestamp);
    if (!isNaN(standardDate.getTime())) {
      return standardDate;
    }
    
    // Try UFED format: DD/MM/YYYY HH:MM:SS (timezone) e.g., "26/08/2024 17:48:01 (-4)"
    const ufedMatch = timestamp.match(/^(\d{2})\/(\d{2})\/(\d{4})\s+(\d{2}):(\d{2}):(\d{2})\s*\(([+-]?\d+)\)?$/);
    if (ufedMatch) {
      const [, day, month, year, hour, minute, second, tzOffset] = ufedMatch;
      // Parse timezone offset - UFED uses (-4) meaning UTC-4
      const offset = parseInt(tzOffset, 10);
      // Create date string in ISO format
      const isoString = `${year}-${month}-${day}T${hour}:${minute}:${second}${offset >= 0 ? '+' : ''}${String(offset).padStart(2, '0')}:00`;
      const d = new Date(isoString);
      if (!isNaN(d.getTime())) {
        return d;
      }
      // Fallback: just use date parts directly
      return new Date(parseInt(year), parseInt(month) - 1, parseInt(day), parseInt(hour), parseInt(minute), parseInt(second));
    }
    
    // Try DD/MM/YYYY format without time
    const dmyMatch = timestamp.match(/^(\d{2})\/(\d{2})\/(\d{4})$/);
    if (dmyMatch) {
      const [, day, month, year] = dmyMatch;
      return new Date(parseInt(year), parseInt(month) - 1, parseInt(day));
    }
    
    return null;
  };

  // Format hash timestamp for display
  const formatHashDate = (timestamp: string): string => {
    try {
      const d = parseTimestamp(timestamp);
      if (d) {
        return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: '2-digit' });
      }
      return timestamp;
    } catch {
      return timestamp;
    }
  };

  return {
    // State
    selectedHashAlgorithm,
    setSelectedHashAlgorithm,
    fileHashMap,
    setFileHashMap,
    segmentResults,
    segmentVerifyProgress,
    hashHistory,
    
    // Actions
    hashSingleFile,
    hashSelectedFiles,
    hashAllFiles,
    verifySegments,
    importStoredHashesToHistory,
    
    // Helpers
    getAllStoredHashesSorted,
    formatHashDate,
    recordHashToHistory,
  };
}

export type HashManager = ReturnType<typeof useHashManager>;
