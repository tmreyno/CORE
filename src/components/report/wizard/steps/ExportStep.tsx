// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ExportStep - Final wizard step for report export
 */

import { For, Show } from "solid-js";
import {
  HiOutlineDocument,
  HiOutlineDocumentCheck,
  HiOutlineUser,
  HiOutlineUserGroup,
  HiOutlineCheckCircle,
  HiOutlineXCircle,
  HiOutlineArrowUpTray,
  HiOutlineExclamationTriangle,
} from "../../../icons";
import { useWizard } from "../WizardContext";

export function ExportStep() {
  const ctx = useWizard();

  return (
    <div class="space-y-4">
      <h3 class="text-lg font-medium">Export Report</h3>

      <div class="grid grid-cols-2 gap-3">
        <For each={ctx.outputFormats()}>
          {(format) => (
            <button
              class={`p-4 rounded border text-left transition-colors ${
                ctx.selectedFormat() === format.format
                  ? 'border-accent bg-accent/10'
                  : format.supported
                    ? 'border-border hover:border-accent/50'
                    : 'border-border/30 hover:border-accent/30 opacity-50 cursor-not-allowed'
              }`}
              onClick={() => format.supported && ctx.setSelectedFormat(format.format)}
              disabled={!format.supported}
            >
              <div class="flex items-center gap-2 mb-1">
                <HiOutlineDocument class="w-5 h-5" />
                <span class="font-medium">{format.name}</span>
                <span class="text-xs text-text/50">.{format.extension}</span>
              </div>
              <p class="text-xs text-text/50">{format.description}</p>
              <Show when={!format.supported}>
                <span class="text-xs text-warning mt-1 block">Coming soon</span>
              </Show>
            </button>
          )}
        </For>
      </div>

      <Show when={ctx.exportError()}>
        <div class="p-3 bg-error/10 border border-error/30 rounded-lg text-error text-sm">
          Export failed: {ctx.exportError()}
        </div>
      </Show>

      {/* Signature & Approval Section */}
      <SignatureSection />

      {/* Export Button */}
      <div class="pt-5 border-t border-border/30">
        <button
          class="w-full flex items-center justify-center gap-2 px-4 py-3 rounded-xl text-sm font-semibold transition-all duration-200 bg-accent text-white hover:bg-accent/90 shadow-lg shadow-accent/25 disabled:opacity-50 disabled:cursor-not-allowed disabled:shadow-none"
          onClick={ctx.exportReport}
          disabled={ctx.exporting() || !ctx.caseInfo().case_number || !ctx.examiner().name}
        >
          <HiOutlineArrowUpTray class="w-5 h-5" />
          {ctx.exporting() ? "Exporting..." : `Export as ${ctx.outputFormats().find(f => f.format === ctx.selectedFormat())?.name || ctx.selectedFormat()}`}
        </button>

        <Show when={!ctx.caseInfo().case_number || !ctx.examiner().name}>
          <p class="text-sm text-warning flex items-center justify-center gap-2 mt-3">
            <HiOutlineExclamationTriangle class="w-4 h-4" />
            Please fill in required fields: Case Number and Examiner Name
          </p>
        </Show>
      </div>
    </div>
  );
}

/**
 * Signature & Approval sub-section
 */
function SignatureSection() {
  const ctx = useWizard();

  return (
    <div class="space-y-4 pt-5 border-t border-border/30">
      <div class="flex items-center gap-2">
        <HiOutlineDocumentCheck class="w-5 h-5 text-accent" />
        <h4 class="text-sm font-semibold">Signature & Approval</h4>
      </div>

      {/* Examiner Signature */}
      <div class="p-4 bg-surface/50 rounded-xl border border-border/30 space-y-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <HiOutlineUser class="w-4 h-4 text-accent" />
            <span class="text-sm font-medium">Examiner Signature</span>
          </div>
          <Show when={ctx.examiner().name && !ctx.examinerSignature()}>
            <button
              class="text-xs px-2 py-1 text-accent bg-accent/10 rounded-md hover:bg-accent/20 transition-colors"
              onClick={() => {
                ctx.setExaminerSignature(ctx.examiner().name);
                ctx.setExaminerSignedDate(new Date().toISOString().slice(0, 16));
              }}
            >
              Auto-fill
            </button>
          </Show>
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="label">Digital Signature</label>
            <input
              type="text"
              class="input-sm italic"
              placeholder="Type your full name"
              value={ctx.examinerSignature()}
              onInput={(e) => ctx.setExaminerSignature(e.currentTarget.value)}
            />
          </div>
          <div>
            <label class="label">Date Signed</label>
            <input
              type="datetime-local"
              class="input-sm"
              value={ctx.examinerSignedDate()}
              onInput={(e) => ctx.setExaminerSignedDate(e.currentTarget.value)}
            />
          </div>
        </div>
      </div>

      {/* Supervisor Approval */}
      <div class="card space-y-3">
        <div class="flex items-center gap-2">
          <HiOutlineUserGroup class="w-4 h-4 text-warning" />
          <span class="text-sm font-medium">Supervisor Approval</span>
          <span class="badge badge-accent/30">Optional</span>
        </div>
        <div>
          <label class="label">Supervisor Name</label>
          <input
            type="text"
            class="input-sm"
            placeholder="Supervisor's full name"
            value={ctx.supervisorName()}
            onInput={(e) => ctx.setSupervisorName(e.currentTarget.value)}
          />
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="label">Supervisor Signature</label>
            <input
              type="text"
              class="input-sm italic"
              placeholder="Type name as signature"
              value={ctx.supervisorSignature()}
              onInput={(e) => ctx.setSupervisorSignature(e.currentTarget.value)}
            />
          </div>
          <div>
            <label class="label">Date Approved</label>
            <input
              type="datetime-local"
              class="input-sm"
              value={ctx.supervisorSignedDate()}
              onInput={(e) => ctx.setSupervisorSignedDate(e.currentTarget.value)}
            />
          </div>
        </div>
        <div>
          <label class="label">Approval Notes</label>
          <textarea
            class="textarea text-sm"
            rows={2}
            placeholder="Any notes regarding approval..."
            value={ctx.approvalNotes()}
            onInput={(e) => ctx.setApprovalNotes(e.currentTarget.value)}
          />
        </div>
      </div>

      {/* Digital Signature Confirmation */}
      <label class="flex items-start gap-3 p-4 bg-accent/5 border-2 border-accent/20 rounded-xl cursor-pointer hover:bg-accent/10 hover:border-accent/30 transition-all">
        <div class={`w-5 h-5 rounded-md border-2 flex items-center justify-center mt-0.5 transition-colors ${
          ctx.digitalSignatureConfirmed() ? 'bg-accent border-accent' : 'border-accent/50'
        }`}>
          <Show when={ctx.digitalSignatureConfirmed()}>
            <span class="text-white text-xs font-bold">✓</span>
          </Show>
        </div>
        <input
          type="checkbox"
          class="sr-only"
          checked={ctx.digitalSignatureConfirmed()}
          onChange={(e) => ctx.setDigitalSignatureConfirmed(e.currentTarget.checked)}
        />
        <div>
          <span class="text-sm font-medium text-text">I confirm this report is accurate and complete</span>
          <p class="text-xs text-text/50 mt-1">
            By checking this box, I certify that all information contained in this forensic report
            is true and accurate to the best of my knowledge.
          </p>
        </div>
      </label>

      {/* Signature Status */}
      <div class="flex items-center justify-center gap-6 py-3 px-4 bg-surface/30 rounded-xl">
        <div class={`flex items-center gap-2 text-sm ${ctx.examinerSignature() ? 'text-success' : 'text-text/30'}`}>
          {ctx.examinerSignature() ? <HiOutlineCheckCircle class="w-5 h-5" /> : <HiOutlineXCircle class="w-5 h-5" />}
          <span>Examiner</span>
        </div>
        <div class="w-px h-4 bg-border/50" />
        <div class={`flex items-center gap-2 text-sm ${ctx.supervisorSignature() ? 'text-success' : 'text-text/30'}`}>
          {ctx.supervisorSignature() ? <HiOutlineCheckCircle class="w-5 h-5" /> : <HiOutlineXCircle class="w-5 h-5" />}
          <span>Supervisor</span>
        </div>
        <div class="w-px h-4 bg-border/50" />
        <div class={`flex items-center gap-2 text-sm ${ctx.digitalSignatureConfirmed() ? 'text-success' : 'text-text/30'}`}>
          {ctx.digitalSignatureConfirmed() ? <HiOutlineCheckCircle class="w-5 h-5" /> : <HiOutlineXCircle class="w-5 h-5" />}
          <span>Certified</span>
        </div>
      </div>
    </div>
  );
}
