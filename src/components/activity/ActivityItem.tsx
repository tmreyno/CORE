// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import {
  HiOutlinePlay,
  HiOutlinePause,
  HiOutlineXMark,
  HiOutlineFolderOpen,
  HiOutlineChevronRight,
  HiOutlineChevronDown,
  HiOutlineFolder,
  HiOutlineDocument,
} from "solid-icons/hi";
import type { ExportActivity } from "../../types/exportActivity";
import { usePreferences } from "../preferences";
import {
  getTypeIcon,
  getStatusIcon,
  getStatusColor,
  formatBytes,
  calculateSpeed,
  getSpeedColor,
  calculateETA,
  formatCompressionRatio,
  formatDuration,
  handleOpenDestination,
  getFileName,
} from "./activityHelpers";

interface ActivityItemProps {
  activity: ExportActivity;
  isExpanded: boolean;
  onToggleExpanded: () => void;
  onPause?: (id: string) => void;
  onResume?: (id: string) => void;
  onCancel?: (id: string) => void;
  onClear?: (id: string) => void;
}

export const ActivityItem: Component<ActivityItemProps> = (props) => {
  const { preferences } = usePreferences();

  // Preference helper with fallback
  const pref = (key: string, fallback: any) => {
    const prefs = preferences() as Record<string, any>;
    return prefs[key] ?? fallback;
  };

  // Calculate derived values
  const TypeIcon = getTypeIcon(props.activity.type);
  const StatusIcon = getStatusIcon(props.activity.status);
  const statusColor = getStatusColor(props.activity.status);

  const duration =
    props.activity.startTime && props.activity.endTime
      ? props.activity.endTime.getTime() - props.activity.startTime.getTime()
      : props.activity.startTime
      ? Date.now() - props.activity.startTime.getTime()
      : null;

  const speed = calculateSpeed(props.activity);
  const speedColor = getSpeedColor(props.activity);
  const eta = calculateETA(props.activity);
  const compressionRatio = formatCompressionRatio(props.activity);

  const isActive = props.activity.status === "running" || props.activity.status === "pending";
  const isFinished =
    props.activity.status === "completed" ||
    props.activity.status === "failed" ||
    props.activity.status === "cancelled";
  const isPaused = props.activity.status === "paused";

  // Current file progress
  const currentFileTotal = props.activity.progress?.currentFileTotal || 0;
  const currentFileBytes = props.activity.progress?.currentFileBytes || 0;
  const currentFilePercent = currentFileTotal > 0 ? (currentFileBytes / currentFileTotal) * 100 : 0;

  // File icon based on extension
  const FileIcon = props.activity.progress?.currentFile?.endsWith("/")
    ? HiOutlineFolder
    : HiOutlineDocument;

  return (
    <div
      class={`card p-2 space-y-1.5 ${
        pref("activityPulseAnimation", true) && props.activity.status === "running"
          ? "animate-pulse-slow"
          : ""
      }`}
    >
      {/* Header */}
      <div class="flex items-start gap-1.5 justify-between">
        {/* Left: Icon + Type + Name */}
        <div class="flex items-center gap-1.5 min-w-0 flex-1">
          <TypeIcon class="w-3.5 h-3.5 text-accent flex-shrink-0" />
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-1">
              <span class="text-[11px] text-txt-muted">
                {props.activity.type.charAt(0).toUpperCase() + props.activity.type.slice(1)}
              </span>
              <span class="text-[11px] text-txt-muted">→</span>
              <span
                class="text-xs text-txt truncate"
                title={props.activity.destination}
              >
                {getFileName(props.activity.destination)}
              </span>
            </div>
          </div>
        </div>

        {/* Right: Status + Actions */}
        <div class="flex items-center gap-1.5 flex-shrink-0">
          {/* Status */}
          <div class="flex items-center gap-0.5">
            <StatusIcon class={`w-3 h-3 flex-shrink-0 ${statusColor}`} />
            <Show when={isPaused}>
              <span class="text-[9px] bg-warning/20 text-warning px-1 rounded">PAUSED</span>
            </Show>
          </div>

          {/* Actions */}
          <div class="flex items-center gap-0.5">
            <Show when={isActive}>
              <Show when={!isPaused}>
                <button
                  onClick={() => props.onPause?.(props.activity.id)}
                  class="p-0.5 hover:bg-bg-secondary rounded"
                  title="Pause operation"
                >
                  <HiOutlinePause class="w-3 h-3 text-txt-muted" />
                </button>
              </Show>
              <Show when={isPaused && props.onResume}>
                <button
                  onClick={() => props.onResume?.(props.activity.id)}
                  class="p-0.5 hover:bg-bg-secondary rounded"
                  title="Resume operation"
                >
                  <HiOutlinePlay class="w-3 h-3 text-accent" />
                </button>
              </Show>
              <Show when={props.onCancel}>
                <button
                  onClick={() => props.onCancel?.(props.activity.id)}
                  class="p-0.5 hover:bg-bg-secondary rounded"
                  title="Cancel operation"
                >
                  <HiOutlineXMark class="w-3 h-3 text-txt-muted" />
                </button>
              </Show>
            </Show>
            <Show when={isFinished}>
              <button
                onClick={() => handleOpenDestination(props.activity.destination)}
                class="p-0.5 hover:bg-bg-secondary rounded"
                title="Open destination folder"
              >
                <HiOutlineFolderOpen class="w-3 h-3 text-txt-muted" />
              </button>
              <Show when={props.onClear}>
                <button
                  onClick={() => props.onClear?.(props.activity.id)}
                  class="p-0.5 hover:bg-bg-secondary rounded"
                  title="Clear from list"
                >
                  <HiOutlineXMark class="w-3 h-3 text-txt-muted" />
                </button>
              </Show>
            </Show>
          </div>
        </div>
      </div>

      {/* Progress */}
      <Show when={props.activity.progress && props.activity.status === "running"}>
        <div class="space-y-1">
          {/* Overall Progress Bar */}
          <div class="w-full h-1 bg-bg-secondary rounded-full overflow-hidden">
            <div
              class="h-full bg-accent transition-all duration-300"
              style={{ width: `${props.activity.progress?.percent || 0}%` }}
            />
          </div>
          
          {/* Overall Stats */}
          <div class="flex items-center justify-between text-[10px] text-txt-muted leading-tight">
            <div class="flex items-center gap-1.5 flex-wrap">
              <span title="Bytes processed / Total">
                {formatBytes(props.activity.progress?.bytesProcessed || 0)} / {formatBytes(props.activity.progress?.totalBytes || 0)}
              </span>
              <Show when={pref('activityShowSpeed', true) && speed}>
                <span class={speedColor || "text-accent"} title="Transfer speed">• {speed}</span>
              </Show>
              <Show when={pref('activityShowFileCount', true) && props.activity.progress?.totalFiles}>
                <span title="Files processed / Total">
                  • {props.activity.progress?.currentFileIndex || 0}/{props.activity.progress?.totalFiles} files
                </span>
              </Show>
            </div>
            <div class="flex items-center gap-1.5">
              <span>{props.activity.progress?.percent.toFixed(1)}%</span>
              <Show when={pref('activityShowETA', true) && eta}>
                <span title="Estimated time remaining">• {eta}</span>
              </Show>
            </div>
          </div>
          
          {/* Current File Progress - Shown when preference enabled */}
          <Show when={pref('activityShowCurrentFile', true) && props.activity.progress?.currentFile}>
            <div class="border-t border-border/50 pt-1 space-y-0.5">
              {/* Current File Header */}
              <div class="flex items-center gap-1 px-1">
                <FileIcon class="w-3 h-3 text-accent flex-shrink-0" />
                <span 
                  class="text-[10px] text-txt truncate flex-1"
                  title={props.activity.progress?.currentFile}
                >
                  {getFileName(props.activity.progress!.currentFile!)}
                </span>
                <span class="text-[9px] text-txt-muted font-medium">
                  {currentFilePercent.toFixed(0)}%
                </span>
              </div>
              
              {/* Current File Progress Bar */}
              <div class="w-full h-1 bg-bg-secondary rounded-full overflow-hidden mx-1">
                <div
                  class="h-full bg-info transition-all duration-300"
                  style={{ width: `${currentFilePercent}%` }}
                />
              </div>
              
              {/* Current File Stats */}
              <div class="flex items-center justify-between text-[9px] text-txt-muted px-1">
                <span title="Current file processed / Total">
                  {formatBytes(currentFileBytes)} / {formatBytes(currentFileTotal)}
                </span>
                <Show when={currentFileTotal > 0}>
                  <span title="Remaining">
                    {formatBytes(currentFileTotal - currentFileBytes)} left
                  </span>
                </Show>
              </div>
            </div>
          </Show>
          
          {/* Expandable Detailed View */}
          <Show when={pref('activityShowCurrentFile', true) && props.activity.progress?.currentFile}>
            <div class="border-t border-border/50 pt-1">
              <button
                class="w-full flex items-center gap-1 hover:bg-bg-hover rounded px-1 py-0.5 transition-colors"
                onClick={() => props.onToggleExpanded()}
              >
                <Show when={props.isExpanded} fallback={<HiOutlineChevronRight class="w-3 h-3 text-txt-muted flex-shrink-0" />}>
                  <HiOutlineChevronDown class="w-3 h-3 text-txt-muted flex-shrink-0" />
                </Show>
                <span class="text-[9px] text-txt-muted">
                  {props.isExpanded ? "Hide" : "Show"} details
                </span>
              </button>
              
              {/* Expanded Details */}
              <Show when={props.isExpanded}>
                <div class="pl-5 space-y-1 text-[9px] text-txt-muted">
                  {/* File Metadata Grid (progress bar removed - shown above always) */}
                  <div class="grid grid-cols-2 gap-x-2 gap-y-0.5">
                    <div class="flex justify-between">
                      <span class="text-txt-muted">Size:</span>
                      <span class="text-txt font-medium">{formatBytes(currentFileTotal)}</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-txt-muted">Processed:</span>
                      <span class="text-txt font-medium">{formatBytes(currentFileBytes)}</span>
                    </div>
                    <Show when={speed}>
                      <div class="flex justify-between">
                        <span class="text-txt-muted">Speed:</span>
                        <span class={`font-medium ${speedColor || "text-accent"}`}>{speed}</span>
                      </div>
                    </Show>
                    <Show when={currentFileTotal > 0}>
                      <div class="flex justify-between">
                        <span class="text-txt-muted">Remaining:</span>
                        <span class="text-txt font-medium">
                          {formatBytes(currentFileTotal - currentFileBytes)}
                        </span>
                      </div>
                    </Show>
                  </div>
                  
                  {/* Full Path (truncated) */}
                  <div class="border-t border-border/30 pt-1">
                    <div class="text-txt-muted">Path:</div>
                    <div 
                      class="text-txt font-mono break-all leading-tight mt-0.5"
                      title={props.activity.progress?.currentFile}
                    >
                      {props.activity.progress?.currentFile}
                    </div>
                  </div>
                </div>
              </Show>
            </div>
          </Show>
        </div>
      </Show>

      <Show when={props.activity.error}>
        <div 
          class="text-[10px] text-error bg-error/10 rounded px-1.5 py-0.5 truncate"
          title={props.activity.error}
        >
          {props.activity.error}
        </div>
      </Show>

      {/* Metadata */}
      <div class="flex items-center justify-between text-[10px] text-txt-muted leading-tight">
        <div class="flex items-center gap-1.5 flex-wrap">
          <Show when={pref('activityShowFileCount', true) && props.activity.metadata?.itemCount}>
            <span title="Number of items">{props.activity.metadata!.itemCount} item{props.activity.metadata!.itemCount !== 1 ? "s" : ""}</span>
          </Show>
          <Show when={props.activity.metadata?.compressionLevel}>
            <span title="Compression level">• {props.activity.metadata!.compressionLevel}</span>
          </Show>
          <Show when={props.activity.metadata?.encrypted}>
            <span class="text-accent" title="AES-256 encrypted">• 🔒</span>
          </Show>
          <Show when={pref('activityShowCompressionRatio', true) && compressionRatio}>
            <span title="Compression ratio">• {compressionRatio}</span>
          </Show>
          <Show when={pref('activityShowThreadCount', true) && props.activity.metadata?.threadCount}>
            <span title="Thread count">• {props.activity.metadata!.threadCount} threads</span>
          </Show>
        </div>
        <Show when={duration !== null}>
          <span title="Duration">{formatDuration(duration!)}</span>
        </Show>
      </div>
    </div>
  );
};
