// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceCollectionModal — Standalone on-site evidence collection form.
 *
 * This is a **field/acquisition tool** — completely separate from the Report
 * Wizard. It lets examiners document evidence collection details (devices,
 * conditions, packaging, imaging info, etc.) during on-site acquisition,
 * even before evidence is added to the project.
 *
 * Architecture:
 *  - Uses the `evidence_collection` JSON schema template via `useFormTemplate`
 *  - Renders via `SchemaFormRenderer` (same engine used for all forms)
 *  - State is fully self-contained (no WizardContext dependency)
 *  - Auto-saves to .ffxdb via `useFormPersistence` (debounced)
 *  - Manual save also writes via the `cocDbSync` persistence layer for
 *    backward-compatible access to evidence collection DB records
 */

import { createSignal, onMount, Show } from "solid-js";
import { HiOutlineXMark, HiOutlineArchiveBoxArrowDown } from "./icons";
import { useFormTemplate } from "../templates/useFormTemplate";
import { useFormPersistence } from "../templates/useFormPersistence";
import { SchemaFormRenderer } from "../templates/SchemaFormRenderer";
import { generateEvidenceItemNumber } from "./report/wizard/utils/reportNumbering";
import {
  persistEvidenceCollectionToDb,
  loadEvidenceCollectionFromDb,
} from "./report/wizard/cocDbSync";
import { generateId } from "../types/project";
import type { EvidenceCollectionData, CollectedItem } from "./report/types";
import type { FormData } from "../templates/types";
import type { SectionSchema } from "../templates/types";
import { logger } from "../utils/logger";

const log = logger.scope("EvidenceCollectionModal");

export interface EvidenceCollectionModalProps {
  /** Case number for numbering and DB association */
  caseNumber?: string;
  /** Project name (shown in header) */
  projectName?: string;
  /** Examiner name — auto-seeds the "Collecting Officer" field */
  examinerName?: string;
  /** Close callback */
  onClose: () => void;
}

// =============================================================================
// Data Conversion Helpers
// =============================================================================

/** Convert typed EvidenceCollectionData → flat FormData for the schema form */
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

/** Convert flat FormData → typed EvidenceCollectionData for DB persistence */
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
// Component
// =============================================================================

export function EvidenceCollectionModal(props: EvidenceCollectionModalProps) {
  const [collectionId, setCollectionId] = createSignal<string>(generateId());
  const [saving, setSaving] = createSignal(false);
  const [loaded, setLoaded] = createSignal(false);

  // Schema-driven form — fully self-contained, no WizardContext
  const form = useFormTemplate({
    templateId: "evidence_collection",
  });

  // Auto-persist via form submission table (debounced)
  useFormPersistence({
    templateId: "evidence_collection",
    templateVersion: "1.0.0",
    caseNumber: () => props.caseNumber,
    data: form.data,
  });

  // Load existing data from DB on mount
  onMount(async () => {
    try {
      const result = await loadEvidenceCollectionFromDb(props.caseNumber);
      if (result) {
        const fd = evidenceToFormData(result.data);
        form.setData(fd);
        setCollectionId(result.collectionId);
        log.info("Loaded evidence collection from .ffxdb");
      }
    } catch (e) {
      log.warn("Could not load evidence collection from DB:", e);
    }

    // Auto-seed collecting officer from examiner name if field is empty
    const officer = form.getValue("collecting_officer") as string;
    if (!officer && props.examinerName) {
      form.setValue("collecting_officer", props.examinerName);
    }

    setLoaded(true);
  });

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

  // Manual save to legacy evidence collection DB tables
  const handleSave = async () => {
    setSaving(true);
    try {
      const data = formDataToEvidence(form.data());
      persistEvidenceCollectionToDb(data, collectionId(), props.caseNumber);
      log.info("Evidence collection saved to .ffxdb");
    } catch (e) {
      log.error("Failed to save evidence collection:", e);
    } finally {
      setSaving(false);
    }
  };

  const handleSaveAndClose = async () => {
    await handleSave();
    props.onClose();
  };

  return (
    <div class="modal-overlay" onClick={(e) => { if (e.target === e.currentTarget) props.onClose(); }}>
      <div class="modal-content w-[900px] max-h-[90vh] flex flex-col">
        {/* Header */}
        <div class="modal-header">
          <div class="flex items-center gap-3">
            <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-accent/20 to-accent/5 flex items-center justify-center">
              <HiOutlineArchiveBoxArrowDown class="w-5 h-5 text-accent" />
            </div>
            <div>
              <h2 class="text-lg font-semibold">Evidence Collection</h2>
              <p class="text-xs text-txt-muted">
                On-site acquisition &amp; collection form
                <Show when={props.projectName}>
                  <span> — {props.projectName}</span>
                </Show>
              </p>
            </div>
          </div>
          <button class="icon-btn-sm" onClick={props.onClose} title="Close">
            <HiOutlineXMark class="w-5 h-5" />
          </button>
        </div>

        {/* Body */}
        <div class="modal-body overflow-y-auto flex-1">
          <Show when={loaded()} fallback={
            <div class="flex items-center justify-center py-12">
              <div class="animate-pulse-slow text-txt-muted">Loading evidence collection data…</div>
            </div>
          }>
            <SchemaFormRenderer form={enhancedForm} />
          </Show>
        </div>

        {/* Footer */}
        <div class="modal-footer justify-end">
          <button class="btn btn-secondary" onClick={props.onClose}>
            Cancel
          </button>
          <button class="btn btn-secondary" onClick={handleSave} disabled={saving()}>
            {saving() ? "Saving…" : "Save"}
          </button>
          <button class="btn btn-primary" onClick={handleSaveAndClose} disabled={saving()}>
            {saving() ? "Saving…" : "Save & Close"}
          </button>
        </div>
      </div>
    </div>
  );
}
