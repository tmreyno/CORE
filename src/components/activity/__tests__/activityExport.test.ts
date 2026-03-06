// =============================================================================
// activityExport — CSV and JSON export tests
// =============================================================================

import { describe, it, expect } from "vitest";
import { activitiesToCsv, activitiesToJson } from "../activityExport";
import type { ActivityLogEntry } from "../../../types/project";

const ENTRY_1: ActivityLogEntry = {
  id: "1",
  timestamp: "2026-03-01T10:00:00Z",
  category: "file",
  action: "open",
  description: "Opened evidence file",
  user: "John Doe",
  file_path: "/evidence/case1.E01",
};

const ENTRY_2: ActivityLogEntry = {
  id: "2",
  timestamp: "2026-03-01T11:00:00Z",
  category: "hash",
  action: "compute",
  description: 'Hash with "special" chars, and commas',
  user: "Jane",
  file_path: undefined,
};

const ENTRY_NEWLINE: ActivityLogEntry = {
  id: "3",
  timestamp: "2026-03-01T12:00:00Z",
  category: "export",
  action: "generate",
  description: "Multi\nline\ndescription",
  user: "Admin",
};

describe("activitiesToCsv", () => {
  it("returns header for empty entries", () => {
    const csv = activitiesToCsv([]);
    expect(csv).toBe("Timestamp,Category,Action,Description,User,File Path");
  });

  it("formats a single entry as CSV row", () => {
    const csv = activitiesToCsv([ENTRY_1]);
    const lines = csv.split("\n");
    expect(lines).toHaveLength(2);
    expect(lines[0]).toBe("Timestamp,Category,Action,Description,User,File Path");
    expect(lines[1]).toBe("2026-03-01T10:00:00Z,file,open,Opened evidence file,John Doe,/evidence/case1.E01");
  });

  it("escapes values containing commas", () => {
    const csv = activitiesToCsv([ENTRY_2]);
    const lines = csv.split("\n");
    // Description has commas and quotes — should be escaped
    expect(lines[1]).toContain('"Hash with ""special"" chars, and commas"');
  });

  it("escapes values containing double quotes", () => {
    const csv = activitiesToCsv([ENTRY_2]);
    // Double quotes should be doubled inside CSV
    expect(csv).toContain('""special""');
  });

  it("escapes values containing newlines", () => {
    const csv = activitiesToCsv([ENTRY_NEWLINE]);
    // Newlines in description should cause quoting
    expect(csv).toContain('"Multi\nline\ndescription"');
  });

  it("outputs empty string for undefined fields", () => {
    const csv = activitiesToCsv([ENTRY_2]);
    // file_path is undefined → empty at end of row
    const lastLine = csv.split("\n")[1];
    expect(lastLine.endsWith(",")).toBe(true); // empty file_path generates trailing empty
  });

  it("handles multiple entries", () => {
    const csv = activitiesToCsv([ENTRY_1, ENTRY_2]);
    const lines = csv.split("\n");
    expect(lines).toHaveLength(3); // header + 2 rows
  });
});

describe("activitiesToJson", () => {
  it("returns empty array for no entries", () => {
    expect(activitiesToJson([])).toBe("[]");
  });

  it("returns pretty-printed JSON", () => {
    const json = activitiesToJson([ENTRY_1]);
    const parsed = JSON.parse(json);
    expect(parsed).toHaveLength(1);
    expect(parsed[0].id).toBe("1");
    expect(parsed[0].action).toBe("open");
  });

  it("uses 2-space indentation", () => {
    const json = activitiesToJson([ENTRY_1]);
    // Pretty-printed JSON should have indentation
    expect(json).toContain("  ");
    expect(json).not.toBe(JSON.stringify([ENTRY_1])); // not minified
  });

  it("preserves all fields including optional ones", () => {
    const json = activitiesToJson([ENTRY_1]);
    const parsed = JSON.parse(json);
    expect(parsed[0]).toHaveProperty("file_path", "/evidence/case1.E01");
  });

  it("round-trips through JSON.parse", () => {
    const entries = [ENTRY_1, ENTRY_2];
    const json = activitiesToJson(entries);
    const parsed = JSON.parse(json);
    expect(parsed).toHaveLength(2);
    expect(parsed[0].timestamp).toBe(ENTRY_1.timestamp);
    expect(parsed[1].description).toBe(ENTRY_2.description);
  });
});
