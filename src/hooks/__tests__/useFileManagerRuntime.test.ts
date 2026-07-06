// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it } from "vitest";
import { ask } from "@tauri-apps/plugin-dialog";
import { mockInvoke, mockListen } from "../../__tests__/setup";
import { useFileManager } from "../useFileManager";

describe("useFileManager runtime guards", () => {
  it("does not request system stats outside a Tauri runtime", async () => {
    const fileManager = useFileManager();
    const cleanup = await fileManager.setupSystemStatsListener();

    expect(mockInvoke).not.toHaveBeenCalledWith("get_system_stats");
    expect(mockListen).not.toHaveBeenCalledWith("system-stats", expect.any(Function));
    expect(() => cleanup()).not.toThrow();
  });

  it("does not open a native directory picker outside a Tauri runtime", async () => {
    const fileManager = useFileManager();

    await fileManager.browseScanDir();

    expect(fileManager.statusKind()).toBe("error");
    expect(fileManager.statusMessage()).toContain("Directory browsing is available in the desktop app");
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("does not invoke streaming scan outside a Tauri runtime", async () => {
    const fileManager = useFileManager();

    await fileManager.scanForFiles("/cases/evidence");

    expect(fileManager.statusKind()).toBe("error");
    expect(fileManager.statusMessage()).toContain("Evidence directory scanning is available in the desktop app");
    expect(mockListen).not.toHaveBeenCalledWith("scan-file-found", expect.any(Function));
    expect(mockInvoke).not.toHaveBeenCalledWith("scan_directory_streaming", expect.anything());
  });

  it("does not open a native large-container prompt outside a Tauri runtime", async () => {
    const fileManager = useFileManager();
    const file = {
      path: "/evidence/large.E01",
      filename: "large.E01",
      container_type: "ewf",
      size: 51 * 1024 * 1024 * 1024,
    };

    await fileManager.selectAndViewFile(file);

    expect(ask).not.toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalled();
    expect(fileManager.activeFile()).toBe(file);
  });

  it("does not invoke container metadata loaders outside a Tauri runtime", async () => {
    const fileManager = useFileManager();
    const file = {
      path: "/evidence/source.AD1",
      filename: "source.AD1",
      container_type: "ad1",
      size: 1024,
    };

    fileManager.restoreDiscoveredFiles([file]);

    await expect(fileManager.loadFileInfo(file)).rejects.toThrow(
      "Container metadata loading is available in the desktop app",
    );
    await fileManager.loadAllInfo();
    await fileManager.loadStoredHashesInBackground();
    await fileManager.selectAndViewFile(file);

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(fileManager.activeFile()).toBe(file);
  });
});
