// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Evidence Matching & Conflict Detection Engine
 *
 * Matches collected items (user-entered evidence collection form data) to
 * evidence containers (discovered files with container metadata). When conflicts
 * are detected between user-entered values and container-extracted values,
 * builds a structured diff that the UI presents for field-level resolution.
 *
 * Matching criteria (in priority order):
 *   1. evidence_file_id — direct FK link (already matched)
 *   2. item_number + container evidence_number
 *   3. serial_number
 *   4. Container filename contains item description or vice versa
 */

import type { DiscoveredFile, ContainerInfo } from "../../types";
import type { DbCollectedItem, DbEvidenceDataAlternative } from "../../types/projectDb";
import { extractItemFieldsFromEvidence } from "./evidenceAutoFill";

// =============================================================================
// Types
// =============================================================================

/** A single field-level conflict between user and container data */
export interface FieldConflict {
  /** The field name (e.g., "serial_number", "model") */
  fieldName: string;
  /** Human-readable label for the field */
  fieldLabel: string;
  /** The value the user entered */
  userValue: string;
  /** The value extracted from the container */
  containerValue: string;
  /** Which source is currently chosen ("user" | "container") */
  chosenSource: "user" | "container";
}

/** A match between a collected item and an evidence container */
export interface EvidenceMatch {
  /** The collected item */
  collectedItem: DbCollectedItem;
  /** The matched evidence file */
  evidenceFile: DiscoveredFile;
  /** Container info for the matched file */
  containerInfo: ContainerInfo | undefined;
  /** How the match was found */
  matchType: "evidence_file_id" | "item_number" | "serial_number" | "filename";
  /** Match confidence: high = exact ID/serial, medium = item number, low = filename */
  confidence: "high" | "medium" | "low";
  /** Field-level conflicts detected */
  conflicts: FieldConflict[];
  /** Fields that match (no conflict) — for reference */
  matchingFields: string[];
  /** Fields only in container (user didn't enter anything) */
  containerOnlyFields: { fieldName: string; fieldLabel: string; value: string }[];
}

/** Result of running the matching engine across all items */
export interface MatchingResult {
  /** Collected items that matched an evidence container */
  matched: EvidenceMatch[];
  /** Collected items with no matching container found */
  unmatchedItems: DbCollectedItem[];
  /** Evidence files with no matching collected item */
  unmatchedFiles: DiscoveredFile[];
  /** Total conflicts found across all matches */
  totalConflicts: number;
  /** Total container-only fields (enrichment opportunities) */
  totalEnrichments: number;
}

// =============================================================================
// Field Labels for Human-Readable Display
// =============================================================================

const FIELD_LABELS: Record<string, string> = {
  description: "Description",
  serial_number: "Serial Number",
  model: "Model",
  brand: "Brand / Make",
  make: "Make",
  imei: "IMEI",
  item_number: "Item Number",
  device_type: "Device Type",
  image_format: "Image Format",
  acquisition_method: "Acquisition Method",
  connection_method: "Connection Method",
  item_collection_datetime: "Collection Date/Time",
  item_system_datetime: "System Date/Time",
  item_collecting_officer: "Collecting Officer",
  notes: "Notes",
  other_identifiers: "Other Identifiers",
  storage_notes: "Storage Notes",
  building: "Building / Location",
  room: "Room",
  storage_interface: "Storage Interface",
  color: "Color",
  condition: "Condition",
  packaging: "Packaging",
};

/** Fields compared for conflicts (snake_case — matching form field names and
 *  extractItemFieldsFromEvidence() output keys) */
const COMPARABLE_FIELDS: string[] = [
  "description",
  "serial_number",
  "model",
  "brand",
  "make",
  "imei",
  "item_number",
  "device_type",
  "image_format",
  "acquisition_method",
  "item_collection_datetime",
  "item_system_datetime",
  "item_collecting_officer",
  "notes",
  "other_identifiers",
  "storage_notes",
  "building",
  "room",
  "storage_interface",
  "color",
];

// =============================================================================
// Matching Engine
// =============================================================================

/**
 * Match collected items to evidence containers and detect field-level conflicts.
 *
 * @param collectedItems - Items from the evidence collection form
 * @param discoveredFiles - Evidence files discovered by the file manager
 * @param infoMap - Container metadata map (path → ContainerInfo)
 * @param caseNumber - Optional case number for context
 */
export function matchEvidenceToCollectedItems(
  collectedItems: DbCollectedItem[],
  discoveredFiles: DiscoveredFile[],
  infoMap: Map<string, ContainerInfo>,
  caseNumber?: string,
): MatchingResult {
  const matched: EvidenceMatch[] = [];
  const matchedFileIndices = new Set<number>();
  const matchedItemIndices = new Set<number>();

  // First pass: match by evidence_file_id (direct FK — highest confidence)
  for (let i = 0; i < collectedItems.length; i++) {
    const item = collectedItems[i];
    if (!item.evidenceFileId) continue;

    const fileIdx = discoveredFiles.findIndex(
      (f) => f.path === item.evidenceFileId,
    );
    if (fileIdx >= 0) {
      const file = discoveredFiles[fileIdx];
      const info = infoMap.get(file.path);
      const match = buildMatch(item, file, info, "evidence_file_id", "high", caseNumber);
      matched.push(match);
      matchedFileIndices.add(fileIdx);
      matchedItemIndices.add(i);
    }
  }

  // Second pass: match by item_number ↔ evidence_number
  for (let i = 0; i < collectedItems.length; i++) {
    if (matchedItemIndices.has(i)) continue;
    const item = collectedItems[i];
    if (!item.itemNumber) continue;

    const normalizedItemNum = normalizeString(item.itemNumber);

    for (let j = 0; j < discoveredFiles.length; j++) {
      if (matchedFileIndices.has(j)) continue;
      const file = discoveredFiles[j];
      const info = infoMap.get(file.path);
      if (!info) continue;

      const evidenceNumber = getEvidenceNumber(info);
      if (evidenceNumber && normalizeString(evidenceNumber) === normalizedItemNum) {
        const match = buildMatch(item, file, info, "item_number", "medium", caseNumber);
        matched.push(match);
        matchedFileIndices.add(j);
        matchedItemIndices.add(i);
        break;
      }
    }
  }

  // Third pass: match by serial_number
  for (let i = 0; i < collectedItems.length; i++) {
    if (matchedItemIndices.has(i)) continue;
    const item = collectedItems[i];
    if (!item.serialNumber) continue;

    const normalizedSerial = normalizeString(item.serialNumber);

    for (let j = 0; j < discoveredFiles.length; j++) {
      if (matchedFileIndices.has(j)) continue;
      const file = discoveredFiles[j];
      const info = infoMap.get(file.path);
      if (!info) continue;

      const containerSerial = getContainerSerial(info);
      if (containerSerial && normalizeString(containerSerial) === normalizedSerial) {
        const match = buildMatch(item, file, info, "serial_number", "high", caseNumber);
        matched.push(match);
        matchedFileIndices.add(j);
        matchedItemIndices.add(i);
        break;
      }
    }
  }

  // Fourth pass: match by filename similarity
  for (let i = 0; i < collectedItems.length; i++) {
    if (matchedItemIndices.has(i)) continue;
    const item = collectedItems[i];
    if (!item.description) continue;

    const normalizedDesc = normalizeString(item.description);

    for (let j = 0; j < discoveredFiles.length; j++) {
      if (matchedFileIndices.has(j)) continue;
      const file = discoveredFiles[j];
      const normalizedFilename = normalizeString(file.filename);

      // Check if description contains filename or vice versa
      if (
        normalizedDesc === normalizedFilename ||
        normalizedDesc.includes(normalizedFilename) ||
        normalizedFilename.includes(normalizedDesc)
      ) {
        const info = infoMap.get(file.path);
        const match = buildMatch(item, file, info, "filename", "low", caseNumber);
        matched.push(match);
        matchedFileIndices.add(j);
        matchedItemIndices.add(i);
        break;
      }
    }
  }

  // Collect unmatched
  const unmatchedItems = collectedItems.filter((_, i) => !matchedItemIndices.has(i));
  const unmatchedFiles = discoveredFiles.filter((_, i) => !matchedFileIndices.has(i));

  const totalConflicts = matched.reduce((sum, m) => sum + m.conflicts.length, 0);
  const totalEnrichments = matched.reduce((sum, m) => sum + m.containerOnlyFields.length, 0);

  return { matched, unmatchedItems, unmatchedFiles, totalConflicts, totalEnrichments };
}

// =============================================================================
// Build Match with Conflict Detection
// =============================================================================

function buildMatch(
  item: DbCollectedItem,
  file: DiscoveredFile,
  info: ContainerInfo | undefined,
  matchType: EvidenceMatch["matchType"],
  confidence: EvidenceMatch["confidence"],
  caseNumber?: string,
): EvidenceMatch {
  const containerFields = extractItemFieldsFromEvidence(file, info, caseNumber);
  const conflicts: FieldConflict[] = [];
  const matchingFields: string[] = [];
  const containerOnlyFields: { fieldName: string; fieldLabel: string; value: string }[] = [];

  for (const field of COMPARABLE_FIELDS) {
    const userVal = normalizeValue(getItemField(item, field));
    const containerVal = normalizeValue(containerFields[field]);

    if (!userVal && !containerVal) continue; // Both empty — skip

    if (!userVal && containerVal) {
      // Container has data user didn't enter → enrichment opportunity
      containerOnlyFields.push({
        fieldName: field,
        fieldLabel: FIELD_LABELS[field] || field,
        value: containerFields[field],
      });
    } else if (userVal && containerVal && userVal !== containerVal) {
      // Both have different values → conflict
      conflicts.push({
        fieldName: field,
        fieldLabel: FIELD_LABELS[field] || field,
        userValue: getItemField(item, field),
        containerValue: containerFields[field],
        chosenSource: "user", // Default: prefer user-entered data
      });
    } else if (userVal && containerVal && userVal === containerVal) {
      // Values match — no conflict
      matchingFields.push(field);
    }
    // userVal && !containerVal → user-only field, no action needed
  }

  return {
    collectedItem: item,
    evidenceFile: file,
    containerInfo: info,
    matchType,
    confidence,
    conflicts,
    matchingFields,
    containerOnlyFields,
  };
}

// =============================================================================
// Convert Conflict Resolutions to Alternative Records
// =============================================================================

/**
 * Create DbEvidenceDataAlternative records from resolved conflicts.
 * Call this after the user has made their choices.
 *
 * @param match - The evidence match with user-resolved conflicts
 * @param resolvedBy - The examiner who resolved the conflicts
 */
export function buildAlternativeRecords(
  match: EvidenceMatch,
  resolvedBy: string,
): DbEvidenceDataAlternative[] {
  const now = new Date().toISOString();
  return match.conflicts.map((c) => ({
    id: `eda-${match.collectedItem.id}-${c.fieldName}-${Date.now()}`,
    collectedItemId: match.collectedItem.id,
    evidenceFileId: match.evidenceFile.path,
    fieldName: c.fieldName,
    chosenSource: c.chosenSource,
    userValue: c.userValue,
    containerValue: c.containerValue,
    resolvedBy,
    resolvedAt: now,
  }));
}

/**
 * Apply resolved conflicts to a collected item. Returns a new item with
 * the chosen values applied and container-only fields populated.
 *
 * @param item - Original collected item
 * @param match - The evidence match with resolved conflicts
 */
export function applyResolutions(
  item: DbCollectedItem,
  match: EvidenceMatch,
): DbCollectedItem {
  const updated = { ...item };

  // Apply conflict resolutions
  for (const conflict of match.conflicts) {
    const value =
      conflict.chosenSource === "container"
        ? conflict.containerValue
        : conflict.userValue;
    setItemField(updated, conflict.fieldName, value);
  }

  // Apply container-only enrichments (user had no data — container fills the gap)
  for (const enrichment of match.containerOnlyFields) {
    setItemField(updated, enrichment.fieldName, enrichment.value);
  }

  // Link to evidence file if not already linked
  if (!updated.evidenceFileId) {
    updated.evidenceFileId = match.evidenceFile.path;
  }

  return updated;
}

// =============================================================================
// Helpers
// =============================================================================

/** Normalize a string for comparison (lowercase, trim, collapse whitespace) */
function normalizeString(s: string): string {
  return s.toLowerCase().trim().replace(/\s+/g, " ");
}

/** Normalize a value for comparison (trim whitespace, lowercase) */
function normalizeValue(s: string | undefined | null): string {
  if (!s) return "";
  return s.trim().toLowerCase();
}

/** Get evidence number from container info */
function getEvidenceNumber(info: ContainerInfo): string | undefined {
  if (info.e01?.evidence_number) return info.e01.evidence_number;
  if (info.l01?.evidence_number) return info.l01.evidence_number;
  if (info.ad1?.companion_log?.evidence_number) return info.ad1.companion_log.evidence_number;
  if (info.ufed?.evidence_number) return info.ufed.evidence_number;
  return undefined;
}

/** Get serial number from container info */
function getContainerSerial(info: ContainerInfo): string | undefined {
  if (info.e01?.serial_number) return info.e01.serial_number;
  if (info.l01?.serial_number) return info.l01.serial_number;
  if (info.ufed?.device_info?.serial_number) return info.ufed.device_info.serial_number;
  return undefined;
}

/** Get a field value from a DbCollectedItem by snake_case field name.
 *  DbCollectedItem uses camelCase props; extractItemFieldsFromEvidence uses snake_case. */
function getItemField(item: DbCollectedItem, snakeField: string): string {
  const camelField = toCamelCase(snakeField);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const val = (item as any)[camelField];
  return val ?? "";
}

/** Set a field value on a DbCollectedItem by snake_case field name */
function setItemField(item: DbCollectedItem, snakeField: string, value: string): void {
  const camelField = toCamelCase(snakeField);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (item as any)[camelField] = value;
}

/** Convert snake_case to camelCase */
function toCamelCase(s: string): string {
  return s.replace(/_([a-z])/g, (_, c) => c.toUpperCase());
}
