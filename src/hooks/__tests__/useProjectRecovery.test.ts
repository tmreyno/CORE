// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { createRoot } from "solid-js";
import {
  useProjectRecovery,
  type BackupType,
  type ProjectHealthStatus,
  type IssueSeverity,
} from "../useProjectRecovery";

// ---------------------------------------------------------------------------
// Mock Tauri invoke
// ---------------------------------------------------------------------------

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
const mockInvoke = vi.mocked(invoke);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function createHook() {
  let hook!: ReturnType<typeof useProjectRecovery>;
  let dispose!: () => void;
  createRoot((d) => {
    dispose = d;
    hook = useProjectRecovery();
  });
  return { hook, dispose };
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const projectPath = "/path/to/project.cffx";

const backupFiles = [
  {
    path: "/backups/v1.cffx",
    metadata: {
      original_path: projectPath,
      created_at: "2025-06-01T12:00:00Z",
      app_version: "1.0.0",
      file_size: 5000,
      backup_type: "ManualBackup" as BackupType,
      user: "tester",
    },
  },
  {
    path: "/backups/v2.cffx",
    metadata: {
      original_path: projectPath,
      created_at: "2025-06-02T12:00:00Z",
      app_version: "1.0.0",
      file_size: 6000,
      backup_type: "AutoSave" as BackupType,
      user: null,
    },
  },
];

const healthResult = {
  status: "Healthy" as ProjectHealthStatus,
  issues: [],
  checked_at: "2025-06-01T12:00:00Z",
  file_size: 10000,
  activity_log_size: 500,
  tab_count: 3,
  session_count: 10,
  has_backup: true,
  version_count: 2,
};

const recoveryInfo = {
  has_autosave: true,
  autosave_path: "/tmp/autosave.cffx",
  autosave_age_seconds: 120,
  autosave_is_newer: true,
  has_backup: true,
  backup_path: "/backups/latest.cffx",
};

// ==========================================================================
// Tests
// ==========================================================================

describe("useProjectRecovery", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // -----------------------------------------------------------------------
  // Initial state
  // -----------------------------------------------------------------------
  describe("initial state", () => {
    it("starts with empty backups", () => {
      const { hook, dispose } = createHook();
      expect(hook.backups()).toEqual([]);
      dispose();
    });

    it("starts with null health", () => {
      const { hook, dispose } = createHook();
      expect(hook.health()).toBeNull();
      dispose();
    });

    it("starts with empty versions", () => {
      const { hook, dispose } = createHook();
      expect(hook.versions()).toEqual([]);
      dispose();
    });

    it("starts with null recoveryInfo", () => {
      const { hook, dispose } = createHook();
      expect(hook.recoveryInfo()).toBeNull();
      dispose();
    });

    it("starts not loading", () => {
      const { hook, dispose } = createHook();
      expect(hook.loading()).toBe(false);
      dispose();
    });

    it("starts with no error", () => {
      const { hook, dispose } = createHook();
      expect(hook.error()).toBeNull();
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // createBackup
  // -----------------------------------------------------------------------
  describe("createBackup", () => {
    it("invokes backend and returns backup path", async () => {
      const { hook, dispose } = createHook();
      // First call: createBackup, second: listVersions (called inside)
      mockInvoke
        .mockResolvedValueOnce("/backups/new.cffx")
        .mockResolvedValueOnce(backupFiles);

      const result = await hook.createBackup(projectPath, "ManualBackup", "user1");

      expect(mockInvoke).toHaveBeenCalledWith("project_create_backup", {
        project_path: projectPath,
        backup_type: "ManualBackup",
        user: "user1",
      });
      expect(result).toBe("/backups/new.cffx");
      expect(hook.loading()).toBe(false);
      dispose();
    });

    it("defaults to ManualBackup type and null user", async () => {
      const { hook, dispose } = createHook();
      mockInvoke
        .mockResolvedValueOnce("/backups/new.cffx")
        .mockResolvedValueOnce([]);

      await hook.createBackup(projectPath);

      expect(mockInvoke).toHaveBeenCalledWith("project_create_backup", {
        project_path: projectPath,
        backup_type: "ManualBackup",
        user: null,
      });
      dispose();
    });

    it("sets error on failure", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockRejectedValueOnce(new Error("Backup failed"));

      const result = await hook.createBackup(projectPath);

      expect(result).toBeNull();
      expect(hook.error()).toBe("Backup failed");
      expect(hook.loading()).toBe(false);
      dispose();
    });

    it("refreshes versions after successful backup", async () => {
      const { hook, dispose } = createHook();
      mockInvoke
        .mockResolvedValueOnce("/backups/new.cffx")
        .mockResolvedValueOnce(backupFiles);

      await hook.createBackup(projectPath);

      // listVersions should have been called
      expect(mockInvoke).toHaveBeenCalledWith("project_list_versions", {
        project_path: projectPath,
      });
      expect(hook.versions()).toEqual(backupFiles);
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // createVersionBackup
  // -----------------------------------------------------------------------
  describe("createVersionBackup", () => {
    it("invokes backend and returns version path", async () => {
      const { hook, dispose } = createHook();
      mockInvoke
        .mockResolvedValueOnce("/versions/v3.cffx")
        .mockResolvedValueOnce(backupFiles);

      const result = await hook.createVersionBackup(projectPath);

      expect(mockInvoke).toHaveBeenCalledWith("project_create_version", {
        project_path: projectPath,
      });
      expect(result).toBe("/versions/v3.cffx");
      dispose();
    });

    it("sets error on failure", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockRejectedValueOnce(new Error("Version failed"));

      const result = await hook.createVersionBackup(projectPath);

      expect(result).toBeNull();
      expect(hook.error()).toBe("Version failed");
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // listVersions
  // -----------------------------------------------------------------------
  describe("listVersions", () => {
    it("fetches and stores versions", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockResolvedValueOnce(backupFiles);

      const result = await hook.listVersions(projectPath);

      expect(result).toEqual(backupFiles);
      expect(hook.versions()).toEqual(backupFiles);
      expect(hook.backups()).toEqual(backupFiles); // also set for compat
      dispose();
    });

    it("returns empty array on failure", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockRejectedValueOnce(new Error("List failed"));

      const result = await hook.listVersions(projectPath);

      expect(result).toEqual([]);
      expect(hook.error()).toBe("List failed");
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // checkRecovery
  // -----------------------------------------------------------------------
  describe("checkRecovery", () => {
    it("fetches and stores recovery info", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockResolvedValueOnce(recoveryInfo);

      const result = await hook.checkRecovery(projectPath);

      expect(result).toEqual(recoveryInfo);
      expect(hook.recoveryInfo()).toEqual(recoveryInfo);
      dispose();
    });

    it("returns null on failure", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockRejectedValueOnce(new Error("Recovery check failed"));

      const result = await hook.checkRecovery(projectPath);

      expect(result).toBeNull();
      expect(hook.error()).toBe("Recovery check failed");
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // recoverFromAutosave
  // -----------------------------------------------------------------------
  describe("recoverFromAutosave", () => {
    it("returns recovered project", async () => {
      const { hook, dispose } = createHook();
      const project = { name: "Recovered" };
      mockInvoke.mockResolvedValueOnce(project);

      const result = await hook.recoverFromAutosave(projectPath);

      expect(mockInvoke).toHaveBeenCalledWith("project_recover_autosave", {
        project_path: projectPath,
      });
      expect(result).toEqual(project);
      dispose();
    });

    it("returns null on failure", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockRejectedValueOnce(new Error("Recovery failed"));

      const result = await hook.recoverFromAutosave(projectPath);

      expect(result).toBeNull();
      expect(hook.error()).toBe("Recovery failed");
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // clearAutosave
  // -----------------------------------------------------------------------
  describe("clearAutosave", () => {
    it("returns true on success", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockResolvedValueOnce(undefined);

      const result = await hook.clearAutosave(projectPath);

      expect(result).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith("project_clear_autosave", {
        project_path: projectPath,
      });
      dispose();
    });

    it("returns false on failure", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockRejectedValueOnce(new Error("Clear failed"));

      const result = await hook.clearAutosave(projectPath);

      expect(result).toBe(false);
      expect(hook.error()).toBe("Clear failed");
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // checkHealth
  // -----------------------------------------------------------------------
  describe("checkHealth", () => {
    it("fetches and stores health result", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockResolvedValueOnce(healthResult);

      const result = await hook.checkHealth(projectPath);

      expect(result).toEqual(healthResult);
      expect(hook.health()).toEqual(healthResult);
      dispose();
    });

    it("returns null on failure", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockRejectedValueOnce(new Error("Health check failed"));

      const result = await hook.checkHealth(projectPath);

      expect(result).toBeNull();
      expect(hook.error()).toBe("Health check failed");
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Pure helpers
  // -----------------------------------------------------------------------
  describe("getHealthStatusColor", () => {
    it.each([
      ["Healthy", "text-success"],
      ["Warning", "text-warning"],
      ["Critical", "text-error"],
    ] as [ProjectHealthStatus, string][])(
      "maps %s to %s",
      (status, expected) => {
        const { hook, dispose } = createHook();
        expect(hook.getHealthStatusColor(status)).toBe(expected);
        dispose();
      }
    );

    it("returns text-txt-muted for unknown status", () => {
      const { hook, dispose } = createHook();
      expect(hook.getHealthStatusColor("Unknown" as any)).toBe("text-txt-muted");
      dispose();
    });
  });

  describe("getSeverityColor", () => {
    it.each([
      ["Info", "text-info"],
      ["Warning", "text-warning"],
      ["Error", "text-error"],
      ["Critical", "text-error"],
    ] as [IssueSeverity, string][])(
      "maps %s to %s",
      (severity, expected) => {
        const { hook, dispose } = createHook();
        expect(hook.getSeverityColor(severity)).toBe(expected);
        dispose();
      }
    );

    it("returns text-txt-muted for unknown severity", () => {
      const { hook, dispose } = createHook();
      expect(hook.getSeverityColor("Unknown" as any)).toBe("text-txt-muted");
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Loading / error state management
  // -----------------------------------------------------------------------
  describe("loading and error state", () => {
    it("clears error before each operation", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockRejectedValueOnce(new Error("first"));
      await hook.checkHealth(projectPath);
      expect(hook.error()).toBe("first");

      mockInvoke.mockResolvedValueOnce(healthResult);
      await hook.checkHealth(projectPath);
      expect(hook.error()).toBeNull();
      dispose();
    });

    it("handles string errors", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockRejectedValueOnce("string error");

      await hook.checkHealth(projectPath);

      expect(hook.error()).toBe("string error");
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Backup types
  // -----------------------------------------------------------------------
  describe("backup types", () => {
    const types: BackupType[] = ["ManualSave", "AutoSave", "ManualBackup", "PreOperation"];

    it.each(types)("passes %s backup type to backend", async (type) => {
      const { hook, dispose } = createHook();
      mockInvoke
        .mockResolvedValueOnce("/backup/path")
        .mockResolvedValueOnce([]);

      await hook.createBackup(projectPath, type);

      expect(mockInvoke).toHaveBeenCalledWith("project_create_backup", expect.objectContaining({
        backup_type: type,
      }));
      dispose();
    });
  });
});
