// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * StatusBar - Application status footer
 * 
 * Displays:
 * - Current operation status with animated icons
 * - Evidence file counts and total size
 * - Background task progress indicators
 * - System resource usage (CPU, memory, threads)
 * - Autosave status
 * - Quick action buttons
 */

import { Show, For, JSX } from "solid-js";
import type { SystemStats } from "../hooks";
import { formatBytes } from "../utils";
import {
  HiOutlineCheckCircle,
  HiOutlineXCircle,
  HiOutlineArrowPath,
  HiOutlineMinusCircle,
  HiOutlineXMark,
  HiOutlineFolderOpen,
  HiOutlineCircleStack,
  HiOutlineCheckBadge,
  HiOutlineFire,
  HiOutlineCpuChip,
  HiOutlineBars3BottomLeft,
  HiOutlineCloud,
  HiOutlineCloudArrowUp,
  HiOutlineExclamationTriangle,
  HiOutlineClipboardDocumentList,
  HiOutlineBookmark,
  HiOutlineDocumentText,
} from "./icons";

/** Autosave status for project persistence */
export type AutoSaveStatus = "idle" | "saving" | "saved" | "modified" | "error";

// Progress item for background tasks
export interface ProgressItem {
  id: string;
  label: string;
  progress: number; // 0-100
  indeterminate?: boolean;
  onCancel?: () => void;
  onClick?: () => void; // Click to navigate to the task
}

// Quick action button
export interface QuickAction {
  icon: JSX.Element;
  label: string;
  onClick: () => void;
}

interface StatusBarProps {
  statusKind: "idle" | "working" | "ok" | "error";
  statusMessage: string;
  discoveredCount: number;
  totalSize: number;
  selectedCount: number;
  systemStats: SystemStats | null;
  // New props
  progressItems?: ProgressItem[];
  quickActions?: QuickAction[];
  // Autosave indicator
  autoSaveStatus?: AutoSaveStatus;
  autoSaveEnabled?: boolean;
  lastAutoSave?: Date | null;
  onAutoSaveToggle?: () => void;
  // Project stats
  activityCount?: number;
  bookmarkCount?: number;
  noteCount?: number;
}

export function StatusBar(props: StatusBarProps) {
  // Status icon with appropriate styling based on kind
  const StatusIcon = () => {
    const baseClass = "w-3.5 h-3.5";
    switch (props.statusKind) {
      case "working": 
        return (
          <span class="flex items-center justify-center w-5 h-5 rounded-full bg-warning/10">
            <HiOutlineArrowPath class={`${baseClass} animate-spin text-warning`} />
          </span>
        );
      case "ok": 
        return (
          <span class="flex items-center justify-center w-5 h-5 rounded-full bg-success/10">
            <HiOutlineCheckCircle class={`${baseClass} text-success`} />
          </span>
        );
      case "error": 
        return (
          <span class="flex items-center justify-center w-5 h-5 rounded-full bg-error/10">
            <HiOutlineXCircle class={`${baseClass} text-error`} />
          </span>
        );
      default: 
        return (
          <span class="flex items-center justify-center w-5 h-5 rounded-full bg-bg-hover">
            <HiOutlineMinusCircle class={`${baseClass} text-txt-muted`} />
          </span>
        );
    }
  };

  // Autosave indicator
  const AutoSaveIndicator = () => {
    const status = props.autoSaveStatus || "idle";
    const enabled = props.autoSaveEnabled ?? false;
    const lastSave = props.lastAutoSave;
    
    // Format last save time
    const formatLastSave = () => {
      if (!lastSave) return "";
      const now = new Date();
      const diff = Math.floor((now.getTime() - lastSave.getTime()) / 1000);
      if (diff < 60) return "just now";
      if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
      return lastSave.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    };
    
    const getStatusIcon = () => {
      switch (status) {
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
      switch (status) {
        case "saving": return "Saving...";
        case "saved": return `Saved ${formatLastSave()}`;
        case "modified": return "Unsaved changes";
        case "error": return "Save failed";
        default: return enabled ? "Autosave on" : "Autosave off";
      }
    };
    
    const getTooltip = () => {
      const lines = [];
      lines.push(enabled ? "Autosave enabled" : "Autosave disabled");
      if (lastSave) lines.push(`Last saved: ${lastSave.toLocaleString()}`);
      if (status === "modified") lines.push("Click to save now");
      return lines.join("\n");
    };
    
    return (
      <button
        class={`flex items-center gap-1 text-xs px-1.5 py-0.5 rounded transition-colors cursor-pointer bg-transparent border-none ${
          status === "modified" ? "bg-warning/10 hover:bg-warning/20" :
          status === "error" ? "bg-error/10 hover:bg-error/20" :
          "hover:bg-bg-hover"
        }`}
        onClick={props.onAutoSaveToggle}
        title={getTooltip()}
      >
        {getStatusIcon()}
        <span class={`${
          status === "saving" ? "text-accent" :
          status === "saved" ? "text-success" :
          status === "modified" ? "text-warning" :
          status === "error" ? "text-error" :
          "text-txt-muted"
        }`}>
          {getStatusText()}
        </span>
      </button>
    );
  };

  // Format CPU usage - show cores used (e.g., "4.9 cores" instead of "489%")
  const formatCpuUsage = (cpuPercent: number | undefined, cores: number | undefined) => {
    if (cpuPercent === undefined || cores === undefined) return '—';
    const coresUsed = cpuPercent / 100;
    if (coresUsed >= 1) {
      return `${coresUsed.toFixed(1)}/${cores}`;
    }
    return `${cpuPercent.toFixed(0)}%`;
  };

  // Parse status message to add animated icons
  const renderStatusMessage = () => {
    const msg = props.statusMessage;
    // Replace hash symbol with animated version, remove book emoji
    const cleanedMsg = msg.replace(/📖/g, '');
    const parts = cleanedMsg.split(/(#)/g);
    return parts.map((part) => {
      if (part === "#") {
        return <span class="inline-block text-base font-black animate-spin-slow">#</span>;
      }
      return part;
    });
  };

  const statusBg = () => {
    switch (props.statusKind) {
      case "idle": return "bg-bg-card text-txt-muted";
      case "working": return "bg-warning-soft text-warning";
      case "ok": return "bg-success-soft text-success";
      case "error": return "bg-error-soft text-error";
    }
  };

  return (
    <footer class="status-bar flex flex-col bg-bg-toolbar border-t border-border shrink-0" role="status" aria-live="polite" aria-label="Application status">
      <div class={`flex items-center gap-2 px-4 py-1.5 text-sm ${statusBg()}`}>
        <span class="shrink-0"><StatusIcon /></span>
        <span class="font-medium truncate flex items-center gap-0.5">{renderStatusMessage()}</span>
        
        {/* Progress items for background tasks */}
        <Show when={props.progressItems && props.progressItems.length > 0}>
          <div class="flex items-center gap-3 ml-2 pl-2 border-l border-border/30">
            <For each={props.progressItems}>
              {(item) => (
                <div 
                  class={`flex items-center gap-2 max-w-48 ${item.onClick ? 'cursor-pointer hover:bg-bg-hover rounded px-1 -mx-1' : ''}`}
                  onClick={item.onClick}
                  title={item.onClick ? "Click to view transfer details" : undefined}
                >
                  <span class="text-xs text-txt-muted truncate">{item.label}</span>
                  <div class="w-16 h-1 bg-bg rounded-full overflow-hidden">
                    <div
                      class={`h-full bg-accent transition-all duration-300 ${
                        item.indeterminate ? 'animate-[indeterminate_1.5s_infinite]' : ''
                      }`}
                      style={{ width: item.indeterminate ? '30%' : `${item.progress}%` }}
                    />
                  </div>
                  <Show when={!item.indeterminate}>
                    <span class="text-xs text-txt-muted">{Math.round(item.progress)}%</span>
                  </Show>
                  <Show when={item.onCancel}>
                    <button
                      class="text-txt-muted hover:text-txt cursor-pointer bg-transparent border-none p-0.5"
                      onClick={(e) => { e.stopPropagation(); item.onCancel?.(); }}
                      title="Cancel"
                    >
                      <HiOutlineXMark class="w-2 h-2" />
                    </button>
                  </Show>
                </div>
              )}
            </For>
          </div>
        </Show>
        
        <div class="flex items-center gap-1.5 text-xs opacity-80">
          <Show when={props.discoveredCount > 0}>
            <span class="flex items-center gap-0.5"><HiOutlineFolderOpen class="w-3 h-3" /> {props.discoveredCount}</span>
            <span class="flex items-center gap-0.5"><HiOutlineCircleStack class="w-3 h-3" /> {formatBytes(props.totalSize)}</span>
          </Show>
          <Show when={props.selectedCount > 0}>
            <span class="flex items-center gap-0.5"><HiOutlineCheckBadge class="w-3 h-3" /> {props.selectedCount}</span>
          </Show>
        </div>
        
        {/* Project Stats: activity events, bookmarks, notes */}
        <Show when={(props.activityCount ?? 0) > 0 || (props.bookmarkCount ?? 0) > 0 || (props.noteCount ?? 0) > 0}>
          <div class="flex items-center gap-1.5 text-xs opacity-80 pl-2 border-l border-border/30">
            <Show when={(props.activityCount ?? 0) > 0}>
              <span class="flex items-center gap-0.5" title={`${props.activityCount} activity events`}>
                <HiOutlineClipboardDocumentList class="w-3 h-3" /> {props.activityCount}
              </span>
            </Show>
            <Show when={(props.bookmarkCount ?? 0) > 0}>
              <span class="flex items-center gap-0.5" title={`${props.bookmarkCount} bookmarks`}>
                <HiOutlineBookmark class="w-3 h-3" /> {props.bookmarkCount}
              </span>
            </Show>
            <Show when={(props.noteCount ?? 0) > 0}>
              <span class="flex items-center gap-0.5" title={`${props.noteCount} notes`}>
                <HiOutlineDocumentText class="w-3 h-3" /> {props.noteCount}
              </span>
            </Show>
          </div>
        </Show>
        
        {/* Autosave indicator */}
        <Show when={props.autoSaveStatus !== undefined}>
          <div class="flex items-center pl-2 border-l border-border/30">
            <AutoSaveIndicator />
          </div>
        </Show>
        
        <div class="flex items-center gap-2 ml-auto text-xs font-mono pl-3 border-l border-border/30">
          <Show when={props.systemStats}>
            <span class="flex items-center gap-0.5" title={`App CPU: ${props.systemStats!.appCpuUsage?.toFixed(1) ?? '—'}% (${((props.systemStats!.appCpuUsage ?? 0) / 100).toFixed(1)} cores)\nSystem CPU: ${props.systemStats!.cpuUsage?.toFixed(1) ?? '—'}%\nCores: ${props.systemStats!.cpuCores ?? '—'}`}>
              <HiOutlineFire class="w-3 h-3" /> {formatCpuUsage(props.systemStats!.appCpuUsage, props.systemStats!.cpuCores)}
            </span>
            <span class="flex items-center gap-0.5" title={`App Memory: ${formatBytes(props.systemStats!.appMemory ?? 0)}\nSystem: ${formatBytes(props.systemStats!.memoryUsed ?? 0)} / ${formatBytes(props.systemStats!.memoryTotal ?? 0)} (${props.systemStats!.memoryPercent?.toFixed(1) ?? '—'}%)`}>
              <HiOutlineCpuChip class="w-3 h-3" /> {formatBytes(props.systemStats!.appMemory ?? 0)}
            </span>
            <span class="flex items-center gap-0.5" title={`Worker threads: ${props.systemStats!.appThreads ?? '—'}`}>
              <HiOutlineBars3BottomLeft class="w-3 h-3" /> {props.systemStats!.appThreads ?? '—'}
            </span>
          </Show>
          
          {/* Quick actions */}
          <Show when={props.quickActions && props.quickActions.length > 0}>
            <div class="flex items-center gap-1 ml-2 pl-2 border-l border-border/30">
              <For each={props.quickActions}>
                {(action) => (
                  <button
                    class="p-1 text-txt-muted hover:text-txt hover:bg-bg-hover rounded transition-colors cursor-pointer bg-transparent border-none"
                    onClick={action.onClick}
                    title={action.label}
                    aria-label={action.label}
                  >
                    {action.icon}
                  </button>
                )}
              </For>
            </div>
          </Show>
        </div>
      </div>
    </footer>
  );
}
