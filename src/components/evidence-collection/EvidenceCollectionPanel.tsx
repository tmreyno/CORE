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

  // Build examiner auto-fill context from props
  const examinerAutoFill = () => {
    const name = props.examinerName;
    if (name) {
      return { examiner: { name } as Record<string, import("../../templates/types").FormValue> };
    }
    return undefined;
  };

  // Schema-driven form — fully self-contained
  const form = useFormTemplate({
    templateId: "evidence_collection",
    autoFillContext: examinerAutoFill(),
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
    </div>
  );
};
