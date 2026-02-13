// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { createRoot } from "solid-js";
import {
  useProjectComparison,
  type ProjectComparison,
  type MergeResult,
  type MergeStrategy,
} from "../useProjectComparison";

// ---------------------------------------------------------------------------
// Mock Tauri invoke
// ---------------------------------------------------------------------------

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
const mockInvoke = vi.mocked(invoke);

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

function makeComparison(overrides: Partial<ProjectComparison> = {}): ProjectComparison {
  return {
    project_a_name: "Project A",
    project_b_name: "Project B",
    compared_at: "2025-06-01T12:00:00Z",
    summary: {
      unique_to_a: 3,
      unique_to_b: 2,
      common: 10,
      modified: 1,
      similarity_percent: 75,
      total_differences: 6,
    },
    bookmarks: {
      only_in_a: ["bm-a1", "bm-a2"],
      only_in_b: ["bm-b1"],
      common: ["bm-shared"],
      modified: ["bm-mod"],
    },
    notes: {
      only_in_a: ["note-a"],
      only_in_b: ["note-b1", "note-b2"],
      common: ["note-shared"],
      modified: [],
    },
    evidence: {
      only_in_a: ["ev-a"],
      only_in_b: [],
      common: ["ev-shared1", "ev-shared2"],
    },
    activity: {
      count_a: 50,
      count_b: 30,
      types_only_in_a: ["hash"],
      types_only_in_b: [],
      common_types: ["view", "open"],
    },
    conflicts: [
      {
        conflict_type: "bookmark_conflict",
        description: "Bookmark X differs",
        item_a: { id: 1 },
        item_b: { id: 2 },
        resolution: "manual",
      },
    ],
    recommendations: ["Merge bookmarks from A"],
    ...overrides,
  };
}

function makeMergeResult(overrides: Partial<MergeResult> = {}): MergeResult {
  return {
    merged_project: {} as any,
    conflicts: [],
    summary: "Merged successfully",
    items_merged: 15,
    items_skipped: 2,
    ...overrides,
  };
}

const projectA = { name: "A" } as any;
const projectB = { name: "B" } as any;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function createHook() {
  let hook!: ReturnType<typeof useProjectComparison>;
  let dispose!: () => void;
  createRoot((d) => {
    dispose = d;
    hook = useProjectComparison();
  });
  return { hook, dispose };
}

// ==========================================================================
// Tests
// ==========================================================================

describe("useProjectComparison", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // -----------------------------------------------------------------------
  // Initial state
  // -----------------------------------------------------------------------
  describe("initial state", () => {
    it("starts with null comparison", () => {
      const { hook, dispose } = createHook();
      expect(hook.comparison()).toBeNull();
      dispose();
    });

    it("starts with null merge result", () => {
      const { hook, dispose } = createHook();
      expect(hook.mergeResult()).toBeNull();
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
  // compareProjects
  // -----------------------------------------------------------------------
  describe("compareProjects", () => {
    it("invokes backend and stores result", async () => {
      const { hook, dispose } = createHook();
      const comp = makeComparison();
      mockInvoke.mockResolvedValueOnce(comp);

      const result = await hook.compareProjects(projectA, projectB);

      expect(mockInvoke).toHaveBeenCalledWith("project_compare", {
        project_a: projectA,
        project_b: projectB,
      });
      expect(result).toBe(comp);
      expect(hook.comparison()).toBe(comp);
      expect(hook.loading()).toBe(false);
      dispose();
    });

    it("sets error on failure", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockRejectedValueOnce(new Error("Compare failed"));

      const result = await hook.compareProjects(projectA, projectB);

      expect(result).toBeNull();
      expect(hook.error()).toBe("Compare failed");
      expect(hook.loading()).toBe(false);
      dispose();
    });

    it("handles non-Error rejection", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockRejectedValueOnce("string error");

      await hook.compareProjects(projectA, projectB);

      expect(hook.error()).toBe("string error");
      dispose();
    });

    it("clears previous error on new call", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockRejectedValueOnce(new Error("first"));
      await hook.compareProjects(projectA, projectB);
      expect(hook.error()).toBe("first");

      mockInvoke.mockResolvedValueOnce(makeComparison());
      await hook.compareProjects(projectA, projectB);
      expect(hook.error()).toBeNull();
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // mergeProjects
  // -----------------------------------------------------------------------
  describe("mergeProjects", () => {
    it("invokes backend with strategy and stores result", async () => {
      const { hook, dispose } = createHook();
      const mr = makeMergeResult();
      mockInvoke.mockResolvedValueOnce(mr);

      const result = await hook.mergeProjects(projectA, projectB, "PreferA");

      expect(mockInvoke).toHaveBeenCalledWith("project_merge", {
        project_a: projectA,
        project_b: projectB,
        strategy: "PreferA",
      });
      expect(result).toBe(mr);
      expect(hook.mergeResult()).toBe(mr);
      dispose();
    });

    it("defaults to Manual strategy", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockResolvedValueOnce(makeMergeResult());

      await hook.mergeProjects(projectA, projectB);

      expect(mockInvoke).toHaveBeenCalledWith("project_merge", expect.objectContaining({
        strategy: "Manual",
      }));
      dispose();
    });

    it("sets error on failure", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockRejectedValueOnce(new Error("Merge failed"));

      const result = await hook.mergeProjects(projectA, projectB, "KeepBoth");

      expect(result).toBeNull();
      expect(hook.error()).toBe("Merge failed");
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // syncBookmarks
  // -----------------------------------------------------------------------
  describe("syncBookmarks", () => {
    it("invokes backend with overwrite flag", async () => {
      const { hook, dispose } = createHook();
      const merged = { name: "merged" } as any;
      mockInvoke.mockResolvedValueOnce(merged);

      const result = await hook.syncBookmarks(projectA, projectB, true);

      expect(mockInvoke).toHaveBeenCalledWith("project_sync_bookmarks", {
        target: projectA,
        source: projectB,
        overwrite: true,
      });
      expect(result).toBe(merged);
      dispose();
    });

    it("defaults overwrite to false", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockResolvedValueOnce({});

      await hook.syncBookmarks(projectA, projectB);

      expect(mockInvoke).toHaveBeenCalledWith("project_sync_bookmarks", {
        target: projectA,
        source: projectB,
        overwrite: false,
      });
      dispose();
    });

    it("sets error on failure", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockRejectedValueOnce(new Error("Sync failed"));

      const result = await hook.syncBookmarks(projectA, projectB);

      expect(result).toBeNull();
      expect(hook.error()).toBe("Sync failed");
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // syncNotes
  // -----------------------------------------------------------------------
  describe("syncNotes", () => {
    it("invokes backend with overwrite flag", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockResolvedValueOnce({});

      await hook.syncNotes(projectA, projectB, true);

      expect(mockInvoke).toHaveBeenCalledWith("project_sync_notes", {
        target: projectA,
        source: projectB,
        overwrite: true,
      });
      dispose();
    });

    it("sets error on failure", async () => {
      const { hook, dispose } = createHook();
      mockInvoke.mockRejectedValueOnce(new Error("Note sync error"));

      const result = await hook.syncNotes(projectA, projectB);

      expect(result).toBeNull();
      expect(hook.error()).toBe("Note sync error");
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Utility getters — no comparison loaded
  // -----------------------------------------------------------------------
  describe("utility getters — null comparison", () => {
    it("getUniqueToA returns empty arrays", () => {
      const { hook, dispose } = createHook();
      expect(hook.getUniqueToA()).toEqual({
        bookmarks: [],
        notes: [],
        evidence: [],
      });
      dispose();
    });

    it("getUniqueToB returns empty arrays", () => {
      const { hook, dispose } = createHook();
      expect(hook.getUniqueToB()).toEqual({
        bookmarks: [],
        notes: [],
        evidence: [],
      });
      dispose();
    });

    it("getCommonItems returns empty arrays", () => {
      const { hook, dispose } = createHook();
      expect(hook.getCommonItems()).toEqual({
        bookmarks: [],
        notes: [],
        evidence: [],
      });
      dispose();
    });

    it("getModifiedItems returns empty arrays", () => {
      const { hook, dispose } = createHook();
      expect(hook.getModifiedItems()).toEqual({
        bookmarks: [],
        notes: [],
      });
      dispose();
    });

    it("getConflicts returns empty array", () => {
      const { hook, dispose } = createHook();
      expect(hook.getConflicts()).toEqual([]);
      dispose();
    });

    it("areSimilar returns false", () => {
      const { hook, dispose } = createHook();
      expect(hook.areSimilar()).toBe(false);
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Utility getters — with comparison loaded
  // -----------------------------------------------------------------------
  describe("utility getters — with comparison", () => {
    async function loadComparison(comp: ProjectComparison) {
      const ctx = createHook();
      mockInvoke.mockResolvedValueOnce(comp);
      await ctx.hook.compareProjects(projectA, projectB);
      return ctx;
    }

    it("getUniqueToA extracts only_in_a from bookmarks, notes, evidence", async () => {
      const { hook, dispose } = await loadComparison(makeComparison());

      expect(hook.getUniqueToA()).toEqual({
        bookmarks: ["bm-a1", "bm-a2"],
        notes: ["note-a"],
        evidence: ["ev-a"],
      });
      dispose();
    });

    it("getUniqueToB extracts only_in_b from bookmarks, notes, evidence", async () => {
      const { hook, dispose } = await loadComparison(makeComparison());

      expect(hook.getUniqueToB()).toEqual({
        bookmarks: ["bm-b1"],
        notes: ["note-b1", "note-b2"],
        evidence: [],
      });
      dispose();
    });

    it("getCommonItems extracts common from bookmarks, notes, evidence", async () => {
      const { hook, dispose } = await loadComparison(makeComparison());

      expect(hook.getCommonItems()).toEqual({
        bookmarks: ["bm-shared"],
        notes: ["note-shared"],
        evidence: ["ev-shared1", "ev-shared2"],
      });
      dispose();
    });

    it("getModifiedItems extracts modified bookmarks and notes", async () => {
      const { hook, dispose } = await loadComparison(makeComparison());

      expect(hook.getModifiedItems()).toEqual({
        bookmarks: ["bm-mod"],
        notes: [],
      });
      dispose();
    });

    it("getConflicts returns conflicts array", async () => {
      const { hook, dispose } = await loadComparison(makeComparison());

      const conflicts = hook.getConflicts();
      expect(conflicts).toHaveLength(1);
      expect(conflicts[0].conflict_type).toBe("bookmark_conflict");
      dispose();
    });

    it("areSimilar returns false when similarity <= 80", async () => {
      const comp = makeComparison();
      comp.summary.similarity_percent = 80;
      const { hook, dispose } = await loadComparison(comp);

      expect(hook.areSimilar()).toBe(false);
      dispose();
    });

    it("areSimilar returns true when similarity > 80", async () => {
      const comp = makeComparison();
      comp.summary.similarity_percent = 81;
      const { hook, dispose } = await loadComparison(comp);

      expect(hook.areSimilar()).toBe(true);
      dispose();
    });

    it("areSimilar returns true for 100% similarity", async () => {
      const comp = makeComparison();
      comp.summary.similarity_percent = 100;
      const { hook, dispose } = await loadComparison(comp);

      expect(hook.areSimilar()).toBe(true);
      dispose();
    });
  });

  // -----------------------------------------------------------------------
  // Merge strategies
  // -----------------------------------------------------------------------
  describe("merge strategies", () => {
    const strategies: MergeStrategy[] = ["PreferA", "PreferB", "KeepBoth", "Skip", "Manual"];

    it.each(strategies)("passes %s strategy to backend", async (strategy) => {
      const { hook, dispose } = createHook();
      mockInvoke.mockResolvedValueOnce(makeMergeResult());

      await hook.mergeProjects(projectA, projectB, strategy);

      expect(mockInvoke).toHaveBeenCalledWith("project_merge", expect.objectContaining({
        strategy,
      }));
      dispose();
    });
  });
});
