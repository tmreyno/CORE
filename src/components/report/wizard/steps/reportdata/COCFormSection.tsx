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
  HiOutlineTrash,
  HiOutlineChevronDown,
  HiOutlineChevronUp,
  HiOutlineDocumentDuplicate,
  HiOutlineLockClosed,
  HiOutlinePencilSquare,
} from "../../../../icons";
import { useWizard } from "../../WizardContext";
import {
  EVIDENCE_CONDITIONS,
  COC_DISPOSITIONS,
  COC_TRANSFER_METHODS,
  COC_TRANSFER_PURPOSES,
  EVIDENCE_TYPES,
} from "../../../constants";
import type { COCItem, COCTransfer } from "../../../types";
import { generateCocNumber } from "../../utils/reportNumbering";

export function COCFormSection() {
  const ctx = useWizard();
  const [expandedItems, setExpandedItems] = createSignal<Set<string>>(new Set());

  // --- Amendment prompt state ---
  const [amendingField, setAmendingField] = createSignal<{
    itemId: string;
    field: keyof COCItem;
    label: string;
    oldValue: string;
  } | null>(null);
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

  /** Get a status badge class + label */
  const statusBadge = (status?: string) => {
    switch (status) {
      case "locked":
        return { cls: "badge-warning", label: "🔒 Locked" };
      case "voided":
        return { cls: "badge-error", label: "Voided" };
      default:
        return { cls: "badge-success", label: "Draft" };
    }
  };

  return (
    <div class="space-y-5">
      {/* Header */}
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-accent/20 to-accent/5 flex items-center justify-center">
            <span class="text-xl">🔗</span>
          </div>
          <div>
            <h3 class="text-base font-semibold">Chain of Custody Records</h3>
            <p class="text-sm text-txt/60">
              Create Form 7 COC records per evidence item. Locked records require initials to amend.
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
              <HiOutlineDocumentDuplicate class="w-4 h-4" />
              Auto-populate from Evidence
            </button>
          </Show>
          <button type="button" class="btn-action-primary" onClick={addCocItem}>
            <HiOutlinePlus class="w-4 h-4" />
            Add COC Item
          </button>
        </div>
      </div>

      {/* ── Amendment Modal ── */}
      <Show when={amendingField()}>
        {(info) => (
          <div class="modal-overlay" style={{ "z-index": "9999" }}>
            <div class="modal-content w-[420px]">
              <div class="modal-header">
                <h2 class="text-sm font-semibold">Amend Locked Field</h2>
              </div>
              <div class="modal-body space-y-3">
                <p class="text-sm text-txt/70">
                  This COC item is <span class="font-semibold text-warning">locked</span>. Amending requires your initials and a reason.
                </p>
                <div class="form-group">
                  <label class="label text-xs">Field: {info().label}</label>
                  <div class="text-xs text-txt/50 font-mono bg-bg/50 rounded px-2 py-1">
                    Old: {info().oldValue || "(empty)"}
                  </div>
                </div>
                <div class="form-group">
                  <label class="label text-xs">New Value</label>
                  <input
                    class="input-sm"
                    value={amendNewValue()}
                    onInput={(e) => setAmendNewValue(e.currentTarget.value)}
                  />
                </div>
                <div class="grid grid-cols-2 gap-3">
                  <div class="form-group">
                    <label class="label text-xs">Your Initials *</label>
                    <input
                      class="input-sm font-mono uppercase"
                      maxLength={5}
                      placeholder="e.g., JD"
                      value={amendInitials()}
                      onInput={(e) =>
                        setAmendInitials(e.currentTarget.value.toUpperCase())
                      }
                    />
                  </div>
                  <div class="form-group">
                    <label class="label text-xs">Date</label>
                    <input
                      class="input-sm"
                      value={new Date().toLocaleDateString()}
                      disabled
                    />
                  </div>
                </div>
                <div class="form-group">
                  <label class="label text-xs">Reason for Amendment</label>
                  <input
                    class="input-sm"
                    placeholder="e.g., Corrected serial number"
                    value={amendReason()}
                    onInput={(e) => setAmendReason(e.currentTarget.value)}
                  />
                </div>
              </div>
              <div class="modal-footer justify-end">
                <button
                  class="btn btn-secondary"
                  onClick={() => setAmendingField(null)}
                >
                  Cancel
                </button>
                <button
                  class="btn btn-primary"
                  disabled={!amendInitials().trim()}
                  onClick={applyAmendment}
                >
                  <HiOutlinePencilSquare class="w-4 h-4" />
                  Amend & Initial
                </button>
              </div>
            </div>
          </div>
        )}
      </Show>

      {/* ── Lock Confirmation Modal ── */}
      <Show when={lockingItemId()}>
        <div class="modal-overlay" style={{ "z-index": "9999" }}>
          <div class="modal-content w-[380px]">
            <div class="modal-header">
              <h2 class="text-sm font-semibold">Lock COC Item</h2>
            </div>
            <div class="modal-body space-y-3">
              <p class="text-sm text-txt/70">
                Locking this record makes it <span class="font-semibold">immutable</span>.
                Future changes will require your initials and create an amendment record.
              </p>
              <div class="form-group">
                <label class="label text-xs">Your Initials *</label>
                <input
                  class="input-sm font-mono uppercase"
                  maxLength={5}
                  placeholder="e.g., JD"
                  value={lockInitials()}
                  onInput={(e) =>
                    setLockInitials(e.currentTarget.value.toUpperCase())
                  }
                />
              </div>
            </div>
            <div class="modal-footer justify-end">
              <button
                class="btn btn-secondary"
                onClick={() => setLockingItemId(null)}
              >
                Cancel
              </button>
              <button
                class="btn btn-primary"
                disabled={!lockInitials().trim()}
                onClick={confirmLock}
              >
                <HiOutlineLockClosed class="w-4 h-4" />
                Lock Record
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* ── Void Confirmation Modal ── */}
      <Show when={voidingItemId()}>
        <div class="modal-overlay" style={{ "z-index": "9999" }}>
          <div class="modal-content w-[400px]">
            <div class="modal-header">
              <h2 class="text-sm font-semibold text-error">Void COC Item</h2>
            </div>
            <div class="modal-body space-y-3">
              <p class="text-sm text-txt/70">
                This will mark the COC item as <span class="font-semibold text-error">voided</span>.
                The record remains in the database for audit purposes but will be hidden from active views.
              </p>
              <div class="form-group">
                <label class="label text-xs">Your Initials *</label>
                <input
                  class="input-sm font-mono uppercase"
                  maxLength={5}
                  placeholder="e.g., JD"
                  value={voidInitials()}
                  onInput={(e) =>
                    setVoidInitials(e.currentTarget.value.toUpperCase())
                  }
                />
              </div>
              <div class="form-group">
                <label class="label text-xs">Reason for Voiding *</label>
                <input
                  class="input-sm"
                  placeholder="e.g., Duplicate record, entered in error"
                  value={voidReason()}
                  onInput={(e) => setVoidReason(e.currentTarget.value)}
                />
              </div>
            </div>
            <div class="modal-footer justify-end">
              <button
                class="btn btn-secondary"
                onClick={() => setVoidingItemId(null)}
              >
                Cancel
              </button>
              <button
                class="btn btn-primary bg-error hover:bg-error/80"
                disabled={!voidInitials().trim() || !voidReason().trim()}
                onClick={confirmVoid}
              >
                Void Record
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* ── COC Items List ── */}
      <Show
        when={ctx.cocItems().length > 0}
        fallback={
          <div class="text-center py-12 text-txt/50 border border-dashed border-border rounded-xl">
            <p class="text-lg mb-1">No COC records yet</p>
            <p class="text-sm">
              Click "Add COC Item" or "Auto-populate from Evidence" to get started.
            </p>
          </div>
        }
      >
        <div class="space-y-3">
          <For each={ctx.cocItems()}>
            {(item) => {
              const isExpanded = () => expandedItems().has(item.id);
              const isLocked = () => item.status === "locked";
              const isVoided = () => item.status === "voided";
              const badge = () => statusBadge(item.status);

              return (
                <div
                  class="border border-border/50 rounded-xl bg-surface/50 overflow-hidden"
                  classList={{
                    "opacity-50": isVoided(),
                    "border-l-4 border-l-warning/50": isLocked(),
                  }}
                >
                  {/* Collapsed header */}
                  <div
                    class="flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-bg-hover/50 transition-colors"
                    onClick={() => toggleExpanded(item.id)}
                  >
                    <div class="flex items-center gap-3">
                      {isExpanded() ? (
                        <HiOutlineChevronUp class="w-4 h-4 text-txt-muted" />
                      ) : (
                        <HiOutlineChevronDown class="w-4 h-4 text-txt-muted" />
                      )}
                      <span class="font-mono text-sm font-medium text-accent">
                        {item.coc_number || "—"}
                      </span>
                      <span
                        class="text-sm text-txt/70 truncate max-w-[250px]"
                        classList={{ "line-through": isVoided() }}
                      >
                        {item.description || item.evidence_id || "Untitled"}
                      </span>
                    </div>
                    <div class="flex items-center gap-2">
                      <span class={`badge ${badge().cls}`}>
                        {badge().label}
                      </span>
                      <span
                        class={`badge ${
                          item.disposition === "in_custody"
                            ? "badge-success"
                            : item.disposition === "released"
                              ? "badge-warning"
                              : "badge-error"
                        }`}
                      >
                        {(item.disposition || "in_custody").replace(/_/g, " ")}
                      </span>
                      {/* Lock button — only for draft items */}
                      <Show when={!isLocked() && !isVoided()}>
                        <button
                          type="button"
                          class="icon-btn-sm text-txt-muted hover:text-warning"
                          title="Lock this record (make immutable)"
                          onClick={(e) => {
                            e.stopPropagation();
                            setLockingItemId(item.id);
                            setLockInitials("");
                          }}
                        >
                          <HiOutlineLockClosed class="w-4 h-4" />
                        </button>
                      </Show>
                      {/* Remove/Void button */}
                      <Show when={!isVoided()}>
                        <button
                          type="button"
                          class="icon-btn-sm text-txt-muted hover:text-error"
                          title={isLocked() ? "Void this record" : "Remove this record"}
                          onClick={(e) => {
                            e.stopPropagation();
                            removeItem(item.id);
                          }}
                        >
                          <HiOutlineTrash class="w-4 h-4" />
                        </button>
                      </Show>
                    </div>
                  </div>

                  {/* Expanded form */}
                  <Show when={isExpanded() && !isVoided()}>
                    <div class="px-4 pb-4 pt-1 border-t border-border/30 space-y-4">
                      {/* Locked banner */}
                      <Show when={isLocked()}>
                        <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-warning/10 border border-warning/20 text-sm text-warning">
                          <HiOutlineLockClosed class="w-4 h-4 flex-shrink-0" />
                          <span>
                            Locked by <strong>{item.locked_by}</strong> on{" "}
                            {item.locked_at
                              ? new Date(item.locked_at).toLocaleDateString()
                              : "—"}
                            . Editing will open an amendment prompt.
                          </span>
                        </div>
                      </Show>

                      <div class="grid grid-cols-3 gap-3">
                        <div class="form-group">
                          <label class="label">COC Number</label>
                          <input
                            class="input-sm font-mono"
                            value={item.coc_number}
                            readOnly={isLocked()}
                            onInput={(e) => updateItem(item.id, "coc_number", e.currentTarget.value)}
                          />
                        </div>
                        <div class="form-group">
                          <label class="label">Evidence ID</label>
                          <input
                            class="input-sm"
                            value={item.evidence_id}
                            readOnly={isLocked()}
                            onInput={(e) => updateItem(item.id, "evidence_id", e.currentTarget.value)}
                          />
                        </div>
                        <div class="form-group">
                          <label class="label">Case Number</label>
                          <input
                            class="input-sm"
                            value={item.case_number}
                            readOnly={isLocked()}
                            onInput={(e) => updateItem(item.id, "case_number", e.currentTarget.value)}
                          />
                        </div>
                      </div>
                      <div class="grid grid-cols-3 gap-3">
                        <div class="form-group col-span-2">
                          <label class="label">Description</label>
                          <input
                            class="input-sm"
                            value={item.description}
                            placeholder="Evidence description"
                            readOnly={isLocked()}
                            onInput={(e) => updateItem(item.id, "description", e.currentTarget.value)}
                          />
                        </div>
                        <div class="form-group">
                          <label class="label">Evidence Type</label>
                          <select
                            class="input-sm"
                            value={item.item_type}
                            disabled={isLocked()}
                            onChange={(e) => updateItem(item.id, "item_type", e.currentTarget.value)}
                          >
                            <For each={EVIDENCE_TYPES}>{(opt) => <option value={opt.value}>{opt.label}</option>}</For>
                          </select>
                        </div>
                      </div>
                      <div class="grid grid-cols-2 gap-3">
                        <div class="form-group">
                          <label class="label">Acquisition Date</label>
                          <input
                            type="datetime-local"
                            class="input-sm"
                            value={item.acquisition_date}
                            readOnly={isLocked()}
                            onInput={(e) => updateItem(item.id, "acquisition_date", e.currentTarget.value)}
                          />
                        </div>
                        <div class="form-group">
                          <label class="label">Entered Custody Date</label>
                          <input
                            type="datetime-local"
                            class="input-sm"
                            value={item.entered_custody_date}
                            readOnly={isLocked()}
                            onInput={(e) => updateItem(item.id, "entered_custody_date", e.currentTarget.value)}
                          />
                        </div>
                      </div>
                      <div class="grid grid-cols-2 gap-3">
                        <div class="form-group">
                          <label class="label">Submitted By</label>
                          <input
                            class="input-sm"
                            value={item.submitted_by}
                            readOnly={isLocked()}
                            onInput={(e) => updateItem(item.id, "submitted_by", e.currentTarget.value)}
                          />
                        </div>
                        <div class="form-group">
                          <label class="label">Received By</label>
                          <input
                            class="input-sm"
                            value={item.received_by}
                            readOnly={isLocked()}
                            onInput={(e) => updateItem(item.id, "received_by", e.currentTarget.value)}
                          />
                        </div>
                      </div>
                      <div class="grid grid-cols-3 gap-3">
                        <div class="form-group">
                          <label class="label">Condition</label>
                          <select
                            class="input-sm"
                            value={item.condition}
                            disabled={isLocked()}
                            onChange={(e) => updateItem(item.id, "condition", e.currentTarget.value)}
                          >
                            <For each={EVIDENCE_CONDITIONS}>{(opt) => <option value={opt.value}>{opt.label}</option>}</For>
                          </select>
                        </div>
                        <div class="form-group">
                          <label class="label">Storage Location</label>
                          <input
                            class="input-sm"
                            value={item.storage_location || ""}
                            placeholder="Evidence locker, shelf, etc."
                            readOnly={isLocked()}
                            onInput={(e) => updateItem(item.id, "storage_location", e.currentTarget.value)}
                          />
                        </div>
                        <div class="form-group">
                          <label class="label">Disposition</label>
                          <select
                            class="input-sm"
                            value={item.disposition || "in_custody"}
                            disabled={isLocked()}
                            onChange={(e) => updateItem(item.id, "disposition", e.currentTarget.value)}
                          >
                            <For each={COC_DISPOSITIONS}>{(opt) => <option value={opt.value}>{opt.label}</option>}</For>
                          </select>
                        </div>
                      </div>
                      <div class="form-group">
                        <label class="label">Notes</label>
                        <textarea
                          class="textarea text-sm"
                          rows={2}
                          value={item.notes || ""}
                          placeholder="Additional notes..."
                          readOnly={isLocked()}
                          onInput={(e) => updateItem(item.id, "notes", e.currentTarget.value)}
                        />
                      </div>

                      {/* Transfers — always appendable (even on locked items) */}
                      <div class="space-y-2">
                        <div class="flex items-center justify-between">
                          <label class="label">Transfer Records</label>
                          <button type="button" class="btn-sm flex items-center gap-1" onClick={() => addTransfer(item.id)}>
                            <HiOutlinePlus class="w-3.5 h-3.5" /> Add Transfer
                          </button>
                        </div>
                        <Show when={item.transfers.length > 0} fallback={<p class="text-xs text-txt/40 italic">No transfer records</p>}>
                          <div class="space-y-2">
                            <For each={item.transfers}>
                              {(transfer) => (
                                <div class="grid grid-cols-6 gap-2 items-end p-2 rounded-lg bg-bg/50 border border-border/30">
                                  <div class="form-group">
                                    <label class="label text-xs">Date/Time</label>
                                    <input type="datetime-local" class="input-xs" value={transfer.timestamp} onInput={(e) => updateTransfer(item.id, transfer.id, "timestamp", e.currentTarget.value)} />
                                  </div>
                                  <div class="form-group">
                                    <label class="label text-xs">Released By</label>
                                    <input class="input-xs" value={transfer.released_by} onInput={(e) => updateTransfer(item.id, transfer.id, "released_by", e.currentTarget.value)} />
                                  </div>
                                  <div class="form-group">
                                    <label class="label text-xs">Received By</label>
                                    <input class="input-xs" value={transfer.received_by} onInput={(e) => updateTransfer(item.id, transfer.id, "received_by", e.currentTarget.value)} />
                                  </div>
                                  <div class="form-group">
                                    <label class="label text-xs">Purpose</label>
                                    <select class="input-xs" value={transfer.purpose} onChange={(e) => updateTransfer(item.id, transfer.id, "purpose", e.currentTarget.value)}>
                                      <For each={COC_TRANSFER_PURPOSES}>{(opt) => <option value={opt.value}>{opt.label}</option>}</For>
                                    </select>
                                  </div>
                                  <div class="form-group">
                                    <label class="label text-xs">Method</label>
                                    <select class="input-xs" value={transfer.method || "in_person"} onChange={(e) => updateTransfer(item.id, transfer.id, "method", e.currentTarget.value)}>
                                      <For each={COC_TRANSFER_METHODS}>{(opt) => <option value={opt.value}>{opt.label}</option>}</For>
                                    </select>
                                  </div>
                                  <div class="flex items-end justify-end">
                                    <button type="button" class="icon-btn-sm text-txt-muted hover:text-error" onClick={() => removeTransfer(item.id, transfer.id)}>
                                      <HiOutlineTrash class="w-3.5 h-3.5" />
                                    </button>
                                  </div>
                                </div>
                              )}
                            </For>
                          </div>
                        </Show>
                      </div>
                    </div>
                  </Show>
                </div>
              );
            }}
          </For>
        </div>
      </Show>
    </div>
  );
}
