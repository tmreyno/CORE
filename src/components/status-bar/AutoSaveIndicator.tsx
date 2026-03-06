// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { AutoSaveStatus } from "./types";
import {
  HiOutlineCloud,
  HiOutlineCloudArrowUp,
  HiOutlineExclamationTriangle,
} from "../icons";

interface AutoSaveIndicatorProps {
  status: AutoSaveStatus;
  enabled: boolean;
  lastSave?: Date | null;
  onToggle?: () => void;
}

/** Autosave status indicator — shows cloud icon + status text */
export function AutoSaveIndicator(props: AutoSaveIndicatorProps) {
  const formatLastSave = () => {
    if (!props.lastSave) return "";
    const now = new Date();
    const diff = Math.floor((now.getTime() - props.lastSave.getTime()) / 1000);
    if (diff < 60) return "just now";
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    return props.lastSave.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  };

  const getStatusIcon = () => {
    switch (props.status) {
      case "saving":
        return <HiOutlineCloudArrowUp class="w-3.5 h-3.5 animate-pulse text-accent" />;
      case "saved":
        return <HiOutlineCloud class="w-3.5 h-3.5 text-success" />;
      case "modified":
        return <HiOutlineCloud class="w-3.5 h-3.5 text-warning" />;
      case "error":
        return <HiOutlineExclamationTriangle class="w-3.5 h-3.5 text-error" />;
      default:
        return <HiOutlineCloud class="w-3.5 h-3.5 text-txt-muted" />;
    }
  };

  const getStatusText = () => {
    switch (props.status) {
      case "saving": return "Saving...";
      case "saved": return `Saved ${formatLastSave()}`;
      case "modified": return "Unsaved changes";
      case "error": return "Save failed";
      default: return props.enabled ? "Autosave on" : "Autosave off";
    }
  };

  const getTooltip = () => {
    const lines: string[] = [];
    lines.push(props.enabled ? "Autosave enabled" : "Autosave disabled");
    if (props.lastSave) lines.push(`Last saved: ${props.lastSave.toLocaleString()}`);
    if (props.status === "modified") lines.push("Click to save now");
    return lines.join("\n");
  };

  return (
    <button
      class={`flex items-center gap-1 text-xs px-1.5 py-0.5 rounded transition-colors cursor-pointer bg-transparent border-none ${
        props.status === "modified" ? "bg-warning/10 hover:bg-warning/20" :
        props.status === "error" ? "bg-error/10 hover:bg-error/20" :
        "hover:bg-bg-hover"
      }`}
      onClick={props.onToggle}
      title={getTooltip()}
    >
      {getStatusIcon()}
      <span class={`${
        props.status === "saving" ? "text-accent" :
        props.status === "saved" ? "text-success" :
        props.status === "modified" ? "text-warning" :
        props.status === "error" ? "text-error" :
        "text-txt-muted"
      }`}>
        {getStatusText()}
      </span>
    </button>
  );
}
