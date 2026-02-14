// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { getFormatIcon } from "../documentHelpers";

// =============================================================================
// getFormatIcon
// =============================================================================
describe("getFormatIcon", () => {
  it('returns 📄 for PDF', () => {
    expect(getFormatIcon("pdf")).toBe("📄");
    expect(getFormatIcon("PDF")).toBe("📄");
  });

  it('returns 📝 for Word documents', () => {
    expect(getFormatIcon("docx")).toBe("📝");
    expect(getFormatIcon("doc")).toBe("📝");
    expect(getFormatIcon("DOCX")).toBe("📝");
  });

  it('returns 🌐 for HTML', () => {
    expect(getFormatIcon("html")).toBe("🌐");
    expect(getFormatIcon("htm")).toBe("🌐");
    expect(getFormatIcon("HTML")).toBe("🌐");
  });

  it('returns 📋 for Markdown', () => {
    expect(getFormatIcon("markdown")).toBe("📋");
    expect(getFormatIcon("md")).toBe("📋");
    expect(getFormatIcon("MD")).toBe("📋");
  });

  it('returns 📃 for unknown formats', () => {
    expect(getFormatIcon("xlsx")).toBe("📃");
    expect(getFormatIcon("csv")).toBe("📃");
    expect(getFormatIcon("txt")).toBe("📃");
    expect(getFormatIcon("")).toBe("📃");
  });
});
