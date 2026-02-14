// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  createActivity,
  updateProgress,
  completeActivity,
  failActivity,
  cancelActivity,
  calculateSpeed,
  formatSpeed,
  calculateETA,
  formatETA,
  getDuration,
  formatDuration,
} from "../types/activity";
import type { Activity, ActivityProgress } from "../types/activity";

// Helper to create a base activity for testing
function makeActivity(overrides: Partial<Activity> = {}): Activity {
  return {
    id: "test-123",
    type: "archive",
    status: "pending",
    destination: "/output/archive.7z",
    sourceCount: 5,
    startTime: new Date("2024-01-15T10:00:00Z"),
    ...overrides,
  };
}

// =============================================================================
// createActivity
// =============================================================================
describe("createActivity", () => {
  it("creates activity with required fields", () => {
    const activity = createActivity("archive", "/out/file.7z", 3);
    expect(activity.type).toBe("archive");
    expect(activity.status).toBe("pending");
    expect(activity.destination).toBe("/out/file.7z");
    expect(activity.sourceCount).toBe(3);
    expect(activity.startTime).toBeInstanceOf(Date);
  });

  it("generates unique ids", () => {
    const a1 = createActivity("archive", "/a", 1);
    const a2 = createActivity("archive", "/b", 2);
    expect(a1.id).not.toBe(a2.id);
  });

  it("includes optional properties when provided", () => {
    const activity = createActivity("archive", "/out", 1, {
      compressionLevel: 9,
      encrypted: true,
    });
    expect(activity.compressionLevel).toBe(9);
    expect(activity.encrypted).toBe(true);
  });

  it("supports all activity types", () => {
    expect(createActivity("archive", "/a", 1).type).toBe("archive");
    expect(createActivity("export", "/b", 1).type).toBe("export");
    expect(createActivity("copy", "/c", 1).type).toBe("copy");
    expect(createActivity("tool", "/d", 1).type).toBe("tool");
  });
});

// =============================================================================
// updateProgress
// =============================================================================
describe("updateProgress", () => {
  it("sets status to running", () => {
    const activity = makeActivity();
    const progress: ActivityProgress = { percent: 50 };
    const updated = updateProgress(activity, progress);
    expect(updated.status).toBe("running");
  });

  it("attaches progress data", () => {
    const activity = makeActivity();
    const progress: ActivityProgress = {
      percent: 75,
      bytesProcessed: 1024,
      bytesTotal: 2048,
      currentFile: "test.txt",
    };
    const updated = updateProgress(activity, progress);
    expect(updated.progress!.percent).toBe(75);
    expect(updated.progress!.bytesProcessed).toBe(1024);
    expect(updated.progress!.currentFile).toBe("test.txt");
  });

  it("does not mutate original activity", () => {
    const activity = makeActivity();
    const updated = updateProgress(activity, { percent: 50 });
    expect(activity.status).toBe("pending");
    expect(updated.status).toBe("running");
  });
});

// =============================================================================
// completeActivity
// =============================================================================
describe("completeActivity", () => {
  it("sets status to completed", () => {
    const activity = makeActivity({ status: "running" });
    const completed = completeActivity(activity);
    expect(completed.status).toBe("completed");
  });

  it("sets endTime", () => {
    const completed = completeActivity(makeActivity());
    expect(completed.endTime).toBeInstanceOf(Date);
  });

  it("sets progress to 100% if progress existed", () => {
    const activity = makeActivity({
      progress: { percent: 80, bytesProcessed: 800, bytesTotal: 1000 },
    });
    const completed = completeActivity(activity);
    expect(completed.progress!.percent).toBe(100);
  });

  it("leaves progress undefined if none existed", () => {
    const completed = completeActivity(makeActivity());
    expect(completed.progress).toBeUndefined();
  });
});

// =============================================================================
// failActivity
// =============================================================================
describe("failActivity", () => {
  it("sets status to failed with error message", () => {
    const failed = failActivity(makeActivity(), "Disk full");
    expect(failed.status).toBe("failed");
    expect(failed.error).toBe("Disk full");
  });

  it("sets endTime", () => {
    const failed = failActivity(makeActivity(), "Error");
    expect(failed.endTime).toBeInstanceOf(Date);
  });
});

// =============================================================================
// cancelActivity
// =============================================================================
describe("cancelActivity", () => {
  it("sets status to cancelled", () => {
    const cancelled = cancelActivity(makeActivity());
    expect(cancelled.status).toBe("cancelled");
  });

  it("sets endTime", () => {
    const cancelled = cancelActivity(makeActivity());
    expect(cancelled.endTime).toBeInstanceOf(Date);
  });
});

// =============================================================================
// calculateSpeed
// =============================================================================
describe("calculateSpeed", () => {
  it("returns null when no progress", () => {
    expect(calculateSpeed(makeActivity())).toBeNull();
  });

  it("returns null when not running", () => {
    const activity = makeActivity({
      status: "completed",
      progress: { percent: 100, bytesProcessed: 1000 },
    });
    expect(calculateSpeed(activity)).toBeNull();
  });

  it("returns null when no bytes processed", () => {
    const activity = makeActivity({
      status: "running",
      progress: { percent: 50 },
    });
    expect(calculateSpeed(activity)).toBeNull();
  });

  it("returns null when elapsed < 1 second", () => {
    const activity = makeActivity({
      status: "running",
      startTime: new Date(), // just now
      progress: { percent: 50, bytesProcessed: 1000 },
    });
    expect(calculateSpeed(activity)).toBeNull();
  });

  it("calculates bytes per second for running activity", () => {
    const activity = makeActivity({
      status: "running",
      startTime: new Date(Date.now() - 10000), // 10 seconds ago
      progress: { percent: 50, bytesProcessed: 10000 },
    });
    const speed = calculateSpeed(activity);
    expect(speed).toBeGreaterThan(900); // ~1000 bytes/sec
    expect(speed).toBeLessThan(1100);
  });
});

// =============================================================================
// formatSpeed
// =============================================================================
describe("formatSpeed", () => {
  it("formats bytes per second", () => {
    expect(formatSpeed(100)).toBe("100.0 B/s");
  });

  it("formats KB per second", () => {
    expect(formatSpeed(1024)).toBe("1.00 KB/s");
  });

  it("formats MB per second", () => {
    expect(formatSpeed(1048576)).toBe("1.00 MB/s");
  });
});

// =============================================================================
// calculateETA
// =============================================================================
describe("calculateETA", () => {
  it("returns null when no progress", () => {
    expect(calculateETA(makeActivity())).toBeNull();
  });

  it("returns null when not running", () => {
    const activity = makeActivity({
      status: "completed",
      progress: { percent: 100, bytesProcessed: 1000, bytesTotal: 1000 },
    });
    expect(calculateETA(activity)).toBeNull();
  });

  it("returns null when elapsed < 2 seconds", () => {
    const activity = makeActivity({
      status: "running",
      startTime: new Date(),
      progress: { percent: 50, bytesProcessed: 500, bytesTotal: 1000 },
    });
    expect(calculateETA(activity)).toBeNull();
  });

  it("calculates remaining time in ms", () => {
    const activity = makeActivity({
      status: "running",
      startTime: new Date(Date.now() - 10000), // 10 seconds ago
      progress: { percent: 50, bytesProcessed: 500, bytesTotal: 1000 },
    });
    const eta = calculateETA(activity);
    expect(eta).toBeGreaterThan(8000);
    expect(eta).toBeLessThan(12000);
  });
});

// =============================================================================
// formatETA
// =============================================================================
describe("formatETA", () => {
  it("formats seconds", () => {
    expect(formatETA(30000)).toBe("30s");
    expect(formatETA(5000)).toBe("5s");
  });

  it("formats minutes", () => {
    expect(formatETA(120000)).toBe("2m");
    expect(formatETA(300000)).toBe("5m");
  });

  it("formats hours and minutes", () => {
    expect(formatETA(3660000)).toBe("1h 1m");
    expect(formatETA(7200000)).toBe("2h 0m");
  });
});

// =============================================================================
// getDuration
// =============================================================================
describe("getDuration", () => {
  it("returns null when no endTime", () => {
    expect(getDuration(makeActivity())).toBeNull();
  });

  it("calculates duration in ms", () => {
    const activity = makeActivity({
      startTime: new Date("2024-01-15T10:00:00Z"),
      endTime: new Date("2024-01-15T10:05:00Z"),
    });
    expect(getDuration(activity)).toBe(300000); // 5 minutes
  });
});

// =============================================================================
// formatDuration (activity version - ms input)
// =============================================================================
describe("formatDuration (activity)", () => {
  it("formats seconds", () => {
    expect(formatDuration(5000)).toBe("5s");
    expect(formatDuration(45000)).toBe("45s");
  });

  it("formats minutes and seconds", () => {
    expect(formatDuration(90000)).toBe("1m 30s");
    expect(formatDuration(125000)).toBe("2m 5s");
  });

  it("formats hours and minutes", () => {
    expect(formatDuration(3660000)).toBe("1h 1m");
    expect(formatDuration(7200000)).toBe("2h 0m");
  });
});
