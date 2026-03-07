// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * COCFormSection - Chain of Custody Form 7 data entry.
 * Generates one COCItem per evidence item with transfer tracking.
 *
 * Immutability model (schema v5):
 *  - Items start as "draft" — freely editable.
 *  - Locking makes an item immutable (requires initials).
 *  - Locked items show read-only fields with a 🔒 indicator.
 *  - Amendments on locked items require initials + reason (inline modal).
 *  - Voided (soft-deleted) items show strikethrough + reduced opacity.
 *  - All actions are logged to the COC audit trail.
 */

import { For, Show, createSignal } from "solid-js";
import {
  HiOutlinePlus,
  HiOutlineDocumentDuplicate,
} from "../../../../icons";
import { useWizard } from "../../WizardContext";
import type { COCItem, COCTransfer } from "../../../types";
import { generateCocNumber } from "../../utils/reportNumbering";
import { AmendmentModal, LockConfirmationModal, VoidConfirmationModal } from "./COCModals";
import type { AmendFieldInfo } from "./COCModals";
import { COCItemRow } from "./COCItemRow";

export function COCFormSection() {
  const ctx = useWizard();
  const [expandedItems, setExpandedItems] = createSignal<Set<string>>(new Set());

  // --- Amendment prompt state ---
  const [amendingField, setAmendingField] = createSignal<AmendFieldInfo | null>(null);
  const [amendInitials, setAmendInitials] = createSignal("");
  const [amendNewValue, setAmendNewValue] = createSignal("");
  const [amendReason, setAmendReason] = createSignal("");

  // --- Void prompt state ---
  const [voidingItemId, setVoidingItemId] = createSignal<string | null>(null);
  const [voidInitials, setVoidInitials] = createSignal("");
  const [voidReason, setVoidReason] = createSignal("");

  // --- Lock prompt state ---
  const [lockingItemId, setLockingItemId] = createSignal<string | null>(null);
  const [lockInitials, setLockInitials] = createSignal("");

  const toggleExpanded = (id: string) => {
    setExpandedItems((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const addCocItem = () => {
    const caseNum = ctx.caseInfo().case_number || undefined;
    const newItem: COCItem = {
      id: crypto.randomUUID(),
      coc_number: generateCocNumber(caseNum),
      evidence_id: "",
      case_number: ctx.caseInfo().case_number || "",
      description: "",
      item_type: "HardDrive",
      condition: "sealed",
      acquisition_date: "",
      entered_custody_date: "",
      submitted_by: ctx.examiner().name || "",
      received_by: "",
      storage_location: "",
      transfers: [],
      intake_hashes: [],
      notes: "",
      disposition: "in_custody",
      status: "draft",
    };
    ctx.setCocItems([...ctx.cocItems(), newItem]);
    setExpandedItems((prev) => new Set(prev).add(newItem.id));
  };

  /** Update a COCItem field. If the item is locked, opens the amendment modal. */
  const updateItem = (id: string, field: keyof COCItem, value: unknown) => {
    const item = ctx.cocItems().find((i) => i.id === id);
    if (!item) return;

    // Locked items require amendment workflow
    if (item.status === "locked") {
      setAmendingField({
        itemId: id,
        field,
        label: field.replace(/_/g, " "),
        oldValue: String((item as unknown as Record<string, unknown>)[field] ?? ""),
      });
      setAmendNewValue(String(value));
      setAmendInitials("");
      setAmendReason("");
      return;
    }

    // Draft items: apply directly
    ctx.setCocItems(
      ctx.cocItems().map((i) =>
        i.id === id ? { ...i, [field]: value } : i
      )
    );
  };

  /** Apply an amendment to a locked item after initials are provided */
  const applyAmendment = () => {
    const info = amendingField();
    if (!info || !amendInitials().trim()) return;
    ctx.setCocItems(
      ctx.cocItems().map((i) =>
        i.id === info.itemId ? { ...i, [info.field]: amendNewValue() } : i
      )
    );
    setAmendingField(null);
  };

  /** Lock a COC item (makes it immutable; future edits require amendment) */
  const confirmLock = () => {
    const id = lockingItemId();
    const initials = lockInitials().trim();
    if (!id || !initials) return;
    ctx.setCocItems(
      ctx.cocItems().map((i) =>
        i.id === id
          ? {
              ...i,
              status: "locked" as const,
              locked_at: new Date().toISOString(),
              locked_by: initials,
            }
          : i
      )
    );
    setLockingItemId(null);
    setLockInitials("");
  };

  /** Void (soft-delete) a COC item with audit trail */
  const confirmVoid = () => {
    const id = voidingItemId();
    const initials = voidInitials().trim();
    const reason = voidReason().trim();
    if (!id || !initials || !reason) return;
    ctx.setCocItems(
      ctx.cocItems().map((i) =>
        i.id === id ? { ...i, status: "voided" as const } : i
      )
    );
    setVoidingItemId(null);
    setVoidInitials("");
    setVoidReason("");
  };

  /** Remove (draft) or void (locked) a COC item */
  const removeItem = (id: string) => {
    const item = ctx.cocItems().find((i) => i.id === id);
    if (!item) return;
    if (item.status === "locked") {
      // Locked items must go through void workflow
      setVoidingItemId(id);
      return;
    }
    // Draft items can be removed directly
    ctx.setCocItems(ctx.cocItems().filter((i) => i.id !== id));
  };

  const addTransfer = (cocId: string) => {
    const transfer: COCTransfer = {
      id: crypto.randomUUID(),
      timestamp: new Date().toISOString().slice(0, 16),
      released_by: "",
      received_by: "",
      purpose: "examination",
    };
    ctx.setCocItems(
      ctx.cocItems().map((item) =>
        item.id === cocId
          ? { ...item, transfers: [...item.transfers, transfer] }
          : item
      )
    );
  };

  const updateTransfer = (
    cocId: string,
    transferId: string,
    field: keyof COCTransfer,
    value: string
  ) => {
    ctx.setCocItems(
      ctx.cocItems().map((item) =>
        item.id === cocId
          ? {
              ...item,
              transfers: item.transfers.map((t) =>
                t.id === transferId ? { ...t, [field]: value } : t
              ),
            }
          : item
      )
    );
  };

  const removeTransfer = (cocId: string, transferId: string) => {
    ctx.setCocItems(
      ctx.cocItems().map((item) =>
        item.id === cocId
          ? {
              ...item,
              transfers: item.transfers.filter((t) => t.id !== transferId),
            }
          : item
      )
    );
  };

  /** Auto-populate COC items from selected evidence groups */
  const autoPopulate = () => {
    const selected = ctx.selectedEvidence();
    if (selected.size === 0) return;

    const groups = ctx.groupedEvidence().filter((g) =>
      selected.has(g.primaryFile.path)
    );
    const existingIds = new Set(ctx.cocItems().map((c) => c.evidence_id));
    const newItems: COCItem[] = [];
    const caseNum = ctx.caseInfo().case_number || "0000";
    let idx = ctx.cocItems().length;

    for (const group of groups) {
      const evId = group.primaryFile.path;
      if (existingIds.has(evId)) continue;
      idx++;
      newItems.push({
        id: crypto.randomUUID(),
        coc_number: `${caseNum}-COC-${String(idx).padStart(3, "0")}`,
        evidence_id: evId,
        case_number: caseNum,
        description: group.baseName,
        item_type: "HardDrive",
        condition: "sealed",
        acquisition_date: "",
        entered_custody_date: "",
        submitted_by: ctx.examiner().name || "",
        received_by: "",
        storage_location: "",
        transfers: [],
        intake_hashes: [],
        notes: "",
        disposition: "in_custody",
        status: "draft",
      });
    }

    if (newItems.length > 0) {
      ctx.setCocItems([...ctx.cocItems(), ...newItems]);
      const newExpanded = new Set(expandedItems());
      newItems.forEach((i) => newExpanded.add(i.id));
      setExpandedItems(newExpanded);
    }
  };

  return (
    <div class="space-y-3">
      {/* Header */}
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="text-base">🔗</span>
          <div>
            <h3 class="text-sm font-semibold">Chain of Custody Records</h3>
            <p class="text-xs text-txt/50">
              Form 7 COC per evidence item. Locked records require initials to amend.
            </p>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <Show when={ctx.selectedEvidence().size > 0}>
            <button
              type="button"
              class="btn-sm flex items-center gap-1.5 text-accent"
              onClick={autoPopulate}
            >
              <HiOutlineDocumentDuplicate class="w-3.5 h-3.5" />
              Auto-populate
            </button>
          </Show>
          <button type="button" class="btn-sm" onClick={addCocItem}>
            <HiOutlinePlus class="w-3.5 h-3.5" />
            Add COC Item
          </button>
        </div>
      </div>

      {/* ── Modals ── */}
      <AmendmentModal
        info={amendingField()}
        newValue={amendNewValue()}
        onNewValueChange={(v) => setAmendNewValue(v)}
        initials={amendInitials()}
        onInitialsChange={(v) => setAmendInitials(v)}
        reason={amendReason()}
        onReasonChange={(v) => setAmendReason(v)}
        onApply={applyAmendment}
        onClose={() => setAmendingField(null)}
      />
      <LockConfirmationModal
        itemId={lockingItemId()}
        initials={lockInitials()}
        onInitialsChange={(v) => setLockInitials(v)}
        onConfirm={confirmLock}
        onClose={() => setLockingItemId(null)}
      />
      <VoidConfirmationModal
        itemId={voidingItemId()}
        initials={voidInitials()}
        onInitialsChange={(v) => setVoidInitials(v)}
        reason={voidReason()}
        onReasonChange={(v) => setVoidReason(v)}
        onConfirm={confirmVoid}
        onClose={() => setVoidingItemId(null)}
      />

      {/* ── COC Items List ── */}
      <Show
        when={ctx.cocItems().length > 0}
        fallback={
          <div class="text-center py-6 text-txt/50 border border-dashed border-border rounded-lg">
            <p class="text-sm mb-0.5">No COC records yet</p>
            <p class="text-xs">Add items or auto-populate from evidence.</p>
          </div>
        }
      >
        <div class="space-y-3">
          <For each={ctx.cocItems()}>
            {(item) => (
              <COCItemRow
                item={item}
                isExpanded={expandedItems().has(item.id)}
                onToggle={() => toggleExpanded(item.id)}
                onUpdate={(field, value) => updateItem(item.id, field, value)}
                onRemove={() => removeItem(item.id)}
                onLock={() => {
                  setLockingItemId(item.id);
                  setLockInitials("");
                }}
                onAddTransfer={() => addTransfer(item.id)}
                onUpdateTransfer={(tid, field, value) => updateTransfer(item.id, tid, field, value)}
                onRemoveTransfer={(tid) => removeTransfer(item.id, tid)}
              />
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}
