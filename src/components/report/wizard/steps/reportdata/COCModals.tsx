// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * COCModals — Modal dialogs for COC immutability operations.
 *
 * - AmendmentModal: Amend a locked field (requires initials + reason)
 * - LockConfirmationModal: Lock a draft COC item (makes immutable)
 * - VoidConfirmationModal: Void (soft-delete) a COC item
 */

import { Show } from "solid-js";
import {
  HiOutlinePencilSquare,
  HiOutlineLockClosed,
} from "../../../../icons";
import type { COCItem } from "../../../types";

// ── Shared types ──

export interface AmendFieldInfo {
  itemId: string;
  field: keyof COCItem;
  label: string;
  oldValue: string;
}

// ── Amendment Modal ──

export interface AmendmentModalProps {
  info: AmendFieldInfo | null;
  newValue: string;
  onNewValueChange: (value: string) => void;
  initials: string;
  onInitialsChange: (value: string) => void;
  reason: string;
  onReasonChange: (value: string) => void;
  onApply: () => void;
  onClose: () => void;
}

export function AmendmentModal(props: AmendmentModalProps) {
  return (
    <Show when={props.info}>
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
                  value={props.newValue}
                  onInput={(e) => props.onNewValueChange(e.currentTarget.value)}
                />
              </div>
              <div class="grid grid-cols-2 gap-3">
                <div class="form-group">
                  <label class="label text-xs">Your Initials *</label>
                  <input
                    class="input-sm font-mono uppercase"
                    maxLength={5}
                    placeholder="e.g., JD"
                    value={props.initials}
                    onInput={(e) =>
                      props.onInitialsChange(e.currentTarget.value.toUpperCase())
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
                  value={props.reason}
                  onInput={(e) => props.onReasonChange(e.currentTarget.value)}
                />
              </div>
            </div>
            <div class="modal-footer justify-end">
              <button
                class="btn btn-secondary"
                onClick={props.onClose}
              >
                Cancel
              </button>
              <button
                class="btn btn-primary"
                disabled={!props.initials.trim()}
                onClick={props.onApply}
              >
                <HiOutlinePencilSquare class="w-4 h-4" />
                Amend &amp; Initial
              </button>
            </div>
          </div>
        </div>
      )}
    </Show>
  );
}

// ── Lock Confirmation Modal ──

export interface LockConfirmationModalProps {
  itemId: string | null;
  initials: string;
  onInitialsChange: (value: string) => void;
  onConfirm: () => void;
  onClose: () => void;
}

export function LockConfirmationModal(props: LockConfirmationModalProps) {
  return (
    <Show when={props.itemId}>
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
                value={props.initials}
                onInput={(e) =>
                  props.onInitialsChange(e.currentTarget.value.toUpperCase())
                }
              />
            </div>
          </div>
          <div class="modal-footer justify-end">
            <button
              class="btn btn-secondary"
              onClick={props.onClose}
            >
              Cancel
            </button>
            <button
              class="btn btn-primary"
              disabled={!props.initials.trim()}
              onClick={props.onConfirm}
            >
              <HiOutlineLockClosed class="w-4 h-4" />
              Lock Record
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
}

// ── Void Confirmation Modal ──

export interface VoidConfirmationModalProps {
  itemId: string | null;
  initials: string;
  onInitialsChange: (value: string) => void;
  reason: string;
  onReasonChange: (value: string) => void;
  onConfirm: () => void;
  onClose: () => void;
}

export function VoidConfirmationModal(props: VoidConfirmationModalProps) {
  return (
    <Show when={props.itemId}>
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
                value={props.initials}
                onInput={(e) =>
                  props.onInitialsChange(e.currentTarget.value.toUpperCase())
                }
              />
            </div>
            <div class="form-group">
              <label class="label text-xs">Reason for Voiding *</label>
              <input
                class="input-sm"
                placeholder="e.g., Duplicate record, entered in error"
                value={props.reason}
                onInput={(e) => props.onReasonChange(e.currentTarget.value)}
              />
            </div>
          </div>
          <div class="modal-footer justify-end">
            <button
              class="btn btn-secondary"
              onClick={props.onClose}
            >
              Cancel
            </button>
            <button
              class="btn btn-primary bg-error hover:bg-error/80"
              disabled={!props.initials.trim() || !props.reason.trim()}
              onClick={props.onConfirm}
            >
              Void Record
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
}
