// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Type converters between wizard-local types and their DB counterparts.
 *
 * Converts COCItem ↔ DbCocItem, COCTransfer ↔ DbCocTransfer,
 * EvidenceCollectionData ↔ DbEvidenceCollection, CollectedItem ↔ DbCollectedItem.
 */

import { nowISO } from "../../../types/project";
import type { COCItem, COCTransfer, EvidenceCollectionData, CollectedItem } from "../types";
import type {
  DbCocItem,
  DbCocTransfer,
  DbEvidenceCollection,
  DbCollectedItem,
} from "../../../types/projectDb";

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
    // Form 7-01 Header
    caseTitle: item.case_title,
    office: item.office,
    // Owner / Source / Contact
    ownerName: item.owner_name,
    ownerAddress: item.owner_address,
    ownerPhone: item.owner_phone,
    source: item.source,
    otherContactName: item.other_contact_name,
    otherContactRelation: item.other_contact_relation,
    otherContactPhone: item.other_contact_phone,
    // Collection Method
    collectionMethod: item.collection_method,
    collectionMethodOther: item.collection_method_other,
    // Item Details
    make: item.make,
    model: item.model,
    serialNumber: item.serial_number,
    capacity: item.capacity,
    condition: item.condition,
    // Custody / Collection
    acquisitionDate: item.acquisition_date || "",
    enteredCustodyDate: item.entered_custody_date || "",
    submittedBy: item.submitted_by || "",
    collectedDate: item.collected_date,
    receivedBy: item.received_by || "",
    receivedLocation: item.received_location,
    storageLocation: item.storage_location,
    reasonSubmitted: item.reason_submitted,
    intakeHashesJson:
      item.intake_hashes.length > 0
        ? JSON.stringify(item.intake_hashes)
        : undefined,
    notes: item.notes,
    // Final Disposition
    disposition: item.disposition,
    dispositionBy: item.disposition_by,
    returnedTo: item.returned_to,
    destructionDate: item.destruction_date,
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
    // Form 7-01 Header
    case_title: db.caseTitle,
    office: db.office,
    // Owner / Source / Contact
    owner_name: db.ownerName,
    owner_address: db.ownerAddress,
    owner_phone: db.ownerPhone,
    source: db.source,
    other_contact_name: db.otherContactName,
    other_contact_relation: db.otherContactRelation,
    other_contact_phone: db.otherContactPhone,
    // Collection Method
    collection_method: db.collectionMethod,
    collection_method_other: db.collectionMethodOther,
    // Item Details
    make: db.make,
    model: db.model,
    serial_number: db.serialNumber,
    capacity: db.capacity,
    condition: db.condition,
    // Custody / Collection
    acquisition_date: db.acquisitionDate || "",
    entered_custody_date: db.enteredCustodyDate || "",
    submitted_by: db.submittedBy || "",
    collected_date: db.collectedDate,
    received_by: db.receivedBy || "",
    received_location: db.receivedLocation,
    storage_location: db.storageLocation,
    reason_submitted: db.reasonSubmitted,
    transfers: [], // loaded separately
    intake_hashes: db.intakeHashesJson
      ? JSON.parse(db.intakeHashesJson)
      : [],
    notes: db.notes,
    // Final Disposition
    disposition: db.disposition as COCItem["disposition"],
    disposition_by: db.dispositionBy,
    returned_to: db.returnedTo,
    destruction_date: db.destructionDate,
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
    storageLocation: transfer.storage_location,
    storageDate: transfer.storage_date,
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
    storage_location: db.storageLocation,
    storage_date: db.storageDate,
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
    collectedItemToDb(item, collectionId, item.coc_item_id, item.evidence_file_id)
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

    // Linkage
    evidence_file_id: db.evidenceFileId || undefined,
    coc_item_id: db.cocItemId || undefined,
  };
}
