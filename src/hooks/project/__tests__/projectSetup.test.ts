// =============================================================================
// projectSetup — createDocumentEntry tests
// =============================================================================

import { describe, it, expect } from "vitest";
import { createDocumentEntry } from "../projectSetup";
import type { CaseDocument } from "../../../types";

function makeDoc(overrides: Partial<CaseDocument> = {}): CaseDocument {
  return {
    path: "/evidence/report.pdf",
    filename: "report.pdf",
    size: 12345,
    format: "pdf",
    modified: "2026-01-15",
    document_type: "forensic_report",
    case_number: "C-001",
    evidence_id: "EV-01",
    ...overrides,
  } as CaseDocument;
}

describe("createDocumentEntry", () => {
  it("maps CaseDocument to SelectedEntry", () => {
    const entry = createDocumentEntry(makeDoc());
    expect(entry.containerPath).toBe("/evidence/report.pdf");
    expect(entry.entryPath).toBe("/evidence/report.pdf");
    expect(entry.name).toBe("report.pdf");
    expect(entry.size).toBe(12345);
    expect(entry.isDir).toBe(false);
  });

  it("defaults isDiskFile to true", () => {
    const entry = createDocumentEntry(makeDoc());
    expect(entry.isDiskFile).toBe(true);
  });

  it("accepts isDiskFile = false", () => {
    const entry = createDocumentEntry(makeDoc(), false);
    expect(entry.isDiskFile).toBe(false);
  });

  it("sets containerType from format", () => {
    const entry = createDocumentEntry(makeDoc({ format: "docx" }));
    expect(entry.containerType).toBe("docx");
  });

  it("falls back to 'file' when format is undefined", () => {
    const entry = createDocumentEntry(makeDoc({ format: undefined }));
    expect(entry.containerType).toBe("file");
  });

  it("populates metadata from document fields", () => {
    const entry = createDocumentEntry(makeDoc());
    expect(entry.metadata?.document_type).toBe("forensic_report");
    expect(entry.metadata?.case_number).toBe("C-001");
    expect(entry.metadata?.evidence_id).toBe("EV-01");
    expect(entry.metadata?.format).toBe("pdf");
    expect(entry.metadata?.modified).toBe("2026-01-15");
  });

  it("handles missing optional metadata fields", () => {
    const entry = createDocumentEntry(
      makeDoc({ document_type: undefined, case_number: undefined, evidence_id: undefined }),
    );
    expect(entry.metadata?.document_type).toBeUndefined();
    expect(entry.metadata?.case_number).toBeUndefined();
    expect(entry.metadata?.evidence_id).toBeUndefined();
  });
});
