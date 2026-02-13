// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { createRoot, createSignal } from "solid-js";
import { useActivityLogging, type UseActivityLoggingDeps } from "../useActivityLogging";
import type { DiscoveredFile } from "../../types";
import type { FileHashInfo } from "../../types/hash";

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

describe("useActivityLogging", () => {
  let logActivity: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    logActivity = vi.fn();
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
});
