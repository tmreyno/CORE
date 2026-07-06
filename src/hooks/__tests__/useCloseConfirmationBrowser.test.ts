// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { afterEach, describe, expect, it, vi } from "vitest";
import { confirmUnsavedChanges, useCloseConfirmation } from "../useCloseConfirmation";

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

vi.mock("../../utils/logger", () => ({
  logger: {
    scope: () => ({
      debug: vi.fn(),
      warn: vi.fn(),
      info: vi.fn(),
      error: vi.fn(),
    }),
  },
}));

vi.mock("@core-suite/desktop-hooks", () => ({
  useCloseConfirmation: vi.fn(),
  confirmUnsavedChanges: vi.fn(),
}));

afterEach(() => {
  vi.restoreAllMocks();
});

describe("browser close confirmation guards", () => {
  it("does not register native close confirmation outside Tauri", () => {
    expect(useCloseConfirmation({ hasUnsavedChanges: () => true })).toBeUndefined();
  });

  it("returns discard when the user confirms the browser discard prompt", async () => {
    const confirm = vi.spyOn(window, "confirm").mockReturnValueOnce(true);

    await expect(confirmUnsavedChanges({
      title: "Save Project Before Closing?",
      message: "The current project has unsaved changes.",
    })).resolves.toBe("discard");

    expect(confirm).toHaveBeenCalledWith(expect.stringContaining("Continue and discard changes?"));
  });

  it("returns cancel when the user rejects the browser discard prompt", async () => {
    vi.spyOn(window, "confirm").mockReturnValueOnce(false);

    await expect(confirmUnsavedChanges()).resolves.toBe("cancel");
  });
});
