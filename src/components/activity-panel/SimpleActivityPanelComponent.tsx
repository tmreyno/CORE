// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For, Component, createMemo } from "solid-js";
import { getDirname } from "../../utils/pathUtils";
import { invoke } from "@tauri-apps/api/core";
import { logger } from "../../utils/logger";
import { ActivityCard } from "./ActivityCard";
import type { SimpleActivityPanelProps } from "./types";

const log = logger.scope("SimpleActivityPanel");

/**
 * Simplified activity panel - shows real progress from the library
 */
export const SimpleActivityPanelComponent: Component<SimpleActivityPanelProps> =
  (props) => {
    // Sort: running first, then pending, then completed
    const sortedActivities = createMemo(() => {
      return [...props.activities].sort((a, b) => {
        const order = {
          running: 0,
          paused: 1,
          pending: 2,
          completed: 3,
          failed: 4,
          cancelled: 5,
        };
        return order[a.status] - order[b.status];
      });
    });

    const activeCount = createMemo(
      () =>
        props.activities.filter(
          (a) =>
            a.status === "running" ||
            a.status === "pending" ||
            a.status === "paused",
        ).length,
    );

    const handleOpenDestination = async (path: string) => {
      try {
        const dir = getDirname(path) || path;
        await invoke("plugin:opener|open_path", { path: dir });
      } catch (error) {
        log.error("Failed to open destination:", error);
      }
    };

    return (
      <div class="flex flex-col h-full bg-bg">
        {/* Header */}
        <div class="flex items-center justify-between px-3 py-2 border-b border-border bg-bg-secondary">
          <span class="text-xs font-medium text-txt">
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
