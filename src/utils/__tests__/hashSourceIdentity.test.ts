// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it } from "vitest";
import { buildLocalFileHashSourceFields } from "../hashSourceIdentity";

describe("buildLocalFileHashSourceFields", () => {
  it("uses the local file path as the canonical source identity", () => {
    const fields = buildLocalFileHashSourceFields("/case/evidence/photo.jpg");

    expect(fields.sourceId).toBe("/case/evidence/photo.jpg");
    expect(JSON.parse(fields.sourceRefJson ?? "")).toEqual({
      kind: "localFile",
      path: "/case/evidence/photo.jpg",
    });
  });
});
