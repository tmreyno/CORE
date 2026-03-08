// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * COC prefill utility — maps evidence container metadata and evidence
 * collection form data to COCItem fields for Form 7-01 alignment.
 *
 * Data sources (priority order):
 * 1. Evidence collection forms (CollectedItem, EvidenceCollectionData)
 * 2. Container metadata (ContainerInfo — E01/AD1/UFED headers)
 * 3. Discovered file metadata (filename, size, type)
 */

import type { ContainerInfo } from "../../../../types";
import type { COCItem, EvidenceCollectionData, CollectedItem, HashValue, HashAlgorithmType } from "../../types";
import type { EvidenceGroup } from "../types";

/** Normalize a hash algorithm string to HashAlgorithmType union (best-effort) */
function normalizeAlgorithm(raw: string): HashAlgorithmType {
  const upper = raw.toUpperCase().replace(/[^A-Z0-9]/g, "");
  if (upper === "MD5") return "MD5";
  if (upper === "SHA1" || upper === "SHA160") return "SHA1";
  if (upper === "SHA256") return "SHA256";
  if (upper === "SHA512") return "SHA512";
  if (upper.includes("BLAKE2")) return "Blake2b";
  if (upper.includes("BLAKE3")) return "Blake3";
  if (upper === "XXH3") return "XXH3";
  if (upper === "XXH64") return "XXH64";
  return "SHA256"; // fallback
}

/** Build a HashValue from algorithm + hash string + item label */
function makeHash(algorithm: string, hash: string, item: string): HashValue {
  return { item, algorithm: normalizeAlgorithm(algorithm), value: hash };
}

// =============================================================================
// Container Type → Evidence Type Mapping (for COCItem.item_type)
// =============================================================================

const DEVICE_TYPE_TO_EVIDENCE_TYPE: Record<string, string> = {
  desktop_computer: "Desktop",
  laptop: "Laptop",
  mobile_phone: "MobileDevice",
  tablet: "MobileDevice",
  server: "Server",
  nas: "Server",
  external_hdd: "ExternalDrive",
  external_ssd: "ExternalDrive",
  usb_flash: "UsbDrive",
  sd_card: "MemoryCard",
  optical_disc: "OpticalMedia",
  flash_media: "MemoryCard",
  network_device: "NetworkDevice",
  printer: "Printer",
  camera: "Camera",
  drone: "Drone",
  wearable: "MobileDevice",
  gaming_console: "Other",
  iot_device: "Other",
  vehicle_infotainment: "Other",
  virtual_machine: "Other",
  cloud: "Other",
};

const CONTAINER_TO_EVIDENCE_TYPE: Record<string, string> = {
  ad1: "HardDrive",
  e01: "HardDrive",
  l01: "HardDrive",
  ex01: "HardDrive",
  raw: "HardDrive",
  ufed: "MobileDevice",
  ufd: "MobileDevice",
  ufdr: "MobileDevice",
  mem: "Other",
};

// =============================================================================
// Core: Build COCItem from Evidence Group + Container Info
// =============================================================================

/**
 * Build a pre-filled COCItem from an evidence group and its container info.
 *
 * Pulls data from:
 * - ContainerInfo (E01/AD1/UFED headers: serial, model, dates, examiner, hashes)
 * - DiscoveredFile (filename, size, container type)
 * - Examiner/case info from wizard context
 */
export function prefillCocFromContainer(
  group: EvidenceGroup,
  info: ContainerInfo | undefined,
  caseNumber: string,
  examiner: string,
  caseTitle?: string,
): Partial<COCItem> {
  const file = group.primaryFile;
  const fields: Partial<COCItem> = {
    case_number: caseNumber,
    case_title: caseTitle,
    evidence_id: "",
    description: group.baseName || file.filename,
    item_type: (CONTAINER_TO_EVIDENCE_TYPE[file.container_type.toLowerCase()] || "HardDrive") as COCItem["item_type"],
    condition: "",
    submitted_by: examiner,
    received_by: examiner,
    acquisition_date: "",
    entered_custody_date: "",
    intake_hashes: [],
    transfers: [],
  };

  if (!info) return fields;

  // --- E01 / L01 (EWF) ---
  const ewf = info.e01 || info.l01;
  if (ewf) {
    if (ewf.serial_number) fields.serial_number = ewf.serial_number;
    if (ewf.model) fields.model = ewf.model;
    if (ewf.evidence_number) fields.evidence_id = ewf.evidence_number;
    if (ewf.case_number && !fields.case_number) fields.case_number = ewf.case_number;
    if (ewf.examiner_name) fields.submitted_by = ewf.examiner_name;
    if (ewf.acquiry_date) {
      fields.acquisition_date = ewf.acquiry_date;
      fields.entered_custody_date = ewf.acquiry_date;
      fields.collected_date = ewf.acquiry_date;
    }
    if (ewf.description) fields.description = ewf.description;
    if (ewf.notes) fields.notes = ewf.notes;

    // E01 capacity from total_size
    if (ewf.total_size > 0) {
      fields.capacity = formatSize(ewf.total_size);
    }

    // Stored hashes → intake_hashes
    const hashes: HashValue[] = [];
    if (ewf.stored_hashes) {
      for (const h of ewf.stored_hashes) {
        if (h.hash) {
          hashes.push(makeHash(h.algorithm, h.hash, file.filename));
        }
      }
    }
    if (hashes.length > 0) fields.intake_hashes = hashes;
  }

  // --- AD1 ---
  if (info.ad1) {
    if (info.ad1.logical?.data_source_name) {
      fields.description = info.ad1.logical.data_source_name;
    }
    const cl = info.ad1.companion_log;
    if (cl) {
      if (cl.evidence_number) fields.evidence_id = cl.evidence_number;
      if (cl.case_number && !fields.case_number) fields.case_number = cl.case_number;
      if (cl.examiner) fields.submitted_by = cl.examiner;
      if (cl.acquisition_date) {
        fields.acquisition_date = cl.acquisition_date;
        fields.entered_custody_date = cl.acquisition_date;
        fields.collected_date = cl.acquisition_date;
      }
      if (cl.source_device) fields.make = cl.source_device;
      if (cl.notes) fields.notes = cl.notes;
      // Individual hash fields → intake_hashes
      const adHashes: HashValue[] = [];
      if (cl.md5_hash) adHashes.push(makeHash("MD5", cl.md5_hash, file.filename));
      if (cl.sha1_hash) adHashes.push(makeHash("SHA1", cl.sha1_hash, file.filename));
      if (cl.sha256_hash) adHashes.push(makeHash("SHA256", cl.sha256_hash, file.filename));
      if (adHashes.length > 0) fields.intake_hashes = adHashes;
    }
  }

  // --- UFED (Cellebrite) ---
  if (info.ufed) {
    const di = info.ufed.device_info;
    if (di) {
      if (di.vendor) fields.make = di.vendor;
      if (di.model) fields.model = di.model;
      if (di.full_name && !di.model) fields.model = di.full_name;
      if (di.serial_number) fields.serial_number = di.serial_number;
    }
    fields.item_type = "MobileDevice" as COCItem["item_type"];

    if (info.ufed.case_info?.examiner_name) {
      fields.submitted_by = info.ufed.case_info.examiner_name;
    }
    if (info.ufed.case_info?.case_identifier && !fields.case_number) {
      fields.case_number = info.ufed.case_info.case_identifier;
    }
    if (info.ufed.extraction_info?.start_time) {
      fields.acquisition_date = info.ufed.extraction_info.start_time;
      fields.entered_custody_date = info.ufed.extraction_info.start_time;
      fields.collected_date = info.ufed.extraction_info.start_time;
    }
    if (info.ufed.evidence_number) fields.evidence_id = info.ufed.evidence_number;
    if (info.ufed.case_info?.location) fields.source = info.ufed.case_info.location;
  }

  // --- Companion log (E01 companion .txt) ---
  if (info.companion_log && !info.ad1) {
    const cl = info.companion_log;
    if (cl.examiner) fields.submitted_by = cl.examiner;
    if (cl.case_number && !fields.case_number) fields.case_number = cl.case_number;
    if (cl.evidence_number) fields.evidence_id = cl.evidence_number;
    if (cl.acquisition_started) {
      fields.acquisition_date = cl.acquisition_started;
      fields.entered_custody_date = cl.acquisition_started;
      fields.collected_date = cl.acquisition_started;
    }
    if (cl.stored_hashes?.length) {
      const hashes: HashValue[] = cl.stored_hashes
        .filter((h) => h.hash)
        .map((h) => makeHash(h.algorithm, h.hash, file.filename));
      if (hashes.length > 0) fields.intake_hashes = hashes;
    }
  }

  return fields;
}

// =============================================================================
// Overlay: Enrich COCItem with Evidence Collection Data
// =============================================================================

/**
 * Overlay evidence collection form data onto a partially-populated COCItem.
 * Evidence collection data has higher priority for device details since it
 * represents the examiner's collected information.
 *
 * Matches by evidence_file_id (CollectedItem.evidence_file_id → evidence path).
 */
export function overlayCocFromCollection(
  item: Partial<COCItem>,
  collection: EvidenceCollectionData,
  collectedItem?: CollectedItem,
): Partial<COCItem> {
  // Header-level fields from collection
  if (collection.collecting_officer && !item.submitted_by) {
    item.submitted_by = collection.collecting_officer;
  }
  if (collection.collection_date) {
    if (!item.collected_date) item.collected_date = collection.collection_date;
    if (!item.acquisition_date) item.acquisition_date = collection.collection_date;
    if (!item.entered_custody_date) item.entered_custody_date = collection.collection_date;
  }
  if (collection.authorization) {
    // Map authorization type to collection method
    const auth = collection.authorization.toLowerCase();
    if (auth.includes("warrant")) item.collection_method = "search_warrant";
    else if (auth.includes("subpoena") || auth.includes("grand jury")) item.collection_method = "grand_jury_subpoena";
    else if (auth.includes("consent")) item.collection_method = "consent_seizure";
    else if (auth.includes("voluntary")) item.collection_method = "voluntary_submission";
    else if (auth.includes("digital") || auth.includes("electronic")) item.collection_method = "digital_electronic_capture";
    else if (auth.includes("abandon")) item.collection_method = "abandoned";
  }

  // Item-level fields from matched CollectedItem
  if (collectedItem) {
    // Device details (collection form overrides container defaults)
    if (collectedItem.make) item.make = collectedItem.make;
    if (collectedItem.model) item.model = collectedItem.model;
    if (collectedItem.serial_number) item.serial_number = collectedItem.serial_number;
    if (collectedItem.brand) item.make = collectedItem.brand; // brand is make alias
    if (collectedItem.description) item.description = collectedItem.description;
    if (collectedItem.condition) item.condition = collectedItem.condition;
    if (collectedItem.item_number) item.evidence_id = collectedItem.item_number;

    // Device type → evidence type mapping
    if (collectedItem.device_type) {
      const evidenceType = DEVICE_TYPE_TO_EVIDENCE_TYPE[collectedItem.device_type];
      if (evidenceType) item.item_type = evidenceType as COCItem["item_type"];
    }

    // Collection details
    if (collectedItem.item_collecting_officer) item.submitted_by = collectedItem.item_collecting_officer;
    if (collectedItem.item_collection_datetime) {
      item.collected_date = collectedItem.item_collection_datetime;
      if (!item.acquisition_date) item.acquisition_date = collectedItem.item_collection_datetime;
    }

    // Location
    const locationParts: string[] = [];
    if (collectedItem.building) locationParts.push(collectedItem.building);
    if (collectedItem.room) locationParts.push(collectedItem.room);
    if (collectedItem.location_other) locationParts.push(collectedItem.location_other);
    if (locationParts.length > 0) item.source = locationParts.join(", ");

    // Storage
    if (collectedItem.storage_notes) {
      item.storage_location = collectedItem.storage_notes;
    }
    if (collectedItem.notes) {
      item.notes = item.notes
        ? `${item.notes}\n${collectedItem.notes}`
        : collectedItem.notes;
    }
  }

  return item;
}

// =============================================================================
// Helpers
// =============================================================================

function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}
