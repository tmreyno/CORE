// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";
import { Kbd } from "../../ui/Kbd";

export const ReportsContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Generate forensic examination reports via the Report Wizard. 
      Open with <Kbd keys="Cmd+P" muted /> or <strong>Tools → Generate Report</strong>.
    </p>

    <div class="space-y-3">
      <h4 class="font-semibold text-txt text-sm">Report Sections</h4>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Case Information</div>
          <p class="text-xs text-txt-muted mt-1">Case number, examiner, agency, dates, and authority</p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Evidence Items</div>
          <p class="text-xs text-txt-muted mt-1">Collected items with serial numbers, descriptions, and hash values</p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Findings & Analysis</div>
          <p class="text-xs text-txt-muted mt-1">Detailed findings from examination with supporting evidence</p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Chain of Custody</div>
          <p class="text-xs text-txt-muted mt-1">Immutable COC records with lock/amend/void lifecycle</p>
        </div>
      </div>
    </div>
  </div>
);
