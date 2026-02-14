// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { getTabTypeColor } from "../tabHelpers";

describe("getTabTypeColor", () => {
  it('returns "text-type-ad1" for evidence', () => {
    expect(getTabTypeColor("evidence")).toBe("text-type-ad1");
  });

  it('returns "text-accent" for document', () => {
    expect(getTabTypeColor("document")).toBe("text-accent");
  });

  it('returns "text-type-e01" for entry', () => {
    expect(getTabTypeColor("entry")).toBe("text-type-e01");
  });

  it('returns "text-warning" for export', () => {
    expect(getTabTypeColor("export")).toBe("text-warning");
  });

  it('returns "text-success" for processed', () => {
    expect(getTabTypeColor("processed")).toBe("text-success");
  });

  it('returns "text-txt-muted" for unknown type', () => {
    expect(getTabTypeColor("unknown" as any)).toBe("text-txt-muted");
  });
});
