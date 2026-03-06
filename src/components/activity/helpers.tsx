// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { JSX } from "solid-js";
import {
  HiOutlineClock,
  HiOutlineDocumentText,
  HiOutlineFolder,
  HiOutlineFingerPrint,
  HiOutlineArrowUpTray,
  HiOutlineMagnifyingGlass,
  HiOutlineBookmark,
  HiOutlinePencilSquare,
} from "../icons";

/** Get icon element for activity category */
export const getCategoryIcon = (category: string): JSX.Element => {
  switch (category) {
    case "project":
      return <HiOutlineFolder class="w-3.5 h-3.5" />;
    case "file":
      return <HiOutlineDocumentText class="w-3.5 h-3.5" />;
    case "hash":
      return <HiOutlineFingerPrint class="w-3.5 h-3.5" />;
    case "export":
      return <HiOutlineArrowUpTray class="w-3.5 h-3.5" />;
    case "search":
      return <HiOutlineMagnifyingGlass class="w-3.5 h-3.5" />;
    case "bookmark":
      return <HiOutlineBookmark class="w-3.5 h-3.5" />;
    case "note":
      return <HiOutlinePencilSquare class="w-3.5 h-3.5" />;
    default:
      return <HiOutlineClock class="w-3.5 h-3.5" />;
  }
};

/** Format duration in seconds to human readable */
export const formatDuration = (seconds: number | null | undefined): string => {
  if (!seconds) return "—";
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  return `${hours}h ${mins}m`;
};

/** Format timestamp to relative or absolute */
export const formatTimestamp = (timestamp: string, relative = true): string => {
  const date = new Date(timestamp);
  if (relative) {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return "just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
  }
  return date.toLocaleDateString() + " " + date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
};
