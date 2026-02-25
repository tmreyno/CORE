// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * UserActivitySchemaSection — JSON-schema-driven replacement for
 * UserActivityFormSection.
 *
 * Uses the `user_activity.json` form template rendered via
 * SchemaFormRenderer, while syncing data bidirectionally with the
 * WizardContext signal (userActivityData, setUserActivityData).
 */

import { createEffect, on } from "solid-js";
import { useWizard } from "../../WizardContext";
import { useFormTemplate } from "../../../../../templates/useFormTemplate";
import { useFormPersistence } from "../../../../../templates/useFormPersistence";
import { SchemaFormRenderer } from "../../../../../templates/SchemaFormRenderer";
import type { FormData } from "../../../../../templates/types";
import type { UserActivityData, UserActivityEntry, Severity } from "../../../types";

export function UserActivitySchemaSection() {
  const ctx = useWizard();

  const form = useFormTemplate({
    templateId: "user_activity",
    initialData: wizardToFormData(),
  });

  // --- Convert WizardContext signals → flat FormData ---
  function wizardToFormData(): FormData {
    const d = ctx.userActivityData();
    return {
      // target_info fields
      target_user: d.target_user || "",
      user_aliases: d.user_aliases || [],
      time_range_start: d.time_range_start || "",
      time_range_end: d.time_range_end || "",
      summary: d.summary || "",
      // activity_entries repeatable section → FormData[]
      activity_entries: userActivityEntriesToFormData(d.activity_entries),
    };
  }

  /** Convert typed UserActivityEntry[] → FormData[] */
  function userActivityEntriesToFormData(entries: UserActivityEntry[]): FormData[] {
    return entries.map((e) => ({
      id: e.id,
      timestamp: e.timestamp || "",
      category: e.category || "general",
      description: e.description || "",
      source_artifact: e.source_artifact || "",
      evidence_ref: e.evidence_ref || "",
      user_account: e.user_account || "",
      application: e.application || "",
      significance: e.significance || "Informational",
      notes: e.notes || "",
    }));
  }

  /** Convert FormData[] back to typed UserActivityEntry[] */
  function formDataToUserActivityEntries(items: FormData[]): UserActivityEntry[] {
    return items.map((fd) => ({
      id: (fd.id as string) || crypto.randomUUID(),
      timestamp: (fd.timestamp as string) || "",
      category: (fd.category as string) || "general",
      description: (fd.description as string) || "",
      source_artifact: (fd.source_artifact as string) || "",
      evidence_ref: (fd.evidence_ref as string) || undefined,
      user_account: (fd.user_account as string) || undefined,
      application: (fd.application as string) || undefined,
      significance: ((fd.significance as string) || "Informational") as Severity,
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

        const rawEntries = Array.isArray(fd.activity_entries)
          ? (fd.activity_entries as FormData[])
          : [];

        const uaData: UserActivityData = {
          target_user: (fd.target_user as string) || "",
          user_aliases: parseCommaList(fd.user_aliases),
          time_range_start: (fd.time_range_start as string) || undefined,
          time_range_end: (fd.time_range_end as string) || undefined,
          activity_entries: formDataToUserActivityEntries(rawEntries),
          summary: (fd.summary as string) || undefined,
        };

        ctx.setUserActivityData(uaData);
      },
      { defer: true }
    )
  );

  // --- Auto-save to .ffxdb ---
  useFormPersistence({
    templateId: "user_activity",
    templateVersion: "1.0.0",
    caseNumber: () => ctx.caseInfo().case_number || undefined,
    data: form.data,
  });

  return <SchemaFormRenderer form={form} />;
}
