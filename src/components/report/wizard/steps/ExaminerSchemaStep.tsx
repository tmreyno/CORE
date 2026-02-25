// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ExaminerSchemaStep — JSON-schema-driven replacement for ExaminerStep.
 *
 * Uses the `examiner_info.json` form template rendered via SchemaFormRenderer,
 * while syncing data bidirectionally with the WizardContext signals
 * (examiner, setExaminer).
 */

import { createEffect, on } from "solid-js";
import { useWizard } from "../WizardContext";
import { useFormTemplate } from "../../../../templates/useFormTemplate";
import { useFormPersistence } from "../../../../templates/useFormPersistence";
import { SchemaFormRenderer } from "../../../../templates/SchemaFormRenderer";
import type { FormData } from "../../../../templates/types";

export function ExaminerSchemaStep() {
  const ctx = useWizard();

  // --- Initialize the schema-driven form ---
  const form = useFormTemplate({
    templateId: "examiner_info",
    initialData: wizardToFormData(),
  });

  /** Convert current WizardContext examiner signal into flat FormData */
  function wizardToFormData(): FormData {
    const ex = ctx.examiner();
    return {
      name: ex.name || "",
      title: ex.title || "",
      organization: ex.organization || "",
      badge_number: ex.badge_number || "",
      email: ex.email || "",
      phone: ex.phone || "",
      certifications: ex.certifications || [],
    };
  }

  // --- Sync: form data → WizardContext (on every form change) ---
  createEffect(
    on(
      () => form.data(),
      (fd) => {
        if (!fd) return;

        // Parse certifications — comma_list field stores as string[]
        const certs = Array.isArray(fd.certifications)
          ? (fd.certifications as string[])
          : typeof fd.certifications === "string"
            ? (fd.certifications as string).split(",").map((s) => s.trim()).filter(Boolean)
            : [];

        ctx.setExaminer({
          name: (fd.name as string) || "",
          title: (fd.title as string) || undefined,
          organization: (fd.organization as string) || undefined,
          email: (fd.email as string) || undefined,
          phone: (fd.phone as string) || undefined,
          badge_number: (fd.badge_number as string) || undefined,
          certifications: certs,
        });
      },
      { defer: true }
    )
  );

  // --- Auto-save to .ffxdb ---
  useFormPersistence({
    templateId: "examiner_info",
    templateVersion: "1.0.0",
    caseNumber: () => ctx.caseInfo().case_number || undefined,
    data: form.data,
  });

  return <SchemaFormRenderer form={form} />;
}
