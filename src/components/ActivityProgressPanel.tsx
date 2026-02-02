// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For, Component, createMemo, createSignal } from "solid-js";
import { HiOutlineArchiveBox, HiOutlineArrowUpTray, HiOutlineDocumentDuplicate, HiOutlineCheckCircle, HiOutlineXCircle, HiOutlineExclamationTriangle, HiOutlineClock, HiOutlineXMark, HiOutlineFolderOpen, HiOutlinePause, HiOutlinePlay, HiOutlineCog6Tooth, HiOutlineChevronDown, HiOutlineChevronRight, HiOutlineDocument, HiOutlineFolder } from "solid-icons/hi";
import type { ExportActivity } from "../types/exportActivity";
import { getActivityDuration, formatDuration } from "../types/exportActivity";
import { usePreferences, type AppPreferences } from "./preferences";
import { invoke } from "@tauri-apps/api/core";

interface ActivityProgressPanelProps {
  activities: ExportActivity[];
  onCancel?: (id: string) => void;
  onClear?: (id: string) => void;
  onPause?: (id: string) => void;
  onResume?: (id: string) => void;
  onOpenSettings?: () => void;
}

/**
 * Enhanced ActivityProgressPanel with preferences support
 */
export const ActivityProgressPanel: Component<ActivityProgressPanelProps> = (props) => {
  const { preferences } = usePreferences();
  
  // Track which activities are expanded (activity ID -> boolean)
  const [expandedActivities, setExpandedActivities] = createSignal<Set<string>>(new Set());
  
  const toggleExpanded = (activityId: string) => {
    setExpandedActivities(prev => {
      const next = new Set(prev);
      if (next.has(activityId)) {
        next.delete(activityId);
      } else {
        next.add(activityId);
      }
      return next;
    });
  };
  
  // Group and sort activities based on preferences
  const processedActivities = createMemo(() => {
    let activities = [...props.activities];
    
    // Sort
    switch (preferences().activitySortOrder) {
      case "newest":
        activities.sort((a, b) => b.startTime.getTime() - a.startTime.getTime());
        break;
      case "oldest":
        activities.sort((a, b) => a.startTime.getTime() - b.startTime.getTime());
        break;
      case "name":
        activities.sort((a, b) => a.destination.localeCompare(b.destination));
        break;
      case "progress":
        activities.sort((a, b) => (b.progress?.percent || 0) - (a.progress?.percent || 0));
        break;
    }
    
    // Limit visible count
    if (preferences().activityMaxVisible > 0) {
      activities = activities.slice(0, preferences().activityMaxVisible);
    }
    
    // Group
    if (preferences().activityGrouping === "status") {
      const running = activities.filter(a => a.status === "running" || a.status === "pending");
      const finished = activities.filter(a => a.status === "completed" || a.status === "failed" || a.status === "cancelled");
      return { grouped: true, groups: [
        { title: "Active", activities: running },
        { title: "Finished", activities: finished }
      ]};
    } else if (preferences().activityGrouping === "type") {
      const copy = activities.filter(a => a.type === "copy");
      const exportOps = activities.filter(a => a.type === "export");
      const archive = activities.filter(a => a.type === "archive");
      return { grouped: true, groups: [
        { title: "Archives", activities: archive },
        { title: "Exports", activities: exportOps },
        { title: "Copies", activities: copy }
      ]};
    }
    
    return { grouped: false, activities };
  });
  
  const getTypeIcon = (type: ExportActivity["type"]) => {
    switch (type) {
      case "copy": return HiOutlineDocumentDuplicate;
      case "export": return HiOutlineArrowUpTray;
      case "archive": return HiOutlineArchiveBox;
    }
  };

  const getStatusIcon = (status: ExportActivity["status"]) => {
    switch (status) {
      case "pending": return HiOutlineClock;
      case "running": return HiOutlineClock;
      case "completed": return HiOutlineCheckCircle;
      case "failed": return HiOutlineXCircle;
      case "cancelled": return HiOutlineExclamationTriangle;
    }
  };

  const getStatusColor = (status: ExportActivity["status"]) => {
    switch (status) {
      case "pending": return "text-txt-muted";
      case "running": return "text-accent";
      case "completed": return "text-success";
      case "failed": return "text-error";
      case "cancelled": return "text-warning";
    }
  };

  const getTypeName = (type: ExportActivity["type"]) => {
    switch (type) {
      case "copy": return "Copy";
      case "export": return "Export";
      case "archive": return "Archive";
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
  };
  
  const calculateSpeed = (activity: ExportActivity): string | null => {
    if (!activity.progress || activity.status !== "running" || activity.paused) return null;
    const elapsed = Date.now() - activity.startTime.getTime();
    if (elapsed < 1000) return null;
    const bytesPerMs = activity.progress.bytesProcessed / elapsed;
    const bytesPerSec = bytesPerMs * 1000;
    return `${formatBytes(bytesPerSec)}/s`;
  };
  
  const getSpeedColor = (activity: ExportActivity): string => {
    const colorCoded = preferences()?.activityColorCodedSpeed ?? true;
    if (!colorCoded || !activity.progress) return "";
    const elapsed = Date.now() - activity.startTime.getTime();
    if (elapsed < 1000) return "";
    const bytesPerMs = activity.progress.bytesProcessed / elapsed;
    const mbPerSec = (bytesPerMs * 1000) / (1024 * 1024);
    
    if (mbPerSec > 100) return "text-success";
    if (mbPerSec > 10) return "text-accent";
    return "text-warning";
  };
  
  const calculateETA = (activity: ExportActivity): string | null => {
    if (!activity.progress || activity.status !== "running" || activity.paused || activity.progress.totalBytes === 0) return null;
    const elapsed = Date.now() - activity.startTime.getTime();
    if (elapsed < 2000) return "Calculating...";
    const bytesPerMs = activity.progress.bytesProcessed / elapsed;
    if (bytesPerMs === 0) return null;
    const remaining = activity.progress.totalBytes - activity.progress.bytesProcessed;
    const msRemaining = remaining / bytesPerMs;
    
    if (msRemaining < 60000) {
      return `${Math.ceil(msRemaining / 1000)}s`;
    } else if (msRemaining < 3600000) {
      return `${Math.ceil(msRemaining / 60000)}m`;
    } else {
      const hours = Math.floor(msRemaining / 3600000);
      const minutes = Math.ceil((msRemaining % 3600000) / 60000);
      return `${hours}h ${minutes}m`;
    }
  };
  
  const formatCompressionRatio = (activity: ExportActivity): string | null => {
    if (!activity.metadata?.originalSize || !activity.metadata?.finalSize) return null;
    const ratio = 1 - (activity.metadata.finalSize / activity.metadata.originalSize);
    return `${(ratio * 100).toFixed(1)}% reduction`;
  };
  
  const handleOpenDestination = async (path: string) => {
    try {
      // Open the parent directory using the opener plugin
      const dir = path.substring(0, path.lastIndexOf('/'));
      await invoke('plugin:opener|open_path', { path: dir });
    } catch (error) {
      console.error('Failed to open destination:', error);
    }
  };

  const getFileName = (path: string): string => {
    return path.split("/").pop() || path;
  };

  // Separate component for each activity item - makes preferences reactive
  const ActivityItem: Component<{
    activity: ExportActivity;
    isExpanded: boolean;
    onToggleExpanded: () => void;
    onPause?: (id: string) => void;
    onResume?: (id: string) => void;
    onCancel?: (id: string) => void;
    onClear?: (id: string) => void;
  }> = (itemProps) => {
    const { preferences } = usePreferences();
    
    // Helper to get preference with fallback - THIS IS REACTIVE
    const pref = <K extends keyof AppPreferences>(key: K, fallback: AppPreferences[K]): AppPreferences[K] => {
      return preferences()?.[key] ?? fallback;
    };
    
    const TypeIcon = getTypeIcon(itemProps.activity.type);
    const StatusIcon = getStatusIcon(itemProps.activity.status);
    const statusColor = getStatusColor(itemProps.activity.status);
    const duration = getActivityDuration(itemProps.activity);
    const speed = calculateSpeed(itemProps.activity);
    const speedColor = getSpeedColor(itemProps.activity);
    const eta = calculateETA(itemProps.activity);
    const compressionRatio = formatCompressionRatio(itemProps.activity);
    const isActive = itemProps.activity.status === "pending" || itemProps.activity.status === "running";
    const isFinished = itemProps.activity.status === "completed" || itemProps.activity.status === "failed" || itemProps.activity.status === "cancelled";
    const isPaused = itemProps.activity.paused === true;
    
    // Calculate current file progress
    const currentFileTotal = itemProps.activity.progress?.currentFileTotal || 0;
    const currentFileBytes = itemProps.activity.progress?.currentFileBytes || 0;
    const currentFilePercent = currentFileTotal > 0 ? (currentFileBytes / currentFileTotal) * 100 : 0;
    
    const currentFile = itemProps.activity.progress?.currentFile || "";
    const isFolder = currentFile && !currentFile.includes('.');
    const FileIcon = isFolder ? HiOutlineFolder : HiOutlineDocument;

    return (
      <div 
        class="card p-2.5 space-y-1.5 hover:bg-bg-hover transition-colors"
        classList={{
          "animate-pulse-slow": pref('activityPulseAnimation', true) && isActive && !isPaused
        }}
        title={`${getTypeName(itemProps.activity.type)}: ${itemProps.activity.destination}\nSources: ${itemProps.activity.sources.length} items`}
      >
        {/* Header */}
        <div class="flex items-center justify-between gap-2">
          <div class="flex items-center gap-1.5 flex-1 min-w-0">
            <TypeIcon class="w-3.5 h-3.5 text-txt-muted flex-shrink-0" />
            <span class="text-xs font-medium text-txt truncate">
              {getFileName(itemProps.activity.destination)}
            </span>
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
                  onClick={() => itemProps.onPause?.(itemProps.activity.id)}
                  class="p-0.5 hover:bg-bg-secondary rounded"
                  title="Pause operation"
                >
                  <HiOutlinePause class="w-3 h-3 text-txt-muted" />
                </button>
              </Show>
              <Show when={isPaused && itemProps.onResume}>
                <button
                  onClick={() => itemProps.onResume?.(itemProps.activity.id)}
                  class="p-0.5 hover:bg-bg-secondary rounded"
                  title="Resume operation"
                >
                  <HiOutlinePlay class="w-3 h-3 text-accent" />
                </button>
              </Show>
              <Show when={itemProps.onCancel}>
                <button
                  onClick={() => itemProps.onCancel?.(itemProps.activity.id)}
                  class="p-0.5 hover:bg-bg-secondary rounded"
                  title="Cancel operation"
                >
                  <HiOutlineXMark class="w-3 h-3 text-txt-muted" />
                </button>
              </Show>
            </Show>
            <Show when={isFinished}>
              <button
                onClick={() => handleOpenDestination(itemProps.activity.destination)}
                class="p-0.5 hover:bg-bg-secondary rounded"
                title="Open destination folder"
              >
                <HiOutlineFolderOpen class="w-3 h-3 text-txt-muted" />
              </button>
              <Show when={itemProps.onClear}>
                <button
                  onClick={() => itemProps.onClear?.(itemProps.activity.id)}
                  class="p-0.5 hover:bg-bg-secondary rounded"
                  title="Clear from list"
                >
                  <HiOutlineXMark class="w-3 h-3 text-txt-muted" />
                </button>
              </Show>
            </Show>
          </div>
        </div>

        {/* Progress */}
        <Show when={itemProps.activity.progress && itemProps.activity.status === "running"}>
          <div class="space-y-1">
            {/* Overall Progress Bar */}
            <div class="w-full h-1 bg-bg-secondary rounded-full overflow-hidden">
              <div
                class="h-full bg-accent transition-all duration-300"
                style={{ width: `${itemProps.activity.progress?.percent || 0}%` }}
              />
            </div>
            
            {/* Overall Stats */}
            <div class="flex items-center justify-between text-[10px] text-txt-muted leading-tight">
              <div class="flex items-center gap-1.5 flex-wrap">
                <span title="Bytes processed / Total">
                  {formatBytes(itemProps.activity.progress?.bytesProcessed || 0)} / {formatBytes(itemProps.activity.progress?.totalBytes || 0)}
                </span>
                <Show when={pref('activityShowSpeed', true) && speed}>
                  <span class={speedColor || "text-accent"} title="Transfer speed">• {speed}</span>
                </Show>
                <Show when={pref('activityShowFileCount', true) && itemProps.activity.progress?.totalFiles}>
                  <span title="Files processed / Total">
                    • {itemProps.activity.progress?.currentFileIndex || 0}/{itemProps.activity.progress?.totalFiles} files
                  </span>
                </Show>
              </div>
              <div class="flex items-center gap-1.5">
                <span>{itemProps.activity.progress?.percent.toFixed(1)}%</span>
                <Show when={pref('activityShowETA', true) && eta}>
                  <span title="Estimated time remaining">• {eta}</span>
                </Show>
              </div>
            </div>
            
            {/* Current File Progress - Shown when preference enabled */}
            <Show when={pref('activityShowCurrentFile', true) && itemProps.activity.progress?.currentFile}>
              <div class="border-t border-border/50 pt-1 space-y-0.5">
                {/* Current File Header */}
                <div class="flex items-center gap-1 px-1">
                  <FileIcon class="w-3 h-3 text-accent flex-shrink-0" />
                  <span 
                    class="text-[10px] text-txt truncate flex-1"
                    title={itemProps.activity.progress?.currentFile}
                  >
                    {getFileName(itemProps.activity.progress!.currentFile!)}
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
            <Show when={pref('activityShowCurrentFile', true) && itemProps.activity.progress?.currentFile}>
              <div class="border-t border-border/50 pt-1">
                <button
                  class="w-full flex items-center gap-1 hover:bg-bg-hover rounded px-1 py-0.5 transition-colors"
                  onClick={() => itemProps.onToggleExpanded()}
                >
                  <Show when={itemProps.isExpanded} fallback={<HiOutlineChevronRight class="w-3 h-3 text-txt-muted flex-shrink-0" />}>
                    <HiOutlineChevronDown class="w-3 h-3 text-txt-muted flex-shrink-0" />
                  </Show>
                  <span class="text-[9px] text-txt-muted">
                    {itemProps.isExpanded ? "Hide" : "Show"} details
                  </span>
                </button>
                
                {/* Expanded Details */}
                <Show when={itemProps.isExpanded}>
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
                        title={itemProps.activity.progress?.currentFile}
                      >
                        {itemProps.activity.progress?.currentFile}
                      </div>
                    </div>
                  </div>
                </Show>
              </div>
            </Show>
          </div>
        </Show>

        <Show when={itemProps.activity.error}>
          <div 
            class="text-[10px] text-error bg-error/10 rounded px-1.5 py-0.5 truncate"
            title={itemProps.activity.error}
          >
            {itemProps.activity.error}
          </div>
        </Show>

        {/* Metadata */}
        <div class="flex items-center justify-between text-[10px] text-txt-muted leading-tight">
          <div class="flex items-center gap-1.5 flex-wrap">
            <Show when={pref('activityShowFileCount', true) && itemProps.activity.metadata?.itemCount}>
              <span title="Number of items">{itemProps.activity.metadata!.itemCount} item{itemProps.activity.metadata!.itemCount !== 1 ? "s" : ""}</span>
            </Show>
            <Show when={itemProps.activity.metadata?.compressionLevel}>
              <span title="Compression level">• {itemProps.activity.metadata!.compressionLevel}</span>
            </Show>
            <Show when={itemProps.activity.metadata?.encrypted}>
              <span class="text-accent" title="AES-256 encrypted">• 🔒</span>
            </Show>
            <Show when={pref('activityShowCompressionRatio', true) && compressionRatio}>
              <span title="Compression ratio">• {compressionRatio}</span>
            </Show>
            <Show when={pref('activityShowThreadCount', true) && itemProps.activity.metadata?.threadCount}>
              <span title="Thread count">• {itemProps.activity.metadata!.threadCount} threads</span>
            </Show>
          </div>
          <Show when={duration !== null}>
            <span title="Duration">{formatDuration(duration!)}</span>
          </Show>
        </div>
      </div>
    );
  };

  return (
    <div class="flex flex-col h-full bg-bg">
      <div class="panel-header flex items-center justify-between">
        <h3 class="font-semibold text-txt">Export Activity</h3>
        <Show when={props.onOpenSettings}>
          <button
            onClick={() => props.onOpenSettings?.()}
            class="icon-btn-sm"
            title="Activity Display Settings"
          >
            <HiOutlineCog6Tooth class="w-4 h-4" />
          </button>
        </Show>
      </div>

      <div class="flex-1 overflow-y-auto">
        <Show
          when={props.activities.length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center h-full text-txt-muted p-4 text-center">
              <HiOutlineArchiveBox class="w-12 h-12 mb-2 opacity-50" />
              <p class="text-sm">No export activities</p>
              <p class="text-xs mt-1">Export operations will appear here</p>
            </div>
          }
        >
          <div class="flex flex-col gap-2 p-2">
            <Show
              when={processedActivities().grouped}
              fallback={
                <For each={processedActivities().activities}>
                  {(activity) => (
                    <ActivityItem 
                      activity={activity}
                      isExpanded={expandedActivities().has(activity.id)}
                      onToggleExpanded={() => toggleExpanded(activity.id)}
                      onPause={props.onPause}
                      onResume={props.onResume}
                      onCancel={props.onCancel}
                      onClear={props.onClear}
                    />
                  )}
                </For>
              }
            >
              <For each={processedActivities().groups}>
                {(group) => (
                  <Show when={group.activities.length > 0}>
                    <div class="space-y-2">
                      <div class="text-[10px] font-semibold text-txt-muted uppercase tracking-wider px-1">
                        {group.title}
                      </div>
                      <For each={group.activities}>
                        {(activity) => (
                          <ActivityItem 
                            activity={activity}
                            isExpanded={expandedActivities().has(activity.id)}
                            onToggleExpanded={() => toggleExpanded(activity.id)}
                            onPause={props.onPause}
                            onResume={props.onResume}
                            onCancel={props.onCancel}
                            onClear={props.onClear}
                          />
                        )}
                      </For>
                    </div>
                  </Show>
                )}
              </For>
            </Show>
          </div>
        </Show>
      </div>
    </div>
  );
};
