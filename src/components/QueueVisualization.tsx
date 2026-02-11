// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For, createSignal, onMount, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { createVirtualizer } from "@tanstack/solid-virtual";
import { logger } from "../utils/logger";
const log = logger.scope("QueueVisualization");

// Types matching Rust backend
interface QueueStats {
  totalItems: number;
  completedItems: number;
  activeItems: number;
  throughputMbps: number;
  estimatedSecondsRemaining: number | null;
}

interface QueueItem {
  filePath: string;
  priority: number;
  sizeBytes: number;
  status: "Pending" | "InProgress" | "Completed" | "Failed";
  progress: number;
  errorMessage: string | null;
}

const priorityLabels: Record<number, string> = {
  0: "Low",
  1: "Medium",
  2: "High",
  3: "Critical",
};

const priorityColors: Record<number, string> = {
  0: "text-txt-muted",
  1: "text-info",
  2: "text-warning",
  3: "text-error",
};

const statusColors: Record<string, string> = {
  Pending: "text-txt-muted",
  InProgress: "text-accent",
  Completed: "text-success",
  Failed: "text-error",
};

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

function formatTime(seconds: number | null): string {
  if (seconds === null) return "Calculating...";
  if (seconds < 60) return `${Math.round(seconds)}s`;
  if (seconds < 3600) return `${Math.round(seconds / 60)}m`;
  return `${Math.round(seconds / 3600)}h`;
}

export const QueueVisualization: Component = () => {
  const [stats, setStats] = createSignal<QueueStats | null>(null);
  const [items, setItems] = createSignal<QueueItem[]>([]);
  const [isPaused, setIsPaused] = createSignal(false);
  
  let parentRef: HTMLDivElement | undefined;
  let unlistenStats: UnlistenFn | undefined;
  let unlistenItems: UnlistenFn | undefined;

  onMount(async () => {
    // Listen for queue stats updates
    unlistenStats = await listen<QueueStats>("hash-queue-stats", (event) => {
      setStats(event.payload);
    });

    // Listen for queue items updates
    unlistenItems = await listen<QueueItem[]>("hash-queue-items", (event) => {
      setItems(event.payload);
    });

    // Request initial state
    refreshQueue();
  });

  onCleanup(() => {
    unlistenStats?.();
    unlistenItems?.();
  });

  const refreshQueue = async () => {
    try {
      const currentStats = await invoke<QueueStats>("hash_queue_get_stats");
      setStats(currentStats);
      const currentItems = await invoke<QueueItem[]>("hash_queue_get_items");
      setItems(currentItems);
    } catch (error) {
      log.error("Failed to refresh queue:", error);
    }
  };

  const togglePause = async () => {
    try {
      if (isPaused()) {
        await invoke("hash_queue_resume");
        setIsPaused(false);
      } else {
        await invoke("hash_queue_pause");
        setIsPaused(true);
      }
    } catch (error) {
      log.error("Failed to toggle pause:", error);
    }
  };

  const clearCompleted = async () => {
    try {
      await invoke("hash_queue_clear_completed");
      await refreshQueue();
    } catch (error) {
      log.error("Failed to clear completed:", error);
    }
  };

  // Virtual scrolling for large queues
  const virtualizer = () => {
    if (!parentRef) return null;
    return createVirtualizer({
      count: items().length,
      getScrollElement: () => parentRef,
      estimateSize: () => 80,
      overscan: 5,
    });
  };

  return (
    <div class="card flex flex-col gap-base h-full">
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-semibold text-txt">Hash Queue</h3>
        <div class="flex gap-2">
          <button
            onClick={togglePause}
            class="btn-sm"
          >
            {isPaused() ? "Resume" : "Pause"}
          </button>
          <button
            onClick={clearCompleted}
            class="btn-sm"
          >
            Clear Completed
          </button>
          <button
            onClick={refreshQueue}
            class="btn-sm"
          >
            Refresh
          </button>
        </div>
      </div>

      <Show when={stats()}>
        {(s) => (
          <div class="grid grid-cols-5 gap-4">
            <div class="stat-box">
              <div class="text-txt-muted text-xs">Total</div>
              <div class="text-xl font-semibold text-txt">{s().totalItems}</div>
            </div>
            <div class="stat-box">
              <div class="text-txt-muted text-xs">Completed</div>
              <div class="text-xl font-semibold text-success">{s().completedItems}</div>
            </div>
            <div class="stat-box">
              <div class="text-txt-muted text-xs">Active</div>
              <div class="text-xl font-semibold text-accent">{s().activeItems}</div>
            </div>
            <div class="stat-box">
              <div class="text-txt-muted text-xs">Throughput</div>
              <div class="text-xl font-semibold text-txt">{s().throughputMbps.toFixed(1)} MB/s</div>
            </div>
            <div class="stat-box">
              <div class="text-txt-muted text-xs">ETA</div>
              <div class="text-xl font-semibold text-txt">{formatTime(s().estimatedSecondsRemaining)}</div>
            </div>
          </div>
        )}
      </Show>

      <div class="border-t border-border pt-4">
        <h4 class="text-sm font-medium text-txt mb-2">Queue Items ({items().length})</h4>
        <Show
          when={items().length > 0}
          fallback={<div class="text-txt-muted text-sm">No items in queue</div>}
        >
          <div
            ref={parentRef}
            class="overflow-auto h-96 bg-bg rounded-md border border-border"
          >
            <Show when={virtualizer()}>
              {(v) => (
                <div
                  style={{
                    height: `${v().getTotalSize()}px`,
                    width: "100%",
                    position: "relative",
                  }}
                >
                  <For each={v().getVirtualItems()}>
                    {(virtualRow) => {
                      const item = items()[virtualRow.index];
                      return (
                        <div
                          style={{
                            position: "absolute",
                            top: 0,
                            left: 0,
                            width: "100%",
                            height: `${virtualRow.size}px`,
                            transform: `translateY(${virtualRow.start}px)`,
                          }}
                          class="px-3 py-2 border-b border-border"
                        >
                          <div class="flex items-center justify-between mb-1">
                            <div class="flex items-center gap-2 flex-1 min-w-0">
                              <span
                                class={`text-xs font-medium px-2 py-0.5 rounded ${priorityColors[item.priority]}`}
                              >
                                {priorityLabels[item.priority]}
                              </span>
                              <span class="text-sm text-txt truncate">
                                {item.filePath.split("/").pop()}
                              </span>
                            </div>
                            <div class="flex items-center gap-4">
                              <span class="text-xs text-txt-muted">{formatBytes(item.sizeBytes)}</span>
                              <span class={`text-xs font-medium ${statusColors[item.status]}`}>
                                {item.status}
                              </span>
                            </div>
                          </div>
                          <Show when={item.status === "InProgress"}>
                            <div class="w-full bg-bg-secondary rounded-full h-1.5">
                              <div
                                class="bg-accent h-1.5 rounded-full transition-all duration-300"
                                style={{ width: `${item.progress * 100}%` }}
                              />
                            </div>
                          </Show>
                          <Show when={item.errorMessage}>
                            <div class="text-xs text-error mt-1">{item.errorMessage}</div>
                          </Show>
                        </div>
                      );
                    }}
                  </For>
                </div>
              )}
            </Show>
          </div>
        </Show>
      </div>
    </div>
  );
};
