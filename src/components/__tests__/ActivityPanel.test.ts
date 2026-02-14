// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { activitiesToCsv, activitiesToJson } from "../ActivityPanel";
import type { ActivityLogEntry } from "../../types/project";

function makeEntry(overrides: Partial<ActivityLogEntry> = {}): ActivityLogEntry {
  return {
    id: "entry-1",
    timestamp: "2025-01-15T10:00:00Z",
    user: "analyst1",
    category: "file",
    action: "open",
    description: "Opened evidence file test.E01",
    file_path: "/evidence/test.E01",
    details: {},
    ...overrides,
  };
}

type ActivityFilter = "all" | "project" | "file" | "hash" | "export" | "search" | "bookmark" | "note";
type SortDirection = "newest" | "oldest";

interface FilterOptions {
  filter: ActivityFilter;
  sortDirection: SortDirection;
  searchQuery: string;
  dateFrom: string;
  dateTo: string;
}

function applyFilters(entries: ActivityLogEntry[], opts: FilterOptions): ActivityLogEntry[] {
  let activities = [...entries];
  if (opts.filter !== "all") {
    activities = activities.filter(e => e.category === opts.filter);
  }
  const query = opts.searchQuery.toLowerCase().trim();
  if (query) {
    activities = activities.filter(e =>
      e.description?.toLowerCase().includes(query) ||
      e.action?.toLowerCase().includes(query) ||
      e.user?.toLowerCase().includes(query) ||
      e.file_path?.toLowerCase().includes(query)
    );
  }
  if (opts.dateFrom) {
    const fromDate = new Date(opts.dateFrom);
    fromDate.setHours(0, 0, 0, 0);
    activities = activities.filter(e => new Date(e.timestamp) >= fromDate);
  }
  if (opts.dateTo) {
    const toDate = new Date(opts.dateTo);
    toDate.setHours(23, 59, 59, 999);
    activities = activities.filter(e => new Date(e.timestamp) <= toDate);
  }
  activities.sort((a, b) => {
    const diff = new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime();
    return opts.sortDirection === "newest" ? diff : -diff;
  });
  return activities;
}

const defaultOpts: FilterOptions = {
  filter: "all",
  sortDirection: "newest",
  searchQuery: "",
  dateFrom: "",
  dateTo: "",
};

describe("activitiesToCsv", () => {
  it("produces a header row for empty input", () => {
    const csv = activitiesToCsv([]);
    expect(csv).toBe("Timestamp,Category,Action,Description,User,File Path");
  });

  it("converts entries to CSV rows", () => {
    const entries = [makeEntry({ id: "a", description: "Opened file", category: "file", action: "open" })];
    const csv = activitiesToCsv(entries);
    const lines = csv.split("\n");
    expect(lines).toHaveLength(2);
    expect(lines[1]).toContain("file");
    expect(lines[1]).toContain("open");
    expect(lines[1]).toContain("Opened file");
  });

  it("escapes commas in descriptions", () => {
    const entries = [makeEntry({ description: "File opened, hash computed" })];
    const csv = activitiesToCsv(entries);
    expect(csv).toContain('"File opened, hash computed"');
  });

  it("escapes double quotes in descriptions", () => {
    const entries = [makeEntry({ description: 'Said "hello"' })];
    const csv = activitiesToCsv(entries);
    expect(csv).toContain('"Said ""hello"""');
  });

  it("handles newlines in descriptions", () => {
    const entries = [makeEntry({ description: "Line one\nLine two" })];
    const csv = activitiesToCsv(entries);
    expect(csv).toContain('"Line one\nLine two"');
  });

  it("handles undefined user and file_path", () => {
    const entries = [makeEntry({ user: undefined, file_path: undefined })];
    const csv = activitiesToCsv(entries);
    const lines = csv.split("\n");
    expect(lines[1]).toMatch(/,,$/);
  });

  it("includes all fields in correct order", () => {
    const entry = makeEntry({
      timestamp: "2025-06-01T12:00:00Z",
      category: "hash",
      action: "compute",
      description: "Computed SHA-256",
      user: "examiner",
      file_path: "/evidence/disk.E01",
    });
    const csv = activitiesToCsv([entry]);
    const row = csv.split("\n")[1];
    const cols = row.split(",");
    expect(cols[0]).toBe("2025-06-01T12:00:00Z");
    expect(cols[1]).toBe("hash");
    expect(cols[2]).toBe("compute");
    expect(cols[3]).toBe("Computed SHA-256");
    expect(cols[4]).toBe("examiner");
    expect(cols[5]).toBe("/evidence/disk.E01");
  });

  it("handles multiple entries", () => {
    const entries = [makeEntry({ id: "1" }), makeEntry({ id: "2" }), makeEntry({ id: "3" })];
    const csv = activitiesToCsv(entries);
    const lines = csv.split("\n");
    expect(lines).toHaveLength(4);
  });
});

describe("activitiesToJson", () => {
  it("produces valid JSON", () => {
    const entries = [makeEntry()];
    const json = activitiesToJson(entries);
    const parsed = JSON.parse(json);
    expect(parsed).toHaveLength(1);
    expect(parsed[0].id).toBe("entry-1");
  });

  it("is pretty-printed with 2-space indentation", () => {
    const entries = [makeEntry()];
    const json = activitiesToJson(entries);
    expect(json).toContain("\n");
    expect(json).toContain("  ");
  });

  it("handles empty array", () => {
    const json = activitiesToJson([]);
    expect(JSON.parse(json)).toEqual([]);
  });

  it("preserves all fields", () => {
    const entry = makeEntry({ details: { size: 1024, hash: "abc123" } });
    const json = activitiesToJson([entry]);
    const parsed = JSON.parse(json)[0];
    expect(parsed.id).toBe("entry-1");
    expect(parsed.category).toBe("file");
    expect(parsed.action).toBe("open");
    expect(parsed.details.size).toBe(1024);
    expect(parsed.details.hash).toBe("abc123");
  });
});

describe("Activity filter logic", () => {
  describe("category filter", () => {
    const entries = [
      makeEntry({ id: "e1", category: "file", description: "File action" }),
      makeEntry({ id: "e2", category: "hash", description: "Hash action" }),
      makeEntry({ id: "e3", category: "export", description: "Export action" }),
      makeEntry({ id: "e4", category: "bookmark", description: "Bookmark action" }),
    ];

    it("returns all entries with all filter", () => {
      const result = applyFilters(entries, { ...defaultOpts, filter: "all" });
      expect(result).toHaveLength(4);
    });

    it("filters to file only", () => {
      const result = applyFilters(entries, { ...defaultOpts, filter: "file" });
      expect(result).toHaveLength(1);
      expect(result[0].description).toBe("File action");
    });

    it("filters to hash only", () => {
      const result = applyFilters(entries, { ...defaultOpts, filter: "hash" });
      expect(result).toHaveLength(1);
      expect(result[0].description).toBe("Hash action");
    });

    it("filters to export only", () => {
      const result = applyFilters(entries, { ...defaultOpts, filter: "export" });
      expect(result).toHaveLength(1);
      expect(result[0].description).toBe("Export action");
    });

    it("returns empty for category with no entries", () => {
      const result = applyFilters(entries, { ...defaultOpts, filter: "note" });
      expect(result).toHaveLength(0);
    });
  });

  describe("text search", () => {
    const entries = [
      makeEntry({ id: "e1", description: "Opened evidence.E01", action: "open", user: "analyst1", file_path: "/data/evidence.E01" }),
      makeEntry({ id: "e2", description: "Computed SHA-256 hash", action: "compute", user: "tech2", file_path: "/data/disk.img" }),
      makeEntry({ id: "e3", description: "Exported files to USB", action: "export", user: "analyst1", file_path: "/usb/output" }),
    ];

    it("filters by description match", () => {
      const result = applyFilters(entries, { ...defaultOpts, searchQuery: "SHA-256" });
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe("e2");
    });

    it("filters by action match", () => {
      const result = applyFilters(entries, { ...defaultOpts, searchQuery: "compute" });
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe("e2");
    });

    it("filters by user match", () => {
      const result = applyFilters(entries, { ...defaultOpts, searchQuery: "tech2" });
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe("e2");
    });

    it("filters by file_path match", () => {
      const result = applyFilters(entries, { ...defaultOpts, searchQuery: "/usb" });
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe("e3");
    });

    it("is case-insensitive", () => {
      const result = applyFilters(entries, { ...defaultOpts, searchQuery: "SHA" });
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe("e2");
    });

    it("trims whitespace", () => {
      const result = applyFilters(entries, { ...defaultOpts, searchQuery: "  SHA-256  " });
      expect(result).toHaveLength(1);
    });

    it("returns all when query is empty", () => {
      const result = applyFilters(entries, { ...defaultOpts, searchQuery: "" });
      expect(result).toHaveLength(3);
    });

    it("returns empty when nothing matches", () => {
      const result = applyFilters(entries, { ...defaultOpts, searchQuery: "nonexistent" });
      expect(result).toHaveLength(0);
    });
  });

  describe("date range", () => {
    const entries = [
      makeEntry({ id: "jan", timestamp: "2025-01-15T10:00:00Z", description: "January" }),
      makeEntry({ id: "feb", timestamp: "2025-02-15T10:00:00Z", description: "February" }),
      makeEntry({ id: "mar", timestamp: "2025-03-15T10:00:00Z", description: "March" }),
    ];

    it("filters by dateFrom only", () => {
      const result = applyFilters(entries, { ...defaultOpts, dateFrom: "2025-02-01" });
      expect(result).toHaveLength(2);
      expect(result.map(e => e.id)).toContain("feb");
      expect(result.map(e => e.id)).toContain("mar");
    });

    it("filters by dateTo only", () => {
      const result = applyFilters(entries, { ...defaultOpts, dateTo: "2025-02-28" });
      expect(result).toHaveLength(2);
      expect(result.map(e => e.id)).toContain("jan");
      expect(result.map(e => e.id)).toContain("feb");
    });

    it("filters by date range (from + to)", () => {
      const result = applyFilters(entries, { ...defaultOpts, dateFrom: "2025-02-01", dateTo: "2025-02-28" });
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe("feb");
    });

    it("includes entries on exact from date", () => {
      const result = applyFilters(entries, { ...defaultOpts, dateFrom: "2025-01-15" });
      expect(result).toHaveLength(3);
    });

    it("includes entries on exact to date", () => {
      const result = applyFilters(entries, { ...defaultOpts, dateTo: "2025-03-16" });
      expect(result).toHaveLength(3);
    });

    it("returns empty for impossible date range", () => {
      const result = applyFilters(entries, { ...defaultOpts, dateFrom: "2025-04-01", dateTo: "2025-04-30" });
      expect(result).toHaveLength(0);
    });

    it("returns all when no dates set", () => {
      const result = applyFilters(entries, defaultOpts);
      expect(result).toHaveLength(3);
    });
  });

  describe("sort direction", () => {
    const entries = [
      makeEntry({ id: "old", timestamp: "2025-01-01T10:00:00Z" }),
      makeEntry({ id: "mid", timestamp: "2025-06-01T10:00:00Z" }),
      makeEntry({ id: "new", timestamp: "2025-12-01T10:00:00Z" }),
    ];

    it("sorts newest first by default", () => {
      const result = applyFilters(entries, { ...defaultOpts, sortDirection: "newest" });
      expect(result[0].id).toBe("new");
      expect(result[1].id).toBe("mid");
      expect(result[2].id).toBe("old");
    });

    it("sorts oldest first when toggled", () => {
      const result = applyFilters(entries, { ...defaultOpts, sortDirection: "oldest" });
      expect(result[0].id).toBe("old");
      expect(result[1].id).toBe("mid");
      expect(result[2].id).toBe("new");
    });
  });

  describe("combined filters", () => {
    const entries = [
      makeEntry({ id: "e1", category: "file", timestamp: "2025-01-15T10:00:00Z", description: "File A", action: "open" }),
      makeEntry({ id: "e2", category: "file", timestamp: "2025-02-15T10:00:00Z", description: "File B", action: "scan" }),
      makeEntry({ id: "e3", category: "hash", timestamp: "2025-02-20T10:00:00Z", description: "Hash C", action: "compute" }),
      makeEntry({ id: "e4", category: "file", timestamp: "2025-03-15T10:00:00Z", description: "File D", action: "open" }),
    ];

    it("applies category + text search together", () => {
      const result = applyFilters(entries, { ...defaultOpts, filter: "file", searchQuery: "scan" });
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe("e2");
    });

    it("applies category + date range together", () => {
      const result = applyFilters(entries, { ...defaultOpts, filter: "file", dateFrom: "2025-02-01", dateTo: "2025-02-28" });
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe("e2");
    });

    it("applies search + date range together", () => {
      const result = applyFilters(entries, { ...defaultOpts, searchQuery: "open", dateFrom: "2025-03-01" });
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe("e4");
    });

    it("applies all filters together", () => {
      const result = applyFilters(entries, { filter: "file", sortDirection: "oldest", searchQuery: "File", dateFrom: "2025-02-01", dateTo: "2025-03-31" });
      expect(result).toHaveLength(2);
      expect(result[0].id).toBe("e2");
      expect(result[1].id).toBe("e4");
    });
  });

  describe("hasActiveFilters detection", () => {
    const hasActiveFilters = (opts: FilterOptions): boolean =>
      opts.filter !== "all" || opts.searchQuery !== "" || opts.dateFrom !== "" || opts.dateTo !== "";

    it("returns false for default state", () => {
      expect(hasActiveFilters(defaultOpts)).toBe(false);
    });

    it("returns true when filter is not all", () => {
      expect(hasActiveFilters({ ...defaultOpts, filter: "hash" })).toBe(true);
    });

    it("returns true when searchQuery is set", () => {
      expect(hasActiveFilters({ ...defaultOpts, searchQuery: "test" })).toBe(true);
    });

    it("returns true when dateFrom is set", () => {
      expect(hasActiveFilters({ ...defaultOpts, dateFrom: "2025-01-01" })).toBe(true);
    });

    it("returns true when dateTo is set", () => {
      expect(hasActiveFilters({ ...defaultOpts, dateTo: "2025-12-31" })).toBe(true);
    });

    it("sortDirection alone does not trigger active filter", () => {
      expect(hasActiveFilters({ ...defaultOpts, sortDirection: "oldest" })).toBe(false);
    });
  });

  describe("edge cases", () => {
    it("handles empty activity log", () => {
      const result = applyFilters([], defaultOpts);
      expect(result).toHaveLength(0);
    });

    it("handles entries with undefined description", () => {
      const entries = [makeEntry({ description: undefined as unknown as string })];
      const result = applyFilters(entries, { ...defaultOpts, searchQuery: "anything" });
      expect(result).toHaveLength(0);
    });

    it("handles entries with undefined user", () => {
      const entries = [makeEntry({ user: undefined })];
      const result = applyFilters(entries, { ...defaultOpts, searchQuery: "analyst" });
      expect(result).toHaveLength(0);
    });

    it("does not mutate the original array", () => {
      const entries = [
        makeEntry({ id: "b", timestamp: "2025-01-01T00:00:00Z" }),
        makeEntry({ id: "a", timestamp: "2025-12-01T00:00:00Z" }),
      ];
      const originalOrder = entries.map(e => e.id);
      applyFilters(entries, { ...defaultOpts, sortDirection: "oldest" });
      expect(entries.map(e => e.id)).toEqual(originalOrder);
    });
  });
});
