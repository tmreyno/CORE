import { afterEach, describe, expect, it, vi } from "vitest";
import {
  parseBrowserProjectFile,
  pickBrowserProjectFile,
  uniqueProjectFilePaths,
} from "../useProjectIO";
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

describe("pickBrowserProjectFile", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("resolves null when the browser file picker is cancelled", async () => {
    const clickSpy = vi
      .spyOn(HTMLInputElement.prototype, "click")
      .mockImplementation(function (this: HTMLInputElement) {
        this.dispatchEvent(new Event("cancel"));
      });

    await expect(pickBrowserProjectFile()).resolves.toBeNull();
    expect(clickSpy).toHaveBeenCalledTimes(1);
    expect(document.querySelector('input[type="file"]')).toBeNull();
  });

  it("parses the selected browser project file", async () => {
    const file = {
      name: "browser.cffx",
      text: async () =>
        JSON.stringify({
          name: "Browser Case",
          root_path: "/cases/browser",
        }),
    };

    vi.spyOn(HTMLInputElement.prototype, "click").mockImplementation(function (
      this: HTMLInputElement,
    ) {
      Object.defineProperty(this, "files", {
        configurable: true,
        value: [file],
      });
      this.dispatchEvent(new Event("change"));
    });

    await expect(pickBrowserProjectFile()).resolves.toMatchObject({
      path: "browser.cffx",
      project: {
        name: "Browser Case",
        root_path: "/cases/browser",
      },
    });
    expect(document.querySelector('input[type="file"]')).toBeNull();
  });
});
