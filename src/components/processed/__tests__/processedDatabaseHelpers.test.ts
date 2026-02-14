// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  getDatabaseDisplayName,
  getTotalKeywords,
  getKeywordFiles,
} from "../processedDatabaseHelpers";
import type { ProcessedDatabase, AxiomCaseInfo } from "../../../types/processed";

/** Create a minimal ProcessedDatabase */
function makeDb(overrides: Partial<ProcessedDatabase> = {}): ProcessedDatabase {
  return {
    db_type: "MagnetAxiom",
    path: "/path/to/database.db",
    ...overrides,
  };
}

/** Create a minimal AxiomCaseInfo */
function makeCaseInfo(overrides: Partial<AxiomCaseInfo> = {}): AxiomCaseInfo {
  return {
    case_name: "Test Case",
    evidence_sources: [],
    search_results: [],
    total_artifacts: 0,
    ...overrides,
  };
}

// =============================================================================
// getDatabaseDisplayName
// =============================================================================
describe("getDatabaseDisplayName", () => {
  it("uses case_name from caseInfo when available", () => {
    const db = makeDb({ name: "db-name", case_name: "db-case" });
    const caseInfo = makeCaseInfo({ case_name: "Axiom Case" });
    expect(getDatabaseDisplayName(db, caseInfo)).toBe("Axiom Case");
  });

  it("falls back to db.case_name when caseInfo is undefined", () => {
    const db = makeDb({ case_name: "DB Case Name" });
    expect(getDatabaseDisplayName(db)).toBe("DB Case Name");
  });

  it("falls back to db.name when case_name is not set", () => {
    const db = makeDb({ name: "My Database" });
    expect(getDatabaseDisplayName(db)).toBe("My Database");
  });

  it("falls back to ellipsed path when nothing else is set", () => {
    const db = makeDb({ path: "/very/long/path/to/database.db" });
    const result = getDatabaseDisplayName(db);
    // Should be a shortened version of the path
    expect(result).toBeTruthy();
    expect(typeof result).toBe("string");
  });

  it("falls back to db.case_name over db.name", () => {
    const db = makeDb({ case_name: "Case Name", name: "DB Name" });
    expect(getDatabaseDisplayName(db)).toBe("Case Name");
  });
});

// =============================================================================
// getTotalKeywords
// =============================================================================
describe("getTotalKeywords", () => {
  it("returns 0 when caseInfo is undefined", () => {
    expect(getTotalKeywords()).toBe(0);
  });

  it("returns 0 when keyword_info is undefined", () => {
    const caseInfo = makeCaseInfo();
    expect(getTotalKeywords(caseInfo)).toBe(0);
  });

  it("returns 0 when keywords array is undefined", () => {
    const caseInfo = makeCaseInfo({
      keyword_info: { keyword_files: [] } as any,
    });
    expect(getTotalKeywords(caseInfo)).toBe(0);
  });

  it("returns keyword count", () => {
    const caseInfo = makeCaseInfo({
      keyword_info: {
        keywords: ["keyword1", "keyword2", "keyword3"],
        keyword_files: [],
      } as any,
    });
    expect(getTotalKeywords(caseInfo)).toBe(3);
  });
});

// =============================================================================
// getKeywordFiles
// =============================================================================
describe("getKeywordFiles", () => {
  it("returns empty array when caseInfo is undefined", () => {
    expect(getKeywordFiles()).toEqual([]);
  });

  it("returns empty array when keyword_info is undefined", () => {
    const caseInfo = makeCaseInfo();
    expect(getKeywordFiles(caseInfo)).toEqual([]);
  });

  it("returns empty array when keyword_files is undefined", () => {
    const caseInfo = makeCaseInfo({
      keyword_info: { keywords: [] } as any,
    });
    expect(getKeywordFiles(caseInfo)).toEqual([]);
  });

  it("returns keyword files", () => {
    const files = [
      { path: "/keywords.txt", count: 10 },
      { path: "/more.txt", count: 5 },
    ];
    const caseInfo = makeCaseInfo({
      keyword_info: {
        keywords: [],
        keyword_files: files,
      } as any,
    });
    expect(getKeywordFiles(caseInfo)).toEqual(files);
  });
});
