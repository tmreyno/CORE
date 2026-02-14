// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import type { CustodyRecord, TimelineEvent } from "../../types";
import type { ActivityLogEntry, ProjectSession } from "../../../../types/project";

/**
 * Replicate the session -> custody conversion logic from WizardContext.tsx
 * This tests the pure transformation without needing SolidJS rendering.
 */
function sessionsToCustody(sessions: ProjectSession[], examinerName: string): CustodyRecord[] {
  return sessions.map(session => ({
    timestamp: session.started_at,
    action: session.ended_at ? "Examination session" : "Active session",
    handler: session.user || examinerName || "",
    location: session.hostname || undefined,
    notes: session.summary ||
      (session.duration_seconds
        ? `Duration: ${Math.round(session.duration_seconds / 60)} min (${session.app_version})`
        : `App version: ${session.app_version}`),
  }));
}

/**
 * Replicate the activity log -> timeline conversion from WizardContext.tsx
 */
function activityLogToTimeline(activityLog: ActivityLogEntry[]): TimelineEvent[] {
  return activityLog.map(entry => ({
    timestamp: entry.timestamp,
    event_type: entry.category,
    description: `[${entry.action}] ${entry.description}`,
    source: entry.user || "system",
    evidence_ref: undefined,
    artifact_path: entry.file_path || undefined,
  }));
}

function makeSession(overrides: Partial<ProjectSession> = {}): ProjectSession {
  return {
    session_id: "sess-001",
    user: "analyst1",
    started_at: "2025-01-15T08:00:00Z",
    ended_at: "2025-01-15T12:00:00Z",
    duration_seconds: 14400,
    hostname: "LAB-PC-01",
    app_version: "1.0.0",
    summary: undefined,
    ...overrides,
  };
}

function makeActivity(overrides: Partial<ActivityLogEntry> = {}): ActivityLogEntry {
  return {
    id: "act-001",
    timestamp: "2025-01-15T10:00:00Z",
    user: "analyst1",
    category: "file",
    action: "open",
    description: "Opened evidence.E01",
    file_path: "/evidence/evidence.E01",
    details: {},
    ...overrides,
  };
}

describe("Session to Custody conversion", () => {
  it("converts a completed session to a custody record", () => {
    const sessions = [makeSession()];
    const result = sessionsToCustody(sessions, "Default Examiner");

    expect(result).toHaveLength(1);
    expect(result[0].timestamp).toBe("2025-01-15T08:00:00Z");
    expect(result[0].action).toBe("Examination session");
    expect(result[0].handler).toBe("analyst1");
    expect(result[0].location).toBe("LAB-PC-01");
  });

  it("marks active sessions (no ended_at)", () => {
    const sessions = [makeSession({ ended_at: null, duration_seconds: undefined })];
    const result = sessionsToCustody(sessions, "Default Examiner");

    expect(result[0].action).toBe("Active session");
  });

  it("uses examiner name as fallback when session user is empty", () => {
    const sessions = [makeSession({ user: "" })];
    const result = sessionsToCustody(sessions, "Dr. Smith");

    expect(result[0].handler).toBe("Dr. Smith");
  });

  it("uses session summary as notes when available", () => {
    const sessions = [makeSession({ summary: "Analyzed USB drive contents" })];
    const result = sessionsToCustody(sessions, "");

    expect(result[0].notes).toBe("Analyzed USB drive contents");
  });

  it("computes duration note when no summary provided", () => {
    const sessions = [makeSession({ summary: undefined, duration_seconds: 2700 })];
    const result = sessionsToCustody(sessions, "");

    expect(result[0].notes).toBe("Duration: 45 min (1.0.0)");
  });

  it("shows app version when no summary and no duration", () => {
    const sessions = [makeSession({ summary: undefined, duration_seconds: undefined })];
    const result = sessionsToCustody(sessions, "");

    expect(result[0].notes).toBe("App version: 1.0.0");
  });

  it("handles hostname undefined", () => {
    const sessions = [makeSession({ hostname: undefined })];
    const result = sessionsToCustody(sessions, "");

    expect(result[0].location).toBeUndefined();
  });

  it("converts multiple sessions in order", () => {
    const sessions = [
      makeSession({ session_id: "s1", started_at: "2025-01-15T08:00:00Z", user: "alice" }),
      makeSession({ session_id: "s2", started_at: "2025-01-16T09:00:00Z", user: "bob" }),
      makeSession({ session_id: "s3", started_at: "2025-01-17T10:00:00Z", user: "alice" }),
    ];
    const result = sessionsToCustody(sessions, "");

    expect(result).toHaveLength(3);
    expect(result[0].handler).toBe("alice");
    expect(result[1].handler).toBe("bob");
    expect(result[2].handler).toBe("alice");
  });

  it("returns empty array for empty sessions", () => {
    const result = sessionsToCustody([], "");
    expect(result).toEqual([]);
  });
});

describe("Activity Log to Timeline conversion", () => {
  it("converts an activity entry to a timeline event", () => {
    const entries = [makeActivity()];
    const result = activityLogToTimeline(entries);

    expect(result).toHaveLength(1);
    expect(result[0].timestamp).toBe("2025-01-15T10:00:00Z");
    expect(result[0].event_type).toBe("file");
    expect(result[0].description).toBe("[open] Opened evidence.E01");
    expect(result[0].source).toBe("analyst1");
    expect(result[0].artifact_path).toBe("/evidence/evidence.E01");
  });

  it("uses system as source when user is undefined", () => {
    const entries = [makeActivity({ user: undefined })];
    const result = activityLogToTimeline(entries);

    expect(result[0].source).toBe("system");
  });

  it("handles undefined file_path", () => {
    const entries = [makeActivity({ file_path: undefined })];
    const result = activityLogToTimeline(entries);

    expect(result[0].artifact_path).toBeUndefined();
  });

  it("sets evidence_ref to undefined", () => {
    const entries = [makeActivity()];
    const result = activityLogToTimeline(entries);

    expect(result[0].evidence_ref).toBeUndefined();
  });

  it("formats description as [action] description", () => {
    const entries = [makeActivity({ action: "compute", description: "SHA-256 hash" })];
    const result = activityLogToTimeline(entries);

    expect(result[0].description).toBe("[compute] SHA-256 hash");
  });

  it("maps category to event_type", () => {
    const categories = ["file", "hash", "export", "bookmark", "search", "note"];
    const entries = categories.map((cat, i) =>
      makeActivity({ id: `act-${i}`, category: cat })
    );
    const result = activityLogToTimeline(entries);

    categories.forEach((cat, i) => {
      expect(result[i].event_type).toBe(cat);
    });
  });

  it("converts multiple entries preserving order", () => {
    const entries = [
      makeActivity({ id: "a1", timestamp: "2025-01-15T10:00:00Z", action: "open" }),
      makeActivity({ id: "a2", timestamp: "2025-01-15T11:00:00Z", action: "hash" }),
      makeActivity({ id: "a3", timestamp: "2025-01-15T12:00:00Z", action: "export" }),
    ];
    const result = activityLogToTimeline(entries);

    expect(result).toHaveLength(3);
    expect(result[0].timestamp).toBe("2025-01-15T10:00:00Z");
    expect(result[2].timestamp).toBe("2025-01-15T12:00:00Z");
  });

  it("returns empty array for empty activity log", () => {
    const result = activityLogToTimeline([]);
    expect(result).toEqual([]);
  });
});
