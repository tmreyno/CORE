// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * MergingStep — Step 3 spinner while the merge is running.
 */

import { Component } from "solid-js";

export interface MergingStepProps {
  projectCount: number;
}

export const MergingStep: Component<MergingStepProps> = (props) => {
  return (
    <div class="flex flex-col items-center justify-center py-12 gap-4">
      <div class="w-8 h-8 border-2 border-accent border-t-transparent rounded-full animate-spin" />
      <p class="text-sm text-txt-secondary">Merging {props.projectCount} projects…</p>
      <p class="text-xs text-txt-muted">This may take a moment for large databases.</p>
    </div>
  );
};
