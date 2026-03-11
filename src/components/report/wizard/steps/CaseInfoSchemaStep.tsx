// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * CaseInfoSchemaStep — JSON-schema-driven replacement for CaseInfoStep.
 *
 * Uses the `case_info.json` form template rendered via SchemaFormRenderer,
 * while syncing data bidirectionally with the WizardContext signals
 * (caseInfo, setCaseInfo, metadata, setMetadata).
 *
 * The preset selector remains a custom element outside the JSON template
 * because it drives report-type-specific behavior across the entire wizard.
 */

import { createEffect, For, on } from "solid-js";
import { useWizard } from "../WizardContext";
import { useFormTemplate } from "../../../../templates/useFormTemplate";
import { useFormPersistence } from "../../../../templates/useFormPersistence";
import { SchemaFormRenderer } from "../../../../templates/SchemaFormRenderer";
import { REPORT_PRESETS } from "../../constants";
import type { ReportPreset } from "../../constants";
import type { Classification } from "../../types";
import type { FormData } from "../../../../templates/types";

export function CaseInfoSchemaStep() {
  const ctx = useWizard();

  // --- Initialize the schema-driven form ---
  const form = useFormTemplate({
    templateId: "case_info",
    context: {
      // Inject _report_type so show_when conditions work
      _report_type: ctx.reportType(),
    },
    initialData: wizardToFormData(),
  });

  // --- Seed form data from WizardContext signals ---
  /** Convert current WizardContext signals into flat FormData for the template */
  function wizardToFormData(): FormData {
    const ci = ctx.caseInfo();
    const meta = ctx.metadata();
    return {
      case_number: ci.case_number || "",
      case_name: ci.case_name || "",
      agency: ci.agency || "",
      requestor: ci.requestor || "",
      investigation_type: ci.investigation_type || "",
      classification: meta.classification || "LawEnforcementSensitive",
      request_date: ci.request_date || "",
      exam_start_date: ci.exam_start_date || "",
      exam_end_date: ci.exam_end_date || "",
      description: ci.description || "",
      report_title: meta.title || "",
      report_number: meta.report_number || "",
    };
  }

  // --- Sync: form data → WizardContext (on every form change) ---
  createEffect(
    on(
      () => form.data(),
      (fd) => {
        if (!fd) return;

        // Map form fields back to CaseInfo
        ctx.setCaseInfo({
          case_number: (fd.case_number as string) || "",
          case_name: (fd.case_name as string) || undefined,
          agency: (fd.agency as string) || undefined,
          requestor: (fd.requestor as string) || undefined,
          investigation_type: (fd.investigation_type as string) || undefined,
          request_date: (fd.request_date as string) || undefined,
          exam_start_date: (fd.exam_start_date as string) || undefined,
          exam_end_date: (fd.exam_end_date as string) || undefined,
          description: (fd.description as string) || undefined,
        });

        // Map form fields back to ReportMetadata
        const currentMeta = ctx.metadata();
        ctx.setMetadata({
          ...currentMeta,
          classification: (fd.classification as Classification) || currentMeta.classification,
          title: (fd.report_title as string) || currentMeta.title,
          report_number: (fd.report_number as string) || currentMeta.report_number,
        });
      },
      { defer: true }
    )
  );

  // --- Sync: context changes (report type switch) → form ---
  createEffect(
    on(
      () => ctx.reportType(),
      (rt) => {
        // Update the context value used for show_when conditions
        form.setValue("_report_type", rt);
      },
      { defer: true }
    )
  );

  // --- Auto-save to .ffxdb ---
  useFormPersistence({
    templateId: "case_info",
    templateVersion: "1.0.0",
    caseNumber: () => ctx.caseInfo().case_number || undefined,
    data: form.data,
  });

  return (
    <div class="space-y-3">
      {/* Preset Selector — stays outside the JSON template */}
      <div class="flex items-center gap-2.5 p-2.5 bg-surface/50 rounded-lg border border-border/30">
        <span class="text-base">{ctx.currentPreset()?.icon || "📋"}</span>
        <div class="flex-1 min-w-0">
          <label class="text-xs font-medium block">Report Preset</label>
          <p class="text-compact text-txt/40">Pre-configure based on investigation type</p>
        </div>
        <select
          class="input-sm w-44"
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

      {/* Schema-driven form body */}
      <SchemaFormRenderer form={form} />
    </div>
  );
}
