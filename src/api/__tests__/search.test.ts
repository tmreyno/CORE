// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it, vi } from "vitest";
import { mockListen } from "../../__tests__/setup";
import { onIndexProgress } from "../search";

describe("search api runtime guards", () => {
  it("does not register Tauri progress listeners in a browser-only runtime", async () => {
    const callback = vi.fn();
    const unlisten = await onIndexProgress(callback);

    expect(mockListen).not.toHaveBeenCalled();
    expect(callback).not.toHaveBeenCalled();
    expect(() => unlisten()).not.toThrow();
  });
});
