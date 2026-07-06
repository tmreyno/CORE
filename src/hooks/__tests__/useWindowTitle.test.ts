// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { createSignal } from "solid-js";

const sharedHooks = vi.hoisted(() => ({
  useWindowTitle: vi.fn(),
  setWindowTitle: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@core-suite/desktop-hooks", () => ({
  useWindowTitle: sharedHooks.useWindowTitle,
  setWindowTitle: sharedHooks.setWindowTitle,
}));

vi.mock("../../utils/platform", () => ({
  isTauri: true,
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

beforeEach(() => {
  vi.clearAllMocks();
  sharedHooks.setWindowTitle.mockResolvedValue(undefined);
});

describe("useWindowTitle", () => {
  it("passes project state accessors to the shared desktop hook", () => {
    const [projectName] = createSignal<string | null>("My Case");
    const [modified] = createSignal(false);
    const [projectPath] = createSignal<string | null>("/cases/my-case.cffx");

    useWindowTitle({ projectName, modified, projectPath });

    expect(sharedHooks.useWindowTitle).toHaveBeenCalledTimes(1);
    const [options, config] = sharedHooks.useWindowTitle.mock.calls[0];
    expect(options.projectName).toBe(projectName);
    expect(options.modified).toBe(modified);
    expect(options.projectPath).toBe(projectPath);
    expect(config).toEqual(
      expect.objectContaining({
        appName: "CORE-FFX",
        log: expect.any(Object),
      })
    );
  });

  it("passes through missing optional projectPath", () => {
    const [projectName] = createSignal<string | null>(null);
    const [modified] = createSignal(true);

    useWindowTitle({ projectName, modified });

    const [options] = sharedHooks.useWindowTitle.mock.calls[0];
    expect(options.projectName).toBe(projectName);
    expect(options.modified).toBe(modified);
    expect(options.projectPath).toBeUndefined();
  });
});

describe("setWindowTitle", () => {
  it("delegates direct title updates to the shared desktop helper", async () => {
    await setWindowTitle("Custom Title");

    expect(sharedHooks.setWindowTitle).toHaveBeenCalledWith(
      "Custom Title",
      expect.objectContaining({
        log: expect.any(Object),
      })
    );
  });

  it("returns the shared helper promise", async () => {
    sharedHooks.setWindowTitle.mockResolvedValueOnce(undefined);

    await expect(setWindowTitle("Next Title")).resolves.toBeUndefined();
  });
});
