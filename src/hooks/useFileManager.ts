// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { open, ask } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import type { DiscoveredFile, TreeEntry, ContainerInfo } from "../types";
import { normalizeError, formatBytes } from "../utils";
import { logAuditAction } from "../utils/telemetry";
import { getPreference, getLastPath, setLastPath } from "../components/preferences";

// System stats interface (matches Rust struct with serde rename_all = "camelCase")
export interface SystemStats {
  cpuUsage: number;
  memoryUsed: number;
  memoryTotal: number;
  memoryPercent: number;
  appCpuUsage: number;
  appMemory: number;
  appThreads: number;
  cpuCores: number;
}

export interface FileStatus {
  status: string;
  progress: number;
  error?: string;
  // Decompression progress (for E01/compressed containers)
  chunksProcessed?: number;
  chunksTotal?: number;
}

export function useFileManager() {
  console.log("[DEBUG] FileManager: Hook initialized");
  
  // Directory state
  const [scanDir, setScanDir] = createSignal("");
  const [recursiveScan, setRecursiveScan] = createSignal(true);
  
  // File discovery state
  const [discoveredFiles, setDiscoveredFiles] = createSignal<DiscoveredFile[]>([]);
  const [selectedFiles, setSelectedFiles] = createSignal<Set<string>>(new Set());
  const [activeFile, setActiveFile] = createSignal<DiscoveredFile | null>(null);
  const [hoveredFile, setHoveredFile] = createSignal<string | null>(null);
  
  // File info and status maps
  const [fileInfoMap, setFileInfoMap] = createSignal<Map<string, ContainerInfo>>(new Map());
  const [fileStatusMap, setFileStatusMap] = createSignal<Map<string, FileStatus>>(new Map());
  
  // Tree state for AD1 files
  const [tree, setTree] = createSignal<TreeEntry[]>([]);
  const [treeFilter, setTreeFilter] = createSignal("");
  
  // File type filter - null means show all
  const [typeFilter, setTypeFilter] = createSignal<string | null>(null);
  
  // Keyboard navigation - index of focused file in filtered list
  const [focusedFileIndex, setFocusedFileIndex] = createSignal<number>(-1);
  
  // Status state
  const [busy, setBusy] = createSignal(false);
  const [statusMessage, setStatusMessage] = createSignal("Ready");
  const [statusKind, setStatusKind] = createSignal<"idle" | "working" | "ok" | "error">("idle");
  
  // System stats
  const [systemStats, setSystemStats] = createSignal<SystemStats | null>(null);
  
  // Loading progress state
  const [loadProgress, setLoadProgress] = createSignal<{ show: boolean; title: string; message: string; current: number; total: number; cancelled: boolean }>({
    show: false, title: "", message: "", current: 0, total: 0, cancelled: false
  });
  
  // Setup system stats listener
  const setupSystemStatsListener = async (): Promise<() => void> => {
    try {
      const stats = await invoke<SystemStats>("get_system_stats");
      setSystemStats(stats);
    } catch (e) {
      console.error("Failed to get initial system stats:", e);
    }
    
    const unlisten = await listen<SystemStats>("system-stats", (event) => {
      setSystemStats(event.payload);
    });
    
    return unlisten;
  };

  // Computed values
  const filteredFiles = createMemo(() => {
    const filter = typeFilter();
    if (!filter) return discoveredFiles();
    return discoveredFiles().filter(f => f.container_type === filter);
  });
  
  const allFilesSelected = createMemo(() => {
    const files = filteredFiles();
    return files.length > 0 && files.every(f => selectedFiles().has(f.path));
  });
  
  const selectedCount = createMemo(() => selectedFiles().size);
  
  const filteredTree = createMemo(() => {
    const f = treeFilter().trim().toLowerCase();
    return (f ? tree().filter(e => e.path.toLowerCase().includes(f)) : tree()).slice(0, 500);
  });
  
  const totalSize = createMemo(() => discoveredFiles().reduce((s, f) => s + f.size, 0));
  
  const containerStats = createMemo(() => {
    const stats: Record<string, number> = {};
    discoveredFiles().forEach(f => stats[f.container_type] = (stats[f.container_type] || 0) + 1);
    return stats;
  });

  // Status helpers
  const setWorking = (msg: string) => {
    setBusy(true);
    setStatusKind("working");
    setStatusMessage(msg);
  };
  
  const setOk = (msg: string) => {
    setBusy(false);
    setStatusKind("ok");
    setStatusMessage(msg);
  };
  
  const setError = (msg: string) => {
    setBusy(false);
    setStatusKind("error");
    setStatusMessage(msg);
  };
  
  const updateFileStatus = (path: string, status: string, progress: number, error?: string, chunksProcessed?: number, chunksTotal?: number) => {
    const m = new Map(fileStatusMap());
    m.set(path, { status, progress, error, chunksProcessed, chunksTotal });
    setFileStatusMap(m);
  };

  // Toggle type filter
  const toggleTypeFilter = (type: string) => {
    setTypeFilter(prev => prev === type ? null : type);
    setFocusedFileIndex(-1);
  };
  
  // Keyboard navigation handler for file list
  const handleFileListKeyDown = (e: KeyboardEvent, onSelect: (file: DiscoveredFile) => void, onToggle: (path: string) => void) => {
    const files = filteredFiles();
    if (files.length === 0) return;
    
    const currentIndex = focusedFileIndex();
    let newIndex = currentIndex;
    
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        if (currentIndex < files.length - 1) {
          newIndex = currentIndex + 1;
        } else if (currentIndex === -1 && files.length > 0) {
          newIndex = 0;
        }
        break;
      case "ArrowUp":
        e.preventDefault();
        if (currentIndex > 0) {
          newIndex = currentIndex - 1;
        }
        break;
      case "Enter":
        e.preventDefault();
        if (currentIndex >= 0 && currentIndex < files.length) {
          onSelect(files[currentIndex]);
        }
        return;
      case " ":
        e.preventDefault();
        if (currentIndex >= 0 && currentIndex < files.length) {
          onToggle(files[currentIndex].path);
        }
        return;
      case "Home":
        e.preventDefault();
        if (files.length > 0) newIndex = 0;
        break;
      case "End":
        e.preventDefault();
        if (files.length > 0) newIndex = files.length - 1;
        break;
      case "Escape":
        e.preventDefault();
        setTypeFilter(null);
        setFocusedFileIndex(-1);
        return;
      default:
        return;
    }
    
    if (newIndex !== currentIndex && newIndex >= 0) {
      setFocusedFileIndex(newIndex);
      setTimeout(() => {
        const fileList = document.querySelector('.file-list');
        const focusedRow = fileList?.querySelector(`[data-index="${newIndex}"]`);
        focusedRow?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
      }, 0);
    }
  };

  // File selection
  const toggleFileSelection = (path: string) => {
    const c = new Set(selectedFiles());
    c.has(path) ? c.delete(path) : c.add(path);
    setSelectedFiles(c);
  };
  
  const toggleSelectAll = () => {
    const files = filteredFiles();
    const current = new Set(selectedFiles());
    
    if (allFilesSelected()) {
      // Deselect only the filtered files (keep other selections)
      files.forEach(f => current.delete(f.path));
    } else {
      // Add filtered files to current selection (accumulate)
      files.forEach(f => current.add(f.path));
    }
    
    setSelectedFiles(current);
  };

  // Browse for directory
  const browseScanDir = async (): Promise<void> => {
    console.log("[DEBUG] FileManager: browseScanDir called");
    try {
      const defaultPath = getLastPath("evidence");
      console.log(`[DEBUG] FileManager: Opening directory picker, defaultPath=${defaultPath}`);
      const selected = await open({ 
        title: "Select Evidence Directory", 
        multiple: false, 
        directory: true,
        defaultPath,
      });
      if (selected) {
        console.log(`[DEBUG] FileManager: User selected directory: ${selected}`);
        setLastPath("evidence", selected);
        setScanDir(selected);
        await scanForFiles(selected);
      } else {
        console.log("[DEBUG] FileManager: User cancelled directory picker");
      }
    } catch (err) {
      console.error("[DEBUG] FileManager: browseScanDir FAILED:", err);
      setError(normalizeError(err));
    }
  };

  // Scan for files
  // If skipHashLoading is true, don't auto-load hashes (they were pre-loaded elsewhere)
  const scanForFiles = async (dir?: string, preloadedInfo?: Map<string, ContainerInfo>, skipHashLoading = false): Promise<void> => {
    const targetDir = dir || scanDir();
    console.log(`[DEBUG] FileManager: scanForFiles called, dir=${targetDir}, recursive=${recursiveScan()}, skipHashLoading=${skipHashLoading}`);
    
    if (!targetDir.trim()) {
      console.log("[DEBUG] FileManager: No directory specified");
      setError("Select a directory first");
      return;
    }
    
    // Clear previous results
    console.log("[DEBUG] FileManager: Clearing previous scan results");
    setDiscoveredFiles([]);
    setSelectedFiles(new Set<string>());
    setFileInfoMap(new Map());
    setFileStatusMap(new Map());
    setActiveFile(null);
    setTree([]);
    setWorking("Scanning for evidence files...");
    
    // Use streaming scan
    const unlisten = await listen<DiscoveredFile>("scan-file-found", (e) => {
      const file = e.payload;
      setDiscoveredFiles(prev => [...prev, file]);
    });
    
    try {
      console.log("[DEBUG] FileManager: Invoking scan_directory_streaming");
      const count = await invoke<number>("scan_directory_streaming", { dirPath: targetDir, recursive: recursiveScan() });
      console.log(`[DEBUG] FileManager: Scan complete, found ${count} files`);
      setOk(`Found ${count} evidence file(s) • ${formatBytes(discoveredFiles().reduce((s, f) => s + f.size, 0))}`);
      
      // If pre-loaded info was provided, use it
      if (preloadedInfo && preloadedInfo.size > 0) {
        setFileInfoMap(preloadedInfo);
        setOk(`Found ${count} file(s) • Stored hashes loaded`);
      } else if (!skipHashLoading) {
        // Auto-load only stored hashes (fast info) after scan
        loadStoredHashesInBackground();
      } else {
        setOk(`Found ${count} file(s) • Hashes pre-loaded`);
      }
    } catch (err) {
      setError(normalizeError(err));
    } finally {
      unlisten();
    }
  };
  
  // Load only stored hashes in background (fast - no heavy parsing)
  const loadStoredHashesInBackground = async (): Promise<void> => {
    const files = discoveredFiles();
    if (files.length === 0) return;
    
    let loaded = 0;
    const total = files.length;
    
    setLoadProgress({ show: true, title: "Loading Stored Hashes", message: "Reading container headers...", current: 0, total, cancelled: false });
    
    for (const file of files) {
      // Check for cancellation
      if (loadProgress().cancelled) {
        setLoadProgress(prev => ({ ...prev, show: false }));
        setOk(`Cancelled • Loaded ${loaded}/${total} files`);
        return;
      }
      
      if (fileInfoMap().has(file.path)) {
        loaded++;
        setLoadProgress(prev => ({ ...prev, current: loaded, message: `${file.filename}` }));
        continue;
      }
      
      try {
        const result = await invoke<ContainerInfo>("logical_info_fast", { inputPath: file.path });
        setFileInfoMap(prev => {
          const m = new Map(prev);
          m.set(file.path, result);
          return m;
        });
        loaded++;
        setLoadProgress(prev => ({ ...prev, current: loaded, message: `${file.filename}` }));
      } catch (err) {
        console.warn(`Failed to load info for ${file.filename}:`, err);
        loaded++;
        setLoadProgress(prev => ({ ...prev, current: loaded }));
      }
    }
    
    setLoadProgress(prev => ({ ...prev, show: false }));
    setOk(`Found ${total} file(s) • Stored hashes loaded`);
  };
  
  // Cancel loading
  const cancelLoading = () => {
    setLoadProgress(prev => ({ ...prev, cancelled: true }));
  };

  // Load file info for a single file
  const loadFileInfo = async (file: DiscoveredFile, includeTree = false): Promise<ContainerInfo> => {
    updateFileStatus(file.path, "reading-metadata", 0);
    try {
      const result = await invoke<ContainerInfo>("logical_info", { inputPath: file.path, includeTree });
      const m = new Map(fileInfoMap());
      m.set(file.path, result);
      setFileInfoMap(m);
      updateFileStatus(file.path, "loaded", 100);
      if (includeTree && result.ad1?.tree) {
        setTree(result.ad1.tree);
        setActiveFile(file);
      }
      return result;
    } catch (err) {
      updateFileStatus(file.path, "error", 0, normalizeError(err));
      throw err;
    }
  };

  // Load all file info (full details with progress modal)
  const loadAllInfo = async (): Promise<void> => {
    const files = discoveredFiles();
    if (files.length === 0) return;
    
    const total = files.length;
    let loaded = 0;
    
    setLoadProgress({ show: true, title: "Loading Full Details", message: "Parsing container metadata...", current: 0, total, cancelled: false });
    
    for (const file of files) {
      // Check for cancellation
      if (loadProgress().cancelled) {
        setLoadProgress(prev => ({ ...prev, show: false }));
        setOk(`Cancelled • Loaded ${loaded}/${total} files`);
        return;
      }
      
      setLoadProgress(prev => ({ ...prev, current: loaded, message: `${file.filename}` }));
      
      if (!fileInfoMap().has(file.path)) {
        try {
          const result = await invoke<ContainerInfo>("logical_info", { inputPath: file.path, includeTree: false });
          setFileInfoMap(prev => {
            const m = new Map(prev);
            m.set(file.path, result);
            return m;
          });
        } catch (err) {
          console.warn(`Failed to load info for ${file.filename}:`, err);
        }
      }
      loaded++;
    }
    
    setLoadProgress(prev => ({ ...prev, show: false }));
    setOk(`Loaded full details for ${loaded} files`);
  };

  // Select and view file
  // NOTE: We intentionally do NOT load the full tree here (includeTree: false)
  // because it can take 15-20 seconds for large AD1 files.
  // The EvidenceTree component uses V2 lazy loading APIs which are ~17,000x faster
  // (1ms for root children vs 17s for full tree parsing).
  const selectAndViewFile = async (file: DiscoveredFile): Promise<void> => {
    // Check for large container warning preference
    if (getPreference("warnOnLargeContainers")) {
      const thresholdGb = getPreference("largeContainerThresholdGb");
      const thresholdBytes = thresholdGb * 1024 * 1024 * 1024;
      
      if (file.size > thresholdBytes) {
        const confirmed = await ask(
          `This container (${formatBytes(file.size)}) exceeds ${thresholdGb}GB.\n\nLarge containers may take longer to process and use more memory. Continue?`,
          { title: "Large Container", kind: "warning" }
        );
        if (!confirmed) return;
      }
    }
    
    setActiveFile(file);
    
    // Audit log file opened
    logAuditAction("file_opened", {
      path: file.path,
      filename: file.filename,
      containerType: file.container_type,
      size: file.size,
    });
    
    const existingInfo = fileInfoMap().get(file.path);
    
    // Load basic container info (without tree) if not already cached
    if (!existingInfo) {
      try {
        await loadFileInfo(file, false);  // Fast: ~3ms for AD1, ~5s for E01
      } catch (err) {
        // Log but don't propagate - file may have missing segments
        console.warn(`Failed to load info for ${file.filename}:`, normalizeError(err));
      }
    }
    // Tree is populated by EvidenceTree via V2 lazy loading APIs
  };

  // ===== PROJECT RESTORE FUNCTIONS =====

  /**
   * Restore discovered files from project cache.
   * Called when loading a project to avoid re-scanning directories.
   */
  const restoreDiscoveredFiles = (files: DiscoveredFile[]) => {
    if (!files || files.length === 0) return;
    setDiscoveredFiles(files);
    setOk(`Restored ${files.length} discovered files from project cache`);
    console.log("[FileManager] Restored discovered files:", files.length);
  };

  /**
   * Restore file info map from project cache.
   * Called when loading a project to avoid re-loading container info.
   */
  const restoreFileInfoMap = (infoMap: Record<string, ContainerInfo>) => {
    if (!infoMap || Object.keys(infoMap).length === 0) return;
    const map = new Map<string, ContainerInfo>();
    for (const [path, info] of Object.entries(infoMap)) {
      map.set(path, info);
    }
    setFileInfoMap(map);
    console.log("[FileManager] Restored file info cache:", map.size, "entries");
  };

  /**
   * Clear all file manager state (for new project or reset)
   */
  const clearAll = () => {
    setDiscoveredFiles([]);
    setSelectedFiles(new Set<string>());
    setActiveFile(null);
    setFileInfoMap(new Map());
    setFileStatusMap(new Map());
    setTree([]);
    setTreeFilter("");
    setTypeFilter(null);
    setFocusedFileIndex(-1);
    setScanDir("");
    setStatusMessage("Ready");
    setStatusKind("idle");
    console.log("[FileManager] Cleared all state");
  };

  return {
    // State
    scanDir,
    setScanDir,
    recursiveScan,
    setRecursiveScan,
    discoveredFiles,
    selectedFiles,
    setSelectedFiles,
    activeFile,
    setActiveFile,
    hoveredFile,
    setHoveredFile,
    fileInfoMap,
    setFileInfoMap,
    fileStatusMap,
    tree,
    treeFilter,
    setTreeFilter,
    typeFilter,
    setTypeFilter,
    focusedFileIndex,
    setFocusedFileIndex,
    busy,
    statusMessage,
    statusKind,
    systemStats,
    loadProgress,
    
    // Computed
    filteredFiles,
    allFilesSelected,
    selectedCount,
    filteredTree,
    totalSize,
    containerStats,
    
    // Actions
    setWorking,
    setOk,
    setError,
    updateFileStatus,
    toggleTypeFilter,
    handleFileListKeyDown,
    toggleFileSelection,
    toggleSelectAll,
    browseScanDir,
    scanForFiles,
    loadFileInfo,
    loadAllInfo,
    selectAndViewFile,
    setupSystemStatsListener,
    cancelLoading,
    /** Add a discovered file (e.g., for nested containers) */
    addDiscoveredFile: (file: DiscoveredFile) => {
      // Check if file already exists
      const exists = discoveredFiles().some(f => f.path === file.path);
      if (!exists) {
        setDiscoveredFiles(prev => [...prev, file]);
      }
    },
    // Project restore functions
    restoreDiscoveredFiles,
    restoreFileInfoMap,
    clearAll,
  };
}

export type FileManager = ReturnType<typeof useFileManager>;
