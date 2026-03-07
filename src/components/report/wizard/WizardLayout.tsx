// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * WizardLayout - Main layout component with header, step navigation, and footer
 */

import { For, Show } from "solid-js";
import { HiOutlineClipboardDocumentList, HiOutlineXMark } from "../../icons";
import { useWizard } from "./WizardContext";
import {
  CaseInfoSchemaStep,
  ExaminerSchemaStep,
  EvidenceStep,
  FindingsStep,
  PreviewStep,
  ExportStep,
  ReportTypeStep,
  ReportDataStep,
} from "./steps";

interface WizardLayoutProps {
  onClose: () => void;
}

export function WizardLayout(props: WizardLayoutProps) {
  const ctx = useWizard();

  return (
    <>
      {/* Header */}
      <div class="flex items-center justify-between px-4 py-2.5 border-b border-border/50">
        <h2 id="report-wizard-title" class="text-sm font-semibold flex items-center gap-2">
          <div class="w-6 h-6 rounded-md bg-accent/10 flex items-center justify-center">
            <HiOutlineClipboardDocumentList class="w-3.5 h-3.5 text-accent" />
          </div>
          Generate Forensic Report
        </h2>
        <button
          class="text-txt-muted hover:text-txt hover:bg-bg-hover p-1 rounded-md transition-colors"
          onClick={props.onClose}
          aria-label="Close report wizard"
        >
          <HiOutlineXMark class="w-4 h-4" />
        </button>
      </div>

      {/* Step indicators - compact horizontal stepper */}
      <div class="px-4 py-1.5 border-b border-border/50 bg-surface/30">
        <div class="flex items-center justify-between">
          <For each={ctx.activeSteps()}>
            {(step, index) => {
              const stepIndex = () => ctx.activeSteps().findIndex(s => s.id === ctx.currentStep());
              const isActive = () => ctx.currentStep() === step.id;
              const isCompleted = () => index() < stepIndex();
              const isClickable = () => index() <= stepIndex() + 1;

              return (
                <>
                  <button
                    class={`flex items-center gap-1.5 px-2 py-1 rounded-md text-xs font-medium transition-all ${
                      isActive()
                        ? 'bg-accent text-white shadow-sm shadow-accent/25'
                        : isCompleted()
                          ? 'text-accent hover:bg-accent/10'
                          : 'text-txt-muted hover:bg-bg-hover'
                    } ${!isClickable() ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}`}
                    onClick={() => isClickable() && ctx.goToStep(step.id)}
                    disabled={!isClickable()}
                  >
                    <div class={`w-4.5 h-4.5 rounded-full flex items-center justify-center text-[10px] font-bold ${
                      isActive()
                        ? 'bg-accent/20 text-accent'
                        : isCompleted()
                          ? 'bg-accent/20 text-accent'
                          : 'bg-bg-hover'
                    }`}>
                      {isCompleted() ? '✓' : index() + 1}
                    </div>
                    <span class="hidden sm:inline">{step.label}</span>
                  </button>
                  <Show when={index() < ctx.activeSteps().length - 1}>
                    <div class={`flex-1 h-px mx-1 ${
                      isCompleted() ? 'bg-accent' : 'bg-border'
                    }`} />
                  </Show>
                </>
              );
            }}
          </For>
        </div>
      </div>

      {/* Content area */}
      <div class="flex-1 overflow-y-auto p-4">
        <Show when={ctx.currentStep() === "report_type"}>
          <ReportTypeStep />
        </Show>

        <Show when={ctx.currentStep() === "case"}>
          <CaseInfoSchemaStep />
        </Show>

        <Show when={ctx.currentStep() === "examiner"}>
          <ExaminerSchemaStep />
        </Show>

        <Show when={ctx.currentStep() === "evidence"}>
          <EvidenceStep />
        </Show>

        <Show when={ctx.currentStep() === "report_data"}>
          <ReportDataStep />
        </Show>

        <Show when={ctx.currentStep() === "findings"}>
          <FindingsStep />
        </Show>

        <Show when={ctx.currentStep() === "preview"}>
          <PreviewStep />
        </Show>

        <Show when={ctx.currentStep() === "export"}>
          <ExportStep />
        </Show>
      </div>

      {/* Footer navigation */}
      <div class="flex items-center justify-between px-4 py-2.5 border-t border-border/50 bg-surface/30">
        <button
          class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-all duration-200 bg-surface border border-border/50 text-txt/70 hover:text-txt hover:bg-bg-hover disabled:opacity-40 disabled:cursor-not-allowed"
          onClick={ctx.prevStep}
          disabled={!ctx.canGoPrev()}
        >
          <span>←</span>
          <span>Previous</span>
        </button>

        <span class="text-xs text-txt-muted">
          Step {ctx.activeSteps().findIndex(s => s.id === ctx.currentStep()) + 1} of {ctx.activeSteps().length}
        </span>

        <Show when={ctx.currentStep() !== "export"}>
          <button
            class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-all duration-200 bg-accent text-white hover:bg-accent/90 shadow-sm shadow-accent/25"
            onClick={ctx.nextStep}
          >
            <span>Next</span>
            <span>→</span>
          </button>
        </Show>

        <Show when={ctx.currentStep() === "export"}>
          <button
            class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-all duration-200 bg-surface border border-border/50 text-txt/70 hover:text-txt hover:bg-bg-hover"
            onClick={props.onClose}
          >
            Close
          </button>
        </Show>
      </div>
    </>
  );
}
