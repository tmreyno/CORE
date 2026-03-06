// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";

export const ChainOfCustodyContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Chain of Custody (COC) records use an append-only immutability model to ensure forensic integrity
      and maintain a complete audit trail for all evidence handling.
    </p>

    <div class="space-y-3">
      <h4 class="font-semibold text-txt text-sm">Status Lifecycle</h4>
      <div class="flex items-center gap-2 flex-wrap text-sm">
        <span class="badge badge-success">Draft</span>
        <span class="text-txt-muted">→ freely editable →</span>
        <span class="px-2 py-0.5 rounded bg-warning/20 text-warning text-xs font-medium">🔒 Locked</span>
        <span class="text-txt-muted">→ amend with initials + reason →</span>
        <span class="badge badge-error">Voided</span>
      </div>

      <div class="space-y-2 mt-3">
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm flex items-center gap-2">
            <span class="badge badge-success">Draft</span>
          </div>
          <p class="text-xs text-txt-muted mt-1">
            Freely editable. All fields can be modified and the record can be deleted (removed).
          </p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm flex items-center gap-2">
            <span class="px-2 py-0.5 rounded bg-warning/20 text-warning text-xs font-medium">🔒 Locked</span>
          </div>
          <p class="text-xs text-txt-muted mt-1">
            Immutable. Editing requires examiner's initials and a reason for the change — 
            this creates a formal amendment record in the audit trail. The original value is preserved.
          </p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm flex items-center gap-2">
            <span class="badge badge-error">Voided</span>
          </div>
          <p class="text-xs text-txt-muted mt-1">
            Soft-deleted. The record is hidden from active views but persists in the database for 
            the audit trail. Requires examiner's initials and a reason for voiding.
          </p>
        </div>
      </div>
    </div>
  </div>
);
