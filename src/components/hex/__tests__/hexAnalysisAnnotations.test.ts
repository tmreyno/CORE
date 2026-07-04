// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it } from "vitest";
import type { SourceAnalysis } from "../../../api/commands";
import { buildHexAnalysisAnnotations } from "../hexAnalysisAnnotations";

const timestamp = "2026-07-03T12:00:00.000Z";

function makeAnalysis(overrides: Partial<SourceAnalysis> = {}): SourceAnalysis {
  return {
    sourceRef: {
      kind: "containerEntry",
      containerPath: "/case/evidence.ad1",
      entryPath: "/docs/report.pdf",
      containerType: "ad1",
    },
    sourceId: "ad1:/case/evidence.ad1:/docs/report.pdf",
    totalSize: 4096,
    offset: 0,
    bytesAnalyzed: 1024,
    magicHex: "25 50 44 46",
    signatures: [
      {
        offset: 0,
        description: "PDF Document",
        mimeType: "application/pdf",
        extensions: ["pdf"],
        category: "document",
        confidence: "high",
        magicHex: "25 50 44 46",
      },
      {
        offset: 512,
        description: "ZIP Archive",
        mimeType: "application/zip",
        extensions: ["zip"],
        category: "archive",
        confidence: "medium",
        magicHex: "50 4B 03 04",
      },
    ],
    entropy: 7.8,
    entropyWindows: [
      { offset: 2048, length: 256, entropy: 7.9 },
      { offset: 3072, length: 128, entropy: 0.4 },
    ],
    histogram: [],
    printableBytes: 200,
    nulBytes: 3,
    highBitBytes: 50,
    printableRatio: 0.2,
    isLikelyText: false,
    indicators: [
      {
        indicatorType: "email",
        value: "admin@example.com",
        offset: 128,
        length: 17,
        confidence: "medium",
      },
      {
        indicatorType: "ipv4",
        value: "192.168.1.10",
        offset: 192,
        length: 12,
        confidence: "medium",
      },
    ],
    asciiPreview: "%PDF",
    ...overrides,
  };
}

describe("buildHexAnalysisAnnotations", () => {
  it("builds durable offset annotations from source-analysis findings", () => {
    const annotations = buildHexAnalysisAnnotations(makeAnalysis(), timestamp);

    expect(annotations.map((annotation) => annotation.annotationType)).toEqual([
      "hex-magic",
      "hex-primary-signature",
      "hex-embedded-signature",
      "hex-high-entropy",
      "hex-low-entropy",
      "hex-source-indicator",
      "hex-source-indicator",
    ]);
    expect(annotations[0]).toMatchObject({
      filePath: "ad1:/case/evidence.ad1:/docs/report.pdf",
      containerPath: "/case/evidence.ad1",
      offsetStart: 0,
      offsetEnd: 16,
      createdBy: "hex-viewer",
      createdAt: timestamp,
      modifiedAt: timestamp,
    });
    expect(annotations[2]).toMatchObject({
      label: "Embedded Signature: ZIP Archive",
      offsetStart: 512,
      offsetEnd: 516,
    });
    expect(annotations[5]).toMatchObject({
      label: "Email Indicator",
      content: "Type: email\nValue: admin@example.com\nConfidence: medium",
      offsetStart: 128,
      offsetEnd: 145,
    });
    expect(annotations[6]).toMatchObject({
      label: "IPv4 Indicator",
      offsetStart: 192,
      offsetEnd: 204,
    });
  });

  it("uses stable IDs so repeated analysis does not create duplicate findings", () => {
    const first = buildHexAnalysisAnnotations(makeAnalysis(), timestamp);
    const second = buildHexAnalysisAnnotations(
      makeAnalysis(),
      "2026-07-03T12:01:00.000Z",
    );

    expect(second.map((annotation) => annotation.id)).toEqual(
      first.map((annotation) => annotation.id),
    );
  });
});
