// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Auto-fills evidence collection form fields from loaded container metadata.
 *
 * Container info (E01/AD1/UFED/Archive) contains machine-accurate data like
 * serial numbers, model info, acquisition dates, image formats, and hashes
 * that is more reliable than manual user entry. This module maps that data
 * to the evidence collection form fields.
 */

import type { DiscoveredFile, ContainerInfo } from "../../types";
import type { FormData } from "../../templates/types";
import { generateId } from "./formDataConversion";

// =============================================================================
// Types
// =============================================================================

/** Summary of what can be auto-filled from a single evidence file */
export interface EvidenceFileSummary {
  /** Path to the evidence file */
  path: string;
  /** Display name (filename) */
  filename: string;
  /** Container type string (ad1, e01, l01, ufed, etc.) */
  containerType: string;
  /** Size in bytes */
  size: number;
  /** Number of form fields that can be auto-filled */
  autoFillFieldCount: number;
  /** Human-readable summary of what will be populated */
  autoFillSummary: string[];
}

/** Result of auto-filling header fields from evidence */
export interface HeaderAutoFillResult {
  /** Fields that were populated */
  filledFields: string[];
  /** Form data patch (partial) */
  patch: Partial<FormData>;
}

// =============================================================================
// Container Type → Image Format Mapping
// =============================================================================

const CONTAINER_TYPE_TO_IMAGE_FORMAT: Record<string, string> = {
  ad1: "ad1",
  e01: "e01",
  l01: "l01",
  ex01: "ex01",
  lx01: "l01",
  raw: "dd_raw",
  "001": "001",
  dd: "dd_raw",
  img: "dd_raw",
  bin: "dd_raw",
  dmg: "dmg",
  zip: "zip",
  "7z": "7z",
  tar: "tar",
  ufed: "ufdr",
  ufd: "ufd",
  ufdr: "ufdr",
  ufdx: "ufdr",
  mem: "mem_dump",
  dmp: "mem_dump",
  vmdk: "vmdk",
  vhd: "vhd",
  vhdx: "vhd",
};

// =============================================================================
// UFED Extraction Type → Acquisition Method Mapping
// =============================================================================

const UFED_EXTRACTION_TO_METHOD: Record<string, string> = {
  physical: "ufed_physical",
  "file system": "ufed_file_system",
  "file-system": "ufed_file_system",
  filesystem: "ufed_file_system",
  logical: "ufed_logical",
  "advanced logical": "ufed_logical",
  cloud: "ufed_cloud",
  chinex: "ufed_chinex",
};

// =============================================================================
// UFED Device Hint → Device Type Mapping
// =============================================================================

function inferDeviceType(
  ufedDeviceInfo?: { vendor?: string | null; model?: string | null; full_name?: string | null } | null,
  deviceHint?: string | null,
  containerType?: string,
): string | undefined {
  // UFED device hints
  const hint = (deviceHint || "").toLowerCase();
  if (hint.includes("phone") || hint.includes("iphone") || hint.includes("android") || hint.includes("samsung") || hint.includes("pixel")) {
    return "mobile_phone";
  }
  if (hint.includes("tablet") || hint.includes("ipad")) return "tablet";
  if (hint.includes("watch") || hint.includes("wearable")) return "wearable";
  if (hint.includes("drone") || hint.includes("uav")) return "drone";

  // UFED vendor hints
  const vendor = (ufedDeviceInfo?.vendor || "").toLowerCase();
  const model = (ufedDeviceInfo?.model || ufedDeviceInfo?.full_name || "").toLowerCase();
  if (vendor === "apple" && (model.includes("iphone") || model.includes("ipod"))) return "mobile_phone";
  if (vendor === "apple" && model.includes("ipad")) return "tablet";
  if (vendor === "apple" && model.includes("watch")) return "wearable";
  if (vendor === "samsung" || vendor === "google" || vendor === "huawei" || vendor === "xiaomi" || vendor === "oneplus" || vendor === "oppo") {
    return model.includes("tablet") || model.includes("tab") ? "tablet" : "mobile_phone";
  }

  // Container-level clues
  if (containerType === "ufed" || containerType === "ufd" || containerType === "ufdr") {
    return "mobile_phone"; // UFED is predominantly mobile
  }

  return undefined;
}

// =============================================================================
// Core: Extract Form Fields from ContainerInfo
// =============================================================================

/**
 * Extract collected item form fields from a single evidence file's container info.
 * Returns a partial FormData record (flat key-value) for one collected_items entry.
 */
export function extractItemFieldsFromEvidence(
  file: DiscoveredFile,
  info: ContainerInfo | undefined,
  _caseNumber?: string,
): Record<string, string> {
  const fields: Record<string, string> = {};

  // Image format is always derivable from container type
  const imageFormat = CONTAINER_TYPE_TO_IMAGE_FORMAT[file.container_type.toLowerCase()];
  if (imageFormat) fields.image_format = imageFormat;

  // Description defaults to filename
  fields.description = file.filename;

  if (!info) return fields;

  // --- E01 / L01 (EWF) ---
  const ewf = info.e01 || info.l01;
  if (ewf) {
    if (ewf.serial_number) fields.serial_number = ewf.serial_number;
    if (ewf.model) fields.model = ewf.model;
    if (ewf.evidence_number) fields.item_number = ewf.evidence_number;
    if (ewf.acquiry_date) fields.item_collection_datetime = ewf.acquiry_date;
    if (ewf.system_date) fields.item_system_datetime = ewf.system_date;
    if (ewf.notes) fields.notes = ewf.notes;
    if (ewf.description) {
      fields.description = ewf.description;
    }

    // L01-specific: source name from ltree (data source / device name)
    if (ewf.l01_source_name) {
      // Use source name as description if no header description exists
      if (!ewf.description) {
        fields.description = ewf.l01_source_name;
      }
      // Also set as "make" field for source device identification
      if (!fields.brand) {
        fields.brand = ewf.l01_source_name;
      }
    }

    // L01-specific: evidence number from ltree source (fallback)
    if (!fields.item_number && ewf.l01_source_evidence_number) {
      fields.item_number = ewf.l01_source_evidence_number;
    }

    // E01/L01 total size gives storage info
    const storageDetails: string[] = [];
    if (ewf.total_size > 0) {
      storageDetails.push(`Image size: ${formatSize(ewf.total_size)} (${ewf.compression} compression, ${ewf.sector_count} sectors)`);
    }
    // L01-specific: file count and total acquired bytes from ltree record summary
    if (ewf.l01_file_count && ewf.l01_file_count > 0) {
      storageDetails.push(`Files acquired: ${ewf.l01_file_count.toLocaleString()}`);
    }
    if (ewf.l01_total_bytes && ewf.l01_total_bytes > 0) {
      storageDetails.push(`Source data: ${formatSize(ewf.l01_total_bytes)}`);
    }
    if (storageDetails.length > 0) {
      fields.storage_notes = storageDetails.join(" | ");
    }
  }

  // --- AD1 ---
  if (info.ad1) {
    // Base info from AD1 logical header
    if (info.ad1.logical?.data_source_name) {
      fields.description = info.ad1.logical.data_source_name;
    }
    // Companion log has richer metadata
    const cl = info.ad1.companion_log;
    if (cl) {
      if (cl.evidence_number) fields.item_number = cl.evidence_number;
      if (cl.source_device) fields.brand = cl.source_device;
      if (cl.acquisition_date) fields.item_collection_datetime = cl.acquisition_date;
      if (cl.notes) fields.notes = cl.notes;
      if (cl.acquisition_method) {
        const method = mapAcquisitionMethod(cl.acquisition_method);
        if (method) fields.acquisition_method = method;
      }
      const storageInfo: string[] = [];
      if (cl.acquisition_tool) storageInfo.push(`Tool: ${cl.acquisition_tool}`);
      if (cl.total_items) storageInfo.push(`Items: ${cl.total_items.toLocaleString()}`);
      if (cl.total_size) storageInfo.push(`Source size: ${formatSize(cl.total_size)}`);
      if (storageInfo.length > 0) fields.storage_notes = storageInfo.join(" | ");
    }
    // Volume info
    if (info.ad1.volume) {
      const vol = info.ad1.volume;
      if (vol.volume_serial) fields.other_identifiers = `Volume Serial: ${vol.volume_serial}`;
    }
  }

  // --- UFED (Cellebrite) ---
  if (info.ufed) {
    const di = info.ufed.device_info;
    if (di) {
      if (di.vendor) fields.brand = di.vendor;
      if (di.model) fields.model = di.model;
      if (di.full_name && !di.model) fields.model = di.full_name;
      if (di.serial_number) fields.serial_number = di.serial_number;
      if (di.imei) fields.imei = di.imei;
      // Second IMEI goes to other_identifiers
      if (di.imei2) {
        fields.other_identifiers = fields.other_identifiers
          ? `${fields.other_identifiers}, IMEI2: ${di.imei2}`
          : `IMEI2: ${di.imei2}`;
      }
      if (di.iccid) {
        fields.other_identifiers = fields.other_identifiers
          ? `${fields.other_identifiers}, ICCID: ${di.iccid}`
          : `ICCID: ${di.iccid}`;
      }
    }

    // Device type from device info or hint
    const deviceType = inferDeviceType(di, info.ufed.device_hint, file.container_type);
    if (deviceType) fields.device_type = deviceType;

    // Extraction info → acquisition method + connection
    const ei = info.ufed.extraction_info;
    if (ei) {
      if (ei.extraction_type) {
        const method = UFED_EXTRACTION_TO_METHOD[ei.extraction_type.toLowerCase()];
        if (method) fields.acquisition_method = method;
      }
      if (ei.connection_type) {
        fields.connection_method = mapConnectionMethod(ei.connection_type);
      }
      if (ei.start_time) fields.item_collection_datetime = ei.start_time;

      const toolInfo: string[] = [];
      if (ei.acquisition_tool) toolInfo.push(`Tool: ${ei.acquisition_tool}`);
      if (ei.tool_version) toolInfo.push(`Version: ${ei.tool_version}`);
      if (ei.unit_id) toolInfo.push(`Unit ID: ${ei.unit_id}`);
      if (ei.machine_name) toolInfo.push(`Machine: ${ei.machine_name}`);
      if (toolInfo.length > 0) {
        fields.storage_notes = fields.storage_notes
          ? `${fields.storage_notes} | ${toolInfo.join(" | ")}`
          : toolInfo.join(" | ");
      }
    }

    // Evidence number
    if (info.ufed.evidence_number) fields.item_number = info.ufed.evidence_number;

    // Case info for header
    const ci = info.ufed.case_info;
    if (ci?.location) {
      fields.building = ci.location;
    }
  }

  // --- Companion Log (E01 companion .txt) ---
  if (info.companion_log && !info.ad1) {
    const cl = info.companion_log;
    if (cl.evidence_number) fields.item_number = cl.evidence_number;
    if (cl.examiner) fields.item_collecting_officer = cl.examiner;
    if (cl.acquisition_started) fields.item_collection_datetime = cl.acquisition_started;
    if (cl.notes) fields.notes = cl.notes;
    const hashes = cl.stored_hashes
      .map((h) => `${h.algorithm}: ${h.hash}`)
      .join("\n");
    if (hashes) {
      fields.storage_notes = fields.storage_notes
        ? `${fields.storage_notes}\nHashes:\n${hashes}`
        : `Hashes:\n${hashes}`;
    }
  }

  return fields;
}

/**
 * Extract header-level fields (collection date, examiner, case number, etc.)
 * from the set of all evidence files. Priority: first populated value wins.
 */
export function extractHeaderFieldsFromEvidence(
  files: DiscoveredFile[],
  infoMap: Map<string, ContainerInfo>,
): HeaderAutoFillResult {
  const patch: Partial<FormData> = {};
  const filledFields: string[] = [];

  let examiner: string | undefined;
  let collectionDate: string | undefined;
  let caseNumber: string | undefined;
  let evidenceNumber: string | undefined;

  for (const file of files) {
    const info = infoMap.get(file.path);
    if (!info) continue;

    // E01/L01
    const ewf = info.e01 || info.l01;
    if (ewf) {
      if (!examiner && ewf.examiner_name) examiner = ewf.examiner_name;
      if (!collectionDate && ewf.acquiry_date) collectionDate = ewf.acquiry_date;
      if (!caseNumber && ewf.case_number) caseNumber = ewf.case_number;
      if (!evidenceNumber && ewf.evidence_number) evidenceNumber = ewf.evidence_number;
    }

    // AD1 companion log
    if (info.ad1?.companion_log) {
      const cl = info.ad1.companion_log;
      if (!examiner && cl.examiner) examiner = cl.examiner;
      if (!collectionDate && cl.acquisition_date) collectionDate = cl.acquisition_date;
      if (!caseNumber && cl.case_number) caseNumber = cl.case_number;
      if (!evidenceNumber && cl.evidence_number) evidenceNumber = cl.evidence_number;
    }

    // UFED
    if (info.ufed) {
      if (!examiner && info.ufed.case_info?.examiner_name) {
        examiner = info.ufed.case_info.examiner_name;
      }
      if (!caseNumber && info.ufed.case_info?.case_identifier) {
        caseNumber = info.ufed.case_info.case_identifier;
      }
      if (!collectionDate && info.ufed.extraction_info?.start_time) {
        collectionDate = info.ufed.extraction_info.start_time;
      }
      if (!evidenceNumber && info.ufed.evidence_number) {
        evidenceNumber = info.ufed.evidence_number;
      }
    }

    // Companion log (standalone)
    if (info.companion_log) {
      const cl = info.companion_log;
      if (!examiner && cl.examiner) examiner = cl.examiner;
      if (!caseNumber && cl.case_number) caseNumber = cl.case_number;
      if (!collectionDate && cl.acquisition_started) collectionDate = cl.acquisition_started;
    }
  }

  if (examiner) { patch.collecting_officer = examiner; filledFields.push("collecting_officer"); }
  if (collectionDate) { patch.collection_date = collectionDate; filledFields.push("collection_date"); }

  return { patch, filledFields };
}

/**
 * Build a full collected_items array from discovered evidence files + container info.
 * Each evidence file becomes one collected item with fields auto-populated from metadata.
 */
export function buildCollectedItemsFromEvidence(
  files: DiscoveredFile[],
  infoMap: Map<string, ContainerInfo>,
  caseNumber?: string,
): FormData[] {
  return files.map((file, index) => {
    const info = infoMap.get(file.path);
    const fields = extractItemFieldsFromEvidence(file, info, caseNumber);

    return {
      id: generateId(),
      item_number: fields.item_number || `${caseNumber ? caseNumber + "-" : ""}EV-${String(index + 1).padStart(4, "0")}`,
      description: fields.description || file.filename,
      item_collection_datetime: fields.item_collection_datetime || "",
      item_system_datetime: fields.item_system_datetime || "",
      item_collecting_officer: fields.item_collecting_officer || "",
      item_authorization: "",
      device_type: fields.device_type || "desktop_computer",
      device_type_other: "",
      storage_interface: fields.storage_interface || "sata",
      storage_interface_other: "",
      form_factor: "",
      form_factor_other: "",
      connection_method: fields.connection_method || "",
      connection_method_other: "",
      brand: fields.brand || "",
      make: fields.make || "",
      model: fields.model || "",
      color: "",
      serial_number: fields.serial_number || "",
      imei: fields.imei || "",
      other_identifiers: fields.other_identifiers || "",
      building: fields.building || "",
      room: "",
      location_other: "",
      image_format: fields.image_format || "",
      image_format_other: "",
      acquisition_method: fields.acquisition_method || "",
      acquisition_method_other: "",
      condition: "good",
      packaging: "",
      timezone: "",
      storage_notes: fields.storage_notes || "",
      notes: fields.notes || "",
    } as FormData;
  });
}

// =============================================================================
// Enrichment Keys — Fields that can be auto-populated from container metadata.
// Only empty fields are filled; user-entered data is never overwritten.
// =============================================================================

const ENRICHABLE_FIELDS = [
  "brand", "make", "model", "serial_number", "imei", "other_identifiers",
  "image_format", "acquisition_method", "storage_notes",
  "item_collection_datetime", "item_system_datetime", "item_collecting_officer",
  "device_type", "notes",
] as const;

/** Result of enriching existing form items with container metadata */
export interface EnrichmentResult {
  /** Number of items that were enriched (at least one field filled) */
  enrichedCount: number;
  /** Total number of fields that were filled across all items */
  fieldsFilled: number;
  /** Updated form items (only items that changed are replaced) */
  updatedItems: FormData[];
  /** Whether any changes were made */
  changed: boolean;
}

/**
 * Enrich existing collected items with container metadata.
 *
 * Matches each form item to a discovered evidence file by:
 *   1. evidence_file_id (explicit FK)
 *   2. description exactly matching a filename (case-insensitive)
 *   3. description containing a filename (e.g., "PC-MUS-001.E01 - Hard Drive")
 *
 * For each match, fills ONLY empty fields from the container's metadata.
 * User-entered data is never overwritten. Returns a copy with enriched fields.
 */
export function enrichExistingItemsFromEvidence(
  items: FormData[],
  files: DiscoveredFile[],
  infoMap: Map<string, ContainerInfo>,
  caseNumber?: string,
): EnrichmentResult {
  let enrichedCount = 0;
  let fieldsFilled = 0;
  let changed = false;

  // Build lookup maps for matching
  const fileByName = new Map<string, DiscoveredFile>();
  const fileByPath = new Map<string, DiscoveredFile>();
  for (const f of files) {
    fileByName.set(f.filename.toLowerCase(), f);
    fileByPath.set(f.path, f);
  }

  const updatedItems = items.map((item) => {
    // 1. Match by evidence_file_id (explicit FK to file path)
    let matchedFile: DiscoveredFile | undefined;
    const evidenceFileId = item.evidence_file_id as string;
    if (evidenceFileId) {
      matchedFile = fileByPath.get(evidenceFileId);
    }

    // 2. Match by description = filename
    if (!matchedFile) {
      const desc = ((item.description as string) || "").toLowerCase().trim();
      if (desc) {
        matchedFile = fileByName.get(desc);
      }
    }

    // 3. Match by description containing filename (e.g., "PC-MUS-001.E01 - Hard Drive")
    if (!matchedFile) {
      const desc = ((item.description as string) || "").toLowerCase();
      for (const f of files) {
        if (desc.includes(f.filename.toLowerCase())) {
          matchedFile = f;
          break;
        }
      }
    }

    if (!matchedFile) return item;

    // Extract all available fields from the matched container
    const info = infoMap.get(matchedFile.path);
    const containerFields = extractItemFieldsFromEvidence(matchedFile, info, caseNumber);

    // Fill only empty fields
    let itemFieldsFilled = 0;
    const enriched = { ...item };

    for (const key of ENRICHABLE_FIELDS) {
      const current = (enriched[key] as string) || "";
      const fromContainer = containerFields[key] || "";
      if (!current && fromContainer) {
        (enriched as Record<string, unknown>)[key] = fromContainer;
        itemFieldsFilled++;
      }
    }

    if (itemFieldsFilled > 0) {
      enrichedCount++;
      fieldsFilled += itemFieldsFilled;
      changed = true;
    }

    return enriched;
  });

  return { enrichedCount, fieldsFilled, updatedItems, changed };
}

/**
 * Get a summary of what can be auto-filled for each evidence file.
 * Used to show the user what will be populated before they confirm.
 */
export function getAutoFillSummaries(
  files: DiscoveredFile[],
  infoMap: Map<string, ContainerInfo>,
): EvidenceFileSummary[] {
  return files.map((file) => {
    const info = infoMap.get(file.path);
    const fields = extractItemFieldsFromEvidence(file, info);
    const filledKeys = Object.keys(fields).filter(
      (k) => fields[k] && k !== "description" && k !== "image_format",
    );

    const summary: string[] = [];
    if (fields.serial_number) summary.push(`S/N: ${fields.serial_number}`);
    if (fields.model) summary.push(`Model: ${fields.model}`);
    if (fields.brand) summary.push(`Brand: ${fields.brand}`);
    if (fields.imei) summary.push(`IMEI: ${fields.imei}`);
    if (fields.item_collection_datetime) summary.push("Acquisition date");
    if (fields.acquisition_method) summary.push("Acquisition method");
    if (fields.image_format) summary.push(`Format: ${fields.image_format.toUpperCase()}`);

    // L01-specific enrichment indicators
    const ewfInfo = info?.l01;
    if (ewfInfo?.l01_source_name) summary.push(`Source: ${ewfInfo.l01_source_name}`);
    if (ewfInfo?.l01_file_count && ewfInfo.l01_file_count > 0) {
      summary.push(`${ewfInfo.l01_file_count.toLocaleString()} files`);
    };

    return {
      path: file.path,
      filename: file.filename,
      containerType: file.container_type,
      size: file.size,
      autoFillFieldCount: filledKeys.length,
      autoFillSummary: summary,
    };
  });
}

// =============================================================================
// Helpers
// =============================================================================

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function mapAcquisitionMethod(raw: string): string | undefined {
  const lower = raw.toLowerCase().trim();
  if (lower.includes("physical")) return "physical";
  if (lower.includes("logical") && lower.includes("file")) return "logical_file_folder";
  if (lower.includes("logical") && lower.includes("partition")) return "logical_partition";
  if (lower.includes("logical")) return "logical_file_folder";
  if (lower.includes("file system")) return "file_system";
  if (lower.includes("native")) return "native_file";
  if (lower.includes("targeted")) return "targeted_collection";
  return undefined;
}

function mapConnectionMethod(raw: string): string {
  const lower = raw.toLowerCase().trim();
  if (lower.includes("usb")) return "usb";
  if (lower.includes("cable")) return "usb";
  if (lower.includes("bluetooth")) return "bluetooth";
  if (lower.includes("wifi") || lower.includes("wi-fi")) return "wifi";
  if (lower.includes("network")) return "network";
  // Return raw value — the form has a flexible text input
  return raw;
}
