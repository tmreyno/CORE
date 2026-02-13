// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { useNotifications } from "../useNotifications";

// ---------------------------------------------------------------------------
// Mock IPC
// ---------------------------------------------------------------------------

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

import { invoke } from "@tauri-apps/api/core";
const mockInvoke = vi.mocked(invoke);

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

describe("useNotifications", () => {
  let notifications: ReturnType<typeof useNotifications>;

  beforeEach(() => {
    vi.clearAllMocks();
    notifications = useNotifications();
  });

  // -----------------------------------------------------------------------
  // show()
  // -----------------------------------------------------------------------

  describe("show()", () => {
    it.each(["info", "success", "warning", "error"] as const)(
      "invokes notification_show with type '%s'",
      async (type) => {
        await notifications.show(type, "Title", "Message");
        expect(mockInvoke).toHaveBeenCalledWith("notification_show", {
          notificationType: type,
          title: "Title",
          message: "Message",
        });
      }
    );
  });

  // -----------------------------------------------------------------------
  // Typed convenience methods
  // -----------------------------------------------------------------------

  describe("info()", () => {
    it("invokes notification_info with title and message", async () => {
      await notifications.info("Info Title", "Info body");
      expect(mockInvoke).toHaveBeenCalledWith("notification_info", {
        title: "Info Title",
        message: "Info body",
      });
    });
  });

  describe("success()", () => {
    it("invokes notification_success", async () => {
      await notifications.success("Done", "All complete");
      expect(mockInvoke).toHaveBeenCalledWith("notification_success", {
        title: "Done",
        message: "All complete",
      });
    });
  });

  describe("warning()", () => {
    it("invokes notification_warning", async () => {
      await notifications.warning("Caution", "Low disk space");
      expect(mockInvoke).toHaveBeenCalledWith("notification_warning", {
        title: "Caution",
        message: "Low disk space",
      });
    });
  });

  describe("error()", () => {
    it("invokes notification_error", async () => {
      await notifications.error("Failed", "Hash mismatch");
      expect(mockInvoke).toHaveBeenCalledWith("notification_error", {
        title: "Failed",
        message: "Hash mismatch",
      });
    });
  });

  // -----------------------------------------------------------------------
  // setEnabled()
  // -----------------------------------------------------------------------

  describe("setEnabled()", () => {
    it("enables notifications", async () => {
      await notifications.setEnabled(true);
      expect(mockInvoke).toHaveBeenCalledWith("notification_set_enabled", {
        enabled: true,
      });
    });

    it("disables notifications", async () => {
      await notifications.setEnabled(false);
      expect(mockInvoke).toHaveBeenCalledWith("notification_set_enabled", {
        enabled: false,
      });
    });
  });

  // -----------------------------------------------------------------------
  // operationCompleted()
  // -----------------------------------------------------------------------

  describe("operationCompleted()", () => {
    it("invokes with operation name and duration", async () => {
      await notifications.operationCompleted("Hash Batch", 5432);
      expect(mockInvoke).toHaveBeenCalledWith(
        "notification_operation_completed",
        {
          operationName: "Hash Batch",
          durationMs: 5432,
        }
      );
    });

    it("handles zero duration", async () => {
      await notifications.operationCompleted("Instant Op", 0);
      expect(mockInvoke).toHaveBeenCalledWith(
        "notification_operation_completed",
        {
          operationName: "Instant Op",
          durationMs: 0,
        }
      );
    });
  });

  // -----------------------------------------------------------------------
  // operationFailed()
  // -----------------------------------------------------------------------

  describe("operationFailed()", () => {
    it("invokes with operation name and error message", async () => {
      await notifications.operationFailed("Extract", "Corrupted archive");
      expect(mockInvoke).toHaveBeenCalledWith(
        "notification_operation_failed",
        {
          operationName: "Extract",
          error: "Corrupted archive",
        }
      );
    });
  });

  // -----------------------------------------------------------------------
  // progressMilestone()
  // -----------------------------------------------------------------------

  describe("progressMilestone()", () => {
    it("invokes with current and total", async () => {
      await notifications.progressMilestone("Verify", 50, 100);
      expect(mockInvoke).toHaveBeenCalledWith(
        "notification_progress_milestone",
        {
          operationName: "Verify",
          current: 50,
          total: 100,
        }
      );
    });

    it("handles completed milestone (current === total)", async () => {
      await notifications.progressMilestone("Scan", 200, 200);
      expect(mockInvoke).toHaveBeenCalledWith(
        "notification_progress_milestone",
        {
          operationName: "Scan",
          current: 200,
          total: 200,
        }
      );
    });
  });

  // -----------------------------------------------------------------------
  // recoveryAvailable()
  // -----------------------------------------------------------------------

  describe("recoveryAvailable()", () => {
    it("invokes with operation name", async () => {
      await notifications.recoveryAvailable("Autosave");
      expect(mockInvoke).toHaveBeenCalledWith(
        "notification_recovery_available",
        {
          operationName: "Autosave",
        }
      );
    });
  });

  // -----------------------------------------------------------------------
  // Error propagation
  // -----------------------------------------------------------------------

  describe("error propagation", () => {
    it("propagates IPC errors from invoke", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("IPC failure"));
      await expect(
        notifications.success("Title", "Msg")
      ).rejects.toThrow("IPC failure");
    });
  });

  // -----------------------------------------------------------------------
  // Return shape
  // -----------------------------------------------------------------------

  describe("return shape", () => {
    it("returns all expected methods", () => {
      const keys = Object.keys(notifications).sort();
      expect(keys).toEqual([
        "error",
        "info",
        "operationCompleted",
        "operationFailed",
        "progressMilestone",
        "recoveryAvailable",
        "setEnabled",
        "show",
        "success",
        "warning",
      ]);
    });

    it("all returned values are functions", () => {
      for (const val of Object.values(notifications)) {
        expect(typeof val).toBe("function");
      }
    });
  });
});
