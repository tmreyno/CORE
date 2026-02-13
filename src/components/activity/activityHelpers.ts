// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { openUrl } from "@tauri-apps/plugin-opener";
import type { ExportActivity, ExportActivityType, ExportActivityStatus } from "../../types/exportActivity";
import { formatBytes } from "../../utils";
import {
  HiOutlineArchiveBox,
  HiOutlineDocumentDuplicate,
  HiOutlineArrowDownTray,
  HiOutlineCheckCircle,
  HiOutlineXCircle,
  HiOutlineClock,
  HiOutlineArrowPath,
  HiOutlinePause,
} from "solid-icons/hi";
import { Component } from "solid-js";
import { logger } from "../../utils/logger";
const log = logger.scope("ActivityHelpers");

// Type icon mapping
export const getTypeIcon = (type: ExportActivityType): Component<{ class?: string }> => {
  switch (type) {
    case "archive": return HiOutlineArchiveBox;
    case "copy": return HiOutlineDocumentDuplicate;
    case "export": return HiOutlineArrowDownTray;
  }
};

// Status icon mapping
export const getStatusIcon = (status: ExportActivityStatus): Component<{ class?: string }> => {
  switch (status) {
    case "completed": return HiOutlineCheckCircle;
    case "failed": return HiOutlineXCircle;
    case "cancelled": return HiOutlineXCircle;
    case "pending": return HiOutlineClock;
    case "running": return HiOutlineArrowPath;
    case "paused": return HiOutlinePause;
  }
};

// Status color mapping
export const getStatusColor = (status: ExportActivityStatus): string => {
  switch (status) {
    case "completed": return "text-success";
    case "failed": return "text-error";
    case "cancelled": return "text-txt-muted";
    case "pending": return "text-warning";
    case "running": return "text-accent";
    case "paused": return "text-warning";
  }
};

// Type display name
export const getTypeName = (type: ExportActivityType): string => {
  switch (type) {
    case "archive": return "Archive";
    case "copy": return "Copy";
    case "export": return "Export";
  }
};

// Calculate transfer speed
export const calculateSpeed = (activity: ExportActivity): string | null => {
  if (!activity.startTime || activity.status !== "running") return null;
  const elapsed = Date.now() - activity.startTime.getTime();
  if (elapsed < 1000) return null; // Avoid initial jump
  const bytesPerSecond = (activity.progress?.bytesProcessed || 0) / (elapsed / 1000);
  return `${formatBytes(bytesPerSecond)}/s`;
};

// Get speed color based on transfer rate
export const getSpeedColor = (activity: ExportActivity): string | null => {
  if (!activity.startTime || activity.status !== "running") return null;
  const elapsed = Date.now() - activity.startTime.getTime();
  if (elapsed < 1000) return null;
  const bytesPerSecond = (activity.progress?.bytesProcessed || 0) / (elapsed / 1000);
  const mbps = bytesPerSecond / (1024 ** 2);
  if (mbps > 100) return "text-success";
  if (mbps > 50) return "text-accent";
  if (mbps > 10) return "text-warning";
  return "text-txt-muted";
};

// Calculate estimated time remaining
export const calculateETA = (activity: ExportActivity): string | null => {
  if (!activity.startTime || activity.status !== "running") return null;
  const { bytesProcessed, totalBytes } = activity.progress || {};
  if (!bytesProcessed || !totalBytes || bytesProcessed <= 0) return null;
  
  const elapsed = Date.now() - activity.startTime.getTime();
  if (elapsed < 1000) return null;
  
  const bytesPerMs = bytesProcessed / elapsed;
  const remaining = totalBytes - bytesProcessed;
  const etaMs = remaining / bytesPerMs;
  
  if (etaMs < 60000) return `${Math.round(etaMs / 1000)}s`;
  if (etaMs < 3600000) return `${Math.round(etaMs / 60000)}m`;
  return `${Math.round(etaMs / 3600000)}h`;
};

// Format duration
export const formatDuration = (ms: number): string => {
  if (ms < 1000) return "<1s";
  if (ms < 60000) return `${Math.round(ms / 1000)}s`;
  if (ms < 3600000) return `${Math.round(ms / 60000)}m`;
  return `${Math.round(ms / 3600000)}h`;
};

// Format compression ratio
export const formatCompressionRatio = (activity: ExportActivity): string | null => {
  const { originalSize, finalSize } = activity.metadata || {};
  if (!originalSize || !finalSize) return null;
  const ratio = (finalSize / originalSize) * 100;
  return `${ratio.toFixed(1)}% of original`;
};

// Open destination folder
export const handleOpenDestination = async (path: string): Promise<void> => {
  try {
    await openUrl(path);
  } catch (error) {
    log.error("Failed to open destination:", error);
  }
};

// Extract filename from path
export const getFileName = (path: string): string => {
  return path.split("/").pop() || path.split("\\").pop() || path;
};
