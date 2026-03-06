// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { JSX } from "solid-js";
import type { SystemStats } from "../../hooks";

/** Autosave status for project persistence */
export type AutoSaveStatus = "idle" | "saving" | "saved" | "modified" | "error";

/** Progress item for background tasks */
export interface ProgressItem {
  id: string;
  label: string;
  progress: number; // 0-100
  indeterminate?: boolean;
  onCancel?: () => void;
  onClick?: () => void; // Click to navigate to the task
}

/** Quick action button */
export interface QuickAction {
  icon: JSX.Element;
  label: string;
  onClick: () => void;
}

/** StatusBar props */
export interface StatusBarProps {
  statusKind: "idle" | "working" | "ok" | "error";
  statusMessage: string;
  discoveredCount: number;
  totalSize: number;
  selectedCount: number;
  systemStats: SystemStats | null;
  progressItems?: ProgressItem[];
  quickActions?: QuickAction[];
  autoSaveStatus?: AutoSaveStatus;
  autoSaveEnabled?: boolean;
  lastAutoSave?: Date | null;
  onAutoSaveToggle?: () => void;
  activityCount?: number;
  bookmarkCount?: number;
  noteCount?: number;
  onEvidenceClick?: () => void;
  onBookmarkClick?: () => void;
  onActivityClick?: () => void;
  onPerformanceClick?: () => void;
}
