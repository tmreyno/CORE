// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from 'solid-js';

interface ScanningStepProps {
  scanMessage: () => string;
  error: () => string | null;
}

export const ScanningStep: Component<ScanningStepProps> = (props) => {
  return (
    <div class="scanning-state">
      <div class="spinner" />
      <p>{props.scanMessage()}</p>
      <Show when={props.error()}>
        <p class="error-text">{props.error()}</p>
      </Show>
    </div>
  );
};
