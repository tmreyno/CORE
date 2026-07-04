// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { createSignal } from "solid-js";

const sharedHooks = vi.hoisted(() => ({
  useCloseConfirmation: vi.fn(),
  confirmUnsavedChanges: vi.fn(),
}));

vi.mock("@core-suite/desktop-hooks", () => ({
  useCloseConfirmation: sharedHooks.useCloseConfirmation,
  confirmUnsavedChanges: sharedHooks.confirmUnsavedChanges,
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

import { useCloseConfirmation, confirmUnsavedChanges } from "../useCloseConfirmation";

beforeEach(() => {
  vi.clearAllMocks();
  sharedHooks.confirmUnsavedChanges.mockResolvedValue("cancel");
});

describe("useCloseConfirmation", () => {
  it("passes close options to the shared desktop hook", () => {
    const [hasUnsavedChanges] = createSignal(true);
    const onSave = vi.fn().mockResolvedValue(true);
    const onClose = vi.fn();

    useCloseConfirmation({
      hasUnsavedChanges,
      onSave,
      onClose,
      dialogTitle: "Save Project?",
      dialogMessage: "Would you like to save?",
    });

    expect(sharedHooks.useCloseConfirmation).toHaveBeenCalledTimes(1);
    const [options, config] = sharedHooks.useCloseConfirmation.mock.calls[0];
    expect(options.hasUnsavedChanges).toBe(hasUnsavedChanges);
    expect(options.onSave).toBe(onSave);
    expect(options.onClose).toBe(onClose);
    expect(options.dialogTitle).toBe("Save Project?");
    expect(options.dialogMessage).toBe("Would you like to save?");
    expect(config).toEqual(
      expect.objectContaining({
        log: expect.any(Object),
      })
    );
  });

  it("passes through minimal close options", () => {
    const [hasUnsavedChanges] = createSignal(false);

    useCloseConfirmation({ hasUnsavedChanges });

    const [options] = sharedHooks.useCloseConfirmation.mock.calls[0];
    expect(options.hasUnsavedChanges).toBe(hasUnsavedChanges);
    expect(options.onSave).toBeUndefined();
    expect(options.onClose).toBeUndefined();
    expect(options.dialogTitle).toBeUndefined();
    expect(options.dialogMessage).toBeUndefined();
  });
});

describe("confirmUnsavedChanges", () => {
  it("returns the shared helper result", async () => {
    sharedHooks.confirmUnsavedChanges.mockResolvedValueOnce("save");

    await expect(confirmUnsavedChanges()).resolves.toBe("save");
    expect(sharedHooks.confirmUnsavedChanges).toHaveBeenCalledWith(
      undefined,
      expect.objectContaining({
        log: expect.any(Object),
      })
    );
  });

  it("passes custom dialog text to the shared helper", async () => {
    sharedHooks.confirmUnsavedChanges.mockResolvedValueOnce("discard");
    const options = {
      title: "Leave Page?",
      message: "Changes will be lost.",
    };

    await expect(confirmUnsavedChanges(options)).resolves.toBe("discard");
    expect(sharedHooks.confirmUnsavedChanges).toHaveBeenCalledWith(
      options,
      expect.objectContaining({
        log: expect.any(Object),
      })
    );
  });

  it("can return cancel from the shared helper", async () => {
    sharedHooks.confirmUnsavedChanges.mockResolvedValueOnce("cancel");

    await expect(confirmUnsavedChanges()).resolves.toBe("cancel");
  });
});
