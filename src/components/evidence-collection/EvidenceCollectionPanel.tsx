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

import { createSignal, createEffect, on, onMount, Show, Component } from "solid-js";
import {
  HiOutlineArchiveBoxArrowDown,
  HiOutlineLockClosed,
  HiOutlineCheckBadge,
  HiOutlinePencil,
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

  // Rebuild linked data from DB
  const refreshLinkedData = async () => {
    const nodes = await buildLinkedDataTree(collectionId(), props.caseNumber);
    setLinkedNodes(nodes);
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

      {/* Form body */}
      <div class="flex-1 overflow-y-auto p-4">
        <Show
          when={loaded()}
          fallback={
            <div class="flex items-center justify-center py-12">
              <div class="animate-pulse-slow text-txt-muted">
                Loading evidence collection data…
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
