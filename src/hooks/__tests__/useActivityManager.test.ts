// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi } from "vitest";
import { createRoot } from "solid-js";
import { useActivityManager } from "../useActivityManager";
import type { Activity } from "../../types/activity";

/** Run a test inside a SolidJS reactive root. */
function testWithRoot<T>(fn: () => T): T {
  let result!: T;
  createRoot((dispose) => {
    result = fn();
    dispose();
  });
  return result;
}

/** Create a minimal Activity for testing. */
function makeActivity(overrides: Partial<Activity> = {}): Activity {
  return {
    id: overrides.id ?? `test-${Date.now()}`,
    type: overrides.type ?? "archive",
    status: overrides.status ?? "running",
    destination: overrides.destination ?? "/tmp/out.7z",
    sourceCount: overrides.sourceCount ?? 3,
    startTime: overrides.startTime ?? new Date(),
    ...overrides,
  };
}

describe("useActivityManager", () => {
  // ---------------------------------------------------------------------------
  // Initial state
  // ---------------------------------------------------------------------------
  describe("initial state", () => {
    it("starts with an empty activities list", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        expect(mgr.activities()).toEqual([]);
      });
    });
  });

  // ---------------------------------------------------------------------------
  // setActivities
  // ---------------------------------------------------------------------------
  describe("setActivities", () => {
    it("allows adding activities via the setter", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        const a = makeActivity({ id: "a1" });
        mgr.setActivities([a]);
        expect(mgr.activities()).toHaveLength(1);
        expect(mgr.activities()[0].id).toBe("a1");
      });
    });
  });

  // ---------------------------------------------------------------------------
  // cancel
  // ---------------------------------------------------------------------------
  describe("cancel", () => {
    it("sets a running activity to cancelled with an endTime", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        mgr.setActivities([makeActivity({ id: "c1", status: "running" })]);

        mgr.cancel("c1");

        const result = mgr.activities()[0];
        expect(result.status).toBe("cancelled");
        expect(result.endTime).toBeInstanceOf(Date);
      });
    });

    it("sets a pending activity to cancelled", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        mgr.setActivities([makeActivity({ id: "c2", status: "pending" })]);

        mgr.cancel("c2");
        expect(mgr.activities()[0].status).toBe("cancelled");
      });
    });

    it("sets a paused activity to cancelled", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        mgr.setActivities([makeActivity({ id: "c3", status: "paused" })]);

        mgr.cancel("c3");
        expect(mgr.activities()[0].status).toBe("cancelled");
      });
    });

    it("does not change a completed activity", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        mgr.setActivities([makeActivity({ id: "c4", status: "completed" })]);

        mgr.cancel("c4");
        expect(mgr.activities()[0].status).toBe("completed");
      });
    });

    it("invokes the registered cleanup function on cancel", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        const cleanup = vi.fn();
        mgr.setActivities([makeActivity({ id: "c5", status: "running" })]);
        mgr.registerCleanup("c5", cleanup);

        mgr.cancel("c5");

        expect(cleanup).toHaveBeenCalledOnce();
      });
    });

    it("handles cleanup errors gracefully without throwing", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        const badCleanup = vi.fn(() => {
          throw new Error("cleanup failed");
        });
        mgr.setActivities([makeActivity({ id: "c6", status: "running" })]);
        mgr.registerCleanup("c6", badCleanup);

        // Should not throw
        expect(() => mgr.cancel("c6")).not.toThrow();
        expect(mgr.activities()[0].status).toBe("cancelled");
      });
    });

    it("does not fail when cancelling an activity without cleanup", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        mgr.setActivities([makeActivity({ id: "c7", status: "running" })]);

        expect(() => mgr.cancel("c7")).not.toThrow();
        expect(mgr.activities()[0].status).toBe("cancelled");
      });
    });
  });

  // ---------------------------------------------------------------------------
  // pause
  // ---------------------------------------------------------------------------
  describe("pause", () => {
    it("pauses a running activity", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        mgr.setActivities([makeActivity({ id: "p1", status: "running" })]);

        mgr.pause("p1");
        expect(mgr.activities()[0].status).toBe("paused");
      });
    });

    it("does not pause a pending activity", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        mgr.setActivities([makeActivity({ id: "p2", status: "pending" })]);

        mgr.pause("p2");
        expect(mgr.activities()[0].status).toBe("pending");
      });
    });

    it("does not pause a completed activity", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        mgr.setActivities([makeActivity({ id: "p3", status: "completed" })]);

        mgr.pause("p3");
        expect(mgr.activities()[0].status).toBe("completed");
      });
    });
  });

  // ---------------------------------------------------------------------------
  // resume
  // ---------------------------------------------------------------------------
  describe("resume", () => {
    it("resumes a paused activity to running", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        mgr.setActivities([makeActivity({ id: "r1", status: "paused" })]);

        mgr.resume("r1");
        expect(mgr.activities()[0].status).toBe("running");
      });
    });

    it("does not resume a cancelled activity", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        mgr.setActivities([makeActivity({ id: "r2", status: "cancelled" })]);

        mgr.resume("r2");
        expect(mgr.activities()[0].status).toBe("cancelled");
      });
    });
  });

  // ---------------------------------------------------------------------------
  // clear
  // ---------------------------------------------------------------------------
  describe("clear", () => {
    it("removes an activity from the list", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        mgr.setActivities([
          makeActivity({ id: "x1" }),
          makeActivity({ id: "x2" }),
        ]);

        mgr.clear("x1");
        expect(mgr.activities()).toHaveLength(1);
        expect(mgr.activities()[0].id).toBe("x2");
      });
    });

    it("does nothing for a non-existent ID", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        mgr.setActivities([makeActivity({ id: "x3" })]);

        mgr.clear("does-not-exist");
        expect(mgr.activities()).toHaveLength(1);
      });
    });
  });

  // ---------------------------------------------------------------------------
  // registerCleanup / unregisterCleanup
  // ---------------------------------------------------------------------------
  describe("cleanup registration", () => {
    it("registerCleanup stores and unregisterCleanup removes cleanup", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        const cleanup = vi.fn();
        mgr.setActivities([makeActivity({ id: "reg1", status: "running" })]);

        mgr.registerCleanup("reg1", cleanup);
        mgr.unregisterCleanup("reg1");
        mgr.cancel("reg1");

        // Cleanup should NOT be called because it was unregistered
        expect(cleanup).not.toHaveBeenCalled();
      });
    });
  });

  // ---------------------------------------------------------------------------
  // Multi-activity scenarios
  // ---------------------------------------------------------------------------
  describe("multi-activity scenarios", () => {
    it("cancelling one activity does not affect others", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        mgr.setActivities([
          makeActivity({ id: "m1", status: "running" }),
          makeActivity({ id: "m2", status: "running" }),
        ]);

        mgr.cancel("m1");

        expect(mgr.activities()[0].status).toBe("cancelled");
        expect(mgr.activities()[1].status).toBe("running");
      });
    });

    it("pause-resume cycle preserves other activity states", () => {
      testWithRoot(() => {
        const mgr = useActivityManager();
        mgr.setActivities([
          makeActivity({ id: "m3", status: "running" }),
          makeActivity({ id: "m4", status: "completed" }),
        ]);

        mgr.pause("m3");
        expect(mgr.activities()[0].status).toBe("paused");
        expect(mgr.activities()[1].status).toBe("completed");

        mgr.resume("m3");
        expect(mgr.activities()[0].status).toBe("running");
        expect(mgr.activities()[1].status).toBe("completed");
      });
    });
  });
});
