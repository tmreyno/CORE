// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceCollectionPanel — Non-modal, tab-based evidence collection form.
 *
 * Renders inside a CenterPane tab. Uses the `evidence_collection` JSON schema
 * template via `useFormTemplate` / `SchemaFormRenderer`. Linked data tree
 * is emitted to the right panel via `onLinkedNodesChange`.
 */

import { createSignal, createEffect, on, onMount, Show, For, Component } from "solid-js";
import {
  HiOutlineArchiveBoxArrowDown,
  HiOutlineLockClosed,
  HiOutlineCheckBadge,
  HiOutlinePencil,
  HiOutlineBolt,
  HiOutlineArchiveBox,
  HiOutlineArrowPath,
} from "../icons";
import { useFormTemplate } from "../../templates/useFormTemplate";
import { useFormPersistence } from "../../templates/useFormPersistence";
import { SchemaFormRenderer } from "../../templates/SchemaFormRenderer";
import type { FormData, SectionSchema } from "../../templates/types";
import type { LinkedDataNode } from "../LinkedDataTree";
import {
  persistEvidenceCollectionToDb,
  loadEvidenceCollectionFromDb,
  loadEvidenceCollectionById,
  updateEvidenceCollectionStatus,
} from "../report/wizard/cocDbSync";
import { generateEvidenceItemNumber } from "../report/wizard/utils/reportNumbering";

import { logger } from "../../utils/logger";

import type { EvidenceCollectionPanelProps, CollectionStatus } from "./types";
import { generateId, evidenceToFormData, formDataToEvidence } from "./formDataConversion";
import { buildLinkedDataTree } from "./linkedDataBuilder";
import {
  extractHeaderFieldsFromEvidence,
  buildCollectedItemsFromEvidence,
  getAutoFillSummaries,
} from "./evidenceAutoFill";
import {
  matchEvidenceToCollectedItems,
  type MatchingResult,
} from "./evidenceMatching";
import { EvidenceConflictResolver } from "./EvidenceConflictResolver";
import { dbSync } from "../../hooks/project/useProjectDbSync";
import type { DbCollectedItem, DbEvidenceDataAlternative } from "../../types/projectDb";
import { invoke } from "@tauri-apps/api/core";

const log = logger.scope("EvidenceCollectionPanel");

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

  // Build auto-fill context from props (examiner + project case info)
  const autoFillContext = () => {
    const ctx: Record<string, Record<string, import("../../templates/types").FormValue>> = {};
    if (props.examinerName) {
      ctx.examiner = { name: props.examinerName };
    }
    if (props.caseNumber || props.projectName) {
      ctx.project = {
        ...(props.caseNumber ? { case_number: props.caseNumber } : {}),
        ...(props.projectName ? { name: props.projectName } : {}),
      };
    }
    return Object.keys(ctx).length > 0 ? ctx : undefined;
  };

  // Schema-driven form — fully self-contained
  const form = useFormTemplate({
    templateId: "evidence_collection",
    autoFillContext: autoFillContext(),
  });

  // Auto-persist via form submission table (debounced) — only when not read-only
  useFormPersistence({
    templateId: "evidence_collection",
    templateVersion: "1.1.0",
    caseNumber: () => props.caseNumber,
    data: () => readOnly() ? ({} as FormData) : form.data(),
  });

  // Rebuild linked data from DB
  const refreshLinkedData = async () => {
    const nodes = await buildLinkedDataTree(collectionId(), props.caseNumber);
    setLinkedNodes(nodes);
  };

  // Evidence auto-fill state
  const [showAutoFillPreview, setShowAutoFillPreview] = createSignal(false);

  // Conflict resolution state
  const [matchingResult, setMatchingResult] = createSignal<MatchingResult | null>(null);
  const [showConflictResolver, setShowConflictResolver] = createSignal(false);

  const hasEvidence = () => (props.discoveredFiles?.length ?? 0) > 0;
  const hasExistingItems = () => form.getRepeatableItems("collected_items").length > 0;

  const autoFillSummaries = () => {
    if (!props.discoveredFiles || !props.fileInfoMap) return [];
    return getAutoFillSummaries(props.discoveredFiles, props.fileInfoMap);
  };

  /** Auto-fill header fields + replace collected items with evidence-derived data */
  const handleAutoFillFromEvidence = () => {
    if (readOnly() || !props.discoveredFiles || !props.fileInfoMap) return;

    // 1. Auto-fill header fields (collecting officer, collection date)
    const headerResult = extractHeaderFieldsFromEvidence(props.discoveredFiles, props.fileInfoMap);
    for (const [key, value] of Object.entries(headerResult.patch)) {
      const existing = form.getValue(key) as string;
      if (!existing && value) {
        form.setValue(key, value as string);
      }
    }

    // 2. Build collected items from evidence files
    const items = buildCollectedItemsFromEvidence(
      props.discoveredFiles,
      props.fileInfoMap,
      props.caseNumber,
    );

    if (items.length > 0) {
      // Replace collected_items in form data
      const currentData = { ...form.data() };
      currentData.collected_items = items;
      form.setData(currentData);
      log.info(`Auto-filled ${items.length} collected items from ${props.discoveredFiles.length} evidence files`);
    }

    setShowAutoFillPreview(false);
  };

  /** Run the matching engine to detect conflicts between form data and containers */
  const handleReconcile = async () => {
    if (readOnly() || !props.discoveredFiles || !props.fileInfoMap) return;

    // Build DbCollectedItem[] from current form data for matching
    const formItems = form.getRepeatableItems("collected_items") as FormData[];
    const collectedItems: DbCollectedItem[] = formItems.map((item) => ({
      id: (item.id as string) || generateId(),
      collectionId: collectionId(),
      itemNumber: (item.item_number as string) || "",
      description: (item.description as string) || "",
      foundLocation: (item.found_location as string) || "",
      itemType: (item.item_type as string) || "",
      make: (item.make as string) || undefined,
      model: (item.model as string) || undefined,
      serialNumber: (item.serial_number as string) || undefined,
      condition: (item.condition as string) || "",
      packaging: (item.packaging as string) || "",
      notes: (item.notes as string) || undefined,
      evidenceFileId: (item.evidence_file_id as string) || undefined,
      cocItemId: (item.coc_item_id as string) || undefined,
      itemCollectionDatetime: (item.item_collection_datetime as string) || undefined,
      itemSystemDatetime: (item.item_system_datetime as string) || undefined,
      itemCollectingOfficer: (item.item_collecting_officer as string) || undefined,
      itemAuthorization: (item.item_authorization as string) || undefined,
      deviceType: (item.device_type as string) || undefined,
      deviceTypeOther: (item.device_type_other as string) || undefined,
      storageInterface: (item.storage_interface as string) || undefined,
      storageInterfaceOther: (item.storage_interface_other as string) || undefined,
      brand: (item.brand as string) || undefined,
      color: (item.color as string) || undefined,
      imei: (item.imei as string) || undefined,
      otherIdentifiers: (item.other_identifiers as string) || undefined,
      building: (item.building as string) || undefined,
      room: (item.room as string) || undefined,
      locationOther: (item.location_other as string) || undefined,
      imageFormat: (item.image_format as string) || undefined,
      imageFormatOther: (item.image_format_other as string) || undefined,
      acquisitionMethod: (item.acquisition_method as string) || undefined,
      acquisitionMethodOther: (item.acquisition_method_other as string) || undefined,
      storageNotes: (item.storage_notes as string) || undefined,
    }));

    if (collectedItems.length === 0) {
      log.info("No collected items to reconcile — use 'From Evidence' to populate first");
      return;
    }

    const result = matchEvidenceToCollectedItems(
      collectedItems,
      props.discoveredFiles,
      props.fileInfoMap,
      props.caseNumber,
    );

    setMatchingResult(result);

    if (result.totalConflicts > 0 || result.totalEnrichments > 0) {
      setShowConflictResolver(true);
    } else {
      log.info(`Reconciliation complete: ${result.matched.length} matched, no conflicts`);
    }
  };

  /** Apply conflict resolutions — update form items + save alternatives to DB */
  const handleApplyResolutions = async (
    updatedItems: DbCollectedItem[],
    alternatives: DbEvidenceDataAlternative[],
  ) => {
    // 1. Update form data with resolved values
    const currentData = { ...form.data() };
    const formItems = (currentData.collected_items as FormData[]) || [];

    for (const updated of updatedItems) {
      const idx = formItems.findIndex((fi) => (fi.id as string) === updated.id);
      if (idx >= 0) {
        // Map DbCollectedItem back to form field names (snake_case)
        formItems[idx] = {
          ...formItems[idx],
          description: updated.description,
          serial_number: updated.serialNumber || "",
          model: updated.model || "",
          brand: updated.brand || "",
          make: updated.make || "",
          imei: updated.imei || "",
          item_number: updated.itemNumber,
          device_type: updated.deviceType || "",
          image_format: updated.imageFormat || "",
          acquisition_method: updated.acquisitionMethod || "",
          item_collection_datetime: updated.itemCollectionDatetime || "",
          item_system_datetime: updated.itemSystemDatetime || "",
          item_collecting_officer: updated.itemCollectingOfficer || "",
          notes: updated.notes || "",
          other_identifiers: updated.otherIdentifiers || "",
          storage_notes: updated.storageNotes || "",
          building: updated.building || "",
          room: updated.room || "",
          storage_interface: updated.storageInterface || "",
          color: updated.color || "",
          evidence_file_id: updated.evidenceFileId || "",
        };
      }
    }

    currentData.collected_items = formItems;
    form.setData(currentData);

    // 2. Save alternatives to DB (evidence data alternatives table)
    for (const alt of alternatives) {
      dbSync.upsertEvidenceDataAlternative(alt);
    }

    // 3. Log activity for audit trail
    if (updatedItems.length > 0 || alternatives.length > 0) {
      const conflictCount = alternatives.length;
      const enrichCount = updatedItems.reduce((sum, item) => {
        const match = matchingResult()?.matched.find(
          (m) => m.collectedItem.id === item.id,
        );
        return sum + (match?.containerOnlyFields.length ?? 0);
      }, 0);
      const description = [
        `Reconciled ${updatedItems.length} evidence item${updatedItems.length !== 1 ? "s" : ""}`,
        conflictCount > 0 ? `${conflictCount} conflict${conflictCount !== 1 ? "s" : ""} resolved` : "",
        enrichCount > 0 ? `${enrichCount} field${enrichCount !== 1 ? "s" : ""} enriched from containers` : "",
        `${alternatives.length} alternative${alternatives.length !== 1 ? "s" : ""} archived`,
      ]
        .filter(Boolean)
        .join(", ");

      dbSync.insertActivity({
        id: `act-reconcile-${Date.now()}`,
        timestamp: new Date().toISOString(),
        user: (form.data().lead_examiner as string) || "unknown",
        category: "evidence_collection",
        action: "evidence_reconcile",
        description,
        filePath: null,
        details: JSON.stringify({
          collectionId: collectionId(),
          itemsReconciled: updatedItems.map((i) => i.id),
          conflictsResolved: conflictCount,
          enrichments: enrichCount,
          alternativesArchived: alternatives.length,
        }),
      });

      log.info(description);
    }

    setShowConflictResolver(false);
    setMatchingResult(null);
  };

  // Load existing data from DB on mount
  onMount(async () => {
    try {
      let result: { data: import("../report/types").EvidenceCollectionData; collectionId: string; status?: string } | null = null;

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
    await refreshLinkedData();
  });

  // Rebuild tree when collectionId changes
  createEffect(
    on(
      () => collectionId(),
      async () => {
        if (loaded()) {
          await refreshLinkedData();
        }
      },
    ),
  );

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

  // ── Save & Status Handlers ──────────────────────────────────────────────

  const handleSave = async () => {
    if (readOnly()) return;
    setSaving(true);
    try {
      const data = formDataToEvidence(form.data());
      await persistEvidenceCollectionToDb(data, collectionId(), props.caseNumber, status());
      log.info("Evidence collection saved to .ffxdb");
      await refreshLinkedData();
    } catch (e) {
      log.error("Failed to save evidence collection:", e);
    } finally {
      setSaving(false);
    }
  };

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

  // ── Status Badge ────────────────────────────────────────────────────────

  const statusBadge = () => {
    switch (status()) {
      case "complete":
        return <span class="badge badge-success">Complete</span>;
      case "locked":
        return <span class="badge badge-warning">🔒 Locked</span>;
      default:
        return (
          <span
            class="badge"
            style={{ background: "var(--color-bg-hover)", color: "var(--color-txt-muted)" }}
          >
            Draft
          </span>
        );
    }
  };

  // ── JSX ─────────────────────────────────────────────────────────────────

  return (
    <div class="flex flex-col h-full overflow-hidden bg-bg">
      {/* Header toolbar */}
      <div class="flex items-center justify-between px-4 py-2 border-b border-border bg-bg-secondary shrink-0">
        <div class="flex items-center gap-2">
          <HiOutlineArchiveBoxArrowDown class="w-4 h-4 text-accent" />
          <div>
            <h2 class="text-xs font-semibold flex items-center gap-2">
              Evidence Collection
              {statusBadge()}
            </h2>
            <p class="text-[11px] text-txt-muted">
              {readOnly() ? "Review mode" : "On-site acquisition & collection form"}
              <Show when={props.projectName}>
                <span> — {props.projectName}</span>
              </Show>
            </p>
          </div>
        </div>

        {/* Actions */}
        <div class="flex items-center gap-2">
          <Show when={hasEvidence() && !readOnly() && status() === "draft"}>
            <button
              class="btn-sm"
              onClick={() => setShowAutoFillPreview((v) => !v)}
              title="Populate collected items from loaded evidence"
            >
              <HiOutlineBolt class="w-3.5 h-3.5" />
              From Evidence
            </button>
          </Show>
          <Show when={hasEvidence() && hasExistingItems() && !readOnly() && status() === "draft"}>
            <button
              class="btn-sm"
              onClick={handleReconcile}
              title="Compare form data with container metadata and resolve conflicts"
            >
              <HiOutlineArrowPath class="w-3.5 h-3.5" />
              Reconcile
            </button>
          </Show>
          <Show when={!readOnly() && status() === "draft"}>
            <button
              class="btn-sm"
              onClick={handleMarkComplete}
              disabled={saving()}
              title="Mark as complete"
            >
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

          <Show when={!readOnly()}>
            <button class="btn btn-primary" onClick={handleSave} disabled={saving()}>
              {saving() ? "Saving…" : "Save"}
            </button>
          </Show>
        </div>
      </div>

      {/* Auto-fill preview panel */}
      <Show when={showAutoFillPreview()}>
        <div class="border-b border-border bg-bg-panel px-4 py-3 shrink-0">
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <HiOutlineArchiveBox class="w-4 h-4 text-accent" />
              <span class="text-xs font-semibold text-txt">
                Create items from {autoFillSummaries().length} evidence file{autoFillSummaries().length !== 1 ? "s" : ""}
              </span>
              <span class="text-[10px] text-txt-muted">
                Device IDs, serial numbers, and forensic image details are captured from container headers
              </span>
            </div>
            <div class="flex items-center gap-2">
              <button
                class="btn btn-primary"
                onClick={handleAutoFillFromEvidence}
                disabled={autoFillSummaries().length === 0}
              >
                <HiOutlineBolt class="w-3.5 h-3.5" />
                {hasExistingItems() ? "Replace Items" : "Populate"}
              </button>
              <button class="btn btn-secondary" onClick={() => setShowAutoFillPreview(false)}>
                Cancel
              </button>
            </div>
          </div>

          <Show when={hasExistingItems()}>
            <p class="text-[11px] text-warning mb-2">
              This will replace existing collected items with items created from evidence files.
            </p>
          </Show>

          <div class="max-h-40 overflow-y-auto space-y-1">
            <For each={autoFillSummaries()}>
              {(summary) => (
                <div class="flex items-center gap-2 text-[11px] py-0.5 px-2 rounded bg-bg-hover">
                  <HiOutlineArchiveBox class="w-3 h-3 text-txt-muted shrink-0" />
                  <span class="text-txt font-medium truncate" title={summary.filename}>
                    {summary.filename}
                  </span>
                  <span class="text-txt-muted shrink-0">
                    {summary.containerType.toUpperCase()}
                  </span>
                  <Show when={summary.autoFillFieldCount > 0}>
                    <span class="text-accent shrink-0">
                      {summary.autoFillFieldCount} metadata
                    </span>
                  </Show>
                  <Show when={summary.autoFillSummary.length > 0}>
                    <span class="text-txt-muted truncate" title={summary.autoFillSummary.join(", ")}>
                      ({summary.autoFillSummary.join(", ")})
                    </span>
                  </Show>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>

      {/* Form body */}
      <div class="flex-1 overflow-y-auto p-4">
        <Show
          when={loaded()}
          fallback={
            <div class="flex items-center justify-center py-8">
              <div class="animate-pulse-slow text-txt-muted text-sm">
                Loading…
              </div>
            </div>
          }
        >
          <SchemaFormRenderer form={enhancedForm} readOnly={readOnly()} />
        </Show>
      </div>

      {/* Conflict resolution modal */}
      <Show when={showConflictResolver() && matchingResult()}>
        <EvidenceConflictResolver
          matchingResult={matchingResult()!}
          examinerName={form.data().lead_examiner as string}
          onApply={handleApplyResolutions}
          onCancel={() => {
            setShowConflictResolver(false);
            setMatchingResult(null);
          }}
        />
      </Show>
    </div>
  );
};
