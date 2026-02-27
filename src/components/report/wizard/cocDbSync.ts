// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Conversion utilities and DB persistence for COC and Evidence Collection forms.
 *
 * Converts wizard-local types (COCItem, COCTransfer, EvidenceCollectionData,
 * CollectedItem) to/from their DB counterparts (DbCocItem, DbCocTransfer,
 * DbEvidenceCollection, DbCollectedItem), then syncs via awaitable invoke calls
 * for forensic-critical data (COC items, evidence collections).
 */

import { invoke } from "@tauri-apps/api/core";
import { logger } from "../../../utils/logger";
import { nowISO } from "../../../types/project";
import type { COCItem, COCTransfer, EvidenceCollectionData, CollectedItem, ForensicReport } from "../types";
import type {
  DbCocItem,
  DbCocTransfer,
  DbEvidenceCollection,
  DbCollectedItem,
} from "../../../types/projectDb";

const log = logger.scope("CocDbSync");

// =============================================================================
// COCItem ↔ DbCocItem conversion
// =============================================================================

export function cocItemToDb(item: COCItem): DbCocItem {
  const now = nowISO();
  return {
    id: item.id,
    cocNumber: item.coc_number,
    // NOT NULL columns — use empty string fallback to avoid SQLite constraint violations
    caseNumber: item.case_number || "",
    evidenceId: item.evidence_id || "",
    description: item.description,
    itemType: item.item_type || "HardDrive",
    make: item.make,
    model: item.model,
    serialNumber: item.serial_number,
    capacity: item.capacity,
    condition: item.condition,
    acquisitionDate: item.acquisition_date || "",
    enteredCustodyDate: item.entered_custody_date || "",
    submittedBy: item.submitted_by || "",
    receivedBy: item.received_by || "",
    receivedLocation: item.received_location,
    storageLocation: item.storage_location,
    reasonSubmitted: item.reason_submitted,
    intakeHashesJson:
      item.intake_hashes.length > 0
        ? JSON.stringify(item.intake_hashes)
        : undefined,
    notes: item.notes,
    disposition: item.disposition,
    dispositionDate: item.disposition_date,
    dispositionNotes: item.disposition_notes,
    status: item.status || "draft",
    lockedAt: item.locked_at,
    lockedBy: item.locked_by,
    createdAt: now,
    modifiedAt: now,
  };
}

export function dbToCocItem(db: DbCocItem): COCItem {
  return {
    id: db.id,
    coc_number: db.cocNumber,
    evidence_id: db.evidenceId || "",
    case_number: db.caseNumber || "",
    description: db.description,
    item_type: (db.itemType || "HardDrive") as COCItem["item_type"],
    make: db.make,
    model: db.model,
    serial_number: db.serialNumber,
    capacity: db.capacity,
    condition: db.condition,
    acquisition_date: db.acquisitionDate || "",
    entered_custody_date: db.enteredCustodyDate || "",
    submitted_by: db.submittedBy || "",
    received_by: db.receivedBy || "",
    received_location: db.receivedLocation,
    storage_location: db.storageLocation,
    reason_submitted: db.reasonSubmitted,
    transfers: [], // loaded separately
    intake_hashes: db.intakeHashesJson
      ? JSON.parse(db.intakeHashesJson)
      : [],
    notes: db.notes,
    disposition: db.disposition as COCItem["disposition"],
    disposition_date: db.dispositionDate,
    disposition_notes: db.dispositionNotes,
  };
}

// =============================================================================
// COCTransfer ↔ DbCocTransfer conversion
// =============================================================================

export function cocTransferToDb(
  transfer: COCTransfer,
  cocItemId: string
): DbCocTransfer {
  return {
    id: transfer.id,
    cocItemId,
    timestamp: transfer.timestamp,
    releasedBy: transfer.released_by,
    receivedBy: transfer.received_by,
    purpose: transfer.purpose,
    location: transfer.location,
    method: transfer.method,
    notes: transfer.notes,
  };
}

export function dbToCocTransfer(db: DbCocTransfer): COCTransfer {
  return {
    id: db.id,
    timestamp: db.timestamp,
    released_by: db.releasedBy,
    received_by: db.receivedBy,
    purpose: db.purpose,
    location: db.location,
    method: db.method,
    notes: db.notes,
  };
}

// =============================================================================
// EvidenceCollectionData ↔ DbEvidenceCollection / DbCollectedItem conversion
// =============================================================================

export function evidenceCollectionToDb(
  data: EvidenceCollectionData,
  collectionId: string,
  caseNumber?: string,
  status?: string
): { collection: DbEvidenceCollection; items: DbCollectedItem[] } {
  const now = nowISO();
  const collection: DbEvidenceCollection = {
    id: collectionId,
    // NOT NULL columns — use empty string fallback to avoid SQLite constraint violations
    caseNumber: caseNumber || "",
    collectionDate: data.collection_date || "",
    collectionLocation: "", // location is now per-item (building/room/other)
    collectingOfficer: data.collecting_officer || "",
    authorization: data.authorization || "",
    authorizationDate: data.authorization_date,
    authorizingAuthority: data.authorizing_authority,
    witnessesJson:
      data.witnesses.length > 0 ? JSON.stringify(data.witnesses) : undefined,
    documentationNotes: data.documentation_notes,
    conditions: data.conditions,
    status: status || "draft",
    createdAt: now,
    modifiedAt: now,
  };

  const items: DbCollectedItem[] = data.collected_items.map((item) =>
    collectedItemToDb(item, collectionId)
  );

  return { collection, items };
}

export function collectedItemToDb(
  item: CollectedItem,
  collectionId: string,
  cocItemId?: string,
  evidenceFileId?: string
): DbCollectedItem {
  // Compose foundLocation from structured building/room/other fields (legacy compat)
  const locationParts = [item.building, item.room, item.location_other].filter(Boolean);
  const foundLocation = locationParts.join(", ") || "";

  return {
    id: item.id,
    collectionId,
    cocItemId,
    evidenceFileId,
    // NOT NULL columns — use empty string fallback to avoid SQLite constraint violations
    itemNumber: item.item_number || "",
    description: item.description || "",
    foundLocation,
    itemType: item.device_type === "other" ? (item.device_type_other || "") : (item.device_type || ""),
    make: item.make,
    model: item.model,
    serialNumber: item.serial_number,
    condition: item.condition || "good",
    packaging: item.packaging || "",
    photoRefsJson:
      item.photo_refs && item.photo_refs.length > 0
        ? JSON.stringify(item.photo_refs)
        : undefined,
    notes: item.notes,

    // Per-item collection info (v8)
    itemCollectionDatetime: item.item_collection_datetime || undefined,
    itemSystemDatetime: item.item_system_datetime || undefined,
    itemCollectingOfficer: item.item_collecting_officer || undefined,
    itemAuthorization: item.item_authorization || undefined,

    // Device identification (v8)
    deviceType: item.device_type || undefined,
    deviceTypeOther: item.device_type_other || undefined,
    storageInterface: item.storage_interface || undefined,
    storageInterfaceOther: item.storage_interface_other || undefined,
    brand: item.brand || undefined,
    color: item.color || undefined,
    imei: item.imei || undefined,
    otherIdentifiers: item.other_identifiers || undefined,

    // Location (v8)
    building: item.building || undefined,
    room: item.room || undefined,
    locationOther: item.location_other || undefined,

    // Forensic image (v8)
    imageFormat: item.image_format || undefined,
    imageFormatOther: item.image_format_other || undefined,
    acquisitionMethod: item.acquisition_method || undefined,
    acquisitionMethodOther: item.acquisition_method_other || undefined,

    // Additional info (v8)
    storageNotes: item.storage_notes || undefined,
  };
}

export function dbToEvidenceCollectionData(
  collection: DbEvidenceCollection,
  items: DbCollectedItem[]
): EvidenceCollectionData {
  return {
    collection_date: collection.collectionDate || "",
    collecting_officer: collection.collectingOfficer || "",
    authorization: collection.authorization || "",
    authorization_date: collection.authorizationDate,
    authorizing_authority: collection.authorizingAuthority,
    witnesses: collection.witnessesJson
      ? JSON.parse(collection.witnessesJson)
      : [],
    collected_items: items.map(dbToCollectedItem),
    documentation_notes: collection.documentationNotes,
    conditions: collection.conditions,
  };
}

export function dbToCollectedItem(db: DbCollectedItem): CollectedItem {
  // Use structured location fields if available, else parse legacy foundLocation
  const hasStructuredLocation = db.building || db.room || db.locationOther;
  let building: string | undefined;
  let room: string | undefined;
  let locationOther: string | undefined;

  if (hasStructuredLocation) {
    building = db.building || undefined;
    room = db.room || undefined;
    locationOther = db.locationOther || undefined;
  } else {
    // Legacy fallback: parse comma-separated foundLocation
    const locationParts = (db.foundLocation || "").split(",").map((s) => s.trim());
    building = locationParts[0] || undefined;
    room = locationParts[1] || undefined;
    locationOther = locationParts.slice(2).join(", ") || undefined;
  }

  return {
    id: db.id,
    item_number: db.itemNumber,
    description: db.description,

    // Per-item collection info (v8)
    item_collection_datetime: db.itemCollectionDatetime || undefined,
    item_system_datetime: db.itemSystemDatetime || undefined,
    item_collecting_officer: db.itemCollectingOfficer || undefined,
    item_authorization: db.itemAuthorization || undefined,

    // Device identification — prefer structured v8 fields, fall back to legacy
    device_type: db.deviceType || db.itemType || "desktop_computer",
    device_type_other: db.deviceTypeOther || undefined,
    storage_interface: db.storageInterface || "sata",
    storage_interface_other: db.storageInterfaceOther || undefined,
    brand: db.brand || undefined,
    make: db.make,
    model: db.model,
    color: db.color || undefined,
    serial_number: db.serialNumber,
    imei: db.imei || undefined,
    other_identifiers: db.otherIdentifiers || undefined,

    // Location
    building,
    room,
    location_other: locationOther,

    // Forensic image (v8)
    image_format: db.imageFormat || undefined,
    image_format_other: db.imageFormatOther || undefined,
    acquisition_method: db.acquisitionMethod || undefined,
    acquisition_method_other: db.acquisitionMethodOther || undefined,

    // Condition & packaging
    condition: db.condition,
    packaging: db.packaging || "",

    // Additional info
    storage_notes: db.storageNotes || undefined,
    notes: db.notes,
    photo_refs: db.photoRefsJson ? JSON.parse(db.photoRefsJson) : [],
  };
}

// =============================================================================
// Persist all COC data to DB (awaitable)
// =============================================================================

/**
 * Sync all COC items + transfers to the project DB.
 * Call this when the user leaves the data step or generates a report.
 * Uses direct invoke (awaitable) instead of fire-and-forget dbSync so callers
 * know the save succeeded — critical for forensic chain of custody data.
 */
export async function persistCocItemsToDb(items: COCItem[]): Promise<void> {
  for (const item of items) {
    const dbItem = cocItemToDb(item);
    await invoke("project_db_upsert_coc_item", { item: dbItem });
    for (const transfer of item.transfers) {
      const dbTransfer = cocTransferToDb(transfer, item.id);
      await invoke("project_db_upsert_coc_transfer", { transfer: dbTransfer });
    }
  }
  log.info(`Synced ${items.length} COC items to .ffxdb`);
}

/**
 * Sync evidence collection data to the project DB.
 * Uses a stable collection ID so repeated saves update the same record.
 * Returns a promise so callers can await the save before status transitions.
 */
export async function persistEvidenceCollectionToDb(
  data: EvidenceCollectionData,
  collectionId: string,
  caseNumber?: string,
  status?: string
): Promise<void> {
  const { collection, items } = evidenceCollectionToDb(
    data,
    collectionId,
    caseNumber,
    status
  );
  // Use direct invoke (awaitable) instead of fire-and-forget dbSync
  // so callers know the save succeeded before doing status transitions
  await invoke("project_db_upsert_evidence_collection", { record: collection });
  for (const item of items) {
    await invoke("project_db_upsert_collected_item", { record: item });
  }
  log.info(
    `Saved evidence collection (${items.length} items) to .ffxdb`
  );
}

// =============================================================================
// Load COC data from DB (async — for initial wizard population)
// =============================================================================

/**
 * Load existing COC items + transfers from the project DB.
 * Returns items with their transfers populated.
 */
export async function loadCocItemsFromDb(
  caseNumber?: string
): Promise<COCItem[]> {
  try {
    const dbItems = await invoke<DbCocItem[]>("project_db_get_coc_items", {
      caseNumber: caseNumber || null,
    });
    const allTransfers = await invoke<DbCocTransfer[]>(
      "project_db_get_all_coc_transfers"
    );

    // Group transfers by COC item ID
    const transferMap = new Map<string, DbCocTransfer[]>();
    for (const t of allTransfers) {
      const arr = transferMap.get(t.cocItemId) || [];
      arr.push(t);
      transferMap.set(t.cocItemId, arr);
    }

    return dbItems.map((db) => {
      const item = dbToCocItem(db);
      const transfers = (transferMap.get(db.id) || []).map(dbToCocTransfer);
      return { ...item, transfers };
    });
  } catch (e) {
    log.warn("Failed to load COC items from DB:", e);
    return [];
  }
}

/**
 * Load existing evidence collections from the project DB.
 * Returns the most recent collection with its items populated.
 */
export async function loadEvidenceCollectionFromDb(
  caseNumber?: string
): Promise<{ data: EvidenceCollectionData; collectionId: string } | null> {
  try {
    const collections = await invoke<DbEvidenceCollection[]>(
      "project_db_get_evidence_collections",
      { caseNumber: caseNumber || null }
    );
    if (collections.length === 0) return null;

    // Use the most recently modified collection
    const latest = collections[0];
    const items = await invoke<DbCollectedItem[]>(
      "project_db_get_collected_items",
      { collectionId: latest.id }
    );

    return {
      data: dbToEvidenceCollectionData(latest, items),
      collectionId: latest.id,
    };
  } catch (e) {
    log.warn("Failed to load evidence collection from DB:", e);
    return null;
  }
}

/**
 * Load a specific evidence collection by its ID.
 * Returns the collection with its items populated, or null if not found.
 */
export async function loadEvidenceCollectionById(
  collectionId: string
): Promise<{ data: EvidenceCollectionData; collectionId: string; status: string } | null> {
  try {
    const collection = await invoke<DbEvidenceCollection>(
      "project_db_get_evidence_collection_by_id",
      { id: collectionId }
    );
    const items = await invoke<DbCollectedItem[]>(
      "project_db_get_collected_items",
      { collectionId: collection.id }
    );

    return {
      data: dbToEvidenceCollectionData(collection, items),
      collectionId: collection.id,
      status: collection.status || "draft",
    };
  } catch (e) {
    log.warn("Failed to load evidence collection by ID:", e);
    return null;
  }
}

/**
 * Load all evidence collections (list view — metadata only, no items).
 */
export async function loadAllEvidenceCollections(
  caseNumber?: string
): Promise<DbEvidenceCollection[]> {
  try {
    return await invoke<DbEvidenceCollection[]>(
      "project_db_get_evidence_collections",
      { caseNumber: caseNumber || null }
    );
  } catch (e) {
    log.warn("Failed to load evidence collections:", e);
    return [];
  }
}

/**
 * Update the status of an evidence collection (draft → complete → locked).
 */
export async function updateEvidenceCollectionStatus(
  collectionId: string,
  newStatus: string
): Promise<boolean> {
  try {
    await invoke("project_db_update_evidence_collection_status", {
      id: collectionId,
      newStatus,
    });
    return true;
  } catch (e) {
    log.warn("Failed to update evidence collection status:", e);
    return false;
  }
}

/**
 * Delete an evidence collection (cascades to collected items).
 */
export async function deleteEvidenceCollection(
  collectionId: string
): Promise<boolean> {
  try {
    await invoke("project_db_delete_evidence_collection", { id: collectionId });
    return true;
  } catch (e) {
    log.warn("Failed to delete evidence collection:", e);
    return false;
  }
}

/**
 * Export an evidence collection as PDF.
 * Loads the collection, builds a minimal ForensicReport, shows a save dialog,
 * and invokes the generate_report Tauri command.
 *
 * @returns The output file path on success, or null if cancelled/failed.
 */
export async function exportEvidenceCollectionPdf(
  collectionId: string,
  caseNumber?: string,
): Promise<string | null> {
  try {
    const result = await loadEvidenceCollectionById(collectionId);
    if (!result) {
      log.warn("Collection not found for PDF export:", collectionId);
      return null;
    }

    const { data } = result;

    // Build minimal ForensicReport for the PDF generator
    const report: ForensicReport = {
      metadata: {
        title: "Evidence Collection Report",
        report_number: caseNumber || "EC-001",
        version: "1.0",
        classification: "LawEnforcementSensitive",
        generated_at: new Date().toISOString(),
        generated_by: "CORE-FFX",
      },
      case_info: {
        case_number: caseNumber || "",
        case_name: "",
        agency: "",
        requestor: data.collecting_officer || "",
      },
      examiner: {
        name: data.collecting_officer || "",
        title: "",
        organization: "",
        phone: "",
        email: "",
        certifications: [],
      },
      report_type: "forensic_examination",
      evidence_items: [],
      chain_of_custody: [],
      findings: [],
      timeline: [],
      hash_records: [],
      tools: [],
      appendices: [],
      evidence_collection: data,
    };

    // Show save dialog
    const { save } = await import("@tauri-apps/plugin-dialog");
    const defaultName = `Evidence_Collection_${caseNumber || "report"}_${new Date().toISOString().split("T")[0]}.pdf`;
    const path = await save({
      defaultPath: defaultName,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });

    if (!path) return null;

    // Generate PDF
    const outputPath = await invoke<string>("generate_report", {
      report,
      format: "Pdf",
      outputPath: path,
    });

    log.info("Evidence collection PDF exported:", outputPath);
    return outputPath;
  } catch (e) {
    log.error("Failed to export evidence collection PDF:", e);
    return null;
  }
}

/** Supported evidence collection export formats */
export type EvidenceExportFormat = "pdf" | "csv" | "xlsx" | "html";

const FORMAT_LABELS: Record<EvidenceExportFormat, { name: string; extension: string }> = {
  pdf: { name: "PDF Document", extension: "pdf" },
  csv: { name: "CSV Spreadsheet", extension: "csv" },
  xlsx: { name: "Excel Spreadsheet", extension: "xlsx" },
  html: { name: "HTML Report", extension: "html" },
};

/**
 * Export an evidence collection in the specified format.
 * Loads the collection, shows a save dialog, and invokes the
 * export_evidence_collection Tauri command.
 *
 * @returns The output file path on success, or null if cancelled/failed.
 */
export async function exportEvidenceCollection(
  collectionId: string,
  format: EvidenceExportFormat,
  caseNumber?: string,
): Promise<string | null> {
  try {
    const result = await loadEvidenceCollectionById(collectionId);
    if (!result) {
      log.warn("Collection not found for export:", collectionId);
      return null;
    }

    const { data } = result;
    const info = FORMAT_LABELS[format];
    const { save } = await import("@tauri-apps/plugin-dialog");
    const dateSuffix = new Date().toISOString().split("T")[0];
    const defaultName = `Evidence_Collection_${caseNumber || "report"}_${dateSuffix}.${info.extension}`;

    const path = await save({
      defaultPath: defaultName,
      filters: [{ name: info.name, extensions: [info.extension] }],
    });
    if (!path) return null;

    const outputPath = await invoke<string>("export_evidence_collection", {
      data,
      caseNumber: caseNumber || "",
      format,
      outputPath: path,
    });

    log.info(`Evidence collection ${format.toUpperCase()} exported:`, outputPath);
    return outputPath;
  } catch (e) {
    log.error(`Failed to export evidence collection as ${format}:`, e);
    return null;
  }
}
