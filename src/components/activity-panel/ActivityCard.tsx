// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, Component, createEffect, on } from "solid-js";
import { CoreSpinner, CoreProgressBar } from "@core-suite/icons";
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
  getDuration,
  formatDuration,
} from "../../types/activity";
import { formatBytes } from "../../utils";
import { createProgressTracker } from "../../hooks/useProgressTracker";
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

  const tracker = createProgressTracker();

  // Feed progress updates into the smoothed tracker
  createEffect(
    on(
      () => activity().progress,
      (p) => {
        if (p && activity().status === "running") {
          tracker.update({
            bytesProcessed: p.bytesProcessed || 0,
            bytesTotal: p.bytesTotal || 0,
            percent: p.percent,
          });
        }
      }
    )
  );

  // Reset tracker when activity restarts or changes
  createEffect(
    on(
      () => activity().id,
      () => tracker.reset()
    )
  );

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
        <div class="space-y-1.5">
          {/* Percent */}
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-1.5">
              <CoreSpinner size={12} />
              <Show when={activity().progress?.currentFile}>
                <span class="text-xs text-txt-muted truncate max-w-[140px]">
                  {activity().progress!.currentFile}
                </span>
              </Show>
            </div>
            <span class="text-sm font-semibold text-accent">
              {activity().progress?.percent.toFixed(1)}%
            </span>
          </div>

          {/* Animated progress bar */}
          <CoreProgressBar progress={activity().progress?.percent || 0} height={8} showSpinner={false} />

          {/* Stats row */}
          <div class="flex items-center justify-between text-xs text-txt-muted">
            <span>
              {formatBytes(activity().progress?.bytesProcessed || 0)}
              <Show when={activity().progress?.bytesTotal}>
                {" / "}
                {formatBytes(activity().progress!.bytesTotal || 0)}
              </Show>
            </span>

            <div class="flex items-center gap-2">
              <Show when={tracker.stats().speedBps != null && tracker.stats().speedBps! > 0}>
                <span class="text-accent">
                  {tracker.stats().speedFormatted}
                </span>
              </Show>
              <Show when={tracker.stats().etaMs != null && tracker.stats().etaMs! > 0}>
                <span class="font-medium">
                  ETA: {tracker.stats().etaFormatted}
                </span>
              </Show>
            </div>
          </div>

          {/* Elapsed time */}
          <Show when={tracker.stats().elapsedMs > 1000}>
            <div class="text-xs text-txt-muted">
              Elapsed: {tracker.stats().elapsedFormatted}
            </div>
          </Show>
        </div>
      </Show>

      {/* Completed stats */}
      <Show when={activity().status === "completed" && duration()}>
        <div class="text-xs text-txt-muted">
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
          class="text-xs text-error truncate"
          title={activity().error}
        >
          {activity().error}
        </div>
      </Show>

      {/* Source count */}
      <Show when={activity().sourceCount > 1}>
        <div class="text-xs text-txt-muted">
          {activity().sourceCount} items
        </div>
      </Show>
    </div>
  );
};
