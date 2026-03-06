// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * WizardStepIndicator — Step progress indicator for the Project Setup Wizard.
 *
 * Shows numbered steps with active/complete states. Adapts between the
 * "select folder" flow (step -1) and the "scan → configure → hash" flow (step >= 0).
 */

import { type Accessor, Show } from "solid-js";

export interface WizardStepIndicatorProps {
  step: Accessor<number>;
  showHashLoadingStep: Accessor<boolean>;
}

export const WizardStepIndicator = (props: WizardStepIndicatorProps) => {
  return (
    <div class="wizard-steps">
      <Show when={props.step() === -1}>
        <div class="step active">
          <span class="step-number">1</span>
          <span class="step-label">Select Folder</span>
        </div>
        <div class="step-connector" />
        <div class="step">
          <span class="step-number">2</span>
          <span class="step-label">Scan</span>
        </div>
        <div class="step-connector" />
        <div class="step">
          <span class="step-number">3</span>
          <span class="step-label">Configure</span>
        </div>
      </Show>
      <Show when={props.step() >= 0}>
        <div class="step" classList={{ active: props.step() === 0, complete: props.step() > 0 }}>
          <span class="step-number">1</span>
          <span class="step-label">Scan</span>
        </div>
        <div class="step-connector" />
        <div class="step" classList={{ active: props.step() === 1, complete: props.step() > 1 }}>
          <span class="step-number">2</span>
          <span class="step-label">Configure</span>
        </div>
        <Show when={props.showHashLoadingStep()}>
          <div class="step-connector" />
          <div class="step" classList={{ active: props.step() === 2, complete: props.step() > 2 }}>
            <span class="step-number">3</span>
            <span class="step-label">Load Hashes</span>
          </div>
        </Show>
      </Show>
    </div>
  );
};
