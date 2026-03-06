// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Builds a LinkedDataNode[] tree from DB relationships between
 * collected items, COC records, and evidence files.
 * Rendered in the right panel via LinkedDataPanel.
 */

import { invoke } from "@tauri-apps/api/core";
import type { DbCocItem, DbCollectedItem } from "../../types/projectDb";
import type { LinkedDataNode } from "../LinkedDataTree";
import { getBasename } from "../../utils/pathUtils";
import { logger } from "../../utils/logger";

const log = logger.scope("linkedDataBuilder");

/**
 * Builds the linked data tree for an evidence collection.
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
    // Load collected items for this collection
    let collectedItems: DbCollectedItem[] = [];
    try {
      collectedItems = await invoke<DbCollectedItem[]>(
        "project_db_get_collected_items",
        { collectionId },
      );
    } catch {
      /* no items yet */
    }

    // Load COC items for this case
    let cocItems: DbCocItem[] = [];
    try {
      cocItems = await invoke<DbCocItem[]>(
        "project_db_get_coc_items",
        { caseNumber: caseNumber || null },
      );
    } catch {
      /* no COC items */
    }

    // Build a map of COC items by ID for quick lookup
    const cocMap = new Map<string, DbCocItem>();
    for (const coc of cocItems) {
      cocMap.set(coc.id, coc);
    }

    // Build tree nodes from collected items
    const itemNodes: LinkedDataNode[] = collectedItems.map((item) => {
      const children: LinkedDataNode[] = [];

      // If linked to a COC item, add as child
      if (item.cocItemId && cocMap.has(item.cocItemId)) {
        const coc = cocMap.get(item.cocItemId)!;
        const cocChildren: LinkedDataNode[] = [];

        // If COC is linked to an evidence file, show that too
        if (coc.evidenceFileId) {
          cocChildren.push({
            id: `ef-${coc.evidenceFileId}`,
            label: getBasename(coc.evidenceFileId) || "Evidence File",
            type: "evidence-file",
            linkedId: coc.evidenceFileId,
          });
        }

        children.push({
          id: `coc-${coc.id}`,
          label: `COC ${coc.cocNumber}`,
          sublabel: coc.status,
          type: "coc",
          linkedId: coc.id,
          children: cocChildren.length > 0 ? cocChildren : undefined,
        });
      }

      // If collected item is directly linked to an evidence file
      if (item.evidenceFileId) {
        children.push({
          id: `ef-${item.evidenceFileId}`,
          label: getBasename(item.evidenceFileId) || "Evidence File",
          type: "evidence-file",
          linkedId: item.evidenceFileId,
        });
      }

      return {
        id: `ci-${item.id}`,
        label: item.itemNumber || item.description || "Item",
        sublabel: item.condition,
        type: "collected-item" as const,
        children: children.length > 0 ? children : undefined,
      };
    });

    // Show unlinked COC items that aren't referenced by any collected item
    const linkedCocIds = new Set(
      collectedItems
        .filter((i) => i.cocItemId)
        .map((i) => i.cocItemId!),
    );
    const unlinkedCoc = cocItems.filter((c) => !linkedCocIds.has(c.id));

    const unlinkedCocNodes: LinkedDataNode[] = unlinkedCoc.map((coc) => ({
      id: `coc-unlinked-${coc.id}`,
      label: `COC ${coc.cocNumber}`,
      sublabel: `${coc.status} (unlinked)`,
      type: "coc" as const,
      linkedId: coc.id,
      children: coc.evidenceFileId
        ? [
            {
              id: `ef-${coc.evidenceFileId}`,
              label: getBasename(coc.evidenceFileId) || "Evidence File",
              type: "evidence-file" as const,
              linkedId: coc.evidenceFileId,
            },
          ]
        : undefined,
    }));

    // Root node
    const rootNodes: LinkedDataNode[] = [
      {
        id: `collection-${collectionId}`,
        label: "Evidence Collection",
        sublabel: `${itemNodes.length} item${itemNodes.length !== 1 ? "s" : ""}`,
        type: "collection",
        children: itemNodes.length > 0 ? itemNodes : undefined,
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
