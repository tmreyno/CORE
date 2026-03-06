// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { buildL01ExportOptions } from "../l01Export";

// =============================================================================
// buildL01ExportOptions
// =============================================================================

describe("buildL01ExportOptions", () => {
  const REQUIRED = { sourcePaths: ["/data/folder"], outputPath: "/out/evidence" };

  it("returns required fields as-is", () => {
    const opts = buildL01ExportOptions(REQUIRED);
    expect(opts.sourcePaths).toEqual(["/data/folder"]);
    expect(opts.outputPath).toBe("/out/evidence");
  });

  it("applies forensic defaults", () => {
    const opts = buildL01ExportOptions(REQUIRED);
    expect(opts.compression).toBe("fast");
    expect(opts.hashAlgorithm).toBe("md5");
  });

  it("overrides defaults with explicit values", () => {
    const opts = buildL01ExportOptions({
      ...REQUIRED,
      compression: "best",
      hashAlgorithm: "sha1",
      segmentSize: 2048 * 1024 * 1024,
    });
    expect(opts.compression).toBe("best");
    expect(opts.hashAlgorithm).toBe("sha1");
    expect(opts.segmentSize).toBe(2048 * 1024 * 1024);
  });

  it("passes optional case metadata through", () => {
    const opts = buildL01ExportOptions({
      ...REQUIRED,
      caseNumber: "C-002",
      evidenceNumber: "EV-002",
      examinerName: "John Smith",
      description: "Logical evidence",
      notes: "Collection notes",
    });
    expect(opts.caseNumber).toBe("C-002");
    expect(opts.evidenceNumber).toBe("EV-002");
    expect(opts.examinerName).toBe("John Smith");
    expect(opts.description).toBe("Logical evidence");
    expect(opts.notes).toBe("Collection notes");
  });

  it("leaves optional metadata undefined when not provided", () => {
    const opts = buildL01ExportOptions(REQUIRED);
    expect(opts.segmentSize).toBeUndefined();
    expect(opts.caseNumber).toBeUndefined();
    expect(opts.evidenceNumber).toBeUndefined();
    expect(opts.examinerName).toBeUndefined();
    expect(opts.description).toBeUndefined();
    expect(opts.notes).toBeUndefined();
  });

  it("preserves 0 segment size (means no splitting)", () => {
    const opts = buildL01ExportOptions({ ...REQUIRED, segmentSize: 0 });
    expect(opts.segmentSize).toBe(0);
  });
});
