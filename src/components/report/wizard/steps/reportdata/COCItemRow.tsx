// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * COCItemRow — Individual Chain of Custody item row component.
 *
 * Renders a collapsible card with:
 * - Collapsed header: COC number, description, status/disposition badges, lock/remove buttons
 * - Expanded form: all COC fields, condition/disposition selects, transfer records
 *
 * Supports draft/locked/voided status display.
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
      class="border border-border/50 rounded-xl bg-surface/50 overflow-hidden"
      classList={{
        "opacity-50": isVoided(),
        "border-l-4 border-l-warning/50": isLocked(),
      }}
    >
      {/* Collapsed header */}
      <div
        class="flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-bg-hover/50 transition-colors"
        onClick={props.onToggle}
      >
        <div class="flex items-center gap-3">
          {props.isExpanded ? (
            <HiOutlineChevronUp class="w-4 h-4 text-txt-muted" />
          ) : (
            <HiOutlineChevronDown class="w-4 h-4 text-txt-muted" />
          )}
          <span class="font-mono text-sm font-medium text-accent">
            {props.item.coc_number || "—"}
          </span>
          <span
            class="text-sm text-txt/70 truncate max-w-[250px]"
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

      {/* Expanded form */}
      <Show when={props.isExpanded && !isVoided()}>
        <div class="px-4 pb-4 pt-1 border-t border-border/30 space-y-4">
          {/* Locked banner */}
          <Show when={isLocked()}>
            <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-warning/10 border border-warning/20 text-sm text-warning">
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

          <div class="grid grid-cols-3 gap-3">
            <div class="form-group">
              <label class="label">COC Number</label>
              <input
                class="input-sm font-mono"
                value={props.item.coc_number}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("coc_number", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Evidence ID</label>
              <input
                class="input-sm"
                value={props.item.evidence_id}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("evidence_id", e.currentTarget.value)}
              />
            </div>
            <div class="form-group">
              <label class="label">Case Number</label>
              <input
                class="input-sm"
                value={props.item.case_number}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("case_number", e.currentTarget.value)}
              />
            </div>
          </div>
          <div class="grid grid-cols-3 gap-3">
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
          <div class="grid grid-cols-2 gap-3">
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
          </div>
          <div class="grid grid-cols-2 gap-3">
            <div class="form-group">
              <label class="label">Submitted By</label>
              <input
                class="input-sm"
                value={props.item.submitted_by}
                readOnly={isLocked()}
                onInput={(e) => props.onUpdate("submitted_by", e.currentTarget.value)}
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
          <div class="grid grid-cols-3 gap-3">
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
          </div>
          <div class="form-group">
            <label class="label">Notes</label>
            <textarea
              class="textarea text-sm"
              rows={2}
              value={props.item.notes || ""}
              placeholder="Additional notes..."
              readOnly={isLocked()}
              onInput={(e) => props.onUpdate("notes", e.currentTarget.value)}
            />
          </div>

          {/* Transfers — always appendable (even on locked items) */}
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <label class="label">Transfer Records</label>
              <button type="button" class="btn-sm flex items-center gap-1" onClick={props.onAddTransfer}>
                <HiOutlinePlus class="w-3.5 h-3.5" /> Add Transfer
              </button>
            </div>
            <Show when={props.item.transfers.length > 0} fallback={<p class="text-xs text-txt/40 italic">No transfer records</p>}>
              <div class="space-y-2">
                <For each={props.item.transfers}>
                  {(transfer) => (
                    <div class="grid grid-cols-6 gap-2 items-end p-2 rounded-lg bg-bg/50 border border-border/30">
                      <div class="form-group">
                        <label class="label text-xs">Date/Time</label>
                        <input type="datetime-local" class="input-xs" value={transfer.timestamp} onInput={(e) => props.onUpdateTransfer(transfer.id, "timestamp", e.currentTarget.value)} />
                      </div>
                      <div class="form-group">
                        <label class="label text-xs">Released By</label>
                        <input class="input-xs" value={transfer.released_by} onInput={(e) => props.onUpdateTransfer(transfer.id, "released_by", e.currentTarget.value)} />
                      </div>
                      <div class="form-group">
                        <label class="label text-xs">Received By</label>
                        <input class="input-xs" value={transfer.received_by} onInput={(e) => props.onUpdateTransfer(transfer.id, "received_by", e.currentTarget.value)} />
                      </div>
                      <div class="form-group">
                        <label class="label text-xs">Purpose</label>
                        <select class="input-xs" value={transfer.purpose} onChange={(e) => props.onUpdateTransfer(transfer.id, "purpose", e.currentTarget.value)}>
                          <For each={COC_TRANSFER_PURPOSES}>{(opt) => <option value={opt.value}>{opt.label}</option>}</For>
                        </select>
                      </div>
                      <div class="form-group">
                        <label class="label text-xs">Method</label>
                        <select class="input-xs" value={transfer.method || "in_person"} onChange={(e) => props.onUpdateTransfer(transfer.id, "method", e.currentTarget.value)}>
                          <For each={COC_TRANSFER_METHODS}>{(opt) => <option value={opt.value}>{opt.label}</option>}</For>
                        </select>
                      </div>
                      <div class="flex items-end justify-end">
                        <button type="button" class="icon-btn-sm text-txt-muted hover:text-error" onClick={() => props.onRemoveTransfer(transfer.id)}>
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
}
