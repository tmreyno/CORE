// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, Show, type Component } from "solid-js";
import { useFocusTrap } from "../../hooks/useFocusTrap";

export type ProjectCloseModalStepStatus =
  | "pending"
  | "running"
  | "completed"
  | "warning"
  | "failed"
  | "skipped";

export interface ProjectCloseModalStep {
  id: string;
  label: string;
  detail: string;
  status: ProjectCloseModalStepStatus;
}

interface ProjectCloseModalProps {
  show: boolean;
  title: string;
  message: string;
  steps: ProjectCloseModalStep[];
  error?: string | null;
  onDismiss?: () => void;
}

function StepIndicator(props: { status: ProjectCloseModalStepStatus }) {
  return (
    <span
      class="mt-0.5 flex h-5 w-5 items-center justify-center rounded-full border text-2xs font-semibold"
      classList={{
        "border-border text-txt-muted bg-bg": props.status === "pending" || props.status === "skipped",
        "border-info text-info bg-info/10": props.status === "running",
        "border-success text-success bg-success/10": props.status === "completed",
        "border-warning text-warning bg-warning/10": props.status === "warning",
        "border-error text-error bg-error/10": props.status === "failed",
      }}
      aria-hidden="true"
    >
      <Show when={props.status === "running"} fallback={
        <>
          <Show when={props.status === "completed"}>✓</Show>
          <Show when={props.status === "warning"}>!</Show>
          <Show when={props.status === "failed"}>×</Show>
          <Show when={props.status === "pending"}>•</Show>
          <Show when={props.status === "skipped"}>-</Show>
        </>
      }>
        <span class="h-2.5 w-2.5 animate-spin rounded-full border border-current border-t-transparent" />
      </Show>
    </span>
  );
}

export const ProjectCloseModal: Component<ProjectCloseModalProps> = (props) => {
  let modalRef: HTMLDivElement | undefined;

  useFocusTrap(() => modalRef, () => props.show);

  return (
    <Show when={props.show}>
      <div class="modal-overlay" role="dialog" aria-modal="true" aria-labelledby="project-close-modal-title">
        <div class="modal-content w-[540px] max-w-[92vw]" ref={modalRef}>
          <div class="modal-header">
            <h2 id="project-close-modal-title" class="text-base font-semibold text-txt">{props.title}</h2>
          </div>

          <div class="modal-body space-y-4">
            <p class="text-sm text-txt-muted">{props.message}</p>

            <div class="space-y-2" aria-live="polite">
              <For each={props.steps}>
                {(step) => (
                  <div class="flex gap-3 rounded-lg border border-border/60 bg-bg-secondary px-3 py-2.5">
                    <StepIndicator status={step.status} />
                    <div class="min-w-0 flex-1">
                      <div class="text-sm font-medium text-txt">{step.label}</div>
                      <div class="text-xs text-txt-muted">{step.detail}</div>
                    </div>
                  </div>
                )}
              </For>
            </div>

            <Show when={props.error}>
              <div class="rounded-lg border border-error/50 bg-error/10 px-3 py-2 text-sm text-error">
                {props.error}
              </div>
            </Show>
          </div>

          <Show when={props.onDismiss}>
            <div class="modal-footer justify-end">
              <button class="btn btn-secondary" onClick={() => props.onDismiss?.()}>
                Dismiss
              </button>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
};