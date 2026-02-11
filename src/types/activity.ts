// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Simplified Activity Tracking
 * 
 * Matches the actual progress data provided by the sevenzip-ffi library.
 * Removed over-engineered preferences and fields that were never populated.
 */

export type ActivityType = "archive" | "export" | "copy" | "tool";
export type ActivityStatus = "pending" | "running" | "paused" | "completed" | "failed" | "cancelled";

/**
 * Progress data - matches library callback signature
 */
export interface ActivityProgress {
  /** Total bytes processed */
  bytesProcessed?: number;
  /** Total bytes to process (0 if unknown) */
  bytesTotal?: number;
  /** Progress percentage (0-100) */
  percent: number;
  /** Current file being processed */
  currentFile?: string;
  /** Current file bytes processed */
  currentFileBytes?: number;
  /** Current file total bytes */
  currentFileTotal?: number;
  /** Files processed (for copy/export) */
  filesProcessed?: number;
  /** Total files (for copy/export) */
  totalFiles?: number;
}

/**
 * Activity record - simplified to match actual data flow
 */
export interface Activity {
  /** Unique identifier */
  id: string;
  /** Activity type */
  type: ActivityType;
  /** Current status */
  status: ActivityStatus;
  /** Destination path/archive name */
  destination: string;
  /** Number of source items */
  sourceCount: number;
  /** Start time */
  startTime: Date;
  /** End time (when finished) */
  endTime?: Date;
  /** Current progress */
  progress?: ActivityProgress;
  /** Error message if failed */
  error?: string;
  /** Optional: compression level used */
  compressionLevel?: number;
  /** Optional: encrypted */
  encrypted?: boolean;
}

/**
 * Create a new activity
 */
export function createActivity(
  type: ActivityType,
  destination: string,
  sourceCount: number,
  options?: { compressionLevel?: number; encrypted?: boolean; includeHashes?: boolean; operation?: string }
): Activity {
  return {
    id: `${type}-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`,
    type,
    status: "pending",
    destination,
    sourceCount,
    startTime: new Date(),
    compressionLevel: options?.compressionLevel,
    encrypted: options?.encrypted,
  };
}

/**
 * Update activity progress
 */
export function updateProgress(
  activity: Activity,
  progress: ActivityProgress
): Activity {
  return {
    ...activity,
    status: "running",
    progress,
  };
}

/**
 * Mark activity completed
 */
export function completeActivity(activity: Activity): Activity {
  return {
    ...activity,
    status: "completed",
    endTime: new Date(),
    progress: activity.progress ? { ...activity.progress, percent: 100 } : undefined,
  };
}

/**
 * Mark activity failed
 */
export function failActivity(activity: Activity, error: string): Activity {
  return {
    ...activity,
    status: "failed",
    endTime: new Date(),
    error,
  };
}

/**
 * Mark activity cancelled
 */
export function cancelActivity(activity: Activity): Activity {
  return {
    ...activity,
    status: "cancelled",
    endTime: new Date(),
  };
}

/**
 * Format bytes as human-readable string
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

/**
 * Calculate speed from activity
 */
export function calculateSpeed(activity: Activity): number | null {
  if (!activity.progress || activity.status !== "running") return null;
  if (!activity.progress.bytesProcessed) return null;
  const elapsed = Date.now() - activity.startTime.getTime();
  if (elapsed < 1000) return null;
  return (activity.progress.bytesProcessed / elapsed) * 1000;
}

/**
 * Format speed as string
 */
export function formatSpeed(bytesPerSec: number): string {
  return `${formatBytes(bytesPerSec)}/s`;
}

/**
 * Calculate ETA
 */
export function calculateETA(activity: Activity): number | null {
  if (!activity.progress || activity.status !== "running") return null;
  if (!activity.progress.bytesTotal || !activity.progress.bytesProcessed) return null;
  const elapsed = Date.now() - activity.startTime.getTime();
  if (elapsed < 2000) return null;
  const bytesPerMs = activity.progress.bytesProcessed / elapsed;
  if (bytesPerMs === 0) return null;
  const remaining = activity.progress.bytesTotal - activity.progress.bytesProcessed;
  return remaining / bytesPerMs;
}

/**
 * Format ETA as string
 */
export function formatETA(ms: number): string {
  if (ms < 60000) return `${Math.ceil(ms / 1000)}s`;
  if (ms < 3600000) return `${Math.ceil(ms / 60000)}m`;
  const hours = Math.floor(ms / 3600000);
  const minutes = Math.ceil((ms % 3600000) / 60000);
  return `${hours}h ${minutes}m`;
}

/**
 * Get duration of completed activity
 */
export function getDuration(activity: Activity): number | null {
  if (!activity.endTime) return null;
  return activity.endTime.getTime() - activity.startTime.getTime();
}

/**
 * Format duration as string
 */
export function formatDuration(ms: number): string {
  const seconds = Math.floor(ms / 1000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const secs = seconds % 60;
  if (minutes < 60) return `${minutes}m ${secs}s`;
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  return `${hours}h ${mins}m`;
}
