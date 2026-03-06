// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { buildEwfExportOptions } from "../ewfExport";

// =============================================================================
// buildEwfExportOptions
// =============================================================================

describe("buildEwfExportOptions", () => {
  const REQUIRED = { sourcePaths: ["/img.dd"], outputPath: "/out/image" };

  it("returns required fields as-is", () => {
    const opts = buildEwfExportOptions(REQUIRED);
    expect(opts.sourcePaths).toEqual(["/img.dd"]);
    expect(opts.outputPath).toBe("/out/image");
  });

  it("applies forensic defaults", () => {
    const opts = buildEwfExportOptions(REQUIRED);
    expect(opts.format).toBe("e01");
    expect(opts.compression).toBe("fast");
    expect(opts.compressionMethod).toBe("deflate");
    expect(opts.computeMd5).toBe(true);
    expect(opts.computeSha1).toBe(false);
  });

  it("overrides defaults with explicit values", () => {
    const opts = buildEwfExportOptions({
      ...REQUIRED,
      format: "ex01",
      compression: "best",
      compressionMethod: "bzip2",
      computeMd5: false,
      computeSha1: true,
    });
    expect(opts.format).toBe("ex01");
    expect(opts.compression).toBe("best");
    expect(opts.compressionMethod).toBe("bzip2");
    expect(opts.computeMd5).toBe(false);
    expect(opts.computeSha1).toBe(true);
  });

  it("passes optional case metadata through", () => {
    const opts = buildEwfExportOptions({
      ...REQUIRED,
      caseNumber: "C-001",
      evidenceNumber: "EV-001",
      examinerName: "Jane Doe",
      description: "Disk image",
      notes: "Test notes",
    });
    expect(opts.caseNumber).toBe("C-001");
    expect(opts.evidenceNumber).toBe("EV-001");
    expect(opts.examinerName).toBe("Jane Doe");
    expect(opts.description).toBe("Disk image");
    expect(opts.notes).toBe("Test notes");
  });

  it("leaves optional metadata undefined when not provided", () => {
    const opts = buildEwfExportOptions(REQUIRED);
    expect(opts.caseNumber).toBeUndefined();
    expect(opts.evidenceNumber).toBeUndefined();
    expect(opts.examinerName).toBeUndefined();
    expect(opts.description).toBeUndefined();
    expect(opts.notes).toBeUndefined();
  });
});
