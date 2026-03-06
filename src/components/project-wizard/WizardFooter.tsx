// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * WizardFooter — Footer buttons for the Project Setup Wizard.
 *
 * Shows contextual actions per step: Cancel for step -1,
 * Skip/Cancel/Continue for step 1, Skip Loading for step 2.
 */

import { type Accessor, Show } from "solid-js";

export interface WizardFooterProps {
  step: Accessor<number>;
  scanning: Accessor<boolean>;
  onClose: () => void;
  handleSkip: () => void;
  handleContinue: () => void;
  cancelHashLoading: () => void;
}

export const WizardFooter = (props: WizardFooterProps) => {
  return (
    <div class="wizard-footer">
      <Show when={props.step() === -1}>
        <div class="footer-spacer" />
        <button class="btn-action-secondary" onClick={props.onClose}>
          Cancel
        </button>
      </Show>
      <Show when={props.step() === 1}>
        <button class="btn-action-ghost" onClick={props.handleSkip}>
          Skip
        </button>
        <div class="footer-spacer" />
        <button class="btn-action-secondary" onClick={props.onClose}>
          Cancel
        </button>
        <button
          class="btn-action-primary"
          onClick={props.handleContinue}
          disabled={props.scanning()}
        >
          {props.scanning() ? "Scanning..." : "Continue"}
        </button>
      </Show>
      <Show when={props.step() === 2}>
        <div class="footer-spacer" />
        <button class="btn-action-secondary" onClick={props.cancelHashLoading}>
          Skip Loading
        </button>
      </Show>
    </div>
  );
};
