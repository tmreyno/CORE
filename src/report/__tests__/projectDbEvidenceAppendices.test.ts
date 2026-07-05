// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it } from "vitest";
import {
  buildProjectDbEvidenceAppendices,
  type ProjectDbReportEvidence,
} from "../api";

function makeEvidence(): ProjectDbReportEvidence {
  return {
    evidenceItems: [],
    hashRecords: [],
    hashAlgorithmSummaries: [
      {
        algorithm: "SHA256",
        algorithmLabel: "SHA-256",
        count: 2,
        evidenceFileCount: 1,
        sourceCount: 2,
        latestComputedAt: "2026-02-16T10:00:00Z",
      },
    ],
    verificationResultSummaries: [
      {
        result: "match",
        count: 1,
        hashCount: 1,
        latestVerifiedAt: "2026-02-16T10:01:00Z",
      },
    ],
    artifacts: [],
    artifactSummaries: [
      {
        id: "artifact-1",
        sourceId: "ad1:/case/logical.ad1:/docs/a.txt",
        name: "a.txt",
        category: "text",
        typeDescription: "Plain Text",
        size: 42,
        sizeDisplay: "42 bytes",
        confidence: "high",
        isText: true,
        metadata: {
          "image.dimensions": "4032x3024",
          "exif.make": "CORE",
          "exif.model": "Camera 1",
          "exif.dateTimeOriginal": "2026:02:16 10:01:00",
          "gps.latitude": "37.774900",
          "gps.longitude": "-122.419400",
          "indicators.emailCount": "1",
          "indicators.emails": "admin@example.com",
        },
        extractor: "core-artifact-extractor",
        extractedAt: "2026-02-16T10:02:00Z",
      },
      {
        id: "artifact-2",
        sourceId: "ad1:/case/logical.ad1:/mail/message.eml",
        name: "message.eml",
        category: "email",
        typeDescription: "Email Message",
        size: 512,
        sizeDisplay: "512 bytes",
        confidence: "medium",
        isText: true,
        metadata: {
          "email.from": "Alice <alice@example.com>",
          "email.to": "Bob <bob@example.com>",
          "email.subject": "Quarterly update",
          "email.messageId": "<msg-1@example.com>",
          "email.attachmentCount": "1",
          "email.attachmentNames": "invoice.pdf",
        },
        extractor: "core-artifact-extractor",
        extractedAt: "2026-02-16T10:02:30Z",
      },
      {
        id: "artifact-3",
        sourceId:
          "ad1:/case/logical.ad1:/Library/LaunchAgents/com.core.ffx.agent.plist",
        name: "com.core.ffx.agent.plist",
        category: "config",
        typeDescription: "Apple Property List",
        size: 384,
        sizeDisplay: "384 bytes",
        confidence: "medium",
        isText: true,
        metadata: {
          "plist.format": "xml",
          "plist.rootType": "dictionary",
          "plist.topLevelKeys": "4",
          "plist.Label": "com.core.ffx.agent",
          "plist.ProgramArguments": "/usr/bin/core-ffx scan",
        },
        extractor: "core-artifact-extractor",
        extractedAt: "2026-02-16T10:02:45Z",
      },
      {
        id: "artifact-4",
        sourceId: "ad1:/case/logical.ad1:/Windows/System32/config/SAM",
        name: "SAM",
        category: "system",
        typeDescription: "Windows Registry Hive",
        size: 4096,
        sizeDisplay: "4 KB",
        confidence: "high",
        isText: false,
        metadata: {
          "registry.version": "1.5",
          "registry.dirty": "true",
          "registry.lastWriteTime": "1970-01-01T00:00:00Z",
          "registry.hiveBinsDataSize": "4096",
          "registry.path": "\\SystemRoot\\System32\\Config\\SAM",
        },
        extractor: "core-artifact-extractor",
        extractedAt: "2026-02-16T10:03:00Z",
      },
      {
        id: "artifact-5",
        sourceId: "e01:/case/disk.E01:/sys/class/dmi/id/product_serial",
        name: "product_serial",
        category: "systeminfo",
        typeDescription: "Linux DMI System Information",
        size: 8,
        sizeDisplay: "8 B",
        confidence: "medium",
        isText: true,
        metadata: {
          "system.osFamily": "linux",
          "system.manufacturer": "Dell Inc.",
          "system.model": "Precision 5680",
          "system.serialNumber": "ABC1234",
          "system.uuid": "00112233-4455-6677-8899-aabbccddeeff",
          "os.release.name": "Ubuntu 24.04.2 LTS",
        },
        extractor: "core-artifact-extractor",
        extractedAt: "2026-02-16T10:03:30Z",
      },
      {
        id: "artifact-6",
        sourceId: "ad1:/case/logical.ad1:/mobile/history.sqlite",
        name: "history.sqlite",
        category: "database",
        typeDescription: "SQLite Database",
        size: 8192,
        sizeDisplay: "8 KB",
        confidence: "high",
        isText: false,
        metadata: {
          "sqlite.pageSize": "4096",
          "sqlite.pageCount": "2",
          "sqlite.tableCount": "2",
          "sqlite.viewCount": "1",
          "sqlite.totalRows": "3",
          "sqlite.tableNames": "contacts, logs",
        },
        extractor: "core-artifact-extractor",
        extractedAt: "2026-02-16T10:03:30Z",
      },
    ],
    artifactCategories: [{ category: "text", count: 1 }],
    artifactEvidenceSummaries: [],
    artifactExtractorSummaries: [
      {
        extractor: "core-artifact-extractor",
        count: 1,
        totalSize: 42,
        totalSizeDisplay: "42 bytes",
        textCount: 1,
        categoryCount: 1,
        evidenceFileCount: 1,
        latestExtractedAt: "2026-02-16T10:02:00Z",
      },
    ],
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
        indicators: [
          {
            indicatorType: "email",
            value: "admin@example.com",
            offset: 24,
            length: 17,
            confidence: "medium",
          },
        ],
        indicatorCount: 1,
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
      {
        id: "ann-2",
        filePath: "ad1:/case/logical.ad1:/docs/a.txt",
        containerPath: "/case/logical.ad1",
        annotationType: "hex-source-indicator",
        offsetStart: 24,
        offsetEnd: 41,
        lineStart: null,
        lineEnd: null,
        label: "Email Indicator",
        content: "Type: email\nValue: admin@example.com\nConfidence: medium",
        color: "#06b6d4",
        createdBy: "hex-viewer",
        createdAt: "2026-02-16T10:04:30Z",
        modifiedAt: "2026-02-16T10:04:30Z",
      },
    ],
  };
}

describe("buildProjectDbEvidenceAppendices", () => {
  it("builds hash, artifact, source analysis, and annotation appendices", () => {
    const appendices = buildProjectDbEvidenceAppendices(makeEvidence());

    expect(appendices).toHaveLength(4);
    expect(appendices[0].appendix_id).toBe("A");
    expect(appendices[0].title).toBe("Project Hash and Verification Summary");
    expect(appendices[0].content).toContain("SHA-256");
    expect(appendices[1].content).toContain("core-artifact-extractor");
    expect(appendices[1].content).toContain("dimensions: 4032x3024");
    expect(appendices[1].content).toContain("make: CORE");
    expect(appendices[1].content).toContain("emails: 1");
    expect(appendices[1].content).toContain("subject: Quarterly update");
    expect(appendices[1].content).toContain("attachments: 1");
    expect(appendices[1].content).toContain("label: com.core.ffx.agent");
    expect(appendices[1].content).toContain("registry: 1.5");
    expect(appendices[1].content).toContain("manufacturer: Dell Inc.");
    expect(appendices[1].content).toContain("serial: ABC1234");
    expect(appendices[1].content).toContain("tables: 2");
    expect(appendices[2].content).toContain("PDF Document");
    expect(appendices[2].content).toContain("admin@example.com");
    expect(appendices[2].content).toContain("75.0%");
    expect(appendices[3].title).toBe("Hex Review and Annotation Findings");
    expect(appendices[3].content).toContain("hex-magic");
    expect(appendices[3].content).toContain("hex-source-indicator");
    expect(appendices[3].content).toContain("admin@example.com");
    expect(appendices[3].content).toContain("0x0-0x10");
  });

  it("returns no appendices for empty project DB evidence", () => {
    const empty = makeEvidence();
    empty.hashAlgorithmSummaries = [];
    empty.verificationResultSummaries = [];
    empty.artifactSummaries = [];
    empty.artifactCategories = [];
    empty.artifactExtractorSummaries = [];
    empty.sourceAnalysisSummaries = [];
    empty.sourceAnalysisCategorySummaries = [];
    empty.annotations = [];

    expect(buildProjectDbEvidenceAppendices(empty)).toEqual([]);
  });
});
