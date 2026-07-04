// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it } from "vitest";
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
});
