// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Data conversion helpers between form data (flat key-value)
 * and evidence collection data (typed domain model).
 */

import type { FormData } from "../../templates/types";
import type { EvidenceCollectionData, CollectedItem } from "../report/types";

export function generateId(): string {
  return crypto.randomUUID();
}

/**
 * Generate an evidence collection ID that includes the case/project number.
 * Format: `{caseNumber}-EC-{shortUUID}` or `EC-{uuid}` if no case number.
 */
export function generateCollectionId(caseNumber?: string): string {
  const uuid = crypto.randomUUID();
  if (caseNumber) {
    const shortId = uuid.split("-")[0];
    return `${caseNumber}-EC-${shortId}`;
  }
  return `EC-${uuid}`;
}

/** Convert typed evidence data → flat form data for SchemaFormRenderer */
export function evidenceToFormData(d: EvidenceCollectionData): FormData {
  return {
    collection_date: d.collection_date || "",
    system_date_time: d.system_date_time || "",
    collecting_officer: d.collecting_officer || "",
    authorization: d.authorization || "",
    authorization_date: d.authorization_date || "",
    authorizing_authority: d.authorizing_authority || "",
    witnesses: d.witnesses || [],
    documentation_notes: d.documentation_notes || "",
    conditions: d.conditions || "",
    collected_items: (d.collected_items || []).map((item) => ({
      id: item.id,
      item_number: item.item_number || "",
      description: item.description || "",
      item_collection_datetime: item.item_collection_datetime || "",
      item_system_datetime: item.item_system_datetime || "",
      item_collecting_officer: item.item_collecting_officer || "",
      item_authorization: item.item_authorization || "",
      device_type: item.device_type || "desktop_computer",
      device_type_other: item.device_type_other || "",
      storage_interface: item.storage_interface || "sata",
      storage_interface_other: item.storage_interface_other || "",
      brand: item.brand || "",
      make: item.make || "",
      model: item.model || "",
      color: item.color || "",
      serial_number: item.serial_number || "",
      imei: item.imei || "",
      other_identifiers: item.other_identifiers || "",
      building: item.building || "",
      room: item.room || "",
      location_other: item.location_other || "",
      image_format: item.image_format || "",
      image_format_other: item.image_format_other || "",
      acquisition_method: item.acquisition_method || "",
      acquisition_method_other: item.acquisition_method_other || "",
      condition: item.condition || "good",
      packaging: item.packaging || "",
      storage_notes: item.storage_notes || "",
      notes: item.notes || "",
      photo_refs: item.photo_refs || [],
    })),
  };
}

/** Convert flat form data → typed evidence data */
export function formDataToEvidence(fd: FormData): EvidenceCollectionData {
  const witnesses = Array.isArray(fd.witnesses)
    ? (fd.witnesses as string[])
    : typeof fd.witnesses === "string"
      ? (fd.witnesses as string)
          .split(",")
          .map((s) => s.trim())
          .filter(Boolean)
      : [];

  const rawItems = Array.isArray(fd.collected_items)
    ? (fd.collected_items as FormData[])
    : [];

  const collected_items: CollectedItem[] = rawItems.map((item) => ({
    id: (item.id as string) || crypto.randomUUID(),
    item_number: (item.item_number as string) || "",
    description: (item.description as string) || "",
    item_collection_datetime: (item.item_collection_datetime as string) || undefined,
    item_system_datetime: (item.item_system_datetime as string) || undefined,
    item_collecting_officer: (item.item_collecting_officer as string) || undefined,
    item_authorization: (item.item_authorization as string) || undefined,
    device_type: (item.device_type as string) || "desktop_computer",
    device_type_other: (item.device_type_other as string) || undefined,
    storage_interface: (item.storage_interface as string) || "sata",
    storage_interface_other: (item.storage_interface_other as string) || undefined,
    brand: (item.brand as string) || undefined,
    make: (item.make as string) || undefined,
    model: (item.model as string) || undefined,
    color: (item.color as string) || undefined,
    serial_number: (item.serial_number as string) || undefined,
    imei: (item.imei as string) || undefined,
    other_identifiers: (item.other_identifiers as string) || undefined,
    building: (item.building as string) || undefined,
    room: (item.room as string) || undefined,
    location_other: (item.location_other as string) || undefined,
    image_format: (item.image_format as string) || undefined,
    image_format_other: (item.image_format_other as string) || undefined,
    acquisition_method: (item.acquisition_method as string) || undefined,
    acquisition_method_other: (item.acquisition_method_other as string) || undefined,
    condition: (item.condition as string) || "good",
    packaging: (item.packaging as string) || "",
    storage_notes: (item.storage_notes as string) || undefined,
    notes: (item.notes as string) || undefined,
    photo_refs: Array.isArray(item.photo_refs) ? (item.photo_refs as string[]) : undefined,
  }));

  return {
    collection_date: (fd.collection_date as string) || "",
    system_date_time: (fd.system_date_time as string) || undefined,
    collecting_officer: (fd.collecting_officer as string) || "",
    authorization: (fd.authorization as string) || "",
    authorization_date: (fd.authorization_date as string) || undefined,
    authorizing_authority: (fd.authorizing_authority as string) || undefined,
    witnesses,
    collected_items,
    documentation_notes: (fd.documentation_notes as string) || undefined,
    conditions: (fd.conditions as string) || undefined,
  };
}
