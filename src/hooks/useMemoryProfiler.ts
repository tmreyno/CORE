// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useMemoryProfiler Hook
 * 
 * SolidJS hook for Phase 15: Advanced Memory Profiling
 * Provides memory profiling with leak detection and allocation tracking
 * 
 * @example
 * ```tsx
 * const profiler = useMemoryProfiler();
 * 
 * // Start memory profiling
 * await profiler.startProfiling("container_process");
 * 
 * // ... perform memory-intensive operations ...
 * 
 * // Stop and get report
 * const report = await profiler.stopProfiling("container_process");
 * console.log(`Peak: ${report.peak_memory_mb}MB, Current: ${report.current_memory_mb}MB`);
 * 
 * // Analyze for leaks
 * const leaks = await profiler.analyzeLeaks("container_process");
 * if (leaks.potential_leaks.length > 0) {
 *   log.warn("Memory leaks detected!");
 * }
 * ```
 */

import { invoke } from "@tauri-apps/api/core";
import { createSignal, onCleanup } from "solid-js";
import { logger } from "../utils/logger";
const log = logger.scope("MemoryProfiler");

// ============================================================================
// Type Definitions
// ============================================================================

export interface MemoryReport {
  name: string;
  start_memory_mb: number;
  current_memory_mb: number;
  peak_memory_mb: number;
  allocated_mb: number;
  deallocated_mb: number;
  net_change_mb: number;
  snapshot_count: number;
  duration_ms: number;
  timestamp: number;
}

export interface MemorySnapshot {
  timestamp: number;
  memory_used_mb: number;
  memory_available_mb: number;
  process_memory_mb: number;
  heap_allocated_mb: number;
}

export interface LeakAnalysis {
  profile_name: string;
  potential_leaks: LeakCandidate[];
  total_leaked_mb: number;
  confidence: number;
  analysis_method: string;
  timestamp: number;
}

export interface LeakCandidate {
  description: string;
  leaked_mb: number;
  confidence: number;
  snapshots_affected: number[];
}

export interface MemoryTimeline {
  profile_name: string;
  snapshots: MemorySnapshot[];
  duration_ms: number;
  sample_interval_ms: number;
}

export interface SnapshotComparison {
  snapshot1_index: number;
  snapshot2_index: number;
  memory_diff_mb: number;
  time_diff_ms: number;
  growth_rate_mb_per_sec: number;
}

export interface ProfileSummary {
  name: string;
  is_active: boolean;
  snapshot_count: number;
  duration_ms: number;
  peak_memory_mb: number;
  timestamp: number;
}

export interface ActiveProfile {
  name: string;
  started_at: number;
  snapshot_count: number;
  current_memory_mb: number;
  peak_memory_mb: number;
}

// ============================================================================
// Hook Implementation
// ============================================================================

export function useMemoryProfiler() {
  const [activeProfiles, setActiveProfiles] = createSignal<string[]>([]);
  const [availableProfiles, setAvailableProfiles] = createSignal<ProfileSummary[]>([]);
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  // ============================================================================
  // Profile Management
  // ============================================================================

  /**
   * Start memory profiling
   * @param name Profile identifier
   * @param intervalMs Snapshot interval in milliseconds (default: 1000)
   */
  const startProfiling = async (name: string, intervalMs: number = 1000): Promise<void> => {
    try {
      setIsLoading(true);
      setError(null);
      
      await invoke("memory_start_profiling", { name, intervalMs });
      
      await refreshActiveProfiles();
    } catch (e) {
      const errorMsg = `Failed to start memory profiling '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Stop memory profiling and return report
   */
  const stopProfiling = async (name: string): Promise<MemoryReport> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const report = await invoke<MemoryReport>("memory_stop_profiling", { name });
      
      await refreshActiveProfiles();
      await refreshAvailableProfiles();
      
      return report;
    } catch (e) {
      const errorMsg = `Failed to stop memory profiling '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Check if profiling is active for a specific name
   */
  const isProfiling = async (name: string): Promise<boolean> => {
    try {
      return await invoke<boolean>("memory_is_profiling", { name });
    } catch (e) {
      const errorMsg = `Failed to check memory profiling status for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Get list of currently active memory profiles
   */
  const getActiveProfiles = async (): Promise<ActiveProfile[]> => {
    try {
      const profiles = await invoke<ActiveProfile[]>("memory_get_active");
      setActiveProfiles(profiles.map(p => p.name));
      return profiles;
    } catch (e) {
      const errorMsg = `Failed to get active memory profiles: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Refresh active profiles list (internal helper)
   */
  const refreshActiveProfiles = async (): Promise<void> => {
    try {
      await getActiveProfiles();
    } catch (e) {
      log.error("Failed to refresh active memory profiles:", e);
    }
  };

  // ============================================================================
  // Memory Analysis
  // ============================================================================

  /**
   * Get detailed memory profile report
   */
  const getReport = async (name: string): Promise<MemoryReport> => {
    try {
      setIsLoading(true);
      return await invoke<MemoryReport>("memory_get_report", { name });
    } catch (e) {
      const errorMsg = `Failed to get memory report for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Analyze profile for memory leaks
   */
  const analyzeLeaks = async (name: string): Promise<LeakAnalysis> => {
    try {
      setIsLoading(true);
      return await invoke<LeakAnalysis>("memory_analyze_leaks", { name });
    } catch (e) {
      const errorMsg = `Failed to analyze leaks for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Get memory timeline (all snapshots)
   */
  const getTimeline = async (name: string): Promise<MemoryTimeline> => {
    try {
      setIsLoading(true);
      return await invoke<MemoryTimeline>("memory_get_timeline", { name });
    } catch (e) {
      const errorMsg = `Failed to get memory timeline for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Compare two memory snapshots
   */
  const compareSnapshots = async (
    name: string,
    snapshot1: number,
    snapshot2: number
  ): Promise<SnapshotComparison> => {
    try {
      setIsLoading(true);
      return await invoke<SnapshotComparison>("memory_compare_snapshots", {
        name,
        snapshot1,
        snapshot2,
      });
    } catch (e) {
      const errorMsg = `Failed to compare snapshots for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  // ============================================================================
  // Profile Management
  // ============================================================================

  /**
   * Get summary of a single profile
   */
  const getSummary = async (name: string): Promise<ProfileSummary> => {
    try {
      return await invoke<ProfileSummary>("memory_get_summary", { name });
    } catch (e) {
      const errorMsg = `Failed to get memory summary for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * List all available memory profiles
   */
  const listAllProfiles = async (): Promise<ProfileSummary[]> => {
    try {
      const profiles = await invoke<ProfileSummary[]>("memory_list_all");
      setAvailableProfiles(profiles);
      return profiles;
    } catch (e) {
      const errorMsg = `Failed to list memory profiles: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Refresh available profiles list (internal helper)
   */
  const refreshAvailableProfiles = async (): Promise<void> => {
    try {
      await listAllProfiles();
    } catch (e) {
      log.error("Failed to refresh available memory profiles:", e);
    }
  };

  // ============================================================================
  // Export & Persistence
  // ============================================================================

  /**
   * Export memory profile to JSON file
   */
  const exportProfile = async (name: string, path: string): Promise<void> => {
    try {
      setIsLoading(true);
      await invoke("memory_export", { name, path });
    } catch (e) {
      const errorMsg = `Failed to export memory profile '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Delete a memory profile
   */
  const deleteProfile = async (name: string): Promise<void> => {
    try {
      setIsLoading(true);
      await invoke("memory_delete_profile", { name });
      await refreshAvailableProfiles();
    } catch (e) {
      const errorMsg = `Failed to delete memory profile '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Clear all memory profiles
   */
  const clearAllProfiles = async (): Promise<void> => {
    try {
      setIsLoading(true);
      await invoke("memory_clear_all");
      setAvailableProfiles([]);
      setActiveProfiles([]);
    } catch (e) {
      const errorMsg = `Failed to clear all memory profiles: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  // ============================================================================
  // Convenience Methods
  // ============================================================================

  /**
   * Profile memory usage of an async function
   * Automatically starts/stops profiling and analyzes for leaks
   */
  const profileAsync = async <T>(
    name: string,
    fn: () => Promise<T>,
    intervalMs: number = 1000
  ): Promise<{ result: T; report: MemoryReport; leaks: LeakAnalysis }> => {
    await startProfiling(name, intervalMs);
    
    try {
      const result = await fn();
      const report = await stopProfiling(name);
      const leaks = await analyzeLeaks(name);
      
      return { result, report, leaks };
    } catch (error) {
      // Stop profiling even on error
      await stopProfiling(name).catch(() => {});
      throw error;
    }
  };

  /**
   * Quick memory snapshot comparison
   * Start profiling, wait for duration, then compare first and last snapshots
   */
  const quickLeakCheck = async (
    name: string,
    durationMs: number,
    intervalMs: number = 1000
  ): Promise<LeakAnalysis> => {
    await startProfiling(name, intervalMs);
    
    // Wait for specified duration
    await new Promise(resolve => setTimeout(resolve, durationMs));
    
    await stopProfiling(name);
    return await analyzeLeaks(name);
  };

  /**
   * Initialize hook - load available profiles
   */
  const initialize = async (): Promise<void> => {
    await refreshActiveProfiles();
    await refreshAvailableProfiles();
  };

  // Initialize on mount
  initialize();

  // Clean up on unmount
  onCleanup(() => {
    // Stop all active profiles
    activeProfiles().forEach(name => {
      stopProfiling(name).catch(() => {});
    });
  });

  // ============================================================================
  // Return API
  // ============================================================================

  return {
    // State
    activeProfiles,
    availableProfiles,
    isLoading,
    error,

    // Profile management
    startProfiling,
    stopProfiling,
    isProfiling,
    getActiveProfiles,

    // Analysis
    getReport,
    analyzeLeaks,
    getTimeline,
    compareSnapshots,
    getSummary,
    listAllProfiles,

    // Export & persistence
    exportProfile,
    deleteProfile,
    clearAllProfiles,

    // Convenience
    profileAsync,
    quickLeakCheck,
    
    // Refresh
    refreshActiveProfiles,
    refreshAvailableProfiles,
  };
}
