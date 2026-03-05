// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";
import { createSignal } from "solid-js";
import { createBookmarkManager } from "../useBookmarks";
import type { FFXProject, ProjectBookmark } from "../../../types/project";
import type { ProjectStateSignals, ProjectStateSetters, ActivityLogger } from "../types";

/** Build minimal project state signals and setters for testing */
function createTestState(project: FFXProject | null = null) {
  const [proj, setProj] = createSignal<FFXProject | null>(project);
  const [projectPath, setProjectPath] = createSignal<string | null>(null);
  const [modified, setModified] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [currentUser, setCurrentUser] = createSignal("testuser");
  const [currentSessionId, setCurrentSessionId] = createSignal<string | null>(null);
  const [autoSaveEnabled, setAutoSaveEnabled] = createSignal(false);
  const [lastAutoSave, setLastAutoSave] = createSignal<Date | null>(null);

  const signals: ProjectStateSignals = {
    project: proj,
    projectPath,
    modified,
    error,
    loading,
    currentUser,
    currentSessionId,
    autoSaveEnabled,
    lastAutoSave,
  };

  const setters: ProjectStateSetters = {
    setProject: setProj as any,
    setProjectPath,
    setModified,
    setError,
    setLoading,
    setCurrentUser,
    setCurrentSessionId,
    setAutoSaveEnabled,
    setLastAutoSave,
  };

  return { signals, setters, getProject: proj };
}

/** Create a minimal FFXProject for testing */
function makeProject(overrides: Partial<FFXProject> = {}): FFXProject {
  return {
    name: "Test Project",
    version: "1.0.0",
    created_at: "2025-01-01T00:00:00Z",
    modified_at: "2025-01-01T00:00:00Z",
    bookmarks: [],
    notes: [],
    activity_log: [],
    recent_searches: [],
    sessions: [],
    users: [],
    evidence_items: [],
    metadata: {},
    settings: { max_recent_items: 50 },
    ui_state: {},
    processed_databases: { integrity: {} },
    locations: {},
    ...overrides,
  } as FFXProject;
}

describe("createBookmarkManager", () => {
  let markModified: Mock<() => void>;
  let logger: ActivityLogger;

  beforeEach(() => {
    markModified = vi.fn<() => void>();
    logger = { logActivity: vi.fn<ActivityLogger["logActivity"]>() };
  });

  // -------------------------------------------------------------------------
  // addBookmark
  // -------------------------------------------------------------------------
  describe("addBookmark", () => {
    it("adds a bookmark to the project", () => {
      const project = makeProject();
      const { signals, setters, getProject } = createTestState(project);
      const mgr = createBookmarkManager(signals, setters, markModified, logger);

      mgr.addBookmark({
        target_type: "file",
        target_path: "/evidence/image.jpg",
        name: "Important Image",
      });

      const updated = getProject();
      expect(updated?.bookmarks).toHaveLength(1);
      expect(updated?.bookmarks[0].name).toBe("Important Image");
      expect(updated?.bookmarks[0].target_path).toBe("/evidence/image.jpg");
      expect(updated?.bookmarks[0].target_type).toBe("file");
      expect(updated?.bookmarks[0].created_by).toBe("testuser");
      expect(updated?.bookmarks[0].id).toBeTruthy();
      expect(updated?.bookmarks[0].created_at).toBeTruthy();
    });

    it("calls markModified after adding", () => {
      const project = makeProject();
      const { signals, setters } = createTestState(project);
      const mgr = createBookmarkManager(signals, setters, markModified, logger);

      mgr.addBookmark({
        target_type: "file",
        target_path: "/test",
        name: "Test",
      });

      expect(markModified).toHaveBeenCalledOnce();
    });

    it("logs the activity", () => {
      const project = makeProject();
      const { signals, setters } = createTestState(project);
      const mgr = createBookmarkManager(signals, setters, markModified, logger);

      mgr.addBookmark({
        target_type: "file",
        target_path: "/evidence/file.txt",
        name: "My Bookmark",
      });

      expect(logger.logActivity).toHaveBeenCalledWith(
        "bookmark",
        "add",
        "Added bookmark: My Bookmark",
        "/evidence/file.txt"
      );
    });

    it("does nothing when no project is loaded", () => {
      const { signals, setters } = createTestState(null);
      const mgr = createBookmarkManager(signals, setters, markModified, logger);

      mgr.addBookmark({
        target_type: "file",
        target_path: "/test",
        name: "Test",
      });

      expect(markModified).not.toHaveBeenCalled();
    });

    it("preserves existing bookmarks", () => {
      const existing: ProjectBookmark = {
        id: "existing-1",
        target_type: "file",
        target_path: "/old",
        name: "Old Bookmark",
        created_by: "user1",
        created_at: "2025-01-01T00:00:00Z",
      };
      const project = makeProject({ bookmarks: [existing] });
      const { signals, setters, getProject } = createTestState(project);
      const mgr = createBookmarkManager(signals, setters, markModified, logger);

      mgr.addBookmark({
        target_type: "artifact",
        target_path: "/new",
        name: "New Bookmark",
      });

      expect(getProject()?.bookmarks).toHaveLength(2);
      expect(getProject()?.bookmarks[0].name).toBe("Old Bookmark");
      expect(getProject()?.bookmarks[1].name).toBe("New Bookmark");
    });
  });

  // -------------------------------------------------------------------------
  // updateBookmark
  // -------------------------------------------------------------------------
  describe("updateBookmark", () => {
    it("updates a bookmark's name", () => {
      const bookmark: ProjectBookmark = {
        id: "bk-1",
        target_type: "file",
        target_path: "/file.txt",
        name: "Original Name",
        created_by: "user1",
        created_at: "2025-01-01T00:00:00Z",
      };
      const project = makeProject({ bookmarks: [bookmark] });
      const { signals, setters, getProject } = createTestState(project);
      const mgr = createBookmarkManager(signals, setters, markModified, logger);

      mgr.updateBookmark("bk-1", { name: "Updated Name" });

      expect(getProject()?.bookmarks[0].name).toBe("Updated Name");
      expect(markModified).toHaveBeenCalledOnce();
    });

    it("updates tags and color", () => {
      const bookmark: ProjectBookmark = {
        id: "bk-1",
        target_type: "file",
        target_path: "/file.txt",
        name: "Test",
        created_by: "user1",
        created_at: "2025-01-01T00:00:00Z",
      };
      const project = makeProject({ bookmarks: [bookmark] });
      const { signals, setters, getProject } = createTestState(project);
      const mgr = createBookmarkManager(signals, setters, markModified, logger);

      mgr.updateBookmark("bk-1", { color: "red", tags: ["important"] });

      expect(getProject()?.bookmarks[0].color).toBe("red");
      expect(getProject()?.bookmarks[0].tags).toEqual(["important"]);
    });

    it("does nothing for non-existent bookmark ID", () => {
      const project = makeProject({ bookmarks: [] });
      const { signals, setters } = createTestState(project);
      const mgr = createBookmarkManager(signals, setters, markModified, logger);

      mgr.updateBookmark("nonexistent", { name: "Won't Work" });

      expect(markModified).not.toHaveBeenCalled();
    });

    it("does nothing when no project is loaded", () => {
      const { signals, setters } = createTestState(null);
      const mgr = createBookmarkManager(signals, setters, markModified, logger);

      mgr.updateBookmark("id", { name: "Test" });

      expect(markModified).not.toHaveBeenCalled();
    });
  });

  // -------------------------------------------------------------------------
  // removeBookmark
  // -------------------------------------------------------------------------
  describe("removeBookmark", () => {
    it("removes a bookmark by ID", () => {
      const bookmark: ProjectBookmark = {
        id: "bk-remove",
        target_type: "file",
        target_path: "/to-remove.txt",
        name: "Remove Me",
        created_by: "user1",
        created_at: "2025-01-01T00:00:00Z",
      };
      const project = makeProject({ bookmarks: [bookmark] });
      const { signals, setters, getProject } = createTestState(project);
      const mgr = createBookmarkManager(signals, setters, markModified, logger);

      mgr.removeBookmark("bk-remove");

      expect(getProject()?.bookmarks).toHaveLength(0);
      expect(markModified).toHaveBeenCalledOnce();
    });

    it("logs removal activity with bookmark name and path", () => {
      const bookmark: ProjectBookmark = {
        id: "bk-1",
        target_type: "file",
        target_path: "/file.txt",
        name: "My Bookmark",
        created_by: "user1",
        created_at: "2025-01-01T00:00:00Z",
      };
      const project = makeProject({ bookmarks: [bookmark] });
      const { signals, setters } = createTestState(project);
      const mgr = createBookmarkManager(signals, setters, markModified, logger);

      mgr.removeBookmark("bk-1");

      expect(logger.logActivity).toHaveBeenCalledWith(
        "bookmark",
        "remove",
        "Removed bookmark: My Bookmark",
        "/file.txt"
      );
    });

    it("does nothing when no project is loaded", () => {
      const { signals, setters } = createTestState(null);
      const mgr = createBookmarkManager(signals, setters, markModified, logger);

      mgr.removeBookmark("any-id");

      expect(markModified).not.toHaveBeenCalled();
    });

    it("keeps other bookmarks intact", () => {
      const bk1: ProjectBookmark = {
        id: "keep",
        target_type: "file",
        target_path: "/keep.txt",
        name: "Keep",
        created_by: "user1",
        created_at: "2025-01-01T00:00:00Z",
      };
      const bk2: ProjectBookmark = {
        id: "remove",
        target_type: "file",
        target_path: "/remove.txt",
        name: "Remove",
        created_by: "user1",
        created_at: "2025-01-01T00:00:00Z",
      };
      const project = makeProject({ bookmarks: [bk1, bk2] });
      const { signals, setters, getProject } = createTestState(project);
      const mgr = createBookmarkManager(signals, setters, markModified, logger);

      mgr.removeBookmark("remove");

      expect(getProject()?.bookmarks).toHaveLength(1);
      expect(getProject()?.bookmarks[0].id).toBe("keep");
    });
  });
});
