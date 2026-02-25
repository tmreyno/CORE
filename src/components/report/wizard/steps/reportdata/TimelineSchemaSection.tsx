// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * TimelineSchemaSection — JSON-schema-driven replacement for
 * TimelineFormSection.
 *
 * Uses the `timeline.json` form template rendered via SchemaFormRenderer,
 * while syncing data bidirectionally with the WizardContext signal
 * (timelineReportData, setTimelineReportData).
 *
 * The timeline has two data sections:
 *  - timeline_config: time range, included categories, narrative
 *  - events: repeatable TimelineEvent[]
 *
 * Note: key_events (highlight marking) is preserved from WizardContext
 * but not directly editable via the schema form (it's a toggle action
 * on individual events that would need custom UI).
 */

import { createEffect, on } from "solid-js";
import { useWizard } from "../../WizardContext";
import { useFormTemplate } from "../../../../../templates/useFormTemplate";
import { useFormPersistence } from "../../../../../templates/useFormPersistence";
import { SchemaFormRenderer } from "../../../../../templates/SchemaFormRenderer";
import type { FormData } from "../../../../../templates/types";
import type { TimelineReportData, TimelineEvent } from "../../../types";

export function TimelineSchemaSection() {
  const ctx = useWizard();

  const form = useFormTemplate({
    templateId: "timeline",
    initialData: wizardToFormData(),
  });

  // --- Convert WizardContext signals → flat FormData ---
  function wizardToFormData(): FormData {
    const d = ctx.timelineReportData();
    return {
      // timeline_config fields
      time_range_start: d.time_range_start || "",
      time_range_end: d.time_range_end || "",
      included_categories: d.included_categories || [],
      narrative: d.narrative || "",
      // events repeatable section → FormData[]
      events: timelineEventsToFormData(d.events),
    };
  }

  /** Convert typed TimelineEvent[] → FormData[] */
  function timelineEventsToFormData(events: TimelineEvent[]): FormData[] {
    return events.map((ev, idx) => ({
      id: `event-${idx}-${ev.timestamp}`,
      timestamp: ev.timestamp || "",
      event_type: ev.event_type || "analysis",
      description: ev.description || "",
      source: ev.source || "",
      evidence_ref: ev.evidence_ref || "",
      artifact_path: ev.artifact_path || "",
    }));
  }

  /** Convert FormData[] back to typed TimelineEvent[] */
  function formDataToTimelineEvents(items: FormData[]): TimelineEvent[] {
    return items.map((fd) => ({
      timestamp: (fd.timestamp as string) || "",
      event_type: (fd.event_type as string) || "analysis",
      description: (fd.description as string) || "",
      source: (fd.source as string) || "",
      evidence_ref: (fd.evidence_ref as string) || undefined,
      artifact_path: (fd.artifact_path as string) || undefined,
    }));
  }

  // --- Sync: form data → WizardContext ---
  createEffect(
    on(
      () => form.data(),
      (fd) => {
        if (!fd) return;

        const rawEvents = Array.isArray(fd.events)
          ? (fd.events as FormData[])
          : [];

        // Parse included_categories — multiselect stores as string[]
        const categories = Array.isArray(fd.included_categories)
          ? (fd.included_categories as string[])
          : [];

        const tlData: TimelineReportData = {
          time_range_start: (fd.time_range_start as string) || undefined,
          time_range_end: (fd.time_range_end as string) || undefined,
          included_categories: categories,
          events: formDataToTimelineEvents(rawEvents),
          key_events: ctx.timelineReportData().key_events, // preserve existing
          narrative: (fd.narrative as string) || undefined,
        };

        ctx.setTimelineReportData(tlData);
      },
      { defer: true }
    )
  );

  // --- Auto-save to .ffxdb ---
  useFormPersistence({
    templateId: "timeline",
    templateVersion: "1.0.0",
    caseNumber: () => ctx.caseInfo().case_number || undefined,
    data: form.data,
  });

  return <SchemaFormRenderer form={form} />;
}
