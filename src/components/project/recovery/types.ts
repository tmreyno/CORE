// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * RecoveryModal types and shared helper functions.
 */

import type { Component } from "solid-js";
import {
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
  HiOutlineXCircle,
  HiOutlineShieldCheck,
} from "../../icons";
import type { ProjectHealthStatus } from "../../../hooks/useProjectRecovery";

export interface RecoveryModalProps {
  isOpen: boolean;
  onClose: () => void;
  projectPath: string;
}

// ── Health status helpers ──

export function getHealthIcon(
  status: ProjectHealthStatus,
): Component<{ class?: string }> {
  switch (status) {
    case "Healthy":
      return HiOutlineCheckCircle;
    case "Warning":
      return HiOutlineExclamationTriangle;
    case "Critical":
      return HiOutlineXCircle;
    default:
      return HiOutlineShieldCheck;
  }
}

export function getHealthColor(status: ProjectHealthStatus): string {
  switch (status) {
    case "Healthy":
      return "text-success";
    case "Warning":
      return "text-warning";
    case "Critical":
      return "text-error";
    default:
      return "text-txt-muted";
  }
}

// ── Formatting helpers ──

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

export function formatAutosaveAge(seconds: number | null): string {
  if (seconds === null) return "Unknown";
  if (seconds < 60) return `${seconds} seconds ago`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)} minutes ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)} hours ago`;
  return `${Math.floor(seconds / 86400)} days ago`;
}
