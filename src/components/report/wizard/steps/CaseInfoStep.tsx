// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * CaseInfoStep - First wizard step for case information entry
 */

import { For, Show } from "solid-js";
import { HiOutlineClipboardDocument, HiOutlineCalendarDays } from "../../../icons";
import { CLASSIFICATIONS, INVESTIGATION_TYPES } from "../../constants";
import { REPORT_TEMPLATES } from "../../templates";
import { useWizard } from "../WizardContext";
import type { Classification } from "../../types";

export function CaseInfoStep() {
  const ctx = useWizard();

  return (
    <div class="space-y-5">
      {/* Template Selector */}
      <Show when={ctx.showTemplateSelector()}>
        <div class="mb-6">
          <div class="flex items-center gap-3 mb-4">
            <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-accent/20 to-accent/5 flex items-center justify-center">
              <span class="text-xl">📋</span>
            </div>
            <div>
              <h3 class="text-base font-semibold">Choose Report Template</h3>
              <p class="text-sm text-txt/60">Pre-configure settings based on investigation type</p>
            </div>
          </div>
          <div class="grid grid-cols-2 lg:grid-cols-3 gap-3">
            <For each={REPORT_TEMPLATES}>
              {(template) => (
                <button
                  type="button"
                  class={`group relative p-4 rounded-xl border-2 text-left transition-all duration-200 ${
                    ctx.selectedTemplate() === template.id
                      ? "border-accent bg-accent/5 ring-2 ring-accent/20"
                      : "border-border/50 bg-surface/50 hover:border-accent/50 hover:bg-surface"
                  }`}
                  onClick={() => ctx.applyTemplate(template.id)}
                >
                  <div class="flex items-start gap-3">
                    <span class="text-2xl">{template.icon}</span>
                    <div class="flex-1 min-w-0">
                      <span class="font-medium text-sm block">{template.name}</span>
                      <p class="text-xs text-txt/50 mt-0.5 line-clamp-2">{template.description}</p>
                    </div>
                  </div>
                  <Show when={ctx.selectedTemplate() === template.id}>
                    <div class="absolute top-2 right-2 w-5 h-5 rounded-full bg-accent flex items-center justify-center">
                      <span class="text-white text-xs">✓</span>
                    </div>
                  </Show>
                </button>
              )}
            </For>
          </div>
          <div class="mt-4 flex justify-end">
            <button
              type="button"
              class="text-sm text-txt/50 hover:text-accent transition-colors"
              onClick={() => ctx.setShowTemplateSelector(false)}
            >
              Continue with {ctx.currentTemplate()?.name || "Custom"} →
            </button>
          </div>
        </div>
        <div class="border-t border-border/30 mb-5" />
      </Show>

      {/* Template Badge (when collapsed) */}
      <Show when={!ctx.showTemplateSelector()}>
        <div class="flex items-center justify-between p-3 bg-surface/50 rounded-xl border border-border/30">
          <div class="flex items-center gap-3">
            <span class="text-xl">{ctx.currentTemplate()?.icon || "📋"}</span>
            <div>
              <span class="text-sm font-medium">{ctx.currentTemplate()?.name || "Custom"} Template</span>
              <p class="text-xs text-txt/50">Pre-configured report settings</p>
            </div>
          </div>
          <button
            type="button"
            class="text-sm text-accent hover:underline px-3 py-1.5 rounded-lg hover:bg-accent/10 transition-colors"
            onClick={() => ctx.setShowTemplateSelector(true)}
          >
            Change
          </button>
        </div>
      </Show>

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

      <div>
        <label class="label">Case Description</label>
        <textarea
          class="textarea h-24"
          value={ctx.caseInfo().description || ""}
          onInput={(e) => ctx.setCaseInfo({ ...ctx.caseInfo(), description: e.currentTarget.value || undefined })}
          placeholder="Brief description of the case and examination request..."
        />
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="label">Report Title</label>
          <input
            type="text"
            class="input"
            value={ctx.metadata().title}
            onInput={(e) => ctx.setMetadata({ ...ctx.metadata(), title: e.currentTarget.value })}
          />
        </div>

        <div>
          <label class="label">Report Number</label>
          <input
            type="text"
            class="input"
            value={ctx.metadata().report_number}
            onInput={(e) => ctx.setMetadata({ ...ctx.metadata(), report_number: e.currentTarget.value })}
          />
        </div>
      </div>
    </div>
  );
}
