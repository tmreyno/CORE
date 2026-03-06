// =============================================================================
// useDatabase — pure record factory function tests
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock crypto.randomUUID
let uuidCounter = 0;
vi.stubGlobal("crypto", {
  randomUUID: () => {
    uuidCounter++;
    return `00000000-0000-0000-0000-${String(uuidCounter).padStart(12, "0")}`;
  },
});

// Mock Date to get deterministic timestamps
const MOCK_ISO = "2026-03-06T12:00:00.000Z";
vi.spyOn(Date.prototype, "toISOString").mockReturnValue(MOCK_ISO);

// Mock Tauri invoke (not needed for pure functions, but module imports it)
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

// Mock solid-js
vi.mock("solid-js", () => ({
  createSignal: vi.fn(() => [() => null, vi.fn()]),
}));

import {
  createFileRecord,
  createHashRecord,
  createVerificationRecord,
  createTabRecords,
} from "../useDatabase";

import type { DiscoveredFile } from "../../types";

describe("createFileRecord", () => {
  beforeEach(() => {
    uuidCounter = 0;
  });

  it("builds a DbFileRecord with all required fields", () => {
    const file: DiscoveredFile = {
      path: "/evidence/case1.E01",
      filename: "case1.E01",
      container_type: "e01",
      size: 1024000,
      segment_count: 3,
    };

    const record = createFileRecord("session-abc", file);

    expect(record.id).toMatch(/^00000000-/);
    expect(record.session_id).toBe("session-abc");
    expect(record.path).toBe("/evidence/case1.E01");
    expect(record.filename).toBe("case1.E01");
    expect(record.container_type).toBe("e01");
    expect(record.total_size).toBe(1024000);
    expect(record.segment_count).toBe(3);
    expect(record.discovered_at).toBe(MOCK_ISO);
  });

  it("defaults segment_count to 1 when not provided", () => {
    const file: DiscoveredFile = {
      path: "/evidence/single.ad1",
      filename: "single.ad1",
      container_type: "ad1",
      size: 500,
    };

    const record = createFileRecord("sess-1", file);
    expect(record.segment_count).toBe(1);
  });

  it("generates unique IDs for each call", () => {
    const file: DiscoveredFile = {
      path: "/a.E01",
      filename: "a.E01",
      container_type: "e01",
      size: 100,
    };

    const r1 = createFileRecord("s", file);
    const r2 = createFileRecord("s", file);
    expect(r1.id).not.toBe(r2.id);
  });
});

describe("createHashRecord", () => {
  beforeEach(() => {
    uuidCounter = 0;
  });

  it("builds a DbHashRecord with minimal args", () => {
    const record = createHashRecord("file-1", "SHA-256", "abc123def456");

    expect(record.id).toMatch(/^00000000-/);
    expect(record.file_id).toBe("file-1");
    expect(record.algorithm).toBe("SHA-256");
    expect(record.hash_value).toBe("abc123def456");
    expect(record.computed_at).toBe(MOCK_ISO);
    expect(record.source).toBe("computed");
    expect(record.segment_index).toBeNull();
    expect(record.segment_name).toBeNull();
  });

  it("accepts custom source", () => {
    const record = createHashRecord("f1", "MD5", "deadbeef", "stored");
    expect(record.source).toBe("stored");
  });

  it("accepts imported source", () => {
    const record = createHashRecord("f1", "SHA-1", "aabbcc", "imported");
    expect(record.source).toBe("imported");
  });

  it("records segment info when provided", () => {
    const record = createHashRecord("f1", "SHA-256", "hash", "computed", 2, "case1.E02");
    expect(record.segment_index).toBe(2);
    expect(record.segment_name).toBe("case1.E02");
  });

  it("sets segment fields to null when omitted", () => {
    const record = createHashRecord("f1", "MD5", "hash", "computed", undefined, undefined);
    expect(record.segment_index).toBeNull();
    expect(record.segment_name).toBeNull();
  });
});

describe("createVerificationRecord", () => {
  beforeEach(() => {
    uuidCounter = 0;
  });

  it("builds a match verification record", () => {
    const record = createVerificationRecord(
      "hash-1",
      "match",
      "abc123",
      "abc123"
    );

    expect(record.id).toMatch(/^00000000-/);
    expect(record.hash_id).toBe("hash-1");
    expect(record.verified_at).toBe(MOCK_ISO);
    expect(record.result).toBe("match");
    expect(record.expected_hash).toBe("abc123");
    expect(record.actual_hash).toBe("abc123");
  });

  it("builds a mismatch verification record", () => {
    const record = createVerificationRecord(
      "hash-2",
      "mismatch",
      "expected-hash",
      "actual-hash"
    );

    expect(record.result).toBe("mismatch");
    expect(record.expected_hash).toBe("expected-hash");
    expect(record.actual_hash).toBe("actual-hash");
  });
});

describe("createTabRecords", () => {
  beforeEach(() => {
    uuidCounter = 0;
  });

  it("creates one tab record per path", () => {
    const records = createTabRecords("sess-1", ["/a.E01", "/b.ad1", "/c.zip"]);

    expect(records).toHaveLength(3);
    expect(records[0].file_path).toBe("/a.E01");
    expect(records[1].file_path).toBe("/b.ad1");
    expect(records[2].file_path).toBe("/c.zip");
  });

  it("assigns sequential tab_order", () => {
    const records = createTabRecords("s", ["/x", "/y"]);
    expect(records[0].tab_order).toBe(0);
    expect(records[1].tab_order).toBe(1);
  });

  it("marks the active tab correctly", () => {
    const records = createTabRecords("s", ["/a", "/b", "/c"], "/b");

    expect(records[0].is_active).toBe(false);
    expect(records[1].is_active).toBe(true);
    expect(records[2].is_active).toBe(false);
  });

  it("marks no tabs active when activeFilePath is omitted", () => {
    const records = createTabRecords("s", ["/a", "/b"]);

    expect(records.every((r) => r.is_active === false)).toBe(true);
  });

  it("sets session_id on all records", () => {
    const records = createTabRecords("session-xyz", ["/a"]);
    expect(records[0].session_id).toBe("session-xyz");
  });

  it("returns empty array for empty paths", () => {
    const records = createTabRecords("s", []);
    expect(records).toEqual([]);
  });

  it("generates unique IDs for each tab", () => {
    const records = createTabRecords("s", ["/a", "/b"]);
    expect(records[0].id).not.toBe(records[1].id);
  });
});
