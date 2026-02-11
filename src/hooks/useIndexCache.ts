// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { createSignal, onMount, onCleanup, createMemo } from "solid-js";
import { logger } from "../utils/logger";
const log = logger.scope("IndexCache");

export interface IndexEntry {
  path: string;
  size: number;
  isDir: boolean;
  modifiedTime?: number;
  hash?: string;
}

export interface IndexSummary {
  containerPath: string;
  totalFiles: number;
  totalDirs: number;
  totalSize: number;
  indexedAt: number;
  isComplete: boolean;
}

export interface CacheStats {
  totalContainers: number;
  totalEntries: number;
  dbSizeBytes: number;
}

export interface IndexProgress {
  containerPath: string;
  current: number;
  total: number;
  percent: number;
  status: string;
}

export interface IndexWorkerInfo {
  containerPath: string;
  startedAt: number;
  currentEntries: number;
  isRunning: boolean;
}

export function useIndexCache() {
  const [initialized, setInitialized] = createSignal(false);
  const [stats, setStats] = createSignal<CacheStats | null>(null);
  const [activeWorkers, setActiveWorkers] = createSignal<IndexWorkerInfo[]>([]);
  const [indexProgress, setIndexProgress] = createSignal<Map<string, IndexProgress>>(new Map());

  // Initialize cache on mount
  onMount(async () => {
    try {
      const dbPath = await invoke<string>("project_get_default_path");
      await invoke("index_cache_init", { dbPath: `${dbPath}/index.db` });
      setInitialized(true);
      await refreshStats();
      await refreshActiveWorkers();
    } catch (error) {
      log.error("Failed to initialize index cache:", error);
    }

    // Listen for index progress events
    const unlistenStart = await listen<IndexProgress>("index-start", (event) => {
      setIndexProgress((map) => {
        const newMap = new Map(map);
        newMap.set(event.payload.containerPath, event.payload);
        return newMap;
      });
    });

    const unlistenProgress = await listen<IndexProgress>("index-progress", (event) => {
      setIndexProgress((map) => {
        const newMap = new Map(map);
        newMap.set(event.payload.containerPath, event.payload);
        return newMap;
      });
    });

    const unlistenComplete = await listen<IndexProgress>("index-complete", (event) => {
      setIndexProgress((map) => {
        const newMap = new Map(map);
        newMap.delete(event.payload.containerPath);
        return newMap;
      });
      refreshStats();
      refreshActiveWorkers();
    });

    const unlistenError = await listen<IndexProgress>("index-error", (event) => {
      log.error(`Index error for ${event.payload.containerPath}:`, event.payload.status);
      setIndexProgress((map) => {
        const newMap = new Map(map);
        newMap.delete(event.payload.containerPath);
        return newMap;
      });
      refreshActiveWorkers();
    });

    onCleanup(() => {
      unlistenStart();
      unlistenProgress();
      unlistenComplete();
      unlistenError();
    });
  });

  const refreshStats = async (): Promise<void> => {
    try {
      const stats = await invoke<CacheStats>("index_cache_stats");
      setStats(stats);
    } catch (error) {
      log.error("Failed to get cache stats:", error);
    }
  };

  const refreshActiveWorkers = async (): Promise<void> => {
    try {
      const workers = await invoke<IndexWorkerInfo[]>("index_worker_get_active");
      setActiveWorkers(workers);
    } catch (error) {
      log.error("Failed to get active workers:", error);
    }
  };

  const hasIndex = async (containerPath: string): Promise<boolean> => {
    try {
      return await invoke<boolean>("index_cache_has_index", { containerPath });
    } catch {
      return false;
    }
  };

  const getSummary = async (containerPath: string): Promise<IndexSummary | null> => {
    try {
      return await invoke<IndexSummary | null>("index_cache_get_summary", { containerPath });
    } catch {
      return null;
    }
  };

  const loadIndex = async (containerPath: string): Promise<IndexEntry[]> => {
    try {
      return await invoke<IndexEntry[]>("index_cache_load", { containerPath });
    } catch (error) {
      log.error("Failed to load index:", error);
      return [];
    }
  };

  const startIndexing = async (containerPath: string, containerType: string): Promise<void> => {
    try {
      await invoke("index_worker_start", { containerPath, containerType });
      await refreshActiveWorkers();
    } catch (error) {
      log.error("Failed to start indexing:", error);
      throw error;
    }
  };

  const cancelIndexing = async (containerPath: string): Promise<void> => {
    try {
      await invoke("index_worker_cancel", { containerPath });
      await refreshActiveWorkers();
    } catch (error) {
      log.error("Failed to cancel indexing:", error);
    }
  };

  const invalidate = async (containerPath: string): Promise<void> => {
    try {
      await invoke("index_cache_invalidate", { containerPath });
      await refreshStats();
    } catch (error) {
      log.error("Failed to invalidate cache:", error);
    }
  };

  const clearCache = async (): Promise<void> => {
    try {
      await invoke("index_cache_clear");
      await refreshStats();
    } catch (error) {
      log.error("Failed to clear cache:", error);
    }
  };

  const isIndexing = (containerPath: string): boolean => {
    return activeWorkers().some((w) => w.containerPath === containerPath);
  };

  const getProgress = (containerPath: string): IndexProgress | undefined => {
    return indexProgress().get(containerPath);
  };

  const formattedCacheSize = createMemo(() => {
    const s = stats();
    if (!s) return "0 B";
    const bytes = s.dbSizeBytes;
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  });

  return {
    initialized,
    stats,
    activeWorkers,
    indexProgress: indexProgress(),
    formattedCacheSize,
    hasIndex,
    getSummary,
    loadIndex,
    startIndexing,
    cancelIndexing,
    invalidate,
    clearCache,
    refreshStats,
    refreshActiveWorkers,
    isIndexing,
    getProgress,
  };
}
