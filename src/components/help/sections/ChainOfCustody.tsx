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
            Use this status while you are still filling in details and confirming accuracy.
          </p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm flex items-center gap-2">
            <span class="px-2 py-0.5 rounded bg-warning/20 text-warning text-xs font-medium">🔒 Locked</span>
          </div>
          <p class="text-xs text-txt-muted mt-1">
            Immutable. Editing requires examiner's initials and a reason for the change — 
            this creates a formal amendment record in the audit trail. The original value is preserved
            alongside the corrected value. Lock records once they are verified and complete.
          </p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm flex items-center gap-2">
            <span class="badge badge-error">Voided</span>
          </div>
          <p class="text-xs text-txt-muted mt-1">
            Soft-deleted. The record is hidden from active views but persists in the database for 
            the audit trail. Requires examiner's initials and a reason for voiding. 
            Voided records can never be restored — this is by design for forensic integrity.
          </p>
        </div>
      </div>

      <h4 class="font-semibold text-txt text-sm">How to Create and Manage COC Records</h4>
      <div class="text-txt-secondary text-sm space-y-1 ml-1">
        <p><strong>1.</strong> Open the Report Wizard → navigate to the <strong>Chain of Custody</strong> section.</p>
        <p><strong>2.</strong> Click <strong>Add COC Item</strong> for each evidence transfer.</p>
        <p><strong>3.</strong> Fill in: COC number, case #, evidence description, submitted by, received by, date/time, location.</p>
        <p><strong>4.</strong> Verify all details, then click <strong>Lock</strong> to finalize the record.</p>
        <p><strong>5.</strong> If a correction is needed on a locked record, click <strong>Amend</strong> — you must enter your initials and a written reason.</p>
        <p><strong>6.</strong> To invalidate a record, use <strong>Void</strong> — requires initials and reason. The record remains for audit purposes.</p>
      </div>

      <h4 class="font-semibold text-txt text-sm">Audit Trail</h4>
      <div class="text-txt-secondary text-sm space-y-1 ml-1">
        <p>Every action on a COC record is logged in the audit trail:</p>
        <ul class="ml-4 list-disc text-xs space-y-0.5 text-txt-muted">
          <li><strong>Insert:</strong> Record created with initial values</li>
          <li><strong>Update:</strong> Draft record edited (free edits, no initials required)</li>
          <li><strong>Lock:</strong> Record locked by examiner — becomes immutable</li>
          <li><strong>Amend:</strong> Locked record corrected — original value + new value + initials + reason preserved</li>
          <li><strong>Void:</strong> Record invalidated — reason + initials preserved</li>
          <li><strong>Transfer:</strong> Evidence transfer between parties documented</li>
        </ul>
      </div>
    </div>

    <div class="p-3 bg-warning/10 border border-warning/20 rounded-lg text-sm">
      <p class="text-warning font-medium mb-1">Best Practice</p>
      <p class="text-txt-secondary text-xs">
        Lock COC records <strong>as soon as they are verified</strong>. Leaving records in Draft status risks accidental edits.
        Once locked, the amendment process creates a defensible paper trail — courts and opposing counsel can see exactly 
        what was changed, when, by whom, and why.
      </p>
    </div>
  </div>
);
