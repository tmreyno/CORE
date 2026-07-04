// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createRoot } from "solid-js";
import { describe, expect, it, vi } from "vitest";
import { mockListen } from "../../__tests__/setup";
import { useMenuActions, type UseMenuActionsDeps } from "../useMenuActions";

describe("useMenuActions runtime guards", () => {
  it("does not register native menu listeners outside a Tauri runtime", async () => {
    const deps = new Proxy(
      {},
      {
        get: () => vi.fn(),
      }
    ) as UseMenuActionsDeps;

    createRoot((dispose) => {
      useMenuActions(deps);
      dispose();
    });
    await Promise.resolve();

    expect(mockListen).not.toHaveBeenCalledWith("menu-action", expect.any(Function));
  });
});
