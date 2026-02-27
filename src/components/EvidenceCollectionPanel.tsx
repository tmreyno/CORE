// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceCollectionPanel — Non-modal, tab-based evidence collection form.
 *
 * Refactored from EvidenceCollectionModal to render inside a CenterPane tab
 * instead of a modal overlay. Includes a linked-data tree sidebar showing
 * relationships between collected items, COC records, and evidence files.
 *
 * Architecture:
 *  - Uses the `evidence_collection` JSON schema template via `useFormTemplate`
 *  - Renders via `SchemaFormRenderer` (same engine used for all forms)
 *  - State is fully self-contained (no WizardContext dependency)
 *  - Auto-saves to .ffxdb via `useFormPersistence` (debounced)
 *  - Linked data tree shows COC ↔ Evidence ↔ Collection relationships
 */

import { createSignal, createEffect, on, onMount, Show, Component } from "solid-js";
import {
  HiOutlineArchiveBoxArrowDown,
  HiOutlineLockClosed,
  HiOutlineCheckBadge,
  HiOutlinePencil,
} from "./icons";
import { useFormTemplate } from "../templates/useFormTemplate";
import { useFormPersistence } from "../templates/useFormPersistence";
import { getBasename } from "../utils/pathUtils";
import { SchemaFormRenderer } from "../templates/SchemaFormRenderer";
import type { FormData, SectionSchema } from "../templates/types";
import type { EvidenceCollectionData, CollectedItem } from "./report/types";
import type { DbCocItem, DbCollectedItem } from "../types/projectDb";
import type { LinkedDataNode } from "./LinkedDataTree";
import {
  persistEvidenceCollectionToDb,
  loadEvidenceCollectionFromDb,
  loadEvidenceCollectionById,
  updateEvidenceCollectionStatus,
} from "./report/wizard/cocDbSync";
import { generateEvidenceItemNumber } from "./report/wizard/utils/reportNumbering";
import { invoke } from "@tauri-apps/api/core";
import { logger } from "../utils/logger";

const log = logger.scope("EvidenceCollectionPanel");

type CollectionStatus = "draft" | "complete" | "locked";

export interface EvidenceCollectionPanelProps {
  caseNumber?: string;
  projectName?: string;
  examinerName?: string;
  /** Open specific collection by ID */
  collectionId?: string;
  /** Open in read-only/review mode */
  readOnly?: boolean;
  /** Called when user closes the tab */
  onClose?: () => void;
  /** Called when user wants to open a different collection */
  onOpenCollection?: (collectionId: string, readOnly: boolean) => void;
  /** Called when linked data nodes change (rendered in right panel) */
  onLinkedNodesChange?: (nodes: LinkedDataNode[]) => void;
}

// =============================================================================
// ID / Number Generators
// =============================================================================

function generateId(): string {
  return crypto.randomUUID();
}

// =============================================================================
// Data Conversion Helpers (same as modal)
// =============================================================================

function evidenceToFormData(d: EvidenceCollectionData): FormData {
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
    })),
  };
}

function formDataToEvidence(fd: FormData): EvidenceCollectionData {
  const witnesses = Array.isArray(fd.witnesses)
    ? (fd.witnesses as string[])
    : typeof fd.witnesses === "string"
      ? (fd.witnesses as string).split(",").map((s) => s.trim()).filter(Boolean)
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

// =============================================================================
// Main Panel Component
// =============================================================================

export const EvidenceCollectionPanel: Component<EvidenceCollectionPanelProps> = (props) => {
  const [collectionId, setCollectionId] = createSignal<string>(props.collectionId || generateId());
  const [saving, setSaving] = createSignal(false);
  const [loaded, setLoaded] = createSignal(false);
  const [status, setStatus] = createSignal<CollectionStatus>("draft");
  const [readOnly, setReadOnly] = createSignal(props.readOnly ?? false);

  // Linked data tree state — emitted to right panel via onLinkedNodesChange
  const [, setLinkedNodesInternal] = createSignal<LinkedDataNode[]>([]);
  const setLinkedNodes = (nodes: LinkedDataNode[]) => {
    setLinkedNodesInternal(nodes);
    props.onLinkedNodesChange?.(nodes);
  };

  // Schema-driven form — fully self-contained
  const form = useFormTemplate({
    templateId: "evidence_collection",
  });

  // Auto-persist via form submission table (debounced) — only when not read-only
  useFormPersistence({
    templateId: "evidence_collection",
    templateVersion: "1.0.0",
    caseNumber: () => props.caseNumber,
    data: () => readOnly() ? ({} as FormData) : form.data(),
  });

  // Load existing data from DB on mount
  onMount(async () => {
    try {
      let result: { data: EvidenceCollectionData; collectionId: string; status?: string } | null = null;

      if (props.collectionId) {
        result = await loadEvidenceCollectionById(props.collectionId);
      } else {
        result = await loadEvidenceCollectionFromDb(props.caseNumber);
      }

      if (result) {
        const fd = evidenceToFormData(result.data);
        form.setData(fd);
        setCollectionId(result.collectionId);
        if (result.status) {
          setStatus(result.status as CollectionStatus);
          if (result.status === "locked") {
            setReadOnly(true);
          }
        }
        log.info("Loaded evidence collection from .ffxdb");
      }
    } catch (e) {
      log.warn("Could not load evidence collection from DB:", e);
    }

    // Auto-seed collecting officer from examiner name if field is empty
    if (!readOnly()) {
      const officer = form.getValue("collecting_officer") as string;
      if (!officer && props.examinerName) {
        form.setValue("collecting_officer", props.examinerName);
      }
    }

    setLoaded(true);

    // Build linked data tree after loading
    await buildLinkedDataTree();
  });

  // Rebuild tree when collectionId changes
  createEffect(on(() => collectionId(), async () => {
    if (loaded()) {
      await buildLinkedDataTree();
    }
  }));

  // Build the linked data tree from DB relationships
  const buildLinkedDataTree = async () => {
    try {
      const cId = collectionId();
      const caseNum = props.caseNumber;

      // Load collected items for this collection
      let collectedItems: DbCollectedItem[] = [];
      try {
        collectedItems = await invoke<DbCollectedItem[]>(
          "project_db_get_collected_items",
          { collectionId: cId }
        );
      } catch { /* no items yet */ }

      // Load COC items for this case
      let cocItems: DbCocItem[] = [];
      try {
        cocItems = await invoke<DbCocItem[]>(
          "project_db_get_coc_items",
          { caseNumber: caseNum || null }
        );
      } catch { /* no COC items */ }

      // Build a map of COC items by ID for quick lookup
      const cocMap = new Map<string, DbCocItem>();
      for (const coc of cocItems) {
        cocMap.set(coc.id, coc);
      }

      // Build tree nodes
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

      // Also show unlinked COC items that aren't referenced by any collected item
      const linkedCocIds = new Set(
        collectedItems
          .filter((i) => i.cocItemId)
          .map((i) => i.cocItemId!)
      );
      const unlinkedCoc = cocItems.filter((c) => !linkedCocIds.has(c.id));

      const unlinkedCocNodes: LinkedDataNode[] = unlinkedCoc.map((coc) => ({
        id: `coc-unlinked-${coc.id}`,
        label: `COC ${coc.cocNumber}`,
        sublabel: `${coc.status} (unlinked)`,
        type: "coc" as const,
        linkedId: coc.id,
        children: coc.evidenceFileId
          ? [{
              id: `ef-${coc.evidenceFileId}`,
              label: getBasename(coc.evidenceFileId) || "Evidence File",
              type: "evidence-file" as const,
              linkedId: coc.evidenceFileId,
            }]
          : undefined,
      }));

      // Root node
      const rootNodes: LinkedDataNode[] = [{
        id: `collection-${cId}`,
        label: "Evidence Collection",
        sublabel: `${itemNodes.length} item${itemNodes.length !== 1 ? "s" : ""}`,
        type: "collection",
        children: itemNodes.length > 0 ? itemNodes : undefined,
      }];

      if (unlinkedCocNodes.length > 0) {
        rootNodes.push({
          id: "unlinked-coc",
          label: "Unlinked COC Records",
          sublabel: `${unlinkedCocNodes.length}`,
          type: "coc",
          children: unlinkedCocNodes,
        });
      }

      setLinkedNodes(rootNodes);
    } catch (e) {
      log.warn("Failed to build linked data tree:", e);
    }
  };

  // Override addRepeatableItem to inject auto-generated item numbers
  const originalAdd = form.addRepeatableItem;
  const customAddRepeatableItem = (section: SectionSchema) => {
    originalAdd(section);

    if (section.id === "collected_items") {
      const items = form.getRepeatableItems("collected_items");
      if (items.length > 0) {
        const lastItem = items[items.length - 1];
        form.setRepeatableItemValue(
          "collected_items",
          lastItem.id as string,
          "item_number",
          generateEvidenceItemNumber(props.caseNumber),
        );
      }
    }
  };

  const enhancedForm = { ...form, addRepeatableItem: customAddRepeatableItem };

  // Manual save — awaits the DB writes so callers can trust the data is persisted
  const handleSave = async () => {
    if (readOnly()) return;
    setSaving(true);
    try {
      const data = formDataToEvidence(form.data());
      await persistEvidenceCollectionToDb(data, collectionId(), props.caseNumber, status());
      log.info("Evidence collection saved to .ffxdb");
      // Rebuild tree after save to reflect new items
      await buildLinkedDataTree();
    } catch (e) {
      log.error("Failed to save evidence collection:", e);
    } finally {
      setSaving(false);
    }
  };

  // Status transitions — save first (awaited), then update status
  const handleMarkComplete = async () => {
    setSaving(true);
    try {
      const data = formDataToEvidence(form.data());
      await persistEvidenceCollectionToDb(data, collectionId(), props.caseNumber, status());
      const ok = await updateEvidenceCollectionStatus(collectionId(), "complete");
      if (ok) setStatus("complete");
    } catch (e) {
      log.error("Failed to mark complete:", e);
    } finally {
      setSaving(false);
    }
  };

  const handleLock = async () => {
    setSaving(true);
    try {
      const data = formDataToEvidence(form.data());
      await persistEvidenceCollectionToDb(data, collectionId(), props.caseNumber, status());
      const ok = await updateEvidenceCollectionStatus(collectionId(), "locked");
      if (ok) {
        setStatus("locked");
        setReadOnly(true);
      }
    } catch (e) {
      log.error("Failed to lock collection:", e);
    } finally {
      setSaving(false);
    }
  };

  const handleEnableEdit = () => {
    if (status() !== "locked") {
      setReadOnly(false);
    }
  };

  // Status badge helper
  const statusBadge = () => {
    switch (status()) {
      case "complete":
        return <span class="badge badge-success">Complete</span>;
      case "locked":
        return <span class="badge badge-warning">🔒 Locked</span>;
      default:
        return <span class="badge" style={{ background: "var(--color-bg-hover)", color: "var(--color-txt-muted)" }}>Draft</span>;
    }
  };

  return (
    <div class="flex flex-col h-full overflow-hidden bg-bg">
      {/* Header toolbar */}
      <div class="flex items-center justify-between px-4 py-2.5 border-b border-border bg-bg-secondary shrink-0">
        <div class="flex items-center gap-3">
          <div class="w-7 h-7 rounded-lg bg-gradient-to-br from-accent/20 to-accent/5 flex items-center justify-center">
            <HiOutlineArchiveBoxArrowDown class="w-4 h-4 text-accent" />
          </div>
          <div>
            <h2 class="text-sm font-semibold flex items-center gap-2">
              Evidence Collection
              {statusBadge()}
            </h2>
            <p class="text-xs text-txt-muted">
              {readOnly() ? "Review mode" : "On-site acquisition & collection form"}
              <Show when={props.projectName}>
                <span> — {props.projectName}</span>
              </Show>
            </p>
          </div>
        </div>

        {/* Actions */}
        <div class="flex items-center gap-2">
          {/* Status actions */}
          <Show when={!readOnly() && status() === "draft"}>
            <button class="btn-sm" onClick={handleMarkComplete} disabled={saving()} title="Mark as complete">
              <HiOutlineCheckBadge class="w-3.5 h-3.5" />
              Complete
            </button>
          </Show>
          <Show when={!readOnly() && (status() === "draft" || status() === "complete")}>
            <button class="btn-sm" onClick={handleLock} disabled={saving()} title="Lock collection">
              <HiOutlineLockClosed class="w-3.5 h-3.5" />
              Lock
            </button>
          </Show>
          <Show when={readOnly() && status() !== "locked"}>
            <button class="btn-sm" onClick={handleEnableEdit} title="Switch to edit mode">
              <HiOutlinePencil class="w-3.5 h-3.5" />
              Edit
            </button>
          </Show>

          {/* Save */}
          <Show when={!readOnly()}>
            <button class="btn btn-primary" onClick={handleSave} disabled={saving()}>
              {saving() ? "Saving…" : "Save"}
            </button>
          </Show>
        </div>
      </div>

      {/* Form body */}
      <div class="flex-1 overflow-y-auto p-4">
        <Show when={loaded()} fallback={
          <div class="flex items-center justify-center py-12">
            <div class="animate-pulse-slow text-txt-muted">Loading evidence collection data…</div>
          </div>
        }>
          <SchemaFormRenderer form={enhancedForm} readOnly={readOnly()} />
        </Show>
      </div>
    </div>
  );
};
