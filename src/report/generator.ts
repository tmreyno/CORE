// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// =============================================================================
// REPORT GENERATOR - Convert app state to standardized JSON report
// =============================================================================

import type {
  ForensicReport,
  CaseInfo,
  EvidenceItem,
  HashRecord,
  ContainerMetadata,
  DeviceInfo,
  ExtractionInfo,
} from "./types";
import type {
  DiscoveredFile,
  ContainerInfo,
  StoredHash,
} from "../types";
import { formatBytes, formatDateByPreference } from "../utils";

// Import version from package.json (Vite handles JSON imports at build time)
import { version as APP_VERSION } from "../../package.json";

const APP_NAME = "FFX - Forensic File Explorer";

// =============================================================================
// MAIN REPORT GENERATOR
// =============================================================================

export interface ReportInput {
  /** Discovered files to include */
  files: DiscoveredFile[];
  /** Container info map (path -> info) */
  fileInfoMap: Map<string, ContainerInfo>;
  /** Computed hash map (path -> { algorithm, hash, verified }) */
  fileHashMap: Map<string, { algorithm: string; hash: string; verified?: boolean | null }>;
  /** Working directory */
  workingDirectory: string;
  /** Optional case info override */
  caseInfo?: Partial<CaseInfo>;
  /** Optional report title */
  title?: string;
  /** Optional notes */
  notes?: string;
}

/**
 * Generate a complete forensic report from current app state
 */
export function generateReport(input: ReportInput): ForensicReport {
  const now = new Date().toISOString();
  
  // Extract case info from first container that has it, or use override
  const extractedCaseInfo = extractCaseInfo(input.files, input.fileInfoMap);
  const caseInfo: CaseInfo = {
    ...extractedCaseInfo,
    ...input.caseInfo,
  };

  // Generate evidence items
  const evidence = input.files.map(file => 
    generateEvidenceItem(file, input.fileInfoMap.get(file.path))
  );

  // Generate hash records
  const hashes = generateHashRecords(
    input.files,
    input.fileInfoMap,
    input.fileHashMap
  );

  // Build report
  const report: ForensicReport = {
    schemaVersion: "1.0",
    meta: {
      generatedAt: now,
      generatedBy: APP_NAME,
      appVersion: APP_VERSION,
      title: input.title,
      notes: input.notes,
    },
    case: caseInfo,
    evidence,
    hashes,
    session: {
      startedAt: now, // Would track actual session start in production
      workingDirectory: input.workingDirectory,
      filesDiscovered: input.files.length,
      filesProcessed: input.fileHashMap.size,
    },
  };

  return report;
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Extract case info from container metadata
 */
function extractCaseInfo(
  files: DiscoveredFile[],
  infoMap: Map<string, ContainerInfo>
): CaseInfo {
  const caseInfo: CaseInfo = {};

  for (const file of files) {
    const info = infoMap.get(file.path);
    if (!info) continue;

    // Try AD1 companion log
    if (info.ad1?.companion_log) {
      const log = info.ad1.companion_log;
      if (!caseInfo.caseNumber && log.case_number) caseInfo.caseNumber = log.case_number;
      if (!caseInfo.evidenceNumber && log.evidence_number) caseInfo.evidenceNumber = log.evidence_number;
      if (!caseInfo.examiner && log.examiner) caseInfo.examiner = log.examiner;
      if (!caseInfo.notes && log.notes) caseInfo.notes = log.notes;
    }

    // Try E01
    if (info.e01) {
      const e01 = info.e01;
      if (!caseInfo.caseNumber && e01.case_number) caseInfo.caseNumber = e01.case_number;
      if (!caseInfo.evidenceNumber && e01.evidence_number) caseInfo.evidenceNumber = e01.evidence_number;
      if (!caseInfo.examiner && e01.examiner_name) caseInfo.examiner = e01.examiner_name;
      if (!caseInfo.notes && e01.notes) caseInfo.notes = e01.notes;
    }

    // Try UFED
    if (info.ufed?.case_info) {
      const ufed = info.ufed.case_info;
      if (!caseInfo.caseNumber && ufed.case_identifier) caseInfo.caseNumber = ufed.case_identifier;
      if (!caseInfo.evidenceNumber && ufed.device_name) caseInfo.evidenceNumber = ufed.device_name;
      if (!caseInfo.examiner && ufed.examiner_name) caseInfo.examiner = ufed.examiner_name;
      if (!caseInfo.department && ufed.department) caseInfo.department = ufed.department;
      if (!caseInfo.location && ufed.location) caseInfo.location = ufed.location;
    }

    // Try companion log
    if (info.companion_log) {
      const log = info.companion_log;
      if (!caseInfo.caseNumber && log.case_number) caseInfo.caseNumber = log.case_number;
      if (!caseInfo.evidenceNumber && log.evidence_number) caseInfo.evidenceNumber = log.evidence_number;
      if (!caseInfo.examiner && log.examiner) caseInfo.examiner = log.examiner;
      if (!caseInfo.notes && log.notes) caseInfo.notes = log.notes;
    }

    // Stop if we have all essential fields
    if (caseInfo.caseNumber && caseInfo.evidenceNumber && caseInfo.examiner) {
      break;
    }
  }

  return caseInfo;
}

/**
 * Generate evidence item from discovered file and container info
 */
function generateEvidenceItem(
  file: DiscoveredFile,
  info?: ContainerInfo
): EvidenceItem {
  const item: EvidenceItem = {
    id: file.path, // Use path as unique ID
    filename: file.filename,
    path: file.path,
    containerType: file.container_type,
    size: file.size,
    segmentCount: file.segment_count,
    created: file.created,
    modified: file.modified,
    metadata: generateMetadata(file, info),
  };

  // Add device info if available
  const device = extractDeviceInfo(info);
  if (device && Object.keys(device).length > 0) {
    item.device = device;
  }

  // Add extraction info if available
  const extraction = extractExtractionInfo(info);
  if (extraction && Object.keys(extraction).length > 0) {
    item.extraction = extraction;
  }

  return item;
}

/**
 * Generate container metadata
 */
function generateMetadata(
  file: DiscoveredFile,
  info?: ContainerInfo
): ContainerMetadata {
  const meta: ContainerMetadata = {
    format: file.container_type,
  };

  if (!info) return meta;

  // AD1
  if (info.ad1) {
    const ad1 = info.ad1;
    meta.format = `AD1 (${ad1.logical.signature})`;
    meta.formatVersion = ad1.logical.image_version?.toString();
    meta.itemCount = ad1.item_count;
    meta.chunkInfo = {
      chunkSize: ad1.logical.zlib_chunk_size,
    };
    meta.sourceDescription = ad1.logical.data_source_name;
    meta.notes = ad1.companion_log?.notes ?? undefined;
  }

  // E01
  if (info.e01) {
    const e01 = info.e01;
    meta.format = "EWF (Expert Witness Format)";
    meta.formatVersion = e01.format_version;
    meta.totalSize = e01.total_size;
    meta.compression = e01.compression;
    meta.chunkInfo = {
      chunkCount: e01.chunk_count,
      bytesPerSector: e01.bytes_per_sector,
      sectorsPerChunk: e01.sectors_per_chunk,
    };
    meta.sourceDescription = e01.description;
    meta.notes = e01.notes;
  }

  // L01
  if (info.l01) {
    const l01 = info.l01;
    meta.format = "L01 (Logical Evidence)";
    meta.formatVersion = l01.format_version?.toString();
    meta.totalSize = l01.total_size;
    // L01 uses EwfInfo which doesn't track file count separately
    // itemCount would be derived from item_tree if available
    meta.sourceDescription = l01.description;
  }

  // Raw
  if (info.raw) {
    const raw = info.raw;
    meta.format = "Raw Image";
    meta.totalSize = raw.total_size;
  }

  // Archive
  if (info.archive) {
    const archive = info.archive;
    meta.format = archive.format;
    meta.formatVersion = archive.version ?? undefined;
    meta.totalSize = archive.total_size;
    meta.entryCount = archive.entry_count ?? undefined;
    if (archive.aes_encrypted || archive.encrypted_headers) {
      meta.encryption = {
        encrypted: archive.aes_encrypted,
        headersEncrypted: archive.encrypted_headers,
      };
    }
  }

  // UFED
  if (info.ufed) {
    const ufed = info.ufed;
    meta.format = `UFED (${ufed.format})`;
    meta.totalSize = ufed.size;
  }

  return meta;
}

/**
 * Extract device info from container metadata
 */
function extractDeviceInfo(info?: ContainerInfo): DeviceInfo | undefined {
  if (!info) return undefined;

  // E01
  if (info.e01) {
    return {
      model: info.e01.model,
      serialNumber: info.e01.serial_number,
    };
  }

  // UFED
  if (info.ufed?.device_info) {
    const d = info.ufed.device_info;
    return {
      vendor: d.vendor ?? undefined,
      model: d.model ?? undefined,
      fullName: d.full_name ?? undefined,
      serialNumber: d.serial_number ?? undefined,
      imei: d.imei ?? undefined,
      imei2: d.imei2 ?? undefined,
      iccid: d.iccid ?? undefined,
      osVersion: d.os_version ?? undefined,
    };
  }

  return undefined;
}

/**
 * Extract extraction info from container metadata
 */
function extractExtractionInfo(info?: ContainerInfo): ExtractionInfo | undefined {
  if (!info) return undefined;

  // E01
  if (info.e01) {
    return {
      startTime: info.e01.acquiry_date,
    };
  }

  // UFED
  if (info.ufed?.extraction_info) {
    const e = info.ufed.extraction_info;
    return {
      tool: e.acquisition_tool ?? undefined,
      toolVersion: e.tool_version ?? undefined,
      extractionType: e.extraction_type ?? undefined,
      connectionType: e.connection_type ?? undefined,
      startTime: e.start_time ?? undefined,
      endTime: e.end_time ?? undefined,
      machineName: e.machine_name ?? undefined,
      guid: e.guid ?? undefined,
      unitId: e.unit_id ?? undefined,
    };
  }

  // Companion log
  if (info.companion_log) {
    return {
      tool: info.companion_log.created_by,
      startTime: info.companion_log.acquisition_started,
      endTime: info.companion_log.acquisition_finished,
    };
  }

  return undefined;
}

/**
 * Generate hash records from various sources
 */
function generateHashRecords(
  files: DiscoveredFile[],
  infoMap: Map<string, ContainerInfo>,
  hashMap: Map<string, { algorithm: string; hash: string; verified?: boolean | null }>
): HashRecord[] {
  const records: HashRecord[] = [];
  const now = new Date().toISOString();

  for (const file of files) {
    const info = infoMap.get(file.path);
    const computed = hashMap.get(file.path);

    // Add computed hash
    if (computed) {
      const record: HashRecord = {
        evidenceId: file.path,
        filename: file.filename,
        algorithm: computed.algorithm.toUpperCase(),
        computedHash: computed.hash.toUpperCase(),
        verified: computed.verified ?? null,
        source: "computed",
        computedAt: now,
      };

      // Try to find stored hash and source timestamp
      const stored = findStoredHash(info, computed.algorithm);
      if (stored) {
        record.storedHash = stored.hash.toUpperCase();
        record.source = stored.source as "container" | "companion" | "computed" | "user";
        record.sourceTimestamp = stored.timestamp ?? undefined;
      }

      records.push(record);
    }

    // Add stored hashes that weren't computed
    const storedHashes = getAllStoredHashes(info);
    for (const sh of storedHashes) {
      // Skip if we already have a computed record for this algorithm
      if (computed && computed.algorithm.toLowerCase() === sh.algorithm.toLowerCase()) {
        continue;
      }

      records.push({
        evidenceId: file.path,
        filename: file.filename,
        algorithm: sh.algorithm.toUpperCase(),
        computedHash: "", // Not computed
        storedHash: sh.hash.toUpperCase(),
        verified: null,
        source: (sh.source as "container" | "companion" | "computed" | "user") || "container",
        computedAt: now,
        sourceTimestamp: sh.timestamp ?? undefined,
      });
    }
  }

  return records;
}

/**
 * Find stored hash for a specific algorithm
 */
function findStoredHash(
  info: ContainerInfo | undefined,
  algorithm: string
): StoredHash | undefined {
  if (!info) return undefined;
  
  const algo = algorithm.toLowerCase();

  // E01 stored hashes
  if (info.e01?.stored_hashes) {
    const match = info.e01.stored_hashes.find(h => h.algorithm.toLowerCase() === algo);
    if (match) return match;
  }

  // UFED stored hashes
  if (info.ufed?.stored_hashes) {
    const match = info.ufed.stored_hashes.find(h => h.algorithm.toLowerCase() === algo);
    if (match) return { ...match, source: "container", timestamp: info.ufed.extraction_info?.start_time ?? null };
  }

  // Companion log stored hashes
  if (info.companion_log?.stored_hashes) {
    const match = info.companion_log.stored_hashes.find(h => h.algorithm.toLowerCase() === algo);
    if (match) return match;
  }

  return undefined;
}

/**
 * Get all stored hashes from container info
 */
function getAllStoredHashes(info: ContainerInfo | undefined): StoredHash[] {
  if (!info) return [];
  
  const hashes: StoredHash[] = [];

  // E01
  if (info.e01?.stored_hashes) {
    hashes.push(...info.e01.stored_hashes.map(h => ({ ...h, source: h.source || "container" })));
  }

  // UFED
  if (info.ufed?.stored_hashes) {
    hashes.push(...info.ufed.stored_hashes.map(h => ({
      algorithm: h.algorithm,
      hash: h.hash,
      source: "container" as const,
      timestamp: info.ufed!.extraction_info?.start_time ?? null,
      verified: null,
    })));
  }

  // Companion log
  if (info.companion_log?.stored_hashes) {
    hashes.push(...info.companion_log.stored_hashes.map(h => ({ ...h, source: h.source || "companion" })));
  }

  return hashes;
}

// =============================================================================
// EXPORT FUNCTIONS
// =============================================================================

/**
 * Export report as JSON string
 */
export function exportAsJson(report: ForensicReport, pretty = true): string {
  return JSON.stringify(report, null, pretty ? 2 : undefined);
}

/**
 * Export report as Markdown
 */
export function exportAsMarkdown(report: ForensicReport): string {
  const lines: string[] = [];
  
  // Title
  lines.push(`# ${report.meta.title || "Forensic Evidence Report"}`);
  lines.push("");
  lines.push(`**Generated:** ${formatDate(report.meta.generatedAt)}`);
  lines.push(`**Application:** ${report.meta.generatedBy} v${report.meta.appVersion}`);
  lines.push("");

  // Case Information
  if (report.case.caseNumber || report.case.evidenceNumber || report.case.examiner) {
    lines.push("---");
    lines.push("");
    lines.push("## Case Information");
    lines.push("");
    lines.push("| Field | Value |");
    lines.push("|-------|-------|");
    if (report.case.caseNumber) lines.push(`| Case # | ${report.case.caseNumber} |`);
    if (report.case.evidenceNumber) lines.push(`| Evidence # | ${report.case.evidenceNumber} |`);
    if (report.case.examiner) lines.push(`| Examiner | ${report.case.examiner} |`);
    if (report.case.department) lines.push(`| Department | ${report.case.department} |`);
    if (report.case.location) lines.push(`| Location | ${report.case.location} |`);
    lines.push("");
  }

  // Evidence Items
  lines.push("---");
  lines.push("");
  lines.push("## Evidence Items");
  lines.push("");

  for (const item of report.evidence) {
    lines.push(`### ${item.filename}`);
    lines.push("");
    lines.push(`**Type:** ${item.containerType}  `);
    lines.push(`**Size:** ${formatBytes(item.size)}  `);
    lines.push(`**Path:** \`${item.path}\``);
    lines.push("");

    if (item.device && Object.keys(item.device).length > 0) {
      lines.push("#### Device Information");
      lines.push("");
      lines.push("| Field | Value |");
      lines.push("|-------|-------|");
      if (item.device.vendor) lines.push(`| Vendor | ${item.device.vendor} |`);
      if (item.device.model) lines.push(`| Model | ${item.device.model} |`);
      if (item.device.fullName) lines.push(`| Full Name | ${item.device.fullName} |`);
      if (item.device.serialNumber) lines.push(`| Serial # | ${item.device.serialNumber} |`);
      if (item.device.imei) lines.push(`| IMEI | ${item.device.imei} |`);
      if (item.device.osVersion) lines.push(`| OS | ${item.device.osVersion} |`);
      lines.push("");
    }

    if (item.extraction && Object.keys(item.extraction).length > 0) {
      lines.push("#### Extraction Information");
      lines.push("");
      lines.push("| Field | Value |");
      lines.push("|-------|-------|");
      if (item.extraction.tool) lines.push(`| Tool | ${item.extraction.tool}${item.extraction.toolVersion ? ` v${item.extraction.toolVersion}` : ""} |`);
      if (item.extraction.extractionType) lines.push(`| Type | ${item.extraction.extractionType} |`);
      if (item.extraction.startTime) lines.push(`| Start Time | ${item.extraction.startTime} |`);
      if (item.extraction.endTime) lines.push(`| End Time | ${item.extraction.endTime} |`);
      lines.push("");
    }
  }

  // Hash Verification
  const verifiedHashes = report.hashes.filter(h => h.computedHash);
  if (verifiedHashes.length > 0) {
    lines.push("---");
    lines.push("");
    lines.push("## Hash Verification");
    lines.push("");

    for (const hash of verifiedHashes) {
      const status = hash.verified === true ? "✓ VERIFIED" : 
                     hash.verified === false ? "✗ MISMATCH" : 
                     "○ Computed";
      lines.push(`### ${hash.filename} (${hash.algorithm})`);
      lines.push("");
      lines.push(`**Status:** ${status}  `);
      lines.push(`**Computed:** \`${hash.computedHash}\`  `);
      if (hash.storedHash) {
        lines.push(`**Stored:** \`${hash.storedHash}\`  `);
      }
      if (hash.sourceTimestamp) {
        lines.push(`**Source Date:** ${hash.sourceTimestamp}  `);
      }
      lines.push("");
    }
  }

  // Notes
  if (report.meta.notes || report.case.notes) {
    lines.push("---");
    lines.push("");
    lines.push("## Notes");
    lines.push("");
    if (report.case.notes) lines.push(report.case.notes);
    if (report.meta.notes) lines.push(report.meta.notes);
    lines.push("");
  }

  // Footer
  lines.push("---");
  lines.push("");
  lines.push(`*Report generated by ${report.meta.generatedBy}*`);

  return lines.join("\n");
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

function formatDate(iso: string): string {
  return formatDateByPreference(iso, true) || iso;
}
