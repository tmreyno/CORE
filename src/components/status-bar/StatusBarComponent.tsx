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

import { Show, For } from "solid-js";
import { formatBytes } from "../../utils";
import type { StatusBarProps } from "./types";
import { formatCpuUsage } from "./helpers";
import { StatusIcon } from "./StatusIcon";
import { AutoSaveIndicator } from "./AutoSaveIndicator";
import {
  HiOutlineXMark,
  HiOutlineFolderOpen,
  HiOutlineCircleStack,
  HiOutlineCheckBadge,
  HiOutlineFire,
  HiOutlineCpuChip,
  HiOutlineBars3BottomLeft,
  HiOutlineClipboardDocumentList,
  HiOutlineBookmark,
  HiOutlineDocumentText,
} from "../icons";

export function StatusBar(props: StatusBarProps) {
  /** Parse status message to add animated hash icons */
  const renderStatusMessage = () => {
    const msg = props.statusMessage;
    const cleanedMsg = msg.replace(/📖/g, "");
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
        <span class="shrink-0"><StatusIcon kind={props.statusKind} /></span>
        <span class="font-medium truncate flex items-center gap-0.5">{renderStatusMessage()}</span>

        {/* Progress items for background tasks */}
        <Show when={props.progressItems && props.progressItems.length > 0}>
          <div class="flex items-center gap-3 ml-2 pl-2 border-l border-border/30">
            <For each={props.progressItems}>
              {(item) => (
                <div
                  class={`flex items-center gap-2 max-w-48 ${item.onClick ? "cursor-pointer hover:bg-bg-hover rounded px-1 -mx-1" : ""}`}
                  onClick={item.onClick}
                  title={item.onClick ? "Click to view transfer details" : undefined}
                >
                  <span class="text-xs text-txt-muted truncate">{item.label}</span>
                  <div class="w-16 h-1 bg-bg rounded-full overflow-hidden">
                    <div
                      class={`h-full bg-accent transition-all duration-300 ${
                        item.indeterminate ? "animate-[indeterminate_1.5s_infinite]" : ""
                      }`}
                      style={{ width: item.indeterminate ? "30%" : `${item.progress}%` }}
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

        <div class={`flex items-center gap-1.5 text-xs opacity-80 ${props.onEvidenceClick ? "cursor-pointer hover:opacity-100" : ""}`} onClick={props.onEvidenceClick}>
          <Show when={props.discoveredCount > 0}>
            <span class="flex items-center gap-0.5" title="Click to show evidence"><HiOutlineFolderOpen class="w-3 h-3" /> {props.discoveredCount}</span>
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
              <span
                class={`flex items-center gap-0.5 ${props.onActivityClick ? "cursor-pointer hover:opacity-100 hover:text-accent" : ""}`}
                title={`${props.activityCount} activity events — click to show`}
                onClick={props.onActivityClick}
              >
                <HiOutlineClipboardDocumentList class="w-3 h-3" /> {props.activityCount}
              </span>
            </Show>
            <Show when={(props.bookmarkCount ?? 0) > 0}>
              <span
                class={`flex items-center gap-0.5 ${props.onBookmarkClick ? "cursor-pointer hover:opacity-100 hover:text-accent" : ""}`}
                title={`${props.bookmarkCount} bookmarks — click to show`}
                onClick={props.onBookmarkClick}
              >
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
            <AutoSaveIndicator
              status={props.autoSaveStatus || "idle"}
              enabled={props.autoSaveEnabled ?? false}
              lastSave={props.lastAutoSave}
              onToggle={props.onAutoSaveToggle}
            />
          </div>
        </Show>

        <div class={`flex items-center gap-2 ml-auto text-xs font-mono pl-3 border-l border-border/30 ${props.onPerformanceClick ? "cursor-pointer hover:opacity-100" : ""}`} onClick={props.onPerformanceClick}>
          <Show when={props.systemStats}>
            <span class="flex items-center gap-0.5" title={`App CPU: ${props.systemStats!.appCpuUsage?.toFixed(1) ?? "—"}% (${((props.systemStats!.appCpuUsage ?? 0) / 100).toFixed(1)} cores)\nSystem CPU: ${props.systemStats!.cpuUsage?.toFixed(1) ?? "—"}%\nCores: ${props.systemStats!.cpuCores ?? "—"}\nClick to open Performance Monitor`}>
              <HiOutlineFire class="w-3 h-3" /> {formatCpuUsage(props.systemStats!.appCpuUsage, props.systemStats!.cpuCores)}
            </span>
            <span class="flex items-center gap-0.5" title={`App Memory: ${formatBytes(props.systemStats!.appMemory ?? 0)}\nSystem: ${formatBytes(props.systemStats!.memoryUsed ?? 0)} / ${formatBytes(props.systemStats!.memoryTotal ?? 0)} (${props.systemStats!.memoryPercent?.toFixed(1) ?? "—"}%)`}>
              <HiOutlineCpuChip class="w-3 h-3" /> {formatBytes(props.systemStats!.appMemory ?? 0)}
            </span>
            <span class="flex items-center gap-0.5" title={`Worker threads: ${props.systemStats!.appThreads ?? "—"}`}>
              <HiOutlineBars3BottomLeft class="w-3 h-3" /> {props.systemStats!.appThreads ?? "—"}
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
