// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ReportTypeStep - First wizard step to select the report type
 */

import { For } from "solid-js";
import { REPORT_TYPES } from "../../constants";
import { useWizard } from "../WizardContext";
import type { ReportType } from "../../types";

export function ReportTypeStep() {
  const ctx = useWizard();

  return (
    <div class="space-y-5">
      <div class="flex items-center gap-3 mb-2">
        <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-accent/20 to-accent/5 flex items-center justify-center">
          <span class="text-xl">📄</span>
        </div>
        <div>
          <h3 class="text-base font-semibold">Select Report Type</h3>
          <p class="text-sm text-txt/60">Choose the type of forensic report to generate</p>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <For each={REPORT_TYPES}>
          {(rt) => {
            const isSelected = () => ctx.reportType() === rt.value;

            return (
              <button
                type="button"
                class={`group relative p-5 rounded-xl border-2 text-left transition-all duration-200 ${
                  isSelected()
                    ? "border-accent bg-accent/5 ring-2 ring-accent/20"
                    : "border-border/50 bg-surface/50 hover:border-accent/50 hover:bg-surface"
                }`}
                onClick={() => ctx.setReportType(rt.value as ReportType)}
              >
                <div class="flex items-start gap-3">
                  <span class="text-3xl">{rt.icon}</span>
                  <div class="flex-1 min-w-0">
                    <span class="font-semibold text-sm block">{rt.label}</span>
                    <p class="text-xs text-txt/50 mt-1 leading-relaxed">{rt.description}</p>
                  </div>
                </div>
                {isSelected() && (
                  <div class="absolute top-3 right-3 w-6 h-6 rounded-full bg-accent flex items-center justify-center">
                    <span class="text-white text-xs font-bold">✓</span>
                  </div>
                )}
              </button>
            );
          }}
        </For>
      </div>

      <div class="mt-4 p-4 rounded-lg bg-info/5 border border-info/20">
        <p class="text-sm text-txt/70">
          <span class="font-medium text-info">ℹ️ Tip:</span>{" "}
          The wizard steps will adjust based on your selection.
          You can always change this later by navigating back to this step.
        </p>
      </div>
    </div>
  );
}
