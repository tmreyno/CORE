// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { createRoot, createSignal } from "solid-js";

// ---------------------------------------------------------------------------
// Mocks
// ---------------------------------------------------------------------------

type CloseHandler = (event: { preventDefault: () => void }) => Promise<void>;
let capturedCloseHandler: CloseHandler | null = null;

const mockClose = vi.fn().mockResolvedValue(undefined);
const mockOnCloseRequested = vi.fn(async (handler: CloseHandler) => {
  capturedCloseHandler = handler;
  return () => { capturedCloseHandler = null; };
});

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    onCloseRequested: mockOnCloseRequested,
    close: mockClose,
  }),
}));

const mockConfirm = vi.fn<(message: string, options?: unknown) => Promise<boolean>>();

vi.mock("@tauri-apps/plugin-dialog", () => ({
  confirm: (...args: unknown[]) => mockConfirm(args[0] as string, args[1]),
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

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

beforeEach(() => {
  vi.clearAllMocks();
  capturedCloseHandler = null;
});

// ---------------------------------------------------------------------------
// useCloseConfirmation
// ---------------------------------------------------------------------------

describe("useCloseConfirmation", () => {
  it("registers a close listener on mount", async () => {
    await createRoot(async (dispose) => {
      const [hasUnsaved] = createSignal(false);
      useCloseConfirmation({ hasUnsavedChanges: hasUnsaved });

      // Wait for onMount async to complete
      await vi.waitFor(() => {
        expect(mockOnCloseRequested).toHaveBeenCalled();
      });
      dispose();
    });
  });

  it("allows close immediately when no unsaved changes", async () => {
    await createRoot(async (dispose) => {
      const [hasUnsaved] = createSignal(false);
      const onClose = vi.fn();
      useCloseConfirmation({ hasUnsavedChanges: hasUnsaved, onClose });

      await vi.waitFor(() => {
        expect(capturedCloseHandler).not.toBeNull();
      });

      const preventDefault = vi.fn();
      await capturedCloseHandler!({ preventDefault });

      expect(preventDefault).not.toHaveBeenCalled();
      expect(mockConfirm).not.toHaveBeenCalled();
      expect(onClose).toHaveBeenCalled();
      dispose();
    });
  });

  it("shows confirmation dialog when there are unsaved changes", async () => {
    await createRoot(async (dispose) => {
      const [hasUnsaved] = createSignal(true);
      mockConfirm.mockResolvedValueOnce(false);

      useCloseConfirmation({ hasUnsavedChanges: hasUnsaved });

      await vi.waitFor(() => {
        expect(capturedCloseHandler).not.toBeNull();
      });

      const preventDefault = vi.fn();
      await capturedCloseHandler!({ preventDefault });

      expect(preventDefault).toHaveBeenCalled();
      expect(mockConfirm).toHaveBeenCalledWith(
        "You have unsaved changes. Do you want to save before closing?",
        expect.objectContaining({
          title: "Unsaved Changes",
          kind: "warning",
        })
      );
      dispose();
    });
  });

  it("uses custom dialog title and message", async () => {
    await createRoot(async (dispose) => {
      const [hasUnsaved] = createSignal(true);
      mockConfirm.mockResolvedValueOnce(false);

      useCloseConfirmation({
        hasUnsavedChanges: hasUnsaved,
        dialogTitle: "Save Project?",
        dialogMessage: "Would you like to save?",
      });

      await vi.waitFor(() => {
        expect(capturedCloseHandler).not.toBeNull();
      });

      const preventDefault = vi.fn();
      await capturedCloseHandler!({ preventDefault });

      expect(mockConfirm).toHaveBeenCalledWith(
        "Would you like to save?",
        expect.objectContaining({
          title: "Save Project?",
        })
      );
      dispose();
    });
  });

  it("saves and closes when user chooses save and save succeeds", async () => {
    await createRoot(async (dispose) => {
      const [hasUnsaved] = createSignal(true);
      const onSave = vi.fn().mockResolvedValue(true);
      const onClose = vi.fn();
      mockConfirm.mockResolvedValueOnce(true);

      useCloseConfirmation({ hasUnsavedChanges: hasUnsaved, onSave, onClose });

      await vi.waitFor(() => {
        expect(capturedCloseHandler).not.toBeNull();
      });

      await capturedCloseHandler!({ preventDefault: vi.fn() });

      expect(onSave).toHaveBeenCalled();
      expect(onClose).toHaveBeenCalled();
      expect(mockClose).toHaveBeenCalled();
      dispose();
    });
  });

  it("does not close when user chooses save but save fails", async () => {
    await createRoot(async (dispose) => {
      const [hasUnsaved] = createSignal(true);
      const onSave = vi.fn().mockResolvedValue(false);
      const onClose = vi.fn();
      mockConfirm.mockResolvedValueOnce(true);

      useCloseConfirmation({ hasUnsavedChanges: hasUnsaved, onSave, onClose });

      await vi.waitFor(() => {
        expect(capturedCloseHandler).not.toBeNull();
      });

      await capturedCloseHandler!({ preventDefault: vi.fn() });

      expect(onSave).toHaveBeenCalled();
      expect(onClose).not.toHaveBeenCalled();
      expect(mockClose).not.toHaveBeenCalled();
      dispose();
    });
  });

  it("discards and closes when user chooses discard", async () => {
    await createRoot(async (dispose) => {
      const [hasUnsaved] = createSignal(true);
      const onSave = vi.fn();
      const onClose = vi.fn();
      mockConfirm.mockResolvedValueOnce(false);

      useCloseConfirmation({ hasUnsavedChanges: hasUnsaved, onSave, onClose });

      await vi.waitFor(() => {
        expect(capturedCloseHandler).not.toBeNull();
      });

      await capturedCloseHandler!({ preventDefault: vi.fn() });

      expect(onSave).not.toHaveBeenCalled();
      expect(onClose).toHaveBeenCalled();
      expect(mockClose).toHaveBeenCalled();
      dispose();
    });
  });

  it("closes without save handler when user chooses save but no onSave provided", async () => {
    await createRoot(async (dispose) => {
      const [hasUnsaved] = createSignal(true);
      const onClose = vi.fn();
      mockConfirm.mockResolvedValueOnce(true);

      useCloseConfirmation({ hasUnsavedChanges: hasUnsaved, onClose });

      await vi.waitFor(() => {
        expect(capturedCloseHandler).not.toBeNull();
      });

      await capturedCloseHandler!({ preventDefault: vi.fn() });

      // No onSave, so it falls through to discard path
      expect(onClose).toHaveBeenCalled();
      expect(mockClose).toHaveBeenCalled();
      dispose();
    });
  });
});

// ---------------------------------------------------------------------------
// confirmUnsavedChanges (standalone utility)
// ---------------------------------------------------------------------------

describe("confirmUnsavedChanges", () => {
  it("returns 'save' when user confirms", async () => {
    mockConfirm.mockResolvedValueOnce(true);
    const result = await confirmUnsavedChanges();
    expect(result).toBe("save");
  });

  it("returns 'discard' when user cancels", async () => {
    mockConfirm.mockResolvedValueOnce(false);
    const result = await confirmUnsavedChanges();
    expect(result).toBe("discard");
  });

  it("uses default title and message", async () => {
    mockConfirm.mockResolvedValueOnce(true);
    await confirmUnsavedChanges();
    expect(mockConfirm).toHaveBeenCalledWith(
      "You have unsaved changes. What would you like to do?",
      expect.objectContaining({
        title: "Unsaved Changes",
        kind: "warning",
      })
    );
  });

  it("uses custom title and message", async () => {
    mockConfirm.mockResolvedValueOnce(true);
    await confirmUnsavedChanges({
      title: "Leave Page?",
      message: "Changes will be lost.",
    });
    expect(mockConfirm).toHaveBeenCalledWith(
      "Changes will be lost.",
      expect.objectContaining({ title: "Leave Page?" })
    );
  });

  it("returns 'cancel' on error", async () => {
    mockConfirm.mockRejectedValueOnce(new Error("dialog error"));
    const result = await confirmUnsavedChanges();
    expect(result).toBe("cancel");
  });
});
