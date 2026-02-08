// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For, Component, createMemo } from "solid-js";
import {
  HiOutlineArchiveBox,
  HiOutlineArrowUpTray,
  HiOutlineDocumentDuplicate,
  HiOutlineCheckCircle,
  HiOutlineXCircle,
  HiOutlineExclamationTriangle,
  HiOutlineClock,
  HiOutlineXMark,
  HiOutlineFolderOpen,
  HiOutlinePause,
  HiOutlinePlay,
} from "solid-icons/hi";
import type { Activity } from "../types/activity";
import {
  formatBytes,
  calculateSpeed,
  formatSpeed,
  calculateETA,
  formatETA,
  getDuration,
  formatDuration,
} from "../types/activity";
import { invoke } from "@tauri-apps/api/core";

interface SimpleActivityPanelProps {
  activities: Activity[];
  onCancel?: (id: string) => void;
  onClear?: (id: string) => void;
  onPause?: (id: string) => void;
  onResume?: (id: string) => void;
}

/**
 * Simplified activity panel - shows real progress from the library
 */
export const SimpleActivityPanel: Component<SimpleActivityPanelProps> = (props) => {
  // Sort: running first, then pending, then completed
  const sortedActivities = createMemo(() => {
    return [...props.activities].sort((a, b) => {
      const order = { running: 0, paused: 1, pending: 2, completed: 3, failed: 4, cancelled: 5 };
      return order[a.status] - order[b.status];
    });
  });

  const activeCount = createMemo(() =>
    props.activities.filter((a) => a.status === "running" || a.status === "pending" || a.status === "paused").length
  );

  const handleOpenDestination = async (path: string) => {
    try {
      const dir = path.substring(0, path.lastIndexOf("/"));
      await invoke("plugin:opener|open_path", { path: dir });
    } catch (error) {
      console.error("Failed to open destination:", error);
    }
  };

  return (
    <div class="flex flex-col h-full">
      {/* Header */}
      <div class="flex items-center justify-between px-3 py-2 border-b border-border">
        <span class="text-sm font-medium text-txt">
          Activities
          <Show when={activeCount() > 0}>
            <span class="ml-1.5 px-1.5 py-0.5 text-xs bg-accent/20 text-accent rounded">
              {activeCount()}
            </span>
          </Show>
        </span>
      </div>

      {/* Activity List */}
      <div class="flex-1 overflow-y-auto p-2 space-y-2">
        <Show
          when={sortedActivities().length > 0}
          fallback={
            <div class="flex items-center justify-center h-32 text-txt-muted text-sm">
              No activities
            </div>
          }
        >
          <For each={sortedActivities()}>
            {(activity) => (
              <ActivityCard
                activity={activity}
                onCancel={props.onCancel}
                onClear={props.onClear}
                onPause={props.onPause}
                onResume={props.onResume}
                onOpenDestination={handleOpenDestination}
              />
            )}
          </For>
        </Show>
      </div>
    </div>
  );
};

/**
 * Individual activity card
 */
const ActivityCard: Component<{
  activity: Activity;
  onCancel?: (id: string) => void;
  onClear?: (id: string) => void;
  onPause?: (id: string) => void;
  onResume?: (id: string) => void;
  onOpenDestination: (path: string) => void;
}> = (props) => {
  const activity = () => props.activity;

  const TypeIcon = () => {
    switch (activity().type) {
      case "archive":
        return HiOutlineArchiveBox;
      case "export":
        return HiOutlineArrowUpTray;
      case "copy":
        return HiOutlineDocumentDuplicate;
    }
  };

  const StatusIcon = () => {
    switch (activity().status) {
      case "pending":
      case "running":
        return HiOutlineClock;
      case "paused":
        return HiOutlinePause;
      case "completed":
        return HiOutlineCheckCircle;
      case "failed":
        return HiOutlineXCircle;
      case "cancelled":
        return HiOutlineExclamationTriangle;
    }
  };

  const statusColor = () => {
    switch (activity().status) {
      case "pending":
        return "text-txt-muted";
      case "running":
        return "text-accent";
      case "paused":
        return "text-warning";
      case "completed":
        return "text-success";
      case "failed":
        return "text-error";
      case "cancelled":
        return "text-warning";
    }
  };

  const fileName = () => activity().destination.split("/").pop() || activity().destination;

  const speed = () => calculateSpeed(activity());
  const eta = () => calculateETA(activity());
  const duration = () => getDuration(activity());

  const isActive = () => activity().status === "running" || activity().status === "pending" || activity().status === "paused";
  const isFinished = () =>
    activity().status === "completed" ||
    activity().status === "failed" ||
    activity().status === "cancelled";

  const Icon = TypeIcon();
  const Status = StatusIcon();

  return (
    <div class="card p-2.5 space-y-2">
      {/* Header: Icon, Name, Status, Actions */}
      <div class="flex items-center justify-between gap-2">
        <div class="flex items-center gap-1.5 flex-1 min-w-0">
          <Icon class="w-4 h-4 text-txt-muted flex-shrink-0" />
          <span class="text-xs font-medium text-txt truncate">{fileName()}</span>
          <Status class={`w-3.5 h-3.5 flex-shrink-0 ${statusColor()}`} />
        </div>

        <div class="flex items-center gap-1">
          <Show when={isFinished()}>
            <button
              class="icon-btn-sm"
              onClick={() => props.onOpenDestination(activity().destination)}
              title="Open folder"
            >
              <HiOutlineFolderOpen class="w-3.5 h-3.5" />
            </button>
          </Show>

          {/* Pause button - show when running */}
          <Show when={activity().status === "running" && props.onPause}>
            <button
              class="icon-btn-sm text-warning"
              onClick={() => props.onPause?.(activity().id)}
              title="Pause"
            >
              <HiOutlinePause class="w-3.5 h-3.5" />
            </button>
          </Show>

          {/* Resume button - show when paused */}
          <Show when={activity().status === "paused" && props.onResume}>
            <button
              class="icon-btn-sm text-accent"
              onClick={() => props.onResume?.(activity().id)}
              title="Resume"
            >
              <HiOutlinePlay class="w-3.5 h-3.5" />
            </button>
          </Show>

          {/* Cancel button - show for any active status */}
          <Show when={isActive() && props.onCancel}>
            <button
              class="icon-btn-sm text-error"
              onClick={() => props.onCancel?.(activity().id)}
              title="Cancel"
            >
              <HiOutlineXMark class="w-3.5 h-3.5" />
            </button>
          </Show>

          <Show when={isFinished() && props.onClear}>
            <button
              class="icon-btn-sm"
              onClick={() => props.onClear?.(activity().id)}
              title="Clear"
            >
              <HiOutlineXMark class="w-3.5 h-3.5" />
            </button>
          </Show>
        </div>
      </div>

      {/* Progress Bar (when running) */}
      <Show when={activity().progress && activity().status === "running"}>
        <div class="space-y-1">
          {/* Progress bar */}
          <div class="h-1.5 bg-bg-secondary rounded-full overflow-hidden">
            <div
              class="h-full bg-accent transition-all duration-300"
              style={{ width: `${activity().progress?.percent || 0}%` }}
            />
          </div>

          {/* Stats row */}
          <div class="flex items-center justify-between text-[10px] text-txt-muted">
            <span>
              {formatBytes(activity().progress?.bytesProcessed || 0)}
              <Show when={activity().progress?.bytesTotal}>
                {" "}/ {formatBytes(activity().progress!.bytesTotal)}
              </Show>
            </span>

            <div class="flex items-center gap-2">
              <Show when={speed()}>
                <span class="text-accent">{formatSpeed(speed()!)}</span>
              </Show>
              <Show when={eta()}>
                <span>ETA: {formatETA(eta()!)}</span>
              </Show>
              <span>{activity().progress?.percent.toFixed(0)}%</span>
            </div>
          </div>

          {/* Current file */}
          <Show when={activity().progress?.currentFile}>
            <div class="text-[10px] text-txt-muted truncate">
              {activity().progress!.currentFile}
            </div>
          </Show>
        </div>
      </Show>

      {/* Completed stats */}
      <Show when={activity().status === "completed" && duration()}>
        <div class="text-[10px] text-txt-muted">
          Completed in {formatDuration(duration()!)}
          <Show when={activity().progress?.bytesProcessed}>
            {" "}• {formatBytes(activity().progress!.bytesProcessed)}
          </Show>
        </div>
      </Show>

      {/* Error message */}
      <Show when={activity().status === "failed" && activity().error}>
        <div class="text-[10px] text-error truncate" title={activity().error}>
          {activity().error}
        </div>
      </Show>

      {/* Source count */}
      <Show when={activity().sourceCount > 1}>
        <div class="text-[10px] text-txt-muted">{activity().sourceCount} items</div>
      </Show>
    </div>
  );
};

export default SimpleActivityPanel;
