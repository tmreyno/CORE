// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For, Component, createMemo, createSignal } from "solid-js";
import { HiOutlineArchiveBox, HiOutlineCog6Tooth } from "./icons";
import type { ExportActivity } from "../types/exportActivity";
import { usePreferences } from "./preferences";
import { ActivityItem } from "./activity/ActivityItem";

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
