// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { createSignal } from "solid-js";
import { createNoteManager } from "../useNotes";
import type { FFXProject, ProjectNote } from "../../../types/project";
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

describe("createNoteManager", () => {
  let markModified: ReturnType<typeof vi.fn>;
  let logger: ActivityLogger;

  beforeEach(() => {
    markModified = vi.fn();
    logger = { logActivity: vi.fn() };
  });

  // -------------------------------------------------------------------------
  // addNote
  // -------------------------------------------------------------------------
  describe("addNote", () => {
    it("adds a note to the project", () => {
      const project = makeProject();
      const { signals, setters, getProject } = createTestState(project);
      const mgr = createNoteManager(signals, setters, markModified, logger);

      mgr.addNote({
        target_type: "file",
        target_path: "/evidence/image.jpg",
        title: "Image Analysis",
        content: "This image contains relevant metadata.",
      });

      const updated = getProject();
      expect(updated?.notes).toHaveLength(1);
      expect(updated?.notes[0].title).toBe("Image Analysis");
      expect(updated?.notes[0].content).toBe("This image contains relevant metadata.");
      expect(updated?.notes[0].target_path).toBe("/evidence/image.jpg");
      expect(updated?.notes[0].created_by).toBe("testuser");
      expect(updated?.notes[0].id).toBeTruthy();
      expect(updated?.notes[0].created_at).toBeTruthy();
      expect(updated?.notes[0].modified_at).toBeTruthy();
    });

    it("sets created_at and modified_at to the same value on creation", () => {
      const project = makeProject();
      const { signals, setters, getProject } = createTestState(project);
      const mgr = createNoteManager(signals, setters, markModified, logger);

      mgr.addNote({
        target_type: "general",
        title: "Test",
        content: "Content",
      });

      const note = getProject()?.notes[0];
      expect(note?.created_at).toBe(note?.modified_at);
    });

    it("calls markModified after adding", () => {
      const project = makeProject();
      const { signals, setters } = createTestState(project);
      const mgr = createNoteManager(signals, setters, markModified, logger);

      mgr.addNote({
        target_type: "general",
        title: "Test",
        content: "Content",
      });

      expect(markModified).toHaveBeenCalledOnce();
    });

    it("logs the activity", () => {
      const project = makeProject();
      const { signals, setters } = createTestState(project);
      const mgr = createNoteManager(signals, setters, markModified, logger);

      mgr.addNote({
        target_type: "file",
        target_path: "/evidence/file.txt",
        title: "My Note",
        content: "Content here",
      });

      expect(logger.logActivity).toHaveBeenCalledWith(
        "note",
        "add",
        "Added note: My Note",
        "/evidence/file.txt"
      );
    });

    it("does nothing when no project is loaded", () => {
      const { signals, setters } = createTestState(null);
      const mgr = createNoteManager(signals, setters, markModified, logger);

      mgr.addNote({
        target_type: "general",
        title: "Test",
        content: "Content",
      });

      expect(markModified).not.toHaveBeenCalled();
    });

    it("preserves existing notes", () => {
      const existing: ProjectNote = {
        id: "existing-1",
        target_type: "general",
        title: "Old Note",
        content: "Old content",
        created_by: "user1",
        created_at: "2025-01-01T00:00:00Z",
        modified_at: "2025-01-01T00:00:00Z",
      };
      const project = makeProject({ notes: [existing] });
      const { signals, setters, getProject } = createTestState(project);
      const mgr = createNoteManager(signals, setters, markModified, logger);

      mgr.addNote({
        target_type: "file",
        target_path: "/new",
        title: "New Note",
        content: "New content",
      });

      expect(getProject()?.notes).toHaveLength(2);
      expect(getProject()?.notes[0].title).toBe("Old Note");
      expect(getProject()?.notes[1].title).toBe("New Note");
    });

    it("supports optional tags and priority", () => {
      const project = makeProject();
      const { signals, setters, getProject } = createTestState(project);
      const mgr = createNoteManager(signals, setters, markModified, logger);

      mgr.addNote({
        target_type: "case",
        title: "Critical Finding",
        content: "Urgent",
        tags: ["urgent", "reviewed"],
        priority: "critical",
      });

      const note = getProject()?.notes[0];
      expect(note?.tags).toEqual(["urgent", "reviewed"]);
      expect(note?.priority).toBe("critical");
    });
  });

  // -------------------------------------------------------------------------
  // updateNote
  // -------------------------------------------------------------------------
  describe("updateNote", () => {
    it("updates a note's title", () => {
      const note: ProjectNote = {
        id: "note-1",
        target_type: "general",
        title: "Original Title",
        content: "Content",
        created_by: "user1",
        created_at: "2025-01-01T00:00:00Z",
        modified_at: "2025-01-01T00:00:00Z",
      };
      const project = makeProject({ notes: [note] });
      const { signals, setters, getProject } = createTestState(project);
      const mgr = createNoteManager(signals, setters, markModified, logger);

      mgr.updateNote("note-1", { title: "Updated Title" });

      expect(getProject()?.notes[0].title).toBe("Updated Title");
      expect(markModified).toHaveBeenCalledOnce();
    });

    it("updates modified_at timestamp on update", () => {
      const note: ProjectNote = {
        id: "note-1",
        target_type: "general",
        title: "Test",
        content: "Content",
        created_by: "user1",
        created_at: "2025-01-01T00:00:00Z",
        modified_at: "2025-01-01T00:00:00Z",
      };
      const project = makeProject({ notes: [note] });
      const { signals, setters, getProject } = createTestState(project);
      const mgr = createNoteManager(signals, setters, markModified, logger);

      mgr.updateNote("note-1", { content: "Updated content" });

      const updated = getProject()?.notes[0];
      expect(updated?.modified_at).not.toBe("2025-01-01T00:00:00Z");
    });

    it("updates content and tags simultaneously", () => {
      const note: ProjectNote = {
        id: "note-1",
        target_type: "general",
        title: "Test",
        content: "Original",
        created_by: "user1",
        created_at: "2025-01-01T00:00:00Z",
        modified_at: "2025-01-01T00:00:00Z",
      };
      const project = makeProject({ notes: [note] });
      const { signals, setters, getProject } = createTestState(project);
      const mgr = createNoteManager(signals, setters, markModified, logger);

      mgr.updateNote("note-1", {
        content: "New content",
        tags: ["tag1", "tag2"],
      });

      const updated = getProject()?.notes[0];
      expect(updated?.content).toBe("New content");
      expect(updated?.tags).toEqual(["tag1", "tag2"]);
    });

    it("does nothing when no project is loaded", () => {
      const { signals, setters } = createTestState(null);
      const mgr = createNoteManager(signals, setters, markModified, logger);

      mgr.updateNote("id", { title: "Test" });

      expect(markModified).not.toHaveBeenCalled();
    });

    it("logs the update activity", () => {
      const note: ProjectNote = {
        id: "note-42",
        target_type: "general",
        title: "Test",
        content: "Content",
        created_by: "user1",
        created_at: "2025-01-01T00:00:00Z",
        modified_at: "2025-01-01T00:00:00Z",
      };
      const project = makeProject({ notes: [note] });
      const { signals, setters } = createTestState(project);
      const mgr = createNoteManager(signals, setters, markModified, logger);

      mgr.updateNote("note-42", { priority: "high" });

      expect(logger.logActivity).toHaveBeenCalledWith(
        "note",
        "update",
        "Updated note: note-42"
      );
    });

    it("does not affect other notes", () => {
      const note1: ProjectNote = {
        id: "n1",
        target_type: "general",
        title: "Note 1",
        content: "Content 1",
        created_by: "user1",
        created_at: "2025-01-01T00:00:00Z",
        modified_at: "2025-01-01T00:00:00Z",
      };
      const note2: ProjectNote = {
        id: "n2",
        target_type: "general",
        title: "Note 2",
        content: "Content 2",
        created_by: "user1",
        created_at: "2025-01-01T00:00:00Z",
        modified_at: "2025-01-01T00:00:00Z",
      };
      const project = makeProject({ notes: [note1, note2] });
      const { signals, setters, getProject } = createTestState(project);
      const mgr = createNoteManager(signals, setters, markModified, logger);

      mgr.updateNote("n1", { title: "Updated" });

      expect(getProject()?.notes[1].title).toBe("Note 2");
      expect(getProject()?.notes[1].content).toBe("Content 2");
    });
  });
});
