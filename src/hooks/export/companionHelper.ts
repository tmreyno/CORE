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

import { writeCompanionFile, type CompanionFileInput } from "../../api/companion";
import { formatBytes } from "../../api/archiveCreate";
import { dbSync } from "../project/useProjectDbSync";
import type { DbEvidenceCollection, DbCollectedItem } from "../../types/projectDb";

/** All acquisition metadata needed for companion file + evidence collection */
export interface AcquisitionInfo {
  acquisitionType: "e01" | "l01" | "archive" | "file_copy" | "memory" | "triage";
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
}

/**
 * Called on successful acquisition completion. Writes a companion sidecar file
 * and creates an evidence collection record. Both are fire-and-forget.
 */
export function handleAcquisitionComplete(info: AcquisitionInfo): void {
  // 1. Write companion file (fire-and-forget)
  writeCompanionSidecar(info);

  // 2. Create evidence collection record (fire-and-forget)
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
