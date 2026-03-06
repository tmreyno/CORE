// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { formatTimeAgo } from "../helpers";

// =============================================================================
// formatTimeAgo
// =============================================================================

describe("formatTimeAgo", () => {
  it("returns 'just now' for recent dates", () => {
    expect(formatTimeAgo(new Date())).toBe("just now");
  });

  it("returns 'just now' for dates within 59 seconds", () => {
    const d = new Date(Date.now() - 30 * 1000);
    expect(formatTimeAgo(d)).toBe("just now");
  });

  it("formats minutes ago", () => {
    const d = new Date(Date.now() - 5 * 60 * 1000);
    expect(formatTimeAgo(d)).toBe("5m ago");
  });

  it("formats hours ago", () => {
    const d = new Date(Date.now() - 3 * 60 * 60 * 1000);
    expect(formatTimeAgo(d)).toBe("3h ago");
  });

  it("formats days ago", () => {
    const d = new Date(Date.now() - 7 * 24 * 60 * 60 * 1000);
    expect(formatTimeAgo(d)).toBe("7d ago");
  });

  it("falls back to locale date for dates > 30 days", () => {
    const d = new Date(Date.now() - 45 * 24 * 60 * 60 * 1000);
    const result = formatTimeAgo(d);
    // Should be a locale date string, not "Xd ago"
    expect(result).not.toContain("ago");
  });
});
