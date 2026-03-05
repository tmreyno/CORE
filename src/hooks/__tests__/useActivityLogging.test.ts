// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";
import { createRoot, createSignal } from "solid-js";
import { useActivityLogging, type UseActivityLoggingDeps } from "../useActivityLogging";
import type { DiscoveredFile } from "../../types";
import type { FileHashInfo } from "../../types/hash";
import type { Activity } from "../../types/activity";

/**
 * Run a test inside a SolidJS reactive root that stays alive long enough
 * for deferred effects to fire. Ensures dispose is always called.
 */
function testWithRoot(fn: (dispose: () => void) => Promise<void>): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    createRoot(async (dispose) => {
      try {
        await fn(dispose);
        dispose();
        resolve();
      } catch (err) {
        dispose();
        reject(err);
      }
    });
  });
}

/** Flush microtasks so deferred effects can fire. */
function flush(): Promise<void> {
  return new Promise((r) => setTimeout(r, 0));
}

/** Create a minimal DiscoveredFile. */
function makeFile(overrides: Partial<DiscoveredFile> = {}): DiscoveredFile {
  return {
    path: overrides.path ?? "/evidence/test.e01",
    filename: overrides.filename ?? "test.e01",
    container_type: overrides.container_type ?? "E01",
    size: overrides.size ?? 1048576,
    segment_count: overrides.segment_count ?? 1,
    ...overrides,
  };
}

type LogActivityFn = UseActivityLoggingDeps["projectManager"]["logActivity"];

describe("useActivityLogging", () => {
  let logActivity: Mock<LogActivityFn>;

  beforeEach(() => {
    logActivity = vi.fn<LogActivityFn>();
  });

  // ---------------------------------------------------------------------------
  // File selection tracking
  // ---------------------------------------------------------------------------
  describe("file/open tracking", () => {
    it("logs when a new file is selected", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile, setActiveFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap] = createSignal(new Map<string, FileHashInfo>());

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
        };

        useActivityLogging(deps);
        await flush();

        // Select a file
        const file = makeFile({ path: "/evidence/test.e01", filename: "test.e01" });
        setActiveFile(file);
        await flush();

        expect(logActivity).toHaveBeenCalledWith(
          "file",
          "open",
          "Opened file: test.e01",
          "/evidence/test.e01",
          expect.objectContaining({ containerType: "E01" }),
        );

        dispose();
      });
    });

    it("does NOT log when same file is re-selected", async () => {
      await testWithRoot(async (dispose) => {
        const file = makeFile({ path: "/evidence/test.e01" });
        const [activeFile, setActiveFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap] = createSignal(new Map<string, FileHashInfo>());

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
        };

        useActivityLogging(deps);
        await flush();

        // Select same file twice
        setActiveFile(file);
        await flush();
        logActivity.mockClear();

        setActiveFile(file); // same path
        await flush();

        expect(logActivity).not.toHaveBeenCalled();
        dispose();
      });
    });

    it("does NOT log when file is cleared (set to null)", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile, setActiveFile] = createSignal<DiscoveredFile | null>(
          makeFile(),
        );
        const [discoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap] = createSignal(new Map<string, FileHashInfo>());

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
        };

        useActivityLogging(deps);
        await flush();
        logActivity.mockClear();

        setActiveFile(null);
        await flush();

        expect(logActivity).not.toHaveBeenCalled();
        dispose();
      });
    });
  });

  // ---------------------------------------------------------------------------
  // Hash computation tracking
  // ---------------------------------------------------------------------------
  describe("hash/compute tracking", () => {
    it("logs when a new hash is computed", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap, setHashMap] = createSignal(new Map<string, FileHashInfo>());

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
        };

        useActivityLogging(deps);
        await flush();

        // Add a hash entry
        const newMap = new Map<string, FileHashInfo>();
        newMap.set("/evidence/test.e01", {
          algorithm: "SHA-256",
          hash: "abc123def456abc123def456abc123de",
          verified: undefined,
        });
        setHashMap(newMap);
        await flush();

        expect(logActivity).toHaveBeenCalledWith(
          "hash",
          "compute",
          expect.stringContaining("SHA-256"),
          "/evidence/test.e01",
          expect.objectContaining({ algorithm: "SHA-256" }),
        );

        dispose();
      });
    });

    it("logs hash verification completions", async () => {
      await testWithRoot(async () => {
        const [activeFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap, setHashMap] = createSignal(new Map<string, FileHashInfo>());

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
        };

        useActivityLogging(deps);
        await flush();

        // First change: add a hash entry (triggers "compute")
        const map1 = new Map<string, FileHashInfo>();
        map1.set("/evidence/test.e01", {
          algorithm: "MD5",
          hash: "abc123",
          verified: undefined,
        });
        setHashMap(map1);
        await flush();
        logActivity.mockClear();

        // Second change: same hash but verified=true (triggers "verify")
        const map2 = new Map<string, FileHashInfo>();
        map2.set("/evidence/test.e01", {
          algorithm: "MD5",
          hash: "abc123",
          verified: true,
        });
        setHashMap(map2);
        await flush();

        const verifyCalls = logActivity.mock.calls.filter(
          (c: unknown[]) => c[0] === "hash" && c[1] === "verify",
        );
        expect(verifyCalls).toHaveLength(1);
        expect(verifyCalls[0][2]).toContain("MATCH");
      });
    });

    it("logs MISMATCH when verification fails", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap, setHashMap] = createSignal(new Map<string, FileHashInfo>());

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
        };

        useActivityLogging(deps);
        await flush();

        // First change: add hash entry
        const map1 = new Map<string, FileHashInfo>();
        map1.set("/evidence/bad.e01", {
          algorithm: "SHA-1",
          hash: "abc123",
          verified: undefined,
        });
        setHashMap(map1);
        await flush();
        logActivity.mockClear();

        // Second change: verified=false
        const map2 = new Map<string, FileHashInfo>();
        map2.set("/evidence/bad.e01", {
          algorithm: "SHA-1",
          hash: "abc123",
          verified: false,
        });
        setHashMap(map2);
        await flush();

        const verifyCalls = logActivity.mock.calls.filter(
          (c: unknown[]) => c[0] === "hash" && c[1] === "verify",
        );
        expect(verifyCalls).toHaveLength(1);
        expect(verifyCalls[0][2]).toContain("MISMATCH");
        expect(verifyCalls[0][4]).toEqual(expect.objectContaining({ verified: false }));

        dispose();
      });
    });

    it("does NOT log when hash map is empty", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap, setHashMap] = createSignal(new Map<string, FileHashInfo>());

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
        };

        useActivityLogging(deps);
        await flush();

        // Set an empty map
        setHashMap(new Map());
        await flush();

        expect(logActivity).not.toHaveBeenCalled();
        dispose();
      });
    });
  });

  // ---------------------------------------------------------------------------
  // Directory scan tracking
  // ---------------------------------------------------------------------------
  describe("file/scan tracking", () => {
    it("logs when files are discovered in a directory", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles, setDiscoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence/case001");
        const [hashMap] = createSignal(new Map<string, FileHashInfo>());

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
        };

        useActivityLogging(deps);
        await flush();

        // Discover files
        setDiscoveredFiles([
          makeFile({ path: "/evidence/case001/a.e01" }),
          makeFile({ path: "/evidence/case001/b.ad1" }),
          makeFile({ path: "/evidence/case001/c.l01" }),
        ]);
        await flush();

        expect(logActivity).toHaveBeenCalledWith(
          "file",
          "scan",
          expect.stringContaining("3 evidence files"),
          "/evidence/case001",
          expect.objectContaining({ fileCount: 3 }),
        );

        dispose();
      });
    });

    it("does NOT log when files are cleared", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles, setDiscoveredFiles] = createSignal<DiscoveredFile[]>([
          makeFile(),
        ]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap] = createSignal(new Map<string, FileHashInfo>());

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
        };

        useActivityLogging(deps);
        await flush();
        logActivity.mockClear();

        // Clear files
        setDiscoveredFiles([]);
        await flush();

        expect(logActivity).not.toHaveBeenCalled();
        dispose();
      });
    });

    it("includes directory name in log description", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles, setDiscoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/long/path/to/my-case");
        const [hashMap] = createSignal(new Map<string, FileHashInfo>());

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
        };

        useActivityLogging(deps);
        await flush();

        setDiscoveredFiles([makeFile()]);
        await flush();

        expect(logActivity).toHaveBeenCalledWith(
          "file",
          "scan",
          expect.stringContaining("my-case"),
          "/long/path/to/my-case",
          expect.any(Object),
        );

        dispose();
      });
    });
  });

  // ---------------------------------------------------------------------------
  // Deferred behavior
  // ---------------------------------------------------------------------------
  describe("deferred behavior", () => {
    it("does NOT log initial state on mount", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap] = createSignal(new Map<string, FileHashInfo>());

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
        };

        useActivityLogging(deps);
        await flush();

        // No logging should have occurred just from initialization
        expect(logActivity).not.toHaveBeenCalled();
        dispose();
      });
    });
  });

  // ---------------------------------------------------------------------------
  // Export/archive/copy completion tracking
  // ---------------------------------------------------------------------------
  describe("export tracking", () => {
    /** Create a minimal Activity for testing */
    function makeActivity(overrides: Partial<Activity> = {}): Activity {
      return {
        id: overrides.id ?? "act-1",
        type: overrides.type ?? "export",
        status: overrides.status ?? "running",
        destination: overrides.destination ?? "/exports/evidence.7z",
        sourceCount: overrides.sourceCount ?? 3,
        startTime: overrides.startTime ?? new Date(),
        endTime: overrides.endTime ?? undefined,
        progress: overrides.progress ?? undefined,
        error: overrides.error ?? undefined,
      };
    }

    it("logs when an export operation completes", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap] = createSignal(new Map<string, FileHashInfo>());
        const [activities, setActivities] = createSignal<Activity[]>([]);

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
          activities,
        };

        useActivityLogging(deps);
        await flush();

        // Add a completed export activity
        const now = new Date();
        setActivities([makeActivity({
          id: "exp-1",
          type: "export",
          status: "completed",
          destination: "/exports/forensic_export",
          sourceCount: 5,
          startTime: new Date(now.getTime() - 10000),
          endTime: now,
        })]);
        await flush();

        expect(logActivity).toHaveBeenCalledWith(
          "export",
          "complete",
          expect.stringContaining("Export completed"),
          "/exports/forensic_export",
          expect.objectContaining({ type: "export", sourceCount: 5 }),
        );

        dispose();
      });
    });

    it("logs when an archive operation completes", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap] = createSignal(new Map<string, FileHashInfo>());
        const [activities, setActivities] = createSignal<Activity[]>([]);

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
          activities,
        };

        useActivityLogging(deps);
        await flush();

        setActivities([makeActivity({
          id: "arc-1",
          type: "archive",
          status: "completed",
          destination: "/exports/evidence.7z",
        })]);
        await flush();

        expect(logActivity).toHaveBeenCalledWith(
          "export",
          "complete",
          expect.stringContaining("Archive created"),
          "/exports/evidence.7z",
          expect.objectContaining({ type: "archive" }),
        );

        dispose();
      });
    });

    it("logs when an export operation fails", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap] = createSignal(new Map<string, FileHashInfo>());
        const [activities, setActivities] = createSignal<Activity[]>([]);

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
          activities,
        };

        useActivityLogging(deps);
        await flush();

        setActivities([makeActivity({
          id: "fail-1",
          type: "export",
          status: "failed",
          destination: "/exports/broken",
          error: "Permission denied",
        })]);
        await flush();

        expect(logActivity).toHaveBeenCalledWith(
          "export",
          "fail",
          expect.stringContaining("Export"),
          "/exports/broken",
          expect.objectContaining({ error: "Permission denied" }),
        );

        dispose();
      });
    });

    it("logs when an operation is cancelled", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap] = createSignal(new Map<string, FileHashInfo>());
        const [activities, setActivities] = createSignal<Activity[]>([]);

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
          activities,
        };

        useActivityLogging(deps);
        await flush();

        setActivities([makeActivity({
          id: "cancel-1",
          type: "copy",
          status: "cancelled",
          destination: "/exports/cancelled_copy",
        })]);
        await flush();

        expect(logActivity).toHaveBeenCalledWith(
          "export",
          "cancel",
          expect.stringContaining("cancelled"),
          "/exports/cancelled_copy",
          expect.objectContaining({ type: "copy" }),
        );

        dispose();
      });
    });

    it("does NOT log the same activity twice", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap] = createSignal(new Map<string, FileHashInfo>());
        const [activities, setActivities] = createSignal<Activity[]>([]);

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
          activities,
        };

        useActivityLogging(deps);
        await flush();

        const completed = makeActivity({
          id: "dup-1",
          type: "export",
          status: "completed",
          destination: "/exports/test",
        });

        setActivities([completed]);
        await flush();
        const callCount1 = logActivity.mock.calls.filter(
          (c: unknown[]) => c[0] === "export"
        ).length;

        // Set same activity again
        setActivities([completed]);
        await flush();
        const callCount2 = logActivity.mock.calls.filter(
          (c: unknown[]) => c[0] === "export"
        ).length;

        expect(callCount2).toBe(callCount1);

        dispose();
      });
    });

    it("logs running activities as export/start", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap] = createSignal(new Map<string, FileHashInfo>());
        const [activities, setActivities] = createSignal<Activity[]>([]);

        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
          activities,
        };

        useActivityLogging(deps);
        await flush();

        setActivities([makeActivity({
          id: "run-1",
          type: "export",
          status: "running",
          destination: "/exports/in_progress",
        })]);
        await flush();

        const exportCalls = logActivity.mock.calls.filter(
          (c: unknown[]) => c[0] === "export"
        );
        expect(exportCalls).toHaveLength(1);
        expect(exportCalls[0][1]).toBe("start");
        expect(exportCalls[0][2]).toContain("started");

        dispose();
      });
    });

    it("works without activities accessor (backward compatible)", async () => {
      await testWithRoot(async (dispose) => {
        const [activeFile, setActiveFile] = createSignal<DiscoveredFile | null>(null);
        const [discoveredFiles] = createSignal<DiscoveredFile[]>([]);
        const [scanDir] = createSignal("/evidence");
        const [hashMap] = createSignal(new Map<string, FileHashInfo>());

        // No activities accessor — should not crash
        const deps: UseActivityLoggingDeps = {
          fileManager: { activeFile, discoveredFiles, scanDir },
          hashManager: { fileHashMap: hashMap },
          projectManager: { logActivity },
        };

        useActivityLogging(deps);
        await flush();

        // File logging still works
        setActiveFile(makeFile());
        await flush();

        expect(logActivity).toHaveBeenCalledWith(
          "file",
          "open",
          expect.any(String),
          expect.any(String),
          expect.any(Object),
        );

        dispose();
      });
    });
  });
});
