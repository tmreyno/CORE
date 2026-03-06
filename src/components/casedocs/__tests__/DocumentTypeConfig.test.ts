// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { getDocumentFormatIcon } from "../DocumentTypeConfig";

// =============================================================================
// getDocumentFormatIcon
// =============================================================================

describe("getDocumentFormatIcon", () => {
  it("returns red color for pdf", () => {
    const { color } = getDocumentFormatIcon("pdf");
    expect(color).toBe("text-red-400");
  });

  it("returns blue color for docx", () => {
    const { color } = getDocumentFormatIcon("docx");
    expect(color).toBe("text-blue-400");
  });

  it("returns green color for xlsx", () => {
    const { color } = getDocumentFormatIcon("xlsx");
    expect(color).toBe("text-green-400");
  });

  it("returns orange color for pptx", () => {
    const { color } = getDocumentFormatIcon("pptx");
    expect(color).toBe("text-orange-400");
  });

  it("returns purple color for jpg", () => {
    const { color } = getDocumentFormatIcon("jpg");
    expect(color).toBe("text-purple-400");
  });

  it("returns cyan color for eml", () => {
    const { color } = getDocumentFormatIcon("eml");
    expect(color).toBe("text-cyan-400");
  });

  it("returns amber color for zip", () => {
    const { color } = getDocumentFormatIcon("zip");
    expect(color).toBe("text-amber-400");
  });

  it("returns emerald color for sqlite", () => {
    const { color } = getDocumentFormatIcon("sqlite");
    expect(color).toBe("text-emerald-400");
  });

  it("returns indigo color for mp4", () => {
    const { color } = getDocumentFormatIcon("mp4");
    expect(color).toBe("text-indigo-400");
  });

  it("returns pink color for mp3", () => {
    const { color } = getDocumentFormatIcon("mp3");
    expect(color).toBe("text-pink-400");
  });

  it("handles case-insensitive format strings", () => {
    expect(getDocumentFormatIcon("PDF").color).toBe("text-red-400");
    expect(getDocumentFormatIcon("XLSX").color).toBe("text-green-400");
  });

  it("strips leading dot from format string", () => {
    expect(getDocumentFormatIcon(".pdf").color).toBe("text-red-400");
    expect(getDocumentFormatIcon(".docx").color).toBe("text-blue-400");
  });

  it("returns default config for unknown format", () => {
    const { color } = getDocumentFormatIcon("xyz123");
    expect(color).toBe("text-txt-muted");
  });

  it("returns a component icon for each known format", () => {
    const result = getDocumentFormatIcon("pdf");
    expect(typeof result.icon).toBe("function");
  });
});
