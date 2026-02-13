// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { getDocumentTypeLabel, getDocumentTypeIcon } from "../discovery";

// =============================================================================
// Discovery Helper Tests
// =============================================================================

describe("Discovery Helpers", () => {
  describe("getDocumentTypeLabel", () => {
    it("should return 'Chain of Custody' for ChainOfCustody type", () => {
      expect(getDocumentTypeLabel("ChainOfCustody")).toBe("Chain of Custody");
    });

    it("should return 'Evidence Intake' for EvidenceIntake type", () => {
      expect(getDocumentTypeLabel("EvidenceIntake")).toBe("Evidence Intake");
    });

    it("should return 'Case Notes' for CaseNotes type", () => {
      expect(getDocumentTypeLabel("CaseNotes")).toBe("Case Notes");
    });

    it("should return 'Evidence Receipt' for EvidenceReceipt type", () => {
      expect(getDocumentTypeLabel("EvidenceReceipt")).toBe("Evidence Receipt");
    });

    it("should return 'Lab Request' for LabRequest type", () => {
      expect(getDocumentTypeLabel("LabRequest")).toBe("Lab Request");
    });

    it("should return 'External Report' for ExternalReport type", () => {
      expect(getDocumentTypeLabel("ExternalReport")).toBe("External Report");
    });

    it("should return 'Other Document' for Other type", () => {
      expect(getDocumentTypeLabel("Other")).toBe("Other Document");
    });

    it("should return 'Other Document' for unknown types", () => {
      // @ts-expect-error testing invalid input
      expect(getDocumentTypeLabel("InvalidType")).toBe("Other Document");
    });

    it("should have unique labels for each known type", () => {
      const types = ["ChainOfCustody", "EvidenceIntake", "CaseNotes", "EvidenceReceipt", "LabRequest", "ExternalReport", "Other"] as const;
      const labels = types.map(t => getDocumentTypeLabel(t));
      const unique = new Set(labels);
      expect(unique.size).toBe(types.length);
    });
  });

  describe("getDocumentTypeIcon", () => {
    it("should return clipboard icon for ChainOfCustody", () => {
      expect(getDocumentTypeIcon("ChainOfCustody")).toBe("📋");
    });

    it("should return memo icon for EvidenceIntake", () => {
      expect(getDocumentTypeIcon("EvidenceIntake")).toBe("📝");
    });

    it("should return notebook icon for CaseNotes", () => {
      expect(getDocumentTypeIcon("CaseNotes")).toBe("📓");
    });

    it("should return receipt icon for EvidenceReceipt", () => {
      expect(getDocumentTypeIcon("EvidenceReceipt")).toBe("🧾");
    });

    it("should return microscope icon for LabRequest", () => {
      expect(getDocumentTypeIcon("LabRequest")).toBe("🔬");
    });

    it("should return document icon for ExternalReport", () => {
      expect(getDocumentTypeIcon("ExternalReport")).toBe("📄");
    });

    it("should return folder icon for Other type", () => {
      expect(getDocumentTypeIcon("Other")).toBe("📁");
    });

    it("should return folder icon for unknown types", () => {
      // @ts-expect-error testing invalid input
      expect(getDocumentTypeIcon("UnknownType")).toBe("📁");
    });

    it("should return non-empty emoji string for all types", () => {
      const types = ["ChainOfCustody", "EvidenceIntake", "CaseNotes", "EvidenceReceipt", "LabRequest", "ExternalReport", "Other"] as const;
      for (const type of types) {
        const icon = getDocumentTypeIcon(type);
        expect(icon.length).toBeGreaterThan(0);
      }
    });
  });
});
