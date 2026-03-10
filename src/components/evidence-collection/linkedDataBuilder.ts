// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Builds a LinkedDataNode[] tree from DB relationships between
 * collected items, COC records, and evidence files.
 * Rendered in the right panel via LinkedDataPanel.
 *
 * Enriches nodes with metadata from evidence files, hashes, and container info
 * so the tree shows container type badges, hash status, device IDs, and more.
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  DbCocItem,
  DbCollectedItem,
  DbEvidenceFile,
  DbProjectHash,
  DbProjectVerification,
  DbEvidenceCollection,
} from "../../types/projectDb";
import type { LinkedDataNode, NodeMetadata } from "../LinkedDataTree";
import { getBasename } from "../../utils/pathUtils";
import { logger } from "../../utils/logger";

const log = logger.scope("linkedDataBuilder");

// =============================================================================
// Helpers — fetch supporting data from DB
// =============================================================================

async function fetchEvidenceFiles(): Promise<Map<string, DbEvidenceFile>> {
  try {
    const files = await invoke<DbEvidenceFile[]>("project_db_get_evidence_files");
    const map = new Map<string, DbEvidenceFile>();
    for (const f of files) {
      map.set(f.id, f);
      // Also index by path for linking via file path
      map.set(f.path, f);
    }
    return map;
  } catch {
    return new Map();
  }
}

async function fetchHashesForFile(fileId: string): Promise<{
  hashStatus: "verified" | "mismatch" | "computed" | "none";
  hashAlgorithm?: string;
  hashValue?: string;
}> {
  try {
    const hashes = await invoke<DbProjectHash[]>(
      "project_db_get_hashes_for_file",
      { fileId },
    );
    if (hashes.length === 0) return { hashStatus: "none" };

    // Check for verification records
    for (const hash of hashes) {
      try {
        const verifications = await invoke<DbProjectVerification[]>(
          "project_db_get_verifications_for_hash",
          { hashId: hash.id },
        );
        if (verifications.length > 0) {
          const latest = verifications[verifications.length - 1];
          return {
            hashStatus: latest.result === "match" ? "verified" : "mismatch",
            hashAlgorithm: hash.algorithm,
            hashValue: hash.hashValue,
          };
        }
      } catch {
        // No verifications table or query failed
      }
    }

    // Hashes exist but no verification
    const best = hashes[0];
    return {
      hashStatus: "computed",
      hashAlgorithm: best.algorithm,
      hashValue: best.hashValue,
    };
  } catch {
    return { hashStatus: "none" };
  }
}

async function fetchCollection(collectionId: string): Promise<DbEvidenceCollection | null> {
  try {
    const collections = await invoke<DbEvidenceCollection[]>(
      "project_db_get_evidence_collections",
      { caseNumber: null },
    );
    return collections.find((c) => c.id === collectionId) ?? null;
  } catch {
    return null;
  }
}

// =============================================================================
// Build evidence file metadata
// =============================================================================

function buildEvidenceFileMetadata(
  ef: DbEvidenceFile,
  hashInfo: { hashStatus: "verified" | "mismatch" | "computed" | "none"; hashAlgorithm?: string; hashValue?: string },
): NodeMetadata {
  return {
    containerType: ef.containerType,
    totalSize: ef.totalSize,
    segmentCount: ef.segmentCount,
    discoveredAt: ef.discoveredAt,
    hashStatus: hashInfo.hashStatus,
    hashAlgorithm: hashInfo.hashAlgorithm,
    hashValue: hashInfo.hashValue,
  };
}

// =============================================================================
// Build collected item metadata
// =============================================================================

function buildCollectedItemMetadata(item: DbCollectedItem): NodeMetadata {
  return {
    deviceType: item.deviceType,
    brand: item.brand,
    make: item.make,
    model: item.model,
    serialNumber: item.serialNumber,
    imei: item.imei,
    imageFormat: item.imageFormat,
    acquisitionMethod: item.acquisitionMethod,
    acquisitionDate: item.itemCollectionDatetime,
    condition: item.condition,
    packaging: item.packaging,
    foundLocation: item.foundLocation,
    itemType: item.itemType,
    notes: item.notes,
  };
}

// =============================================================================
// Build COC item metadata
// =============================================================================

function buildCocMetadata(coc: DbCocItem): NodeMetadata {
  return {
    cocStatus: coc.status,
    submittedBy: coc.submittedBy,
    receivedBy: coc.receivedBy,
    collectionMethod: coc.collectionMethod,
    storageLocation: coc.storageLocation,
    acquisitionDate: coc.acquisitionDate,
    serialNumber: coc.serialNumber,
    make: coc.make,
    model: coc.model,
  };
}

// =============================================================================
// Main builder
// =============================================================================

/**
 * Builds the linked data tree for an evidence collection.
 * Enriches nodes with metadata from evidence files, hashes, and item details.
 *
 * @param collectionId - The collection UUID
 * @param caseNumber - Optional case number for COC item lookup
 * @returns Array of root LinkedDataNode items
 */
export async function buildLinkedDataTree(
  collectionId: string,
  caseNumber?: string,
): Promise<LinkedDataNode[]> {
  try {
    // Parallel fetch: collected items, COC items, evidence files, collection record
    const [collectedItems, cocItems, evidenceFileMap, collection] = await Promise.all([
      invoke<DbCollectedItem[]>("project_db_get_collected_items", { collectionId }).catch(() => [] as DbCollectedItem[]),
      invoke<DbCocItem[]>("project_db_get_coc_items", { caseNumber: caseNumber || null }).catch(() => [] as DbCocItem[]),
      fetchEvidenceFiles(),
      fetchCollection(collectionId),
    ]);

    // Build a map of COC items by ID for quick lookup
    const cocMap = new Map<string, DbCocItem>();
    for (const coc of cocItems) {
      cocMap.set(coc.id, coc);
    }

    // Fetch hash status for all referenced evidence files (parallel)
    const evidenceFileIds = new Set<string>();
    for (const item of collectedItems) {
      if (item.evidenceFileId) evidenceFileIds.add(item.evidenceFileId);
    }
    for (const coc of cocItems) {
      if (coc.evidenceFileId) evidenceFileIds.add(coc.evidenceFileId);
    }

    const hashMap = new Map<string, Awaited<ReturnType<typeof fetchHashesForFile>>>();
    const hashPromises = Array.from(evidenceFileIds).map(async (fileId) => {
      // fileId might be a path — resolve to the evidence file record's id
      const ef = evidenceFileMap.get(fileId);
      const dbId = ef?.id ?? fileId;
      const hashInfo = await fetchHashesForFile(dbId);
      hashMap.set(fileId, hashInfo);
    });
    await Promise.all(hashPromises);

    // Build tree nodes from collected items
    const itemNodes: LinkedDataNode[] = collectedItems.map((item) => {
      const children: LinkedDataNode[] = [];

      // If linked to a COC item, add as child
      if (item.cocItemId && cocMap.has(item.cocItemId)) {
        const coc = cocMap.get(item.cocItemId)!;
        const cocChildren: LinkedDataNode[] = [];

        // If COC is linked to an evidence file, show that too
        if (coc.evidenceFileId) {
          const ef = evidenceFileMap.get(coc.evidenceFileId);
          const hashInfo = hashMap.get(coc.evidenceFileId) ?? { hashStatus: "none" as const };
          cocChildren.push({
            id: `ef-${coc.evidenceFileId}`,
            label: getBasename(coc.evidenceFileId) || "Evidence File",
            type: "evidence-file",
            linkedId: coc.evidenceFileId,
            metadata: ef ? buildEvidenceFileMetadata(ef, hashInfo) : undefined,
          });
        }

        children.push({
          id: `coc-${coc.id}`,
          label: `COC ${coc.cocNumber}`,
          sublabel: coc.submittedBy ? `${coc.submittedBy} → ${coc.receivedBy || "?"}` : undefined,
          type: "coc",
          linkedId: coc.id,
          children: cocChildren.length > 0 ? cocChildren : undefined,
          metadata: buildCocMetadata(coc),
        });
      }

      // If collected item is directly linked to an evidence file
      if (item.evidenceFileId) {
        const ef = evidenceFileMap.get(item.evidenceFileId);
        const hashInfo = hashMap.get(item.evidenceFileId) ?? { hashStatus: "none" as const };
        children.push({
          id: `ef-${item.evidenceFileId}`,
          label: ef?.filename || getBasename(item.evidenceFileId) || "Evidence File",
          type: "evidence-file",
          linkedId: item.evidenceFileId,
          metadata: ef ? buildEvidenceFileMetadata(ef, hashInfo) : undefined,
        });
      }

      // Build sublabel from meaningful item data
      const sublabelParts: string[] = [];
      if (item.make || item.brand) sublabelParts.push(item.make || item.brand || "");
      if (item.model) sublabelParts.push(item.model);
      if (!sublabelParts.length && item.condition) sublabelParts.push(item.condition);

      return {
        id: `ci-${item.id}`,
        label: item.itemNumber
          ? `${item.itemNumber}${item.description ? ` — ${item.description}` : ""}`
          : item.description || "Item",
        sublabel: sublabelParts.join(" ") || undefined,
        type: "collected-item" as const,
        children: children.length > 0 ? children : undefined,
        metadata: buildCollectedItemMetadata(item),
      };
    });

    // Show unlinked COC items that aren't referenced by any collected item
    const linkedCocIds = new Set(
      collectedItems
        .filter((i) => i.cocItemId)
        .map((i) => i.cocItemId!),
    );
    const unlinkedCoc = cocItems.filter((c) => !linkedCocIds.has(c.id));

    const unlinkedCocNodes: LinkedDataNode[] = unlinkedCoc.map((coc) => {
      const cocChildren: LinkedDataNode[] = [];
      if (coc.evidenceFileId) {
        const ef = evidenceFileMap.get(coc.evidenceFileId);
        const hashInfo = hashMap.get(coc.evidenceFileId) ?? { hashStatus: "none" as const };
        cocChildren.push({
          id: `ef-${coc.evidenceFileId}`,
          label: ef?.filename || getBasename(coc.evidenceFileId) || "Evidence File",
          type: "evidence-file" as const,
          linkedId: coc.evidenceFileId,
          metadata: ef ? buildEvidenceFileMetadata(ef, hashInfo) : undefined,
        });
      }

      return {
        id: `coc-unlinked-${coc.id}`,
        label: `COC ${coc.cocNumber}`,
        sublabel: coc.submittedBy ? `${coc.submittedBy} → ${coc.receivedBy || "?"}` : `${coc.status} (unlinked)`,
        type: "coc" as const,
        linkedId: coc.id,
        children: cocChildren.length > 0 ? cocChildren : undefined,
        metadata: buildCocMetadata(coc),
      };
    });

    // Build collection metadata
    const collectionMeta: NodeMetadata = {
      collectionDate: collection?.collectionDate,
      collectionLocation: collection?.collectionLocation,
      collectingOfficer: collection?.collectingOfficer,
      authorization: collection?.authorization,
      status: collection?.status,
    };

    // Root node
    const rootNodes: LinkedDataNode[] = [
      {
        id: `collection-${collectionId}`,
        label: "Evidence Collection",
        sublabel: `${itemNodes.length} item${itemNodes.length !== 1 ? "s" : ""}`,
        type: "collection",
        children: itemNodes.length > 0 ? itemNodes : undefined,
        metadata: collectionMeta,
      },
    ];

    if (unlinkedCocNodes.length > 0) {
      rootNodes.push({
        id: "unlinked-coc",
        label: "Unlinked COC Records",
        sublabel: `${unlinkedCocNodes.length}`,
        type: "coc",
        children: unlinkedCocNodes,
      });
    }

    return rootNodes;
  } catch (e) {
    log.warn("Failed to build linked data tree:", e);
    return [];
  }
}
