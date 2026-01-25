import { createSignal, onMount, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface FileInfo {
  path: string;
  size: number;
  blake3Hash: string;
  modifiedTime?: number;
  isDuplicate: boolean;
}

export interface DuplicateGroup {
  hash: string;
  fileCount: number;
  totalSize: number;
  wastedSpace: number;
  files: FileInfo[];
}

export interface DeduplicationStats {
  totalFiles: number;
  totalSize: number;
  uniqueFiles: number;
  duplicateFiles: number;
  duplicateGroups: number;
  wastedSpace: number;
  spaceSavingsPercent: number;
  largestDuplicateGroup?: string;
  mostWastedHash?: string;
}

export interface DeduplicationProgress {
  filesProcessed: number;
  totalFiles: number;
  percentComplete: number;
  currentFile: string;
  throughputMbps: number;
}

export function useDeduplication() {
  const [initialized, setInitialized] = createSignal(false);
  const [scanning, setScanning] = createSignal(false);
  const [progress, setProgress] = createSignal<DeduplicationProgress | null>(null);
  const [stats, setStats] = createSignal<DeduplicationStats | null>(null);
  const [duplicateGroups, setDuplicateGroups] = createSignal<DuplicateGroup[]>([]);
  const [error, setError] = createSignal<string | null>(null);
  const [selectedGroup, setSelectedGroup] = createSignal<string | null>(null);

  let progressUnlisten: (() => void) | null = null;

  onMount(async () => {
    try {
      // Initialize deduplication engine
      await invoke("dedup_init");
      setInitialized(true);

      // Listen for progress events
      progressUnlisten = await listen<DeduplicationProgress>(
        "deduplication-progress",
        (event) => {
          setProgress(event.payload);
        }
      );
    } catch (err) {
      setError(String(err));
    }
  });

  onCleanup(() => {
    if (progressUnlisten) {
      progressUnlisten();
    }
  });

  const scanFiles = async (filePaths: string[]): Promise<void> => {
    try {
      setScanning(true);
      setError(null);
      setProgress(null);

      await invoke("dedup_scan_files", { filePaths });

      // Get updated statistics
      const newStats = await invoke<DeduplicationStats>("dedup_get_statistics");
      setStats(newStats);

      // Get duplicate groups
      const groups = await invoke<DuplicateGroup[]>("dedup_get_duplicate_groups");
      setDuplicateGroups(groups);

      setScanning(false);
    } catch (err) {
      setError(String(err));
      setScanning(false);
    }
  };

  const getGroupFiles = async (hash: string): Promise<FileInfo[]> => {
    try {
      return await invoke<FileInfo[]>("dedup_get_group_files", { hash });
    } catch (err) {
      setError(String(err));
      return [];
    }
  };

  const exportReport = async (): Promise<string | null> => {
    try {
      return await invoke<string>("dedup_export_json");
    } catch (err) {
      setError(String(err));
      return null;
    }
  };

  const clear = async (): Promise<void> => {
    try {
      await invoke("dedup_clear");
      setStats(null);
      setDuplicateGroups([]);
      setProgress(null);
      setSelectedGroup(null);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  };

  // Helper formatters
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const formatPercent = (value: number): string => {
    return `${value.toFixed(2)}%`;
  };

  return {
    // State
    initialized,
    scanning,
    progress,
    stats,
    duplicateGroups,
    error,
    selectedGroup,

    // Actions
    scanFiles,
    getGroupFiles,
    exportReport,
    clear,
    setSelectedGroup,

    // Formatters
    formatBytes,
    formatPercent,
  };
}
