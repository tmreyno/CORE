// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it, vi } from "vitest";
import { listenForArtifactsUpdated, notifyArtifactsUpdated } from "../artifactEvents";

describe("artifact update events", () => {
  it("notifies listeners with evidence and category details", () => {
    const listener = vi.fn();
    const unlisten = listenForArtifactsUpdated(listener);

    notifyArtifactsUpdated({
      evidenceFileId: "/evidence/disk.E01",
      category: "systeminfo",
    });

    expect(listener).toHaveBeenCalledWith({
      evidenceFileId: "/evidence/disk.E01",
      category: "systeminfo",
    });

    unlisten();
  });
});
