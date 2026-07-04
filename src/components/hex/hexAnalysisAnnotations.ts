// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type {
  EvidenceSourceRef,
  ProjectDbAnnotationRecord,
  SourceAnalysis,
} from "../../api/commands";

const MAX_SIGNATURE_ANNOTATIONS = 24;
const MAX_ENTROPY_ANNOTATIONS = 8;
const MAX_INDICATOR_ANNOTATIONS = 24;

function stableHash(input: string): string {
  let hash = 5381;
  for (let i = 0; i < input.length; i += 1) {
    hash = ((hash << 5) + hash) ^ input.charCodeAt(i);
  }
  return (hash >>> 0).toString(36);
}

function containerPathFromSourceRef(
  sourceRef: EvidenceSourceRef,
): string | null {
  switch (sourceRef.kind) {
    case "containerEntry":
    case "vfsEntry":
    case "nestedContainerEntry":
      return sourceRef.containerPath;
    case "localFile":
      return null;
  }
}

function annotationId(
  sourceId: string,
  annotationType: string,
  offsetStart: number,
  offsetEnd: number,
  label: string,
): string {
  return `hex-${stableHash(`${sourceId}|${annotationType}|${offsetStart}|${offsetEnd}|${label}`)}`;
}

function annotationRecord(
  analysis: SourceAnalysis,
  annotationType: string,
  offsetStart: number,
  offsetEnd: number,
  label: string,
  content: string,
  color: string,
  timestamp: string,
): ProjectDbAnnotationRecord {
  return {
    id: annotationId(
      analysis.sourceId,
      annotationType,
      offsetStart,
      offsetEnd,
      label,
    ),
    filePath: analysis.sourceId,
    containerPath: containerPathFromSourceRef(analysis.sourceRef),
    annotationType,
    offsetStart,
    offsetEnd,
    lineStart: null,
    lineEnd: null,
    label,
    content,
    color,
    createdBy: "hex-viewer",
    createdAt: timestamp,
    modifiedAt: timestamp,
  };
}

function indicatorLabel(indicatorType: string): string {
  switch (indicatorType) {
    case "email":
      return "Email Indicator";
    case "ipv4":
      return "IPv4 Indicator";
    case "url":
      return "URL Indicator";
    case "windows_path":
      return "Windows Path Indicator";
    case "unc_path":
      return "UNC Path Indicator";
    default:
      return "Source Indicator";
  }
}

function indicatorColor(indicatorType: string): string {
  switch (indicatorType) {
    case "email":
      return "#06b6d4";
    case "ipv4":
      return "#14b8a6";
    case "url":
      return "#3b82f6";
    case "windows_path":
    case "unc_path":
      return "#8b5cf6";
    default:
      return "#64748b";
  }
}

export function buildHexAnalysisAnnotations(
  analysis: SourceAnalysis,
  timestamp = new Date().toISOString(),
): ProjectDbAnnotationRecord[] {
  const totalSize = Math.max(0, analysis.totalSize ?? 0);
  if (totalSize === 0) return [];

  const annotations: ProjectDbAnnotationRecord[] = [];

  if (analysis.magicHex) {
    annotations.push(
      annotationRecord(
        analysis,
        "hex-magic",
        0,
        Math.min(16, totalSize),
        "Magic Bytes",
        `Initial signature bytes: ${analysis.magicHex}`,
        "#38bdf8",
        timestamp,
      ),
    );
  }

  for (const signature of (analysis.signatures ?? []).slice(
    0,
    MAX_SIGNATURE_ANNOTATIONS,
  )) {
    const offsetStart = Math.max(0, signature.offset);
    const signatureLength = Math.max(
      1,
      signature.magicHex.split(/\s+/).filter(Boolean).length,
    );
    const offsetEnd = Math.min(totalSize, offsetStart + signatureLength);
    annotations.push(
      annotationRecord(
        analysis,
        signature.offset === 0
          ? "hex-primary-signature"
          : "hex-embedded-signature",
        offsetStart,
        offsetEnd,
        signature.offset === 0
          ? `Primary Signature: ${signature.description}`
          : `Embedded Signature: ${signature.description}`,
        [
          `MIME: ${signature.mimeType}`,
          `Category: ${signature.category}`,
          `Confidence: ${signature.confidence}`,
          `Magic: ${signature.magicHex}`,
        ].join("\n"),
        signature.offset === 0 ? "#22c55e" : "#a78bfa",
        timestamp,
      ),
    );
  }

  for (const window of (analysis.entropyWindows ?? [])
    .filter((item) => item.entropy >= 7.5 || item.entropy <= 1.0)
    .slice(0, MAX_ENTROPY_ANNOTATIONS)) {
    const highEntropy = window.entropy >= 7.5;
    annotations.push(
      annotationRecord(
        analysis,
        highEntropy ? "hex-high-entropy" : "hex-low-entropy",
        window.offset,
        Math.min(totalSize, window.offset + window.length),
        highEntropy ? "High Entropy Window" : "Low Entropy Window",
        `${highEntropy ? "High" : "Low"} entropy byte range: ${window.entropy.toFixed(3)} bits/byte`,
        highEntropy ? "#f59e0b" : "#64748b",
        timestamp,
      ),
    );
  }

  for (const indicator of (analysis.indicators ?? []).slice(
    0,
    MAX_INDICATOR_ANNOTATIONS,
  )) {
    const offsetStart = Math.max(0, indicator.offset);
    const indicatorLength = Math.max(
      1,
      indicator.length || indicator.value.length,
    );
    const offsetEnd = Math.min(totalSize, offsetStart + indicatorLength);
    if (offsetStart >= offsetEnd) continue;

    const label = indicatorLabel(indicator.indicatorType);
    annotations.push(
      annotationRecord(
        analysis,
        "hex-source-indicator",
        offsetStart,
        offsetEnd,
        label,
        [
          `Type: ${indicator.indicatorType}`,
          `Value: ${indicator.value}`,
          `Confidence: ${indicator.confidence}`,
        ].join("\n"),
        indicatorColor(indicator.indicatorType),
        timestamp,
      ),
    );
  }

  const seen = new Set<string>();
  return annotations.filter((annotation) => {
    if (seen.has(annotation.id)) return false;
    seen.add(annotation.id);
    return true;
  });
}
