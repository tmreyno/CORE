// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Activity Panel - Displays project sessions, activity log, and timeline
 */

import { Component, For, Show, createSignal, createMemo } from "solid-js";
import { getBasename } from "../utils/pathUtils";
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
  HiOutlineArrowDownTray,
  HiOutlineArrowUp,
  HiOutlineArrowDown,
  HiOutlineBookmark,
  HiOutlinePencilSquare,
  HiOutlineMagnifyingGlass,
  HiOutlineXMark,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
} from "./icons";
import { save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "./Toast";
import { buildFullTimeline, timelineToCsv, timelineToJson } from "../utils/timelineExport";
import type { FFXProject, ActivityLogEntry, ProjectSession } from "../types/project";

interface ActivityPanelProps {
  project: FFXProject | null;
}

type ActivityFilter = "all" | "project" | "file" | "hash" | "export" | "search" | "bookmark" | "note";
type SortDirection = "newest" | "oldest";

/** Convert activity entries to CSV string */
export const activitiesToCsv = (entries: ActivityLogEntry[]): string => {
  const header = "Timestamp,Category,Action,Description,User,File Path";
  const rows = entries.map(e => {
    const escapeCsv = (val: string | undefined) => {
      if (!val) return "";
      if (val.includes(",") || val.includes('"') || val.includes("\n")) {
        return `"${val.replace(/"/g, '""')}"`;
      }
      return val;
    };
    return [
      e.timestamp,
      e.category,
      e.action,
      escapeCsv(e.description),
      escapeCsv(e.user),
      escapeCsv(e.file_path),
    ].join(",");
  });
  return [header, ...rows].join("\n");
};

/** Convert activity entries to JSON string */
export const activitiesToJson = (entries: ActivityLogEntry[]): string => {
  return JSON.stringify(entries, null, 2);
};

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
  const toast = useToast();
  const [filter, setFilter] = createSignal<ActivityFilter>("all");
  const [sortDirection, setSortDirection] = createSignal<SortDirection>("newest");
  const [searchQuery, setSearchQuery] = createSignal("");
  const [dateFrom, setDateFrom] = createSignal("");
  const [dateTo, setDateTo] = createSignal("");
  const [sessionsExpanded, setSessionsExpanded] = createSignal(true);
  const [activityExpanded, setActivityExpanded] = createSignal(true);
  const [showSearch, setShowSearch] = createSignal(false);
  const [exporting, setExporting] = createSignal(false);

  // Get sessions sorted by most recent
  const sessions = createMemo(() => {
    if (!props.project?.sessions) return [];
    return [...props.project.sessions].sort((a, b) => 
      new Date(b.started_at).getTime() - new Date(a.started_at).getTime()
    );
  });

  // Get filtered, searched, date-filtered, and sorted activity log
  const filteredActivities = createMemo(() => {
    if (!props.project?.activity_log) return [];
    let activities = [...props.project.activity_log];
    
    // Category filter
    if (filter() !== "all") {
      activities = activities.filter(entry => entry.category === filter());
    }

    // Text search
    const query = searchQuery().toLowerCase().trim();
    if (query) {
      activities = activities.filter(entry =>
        entry.description?.toLowerCase().includes(query) ||
        entry.action?.toLowerCase().includes(query) ||
        entry.user?.toLowerCase().includes(query) ||
        entry.file_path?.toLowerCase().includes(query)
      );
    }

    // Date range filter
    const from = dateFrom();
    if (from) {
      const fromDate = new Date(from);
      fromDate.setHours(0, 0, 0, 0);
      activities = activities.filter(entry => new Date(entry.timestamp) >= fromDate);
    }
    const to = dateTo();
    if (to) {
      const toDate = new Date(to);
      toDate.setHours(23, 59, 59, 999);
      activities = activities.filter(entry => new Date(entry.timestamp) <= toDate);
    }

    // Sort
    activities.sort((a, b) => {
      const diff = new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime();
      return sortDirection() === "newest" ? diff : -diff;
    });

    return activities;
  });

  // Current session ID
  const currentSessionId = createMemo(() => props.project?.current_session_id);

  // Stats
  const stats = createMemo(() => ({
    totalSessions: props.project?.sessions?.length || 0,
    totalActivities: props.project?.activity_log?.length || 0,
    users: new Set(props.project?.users?.map(u => u.username) || []).size,
  }));

  // Whether any filter is active
  const hasActiveFilters = createMemo(() =>
    filter() !== "all" || searchQuery() !== "" || dateFrom() !== "" || dateTo() !== ""
  );

  // Clear all filters
  const clearFilters = () => {
    setFilter("all");
    setSearchQuery("");
    setDateFrom("");
    setDateTo("");
  };

  // Export activities
  const handleExport = async (format: "csv" | "json") => {
    const entries = filteredActivities();
    if (entries.length === 0) return;

    setExporting(true);
    try {
      const ext = format === "csv" ? "csv" : "json";
      const path = await save({
        title: "Export Activity Log",
        defaultPath: `activity-log.${ext}`,
        filters: [
          { name: format.toUpperCase(), extensions: [ext] },
        ],
      });

      if (!path) return; // user cancelled

      const content = format === "csv"
        ? activitiesToCsv(entries)
        : activitiesToJson(entries);

      await invoke("write_text_file", { path, content });
      toast.success("Export Complete", `Activity log saved to ${getBasename(path) || path}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      toast.error("Export Failed", message);
    } finally {
      setExporting(false);
    }
  };

  // Export full forensic timeline (activity + sessions + bookmarks + notes)
  const handleFullTimelineExport = async (format: "csv" | "json") => {
    const project = props.project;
    if (!project) return;

    setExporting(true);
    try {
      const ext = format === "csv" ? "csv" : "json";
      const path = await save({
        title: "Export Full Forensic Timeline",
        defaultPath: `forensic-timeline.${ext}`,
        filters: [
          { name: format.toUpperCase(), extensions: [ext] },
        ],
      });

      if (!path) return;

      const timeline = buildFullTimeline(project, project.sessions?.[0]?.user || "examiner");
      const content = format === "csv"
        ? timelineToCsv(timeline)
        : timelineToJson(timeline);

      await invoke("write_text_file", { path, content });
      toast.success(
        "Timeline Exported",
        `${timeline.metadata.total_events} events saved to ${getBasename(path) || path}`,
      );
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      toast.error("Timeline Export Failed", message);
    } finally {
      setExporting(false);
    }
  };

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
          <div class="flex flex-col min-h-0 flex-1">
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
              {/* Toolbar: Filter + Sort + Search + Export */}
              <div class="flex items-center gap-1 px-2 py-1.5 border-b border-border/50 bg-bg-secondary/50">
                <HiOutlineFunnel class="w-3 h-3 text-txt-muted flex-shrink-0" />
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

                {/* Sort toggle */}
                <button
                  class="icon-btn-sm"
                  title={sortDirection() === "newest" ? "Newest first" : "Oldest first"}
                  onClick={() => setSortDirection(d => d === "newest" ? "oldest" : "newest")}
                >
                  {sortDirection() === "newest"
                    ? <HiOutlineArrowDown class="w-3.5 h-3.5" />
                    : <HiOutlineArrowUp class="w-3.5 h-3.5" />}
                </button>

                {/* Search toggle */}
                <button
                  class="icon-btn-sm"
                  classList={{ "text-accent": showSearch() }}
                  title="Search activities"
                  onClick={() => setShowSearch(s => !s)}
                >
                  <HiOutlineMagnifyingGlass class="w-3.5 h-3.5" />
                </button>

                {/* Export dropdown */}
                <div class="relative group">
                  <button
                    class="icon-btn-sm"
                    title="Export activity log"
                    disabled={exporting()}
                  >
                    <HiOutlineArrowDownTray class="w-3.5 h-3.5" />
                  </button>
                  <div class="absolute right-0 top-full mt-1 hidden group-hover:block z-dropdown">
                    <div class="bg-bg-secondary border border-border rounded-lg shadow-lg py-1 min-w-[140px]">
                      <div class="px-3 py-1 text-[10px] font-medium text-txt-muted uppercase tracking-wide">Activity Log</div>
                      <button
                        class="w-full text-left px-3 py-1.5 text-xs text-txt hover:bg-bg-hover disabled:opacity-40"
                        onClick={() => handleExport("csv")}
                        disabled={filteredActivities().length === 0}
                      >
                        Export CSV
                      </button>
                      <button
                        class="w-full text-left px-3 py-1.5 text-xs text-txt hover:bg-bg-hover disabled:opacity-40"
                        onClick={() => handleExport("json")}
                        disabled={filteredActivities().length === 0}
                      >
                        Export JSON
                      </button>
                      <div class="border-t border-border/40 my-1" />
                      <div class="px-3 py-1 text-[10px] font-medium text-txt-muted uppercase tracking-wide">Full Timeline</div>
                      <button
                        class="w-full text-left px-3 py-1.5 text-xs text-txt hover:bg-bg-hover"
                        onClick={() => handleFullTimelineExport("csv")}
                      >
                        Timeline CSV
                      </button>
                      <button
                        class="w-full text-left px-3 py-1.5 text-xs text-txt hover:bg-bg-hover"
                        onClick={() => handleFullTimelineExport("json")}
                      >
                        Timeline JSON
                      </button>
                    </div>
                  </div>
                </div>
              </div>

              {/* Search bar (collapsible) */}
              <Show when={showSearch()}>
                <div class="flex items-center gap-1 px-2 py-1 border-b border-border/50">
                  <HiOutlineMagnifyingGlass class="w-3 h-3 text-txt-muted flex-shrink-0" />
                  <input
                    type="text"
                    class="flex-1 text-xs bg-transparent text-txt border-none outline-none placeholder:text-txt-muted"
                    placeholder="Search descriptions, actions, users..."
                    value={searchQuery()}
                    onInput={(e) => setSearchQuery(e.target.value)}
                  />
                  <Show when={searchQuery()}>
                    <button
                      class="icon-btn-sm"
                      onClick={() => setSearchQuery("")}
                    >
                      <HiOutlineXMark class="w-3 h-3" />
                    </button>
                  </Show>
                </div>
              </Show>

              {/* Date range filter */}
              <Show when={showSearch()}>
                <div class="flex items-center gap-1 px-2 py-1 border-b border-border/50 text-xs">
                  <HiOutlineCalendar class="w-3 h-3 text-txt-muted flex-shrink-0" />
                  <input
                    type="date"
                    class="flex-1 text-xs bg-transparent text-txt border-none outline-none"
                    value={dateFrom()}
                    onInput={(e) => setDateFrom(e.target.value)}
                    placeholder="From"
                    title="From date"
                  />
                  <span class="text-txt-muted">–</span>
                  <input
                    type="date"
                    class="flex-1 text-xs bg-transparent text-txt border-none outline-none"
                    value={dateTo()}
                    onInput={(e) => setDateTo(e.target.value)}
                    placeholder="To"
                    title="To date"
                  />
                </div>
              </Show>

              {/* Active filter indicator */}
              <Show when={hasActiveFilters()}>
                <div class="flex items-center justify-between px-2 py-1 border-b border-border/50 bg-accent/5">
                  <span class="text-[10px] text-accent">
                    {filteredActivities().length} of {stats().totalActivities} events
                  </span>
                  <button
                    class="text-[10px] text-accent hover:text-accent-hover underline"
                    onClick={clearFilters}
                  >
                    Clear filters
                  </button>
                </div>
              </Show>

              {/* Activity List */}
              <div class="flex-1 overflow-y-auto">
                <Show when={filteredActivities().length === 0}>
                  <div class="px-3 py-8 text-center text-xs text-txt-muted">
                    <HiOutlineClock class="w-8 h-8 mx-auto mb-2 opacity-30" />
                    <Show when={hasActiveFilters()} fallback={
                      <>
                        <div>No activity recorded</div>
                        <div class="mt-1 opacity-75">Actions will appear here as you work</div>
                      </>
                    }>
                      <div>No matching activities</div>
                      <div class="mt-1 opacity-75">Try adjusting your filters</div>
                    </Show>
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
