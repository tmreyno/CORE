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
          <p class="text-xs text-txt-muted mt-1">Case number, examiner, agency, dates, and authority. This header section identifies the investigation and the examining authority.</p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Evidence Items</div>
          <p class="text-xs text-txt-muted mt-1">Collected items with serial numbers, descriptions, and hash values. Each piece of evidence is catalogued with acquisition details.</p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Findings & Analysis</div>
          <p class="text-xs text-txt-muted mt-1">Detailed findings from examination with supporting evidence. Document what you found, where you found it, and its significance.</p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Chain of Custody</div>
          <p class="text-xs text-txt-muted mt-1">Immutable COC records with lock/amend/void lifecycle. Automatically pulled from project database — no manual re-entry required.</p>
        </div>
      </div>
    </div>

    <div class="space-y-3">
      <h4 class="font-semibold text-txt text-sm">How to Generate a Report</h4>
      <div class="text-txt-secondary text-sm space-y-1 ml-1">
        <p><strong>1.</strong> Press <Kbd keys="Cmd+P" muted /> or go to <strong>Tools → Generate Report</strong>.</p>
        <p><strong>2.</strong> The wizard walks through each section in order. Fill in the case information header first.</p>
        <p><strong>3.</strong> In the Evidence section, hash values and metadata are pre-populated from the project database.</p>
        <p><strong>4.</strong> Write your findings in the Analysis section — reference specific files, hashes, and locations.</p>
        <p><strong>5.</strong> Review the Chain of Custody section — records are pulled in automatically from your COC entries.</p>
        <p><strong>6.</strong> Preview the complete report before export.</p>
      </div>
    </div>

    <div class="p-3 bg-info/10 border border-info/20 rounded-lg text-sm">
      <p class="text-info font-medium mb-1">Tips for Effective Reports</p>
      <ul class="text-txt-secondary text-xs ml-4 list-disc space-y-0.5">
        <li>Bookmark important files during analysis — they can be referenced in the Findings section.</li>
        <li>Hash all evidence on intake and include the hash values in the Evidence Items section.</li>
        <li>Lock COC records before generating the report — locked records show the audit trail.</li>
        <li>Use clear, factual language in findings — describe what was found, not conclusions about intent.</li>
        <li>The report wizard auto-saves progress — you can close and reopen it without losing data.</li>
      </ul>
    </div>
  </div>
);
