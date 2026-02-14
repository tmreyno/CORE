// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  generateId,
  nowISO,
  createDefaultUIState,
  createDefaultFilterState,
  createDefaultSettings,
  createEmptyProject,
  createActivityEntry,
  PROJECT_FILE_VERSION,
  AUTO_SAVE_INTERVAL_MS,
} from "../project";

// =============================================================================
// generateId
// =============================================================================
describe("generateId", () => {
  it("returns a non-empty string", () => {
    const id = generateId();
    expect(id).toBeTruthy();
    expect(typeof id).toBe("string");
  });

  it("contains a hyphen separator", () => {
    const id = generateId();
    expect(id).toContain("-");
  });

  it("generates unique IDs", () => {
    const ids = new Set(Array.from({ length: 100 }, () => generateId()));
    expect(ids.size).toBe(100);
  });
});

// =============================================================================
// nowISO
// =============================================================================
describe("nowISO", () => {
  it("returns a valid ISO 8601 string", () => {
    const iso = nowISO();
    expect(iso).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/);
    // Parsing back should not produce NaN
    expect(new Date(iso).getTime()).not.toBeNaN();
  });

  it("returns current time (within 1 second)", () => {
    const before = Date.now();
    const iso = nowISO();
    const after = Date.now();
    const ts = new Date(iso).getTime();
    expect(ts).toBeGreaterThanOrEqual(before);
    expect(ts).toBeLessThanOrEqual(after);
  });
});

// =============================================================================
// createDefaultUIState
// =============================================================================
describe("createDefaultUIState", () => {
  it("returns expected default values", () => {
    const state = createDefaultUIState();
    expect(state.left_panel_width).toBe(320);
    expect(state.right_panel_width).toBe(280);
    expect(state.left_panel_collapsed).toBe(false);
    expect(state.right_panel_collapsed).toBe(true);
    expect(state.left_panel_tab).toBe("evidence");
    expect(state.detail_view_mode).toBe("info");
    expect(state.entry_content_view_mode).toBe("auto");
    expect(state.selected_entry).toBeNull();
  });

  it("has empty arrays for tree_state and expanded_containers", () => {
    const state = createDefaultUIState();
    expect(state.tree_state).toEqual([]);
    expect(state.expanded_containers).toEqual([]);
  });

  it("has default preferences", () => {
    const state = createDefaultUIState();
    expect(state.preferences).toEqual({
      theme: "auto",
      confirm_on_close: true,
    });
  });

  it("returns a new object each call (no shared references)", () => {
    const a = createDefaultUIState();
    const b = createDefaultUIState();
    expect(a).not.toBe(b);
    expect(a).toEqual(b);
  });
});

// =============================================================================
// createDefaultFilterState
// =============================================================================
describe("createDefaultFilterState", () => {
  it("returns expected default values", () => {
    const state = createDefaultFilterState();
    expect(state.type_filter).toBeNull();
    expect(state.status_filter).toBeNull();
    expect(state.search_query).toBeNull();
    expect(state.sort_by).toBe("name");
    expect(state.sort_direction).toBe("asc");
  });

  it("returns a new object each call", () => {
    const a = createDefaultFilterState();
    const b = createDefaultFilterState();
    expect(a).not.toBe(b);
    expect(a).toEqual(b);
  });
});

// =============================================================================
// createDefaultSettings
// =============================================================================
describe("createDefaultSettings", () => {
  it("returns expected default values", () => {
    const settings = createDefaultSettings();
    expect(settings.auto_save).toBe(true);
    expect(settings.auto_save_interval).toBe(AUTO_SAVE_INTERVAL_MS);
    expect(settings.default_hash_algorithm).toBe("SHA-256");
    expect(settings.verify_hashes_on_load).toBe(false);
    expect(settings.track_activity).toBe(true);
    expect(settings.max_recent_items).toBe(50);
  });

  it("auto_save_interval is 5 minutes in ms", () => {
    expect(AUTO_SAVE_INTERVAL_MS).toBe(5 * 60 * 1000);
  });
});

// =============================================================================
// createEmptyProject
// =============================================================================
describe("createEmptyProject", () => {
  const rootPath = "/evidence/case-2024";
  const username = "examiner1";
  const appVersion = "1.0.0";

  it("creates project with correct metadata", () => {
    const project = createEmptyProject(rootPath, username, appVersion);
    expect(project.version).toBe(PROJECT_FILE_VERSION);
    expect(project.root_path).toBe(rootPath);
    expect(project.current_user).toBe(username);
    expect(project.created_by_version).toBe(appVersion);
    expect(project.saved_by_version).toBe(appVersion);
  });

  it("uses custom project name when provided", () => {
    const project = createEmptyProject(rootPath, username, appVersion, "My Case");
    expect(project.name).toBe("My Case");
  });

  it("derives name from root path when not provided", () => {
    const project = createEmptyProject(rootPath, username, appVersion);
    expect(project.name).toBe("case-2024");
  });

  it("falls back to 'Untitled Project' for empty path", () => {
    const project = createEmptyProject("", username, appVersion);
    expect(project.name).toBe("Untitled Project");
  });

  it("has unique project_id and session_id", () => {
    const project = createEmptyProject(rootPath, username, appVersion);
    expect(project.project_id).toBeTruthy();
    expect(project.current_session_id).toBeTruthy();
    expect(project.project_id).not.toBe(project.current_session_id);
  });

  it("creates initial user entry", () => {
    const project = createEmptyProject(rootPath, username, appVersion);
    expect(project.users).toHaveLength(1);
    expect(project.users[0].username).toBe(username);
  });

  it("creates initial session", () => {
    const project = createEmptyProject(rootPath, username, appVersion);
    expect(project.sessions).toHaveLength(1);
    expect(project.sessions[0].user).toBe(username);
    expect(project.sessions[0].app_version).toBe(appVersion);
    expect(project.sessions[0].ended_at).toBeNull();
  });

  it("creates initial activity log entry", () => {
    const project = createEmptyProject(rootPath, username, appVersion, "Test Case");
    expect(project.activity_log).toHaveLength(1);
    expect(project.activity_log[0].category).toBe("project");
    expect(project.activity_log[0].action).toBe("create");
    expect(project.activity_log[0].description).toContain("Test Case");
  });

  it("initializes empty collections", () => {
    const project = createEmptyProject(rootPath, username, appVersion);
    expect(project.tabs).toEqual([]);
    expect(project.bookmarks).toEqual([]);
    expect(project.notes).toEqual([]);
    expect(project.reports).toEqual([]);
    expect(project.saved_searches).toEqual([]);
    expect(project.open_directories).toEqual([]);
  });

  it("includes default filter state, UI state, and settings", () => {
    const project = createEmptyProject(rootPath, username, appVersion);
    expect(project.filter_state).toEqual(createDefaultFilterState());
    expect(project.ui_state).toEqual(createDefaultUIState());
    expect(project.settings).toEqual(createDefaultSettings());
  });

  it("has empty hash_history", () => {
    const project = createEmptyProject(rootPath, username, appVersion);
    expect(project.hash_history).toEqual({ files: {} });
  });
});

// =============================================================================
// createActivityEntry
// =============================================================================
describe("createActivityEntry", () => {
  it("creates entry with required fields", () => {
    const entry = createActivityEntry("user1", "hash", "verify", "Verified file.E01");
    expect(entry.user).toBe("user1");
    expect(entry.category).toBe("hash");
    expect(entry.action).toBe("verify");
    expect(entry.description).toBe("Verified file.E01");
    expect(entry.id).toBeTruthy();
    expect(entry.timestamp).toBeTruthy();
  });

  it("includes optional file_path", () => {
    const entry = createActivityEntry("user1", "file", "open", "Opened file", "/path/to/file.ad1");
    expect(entry.file_path).toBe("/path/to/file.ad1");
  });

  it("includes optional details", () => {
    const details = { algorithm: "SHA-256", matched: true };
    const entry = createActivityEntry("user1", "hash", "verify", "Verified", undefined, details);
    expect(entry.details).toEqual(details);
  });

  it("leaves file_path undefined when not provided", () => {
    const entry = createActivityEntry("user1", "project", "save", "Saved project");
    expect(entry.file_path).toBeUndefined();
  });

  it("generates unique IDs for each entry", () => {
    const a = createActivityEntry("u", "project", "a", "a");
    const b = createActivityEntry("u", "project", "b", "b");
    expect(a.id).not.toBe(b.id);
  });

  it("supports all activity categories", () => {
    const categories = ["project", "file", "hash", "export", "search", "bookmark", "note", "tag", "database", "system"] as const;
    for (const cat of categories) {
      const entry = createActivityEntry("user", cat, "test", "test");
      expect(entry.category).toBe(cat);
    }
  });
});
