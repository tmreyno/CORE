// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * companionHelper — Shared helper for post-acquisition tasks:
 * 1. Write a .ffx-companion.json sidecar file alongside the output
 * 2. Auto-create an evidence collection record in .ffxdb
 */

import { invoke } from "@tauri-apps/api/core";
import { writeCompanionFile, type CompanionFileInput } from "../../api/companion";
import { formatBytes } from "../../api/archiveCreate";
import { dbSync } from "../project/useProjectDbSync";
import type { DbEvidenceCollection, DbCollectedItem } from "../../types/projectDb";

/** All acquisition metadata needed for companion file + evidence collection */
export interface AcquisitionInfo {
  acquisitionType: "e01" | "l01" | "raw" | "archive" | "file_copy" | "memory" | "triage";
  outputPath: string;
  sources: string[];

  // Case metadata (optional — not all modes have it)
  caseNumber?: string;
  evidenceNumber?: string;
  examiner?: string;
  description?: string;
  notes?: string;

  // Output details
  format: string;
  segments?: number;
  totalBytes: number;
  totalFiles?: number;
  compressed?: boolean;
  segmentSize?: number;

  // Hashes
  md5?: string;
  sha1?: string;
  sha256?: string;

  // Timing
  startedAt: string;
  completedAt: string;
  durationMs: number;

  // Post-write verification result (optional)
  verifyResult?: "verified" | "failed" | "error" | "skipped";
}

/**
 * Called on successful acquisition completion. Writes a companion sidecar file
 * and creates an evidence collection record. Both are fire-and-forget.
 */
export function handleAcquisitionComplete(info: AcquisitionInfo): void {
  // 1. Write companion file (fire-and-forget)
  writeCompanionSidecar(info);

  // 2. Write acquisition log (.txt) (fire-and-forget)
  writeAcquisitionLog(info);

  // 3. Create evidence collection record (fire-and-forget)
  createEvidenceCollectionRecord(info);
}

// ─── Companion sidecar ────────────────────────────────────────────────────

function writeCompanionSidecar(info: AcquisitionInfo): void {
  const data: CompanionFileInput = {
    acquisitionType: info.acquisitionType,
    case: {
      caseNumber: info.caseNumber || "",
      evidenceNumber: info.evidenceNumber || "",
      examiner: info.examiner || "",
      description: info.description || "",
      notes: info.notes || "",
    },
    source: {
      paths: info.sources,
      totalFiles: info.totalFiles ?? info.sources.length,
      totalBytes: info.totalBytes,
    },
    output: {
      path: info.outputPath,
      format: info.format,
      segments: info.segments ?? 1,
      totalBytes: info.totalBytes,
      compressed: info.compressed ?? false,
      segmentSize: info.segmentSize ?? 0,
    },
    hashes: {
      md5: info.md5 || "",
      sha1: info.sha1 || "",
      sha256: info.sha256 || "",
    },
    timing: {
      startedAt: info.startedAt,
      completedAt: info.completedAt,
      durationMs: info.durationMs,
    },
  };

  writeCompanionFile(info.outputPath, data).catch((err) => {
    console.warn("[companion] Failed to write companion file:", err);
  });
}

// ─── Acquisition log (.txt) ───────────────────────────────────────────────

function writeAcquisitionLog(info: AcquisitionInfo): void {
  const lines: string[] = [];
  const divider = "--------------------------------------------------------------";

  lines.push("Created By CORE-FFX Forensic File Explorer");
  lines.push("");
  lines.push("Case Information:");
  lines.push(`  Case Number:     ${info.caseNumber || "(not specified)"}`);
  lines.push(`  Evidence Number: ${info.evidenceNumber || "(not specified)"}`);
  lines.push(`  Description:     ${info.description || "(not specified)"}`);
  lines.push(`  Examiner:        ${info.examiner || "(not specified)"}`);
  if (info.notes) {
    lines.push(`  Notes:           ${info.notes}`);
  }
  lines.push("");
  lines.push(divider);

  // Source information
  lines.push("");
  lines.push("Source Information:");
  for (const src of info.sources) {
    lines.push(`  Source: ${src}`);
  }
  if (info.totalFiles) {
    lines.push(`  Total Files: ${info.totalFiles.toLocaleString()}`);
  }
  lines.push(`  Total Bytes: ${info.totalBytes.toLocaleString()} (${formatBytes(info.totalBytes)})`);
  lines.push("");
  lines.push(divider);

  // Image / output information
  lines.push("");
  lines.push("Image Information:");
  lines.push(`  Acquisition Type: ${formatAcquisitionType(info.acquisitionType)}`);
  lines.push(`  Output Format:    ${info.format.toUpperCase()}`);
  lines.push(`  Output Path:      ${info.outputPath}`);
  if (info.compressed) {
    lines.push("  Compression:      Yes");
  }
  if (info.segments && info.segments > 1) {
    lines.push(`  Segments:         ${info.segments}`);
  }
  if (info.segmentSize && info.segmentSize > 0) {
    lines.push(`  Segment Size:     ${formatBytes(info.segmentSize)}`);
  }
  lines.push("");

  // Timing information
  lines.push(`  Acquisition started:  ${formatTimestamp(info.startedAt)}`);
  lines.push(`  Acquisition finished: ${formatTimestamp(info.completedAt)}`);
  lines.push(`  Duration:             ${formatDuration(info.durationMs)}`);
  lines.push("");
  lines.push(divider);

  // Hash results
  if (info.md5 || info.sha1 || info.sha256) {
    lines.push("");
    lines.push("Computed Hashes:");
    if (info.md5) {
      lines.push(`  MD5:    ${info.md5}`);
    }
    if (info.sha1) {
      lines.push(`  SHA1:   ${info.sha1}`);
    }
    if (info.sha256) {
      lines.push(`  SHA256: ${info.sha256}`);
    }
    lines.push("");
  }

  // Verification results
  if (info.verifyResult && info.verifyResult !== "skipped") {
    lines.push("Image Verification Results:");
    switch (info.verifyResult) {
      case "verified":
        lines.push("  Status: VERIFIED — image hash matches source data");
        break;
      case "failed":
        lines.push("  Status: FAILED — image hash does NOT match source data");
        break;
      case "error":
        lines.push("  Status: ERROR — verification could not be completed");
        break;
    }
    lines.push("");
  }

  lines.push(divider);
  lines.push("");

  const content = lines.join("\n");

  // Determine log path: <output>.acquisition.log
  const logPath = info.outputPath.replace(/\.[^.]+$/, "") + ".acquisition.log";

  invoke("write_text_file", { path: logPath, content }).catch((err) => {
    console.warn("[companion] Failed to write acquisition log:", err);
  });
}

function formatAcquisitionType(t: string): string {
  const labels: Record<string, string> = {
    e01: "Physical Disk Image (E01/Ex01)",
    l01: "Logical Evidence (L01)",
    raw: "Raw Disk Image (.dd)",
    archive: "7z Archive",
    file_copy: "File Export (Copy)",
    memory: "Live Memory Capture",
    triage: "Forensic Triage Collection",
  };
  return labels[t] || t;
}

function formatTimestamp(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)} seconds`;
  const mins = Math.floor(ms / 60000);
  const secs = Math.floor((ms % 60000) / 1000);
  if (mins < 60) return `${mins} min ${secs} sec`;
  const hrs = Math.floor(mins / 60);
  const remMins = mins % 60;
  return `${hrs} hr ${remMins} min ${secs} sec`;
}

// ─── Evidence collection record ───────────────────────────────────────────

function createEvidenceCollectionRecord(info: AcquisitionInfo): void {
  const now = new Date().toISOString();
  const collectionId = `ec-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`;

  const collection: DbEvidenceCollection = {
    id: collectionId,
    caseNumber: info.caseNumber || "",
    collectionDate: info.completedAt,
    collectionLocation: "",
    collectingOfficer: info.examiner || "",
    authorization: "",
    documentationNotes: buildDocumentationNotes(info),
    status: "draft",
    createdAt: now,
    modifiedAt: now,
  };

  dbSync.upsertEvidenceCollection(collection);

  // Create a collected item for the output
  const itemId = `ci-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`;

  const item: DbCollectedItem = {
    id: itemId,
    collectionId,
    itemNumber: info.evidenceNumber || "1",
    description: buildItemDescription(info),
    foundLocation: "",
    itemType: mapDeviceType(info.acquisitionType),
    condition: "original",
    packaging: "",
    imageFormat: info.format,
    acquisitionMethod: mapAcquisitionMethod(info.acquisitionType),
    storageNotes: buildStorageNotes(info),
    notes: info.notes || "",
  };

  dbSync.upsertCollectedItem(item);
}

// ─── Helpers ──────────────────────────────────────────────────────────────

function buildDocumentationNotes(info: AcquisitionInfo): string {
  const typeLabel = {
    e01: "E01 physical image",
    l01: "L01 logical image",
    raw: "raw disk image",
    archive: "7z archive",
    file_copy: "file export",
    memory: "live memory capture",
    triage: "forensic triage collection",
  }[info.acquisitionType];

  return `Auto-created from ${typeLabel} acquisition. Output: ${info.outputPath}`;
}

function buildItemDescription(info: AcquisitionInfo): string {
  const typeLabel = {
    e01: "E01 forensic image",
    l01: "L01 logical evidence",
    raw: "raw disk image",
    archive: "7z forensic archive",
    file_copy: "forensic file export",
    memory: "live memory dump",
    triage: "triage collection",
  }[info.acquisitionType];

  const basename = info.outputPath.split("/").pop() || info.outputPath;
  return `${typeLabel} — ${basename}`;
}

function mapDeviceType(acquisitionType: string): string {
  switch (acquisitionType) {
    case "e01":
    case "raw":
      return "hard_drive";
    case "memory":
      return "memory";
    case "triage":
      return "computer";
    default:
      return "digital_media";
  }
}

function mapAcquisitionMethod(acquisitionType: string): string {
  switch (acquisitionType) {
    case "e01":
    case "raw":
      return "forensic_image";
    case "l01":
      return "logical_image";
    case "memory":
      return "live_acquisition";
    case "triage":
      return "live_acquisition";
    default:
      return "logical_image";
  }
}

function buildStorageNotes(info: AcquisitionInfo): string {
  const parts: string[] = [];
  parts.push(`Format: ${info.format}`);
  parts.push(`Size: ${formatBytes(info.totalBytes)}`);

  if (info.totalFiles) {
    parts.push(`Files: ${info.totalFiles}`);
  }
  if (info.compressed) {
    parts.push("Compressed");
  }
  if (info.segments && info.segments > 1) {
    parts.push(`Segments: ${info.segments}`);
  }

  const durationStr =
    info.durationMs < 60000
      ? `${(info.durationMs / 1000).toFixed(1)}s`
      : `${Math.floor(info.durationMs / 60000)}m ${Math.floor((info.durationMs % 60000) / 1000)}s`;
  parts.push(`Duration: ${durationStr}`);

  const hashes: string[] = [];
  if (info.md5) hashes.push(`MD5: ${info.md5}`);
  if (info.sha1) hashes.push(`SHA1: ${info.sha1}`);
  if (info.sha256) hashes.push(`SHA256: ${info.sha256}`);
  if (hashes.length > 0) {
    parts.push(hashes.join(", "));
  }

  return parts.join(" | ");
}
