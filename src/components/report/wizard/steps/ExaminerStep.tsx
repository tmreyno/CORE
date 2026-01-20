// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ExaminerStep - Second wizard step for examiner information
 */

import { For, Show } from "solid-js";
import { HiOutlineUser, HiOutlineXMark } from "../../../icons";
import { useWizard } from "../WizardContext";

export function ExaminerStep() {
  const ctx = useWizard();

  return (
    <div class="space-y-5">
      <div class="flex items-center gap-2">
        <HiOutlineUser class="w-5 h-5 text-accent" />
        <h3 class="text-base font-semibold">Examiner Information</h3>
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="label">Full Name *</label>
          <input
            type="text"
            class="input"
            value={ctx.examiner().name}
            onInput={(e) => ctx.setExaminer({ ...ctx.examiner(), name: e.currentTarget.value })}
            placeholder="e.g., John Smith"
          />
        </div>

        <div>
          <label class="label">Title</label>
          <input
            type="text"
            class="input"
            value={ctx.examiner().title || ""}
            onInput={(e) => ctx.setExaminer({ ...ctx.examiner(), title: e.currentTarget.value || undefined })}
            placeholder="e.g., Senior Digital Forensic Examiner"
          />
        </div>

        <div>
          <label class="label">Organization</label>
          <input
            type="text"
            class="input"
            value={ctx.examiner().organization || ""}
            onInput={(e) => ctx.setExaminer({ ...ctx.examiner(), organization: e.currentTarget.value || undefined })}
            placeholder="e.g., Metro Police Forensic Lab"
          />
        </div>

        <div>
          <label class="label">Badge/ID Number</label>
          <input
            type="text"
            class="input"
            value={ctx.examiner().badge_number || ""}
            onInput={(e) => ctx.setExaminer({ ...ctx.examiner(), badge_number: e.currentTarget.value || undefined })}
            placeholder="e.g., F-1234"
          />
        </div>

        <div>
          <label class="label">Email</label>
          <input
            type="email"
            class="input"
            value={ctx.examiner().email || ""}
            onInput={(e) => ctx.setExaminer({ ...ctx.examiner(), email: e.currentTarget.value || undefined })}
            placeholder="e.g., jsmith@agency.gov"
          />
        </div>

        <div>
          <label class="label">Phone</label>
          <input
            type="tel"
            class="input"
            value={ctx.examiner().phone || ""}
            onInput={(e) => ctx.setExaminer({ ...ctx.examiner(), phone: e.currentTarget.value || undefined })}
            placeholder="e.g., (555) 123-4567"
          />
        </div>
      </div>

      {/* Certifications with improved UI */}
      <div class="card">
        <label class="label">Certifications</label>
        <div class="flex gap-2 mb-3">
          <input
            type="text"
            class="input-sm flex-1"
            value={ctx.newCert()}
            onInput={(e) => ctx.setNewCert(e.currentTarget.value)}
            onKeyDown={(e) => e.key === "Enter" && ctx.addCertification()}
            placeholder="e.g., EnCE, GCFE, ACE..."
          />
          <button
            class="px-4 py-2 rounded-lg text-sm font-medium bg-accent/10 text-accent hover:bg-accent/20 transition-colors"
            onClick={ctx.addCertification}
          >
            + Add
          </button>
        </div>
        <div class="flex flex-wrap gap-2">
          <For each={ctx.examiner().certifications}>
            {(cert) => (
              <span class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-accent/10 text-accent rounded-full text-sm font-medium">
                {cert}
                <button
                  class="w-4 h-4 rounded-full bg-accent/20 hover:bg-accent/30 flex items-center justify-center text-xs transition-colors"
                  onClick={() => ctx.removeCertification(cert)}
                >
                  <HiOutlineXMark class="w-3 h-3" />
                </button>
              </span>
            )}
          </For>
          <Show when={ctx.examiner().certifications.length === 0}>
            <span class="text-sm text-txt/40 italic">No certifications added</span>
          </Show>
        </div>
      </div>
    </div>
  );
}
