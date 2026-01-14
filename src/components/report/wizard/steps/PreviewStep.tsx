// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * PreviewStep - Fifth wizard step for report preview
 */

import { Show, For, createEffect } from "solid-js";
import DOMPurify from "dompurify";
import { HiOutlineArrowPath } from "../../../icons";
import { SEVERITIES } from "../../constants";
import { useWizard } from "../WizardContext";

export function PreviewStep() {
  const ctx = useWizard();

  // Generate preview when entering this step
  createEffect(() => {
    if (ctx.currentStep() === "preview") {
      ctx.generatePreview();
    }
  });

  return (
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-medium">Report Preview</h3>
        <button
          class="btn-action-secondary"
          onClick={ctx.generatePreview}
          disabled={ctx.previewLoading()}
        >
          {ctx.previewLoading() ? "Generating..." : "🔄 Refresh Preview"}
        </button>
      </div>

      {/* Report Statistics Summary */}
      <div class="grid grid-cols-4 gap-3">
        <div class="p-3 bg-surface border border-border rounded-lg text-center">
          <div class="text-2xl font-bold text-accent">{ctx.selectedEvidence().size}</div>
          <div class="text-xs text-txt-muted">Evidence Items</div>
        </div>
        <div class="p-3 bg-surface border border-border rounded-lg text-center">
          <div class="text-2xl font-bold text-accent">{ctx.findings().length}</div>
          <div class="text-xs text-txt-muted">Findings</div>
        </div>
        <div class="p-3 bg-surface border border-border rounded-lg text-center">
          <div class="text-2xl font-bold text-accent">{ctx.chainOfCustody().length}</div>
          <div class="text-xs text-txt-muted">Custody Records</div>
        </div>
        <div class="p-3 bg-surface border border-border rounded-lg text-center">
          <div class="text-2xl font-bold text-accent">
            {ctx.findings().filter(f => f.severity === "Critical" || f.severity === "High").length}
          </div>
          <div class="text-xs text-txt-muted">Critical/High</div>
        </div>
      </div>

      {/* Severity Breakdown */}
      <Show when={ctx.findings().length > 0}>
        <div class="p-3 bg-surface border border-border rounded-lg">
          <h4 class="text-sm font-medium mb-2">Finding Severity Breakdown</h4>
          <div class="flex gap-4">
            <For each={SEVERITIES}>
              {(sev) => {
                const count = ctx.findings().filter(f => f.severity === sev.value).length;
                return (
                  <Show when={count > 0}>
                    <div class="flex items-center gap-2">
                      <div class="w-3 h-3 rounded-full" style={{ "background-color": sev.color }} />
                      <span class="text-sm">{sev.label}: {count}</span>
                    </div>
                  </Show>
                );
              }}
            </For>
          </div>
        </div>
      </Show>

      {/* Report Completeness Check */}
      <div class="p-3 bg-surface border border-border rounded-lg">
        <h4 class="text-sm font-medium mb-2">Report Completeness</h4>
        <div class="grid grid-cols-3 gap-2 text-sm">
          <div class="flex items-center gap-2">
            <span class={ctx.caseInfo().case_number ? "text-success" : "text-error"}>
              {ctx.caseInfo().case_number ? "✓" : "✗"}
            </span>
            Case Number
          </div>
          <div class="flex items-center gap-2">
            <span class={ctx.examiner().name ? "text-success" : "text-error"}>
              {ctx.examiner().name ? "✓" : "✗"}
            </span>
            Examiner Name
          </div>
          <div class="flex items-center gap-2">
            <span class={ctx.selectedEvidence().size > 0 ? "text-success" : "text-warning"}>
              {ctx.selectedEvidence().size > 0 ? "✓" : "⚠"}
            </span>
            Evidence Selected
          </div>
          <div class="flex items-center gap-2">
            <span class={ctx.executiveSummary() ? "text-success" : "text-txt-muted"}>
              {ctx.executiveSummary() ? "✓" : "○"}
            </span>
            Executive Summary
          </div>
          <div class="flex items-center gap-2">
            <span class={ctx.methodology() ? "text-success" : "text-txt-muted"}>
              {ctx.methodology() ? "✓" : "○"}
            </span>
            Methodology
          </div>
          <div class="flex items-center gap-2">
            <span class={ctx.conclusions() ? "text-success" : "text-txt-muted"}>
              {ctx.conclusions() ? "✓" : "○"}
            </span>
            Conclusions
          </div>
        </div>
      </div>

      <Show when={ctx.previewLoading()}>
        <div class="flex items-center justify-center py-12">
          <HiOutlineArrowPath class="w-8 h-8 animate-spin text-accent" />
        </div>
      </Show>

      <Show when={!ctx.previewLoading() && ctx.previewHtml()}>
        <div
          class="border border-border rounded bg-white text-black p-4 max-h-[50vh] overflow-auto"
          innerHTML={DOMPurify.sanitize(ctx.previewHtml() || "")}
        />
      </Show>
    </div>
  );
}
