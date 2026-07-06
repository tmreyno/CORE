// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi } from "vitest";
import { buildForensicReport } from "../reportBuilder";
import type { ReportBuilderParams } from "../reportBuilder";
import type { TimelineEvent } from "../../../types";
import type { ProjectDbReportEvidence } from "../../../../../report/api";

// Mock preferences
vi.mock("../../../../preferences", () => ({
  getPreference: vi.fn(() => true),
}));

function makeAccessor<T>(value: T) {
  return () => value;
}

function makeMinimalParams(
  overrides: Partial<ReportBuilderParams> = {},
): ReportBuilderParams {
  return {
    metadata: makeAccessor({
      title: "Test Report",
      report_number: "FR-2025-0001",
      version: "1.0",
      classification: "Internal" as const,
      generated_at: "2025-01-15T10:00:00Z",
      generated_by: "FFX",
    }),
    caseInfo: makeAccessor({ case_number: "CASE-001" }),
    examiner: makeAccessor({ name: "Test Examiner", certifications: [] }),
    executiveSummary: makeAccessor(""),
    scope: makeAccessor(""),
    methodology: makeAccessor(""),
    conclusions: makeAccessor(""),
    findings: makeAccessor([]),
    chainOfCustody: makeAccessor([]),
    groupedEvidence: makeAccessor([]),
    selectedEvidence: makeAccessor(new Set<string>()),
    examinerSignature: makeAccessor(""),
    examinerSignedDate: makeAccessor(""),
    supervisorName: makeAccessor(""),
    supervisorSignature: makeAccessor(""),
    supervisorSignedDate: makeAccessor(""),
    digitalSignatureConfirmed: makeAccessor(false),
    approvalNotes: makeAccessor(""),
    fileInfoMap: new Map(),
    fileHashMap: new Map(),
    ...overrides,
  };
}

describe("buildForensicReport", () => {
  describe("basic report generation", () => {
    it("generates a report with required fields", () => {
      const report = buildForensicReport(makeMinimalParams());
      expect(report.metadata.title).toBe("Test Report");
      expect(report.case_info.case_number).toBe("CASE-001");
      expect(report.examiner.name).toBe("Test Examiner");
    });

    it("includes tools info", () => {
      const report = buildForensicReport(makeMinimalParams());
      expect(report.tools).toHaveLength(1);
      expect(report.tools[0].name).toBe("FFX - Forensic File Xplorer");
    });
  });

  describe("timeline from project activity log", () => {
    it("returns empty timeline when no projectTimeline provided", () => {
      const report = buildForensicReport(makeMinimalParams());
      expect(report.timeline).toEqual([]);
    });

    it("returns empty timeline when projectTimeline accessor returns empty", () => {
      const report = buildForensicReport(
        makeMinimalParams({
          projectTimeline: makeAccessor([]),
        }),
      );
      expect(report.timeline).toEqual([]);
    });

    it("includes timeline events from projectTimeline", () => {
      const events: TimelineEvent[] = [
        {
          timestamp: "2025-01-15T10:00:00Z",
          event_type: "file",
          description: "[open] Opened evidence.E01",
          source: "analyst1",
          artifact_path: "/evidence/evidence.E01",
        },
        {
          timestamp: "2025-01-15T11:00:00Z",
          event_type: "hash",
          description: "[compute] Computed SHA-256",
          source: "analyst1",
        },
      ];

      const report = buildForensicReport(
        makeMinimalParams({
          projectTimeline: makeAccessor(events),
        }),
      );

      expect(report.timeline).toHaveLength(2);
      expect(report.timeline[0].event_type).toBe("file");
      expect(report.timeline[0].description).toBe("[open] Opened evidence.E01");
      expect(report.timeline[0].source).toBe("analyst1");
      expect(report.timeline[0].artifact_path).toBe("/evidence/evidence.E01");
      expect(report.timeline[1].event_type).toBe("hash");
    });

    it("preserves all timeline event fields", () => {
      const event: TimelineEvent = {
        timestamp: "2025-03-01T09:00:00Z",
        event_type: "export",
        description: "[generate] Generated report",
        source: "examiner",
        evidence_ref: "EV001",
        artifact_path: "/output/report.pdf",
      };

      const report = buildForensicReport(
        makeMinimalParams({
          projectTimeline: makeAccessor([event]),
        }),
      );

      expect(report.timeline[0]).toEqual(event);
    });
  });

  describe("chain of custody passthrough", () => {
    it("includes chain of custody records", () => {
      const custody = [
        {
          timestamp: "2025-01-10T08:00:00Z",
          action: "Examination session",
          handler: "analyst1",
          location: "Lab-PC",
          notes: "Duration: 45 min (1.0.0)",
        },
      ];

      const report = buildForensicReport(
        makeMinimalParams({
          chainOfCustody: makeAccessor(custody),
        }),
      );

      expect(report.chain_of_custody).toHaveLength(1);
      expect(report.chain_of_custody[0].action).toBe("Examination session");
      expect(report.chain_of_custody[0].handler).toBe("analyst1");
    });
  });

  describe("project DB engine appendices", () => {
    it("includes project DB engine appendices when evidence is provided", () => {
      const projectDbEvidence: ProjectDbReportEvidence = {
        evidenceItems: [],
        hashRecords: [
          {
            item: "logical.ad1",
            source_id: "ad1:/case/logical.ad1:/docs/a.txt",
            source_ref: {
              kind: "ad1",
              container_path: "/case/logical.ad1",
              entry_path: "/docs/a.txt",
            },
            algorithm: "SHA256",
            value: "abcdef1234567890",
            computed_at: "2026-02-16T10:02:00Z",
            verified: true,
          },
        ],
        hashAlgorithmSummaries: [
          {
            algorithm: "SHA256",
            algorithmLabel: "SHA-256",
            count: 1,
            evidenceFileCount: 1,
            sourceCount: 1,
            latestComputedAt: null,
          },
        ],
        verificationResultSummaries: [],
        artifacts: [],
        artifactSummaries: [],
        artifactCategories: [],
        artifactEvidenceSummaries: [],
        artifactExtractorSummaries: [],
        sourceAnalyses: [],
        sourceAnalysisSummaries: [
          {
            id: "analysis-1",
            sourceId: "ad1:/case/logical.ad1:/docs/a.txt",
            totalSize: 42,
            totalSizeDisplay: "42 bytes",
            offset: 0,
            bytesAnalyzed: 42,
            bytesAnalyzedDisplay: "42 bytes",
            magicHex: "25 50 44 46",
            signatureCount: 1,
            primarySignature: "PDF Document",
            primaryMimeType: "application/pdf",
            primaryCategory: "document",
            entropy: 4.25,
            printableRatio: 0.75,
            isLikelyText: true,
            indicators: [],
            indicatorCount: 0,
            analyzedAt: "2026-02-16T10:03:00Z",
            analyzer: "core-source-analysis",
          },
        ],
        sourceAnalysisCategorySummaries: [
          {
            category: "document",
            count: 1,
            evidenceFileCount: 1,
            avgEntropy: 4.25,
            textLikeCount: 1,
            latestAnalyzedAt: "2026-02-16T10:03:00Z",
          },
        ],
        annotations: [
          {
            id: "ann-1",
            filePath: "ad1:/case/logical.ad1:/docs/a.txt",
            containerPath: "/case/logical.ad1",
            annotationType: "hex-magic",
            offsetStart: 0,
            offsetEnd: 16,
            lineStart: null,
            lineEnd: null,
            label: "Magic Bytes",
            content: "Initial signature bytes: 25 50 44 46",
            color: "#38bdf8",
            createdBy: "hex-viewer",
            createdAt: "2026-02-16T10:04:00Z",
            modifiedAt: "2026-02-16T10:04:00Z",
          },
        ],
      };

      const report = buildForensicReport(
        makeMinimalParams({
          projectDbEvidence: makeAccessor(projectDbEvidence),
        }),
      );

      expect(report.appendices.length).toBe(3);
      expect(report.appendices[0].title).toBe(
        "Project Hash and Verification Summary",
      );
      expect(report.appendices[0].content).toContain("Hash Records");
      expect(report.appendices[0].content).toContain(
        "ad1:/case/logical.ad1:/docs/a.txt",
      );
      expect(report.appendices[0].content).toContain("abcdef1234567890");
      expect(report.appendices[1].title).toBe("Source Analysis Summary");
      expect(report.appendices[1].content).toContain("PDF Document");
      expect(report.appendices[2].title).toBe(
        "Hex Review and Annotation Findings",
      );
      expect(report.appendices[2].content).toContain("hex-magic");
    });
  });

  describe("signatures", () => {
    it("includes examiner signature when provided", () => {
      const report = buildForensicReport(
        makeMinimalParams({
          examinerSignature: makeAccessor("John Doe"),
          examinerSignedDate: makeAccessor("2025-01-15"),
          digitalSignatureConfirmed: makeAccessor(true),
        }),
      );

      expect(report.signatures).toBeDefined();
      expect(report.signatures).toHaveLength(1);
      expect(report.signatures![0].role).toBe("examiner");
      expect(report.signatures![0].certified).toBe(true);
    });

    it("omits signatures when none provided", () => {
      const report = buildForensicReport(makeMinimalParams());
      expect(report.signatures).toBeUndefined();
    });
  });
});
