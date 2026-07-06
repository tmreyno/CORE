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
      makeEntry({ containerType: "ad1", dataAddr: 8192, itemAddr: 4096 }),
    );

    expect(source).toEqual({
      containerPath: "/case/evidence.ad1",
      entryPath: "/Users/alice/report.pdf",
      containerType: "ad1",
      size: 128,
      dataAddr: 8192,
      itemAddr: 4096,
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

  it("infers numbered raw VFS entry source type from a segmented parent container", () => {
    const source = buildEvidenceSourceInput(
      null,
      makeEntry({
        containerPath: "/case/SCHARDT/SCHARDT.001",
        entryPath: "/Partition_1_Ntfs/Windows/System32/config/SYSTEM",
        name: "SYSTEM",
        isVfsEntry: true,
        containerType: "vfs",
      }),
    );

    expect(source).toEqual({
      containerPath: "/case/SCHARDT/SCHARDT.001",
      entryPath: "/Partition_1_Ntfs/Windows/System32/config/SYSTEM",
      containerType: "raw",
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

  it("normalizes saved raw image display labels for VFS entries", () => {
    const source = buildEvidenceSourceInput(
      null,
      makeEntry({
        containerPath: "/case/usb.dd",
        entryPath: "/Partition_1_Ntfs/Windows/System32/config/SYSTEM",
        name: "SYSTEM",
        isVfsEntry: true,
        containerType: "Raw Image",
      }),
    );

    expect(source).toEqual({
      containerPath: "/case/usb.dd",
      entryPath: "/Partition_1_Ntfs/Windows/System32/config/SYSTEM",
      containerType: "raw",
      size: 128,
    });
  });

  it("normalizes saved E01 display labels for VFS entries", () => {
    const source = buildEvidenceSourceInput(
      null,
      makeEntry({
        containerPath: "/case/workstation.E01",
        entryPath: "/Partition1_NTFS/CONFIG.SYS",
        name: "CONFIG.SYS",
        isVfsEntry: true,
        containerType: "EnCase (E01)",
      }),
    );

    expect(source).toEqual({
      containerPath: "/case/workstation.E01",
      entryPath: "/Partition1_NTFS/CONFIG.SYS",
      containerType: "e01",
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

  it("normalizes saved logical TAR display labels from the container path", () => {
    const source = buildEvidenceSourceInput(
      null,
      makeEntry({
        containerPath: "/case/mobile-data.tar",
        entryPath: "data/system/users/0/settings_secure.xml",
        name: "settings_secure.xml",
        isArchiveEntry: true,
        containerType: "TAR (Logical)",
      }),
    );

    expect(source).toEqual({
      containerPath: "/case/mobile-data.tar",
      entryPath: "data/system/users/0/settings_secure.xml",
      containerType: "tar",
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
      containerType: "e01",
      size: 128,
    });
  });

  it("normalizes nested archive display labels when splitting compact paths", () => {
    const source = buildEvidenceSourceInput(
      null,
      makeEntry({
        containerPath: "/case/mobile-data.tar",
        entryPath: "payload.zip::system/build.prop",
        name: "build.prop",
        isArchiveEntry: true,
        containerType: "TAR (Logical)",
      }),
    );

    expect(source).toEqual({
      containerPath: "/case/mobile-data.tar",
      nestedArchivePath: "payload.zip",
      entryPath: "system/build.prop",
      containerType: "tar",
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
