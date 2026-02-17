// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * timelineExport - Comprehensive forensic timeline export utility.
 *
 * Merges activity log, sessions, bookmarks, notes, and hash events into a
 * unified chronological timeline exportable as CSV or JSON.
 */

import type { FFXProject, ActivityLogEntry, ProjectSession, ProjectBookmark, ProjectNote } from "../types/project";

// =============================================================================
// Types
// =============================================================================

export interface TimelineEvent {
  /** ISO timestamp */
  timestamp: string;
  /** Event category */
  category: "activity" | "session_start" | "session_end" | "bookmark" | "note" | "hash" | "evidence";
  /** Event action or type */
  action: string;
  /** Human-readable description */
  description: string;
  /** User who performed the action */
  user: string;
  /** Related file path (if applicable) */
  file_path: string;
  /** Additional details serialized as JSON string */
  details: string;
}

export interface FullTimelineExport {
  metadata: {
    project_name: string;
    export_date: string;
    exported_by: string;
    total_events: number;
    date_range: [string, string] | null;
    format_version: string;
  };
  events: TimelineEvent[];
}

// =============================================================================
// Event Builders
// =============================================================================

function activityToEvent(entry: ActivityLogEntry): TimelineEvent {
  return {
    timestamp: entry.timestamp,
    category: "activity",
    action: entry.action,
    description: entry.description,
    user: entry.user,
    file_path: entry.file_path || "",
    details: entry.details ? JSON.stringify(entry.details) : "",
  };
}

function sessionStartEvent(session: ProjectSession): TimelineEvent {
  return {
    timestamp: session.started_at,
    category: "session_start",
    action: "session_started",
    description: `Session started${session.hostname ? ` on ${session.hostname}` : ""} (${session.app_version})`,
    user: session.user,
    file_path: "",
    details: JSON.stringify({
      session_id: session.session_id,
      hostname: session.hostname,
      app_version: session.app_version,
    }),
  };
}

function sessionEndEvent(session: ProjectSession): TimelineEvent | null {
  if (!session.ended_at) return null;
  const durationStr = session.duration_seconds
    ? ` (${Math.floor(session.duration_seconds / 60)}m ${session.duration_seconds % 60}s)`
    : "";
  return {
    timestamp: session.ended_at,
    category: "session_end",
    action: "session_ended",
    description: `Session ended${durationStr}${session.summary ? `: ${session.summary}` : ""}`,
    user: session.user,
    file_path: "",
    details: JSON.stringify({
      session_id: session.session_id,
      duration_seconds: session.duration_seconds,
      summary: session.summary,
    }),
  };
}

function bookmarkToEvent(bookmark: ProjectBookmark): TimelineEvent {
  return {
    timestamp: bookmark.created_at,
    category: "bookmark",
    action: "bookmark_created",
    description: `Bookmark: ${bookmark.name}${bookmark.notes ? ` — ${bookmark.notes}` : ""}`,
    user: bookmark.created_by || "",
    file_path: bookmark.target_path || "",
    details: JSON.stringify({
      bookmark_id: bookmark.id,
      target_type: bookmark.target_type,
      tags: bookmark.tags,
      color: bookmark.color,
    }),
  };
}

function noteToEvent(note: ProjectNote): TimelineEvent {
  const preview = (note.content || "").slice(0, 120).replace(/\n/g, " ");
  return {
    timestamp: note.created_at,
    category: "note",
    action: "note_created",
    description: `Note: ${note.title || "Untitled"}${preview ? ` — ${preview}` : ""}`,
    user: note.created_by || "",
    file_path: note.target_path || "",
    details: JSON.stringify({
      note_id: note.id,
      target_type: note.target_type,
      priority: note.priority,
      tags: note.tags,
    }),
  };
}

// =============================================================================
// Timeline Builder
// =============================================================================

/**
 * Build a comprehensive forensic timeline from all project data sources.
 * Events are returned in chronological order (oldest first).
 */
export function buildFullTimeline(project: FFXProject, exportedBy: string = "user"): FullTimelineExport {
  const events: TimelineEvent[] = [];

  // 1. Activity log entries
  if (project.activity_log) {
    for (const entry of project.activity_log) {
      events.push(activityToEvent(entry));
    }
  }

  // 2. Session start/end events
  if (project.sessions) {
    for (const session of project.sessions) {
      events.push(sessionStartEvent(session));
      const endEvt = sessionEndEvent(session);
      if (endEvt) events.push(endEvt);
    }
  }

  // 3. Bookmarks
  if (project.bookmarks) {
    for (const bookmark of project.bookmarks) {
      events.push(bookmarkToEvent(bookmark));
    }
  }

  // 4. Notes
  if (project.notes) {
    for (const note of project.notes) {
      events.push(noteToEvent(note));
    }
  }

  // Sort chronologically (oldest first for forensic timeline)
  events.sort((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime());

  // Compute date range
  const dateRange: [string, string] | null = events.length > 0
    ? [events[0].timestamp, events[events.length - 1].timestamp]
    : null;

  return {
    metadata: {
      project_name: project.name || "Unknown",
      export_date: new Date().toISOString(),
      exported_by: exportedBy,
      total_events: events.length,
      date_range: dateRange,
      format_version: "1.0",
    },
    events,
  };
}

// =============================================================================
// Export Formatters
// =============================================================================

/**
 * Convert timeline to CSV string.
 */
export function timelineToCsv(timeline: FullTimelineExport): string {
  const header = "Timestamp,Category,Action,Description,User,File Path,Details";
  const rows = timeline.events.map(e => {
    const esc = (val: string) => {
      if (!val) return "";
      if (val.includes(",") || val.includes('"') || val.includes("\n")) {
        return `"${val.replace(/"/g, '""')}"`;
      }
      return val;
    };
    return [
      e.timestamp,
      e.category,
      esc(e.action),
      esc(e.description),
      esc(e.user),
      esc(e.file_path),
      esc(e.details),
    ].join(",");
  });
  return [header, ...rows].join("\n");
}

/**
 * Convert timeline to pretty-printed JSON string.
 */
export function timelineToJson(timeline: FullTimelineExport): string {
  return JSON.stringify(timeline, null, 2);
}
