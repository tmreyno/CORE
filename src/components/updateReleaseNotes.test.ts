// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it } from "vitest";
import {
  getBundledUpdateReleaseNotes,
  mergeUpdateReleaseNotes,
} from "./updateReleaseNotes";

describe("update release notes", () => {
  it("shows 0.1.112 through 0.1.115 when updating from an older version", () => {
    const notes = getBundledUpdateReleaseNotes("0.1.115", "0.1.111");

    expect(notes).toContain("CORE-FFX Update Review");
    expect(notes).toContain("CORE-FFX 0.1.115");
    expect(notes).toContain("CORE-FFX 0.1.114");
    expect(notes).toContain("CORE-FFX 0.1.113");
    expect(notes).toContain("CORE-FFX 0.1.112");
    expect(notes.indexOf("CORE-FFX 0.1.115")).toBeLessThan(
      notes.indexOf("CORE-FFX 0.1.114")
    );
    expect(notes.indexOf("CORE-FFX 0.1.114")).toBeLessThan(
      notes.indexOf("CORE-FFX 0.1.113")
    );
    expect(notes.indexOf("CORE-FFX 0.1.113")).toBeLessThan(
      notes.indexOf("CORE-FFX 0.1.112")
    );
  });

  it("shows only the next bundled release when current version is known", () => {
    const notes = getBundledUpdateReleaseNotes("v0.1.114", "0.1.113");

    expect(notes).toContain("CORE-FFX 0.1.114");
    expect(notes).not.toContain("CORE-FFX 0.1.113");
    expect(notes).not.toContain("CORE-FFX 0.1.112");
  });

  it("keeps unknown future release bodies unchanged", () => {
    expect(
      mergeUpdateReleaseNotes("Remote release body", "0.1.116", "0.1.115")
    ).toBe("Remote release body");
  });

  it("uses bundled notes when the update manifest has no body", () => {
    const notes = mergeUpdateReleaseNotes("", "0.1.114", "0.1.111");

    expect(notes).toContain("Release Publishing Recovery");
    expect(notes).toContain("Project open repair");
    expect(notes).toContain("Single Cargo lockfile");
  });

  it("appends remote manifest notes after bundled notes", () => {
    const notes = mergeUpdateReleaseNotes(
      "Remote signing and asset notes.",
      "0.1.115",
      "0.1.114"
    );

    expect(notes).toContain("CORE-FFX 0.1.115");
    expect(notes).toContain("AD1 range reads");
    expect(notes).toContain("### Release Manifest Notes");
    expect(notes).toContain("Remote signing and asset notes.");
  });

  it("does not duplicate matching remote notes", () => {
    const remoteNotes = [
      "## CORE-FFX 0.1.115",
      "- **Release Publishing Recovery:** Existing note.",
    ].join("\n");

    expect(mergeUpdateReleaseNotes(remoteNotes, "0.1.115", "0.1.114")).toBe(
      remoteNotes
    );
  });
});
