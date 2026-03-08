// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DB persistence functions for COC and Evidence Collection records.
 *
 * Uses direct invoke() calls (awaitable) instead of fire-and-forget dbSync,
 * because forensic chain-of-custody data must confirm the save succeeded
 * before callers proceed to status transitions.
 */

import { invoke } from "@tauri-apps/api/core";
import { logger } from "../../../utils/logger";
import type { COCItem, EvidenceCollectionData } from "../types";
import type {
  DbCocItem,
  DbCocTransfer,
  DbEvidenceCollection,
  DbCollectedItem,
} from "../../../types/projectDb";
import {
  cocItemToDb,
  cocTransferToDb,
  dbToCocItem,
  dbToCocTransfer,
  evidenceCollectionToDb,
  dbToEvidenceCollectionData,
} from "./cocConverters";

const log = logger.scope("CocPersistence");

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
