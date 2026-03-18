// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal } from "solid-js";
import { scanForAcquisitions } from "../api/importAcquisitions";
import type { DiscoveredAcquisition, ImportResult } from "../api/importAcquisitions";
import type { CompanionFile } from "../api/companion";
import type { DiscoveredFile } from "../types/container";
import type { DbEvidenceFile, DbProjectHash, DbEvidenceCollection, DbCollectedItem } from "../types/projectDb";
import { dbSync } from "./project/useProjectDbSync";
import { logger } from "../utils/logger";

const log = logger.scope("ImportAcquisitions");

// ─── Container type mapping ─────────────────────────────────────────────────

function mapContainerType(acquisitionType: string): string {
  switch (acquisitionType) {
    case "e01": return "e01";
    case "l01": return "l01";
    case "aff4": return "aff4";
    case "raw": return "raw";
    case "archive": return "archive";
    case "file_copy": return "raw";
    case "memory": return "raw";
    case "triage": return "raw";
    default: return "raw";
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
    case "triage":
      return "live_acquisition";
    default:
      return "logical_image";
  }
}

const TYPE_LABELS: Record<string, string> = {
  e01: "E01 forensic image",
  l01: "L01 logical evidence",
  aff4: "AFF4 forensic container",
  raw: "raw disk image",
  archive: "7z forensic archive",
  file_copy: "forensic file export",
  memory: "live memory dump",
  triage: "triage collection",
};

function uniqueId(prefix: string): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`;
}

function basename(path: string): string {
  return path.split(/[/\\]/).pop() || path;
}

// ─── Hook ────────────────────────────────────────────────────────────────────

export interface ImportAcquisitionsOptions {
  /** Callback when a file is imported (add to file manager tree) */
  onFileImported?: (file: DiscoveredFile) => void;
}

export function useImportAcquisitions() {
  const [scanning, setScanning] = createSignal(false);
  const [results, setResults] = createSignal<DiscoveredAcquisition[]>([]);
  const [selected, setSelected] = createSignal<Set<string>>(new Set());
  const [importing, setImporting] = createSignal(false);
  const [importResult, setImportResult] = createSignal<ImportResult | null>(null);
  const [error, setError] = createSignal<string | null>(null);

  /** Scan a directory for companion files */
  async function scan(dirPath: string): Promise<void> {
    setScanning(true);
    setError(null);
    setResults([]);
    setSelected(new Set<string>());
    setImportResult(null);

    try {
      const found = await scanForAcquisitions(dirPath);
      setResults(found);
      // Pre-select all that have existing output files
      const preSelected = new Set(
        found.filter(a => a.outputExists).map(a => a.companionPath),
      );
      setSelected(preSelected);
      log.info(`Scan complete: ${found.length} acquisitions found, ${preSelected.size} with existing output`);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      log.error("Scan failed:", msg);
    } finally {
      setScanning(false);
    }
  }

  function toggleSelect(companionPath: string): void {
    setSelected(prev => {
      const next = new Set(prev);
      if (next.has(companionPath)) {
        next.delete(companionPath);
      } else {
        next.add(companionPath);
      }
      return next;
    });
  }

  function selectAll(): void {
    setSelected(new Set(results().map(a => a.companionPath)));
  }

  function deselectAll(): void {
    setSelected(new Set<string>());
  }

  /** Import selected acquisitions into the current project */
  async function importSelected(
    knownPaths: Set<string>,
    options?: ImportAcquisitionsOptions,
  ): Promise<ImportResult> {
    setImporting(true);
    const result: ImportResult = { imported: 0, skipped: 0, errors: [] };

    try {
      const selectedSet = selected();
      const acquisitions = results().filter(a => selectedSet.has(a.companionPath));

      for (const acq of acquisitions) {
        try {
          const c = acq.companion;
          const outputPath = c.output.primaryPath;

          // Skip if already imported
          if (knownPaths.has(outputPath)) {
            result.skipped++;
            log.debug(`Skipped (already imported): ${outputPath}`);
            continue;
          }

          importSingleAcquisition(c, options);
          result.imported++;
          log.info(`Imported: ${outputPath}`);
        } catch (err) {
          const msg = err instanceof Error ? err.message : String(err);
          result.errors.push(`${acq.companion.output.primaryPath}: ${msg}`);
          log.warn(`Import failed for ${acq.companionPath}:`, msg);
        }
      }

      setImportResult(result);
      log.info(
        `Import complete: ${result.imported} imported, ${result.skipped} skipped, ${result.errors.length} errors`,
      );
      return result;
    } finally {
      setImporting(false);
    }
  }

  function reset(): void {
    setScanning(false);
    setResults([]);
    setSelected(new Set<string>());
    setImporting(false);
    setImportResult(null);
    setError(null);
  }

  return {
    // State
    scanning,
    results,
    selected,
    importing,
    importResult,
    error,
    // Actions
    scan,
    toggleSelect,
    selectAll,
    deselectAll,
    importSelected,
    reset,
  };
}

// ─── Import logic (DB writes) ───────────────────────────────────────────────

function importSingleAcquisition(
  c: CompanionFile,
  options?: ImportAcquisitionsOptions,
): void {
  const now = new Date().toISOString();
  const outputPath = c.output.primaryPath;
  const filename = basename(outputPath);
  const containerType = mapContainerType(c.acquisitionType);

  // 1. Evidence file record
  const evidenceFile: DbEvidenceFile = {
    id: outputPath,
    path: outputPath,
    filename,
    containerType,
    totalSize: c.output.totalBytes,
    segmentCount: c.output.segments?.length ?? 1,
    discoveredAt: now,
  };
  dbSync.upsertEvidenceFile(evidenceFile);

  // 2. Hash records (source = 'imported')
  const fileId = outputPath;
  if (c.hashes.md5) {
    dbSync.insertHash(buildHashRecord(fileId, "MD5", c.hashes.md5, c.timing.completedAt));
  }
  if (c.hashes.sha1) {
    dbSync.insertHash(buildHashRecord(fileId, "SHA-1", c.hashes.sha1, c.timing.completedAt));
  }
  if (c.hashes.sha256) {
    dbSync.insertHash(buildHashRecord(fileId, "SHA-256", c.hashes.sha256, c.timing.completedAt));
  }

  // 3. Evidence collection + collected item
  const collectionId = uniqueId("ec-import");
  const collection: DbEvidenceCollection = {
    id: collectionId,
    caseNumber: c.case?.caseNumber || "",
    collectionDate: c.timing.completedAt || "",
    collectionLocation: c.system?.hostname || "",
    collectingOfficer: c.case?.examiner || c.system?.username || "",
    authorization: "",
    documentationNotes: buildImportNotes(c),
    status: "draft",
    createdAt: now,
    modifiedAt: now,
  };
  dbSync.upsertEvidenceCollection(collection);

  const itemId = uniqueId("ci-import");
  const typeLabel = TYPE_LABELS[c.acquisitionType] || c.acquisitionType;
  const item: DbCollectedItem = {
    id: itemId,
    collectionId,
    evidenceFileId: fileId,
    itemNumber: c.case?.evidenceNumber || "1",
    description: `${typeLabel} — ${filename}`,
    foundLocation: c.source.paths?.join("; ") || "",
    itemType: "digital_media",
    condition: "original",
    packaging: "",
    imageFormat: c.output.format,
    acquisitionMethod: mapAcquisitionMethod(c.acquisitionType),
    storageNotes: buildStorageNotes(c),
    notes: c.case?.notes || "",
    itemCollectionDatetime: c.timing.startedAt,
    itemSystemDatetime: c.timing.completedAt,
    itemCollectingOfficer: c.case?.examiner || c.system?.username || "",
    building: c.system?.hostname || "",
  };
  dbSync.upsertCollectedItem(item);

  // 4. Notify caller to add file to tree
  if (options?.onFileImported) {
    const file: DiscoveredFile = {
      path: outputPath,
      filename,
      container_type: containerType,
      size: c.output.totalBytes,
      segment_count: c.output.segments?.length ?? 1,
    };
    options.onFileImported(file);
  }
}

function buildHashRecord(
  fileId: string,
  algorithm: string,
  hashValue: string,
  computedAt: string,
): DbProjectHash {
  return {
    id: uniqueId(`hash-import-${algorithm.toLowerCase()}`),
    fileId,
    algorithm,
    hashValue,
    computedAt,
    source: "imported",
  };
}

function buildImportNotes(c: CompanionFile): string {
  const typeLabel = TYPE_LABELS[c.acquisitionType] || c.acquisitionType;
  const parts = [
    `Imported from companion file. Original ${typeLabel} acquisition.`,
    `Tool: ${c.tool} v${c.toolVersion}`,
    `Output: ${c.output.primaryPath}`,
  ];
  if (c.system?.hostname) parts.push(`System: ${c.system.hostname}`);
  if (c.system?.username) parts.push(`Operator: ${c.system.username}`);
  return parts.join(" | ");
}

function buildStorageNotes(c: CompanionFile): string {
  const parts: string[] = [];
  parts.push(`Format: ${c.output.format}`);
  parts.push(`Size: ${formatBytes(c.output.totalBytes)}`);
  if (c.output.totalFiles) parts.push(`Files: ${c.output.totalFiles}`);
  if (c.output.compressed) parts.push("Compressed");
  if (c.output.segments && c.output.segments.length > 1) {
    parts.push(`Segments: ${c.output.segments.length}`);
  }
  if (c.timing.durationMs) {
    const dur = c.timing.durationMs;
    const durStr = dur < 60000
      ? `${(dur / 1000).toFixed(1)}s`
      : `${Math.floor(dur / 60000)}m ${Math.floor((dur % 60000) / 1000)}s`;
    parts.push(`Duration: ${durStr}`);
  }
  const hashes: string[] = [];
  if (c.hashes.md5) hashes.push(`MD5: ${c.hashes.md5}`);
  if (c.hashes.sha1) hashes.push(`SHA1: ${c.hashes.sha1}`);
  if (c.hashes.sha256) hashes.push(`SHA256: ${c.hashes.sha256}`);
  if (hashes.length > 0) parts.push(hashes.join(", "));
  return parts.join(" | ");
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}
