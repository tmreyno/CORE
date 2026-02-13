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

const mockSetTitle = vi.fn().mockResolvedValue(undefined);

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    setTitle: mockSetTitle,
  }),
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

import { useWindowTitle, setWindowTitle } from "../useWindowTitle";

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

beforeEach(() => {
  vi.clearAllMocks();
});

// ---------------------------------------------------------------------------
// useWindowTitle hook
// ---------------------------------------------------------------------------

describe("useWindowTitle", () => {
  it("sets title to APP_NAME when no project is open", async () => {
    await createRoot(async (dispose) => {
      const [projectName] = createSignal<string | null>(null);
      const [modified] = createSignal(false);

      useWindowTitle({ projectName, modified });

      // Allow effect to run
      await vi.waitFor(() => {
        expect(mockSetTitle).toHaveBeenCalledWith("CORE-FFX");
      });
      dispose();
    });
  });

  it("sets title with project name when project is open", async () => {
    await createRoot(async (dispose) => {
      const [projectName] = createSignal<string | null>("My Case");
      const [modified] = createSignal(false);

      useWindowTitle({ projectName, modified });

      await vi.waitFor(() => {
        expect(mockSetTitle).toHaveBeenCalledWith("My Case - CORE-FFX");
      });
      dispose();
    });
  });

  it("adds unsaved indicator when modified is true", async () => {
    await createRoot(async (dispose) => {
      const [projectName] = createSignal<string | null>("Evidence Review");
      const [modified] = createSignal(true);

      useWindowTitle({ projectName, modified });

      await vi.waitFor(() => {
        expect(mockSetTitle).toHaveBeenCalledWith("● Evidence Review - CORE-FFX");
      });
      dispose();
    });
  });

  it("does not add unsaved indicator when no project is open even if modified", async () => {
    await createRoot(async (dispose) => {
      const [projectName] = createSignal<string | null>(null);
      const [modified] = createSignal(true);

      useWindowTitle({ projectName, modified });

      await vi.waitFor(() => {
        expect(mockSetTitle).toHaveBeenCalledWith("CORE-FFX");
      });
      dispose();
    });
  });

  it("updates title reactively when project name changes", async () => {
    await createRoot(async (dispose) => {
      const [projectName, setProjectName] = createSignal<string | null>(null);
      const [modified] = createSignal(false);

      useWindowTitle({ projectName, modified });

      await vi.waitFor(() => {
        expect(mockSetTitle).toHaveBeenCalledWith("CORE-FFX");
      });

      mockSetTitle.mockClear();
      setProjectName("New Case");

      await vi.waitFor(() => {
        expect(mockSetTitle).toHaveBeenCalledWith("New Case - CORE-FFX");
      });
      dispose();
    });
  });

  it("updates title reactively when modified state changes", async () => {
    await createRoot(async (dispose) => {
      const [projectName] = createSignal<string | null>("Active Case");
      const [modified, setModified] = createSignal(false);

      useWindowTitle({ projectName, modified });

      await vi.waitFor(() => {
        expect(mockSetTitle).toHaveBeenCalledWith("Active Case - CORE-FFX");
      });

      mockSetTitle.mockClear();
      setModified(true);

      await vi.waitFor(() => {
        expect(mockSetTitle).toHaveBeenCalledWith("● Active Case - CORE-FFX");
      });
      dispose();
    });
  });

  it("accepts optional projectPath without affecting title", async () => {
    await createRoot(async (dispose) => {
      const [projectName] = createSignal<string | null>("Path Case");
      const [modified] = createSignal(false);
      const [projectPath] = createSignal<string | null>("/some/path/case.cffx");

      useWindowTitle({ projectName, modified, projectPath });

      await vi.waitFor(() => {
        expect(mockSetTitle).toHaveBeenCalledWith("Path Case - CORE-FFX");
      });
      dispose();
    });
  });

  it("handles setTitle failure gracefully", async () => {
    mockSetTitle.mockRejectedValueOnce(new Error("window error"));

    await createRoot(async (dispose) => {
      const [projectName] = createSignal<string | null>("Failing");
      const [modified] = createSignal(false);

      // Should not throw
      useWindowTitle({ projectName, modified });

      // Give effect time to run and handle error
      await new Promise((r) => setTimeout(r, 50));
      dispose();
    });
  });
});

// ---------------------------------------------------------------------------
// setWindowTitle utility
// ---------------------------------------------------------------------------

describe("setWindowTitle", () => {
  it("sets the window title directly", async () => {
    await setWindowTitle("Custom Title");
    expect(mockSetTitle).toHaveBeenCalledWith("Custom Title");
  });

  it("handles failure gracefully", async () => {
    mockSetTitle.mockRejectedValueOnce(new Error("fail"));
    // Should not throw
    await setWindowTitle("Will Fail");
  });
});
