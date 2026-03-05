// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";
import { createSignal } from "solid-js";
import {
  createSearchHistoryManager,
  createProcessedDbManager,
  createProjectLocationsManager,
} from "../useProjectHelpers";
import type { FFXProject, ProcessedDbIntegrity } from "../../../types/project";
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

// =============================================================================
// createSearchHistoryManager
// =============================================================================
describe("createSearchHistoryManager", () => {
  let markModified: Mock<() => void>;
  let logger: ActivityLogger;

  beforeEach(() => {
    markModified = vi.fn<() => void>();
    logger = { logActivity: vi.fn<ActivityLogger["logActivity"]>() };
  });

  it("adds a recent search to the project", () => {
    const project = makeProject();
    const { signals, setters, getProject } = createTestState(project);
    const mgr = createSearchHistoryManager(signals, setters, markModified, logger);

    mgr.addRecentSearch("malware.exe", 5);

    const updated = getProject();
    expect(updated?.recent_searches).toHaveLength(1);
    expect(updated?.recent_searches[0].query).toBe("malware.exe");
    expect(updated?.recent_searches[0].result_count).toBe(5);
    expect(updated?.recent_searches[0].timestamp).toBeTruthy();
  });

  it("calls markModified after adding search", () => {
    const project = makeProject();
    const { signals, setters } = createTestState(project);
    const mgr = createSearchHistoryManager(signals, setters, markModified, logger);

    mgr.addRecentSearch("test", 0);

    expect(markModified).toHaveBeenCalledOnce();
  });

  it("logs the search activity", () => {
    const project = makeProject();
    const { signals, setters } = createTestState(project);
    const mgr = createSearchHistoryManager(signals, setters, markModified, logger);

    mgr.addRecentSearch("password.txt", 12);

    expect(logger.logActivity).toHaveBeenCalledWith(
      "search",
      "perform",
      'Searched: "password.txt" (12 results)'
    );
  });

  it("deduplicates searches by query", () => {
    const project = makeProject({
      recent_searches: [
        { query: "same-query", timestamp: "2025-01-01T00:00:00Z", result_count: 3 },
      ],
    });
    const { signals, setters, getProject } = createTestState(project);
    const mgr = createSearchHistoryManager(signals, setters, markModified, logger);

    mgr.addRecentSearch("same-query", 10);

    const updated = getProject();
    expect(updated?.recent_searches).toHaveLength(1);
    expect(updated?.recent_searches[0].result_count).toBe(10); // newer entry wins
  });

  it("puts most recent search first", () => {
    const project = makeProject({
      recent_searches: [
        { query: "older", timestamp: "2025-01-01T00:00:00Z", result_count: 1 },
      ],
    });
    const { signals, setters, getProject } = createTestState(project);
    const mgr = createSearchHistoryManager(signals, setters, markModified, logger);

    mgr.addRecentSearch("newer", 2);

    const searches = getProject()?.recent_searches || [];
    expect(searches[0].query).toBe("newer");
    expect(searches[1].query).toBe("older");
  });

  it("respects max_recent_items limit", () => {
    const existingSearches = Array.from({ length: 50 }, (_, i) => ({
      query: `search-${i}`,
      timestamp: "2025-01-01T00:00:00Z",
      result_count: i,
    }));
    const project = makeProject({
      recent_searches: existingSearches,
      settings: { max_recent_items: 50 } as any,
    });
    const { signals, setters, getProject } = createTestState(project);
    const mgr = createSearchHistoryManager(signals, setters, markModified, logger);

    mgr.addRecentSearch("new-search", 99);

    const searches = getProject()?.recent_searches || [];
    expect(searches.length).toBeLessThanOrEqual(50);
    expect(searches[0].query).toBe("new-search");
  });

  it("does nothing when no project is loaded", () => {
    const { signals, setters } = createTestState(null);
    const mgr = createSearchHistoryManager(signals, setters, markModified, logger);

    mgr.addRecentSearch("test", 0);

    expect(markModified).not.toHaveBeenCalled();
  });
});

// =============================================================================
// createProcessedDbManager
// =============================================================================
describe("createProcessedDbManager", () => {
  let markModified: Mock<() => void>;
  let logger: ActivityLogger;

  beforeEach(() => {
    markModified = vi.fn<() => void>();
    logger = { logActivity: vi.fn<ActivityLogger["logActivity"]>() };
  });

  it("updates processed database integrity", () => {
    const project = makeProject();
    const { signals, setters, getProject } = createTestState(project);
    const mgr = createProcessedDbManager(signals, setters, markModified, logger);

    const integrity: ProcessedDbIntegrity = {
      path: "/path/to/db.sqlite",
      file_size: 1024,
      baseline_hash: "abc123",
      baseline_timestamp: "2025-01-01T00:00:00Z",
      status: "unchanged",
    };

    mgr.updateProcessedDbIntegrity("/path/to/db.sqlite", integrity);

    const updated = getProject();
    const stored = (updated?.processed_databases as any)?.integrity?.["/path/to/db.sqlite"];
    expect(stored).toBeDefined();
    expect(stored.status).toBe("unchanged");
  });

  it("calls markModified after update", () => {
    const project = makeProject();
    const { signals, setters } = createTestState(project);
    const mgr = createProcessedDbManager(signals, setters, markModified, logger);

    const integrity: ProcessedDbIntegrity = {
      path: "/db.sqlite",
      file_size: 2048,
      baseline_hash: "def456",
      baseline_timestamp: "2025-06-01T00:00:00Z",
      status: "unchanged",
    };
    mgr.updateProcessedDbIntegrity("/db.sqlite", integrity);

    expect(markModified).toHaveBeenCalledOnce();
  });

  it("logs the verification activity", () => {
    const project = makeProject();
    const { signals, setters } = createTestState(project);
    const mgr = createProcessedDbManager(signals, setters, markModified, logger);

    const integrity: ProcessedDbIntegrity = {
      path: "/db.sqlite",
      file_size: 4096,
      baseline_hash: "ghi789",
      baseline_timestamp: "2025-06-01T00:00:00Z",
      status: "modified",
      changes: ["table_added", "row_modified", "row_deleted"],
    };
    mgr.updateProcessedDbIntegrity("/db.sqlite", integrity);

    expect(logger.logActivity).toHaveBeenCalledWith(
      "database",
      "verify",
      "Updated integrity for: /db.sqlite",
      "/db.sqlite",
      expect.objectContaining({ status: "modified", changes: ["table_added", "row_modified", "row_deleted"] })
    );
  });

  it("does nothing when no project is loaded", () => {
    const { signals, setters } = createTestState(null);
    const mgr = createProcessedDbManager(signals, setters, markModified, logger);

    const integrity: ProcessedDbIntegrity = {
      path: "/db.sqlite",
      file_size: 1024,
      baseline_hash: "abc",
      baseline_timestamp: "2025-01-01T00:00:00Z",
      status: "not_verified",
    };
    mgr.updateProcessedDbIntegrity("/db.sqlite", integrity);

    expect(markModified).not.toHaveBeenCalled();
  });
});

// =============================================================================
// createProjectLocationsManager
// =============================================================================
describe("createProjectLocationsManager", () => {
  let markModified: Mock<() => void>;
  let logger: ActivityLogger;

  beforeEach(() => {
    markModified = vi.fn<() => void>();
    logger = { logActivity: vi.fn<ActivityLogger["logActivity"]>() };
  });

  it("updates project locations", () => {
    const project = makeProject();
    const { signals, setters, getProject } = createTestState(project);
    const mgr = createProjectLocationsManager(signals, setters, markModified, logger);

    mgr.updateLocations({
      evidence_path: "/evidence",
      processed_db_path: "/processed",
      case_documents_path: "/docs",
    } as any);

    const updated = getProject();
    expect((updated?.locations as any)?.evidence_path).toBe("/evidence");
    expect((updated?.locations as any)?.processed_db_path).toBe("/processed");
    expect((updated?.locations as any)?.case_documents_path).toBe("/docs");
  });

  it("calls markModified after update", () => {
    const project = makeProject();
    const { signals, setters } = createTestState(project);
    const mgr = createProjectLocationsManager(signals, setters, markModified, logger);

    mgr.updateLocations({ evidence_path: "/test" } as any);

    expect(markModified).toHaveBeenCalledOnce();
  });

  it("logs the locations update activity", () => {
    const project = makeProject();
    const { signals, setters } = createTestState(project);
    const mgr = createProjectLocationsManager(signals, setters, markModified, logger);

    mgr.updateLocations({
      evidence_path: "/my-evidence",
      processed_db_path: "/my-db",
      case_documents_path: "/my-docs",
    } as any);

    expect(logger.logActivity).toHaveBeenCalledWith(
      "project",
      "update",
      "Updated project locations: evidence=/my-evidence",
      undefined,
      expect.objectContaining({
        evidence_path: "/my-evidence",
        processed_db_path: "/my-db",
        case_documents_path: "/my-docs",
      })
    );
  });

  it("does nothing when no project is loaded", () => {
    const { signals, setters } = createTestState(null);
    const mgr = createProjectLocationsManager(signals, setters, markModified, logger);

    mgr.updateLocations({ evidence_path: "/test" } as any);

    expect(markModified).not.toHaveBeenCalled();
  });
});
