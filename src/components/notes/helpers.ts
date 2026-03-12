// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { NoteTargetType } from "./types";
import {
  HiOutlineDocumentText,
  HiOutlineBeaker,
  HiOutlineCircleStack,
  HiOutlineBriefcase,
  HiOutlineChatBubbleBottomCenterText,
} from "../icons";
import type { Component } from "solid-js";

// =============================================================================
// Icon / Label helpers
// =============================================================================

export function getNoteTargetTypeLabel(type: NoteTargetType): string {
  switch (type) {
    case "file": return "File";
    case "artifact": return "Artifact";
    case "database": return "Database";
    case "case": return "Case";
    case "general": return "General";
    default: return type;
  }
}

export function getNoteTargetTypeIcon(type: NoteTargetType): Component<{ class?: string }> {
  switch (type) {
    case "file": return HiOutlineDocumentText;
    case "artifact": return HiOutlineBeaker;
    case "database": return HiOutlineCircleStack;
    case "case": return HiOutlineBriefcase;
    case "general": return HiOutlineChatBubbleBottomCenterText;
    default: return HiOutlineDocumentText;
  }
}

export function getPriorityColor(priority?: string): string {
  switch (priority) {
    case "critical": return "text-error";
    case "high": return "text-warning";
    case "normal": return "text-txt";
    case "low": return "text-txt-muted";
    default: return "text-txt";
  }
}

export function getPriorityLabel(priority?: string): string {
  switch (priority) {
    case "critical": return "Critical";
    case "high": return "High";
    case "normal": return "Normal";
    case "low": return "Low";
    default: return "Normal";
  }
}

/** Format a date string for display */
export function formatNoteDate(dateStr: string): string {
  try {
    const d = new Date(dateStr);
    const now = new Date();
    const diff = now.getTime() - d.getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return "Just now";
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    if (days < 7) return `${days}d ago`;
    return d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
  } catch {
    return dateStr;
  }
}
