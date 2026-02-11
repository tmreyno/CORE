// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ComparisonSummaryBar Component
 * 
 * Summary statistics bar showing:
 * - Total items count
 * - Common items count
 * - Modified items count
 * - Conflicts count
 * - Similarity percentage with color coding
 */

import { Component, Show } from "solid-js";
import type { ProjectComparison } from "../../hooks/useProjectComparison";
import { getSimilarityColor } from "./comparisonHelpers";

interface ComparisonSummaryBarProps {
  comparison: ProjectComparison | undefined;
}

export const ComparisonSummaryBar: Component<ComparisonSummaryBarProps> = (props) => {
  return (
    <Show when={props.comparison}>
      {(comp) => (
        <div class="p-4 border-b border-border bg-bg">
          <div class="grid grid-cols-5 gap-4 text-center">
            <div>
              <div class="text-2xl font-bold text-accent">
                {comp().summary.unique_to_a + comp().summary.unique_to_b + comp().summary.common}
              </div>
              <div class="text-xs text-txt-muted">Total Items</div>
            </div>
            <div>
              <div class="text-2xl font-bold text-success">
                {comp().summary.common}
              </div>
              <div class="text-xs text-txt-muted">Common</div>
            </div>
            <div>
              <div class="text-2xl font-bold text-warning">
                {comp().summary.modified}
              </div>
              <div class="text-xs text-txt-muted">Modified</div>
            </div>
            <div>
              <div class="text-2xl font-bold text-error">
                {comp().conflicts.length}
              </div>
              <div class="text-xs text-txt-muted">Conflicts</div>
            </div>
            <div>
              <div
                class={`text-2xl font-bold ${getSimilarityColor(
                  comp().summary.similarity_percent
                )}`}
              >
                {comp().summary.similarity_percent.toFixed(0)}%
              </div>
              <div class="text-xs text-txt-muted">Similarity</div>
            </div>
          </div>
        </div>
      )}
    </Show>
  );
};
