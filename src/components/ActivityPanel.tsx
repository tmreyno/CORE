// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Activity Panel - Displays project sessions, activity log, and timeline
 */

import { Component, For, Show, createSignal, createMemo } from "solid-js";
import {
  HiOutlineClock,
  HiOutlineUser,
  HiOutlineCalendar,
  HiOutlineFunnel,
  HiOutlineArrowPath,
  HiOutlineDocumentText,
  HiOutlineFolder,
  HiOutlineFingerPrint,
  HiOutlineArrowUpTray,
  HiOutlineBookmark,
  HiOutlinePencilSquare,
  HiOutlineMagnifyingGlass,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
} from "./icons";
import type { FFXProject, ActivityLogEntry, ProjectSession } from "../types/project";

interface ActivityPanelProps {
  project: FFXProject | null;
}

type ActivityFilter = "all" | "project" | "file" | "hash" | "export" | "search" | "bookmark" | "note";

/** Get icon for activity category */
const getCategoryIcon = (category: string) => {
  switch (category) {
    case "project":
      return <HiOutlineFolder class="w-3.5 h-3.5" />;
    case "file":
      return <HiOutlineDocumentText class="w-3.5 h-3.5" />;
    case "hash":
      return <HiOutlineFingerPrint class="w-3.5 h-3.5" />;
    case "export":
      return <HiOutlineArrowUpTray class="w-3.5 h-3.5" />;
    case "search":
      return <HiOutlineMagnifyingGlass class="w-3.5 h-3.5" />;
    case "bookmark":
      return <HiOutlineBookmark class="w-3.5 h-3.5" />;
    case "note":
      return <HiOutlinePencilSquare class="w-3.5 h-3.5" />;
    default:
      return <HiOutlineClock class="w-3.5 h-3.5" />;
  }
};

/** Format duration in seconds to human readable */
const formatDuration = (seconds: number | null | undefined): string => {
  if (!seconds) return "—";
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  return `${hours}h ${mins}m`;
};

/** Format timestamp to relative or absolute */
const formatTimestamp = (timestamp: string, relative = true): string => {
  const date = new Date(timestamp);
  if (relative) {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);
    
    if (diffMins < 1) return "just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
  }
  return date.toLocaleDateString() + " " + date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
};

/** Session item component */
const SessionItem: Component<{ session: ProjectSession; isActive?: boolean }> = (props) => {
  return (
    <div class={`px-3 py-2 border-b border-border/50 ${props.isActive ? "bg-accent/10" : "hover:bg-bg-hover"}`}>
      <div class="flex items-center gap-2 text-sm">
        <HiOutlineUser class="w-3.5 h-3.5 text-txt-muted" />
        <span class="text-txt font-medium">{props.session.user}</span>
        <Show when={props.isActive}>
          <span class="text-[10px] px-1.5 py-0.5 bg-success/20 text-success rounded">ACTIVE</span>
        </Show>
      </div>
      <div class="flex items-center gap-3 mt-1 text-xs text-txt-muted">
        <span class="flex items-center gap-1">
          <HiOutlineCalendar class="w-3 h-3" />
          {formatTimestamp(props.session.started_at, false)}
        </span>
        <span class="flex items-center gap-1">
          <HiOutlineClock class="w-3 h-3" />
          {props.session.ended_at 
            ? formatDuration(props.session.duration_seconds) 
            : "In progress"}
        </span>
      </div>
      <Show when={props.session.app_version}>
        <div class="text-[10px] text-txt-muted mt-1">
          v{props.session.app_version}
        </div>
      </Show>
    </div>
  );
};

/** Activity log item component */
const ActivityItem: Component<{ entry: ActivityLogEntry }> = (props) => {
  return (
    <div class="px-3 py-2 border-b border-border/30 hover:bg-bg-hover group">
      <div class="flex items-start gap-2">
        <div class="mt-0.5 text-txt-muted">
          {getCategoryIcon(props.entry.category)}
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-sm text-txt truncate">{props.entry.description}</div>
          <div class="flex items-center gap-2 mt-0.5 text-[10px] text-txt-muted">
            <span>{formatTimestamp(props.entry.timestamp)}</span>
            <span class="opacity-50">•</span>
            <span class="capitalize">{props.entry.action}</span>
            <Show when={props.entry.user}>
              <span class="opacity-50">•</span>
              <span>{props.entry.user}</span>
            </Show>
          </div>
        </div>
      </div>
    </div>
  );
};

export const ActivityPanel: Component<ActivityPanelProps> = (props) => {
  const [filter, setFilter] = createSignal<ActivityFilter>("all");
  const [sessionsExpanded, setSessionsExpanded] = createSignal(true);
  const [activityExpanded, setActivityExpanded] = createSignal(true);

  // Get sessions sorted by most recent
  const sessions = createMemo(() => {
    if (!props.project?.sessions) return [];
    return [...props.project.sessions].sort((a, b) => 
      new Date(b.started_at).getTime() - new Date(a.started_at).getTime()
    );
  });

  // Get filtered activity log
  const filteredActivities = createMemo(() => {
    if (!props.project?.activity_log) return [];
    const activities = [...props.project.activity_log];
    
    if (filter() === "all") {
      return activities;
    }
    
    return activities.filter(entry => entry.category === filter());
  });

  // Current session ID
  const currentSessionId = createMemo(() => props.project?.current_session_id);

  // Stats
  const stats = createMemo(() => ({
    totalSessions: props.project?.sessions?.length || 0,
    totalActivities: props.project?.activity_log?.length || 0,
    users: new Set(props.project?.users?.map(u => u.username) || []).size,
  }));

  return (
    <div class="flex flex-col h-full bg-bg">
      {/* Header */}
      <div class="flex items-center justify-between px-3 py-2 border-b border-border bg-bg-secondary">
        <div class="flex items-center gap-2">
          <HiOutlineClock class="w-4 h-4 text-accent" />
          <span class="text-sm font-medium text-txt">Activity</span>
        </div>
        <Show when={props.project}>
          <div class="text-xs text-txt-muted">
            {stats().totalActivities} events
          </div>
        </Show>
      </div>

      <Show when={!props.project} fallback={
        <div class="flex-1 overflow-y-auto">
          {/* Sessions Section */}
          <div class="border-b border-border">
            <button
              class="w-full flex items-center justify-between px-3 py-2 hover:bg-bg-hover"
              onClick={() => setSessionsExpanded(!sessionsExpanded())}
            >
              <div class="flex items-center gap-2">
                {sessionsExpanded() ? <HiOutlineChevronDown class="w-3.5 h-3.5" /> : <HiOutlineChevronRight class="w-3.5 h-3.5" />}
                <HiOutlineUser class="w-4 h-4 text-txt-muted" />
                <span class="text-sm font-medium text-txt">Sessions</span>
              </div>
              <span class="text-xs text-txt-muted">{stats().totalSessions}</span>
            </button>
            
            <Show when={sessionsExpanded()}>
              <div class="max-h-48 overflow-y-auto">
                <Show when={sessions().length === 0}>
                  <div class="px-3 py-4 text-center text-xs text-txt-muted">
                    No sessions recorded
                  </div>
                </Show>
                <For each={sessions()}>
                  {(session) => (
                    <SessionItem 
                      session={session} 
                      isActive={session.session_id === currentSessionId()}
                    />
                  )}
                </For>
              </div>
            </Show>
          </div>

          {/* Activity Log Section */}
          <div>
            <button
              class="w-full flex items-center justify-between px-3 py-2 hover:bg-bg-hover border-b border-border"
              onClick={() => setActivityExpanded(!activityExpanded())}
            >
              <div class="flex items-center gap-2">
                {activityExpanded() ? <HiOutlineChevronDown class="w-3.5 h-3.5" /> : <HiOutlineChevronRight class="w-3.5 h-3.5" />}
                <HiOutlineArrowPath class="w-4 h-4 text-txt-muted" />
                <span class="text-sm font-medium text-txt">Activity Log</span>
              </div>
              <span class="text-xs text-txt-muted">{filteredActivities().length}</span>
            </button>

            <Show when={activityExpanded()}>
              {/* Filter */}
              <div class="flex items-center gap-1 px-2 py-1.5 border-b border-border/50 bg-bg-secondary/50">
                <HiOutlineFunnel class="w-3 h-3 text-txt-muted" />
                <select
                  class="flex-1 text-xs bg-transparent text-txt border-none outline-none cursor-pointer"
                  value={filter()}
                  onChange={(e) => setFilter(e.target.value as ActivityFilter)}
                >
                  <option value="all">All Activities</option>
                  <option value="project">Project</option>
                  <option value="file">Files</option>
                  <option value="hash">Hashing</option>
                  <option value="export">Export</option>
                  <option value="search">Search</option>
                  <option value="bookmark">Bookmarks</option>
                  <option value="note">Notes</option>
                </select>
              </div>

              {/* Activity List */}
              <div class="flex-1 overflow-y-auto">
                <Show when={filteredActivities().length === 0}>
                  <div class="px-3 py-8 text-center text-xs text-txt-muted">
                    <HiOutlineClock class="w-8 h-8 mx-auto mb-2 opacity-30" />
                    <div>No activity recorded</div>
                    <div class="mt-1 opacity-75">Actions will appear here as you work</div>
                  </div>
                </Show>
                <For each={filteredActivities()}>
                  {(entry) => <ActivityItem entry={entry} />}
                </For>
              </div>
            </Show>
          </div>
        </div>
      }>
        {/* No project loaded */}
        <div class="flex-1 flex flex-col items-center justify-center p-4 text-center">
          <HiOutlineClock class="w-12 h-12 text-txt-muted/30 mb-3" />
          <div class="text-sm text-txt-muted">No project loaded</div>
          <div class="text-xs text-txt-muted/75 mt-1">
            Load or create a project to view activity
          </div>
        </div>
      </Show>
    </div>
  );
};
