// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * COCItemRow — Individual Chain of Custody item row component.
 *
 * Renders a collapsible card structured to match EPA CID OCEFT Form 7-01:
 *  1. Header: Case Title, Office, Case Number, COC#
 *  2. Owner / Source / Contact
 *  3. Collection Method (checkboxes matching Form 7-01)
 *  4. Item Details (Item/Box Number, Description, Type, Make, Model, etc.)
 *  5. Collection & Custody (Collected By, Date, Storage Location)
 *  6. Remarks / Notes
 *  7. Transfer Records (Relinquished to, Storage Location, Date)
 *  8. Final Disposition
 *
 * Supports draft/locked/voided status via immutability model (schema v5+).
 */

import { Show, For } from "solid-js";
import {
  HiOutlinePlus,
  HiOutlineTrash,
  HiOutlineChevronDown,
  HiOutlineChevronUp,
  HiOutlineLockClosed,
} from "../../../../icons";
import {
  EVIDENCE_CONDITIONS,
  COC_DISPOSITIONS,
  COC_TRANSFER_METHODS,
  COC_TRANSFER_PURPOSES,
  EVIDENCE_TYPES,
  COC_COLLECTION_METHODS,
} from "../../../constants";
import type { COCItem, COCTransfer } from "../../../types";

// ── Helpers ──

/** Map a status string to a badge CSS class + label */
function statusBadge(status?: string): { cls: string; label: string } {
  switch (status) {
    case "locked":
      return { cls: "badge-warning", label: "🔒 Locked" };
    case "voided":
      return { cls: "badge-error", label: "Voided" };
    default:
      return { cls: "badge-success", label: "Draft" };
  }
}

// ── Section Header ──

function SectionHeader(props: { title: string; number?: string }) {
  return (
    <div class="flex items-center gap-2 pt-2 pb-0.5 border-b border-border/30">
      <Show when={props.number}>
        <span class="text-2xs font-bold text-accent/70 bg-accent/10 rounded px-1.5 py-0.5">
          {props.number}
        </span>
      </Show>
      <span class="text-xs font-semibold text-txt/80 uppercase tracking-wider">
        {props.title}
      </span>
    </div>
  );
}

// ── Props ──

export interface COCItemRowProps {
  item: COCItem;
  isExpanded: boolean;
  onToggle: () => void;
  onUpdate: (field: keyof COCItem, value: unknown) => void;
  onRemove: () => void;
  onLock: () => void;
  onAddTransfer: () => void;
  onUpdateTransfer: (transferId: string, field: keyof COCTransfer, value: string) => void;
  onRemoveTransfer: (transferId: string) => void;
}

// ── Component ──

export function COCItemRow(props: COCItemRowProps) {
  const isLocked = () => props.item.status === "locked";
  const isVoided = () => props.item.status === "voided";
  const badge = () => statusBadge(props.item.status);

  return (
    <div
      class="border border-border/50 rounded-lg bg-surface/50 overflow-hidden"
      classList={{
        "opacity-50": isVoided(),
        "border-l-4 border-l-warning/50": isLocked(),
      }}
    >
      {/* ── Collapsed Header ── */}
      <div
        class="flex items-center justify-between px-3 py-2 cursor-pointer hover:bg-bg-hover/50 transition-colors"
        onClick={props.onToggle}
      >
        <div class="flex items-center gap-2">
          {props.isExpanded ? (
            <HiOutlineChevronUp class="w-4 h-4 text-txt-muted" />
          ) : (
            <HiOutlineChevronDown class="w-4 h-4 text-txt-muted" />
          )}
          <span class="font-mono text-xs font-medium text-accent">
            {props.item.coc_number || "—"}
          </span>
          <span
            class="text-xs text-txt/70 truncate max-w-[250px]"
            classList={{ "line-through": isVoided() }}
          >
            {props.item.description || props.item.evidence_id || "Untitled"}
          </span>
        </div>
        <div class="flex items-center gap-2">
          <span class={`badge ${badge().cls}`}>
            {badge().label}
          </span>
          <span
            class={`badge ${
              props.item.disposition === "in_custody"
                ? "badge-success"
                : props.item.disposition === "released"
                  ? "badge-warning"
                  : "badge-error"
            }`}
          >
            {(props.item.disposition || "in_custody").replace(/_/g, " ")}
          </span>
          {/* Lock button — only for draft items */}
          <Show when={!isLocked() && !isVoided()}>
            <button
              type="button"
              class="icon-btn-sm text-txt-muted hover:text-warning"
              title="Lock this record (make immutable)"
              onClick={(e) => {
                e.stopPropagation();
                props.onLock();
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
                props.onRemove();
              }}
            >
              <HiOutlineTrash class="w-4 h-4" />
            </button>
          </Show>
        </div>
      </div>

      {/* ── Expanded Form (Form 7-01 layout) ── */}
      <Show when={props.isExpanded && !isVoided()}>
        <div class="px-3 pb-3 pt-1 border-t border-border/30 space-y-3">
          {/* Locked banner */}
          <Show when={isLocked()}>
            <div class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-md bg-warning/10 border border-warning/20 text-xs text-warning">
              <HiOutlineLockClosed class="w-4 h-4 flex-shrink-0" />
              <span>
                Locked by <strong>{props.item.locked_by}</strong> on{" "}
                {props.item.locked_at
                  ? new Date(props.item.locked_at).toLocaleDateString()
                  : "—"}
                . Editing will open an amendment prompt.
              </span>
            </div>
          </Show>

          {/* ─── 1. Header Section (Form 7-01 top row) ─── */}
          <SectionHeader title="Case Information" number="1" />
          <div class="grid grid-cols-4 gap-2">
            <div class="form-group col-span-2">
              <label class="label">Case Title</label>
              <input
                class="input-sm"
                value={props.item.case_title || ""}
                placeholder="Case title"
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("case_title", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Office</label>
              <input
                class="input-sm"
                value={props.item.office || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("office", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">COC #</label>
              <input
                class="input-sm font-mono"
                value={props.item.coc_number}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("coc_number", e.currentTarget.value)}
              />
            </div>
          </div>
          <div class="grid grid-cols-2 gap-2">
            <div class="form-group">
              <label class="label">Case Number</label>
              <input
                class="input-sm"
                value={props.item.case_number}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("case_number", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Evidence ID (Item/Box Number)</label>
              <input
                class="input-sm"
                value={props.item.evidence_id}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("evidence_id", e.currentTarget.value)}
              />
            </div>
          </div>

          {/* ─── 2. Owner / Source / Contact ─── */}
          <SectionHeader title="Owner / Source / Contact" number="2" />
          <div class="grid grid-cols-3 gap-2">
            <div class="form-group">
              <label class="label">Owner Name</label>
              <input
                class="input-sm"
                value={props.item.owner_name || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("owner_name", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Address</label>
              <input
                class="input-sm"
                value={props.item.owner_address || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("owner_address", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Phone Number</label>
              <input
                class="input-sm"
                value={props.item.owner_phone || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("owner_phone", e.currentTarget.value)}
              />
            </div>
          </div>
          <div class="form-group">
            <label class="label">Source</label>
            <input
              class="input-sm"
              value={props.item.source || ""}
              placeholder="Source of the evidence"
              readOnly={isLocked()}
              onInput={(e) => props.onUpdate("source", e.currentTarget.value)}
            />
          </div>
          <div class="grid grid-cols-3 gap-2">
            <div class="form-group">
              <label class="label">Other Contact Name</label>
              <input
                class="input-sm"
                value={props.item.other_contact_name || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("other_contact_name", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Relation to Owner</label>
              <input
                class="input-sm"
                value={props.item.other_contact_relation || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("other_contact_relation", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Phone Number</label>
              <input
                class="input-sm"
                value={props.item.other_contact_phone || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("other_contact_phone", e.currentTarget.value)}
              />
            </div>
          </div>

          {/* ─── 3. Collection Method (Form 7-01 checkboxes) ─── */}
          <SectionHeader title="Collection Method" number="3" />
          <div class="flex flex-wrap gap-x-4 gap-y-1.5 py-1">
            <For each={COC_COLLECTION_METHODS}>
              {(method) => (
                <label class="flex items-center gap-1.5 text-xs text-txt/80 cursor-pointer">
                  <input
                    type="radio"
                    name={`coc-method-${props.item.id}`}
                    class="accent-accent"
                    checked={props.item.collection_method === method.value}
                    disabled={isLocked()}
                    onChange={() => props.onUpdate("collection_method", method.value)}
                  />
                  {method.label}
                </label>
              )}
            </For>
          </div>
          <Show when={props.item.collection_method === "other"}>
            <div class="form-group">
              <label class="label">Other (Describe)</label>
              <input
                class="input-sm"
                value={props.item.collection_method_other || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("collection_method_other", e.currentTarget.value)}
              />
            </div>
          </Show>

          {/* ─── 4. Item Details ─── */}
          <SectionHeader title="Item Details" number="4" />
          <div class="grid grid-cols-3 gap-2">
            <div class="form-group col-span-2">
              <label class="label">Description</label>
              <input
                class="input-sm"
                value={props.item.description}
                placeholder="Evidence description"
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("description", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Evidence Type</label>
              <select
                class="input-sm"
                value={props.item.item_type}
                disabled={isLocked()}
                onChange={(e) => props.onUpdate("item_type", e.currentTarget.value)}
              >
                <For each={EVIDENCE_TYPES}>{(opt) => <option value={opt.value}>{opt.label}</option>}</For>
              </select>
            </div>
          </div>
          <div class="grid grid-cols-4 gap-2">
            <div class="form-group">
              <label class="label">Make</label>
              <input
                class="input-sm"
                value={props.item.make || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("make", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Model</label>
              <input
                class="input-sm"
                value={props.item.model || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("model", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Serial Number</label>
              <input
                class="input-sm"
                value={props.item.serial_number || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("serial_number", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Capacity</label>
              <input
                class="input-sm"
                value={props.item.capacity || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("capacity", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Condition</label>
              <select
                class="input-sm"
                value={props.item.condition}
                disabled={isLocked()}
                onChange={(e) => props.onUpdate("condition", e.currentTarget.value)}
              >
                <For each={EVIDENCE_CONDITIONS}>{(opt) => <option value={opt.value}>{opt.label}</option>}</For>
              </select>
            </div>
          </div>

          {/* ─── 5. Collection & Custody ─── */}
          <SectionHeader title="Collection & Custody" number="5" />
          <div class="grid grid-cols-3 gap-2">
            <div class="form-group">
              <label class="label">Collected By (Print)</label>
              <input
                class="input-sm"
                value={props.item.submitted_by}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("submitted_by", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Date Collected</label>
              <input
                type="datetime-local"
                class="input-sm"
                value={props.item.collected_date || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("collected_date", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Received By</label>
              <input
                class="input-sm"
                value={props.item.received_by}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("received_by", e.currentTarget.value)}
              />
            </div>
          </div>
          <div class="grid grid-cols-3 gap-2">
            <div class="form-group">
              <label class="label">Acquisition Date</label>
              <input
                type="datetime-local"
                class="input-sm"
                value={props.item.acquisition_date}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("acquisition_date", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Entered Custody Date</label>
              <input
                type="datetime-local"
                class="input-sm"
                value={props.item.entered_custody_date}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("entered_custody_date", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Storage Location</label>
              <input
                class="input-sm"
                value={props.item.storage_location || ""}
                placeholder="Evidence locker, shelf, etc."
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("storage_location", e.currentTarget.value)}
              />
            </div>
          </div>

          {/* Intake Hashes (read-only display) */}
          <Show when={(props.item.intake_hashes ?? []).length > 0}>
            <div class="form-group">
              <label class="label">Intake Hashes</label>
              <div class="flex flex-wrap gap-2">
                <For each={props.item.intake_hashes}>
                  {(h) => (
                    <span class="badge badge-neutral font-mono text-2xs">
                      {h.algorithm}: {h.value.substring(0, 16)}…
                    </span>
                  )}
                </For>
              </div>
            </div>
          </Show>

          {/* ─── 6. Remarks ─── */}
          <SectionHeader title="Remarks" number="6" />
          <div class="form-group">
            <textarea
              class="textarea text-sm"
              rows={2}
              value={props.item.notes || ""}
              placeholder="Remarks / additional notes..."
              readOnly={isLocked()}
              onInput={(e) => props.onUpdate("notes", e.currentTarget.value)}
            />
          </div>

          {/* ─── 7. Transfer Records (Form 7-01 transfer rows) ─── */}
          <SectionHeader title="Transfer Records" number="7" />
          <div class="space-y-2">
            <div class="flex items-center justify-end">
              <button type="button" class="btn-sm flex items-center gap-1" onClick={props.onAddTransfer}>
                <HiOutlinePlus class="w-3.5 h-3.5" /> Add Transfer
              </button>
            </div>
            <Show when={props.item.transfers.length > 0} fallback={<p class="text-xs text-txt/40 italic">No transfer records</p>}>
              <div class="space-y-2">
                <For each={props.item.transfers}>
                  {(transfer, idx) => (
                    <div class="p-2 rounded-lg bg-bg/50 border border-border/30 space-y-1.5">
                      <div class="flex items-center justify-between">
                        <span class="text-2xs font-semibold text-txt-muted uppercase">
                          Transfer #{idx() + 1}
                        </span>
                        <button type="button" class="icon-btn-sm text-txt-muted hover:text-error" onClick={() => props.onRemoveTransfer(transfer.id)}>
                          <HiOutlineTrash class="w-3.5 h-3.5" />
                        </button>
                      </div>
                      <div class="grid grid-cols-4 gap-2 items-end">
                        <div class="form-group">
                          <label class="label text-xs">Relinquished By</label>
                          <input class="input-xs" value={transfer.released_by} onInput={(e) => props.onUpdateTransfer(transfer.id, "released_by", e.currentTarget.value)} />
                        </div>
                        <div class="form-group">
                          <label class="label text-xs">Received By</label>
                          <input class="input-xs" value={transfer.received_by} onInput={(e) => props.onUpdateTransfer(transfer.id, "received_by", e.currentTarget.value)} />
                        </div>
                        <div class="form-group">
                          <label class="label text-xs">Date/Time</label>
                          <input type="datetime-local" class="input-xs" value={transfer.timestamp} onInput={(e) => props.onUpdateTransfer(transfer.id, "timestamp", e.currentTarget.value)} />
                        </div>
                        <div class="form-group">
                          <label class="label text-xs">Purpose</label>
                          <select class="input-xs" value={transfer.purpose} onChange={(e) => props.onUpdateTransfer(transfer.id, "purpose", e.currentTarget.value)}>
                            <For each={COC_TRANSFER_PURPOSES}>{(opt) => <option value={opt.value}>{opt.label}</option>}</For>
                          </select>
                        </div>
                      </div>
                      <div class="grid grid-cols-3 gap-2 items-end">
                        <div class="form-group">
                          <label class="label text-xs">Storage Location</label>
                          <input class="input-xs" value={transfer.storage_location || ""} placeholder="Location after transfer" onInput={(e) => props.onUpdateTransfer(transfer.id, "storage_location", e.currentTarget.value)} />
                        </div>
                        <div class="form-group">
                          <label class="label text-xs">Date Entered Storage</label>
                          <input type="datetime-local" class="input-xs" value={transfer.storage_date || ""} onInput={(e) => props.onUpdateTransfer(transfer.id, "storage_date", e.currentTarget.value)} />
                        </div>
                        <div class="form-group">
                          <label class="label text-xs">Method</label>
                          <select class="input-xs" value={transfer.method || "in_person"} onChange={(e) => props.onUpdateTransfer(transfer.id, "method", e.currentTarget.value)}>
                            <For each={COC_TRANSFER_METHODS}>{(opt) => <option value={opt.value}>{opt.label}</option>}</For>
                          </select>
                        </div>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </div>

          {/* ─── 8. Final Disposition (Form 7-01 bottom section) ─── */}
          <SectionHeader title="Final Disposition" number="8" />
          <div class="grid grid-cols-3 gap-2">
            <div class="form-group">
              <label class="label">Disposition</label>
              <select
                class="input-sm"
                value={props.item.disposition || "in_custody"}
                disabled={isLocked()}
                onChange={(e) => props.onUpdate("disposition", e.currentTarget.value)}
              >
                <For each={COC_DISPOSITIONS}>{(opt) => <option value={opt.value}>{opt.label}</option>}</For>
              </select>
            </div>
            <div class="form-group">
              <label class="label">Final Disposition By (Print)</label>
              <input
                class="input-sm"
                value={props.item.disposition_by || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("disposition_by", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Returned To (Sign/Date)</label>
              <input
                class="input-sm"
                value={props.item.returned_to || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("returned_to", e.currentTarget.value)}
              />
            </div>
          </div>
          <div class="grid grid-cols-3 gap-2">
            <div class="form-group">
              <label class="label">Destruction Date</label>
              <input
                type="date"
                class="input-sm"
                value={props.item.destruction_date || ""}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("destruction_date", e.currentTarget.value)}
              />
            </div>
            <div class="form-group col-span-2">
              <label class="label">Disposition Notes / Other</label>
              <input
                class="input-sm"
                value={props.item.disposition_notes || ""}
                placeholder="Other disposition details"
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("disposition_notes", e.currentTarget.value)}
              />
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
