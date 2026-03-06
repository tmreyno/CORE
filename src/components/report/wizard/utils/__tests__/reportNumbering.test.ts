// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock getPreference with controllable returns.
// For "reportNumberCounters" we read from localStorage so that nextCounter()
// writes are visible to subsequent getPreference() reads within the same test.
const mockPrefValues: Record<string, any> = {};
vi.mock("../../../../preferences", () => ({
  getPreference: (key: string) => {
    if (key === "reportNumberCounters") {
      // If test explicitly populated counters, use them directly
      const mock = mockPrefValues[key];
      if (mock && Object.keys(mock).length > 0) return mock;
      // Otherwise read from localStorage to reflect nextCounter() writes
      try {
        const stored = globalThis.localStorage?.getItem("ffx-preferences");
        if (stored) {
          const parsed = JSON.parse(stored);
          if (parsed.reportNumberCounters) return parsed.reportNumberCounters;
        }
      } catch { /* fall through */ }
      return mock ?? {};
    }
    return mockPrefValues[key];
  },
}));

// Provide localStorage stub if not available in test environment
const storage: Record<string, string> = {};
if (typeof globalThis.localStorage === "undefined" || typeof globalThis.localStorage?.setItem !== "function") {
  Object.defineProperty(globalThis, "localStorage", {
    value: {
      getItem: (k: string) => storage[k] ?? null,
      setItem: (k: string, v: string) => { storage[k] = v; },
      removeItem: (k: string) => { delete storage[k]; },
      clear: () => { for (const k of Object.keys(storage)) delete storage[k]; },
    },
    writable: true,
  });
}

import {
  generateReportNumber,
  previewReportNumber,
  generateCocNumber,
  previewCocNumber,
  generateEvidenceItemNumber,
  previewEvidenceItemNumber,
  peekCounter,
} from "../reportNumbering";

/**
 * Set up default mock preference values.
 * `nextCounter` reads from localStorage so we mock that too.
 */
beforeEach(() => {
  // Default preference values
  mockPrefValues.reportNumberPrefixes = {
    forensic: "FR",
    analytical: "AR",
  };
  mockPrefValues.reportNumberIncludeYear = true;
  mockPrefValues.reportNumberSeqDigits = 4;
  mockPrefValues.reportNumberCounters = {};
  mockPrefValues.cocNumberPattern = "{case}-COC-{seq}";
  mockPrefValues.evidenceItemPattern = "{case}-EV-{seq}";
  mockPrefValues.defaultAgency = "FBI";

  // Provide localStorage with matching data
  const stored = {
    reportNumberPrefixes: mockPrefValues.reportNumberPrefixes,
    reportNumberCounters: {},
    reportNumberIncludeYear: true,
    reportNumberSeqDigits: 4,
    cocNumberPattern: "{case}-COC-{seq}",
    evidenceItemPattern: "{case}-EV-{seq}",
    defaultAgency: "FBI",
  };
  localStorage.setItem("ffx-preferences", JSON.stringify(stored));
});

// =============================================================================
// peekCounter
// =============================================================================

describe("peekCounter", () => {
  it("returns 1 when counter scope has no prior value", () => {
    expect(peekCounter("newScope")).toBe(1);
  });

  it("returns next value when counter scope exists", () => {
    mockPrefValues.reportNumberCounters = { existingScope: 5 };
    expect(peekCounter("existingScope")).toBe(6);
  });
});

// =============================================================================
// generateReportNumber
// =============================================================================

describe("generateReportNumber", () => {
  it("generates with year and prefix for known type", () => {
    const num = generateReportNumber("forensic" as any);
    const year = new Date().getFullYear();
    expect(num).toBe(`FR-${year}-0001`);
  });

  it("falls back to RPT for unknown type", () => {
    const num = generateReportNumber("unknown" as any);
    const year = new Date().getFullYear();
    expect(num).toBe(`RPT-${year}-0001`);
  });

  it("generates without year when preference is false", () => {
    mockPrefValues.reportNumberIncludeYear = false;
    const num = generateReportNumber("forensic" as any);
    expect(num).toBe("FR-0001");
  });

  it("respects seqDigits setting", () => {
    mockPrefValues.reportNumberSeqDigits = 6;
    const num = generateReportNumber("forensic" as any);
    const year = new Date().getFullYear();
    expect(num).toBe(`FR-${year}-000001`);
  });

  it("increments counter on successive calls", () => {
    generateReportNumber("forensic" as any);
    const second = generateReportNumber("forensic" as any);
    const year = new Date().getFullYear();
    expect(second).toBe(`FR-${year}-0002`);
  });
});

// =============================================================================
// previewReportNumber
// =============================================================================

describe("previewReportNumber", () => {
  it("previews next number without consuming the counter", () => {
    const preview1 = previewReportNumber("forensic" as any);
    const preview2 = previewReportNumber("forensic" as any);
    expect(preview1).toBe(preview2);
  });

  it("matches what generate would produce", () => {
    const preview = previewReportNumber("forensic" as any);
    const generated = generateReportNumber("forensic" as any);
    expect(preview).toBe(generated);
  });
});

// =============================================================================
// generateCocNumber
// =============================================================================

describe("generateCocNumber", () => {
  it("generates COC number with case number", () => {
    const num = generateCocNumber("2024-001");
    expect(num).toBe("2024-001-COC-0001");
  });

  it("uses '0000' when case number is undefined", () => {
    const num = generateCocNumber();
    expect(num).toBe("0000-COC-0001");
  });

  it("supports {agency} and {year} tokens", () => {
    mockPrefValues.cocNumberPattern = "{agency}-{year}-COC-{seq}";
    const year = new Date().getFullYear();
    const num = generateCocNumber();
    expect(num).toBe(`FBI-${year}-COC-0001`);
  });

  it("uses AGY when agency is empty", () => {
    mockPrefValues.cocNumberPattern = "{agency}-COC-{seq}";
    mockPrefValues.defaultAgency = "";
    const num = generateCocNumber();
    expect(num).toBe("AGY-COC-0001");
  });
});

// =============================================================================
// previewCocNumber
// =============================================================================

describe("previewCocNumber", () => {
  it("previews without consuming counter", () => {
    const p1 = previewCocNumber("C-1");
    const p2 = previewCocNumber("C-1");
    expect(p1).toBe(p2);
  });
});

// =============================================================================
// generateEvidenceItemNumber
// =============================================================================

describe("generateEvidenceItemNumber", () => {
  it("generates evidence number with case number", () => {
    const num = generateEvidenceItemNumber("2024-001");
    expect(num).toBe("2024-001-EV-0001");
  });

  it("uses '0000' when case number is undefined", () => {
    const num = generateEvidenceItemNumber();
    expect(num).toBe("0000-EV-0001");
  });

  it("increments on successive calls", () => {
    generateEvidenceItemNumber("C");
    const second = generateEvidenceItemNumber("C");
    expect(second).toBe("C-EV-0002");
  });
});

// =============================================================================
// previewEvidenceItemNumber
// =============================================================================

describe("previewEvidenceItemNumber", () => {
  it("previews without consuming counter", () => {
    const p1 = previewEvidenceItemNumber("C-1");
    const p2 = previewEvidenceItemNumber("C-1");
    expect(p1).toBe(p2);
  });
});
