// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { createSignal } from "solid-js";

/**
 * Timeline visualization data from backend
 */
export interface TimelineVisualization {
  summary: TimelineSummary;
  heatmap: ActivityHeatmap;
  daily_breakdown: DailyBreakdown[];
  type_distribution: TypeDistribution[];
  user_activity: UserActivity[];
  trends: ActivityTrends;
}

export interface TimelineSummary {
  total_activities: number;
  date_range: [string, string];
  active_days: number;
  most_active_day: string;
  most_active_hour: number;
  unique_users: number;
  activity_types: number;
}

export interface ActivityHeatmap {
  data: number[][];
  max_value: number;
  day_labels: string[];
  hour_labels: string[];
}

export interface DailyBreakdown {
  date: string;
  count: number;
  by_type: Record<string, number>;
  duration_minutes: number;
  unique_users: number;
}

export interface TypeDistribution {
  activity_type: string;
  count: number;
  percentage: number;
  color: string;
  rank: number;
}

export interface UserActivity {
  user: string;
  total_activities: number;
  by_type: Record<string, number>;
  first_activity: string;
  last_activity: string;
  active_days: number;
}

export interface ActivityTrends {
  daily_average: number;
  weekly_average: number;
  peak_hour: number;
  peak_day: string;
  trend_direction: "increasing" | "decreasing" | "stable";
  velocity: number;
}

/**
 * Timeline export format
 */
export interface TimelineExport {
  metadata: ExportMetadata;
  activities: ActivityExportEntry[];
  statistics: TimelineSummary;
}

export interface ExportMetadata {
  project_name: string;
  export_date: string;
  exported_by: string;
  format_version: string;
}

export interface ActivityExportEntry {
  timestamp: string;
  activity_type: string;
  description: string;
  user: string;
  details: any;
}

/**
 * Hook for activity timeline visualization and analysis
 */
export function useActivityTimeline() {
  const [visualization, setVisualization] = createSignal<TimelineVisualization | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  /**
   * Get timeline visualization data
   */
  const getVisualization = async (projectPath: string) => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<TimelineVisualization>(
        "timeline_get_visualization",
        { projectPath }
      );
      setVisualization(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to get timeline visualization:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Export timeline to JSON file
   */
  const exportTimeline = async (
    projectPath: string,
    outputPath: string,
    exportedBy: string = "user"
  ): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("timeline_export", {
        projectPath,
        outputPath,
        exportedBy,
      });
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to export timeline:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Get activity patterns
   */
  const getPatterns = async (projectPath: string) => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<ActivityTrends>("timeline_get_patterns", {
        projectPath,
      });
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to get activity patterns:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Calculate heatmap for specific date range
   */
  const getHeatmapData = () => {
    const viz = visualization();
    if (!viz) return null;
    return viz.heatmap;
  };

  /**
   * Get most active periods
   */
  const getMostActivePeriods = (limit: number = 5) => {
    const viz = visualization();
    if (!viz) return [];
    
    return viz.daily_breakdown
      .sort((a, b) => b.count - a.count)
      .slice(0, limit);
  };

  /**
   * Get activity by type breakdown
   */
  const getTypeBreakdown = () => {
    const viz = visualization();
    if (!viz) return [];
    return viz.type_distribution;
  };

  /**
   * Get user activity summary
   */
  const getUserSummary = () => {
    const viz = visualization();
    if (!viz) return [];
    return viz.user_activity;
  };

  /**
   * Get trend analysis
   */
  const getTrends = () => {
    const viz = visualization();
    if (!viz) return null;
    return viz.trends;
  };

  return {
    // State
    visualization,
    loading,
    error,
    // Actions
    getVisualization,
    exportTimeline,
    getPatterns,
    // Utilities
    getHeatmapData,
    getMostActivePeriods,
    getTypeBreakdown,
    getUserSummary,
    getTrends,
  };
}
