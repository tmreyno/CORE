import { describe, expect, it } from "vitest";
import { parseBrowserProjectFile, uniqueProjectFilePaths } from "../useProjectIO";
import type { ProjectTab } from "../../../types/project";

describe("uniqueProjectFilePaths", () => {
  it("deduplicates evidence paths shared by restored entry tabs", () => {
    const tabs: ProjectTab[] = [
      {
        id: "evidence:/evidence/disk.E01",
        type: "evidence",
        file_path: "/evidence/disk.E01",
        name: "disk.E01",
        order: 0,
      },
      {
        id: "entry:/Partition1_NTFS/boot.ini",
        type: "entry",
        file_path: "/evidence/disk.E01",
        name: "boot.ini",
        order: 1,
        entry_path: "/Partition1_NTFS/boot.ini",
      },
      {
        id: "entry:/Partition1_NTFS/pagefile.sys",
        type: "entry",
        file_path: "/evidence/disk.E01",
        name: "pagefile.sys",
        order: 2,
        entry_path: "/Partition1_NTFS/pagefile.sys",
      },
      {
        id: "document:/case/report.pdf",
        type: "document",
        file_path: "/case/report.pdf",
        name: "report.pdf",
        order: 3,
        document_path: "/case/report.pdf",
      },
    ];

    expect(uniqueProjectFilePaths(tabs)).toEqual([
      "/evidence/disk.E01",
      "/case/report.pdf",
    ]);
  });
});

describe("parseBrowserProjectFile", () => {
  it("parses a minimal cffx project for browser preview loading", () => {
    const result = parseBrowserProjectFile(
      JSON.stringify({
        name: "Seed Case",
        root_path: "/cases/seed",
        version: 1,
      }),
      "seed.cffx",
    );

    expect(result.path).toBe("seed.cffx");
    expect(result.project.name).toBe("Seed Case");
    expect(result.project.root_path).toBe("/cases/seed");
    expect(result.project.tabs).toEqual([]);
    expect(result.project.project_id).toBeTruthy();
  });

  it("rejects non-project json", () => {
    expect(() => parseBrowserProjectFile("{}", "bad.cffx")).toThrow(
      "Selected file is not a valid CORE-FFX project",
    );
  });
});
