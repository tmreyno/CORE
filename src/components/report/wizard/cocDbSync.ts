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
 * DbEvidenceCollection, DbCollectedItem), then syncs via fire-and-forget
 * dbSync calls.
 */

import { invoke } from "@tauri-apps/api/core";
import { logger } from "../../../utils/logger";
import { dbSync } from "../../../hooks/project/useProjectDbSync";
import { nowISO } from "../../../types/project";
import type { COCItem, COCTransfer, EvidenceCollectionData, CollectedItem } from "../types";
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
    caseNumber: item.case_number || undefined,
    evidenceId: item.evidence_id || undefined,
    description: item.description,
    itemType: item.item_type || undefined,
    make: item.make,
    model: item.model,
    serialNumber: item.serial_number,
    capacity: item.capacity,
    condition: item.condition,
    acquisitionDate: item.acquisition_date || undefined,
    enteredCustodyDate: item.entered_custody_date || undefined,
    submittedBy: item.submitted_by || undefined,
    receivedBy: item.received_by || undefined,
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
  caseNumber?: string
): { collection: DbEvidenceCollection; items: DbCollectedItem[] } {
  const now = nowISO();
  const collection: DbEvidenceCollection = {
    id: collectionId,
    caseNumber: caseNumber || undefined,
    collectionDate: data.collection_date || undefined,
    collectionLocation: undefined, // location is now per-item (building/room/other)
    collectingOfficer: data.collecting_officer || undefined,
    authorization: data.authorization || undefined,
    authorizationDate: data.authorization_date,
    authorizingAuthority: data.authorizing_authority,
    witnessesJson:
      data.witnesses.length > 0 ? JSON.stringify(data.witnesses) : undefined,
    documentationNotes: data.documentation_notes,
    conditions: data.conditions,
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
  // Compose foundLocation from structured building/room/other fields
  const locationParts = [item.building, item.room, item.location_other].filter(Boolean);
  const foundLocation = locationParts.join(", ") || undefined;

  return {
    id: item.id,
    collectionId,
    cocItemId,
    evidenceFileId,
    itemNumber: item.item_number,
    description: item.description,
    foundLocation,
    itemType: item.device_type === "other" ? item.device_type_other : item.device_type || undefined,
    make: item.make,
    model: item.model,
    serialNumber: item.serial_number,
    condition: item.condition,
    packaging: item.packaging || undefined,
    photoRefsJson:
      item.photo_refs && item.photo_refs.length > 0
        ? JSON.stringify(item.photo_refs)
        : undefined,
    notes: item.notes,
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
  // Parse foundLocation back into structured fields (best effort)
  const locationParts = (db.foundLocation || "").split(",").map((s) => s.trim());

  return {
    id: db.id,
    item_number: db.itemNumber,
    description: db.description,
    device_type: db.itemType || "desktop_computer",
    storage_interface: "sata",
    building: locationParts[0] || undefined,
    room: locationParts[1] || undefined,
    location_other: locationParts.slice(2).join(", ") || undefined,
    make: db.make,
    model: db.model,
    serial_number: db.serialNumber,
    condition: db.condition,
    packaging: db.packaging || "",
    photo_refs: db.photoRefsJson ? JSON.parse(db.photoRefsJson) : [],
    notes: db.notes,
  };
}

// =============================================================================
// Persist all COC data to DB (fire-and-forget)
// =============================================================================

/**
 * Sync all COC items + transfers to the project DB.
 * Call this when the user leaves the data step or generates a report.
 */
export function persistCocItemsToDb(items: COCItem[]): void {
  for (const item of items) {
    dbSync.upsertCocItem(cocItemToDb(item));
    for (const transfer of item.transfers) {
      dbSync.upsertCocTransfer(cocTransferToDb(transfer, item.id));
    }
  }
  log.info(`Synced ${items.length} COC items to .ffxdb`);
}

/**
 * Sync evidence collection data to the project DB.
 * Uses a stable collection ID so repeated saves update the same record.
 */
export function persistEvidenceCollectionToDb(
  data: EvidenceCollectionData,
  collectionId: string,
  caseNumber?: string
): void {
  const { collection, items } = evidenceCollectionToDb(
    data,
    collectionId,
    caseNumber
  );
  dbSync.upsertEvidenceCollection(collection);
  for (const item of items) {
    dbSync.upsertCollectedItem(item);
  }
  log.info(
    `Synced evidence collection (${items.length} items) to .ffxdb`
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
