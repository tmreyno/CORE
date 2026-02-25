// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * CaseInfoStep - First wizard step for case information entry
 */

import { For, Show, createMemo } from "solid-js";
import { HiOutlineClipboardDocument, HiOutlineCalendarDays } from "../../../icons";
import { CLASSIFICATIONS, INVESTIGATION_TYPES, REPORT_PRESETS, REPORT_TYPES, REPORT_TYPE_DEFAULTS } from "../../constants";
import type { ReportPreset } from "../../constants";
import { useWizard } from "../WizardContext";
import type { Classification } from "../../types";

export function CaseInfoStep() {
  const ctx = useWizard();

  /** Report-type-specific defaults for conditional field visibility */
  const typeDefaults = createMemo(() => REPORT_TYPE_DEFAULTS[ctx.reportType()]);

  /** Human-readable report type label */
  const reportTypeLabel = createMemo(() =>
    REPORT_TYPES.find((t) => t.value === ctx.reportType())?.label || "Report"
  );

  return (
    <div class="space-y-5">
      {/* Preset Selector */}
      <div class="flex items-center gap-3 p-3 bg-surface/50 rounded-xl border border-border/30">
        <span class="text-xl">{ctx.currentPreset()?.icon || "📋"}</span>
        <div class="flex-1 min-w-0">
          <label class="text-sm font-medium block">Report Preset</label>
          <p class="text-xs text-txt/50">Pre-configure settings based on investigation type</p>
        </div>
        <select
          class="input-sm w-48"
          value={ctx.selectedPreset()}
          onChange={(e) => ctx.applyPreset(e.currentTarget.value as ReportPreset)}
        >
          <For each={REPORT_PRESETS}>
            {(preset) => (
              <option value={preset.id}>{preset.name}</option>
            )}
          </For>
        </select>
      </div>

      {/* Section Header */}
      <div class="flex items-center gap-2">
        <HiOutlineClipboardDocument class="w-5 h-5 text-accent" />
        <h3 class="text-base font-semibold">Case Information</h3>
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="label">Case Number *</label>
          <input
            type="text"
            class="input"
            value={ctx.caseInfo().case_number}
            onInput={(e) => ctx.setCaseInfo({ ...ctx.caseInfo(), case_number: e.currentTarget.value })}
            placeholder="e.g., 2026-CF-00123"
          />
        </div>

        <div>
          <label class="label">Case Name</label>
          <input
            type="text"
            class="input"
            value={ctx.caseInfo().case_name || ""}
            onInput={(e) => ctx.setCaseInfo({ ...ctx.caseInfo(), case_name: e.currentTarget.value || undefined })}
            placeholder="e.g., State v. John Doe"
          />
        </div>

        <div>
          <label class="label">Agency/Department</label>
          <input
            type="text"
            class="input"
            value={ctx.caseInfo().agency || ""}
            onInput={(e) => ctx.setCaseInfo({ ...ctx.caseInfo(), agency: e.currentTarget.value || undefined })}
            placeholder="e.g., Metro Police Department"
          />
        </div>

        <div>
          <label class="label">Requestor</label>
          <input
            type="text"
            class="input"
            value={ctx.caseInfo().requestor || ""}
            onInput={(e) => ctx.setCaseInfo({ ...ctx.caseInfo(), requestor: e.currentTarget.value || undefined })}
            placeholder="e.g., Det. Jane Smith"
          />
        </div>

        <Show when={typeDefaults().showInvestigationType}>
          <div>
            <label class="label">Investigation Type</label>
            <select
              class="input"
              value={ctx.caseInfo().investigation_type || ""}
              onChange={(e) => ctx.setCaseInfo({ ...ctx.caseInfo(), investigation_type: e.currentTarget.value || undefined })}
            >
            <option value="">Select type...</option>
            <For each={INVESTIGATION_TYPES}>
              {(t) => <option value={t.value}>{t.label}</option>}
            </For>
          </select>
        </div>
        </Show>

        <div>
          <label class="label">Classification</label>
          <select
            class="input"
            value={ctx.metadata().classification}
            onChange={(e) => ctx.setMetadata({ ...ctx.metadata(), classification: e.currentTarget.value as Classification })}
          >
            <For each={CLASSIFICATIONS}>
              {(c) => <option value={c.value}>{c.label}</option>}
            </For>
          </select>
        </div>
      </div>

      {/* Dates Section */}
      <div class="flex items-center gap-2 mt-2">
        <HiOutlineCalendarDays class="w-4 h-4 text-accent/70" />
        <h4 class="text-sm font-medium text-txt/70">Dates</h4>
      </div>

      <div class="grid grid-cols-3 gap-4">
        <div>
          <label class="label">Request Date</label>
          <input
            type="date"
            class="input"
            value={ctx.caseInfo().request_date || ""}
            onInput={(e) => ctx.setCaseInfo({ ...ctx.caseInfo(), request_date: e.currentTarget.value || undefined })}
          />
        </div>

        <div>
          <label class="label">Exam Start Date</label>
          <input
            type="date"
            class="input"
            value={ctx.caseInfo().exam_start_date || ""}
            onInput={(e) => ctx.setCaseInfo({ ...ctx.caseInfo(), exam_start_date: e.currentTarget.value || undefined })}
          />
        </div>

        <div>
          <label class="label">Exam End Date</label>
          <input
            type="date"
            class="input"
            value={ctx.caseInfo().exam_end_date || ""}
            onInput={(e) => ctx.setCaseInfo({ ...ctx.caseInfo(), exam_end_date: e.currentTarget.value || undefined })}
          />
        </div>
      </div>

      <Show when={typeDefaults().showDescription}>
        <div>
          <label class="label">Case Description</label>
          <textarea
            class="textarea h-24"
            value={ctx.caseInfo().description || ""}
            onInput={(e) => ctx.setCaseInfo({ ...ctx.caseInfo(), description: e.currentTarget.value || undefined })}
            placeholder={typeDefaults().descriptionPlaceholder}
          />
        </div>
      </Show>

      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="label">Report Title</label>
          <input
            type="text"
            class="input"
            value={ctx.metadata().title}
            onInput={(e) => {
              ctx.setTitleManuallyEdited(true);
              ctx.setMetadata({ ...ctx.metadata(), title: e.currentTarget.value });
            }}
          />
          <p class="text-xs text-txt/40 mt-1">Auto-set from report type: {reportTypeLabel()}</p>
        </div>

        <div>
          <label class="label">Report Number</label>
          <input
            type="text"
            class="input"
            value={ctx.metadata().report_number}
            onInput={(e) => {
              ctx.setReportNumberManuallyEdited(true);
              ctx.setMetadata({ ...ctx.metadata(), report_number: e.currentTarget.value });
            }}
          />
          <p class="text-xs text-txt/40 mt-1">Auto-generated unique number. Editable if needed.</p>
        </div>
      </div>
    </div>
  );
}
