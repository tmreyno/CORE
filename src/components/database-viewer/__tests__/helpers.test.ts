// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { getColumnTypeColor } from "../helpers";

// =============================================================================
// getColumnTypeColor
// =============================================================================

describe("getColumnTypeColor", () => {
  it("returns blue for integer types", () => {
    expect(getColumnTypeColor("INTEGER")).toBe("text-blue-400");
    expect(getColumnTypeColor("INT")).toBe("text-blue-400");
    expect(getColumnTypeColor("BIGINT")).toBe("text-blue-400");
    expect(getColumnTypeColor("SMALLINT")).toBe("text-blue-400");
    expect(getColumnTypeColor("TINYINT")).toBe("text-blue-400");
  });

  it("returns green for text types", () => {
    expect(getColumnTypeColor("TEXT")).toBe("text-green-400");
    expect(getColumnTypeColor("VARCHAR(255)")).toBe("text-green-400");
    expect(getColumnTypeColor("CHAR(10)")).toBe("text-green-400");
    expect(getColumnTypeColor("CLOB")).toBe("text-green-400");
    expect(getColumnTypeColor("NVARCHAR(100)")).toBe("text-green-400");
  });

  it("returns purple for float types", () => {
    expect(getColumnTypeColor("REAL")).toBe("text-purple-400");
    expect(getColumnTypeColor("FLOAT")).toBe("text-purple-400");
    expect(getColumnTypeColor("DOUBLE")).toBe("text-purple-400");
    expect(getColumnTypeColor("DOUBLE PRECISION")).toBe("text-purple-400");
  });

  it("returns orange for blob type", () => {
    expect(getColumnTypeColor("BLOB")).toBe("text-orange-400");
  });

  it("returns cyan for date/time types", () => {
    expect(getColumnTypeColor("DATE")).toBe("text-cyan-400");
    expect(getColumnTypeColor("DATETIME")).toBe("text-cyan-400");
    expect(getColumnTypeColor("TIMESTAMP")).toBe("text-cyan-400");
  });

  it("returns muted for empty or NULL", () => {
    expect(getColumnTypeColor("")).toBe("text-txt-muted");
    expect(getColumnTypeColor("NULL")).toBe("text-txt-muted");
  });

  it("returns secondary for unrecognized types", () => {
    expect(getColumnTypeColor("CUSTOM_TYPE")).toBe("text-txt-secondary");
    expect(getColumnTypeColor("MONEY")).toBe("text-txt-secondary");
  });

  it("is case-insensitive", () => {
    expect(getColumnTypeColor("integer")).toBe("text-blue-400");
    expect(getColumnTypeColor("text")).toBe("text-green-400");
    expect(getColumnTypeColor("real")).toBe("text-purple-400");
    expect(getColumnTypeColor("blob")).toBe("text-orange-400");
    expect(getColumnTypeColor("datetime")).toBe("text-cyan-400");
  });
});
