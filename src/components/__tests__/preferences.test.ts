// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import {
  DEFAULT_PREFERENCES,
  getPreference,
  getLastPath,
  setLastPath,
  getRecentProjects,
  addRecentProject,
  removeRecentProject,
  clearRecentProjects,
} from "../preferences";

// =============================================================================
// DEFAULT_PREFERENCES
// =============================================================================

describe("DEFAULT_PREFERENCES", () => {
  it("has expected theme default", () => {
    expect(DEFAULT_PREFERENCES.theme).toBe("dark");
  });

  it("has expected accent color", () => {
    expect(DEFAULT_PREFERENCES.accentColor).toBe("cyan");
  });

  it("has expected hash algorithm", () => {
    expect(DEFAULT_PREFERENCES.defaultHashAlgorithm).toBe("SHA-256");
  });

  it("has expected tree density", () => {
    expect(DEFAULT_PREFERENCES.treeDensity).toBe("comfortable");
  });

  it("has sane performance defaults", () => {
    expect(DEFAULT_PREFERENCES.maxConcurrentOperations).toBeGreaterThan(0);
    expect(DEFAULT_PREFERENCES.cacheSizeMb).toBeGreaterThan(0);
    expect(DEFAULT_PREFERENCES.chunkSizeKb).toBeGreaterThan(0);
    expect(DEFAULT_PREFERENCES.workerThreads).toBeGreaterThan(0);
  });

  it("has all shortcut keys defined", () => {
    expect(DEFAULT_PREFERENCES.shortcuts).toBeDefined();
    expect(DEFAULT_PREFERENCES.shortcuts.openFile).toBe("Meta+o");
    expect(DEFAULT_PREFERENCES.shortcuts.save).toBe("Meta+s");
    expect(DEFAULT_PREFERENCES.shortcuts.search).toBe("Meta+f");
    expect(DEFAULT_PREFERENCES.shortcuts.closeModal).toBe("Escape");
  });

  it("has security defaults", () => {
    expect(DEFAULT_PREFERENCES.auditLogging).toBe(true);
    expect(DEFAULT_PREFERENCES.hashVerificationMode).toBe("same-algo");
  });

  it("has report defaults", () => {
    expect(DEFAULT_PREFERENCES.defaultReportTemplate).toBe("standard");
    expect(DEFAULT_PREFERENCES.includeHashesInReports).toBe(true);
    expect(DEFAULT_PREFERENCES.includeTimestampsInReports).toBe(true);
  });
});

// =============================================================================
// localStorage mock helper
// =============================================================================

const STORAGE_KEY = "ffx-preferences";
const LAST_PATH_KEY = "ffx-last-paths";
const RECENT_PROJECTS_KEY = "ffx-recent-projects";

let mockStore: Record<string, string> = {};

function setupLocalStorageMock() {
  mockStore = {};
  vi.stubGlobal("localStorage", {
    getItem: vi.fn((key: string) => mockStore[key] ?? null),
    setItem: vi.fn((key: string, value: string) => {
      mockStore[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete mockStore[key];
    }),
  });
}

// =============================================================================
// getPreference
// =============================================================================

describe("getPreference", () => {
  beforeEach(setupLocalStorageMock);
  afterEach(() => vi.restoreAllMocks());

  it("returns default value when nothing is stored", () => {
    expect(getPreference("theme")).toBe("dark");
    expect(getPreference("fontSize")).toBe(14);
    expect(getPreference("accentColor")).toBe("cyan");
  });

  it("returns stored value when present", () => {
    mockStore[STORAGE_KEY] = JSON.stringify({ theme: "midnight", fontSize: 16 });
    expect(getPreference("theme")).toBe("midnight");
    expect(getPreference("fontSize")).toBe(16);
  });

  it("returns default for missing keys in stored object", () => {
    mockStore[STORAGE_KEY] = JSON.stringify({ theme: "light" });
    expect(getPreference("accentColor")).toBe("cyan"); // not stored, use default
  });

  it("returns default when stored JSON is corrupt", () => {
    mockStore[STORAGE_KEY] = "not-valid-json{{{";
    expect(getPreference("theme")).toBe("dark");
  });
});

// =============================================================================
// getLastPath / setLastPath
// =============================================================================

describe("getLastPath / setLastPath", () => {
  beforeEach(setupLocalStorageMock);
  afterEach(() => vi.restoreAllMocks());

  it("returns undefined when no paths stored and no default", () => {
    // rememberLastPath defaults to true via DEFAULT_PREFERENCES
    // but no stored prefs, getPreference uses default (true)
    expect(getLastPath("project")).toBeUndefined();
    expect(getLastPath("general")).toBeUndefined();
  });

  it("stores and retrieves a path", () => {
    // Enable rememberLastPath
    mockStore[STORAGE_KEY] = JSON.stringify({ rememberLastPath: true });

    setLastPath("evidence", "/path/to/evidence");
    expect(getLastPath("evidence")).toBe("/path/to/evidence");
  });

  it("does not store path when rememberLastPath is disabled", () => {
    mockStore[STORAGE_KEY] = JSON.stringify({ rememberLastPath: false });

    setLastPath("evidence", "/should/not/store");
    // Should not be stored
    expect(mockStore[LAST_PATH_KEY]).toBeUndefined();
  });

  it("returns default evidence path when rememberLastPath is disabled", () => {
    mockStore[STORAGE_KEY] = JSON.stringify({
      rememberLastPath: false,
      defaultEvidencePath: "/default/evidence",
    });

    expect(getLastPath("evidence")).toBe("/default/evidence");
  });

  it("returns default export path when rememberLastPath is disabled", () => {
    mockStore[STORAGE_KEY] = JSON.stringify({
      rememberLastPath: false,
      defaultExportPath: "/default/export",
    });

    expect(getLastPath("export")).toBe("/default/export");
  });
});

// =============================================================================
// Recent Projects
// =============================================================================

describe("getRecentProjects", () => {
  beforeEach(setupLocalStorageMock);
  afterEach(() => vi.restoreAllMocks());

  it("returns empty array when nothing stored", () => {
    expect(getRecentProjects()).toEqual([]);
  });

  it("returns stored projects", () => {
    const projects = [
      { path: "/a.cffx", name: "Project A", lastOpened: "2024-01-01T00:00:00Z" },
      { path: "/b.cffx", name: "Project B", lastOpened: "2024-01-02T00:00:00Z" },
    ];
    mockStore[RECENT_PROJECTS_KEY] = JSON.stringify(projects);
    const result = getRecentProjects();
    expect(result.length).toBe(2);
    expect(result[0].name).toBe("Project A");
  });

  it("limits results to recentFilesCount preference", () => {
    // Default recentFilesCount is 10
    const projects = Array.from({ length: 20 }, (_, i) => ({
      path: `/project-${i}.cffx`,
      name: `Project ${i}`,
      lastOpened: new Date().toISOString(),
    }));
    mockStore[RECENT_PROJECTS_KEY] = JSON.stringify(projects);
    const result = getRecentProjects();
    expect(result.length).toBe(10);
  });

  it("handles corrupt JSON gracefully", () => {
    mockStore[RECENT_PROJECTS_KEY] = "not-json[[[";
    expect(getRecentProjects()).toEqual([]);
  });
});

describe("addRecentProject", () => {
  beforeEach(setupLocalStorageMock);
  afterEach(() => vi.restoreAllMocks());

  it("adds a project to the list", () => {
    addRecentProject("/test.cffx", "Test Project");
    const stored = JSON.parse(mockStore[RECENT_PROJECTS_KEY]);
    expect(stored.length).toBe(1);
    expect(stored[0].path).toBe("/test.cffx");
    expect(stored[0].name).toBe("Test Project");
    expect(stored[0].lastOpened).toBeDefined();
  });

  it("moves duplicate to front and updates timestamp", () => {
    addRecentProject("/a.cffx", "A");
    addRecentProject("/b.cffx", "B");
    addRecentProject("/a.cffx", "A Updated");

    const stored = JSON.parse(mockStore[RECENT_PROJECTS_KEY]);
    expect(stored.length).toBe(2);
    expect(stored[0].path).toBe("/a.cffx");
    expect(stored[0].name).toBe("A Updated");
    expect(stored[1].path).toBe("/b.cffx");
  });

  it("trims list to recentFilesCount", () => {
    // Default is 10
    for (let i = 0; i < 15; i++) {
      addRecentProject(`/project-${i}.cffx`, `Project ${i}`);
    }
    const stored = JSON.parse(mockStore[RECENT_PROJECTS_KEY]);
    expect(stored.length).toBe(10);
    // Most recent should be first
    expect(stored[0].path).toBe("/project-14.cffx");
  });
});

describe("removeRecentProject", () => {
  beforeEach(setupLocalStorageMock);
  afterEach(() => vi.restoreAllMocks());

  it("removes a project by path", () => {
    addRecentProject("/a.cffx", "A");
    addRecentProject("/b.cffx", "B");
    removeRecentProject("/a.cffx");

    const stored = JSON.parse(mockStore[RECENT_PROJECTS_KEY]);
    expect(stored.length).toBe(1);
    expect(stored[0].path).toBe("/b.cffx");
  });

  it("does nothing if path not found", () => {
    addRecentProject("/a.cffx", "A");
    removeRecentProject("/nonexistent.cffx");

    const stored = JSON.parse(mockStore[RECENT_PROJECTS_KEY]);
    expect(stored.length).toBe(1);
  });
});

describe("clearRecentProjects", () => {
  beforeEach(setupLocalStorageMock);
  afterEach(() => vi.restoreAllMocks());

  it("removes all recent projects", () => {
    addRecentProject("/a.cffx", "A");
    addRecentProject("/b.cffx", "B");
    clearRecentProjects();

    expect(mockStore[RECENT_PROJECTS_KEY]).toBeUndefined();
    expect(getRecentProjects()).toEqual([]);
  });
});
