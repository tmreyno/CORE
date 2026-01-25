// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { createSignal } from "solid-js";

// =============================================================================
// Type Definitions - Aligned with backend activity_timeline.rs
// =============================================================================

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
  details: unknown;
}

/**
 * FFXProject interface for timeline commands
 * This matches the backend FFXProject structure
 */
export interface FFXProject {
  name: string;
  path: string;
  created_at: string;
  modified_at: string;
  version: string;
  bookmarks: unknown[];
  notes: unknown[];
  activity_log: unknown[];
  evidence_items: unknown[];
  metadata: Record<string, unknown>;
}

// =============================================================================
// Hook Implementation
// =============================================================================

/**
 * Hook for activity timeline visualization and analysis
 * 
 * Note: Backend commands expect an FFXProject object, not just a path.
 * The project should be loaded using useProject hook first.
 */
export function useActivityTimeline() {
  const [visualization, setVisualization] = createSignal<TimelineVisualization | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  /**
   * Compute timeline visualization data for a project
   * Uses timeline_compute_visualization backend command
   */
  const computeVisualization = async (project: FFXProject): Promise<TimelineVisualization | null> => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<TimelineVisualization>(
        "timeline_compute_visualization",
        { project }
      );
      setVisualization(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to compute timeline visualization:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Export timeline data
   * Uses timeline_export backend command
   */
  const exportTimeline = async (
    project: FFXProject,
    exportedBy: string = "user"
  ): Promise<TimelineExport | null> => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<TimelineExport>("timeline_export", {
        project,
        exported_by: exportedBy,
      });
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to export timeline:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Export timeline to JSON string
   * Uses timeline_export_json backend command
   */
  const exportTimelineJson = async (
    project: FFXProject,
    exportedBy: string = "user"
  ): Promise<string | null> => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<string>("timeline_export_json", {
        project,
        exported_by: exportedBy,
      });
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to export timeline JSON:", err);
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
    computeVisualization,
    exportTimeline,
    exportTimelineJson,
    // Utilities
    getHeatmapData,
    getMostActivePeriods,
    getTypeBreakdown,
    getUserSummary,
    getTrends,
  };
}
