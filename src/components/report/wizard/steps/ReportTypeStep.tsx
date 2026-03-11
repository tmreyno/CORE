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
    <div class="space-y-3">
      <div class="flex items-center gap-2 mb-1">
        <span class="text-base">📄</span>
        <div>
          <h3 class="text-sm font-semibold">Select Report Type</h3>
          <p class="text-xs text-txt/50">Choose the type of forensic report to generate</p>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
        <For each={REPORT_TYPES}>
          {(rt) => {
            const isSelected = () => ctx.reportType() === rt.value;

            return (
              <button
                type="button"
                class={`group relative p-3 rounded-lg border text-left transition-all duration-150 ${
                  isSelected()
                    ? "border-accent bg-accent/5 ring-1 ring-accent/20"
                    : "border-border/50 bg-surface/50 hover:border-accent/50 hover:bg-surface"
                }`}
                onClick={() => ctx.setReportType(rt.value as ReportType)}
              >
                <div class="flex items-start gap-2.5">
                  <span class="text-xl mt-0.5">{rt.icon}</span>
                  <div class="flex-1 min-w-0">
                    <span class="font-medium text-xs block">{rt.label}</span>
                    <p class="text-compact text-txt/50 mt-0.5 leading-relaxed">{rt.description}</p>
                  </div>
                </div>
                {isSelected() && (
                  <div class="absolute top-2 right-2 w-4.5 h-4.5 rounded-full bg-accent flex items-center justify-center">
                    <span class="text-white text-2xs font-bold">✓</span>
                  </div>
                )}
              </button>
            );
          }}
        </For>
      </div>

      <div class="p-2.5 rounded-md bg-info/5 border border-info/20">
        <p class="text-xs text-txt/60">
          <span class="font-medium text-info">ℹ️ Tip:</span>{" "}
          The wizard steps will adjust based on your selection.
        </p>
      </div>
    </div>
  );
}
