// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  buildFullTimeline,
  timelineToCsv,
  timelineToJson,
  type TimelineEvent,
  type FullTimelineExport,
} from "../timelineExport";
import type { FFXProject } from "../../types/project";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const makeProject = (overrides: Partial<FFXProject> = {}): FFXProject =>
  ({
    name: "Test Case",
    path: "/case/test.cffx",
    created_at: "2025-01-01T00:00:00Z",
    modified_at: "2025-01-15T00:00:00Z",
    version: "1",
    activity_log: [],
    sessions: [],
    bookmarks: [],
    notes: [],
    evidence_items: [],
    metadata: {},
    ...overrides,
  }) as unknown as FFXProject;

// ---------------------------------------------------------------------------
// buildFullTimeline
// ---------------------------------------------------------------------------

describe("buildFullTimeline", () => {
  it("returns empty events for empty project", () => {
    const result = buildFullTimeline(makeProject());
    expect(result.metadata.project_name).toBe("Test Case");
    expect(result.metadata.total_events).toBe(0);
    expect(result.metadata.date_range).toBeNull();
    expect(result.events).toEqual([]);
  });

  it("includes activity log entries", () => {
    const result = buildFullTimeline(
      makeProject({
        activity_log: [
          {
            id: "a1",
            timestamp: "2025-01-10T10:00:00Z",
            user: "agent",
            category: "file",
            action: "file_opened",
            description: "Opened disk.e01",
            file_path: "/evidence/disk.e01",
          },
        ],
      }),
    );
    expect(result.events).toHaveLength(1);
    expect(result.events[0].category).toBe("activity");
    expect(result.events[0].action).toBe("file_opened");
    expect(result.events[0].user).toBe("agent");
  });

  it("includes session start and end events", () => {
    const result = buildFullTimeline(
      makeProject({
        sessions: [
          {
            session_id: "s1",
            user: "examiner",
            started_at: "2025-01-10T08:00:00Z",
            ended_at: "2025-01-10T12:00:00Z",
            duration_seconds: 14400,
            app_version: "1.0.0",
            hostname: "lab-pc",
          },
        ],
      }),
    );
    expect(result.events).toHaveLength(2);
    expect(result.events[0].category).toBe("session_start");
    expect(result.events[0].description).toContain("lab-pc");
    expect(result.events[1].category).toBe("session_end");
    expect(result.events[1].description).toContain("240m");
  });

  it("skips session end event if session is still active", () => {
    const result = buildFullTimeline(
      makeProject({
        sessions: [
          {
            session_id: "s1",
            user: "examiner",
            started_at: "2025-01-10T08:00:00Z",
            ended_at: null,
            app_version: "1.0.0",
          },
        ],
      }),
    );
    expect(result.events).toHaveLength(1);
    expect(result.events[0].category).toBe("session_start");
  });

  it("includes bookmark events", () => {
    const result = buildFullTimeline(
      makeProject({
        bookmarks: [
          {
            id: "bm1",
            target_type: "file",
            target_path: "/evidence/disk.e01",
            name: "Important Evidence",
            created_by: "agent",
            created_at: "2025-01-11T09:00:00Z",
            notes: "Key file for case",
            tags: ["important"],
          },
        ],
      }),
    );
    expect(result.events).toHaveLength(1);
    expect(result.events[0].category).toBe("bookmark");
    expect(result.events[0].description).toContain("Important Evidence");
    expect(result.events[0].description).toContain("Key file for case");
  });

  it("includes note events", () => {
    const result = buildFullTimeline(
      makeProject({
        notes: [
          {
            id: "n1",
            target_type: "file",
            target_path: "/evidence/disk.e01",
            title: "Analysis Notes",
            content: "Found suspicious files in the user directory.",
            created_by: "examiner",
            created_at: "2025-01-12T14:00:00Z",
            modified_at: "2025-01-12T14:30:00Z",
            priority: "high",
          },
        ],
      }),
    );
    expect(result.events).toHaveLength(1);
    expect(result.events[0].category).toBe("note");
    expect(result.events[0].description).toContain("Analysis Notes");
    expect(result.events[0].description).toContain("suspicious files");
  });

  it("sorts all events chronologically", () => {
    const result = buildFullTimeline(
      makeProject({
        activity_log: [
          {
            id: "a1",
            timestamp: "2025-01-12T10:00:00Z",
            user: "agent",
            category: "file",
            action: "file_opened",
            description: "Third event",
          },
        ],
        sessions: [
          {
            session_id: "s1",
            user: "examiner",
            started_at: "2025-01-10T08:00:00Z",
            ended_at: null,
            app_version: "1.0.0",
          },
        ],
        bookmarks: [
          {
            id: "bm1",
            target_type: "file",
            target_path: "/test",
            name: "Bookmark",
            created_by: "agent",
            created_at: "2025-01-11T09:00:00Z",
          },
        ],
      }),
    );

    expect(result.events).toHaveLength(3);
    // Session start (Jan 10) < Bookmark (Jan 11) < Activity (Jan 12)
    expect(result.events[0].category).toBe("session_start");
    expect(result.events[1].category).toBe("bookmark");
    expect(result.events[2].category).toBe("activity");
  });

  it("computes correct date range", () => {
    const result = buildFullTimeline(
      makeProject({
        activity_log: [
          {
            id: "a1",
            timestamp: "2025-01-15T10:00:00Z",
            user: "agent",
            category: "file",
            action: "file_opened",
            description: "Last",
          },
          {
            id: "a2",
            timestamp: "2025-01-05T08:00:00Z",
            user: "agent",
            category: "file",
            action: "file_opened",
            description: "First",
          },
        ],
      }),
    );
    expect(result.metadata.date_range).not.toBeNull();
    expect(result.metadata.date_range![0]).toBe("2025-01-05T08:00:00Z");
    expect(result.metadata.date_range![1]).toBe("2025-01-15T10:00:00Z");
    expect(result.metadata.total_events).toBe(2);
  });

  it("sets exported_by from parameter", () => {
    const result = buildFullTimeline(makeProject(), "Special Agent");
    expect(result.metadata.exported_by).toBe("Special Agent");
  });
});

// ---------------------------------------------------------------------------
// timelineToCsv
// ---------------------------------------------------------------------------

describe("timelineToCsv", () => {
  it("produces valid CSV with header", () => {
    const timeline: FullTimelineExport = {
      metadata: {
        project_name: "Test",
        export_date: "2025-01-15T00:00:00Z",
        exported_by: "user",
        total_events: 1,
        date_range: ["2025-01-10T00:00:00Z", "2025-01-10T00:00:00Z"],
        format_version: "1.0",
      },
      events: [
        {
          timestamp: "2025-01-10T10:00:00Z",
          category: "activity",
          action: "file_opened",
          description: "Opened disk.e01",
          user: "agent",
          file_path: "/evidence/disk.e01",
          details: "",
        },
      ],
    };

    const csv = timelineToCsv(timeline);
    const lines = csv.split("\n");
    expect(lines[0]).toBe("Timestamp,Category,Action,Description,User,File Path,Details");
    expect(lines).toHaveLength(2);
    expect(lines[1]).toContain("file_opened");
    expect(lines[1]).toContain("agent");
  });

  it("escapes commas and quotes in CSV fields", () => {
    const timeline: FullTimelineExport = {
      metadata: {
        project_name: "Test",
        export_date: "2025-01-15T00:00:00Z",
        exported_by: "user",
        total_events: 1,
        date_range: null,
        format_version: "1.0",
      },
      events: [
        {
          timestamp: "2025-01-10T10:00:00Z",
          category: "activity",
          action: "test",
          description: 'File "important.doc", found',
          user: "user",
          file_path: "",
          details: "",
        },
      ],
    };

    const csv = timelineToCsv(timeline);
    // Should be wrapped in quotes with escaped inner quotes
    expect(csv).toContain('"File ""important.doc"", found"');
  });
});

// ---------------------------------------------------------------------------
// timelineToJson
// ---------------------------------------------------------------------------

describe("timelineToJson", () => {
  it("produces valid pretty-printed JSON", () => {
    const timeline = buildFullTimeline(
      makeProject({
        activity_log: [
          {
            id: "a1",
            timestamp: "2025-01-10T10:00:00Z",
            user: "agent",
            category: "file",
            action: "file_opened",
            description: "Opened disk.e01",
          },
        ],
      }),
    );

    const json = timelineToJson(timeline);
    const parsed = JSON.parse(json);
    expect(parsed.metadata.project_name).toBe("Test Case");
    expect(parsed.events).toHaveLength(1);
    expect(parsed.events[0].action).toBe("file_opened");
  });
});
