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
import { logger } from "../../utils/logger";
import { APP_NAME, isAcquireEdition } from "../../utils/edition";
import { getBasename } from "../../utils/pathUtils";
import { dbSync } from "../project/useProjectDbSync";

const log = logger.scope("Companion");
const APP_VERSION = typeof __APP_VERSION__ === "string" ? __APP_VERSION__ : "unknown";
import type { DbEvidenceCollection, DbCollectedItem } from "../../types/projectDb";

/** Source drive metadata matched from list_drives() */
export interface SourceDriveInfo {
  devicePath: string;
  name: string;
  mountPoint: string;
  fileSystem: string;
  totalBytes: number;
  kind: string; // "SSD", "HDD", "Unknown"
  isRemovable: boolean;
}

/** All acquisition metadata needed for companion file + evidence collection */
export interface AcquisitionInfo {
  acquisitionType: "e01" | "l01" | "raw" | "aff4" | "archive" | "file_copy" | "memory" | "triage";
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

  // Source device context (optional — gathered at acquisition time)
  sourceDrive?: SourceDriveInfo;
  hostname?: string;
  username?: string;

  // System identification (from Identify phase — avoids re-fetching)
  systemModel?: string;
  systemSerialNumber?: string;
  systemManufacturer?: string;
  osName?: string;
  osVersion?: string;
  /** Pre-collected drives list (avoids re-fetching via list_drives) */
  systemDrives?: SourceDriveInfo[];

  // Pre-created evidence collection IDs (for live updates — upsert pattern)
  collectionId?: string;
  itemId?: string;
}

/**
 * Create an initial evidence collection record when acquisition STARTS.
 * Returns the generated IDs so they can be passed to handleAcquisitionComplete
 * later, which will upsert (update) the same records with final data.
 */
export function startAcquisitionRecord(info: {
  acquisitionType: AcquisitionInfo["acquisitionType"];
  outputPath: string;
  sources: string[];
  caseNumber?: string;
  examiner?: string;
  hostname?: string;
  systemModel?: string;
  systemSerialNumber?: string;
  systemManufacturer?: string;
}): { collectionId: string; itemId: string } {
  const now = new Date().toISOString();
  const collectionId = `ec-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`;
  const itemId = `ci-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`;

  const sysDesc = [info.systemManufacturer, info.systemModel].filter(Boolean).join(" ");
  const docParts = [`Acquisition in progress — ${info.acquisitionType}`];
  if (info.hostname) docParts.push(`System: ${info.hostname}`);
  if (sysDesc) docParts.push(`Device: ${sysDesc}`);

  const collection: DbEvidenceCollection = {
    id: collectionId,
    caseNumber: info.caseNumber || "",
    collectionDate: now,
    collectionLocation: info.hostname || "",
    collectingOfficer: info.examiner || "",
    authorization: "",
    documentationNotes: docParts.join(" | "),
    status: "draft",
    createdAt: now,
    modifiedAt: now,
  };
  if (!isAcquireEdition()) {
    dbSync.upsertEvidenceCollection(collection);
  }

  const item: DbCollectedItem = {
    id: itemId,
    collectionId,
    itemNumber: "1",
    description: `${info.acquisitionType} — in progress`,
    foundLocation: info.sources.join("; "),
    itemType: info.acquisitionType === "memory" ? "memory" : "computer",
    condition: "original",
    packaging: "",
    imageFormat: info.acquisitionType,
    acquisitionMethod: mapAcquisitionMethod(info.acquisitionType),
    storageNotes: "Acquisition in progress…",
    notes: "",
    itemCollectionDatetime: now,
    itemSystemDatetime: "",
    itemCollectingOfficer: info.examiner || "",
    deviceType: info.acquisitionType === "memory" ? "memory" : "computer",
    brand: info.systemManufacturer || "",
    make: info.systemManufacturer || "",
    model: info.systemModel || "",
    serialNumber: info.systemSerialNumber || "",
    building: info.hostname || "",
    storageInterface: "",
    otherIdentifiers: info.systemSerialNumber ? `Serial: ${info.systemSerialNumber}` : "",
  };
  if (!isAcquireEdition()) {
    dbSync.upsertCollectedItem(item);
  }

  return { collectionId, itemId };
}

/**
 * Called on successful acquisition completion. Writes a companion sidecar file
 * and creates an evidence collection record. Both are fire-and-forget.
 *
 * If hostname/username/sourceDrive are not pre-filled, they are gathered
 * asynchronously from the backend before writing the evidence record.
 */
export function handleAcquisitionComplete(info: AcquisitionInfo): void {
  const normalized = normalizeAcquisitionInfo(info);

  // Gather system context if not already provided, then write everything
  gatherSystemContext(normalized).then((enriched) => {
    // 1. Write companion file (fire-and-forget)
    writeCompanionSidecar(enriched);

    // 2. Write acquisition log (.txt) (fire-and-forget)
    writeAcquisitionLog(enriched);

    // 3. Create evidence collection record (fire-and-forget, full edition only)
    if (!isAcquireEdition()) {
      createEvidenceCollectionRecord(enriched);
    }
  });
}

function normalizeAcquisitionInfo(info: AcquisitionInfo): AcquisitionInfo {
  const durationMs = Number.isFinite(info.durationMs)
    ? Math.max(0, Math.round(info.durationMs))
    : 0;

  return {
    ...info,
    durationMs,
  };
}

/**
 * Gather hostname, username, and source drive info if not already provided.
 */
async function gatherSystemContext(info: AcquisitionInfo): Promise<AcquisitionInfo> {
  const enriched = { ...info };

  // Gather hostname + username
  if (!enriched.hostname) {
    try {
      enriched.hostname = await invoke<string>("get_hostname");
    } catch (e) { log.debug("hostname unavailable:", e); }
  }
  if (!enriched.username) {
    try {
      enriched.username = await invoke<string>("get_current_username");
    } catch (e) { log.debug("username unavailable:", e); }
  }

  // Match source paths to mounted drives
  if (!enriched.sourceDrive && enriched.sources.length > 0) {
    try {
      // Use pre-collected drives list if available, otherwise fetch
      const drives = enriched.systemDrives || await invoke<SourceDriveInfo[]>("list_drives");
      const srcPath = enriched.sources[0];
      // Find the drive whose mount point is a prefix of the source path
      // Pick the longest matching mount point for accuracy
      let bestMatch: SourceDriveInfo | undefined;
      let bestLen = 0;
      for (const d of drives) {
        if (d.mountPoint && srcPath.startsWith(d.mountPoint) && d.mountPoint.length > bestLen) {
          bestMatch = d;
          bestLen = d.mountPoint.length;
        }
      }
      if (bestMatch) {
        enriched.sourceDrive = bestMatch;
      }
    } catch (e) { log.debug("drive matching unavailable:", e); }
  }

  return enriched;
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
      primaryPath: info.outputPath,
      format: info.format,
      totalBytes: info.totalBytes,
      totalFiles: info.totalFiles,
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
    system: info.hostname || info.username || info.sourceDrive || info.systemModel || info.systemManufacturer ? {
      hostname: info.hostname || "",
      username: info.username || "",
      sourceDrive: info.sourceDrive?.name || info.sourceDrive?.devicePath || "",
      sourceFileSystem: info.sourceDrive?.fileSystem || "",
      sourceCapacity: info.sourceDrive?.totalBytes || 0,
      sourceDriveType: info.sourceDrive?.kind || "",
      sourceRemovable: info.sourceDrive?.isRemovable ?? false,
      // System identification
      systemModel: info.systemModel || undefined,
      systemSerialNumber: info.systemSerialNumber || undefined,
      systemManufacturer: info.systemManufacturer || undefined,
      osName: info.osName || undefined,
      osVersion: info.osVersion || undefined,
    } : undefined,
  };

  writeCompanionFile(info.outputPath, data).catch((err) => {
    log.warn("Failed to write companion file:", err);
  });
}

// ─── Acquisition log (.txt) ───────────────────────────────────────────────

function writeAcquisitionLog(info: AcquisitionInfo): void {
  const lines: string[] = [];
  const divider = "--------------------------------------------------------------";

  lines.push(`Created By ${APP_NAME} v${APP_VERSION}`);
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

  // Source drive information (auto-detected)
  if (info.sourceDrive) {
    lines.push("");
    lines.push("Source Drive:");
    if (info.sourceDrive.name) {
      lines.push(`  Drive Name:      ${info.sourceDrive.name}`);
    }
    if (info.sourceDrive.devicePath) {
      lines.push(`  Device Path:     ${info.sourceDrive.devicePath}`);
    }
    if (info.sourceDrive.mountPoint) {
      lines.push(`  Mount Point:     ${info.sourceDrive.mountPoint}`);
    }
    if (info.sourceDrive.fileSystem) {
      lines.push(`  File System:     ${info.sourceDrive.fileSystem}`);
    }
    if (info.sourceDrive.kind) {
      lines.push(`  Drive Type:      ${info.sourceDrive.kind}`);
    }
    if (info.sourceDrive.totalBytes) {
      lines.push(`  Drive Capacity:  ${formatBytes(info.sourceDrive.totalBytes)}`);
    }
    lines.push(`  Removable:       ${info.sourceDrive.isRemovable ? "Yes" : "No"}`);
  }

  // System information
  if (info.hostname || info.username || info.systemModel || info.systemManufacturer) {
    lines.push("");
    lines.push("System Information:");
    if (info.hostname) {
      lines.push(`  Hostname:     ${info.hostname}`);
    }
    if (info.username) {
      lines.push(`  Username:     ${info.username}`);
    }
    if (info.systemManufacturer) {
      lines.push(`  Manufacturer: ${info.systemManufacturer}`);
    }
    if (info.systemModel) {
      lines.push(`  Model:        ${info.systemModel}`);
    }
    if (info.systemSerialNumber) {
      lines.push(`  Serial No:    ${info.systemSerialNumber}`);
    }
    if (info.osName) {
      const osStr = info.osVersion ? `${info.osName} ${info.osVersion}` : info.osName;
      lines.push(`  OS:           ${osStr}`);
    }
  }

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
    log.warn("Failed to write acquisition log:", err);
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
  const collectionId = info.collectionId || `ec-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`;

  const hostname = info.hostname || "";
  const username = info.username || "";

  const collection: DbEvidenceCollection = {
    id: collectionId,
    caseNumber: info.caseNumber || "",
    collectionDate: info.completedAt,
    collectionLocation: hostname || "",
    collectingOfficer: info.examiner || username || "",
    authorization: "",
    documentationNotes: buildDocumentationNotes(info),
    status: "draft",
    createdAt: now,
    modifiedAt: now,
  };

  dbSync.upsertEvidenceCollection(collection);

  // Create a collected item for the output
  const itemId = info.itemId || `ci-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`;

  const drive = info.sourceDrive;

  const item: DbCollectedItem = {
    id: itemId,
    collectionId,
    itemNumber: info.evidenceNumber || "1",
    description: buildItemDescription(info),
    foundLocation: info.sources.join("; "),
    itemType: mapDeviceType(info.acquisitionType, drive),
    condition: "original",
    packaging: "",
    // Forensic image info
    imageFormat: info.format,
    acquisitionMethod: mapAcquisitionMethod(info.acquisitionType),
    storageNotes: buildStorageNotes(info),
    notes: info.notes || "",
    // Per-item timestamps
    itemCollectionDatetime: info.startedAt,
    itemSystemDatetime: info.completedAt,
    itemCollectingOfficer: info.examiner || username || "",
    // Device identification from source drive + system identification
    deviceType: mapDeviceType(info.acquisitionType, drive),
    brand: info.systemManufacturer || drive?.name || "",
    make: info.systemManufacturer || "",
    model: info.systemModel || "",
    serialNumber: info.systemSerialNumber || "",
    // Location context from system
    building: hostname,
    // Drive/storage details
    storageInterface: drive?.kind || "",
    otherIdentifiers: buildOtherIdentifiers(info),
  };

  dbSync.upsertCollectedItem(item);
}

// ─── Helpers ──────────────────────────────────────────────────────────────

function buildDocumentationNotes(info: AcquisitionInfo): string {
  const typeLabel = {
    e01: "E01 physical image",
    l01: "L01 logical image",
    raw: "raw disk image",
    aff4: "AFF4 forensic container",
    archive: "7z archive",
    file_copy: "file export",
    memory: "live memory capture",
    triage: "forensic triage collection",
  }[info.acquisitionType];

  const parts: string[] = [
    `Auto-created from ${typeLabel} acquisition.`,
    `Output: ${info.outputPath}`,
  ];
  if (info.hostname) {
    parts.push(`System: ${info.hostname}`);
  }
  if (info.systemManufacturer || info.systemModel) {
    const sysId = [info.systemManufacturer, info.systemModel].filter(Boolean).join(" ");
    parts.push(`Device: ${sysId}`);
  }
  if (info.systemSerialNumber) {
    parts.push(`S/N: ${info.systemSerialNumber}`);
  }
  if (info.username) {
    parts.push(`Operator: ${info.username}`);
  }
  if (info.sourceDrive) {
    const d = info.sourceDrive;
    const driveDesc = [d.name, d.mountPoint, d.fileSystem, d.kind].filter(Boolean).join(", ");
    if (driveDesc) {
      parts.push(`Source drive: ${driveDesc}`);
    }
  }
  return parts.join(" | ");
}

function buildItemDescription(info: AcquisitionInfo): string {
  const typeLabel = {
    e01: "E01 forensic image",
    l01: "L01 logical evidence",
    raw: "raw disk image",
    aff4: "AFF4 forensic container",
    archive: "7z forensic archive",
    file_copy: "forensic file export",
    memory: "live memory dump",
    triage: "triage collection",
  }[info.acquisitionType];

  const basename = getBasename(info.outputPath) || info.outputPath;
  return `${typeLabel} — ${basename}`;
}

function mapDeviceType(acquisitionType: string, drive?: SourceDriveInfo): string {
  if (drive) {
    if (drive.isRemovable) return "removable_media";
    switch (drive.kind?.toLowerCase()) {
      case "ssd": return "hard_drive";
      case "hdd": return "hard_drive";
    }
  }
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

  // Source drive context
  if (info.sourceDrive) {
    const d = info.sourceDrive;
    if (d.kind) parts.push(`Drive: ${d.kind}`);
    if (d.fileSystem) parts.push(`FS: ${d.fileSystem}`);
    if (d.totalBytes) parts.push(`Capacity: ${formatBytes(d.totalBytes)}`);
    if (d.isRemovable) parts.push("Removable");
  }

  return parts.join(" | ");
}

function buildOtherIdentifiers(info: AcquisitionInfo): string {
  const ids: string[] = [];
  if (info.sourceDrive?.devicePath) {
    ids.push(`Device: ${info.sourceDrive.devicePath}`);
  }
  if (info.sourceDrive?.mountPoint) {
    ids.push(`Mount: ${info.sourceDrive.mountPoint}`);
  }
  if (info.sourceDrive?.fileSystem) {
    ids.push(`FS: ${info.sourceDrive.fileSystem}`);
  }
  if (info.sourceDrive?.totalBytes) {
    ids.push(`Capacity: ${formatBytes(info.sourceDrive.totalBytes)}`);
  }
  if (info.hostname) {
    ids.push(`Host: ${info.hostname}`);
  }
  if (info.systemSerialNumber) {
    ids.push(`Serial: ${info.systemSerialNumber}`);
  }
  if (info.systemModel) {
    ids.push(`Model: ${info.systemModel}`);
  }
  if (info.systemManufacturer) {
    ids.push(`Manufacturer: ${info.systemManufacturer}`);
  }
  if (info.osName) {
    const osStr = info.osVersion ? `${info.osName} ${info.osVersion}` : info.osName;
    ids.push(`OS: ${osStr}`);
  }
  return ids.join("; ");
}
