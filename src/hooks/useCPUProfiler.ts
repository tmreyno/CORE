// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useCPUProfiler Hook
 * 
 * SolidJS hook for Phase 14: Advanced CPU Profiling
 * Provides CPU profiling with flamegraph generation using pprof
 * 
 * @example
 * ```tsx
 * const profiler = useCPUProfiler();
 * 
 * // Start profiling
 * await profiler.startProfiling("container_load", 100);
 * 
 * // ... perform operations ...
 * 
 * // Stop profiling and get report
 * const report = await profiler.stopProfiling("container_load");
 * console.log(`Samples: ${report.sample_count}, Duration: ${report.duration_ms}ms`);
 * 
 * // Generate flamegraph
 * const svg = await profiler.generateFlamegraph("container_load");
 * 
 * // Export profile
 * await profiler.exportProfile("container_load", "/path/to/profile.pb");
 * ```
 */

import { invoke } from "@tauri-apps/api/core";
import { createSignal, onCleanup } from "solid-js";
import { logger } from "../utils/logger";
const log = logger.scope("CPUProfiler");

// ============================================================================
// Type Definitions
// ============================================================================

export interface ProfileReport {
  name: string;
  sample_count: number;
  duration_ms: number;
  frequency_hz: number;
  timestamp: number;
  top_functions: FunctionSample[];
}

export interface FunctionSample {
  name: string;
  self_time_ms: number;
  total_time_ms: number;
  call_count: number;
  percent: number;
}

export interface ProfileComparison {
  baseline_name: string;
  current_name: string;
  functions_regressed: FunctionDiff[];
  functions_improved: FunctionDiff[];
  total_samples_baseline: number;
  total_samples_current: number;
  duration_baseline_ms: number;
  duration_current_ms: number;
}

export interface FunctionDiff {
  function_name: string;
  baseline_time_ms: number;
  current_time_ms: number;
  diff_ms: number;
  percent_change: number;
}

export interface ProfileSummary {
  name: string;
  sample_count: number;
  duration_ms: number;
  timestamp: number;
  size_bytes: number;
}

export interface ActiveProfile {
  name: string;
  started_at: number;
  frequency_hz: number;
  samples_collected: number;
}

// ============================================================================
// Hook Implementation
// ============================================================================

export function useCPUProfiler() {
  const [activeProfiles, setActiveProfiles] = createSignal<string[]>([]);
  const [availableProfiles, setAvailableProfiles] = createSignal<ProfileSummary[]>([]);
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  // ============================================================================
  // Profile Management
  // ============================================================================

  /**
   * Start CPU profiling with specified name and frequency
   * @param name Profile identifier
   * @param frequencyHz Sampling frequency (default: 100 Hz)
   */
  const startProfiling = async (name: string, frequencyHz: number = 100): Promise<void> => {
    try {
      setIsLoading(true);
      setError(null);
      
      await invoke("start_profiling", { name, frequencyHz });
      
      // Update active profiles list
      await refreshActiveProfiles();
    } catch (e) {
      const errorMsg = `Failed to start profiling '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Stop CPU profiling and return report
   */
  const stopProfiling = async (name: string): Promise<ProfileReport> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const report = await invoke<ProfileReport>("stop_profiling", { name });
      
      // Update lists
      await refreshActiveProfiles();
      await refreshAvailableProfiles();
      
      return report;
    } catch (e) {
      const errorMsg = `Failed to stop profiling '${name}': ${e}`;
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
      return await invoke<boolean>("is_profiling", { name });
    } catch (e) {
      const errorMsg = `Failed to check profiling status for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Get list of currently active profiles
   */
  const getActiveProfiles = async (): Promise<ActiveProfile[]> => {
    try {
      const profiles = await invoke<ActiveProfile[]>("get_active_profiles");
      setActiveProfiles(profiles.map(p => p.name));
      return profiles;
    } catch (e) {
      const errorMsg = `Failed to get active profiles: ${e}`;
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
      log.error("Failed to refresh active profiles:", e);
    }
  };

  // ============================================================================
  // Profile Analysis
  // ============================================================================

  /**
   * Get detailed profile report
   */
  const getProfileReport = async (name: string): Promise<ProfileReport> => {
    try {
      setIsLoading(true);
      return await invoke<ProfileReport>("get_profile_report", { name });
    } catch (e) {
      const errorMsg = `Failed to get profile report for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Generate flamegraph SVG for a profile
   * Returns SVG as string
   */
  const generateFlamegraph = async (name: string): Promise<string> => {
    try {
      setIsLoading(true);
      return await invoke<string>("generate_flamegraph", { name });
    } catch (e) {
      const errorMsg = `Failed to generate flamegraph for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Compare two profiles
   */
  const compareProfiles = async (
    baselineName: string,
    currentName: string
  ): Promise<ProfileComparison> => {
    try {
      setIsLoading(true);
      return await invoke<ProfileComparison>("compare_profiles", {
        baselineName,
        currentName,
      });
    } catch (e) {
      const errorMsg = `Failed to compare profiles '${baselineName}' vs '${currentName}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Get summary of all available profiles
   */
  const listAllProfiles = async (): Promise<ProfileSummary[]> => {
    try {
      const profiles = await invoke<ProfileSummary[]>("list_all_profiles");
      setAvailableProfiles(profiles);
      return profiles;
    } catch (e) {
      const errorMsg = `Failed to list profiles: ${e}`;
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
      log.error("Failed to refresh available profiles:", e);
    }
  };

  /**
   * Get single profile summary
   */
  const getProfileSummary = async (name: string): Promise<ProfileSummary> => {
    try {
      return await invoke<ProfileSummary>("get_profile_summary", { name });
    } catch (e) {
      const errorMsg = `Failed to get profile summary for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  // ============================================================================
  // Export & Persistence
  // ============================================================================

  /**
   * Export profile to file (protobuf format)
   */
  const exportProfile = async (name: string, path: string): Promise<void> => {
    try {
      setIsLoading(true);
      await invoke("export_profile", { name, path });
    } catch (e) {
      const errorMsg = `Failed to export profile '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Delete a profile
   */
  const deleteProfile = async (name: string): Promise<void> => {
    try {
      setIsLoading(true);
      await invoke("delete_profile", { name });
      await refreshAvailableProfiles();
    } catch (e) {
      const errorMsg = `Failed to delete profile '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Clear all profiles
   */
  const clearAllProfiles = async (): Promise<void> => {
    try {
      setIsLoading(true);
      await invoke("clear_all_profiles");
      setAvailableProfiles([]);
      setActiveProfiles([]);
    } catch (e) {
      const errorMsg = `Failed to clear all profiles: ${e}`;
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
   * Profile an async function
   * Automatically starts/stops profiling around the function execution
   */
  const profileAsync = async <T>(
    name: string,
    fn: () => Promise<T>,
    frequencyHz: number = 100
  ): Promise<{ result: T; report: ProfileReport }> => {
    await startProfiling(name, frequencyHz);
    
    try {
      const result = await fn();
      const report = await stopProfiling(name);
      return { result, report };
    } catch (error) {
      // Stop profiling even on error
      await stopProfiling(name).catch(() => {});
      throw error;
    }
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
    getProfileReport,
    generateFlamegraph,
    compareProfiles,
    getProfileSummary,
    listAllProfiles,

    // Export & persistence
    exportProfile,
    deleteProfile,
    clearAllProfiles,

    // Convenience
    profileAsync,
    
    // Refresh
    refreshActiveProfiles,
    refreshAvailableProfiles,
  };
}
