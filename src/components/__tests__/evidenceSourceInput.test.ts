// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it } from "vitest";
import { buildEvidenceSourceInput } from "../evidenceSourceInput";
import type { SelectedEntry } from "../EvidenceTree/types";

function makeEntry(overrides: Partial<SelectedEntry> = {}): SelectedEntry {
  return {
    containerPath: "/case/evidence.ad1",
    entryPath: "/Users/alice/report.pdf",
    name: "report.pdf",
    size: 128,
    isDir: false,
    ...overrides,
  };
}

describe("buildEvidenceSourceInput", () => {
  it("preserves explicit AD1 entry source type", () => {
    const source = buildEvidenceSourceInput(
      null,
      makeEntry({ containerType: "ad1" }),
    );

    expect(source).toEqual({
      containerPath: "/case/evidence.ad1",
      entryPath: "/Users/alice/report.pdf",
      containerType: "ad1",
      size: 128,
    });
  });

  it("infers VFS entry source type from an E01 parent container", () => {
    const source = buildEvidenceSourceInput(
      null,
      makeEntry({
        containerPath: "/case/disk.E01",
        entryPath: "/Windows/System32/config/SAM",
        name: "SAM",
        isVfsEntry: true,
        containerType: "vfs",
      }),
    );

    expect(source).toEqual({
      containerPath: "/case/disk.E01",
      entryPath: "/Windows/System32/config/SAM",
      containerType: "e01",
      size: 128,
    });
  });

  it("preserves explicit VFS source type from the evidence file", () => {
    const source = buildEvidenceSourceInput(
      null,
      makeEntry({
        containerPath: "/case/disk.raw",
        entryPath: "/Users/alice/report.pdf",
        name: "report.pdf",
        isVfsEntry: true,
        containerType: "raw",
      }),
    );

    expect(source).toEqual({
      containerPath: "/case/disk.raw",
      entryPath: "/Users/alice/report.pdf",
      containerType: "raw",
      size: 128,
    });
  });

  it("preserves explicit archive source type from the evidence file", () => {
    const source = buildEvidenceSourceInput(
      null,
      makeEntry({
        containerPath: "/case/logs.tar.gz",
        entryPath: "logs/system.log",
        name: "system.log",
        isArchiveEntry: true,
        containerType: "tar.gz",
      }),
    );

    expect(source).toEqual({
      containerPath: "/case/logs.tar.gz",
      entryPath: "logs/system.log",
      containerType: "tar.gz",
      size: 128,
    });
  });

  it("infers compound archive source types when no explicit type is present", () => {
    for (const extension of ["tar.gz", "tar.xz", "tar.bz2", "tar.zst", "tar.lz4"]) {
      const source = buildEvidenceSourceInput(
        null,
        makeEntry({
          containerPath: `/case/logs.${extension}`,
          entryPath: "logs/system.log",
          name: "system.log",
          isArchiveEntry: true,
        }),
      );

      expect(source).toEqual({
        containerPath: `/case/logs.${extension}`,
        entryPath: "logs/system.log",
        containerType: extension,
        size: 128,
      });
    }
  });

  it("splits nested archive compact paths into nested source fields", () => {
    const source = buildEvidenceSourceInput(
      null,
      makeEntry({
        containerPath: "/case/disk.E01",
        entryPath: "Users/alice/archive.zip::docs/report.txt",
        name: "report.txt",
        isArchiveEntry: true,
        containerType: "zip",
      }),
    );

    expect(source).toEqual({
      containerPath: "/case/disk.E01",
      nestedArchivePath: "Users/alice/archive.zip",
      entryPath: "docs/report.txt",
      containerType: "zip",
      size: 128,
    });
  });

  it("uses disk source fields for normal local files", () => {
    const source = buildEvidenceSourceInput({
      path: "/case/export.bin",
      filename: "export.bin",
      size: 256,
      container_type: "raw",
    });

    expect(source).toEqual({
      path: "/case/export.bin",
      containerType: "disk",
      size: 256,
    });
  });
});
