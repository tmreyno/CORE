// =============================================================================
// nativeExport constants — forensic preset and compression level tests
// =============================================================================

import { describe, it, expect } from "vitest";
import { FORENSIC_PRESETS, COMPRESSION_LEVELS } from "../constants";
import { CompressionLevel } from "../../../../api/archiveCreate";

describe("FORENSIC_PRESETS", () => {
  it("has exactly 5 presets", () => {
    expect(FORENSIC_PRESETS).toHaveLength(5);
  });

  it("has unique IDs", () => {
    const ids = FORENSIC_PRESETS.map((p) => p.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("has unique names", () => {
    const names = FORENSIC_PRESETS.map((p) => p.name);
    expect(new Set(names).size).toBe(names.length);
  });

  it("all presets use Store compression (forensic standard)", () => {
    for (const preset of FORENSIC_PRESETS) {
      expect(preset.compressionLevel).toBe(CompressionLevel.Store);
    }
  });

  it("all presets have solid = false", () => {
    for (const preset of FORENSIC_PRESETS) {
      expect(preset.solid).toBe(false);
    }
  });

  it("all presets generate manifests except Custom", () => {
    for (const preset of FORENSIC_PRESETS) {
      expect(preset.generateManifest).toBe(true);
    }
  });

  it("Standard preset has expected defaults", () => {
    const std = FORENSIC_PRESETS.find((p) => p.id === "forensic-standard")!;
    expect(std.name).toBe("Standard");
    expect(std.splitSizeMb).toBe(2048);
    expect(std.hashAlgorithm).toBe("SHA-256");
    expect(std.verifyAfterCreate).toBe(true);
    expect(std.includeExaminerInfo).toBe(true);
  });

  it("Court preset uses dual hashes and 4GB split", () => {
    const court = FORENSIC_PRESETS.find((p) => p.id === "forensic-court")!;
    expect(court.splitSizeMb).toBe(4096);
    expect(court.hashAlgorithm).toBe("SHA-256+MD5");
    expect(court.verifyAfterCreate).toBe(true);
  });

  it("Transfer preset does not include examiner info", () => {
    const transfer = FORENSIC_PRESETS.find((p) => p.id === "forensic-transfer")!;
    expect(transfer.includeExaminerInfo).toBe(false);
    expect(transfer.splitSizeMb).toBe(2048);
  });

  it("Long-term preset uses dual hashes", () => {
    const longTerm = FORENSIC_PRESETS.find((p) => p.id === "forensic-archive-long")!;
    expect(longTerm.hashAlgorithm).toBe("SHA-256+MD5");
    expect(longTerm.includeExaminerInfo).toBe(true);
  });

  it("Custom preset has verification off by default", () => {
    const custom = FORENSIC_PRESETS.find((p) => p.id === "custom")!;
    expect(custom.verifyAfterCreate).toBe(false);
    expect(custom.includeExaminerInfo).toBe(false);
  });

  it("every preset has a description", () => {
    for (const preset of FORENSIC_PRESETS) {
      expect(preset.description.length).toBeGreaterThan(0);
    }
  });
});

describe("COMPRESSION_LEVELS", () => {
  it("has exactly 6 levels", () => {
    expect(COMPRESSION_LEVELS).toHaveLength(6);
  });

  it("includes Store (0) as first level", () => {
    expect(COMPRESSION_LEVELS[0].value).toBe(CompressionLevel.Store);
    expect(COMPRESSION_LEVELS[0].label).toContain("Store");
  });

  it("includes Ultra as last level", () => {
    const last = COMPRESSION_LEVELS[COMPRESSION_LEVELS.length - 1];
    expect(last.value).toBe(CompressionLevel.Ultra);
    expect(last.label).toBe("Ultra");
  });

  it("values are in ascending order", () => {
    for (let i = 1; i < COMPRESSION_LEVELS.length; i++) {
      expect(COMPRESSION_LEVELS[i].value).toBeGreaterThan(COMPRESSION_LEVELS[i - 1].value);
    }
  });

  it("all entries have non-empty labels", () => {
    for (const level of COMPRESSION_LEVELS) {
      expect(level.label.length).toBeGreaterThan(0);
    }
  });

  it("maps to known CompressionLevel constants", () => {
    const expectedValues = [
      CompressionLevel.Store,
      CompressionLevel.Fastest,
      CompressionLevel.Fast,
      CompressionLevel.Normal,
      CompressionLevel.Maximum,
      CompressionLevel.Ultra,
    ];
    expect(COMPRESSION_LEVELS.map((l) => l.value)).toEqual(expectedValues);
  });
});
