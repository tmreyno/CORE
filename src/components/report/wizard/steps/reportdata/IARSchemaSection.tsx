// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * IARSchemaSection — JSON-schema-driven replacement for IARFormSection.
 *
 * Uses the `iar.json` form template rendered via SchemaFormRenderer,
 * while syncing data bidirectionally with the WizardContext signals
 * (iarSummary, setIarSummary, iarEntries, setIarEntries).
 *
 * The IAR has two data targets in WizardContext:
 *  - iarSummary: IARSummary (flat fields from investigation_summary section)
 *  - iarEntries: IAREntry[] (repeatable activity_entries section)
 */

import { createEffect, on } from "solid-js";
import { useWizard } from "../../WizardContext";
import { useFormTemplate } from "../../../../../templates/useFormTemplate";
import { useFormPersistence } from "../../../../../templates/useFormPersistence";
import { SchemaFormRenderer } from "../../../../../templates/SchemaFormRenderer";
import type { FormData } from "../../../../../templates/types";
import type { IAREntry, IAREventCategory, IARSummary } from "../../../types";

export function IARSchemaSection() {
  const ctx = useWizard();

  const form = useFormTemplate({
    templateId: "iar",
    initialData: wizardToFormData(),
  });

  // --- Convert WizardContext signals → flat FormData ---
  function wizardToFormData(): FormData {
    const s = ctx.iarSummary();
    return {
      // investigation_summary fields
      investigation_start: s.investigation_start || "",
      investigation_end: s.investigation_end || "",
      lead_examiner: s.lead_examiner || "",
      authorization: s.authorization || "",
      synopsis: s.synopsis || "",
      total_hours: s.total_hours ?? 0,
      // activity_entries repeatable section → FormData[]
      activity_entries: iarEntriesToFormData(ctx.iarEntries()),
    };
  }

  /** Convert typed IAREntry[] → FormData[] for the repeatable section */
  function iarEntriesToFormData(entries: IAREntry[]): FormData[] {
    return entries.map((e) => ({
      id: e.id,
      date: e.date || "",
      end_date: e.end_date || "",
      category: e.category || "analysis",
      personnel: e.personnel || "",
      personnel_role: e.personnel_role || "",
      description: e.description || "",
      evidence_refs: e.evidence_refs || [],
      location: e.location || "",
      hours_spent: e.hours_spent ?? 0,
      outcome: e.outcome || "",
      authorization_ref: e.authorization_ref || "",
      keywords: e.keywords || [],
      tools_used: e.tools_used || [],
      notes: e.notes || "",
    }));
  }

  /** Convert FormData[] back to typed IAREntry[] */
  function formDataToIarEntries(items: FormData[]): IAREntry[] {
    return items.map((fd) => ({
      id: (fd.id as string) || crypto.randomUUID(),
      date: (fd.date as string) || "",
      end_date: (fd.end_date as string) || undefined,
      category: ((fd.category as string) || "analysis") as IAREventCategory,
      personnel: (fd.personnel as string) || "",
      personnel_role: (fd.personnel_role as string) || undefined,
      description: (fd.description as string) || "",
      evidence_refs: parseCommaList(fd.evidence_refs),
      location: (fd.location as string) || undefined,
      hours_spent: typeof fd.hours_spent === "number" ? fd.hours_spent : parseFloat(fd.hours_spent as string) || undefined,
      outcome: (fd.outcome as string) || undefined,
      authorization_ref: (fd.authorization_ref as string) || undefined,
      keywords: parseCommaList(fd.keywords),
      tools_used: parseCommaList(fd.tools_used),
      notes: (fd.notes as string) || undefined,
    }));
  }

  /** Parse a comma_list field value (may be string[] or string) */
  function parseCommaList(val: unknown): string[] {
    if (Array.isArray(val)) return val as string[];
    if (typeof val === "string") return val.split(",").map((s) => s.trim()).filter(Boolean);
    return [];
  }

  // --- Sync: form data → WizardContext ---
  createEffect(
    on(
      () => form.data(),
      (fd) => {
        if (!fd) return;

        // Update iarSummary
        const summary: IARSummary = {
          investigation_start: (fd.investigation_start as string) || "",
          investigation_end: (fd.investigation_end as string) || undefined,
          lead_examiner: (fd.lead_examiner as string) || "",
          synopsis: (fd.synopsis as string) || "",
          authorization: (fd.authorization as string) || "",
          personnel_list: ctx.iarSummary().personnel_list, // preserve existing
          total_hours: typeof fd.total_hours === "number" ? fd.total_hours : parseFloat(fd.total_hours as string) || undefined,
        };
        ctx.setIarSummary(summary);

        // Update iarEntries
        const rawEntries = Array.isArray(fd.activity_entries)
          ? (fd.activity_entries as FormData[])
          : [];
        ctx.setIarEntries(formDataToIarEntries(rawEntries));
      },
      { defer: true }
    )
  );

  // --- Auto-save to .ffxdb ---
  useFormPersistence({
    templateId: "iar",
    templateVersion: "1.0.0",
    caseNumber: () => ctx.caseInfo().case_number || undefined,
    data: form.data,
  });

  return <SchemaFormRenderer form={form} />;
}
