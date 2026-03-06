// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, Component } from "solid-js";
import { getBasename } from "../../utils/pathUtils";
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
} from "../icons";
import {
  calculateSpeed,
  formatSpeed,
  calculateETA,
  formatETA,
  getDuration,
  formatDuration,
} from "../../types/activity";
import { formatBytes } from "../../utils";
import type { ActivityCardProps } from "./types";

/**
 * Individual activity card
 */
export const ActivityCard: Component<ActivityCardProps> = (props) => {
  const activity = () => props.activity;

  const TypeIcon = (): Component<{ class?: string }> => {
    switch (activity().type) {
      case "archive":
        return HiOutlineArchiveBox;
      case "export":
        return HiOutlineArrowUpTray;
      case "copy":
        return HiOutlineDocumentDuplicate;
      default:
        return HiOutlineArchiveBox;
    }
  };

  const StatusIcon = (): Component<{ class?: string }> => {
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
      default:
        return HiOutlineClock;
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

  const fileName = () =>
    getBasename(activity().destination) || activity().destination;

  const speed = () => calculateSpeed(activity());
  const eta = () => calculateETA(activity());
  const duration = () => getDuration(activity());

  const isActive = () =>
    activity().status === "running" ||
    activity().status === "pending" ||
    activity().status === "paused";
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
          <span class="text-xs font-medium text-txt truncate">
            {fileName()}
          </span>
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
                {" "}
                / {formatBytes(activity().progress!.bytesTotal || 0)}
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
            {" "}
            • {formatBytes(activity().progress!.bytesProcessed || 0)}
          </Show>
        </div>
      </Show>

      {/* Error message */}
      <Show when={activity().status === "failed" && activity().error}>
        <div
          class="text-[10px] text-error truncate"
          title={activity().error}
        >
          {activity().error}
        </div>
      </Show>

      {/* Source count */}
      <Show when={activity().sourceCount > 1}>
        <div class="text-[10px] text-txt-muted">
          {activity().sourceCount} items
        </div>
      </Show>
    </div>
  );
};
