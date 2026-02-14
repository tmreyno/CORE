// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { buildProjectLocations } from "../toolbarHelpers";

// =============================================================================
// buildProjectLocations
// =============================================================================
describe("buildProjectLocations", () => {
  it("returns empty array when all paths are undefined", () => {
    const result = buildProjectLocations(undefined, undefined, undefined);
    expect(result).toEqual([]);
  });

  it("returns empty array when all accessors return null", () => {
    const result = buildProjectLocations(
      () => null,
      () => null,
      () => null
    );
    expect(result).toEqual([]);
  });

  it("includes evidence location when evidence path is set", () => {
    const result = buildProjectLocations(
      () => "/path/to/evidence",
      () => null,
      () => null
    );
    expect(result).toHaveLength(1);
    expect(result[0]).toEqual({
      id: "evidence",
      label: "Evidence",
      path: "/path/to/evidence",
      icon: "evidence",
    });
  });

  it("includes processed database location when path is set", () => {
    const result = buildProjectLocations(
      () => null,
      () => "/path/to/processed.db",
      () => null
    );
    expect(result).toHaveLength(1);
    expect(result[0]).toEqual({
      id: "processed",
      label: "Processed Database",
      path: "/path/to/processed.db",
      icon: "database",
    });
  });

  it("includes case documents location when path is set", () => {
    const result = buildProjectLocations(
      () => null,
      () => null,
      () => "/path/to/case-docs"
    );
    expect(result).toHaveLength(1);
    expect(result[0]).toEqual({
      id: "documents",
      label: "Case Documents",
      path: "/path/to/case-docs",
      icon: "documents",
    });
  });

  it("includes all locations when all paths are set", () => {
    const result = buildProjectLocations(
      () => "/evidence",
      () => "/processed",
      () => "/docs"
    );
    expect(result).toHaveLength(3);
    expect(result.map((l) => l.id)).toEqual(["evidence", "processed", "documents"]);
  });

  it("preserves order: evidence, processed, documents", () => {
    const result = buildProjectLocations(
      () => "/a",
      () => "/b",
      () => "/c"
    );
    expect(result[0].id).toBe("evidence");
    expect(result[1].id).toBe("processed");
    expect(result[2].id).toBe("documents");
  });

  it("skips empty string paths", () => {
    // Empty strings are falsy, so they should be skipped
    const result = buildProjectLocations(
      () => "",
      () => "/valid",
      () => ""
    );
    expect(result).toHaveLength(1);
    expect(result[0].id).toBe("processed");
  });
});
