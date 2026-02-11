// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Export/Archive Activity Tracking Types
 * 
 * Tracks all export operations (copy, export with metadata, archive creation)
 * for display in the right panel activity view.
 */

export type ExportActivityType = "copy" | "export" | "archive";
export type ExportActivityStatus = "pending" | "running" | "paused" | "completed" | "failed" | "cancelled";

export interface ExportActivity {
  /** Unique identifier for this activity */
  id: string;
  
  /** Type of export operation */
  type: ExportActivityType;
  
  /** Current status */
  status: ExportActivityStatus;
  
  /** Source files/folders being exported */
  sources: string[];
  
  /** Destination path */
  destination: string;
  
  /** When the activity was created */
  startTime: Date;
  
  /** When the activity finished (completed/failed/cancelled) */
  endTime?: Date;
  
  /** Current progress information */
  progress?: {
    /** Current item being processed */
    currentFile?: string;
    /** Current file index */
    currentFileIndex?: number;
    /** Total number of files */
    totalFiles?: number;
    /** Bytes processed so far */
    bytesProcessed: number;
    /** Total bytes to process */
    totalBytes: number;
    /** Percentage complete (0-100) */
    percent: number;
    /** Current file bytes processed */
    currentFileBytes?: number;
    /** Current file total bytes */
    currentFileTotal?: number;
    /** Bytes processed when last speed calculation was done */
    lastSpeedCheckBytes?: number;
    /** Timestamp of last speed calculation */
    lastSpeedCheckTime?: number;
  };
  
  /** Error message if failed */
  error?: string;
  
  /** Additional metadata */
  metadata?: {
    /** Compression level for archives */
    compressionLevel?: string;
    /** Whether archive is encrypted */
    encrypted?: boolean;
    /** Number of files/folders */
    itemCount?: number;
    /** Archive format (e.g., "7z") */
    format?: string;
    /** Thread count used */
    threadCount?: number;
    /** Original size (uncompressed) */
    originalSize?: number;
    /** Final size (compressed) */
    finalSize?: number;
    /** Compression ratio (0-1) */
    compressionRatio?: number;
  };
  
  /** Whether this activity is paused */
  paused?: boolean;
}

/**
 * Helper to create a new activity
 */
export function createExportActivity(
  type: ExportActivityType,
  sources: string[],
  destination: string,
  metadata?: ExportActivity["metadata"]
): ExportActivity {
  return {
    id: `export-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    type,
    status: "pending",
    sources,
    destination,
    startTime: new Date(),
    metadata,
  };
}

/**
 * Helper to update activity progress
 */
export function updateActivityProgress(
  activity: ExportActivity,
  progress: ExportActivity["progress"]
): ExportActivity {
  return {
    ...activity,
    status: "running",
    progress,
  };
}

/**
 * Helper to mark activity as completed
 */
export function completeActivity(activity: ExportActivity): ExportActivity {
  return {
    ...activity,
    status: "completed",
    endTime: new Date(),
  };
}

/**
 * Helper to mark activity as failed
 */
export function failActivity(activity: ExportActivity, error: string): ExportActivity {
  return {
    ...activity,
    status: "failed",
    endTime: new Date(),
    error,
  };
}

/**
 * Helper to mark activity as cancelled
 */
export function cancelActivity(activity: ExportActivity): ExportActivity {
  return {
    ...activity,
    status: "cancelled",
    endTime: new Date(),
  };
}

/**
 * Get display name for activity type
 */
export function getActivityTypeName(type: ExportActivityType): string {
  switch (type) {
    case "copy": return "Copy";
    case "export": return "Export";
    case "archive": return "Archive";
  }
}

/**
 * Get icon class for activity type
 */
export function getActivityTypeIcon(type: ExportActivityType): string {
  switch (type) {
    case "copy": return "document-duplicate";
    case "export": return "arrow-up-tray";
    case "archive": return "archive-box";
  }
}

/**
 * Calculate duration in seconds
 */
export function getActivityDuration(activity: ExportActivity): number | null {
  if (!activity.endTime) return null;
  return Math.floor((activity.endTime.getTime() - activity.startTime.getTime()) / 1000);
}

/**
 * Format duration as human-readable string
 */
export function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const secs = seconds % 60;
  if (minutes < 60) return `${minutes}m ${secs}s`;
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  return `${hours}h ${mins}m`;
}
