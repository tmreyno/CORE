// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * MergeStrategyModal Component
 * 
 * Modal for selecting merge strategy when combining projects:
 * - PreferA, PreferB, KeepBoth, Skip, Manual options
 * - Radio button selection
 * - Cancel and confirm actions
 */

import { Component, For } from "solid-js";
import type { MergeStrategy } from "../../hooks/useProjectComparison";
import { mergeStrategies } from "./comparisonHelpers";

interface MergeStrategyModalProps {
  isOpen: boolean;
  selectedStrategy: MergeStrategy;
  onStrategyChange: (strategy: MergeStrategy) => void;
  onCancel: () => void;
  onConfirm: () => void;
}

export const MergeStrategyModal: Component<MergeStrategyModalProps> = (props) => {
  if (!props.isOpen) return null;

  return (
    <div class="modal-overlay" onClick={props.onCancel}>
      <div class="modal-content max-w-2xl w-full" onClick={(e) => e.stopPropagation()}>
        <div class="modal-header">
          <h3 class="text-lg font-semibold text-txt">
            Merge Projects - Strategy Selection
          </h3>
        </div>
        <div class="modal-body space-y-4">
          <p class="text-sm text-txt-secondary">
            Choose how to handle conflicts when merging:
          </p>
          <div class="space-y-2">
            <For each={mergeStrategies}>
              {(strategy) => (
                <label class="flex items-start gap-3 p-3 bg-bg rounded-md border border-border hover:bg-bg-hover cursor-pointer">
                  <input
                    type="radio"
                    name="merge-strategy"
                    value={strategy.value}
                    checked={props.selectedStrategy === strategy.value}
                    onChange={(e) =>
                      props.onStrategyChange(
                        e.currentTarget.value as MergeStrategy
                      )
                    }
                    class="mt-1"
                  />
                  <div class="flex-1">
                    <div class="font-medium text-txt">
                      {strategy.label}
                    </div>
                    <div class="text-sm text-txt-secondary">
                      {strategy.desc}
                    </div>
                  </div>
                </label>
              )}
            </For>
          </div>
        </div>
        <div class="modal-footer justify-end">
          <button
            onClick={props.onCancel}
            class="btn-sm"
          >
            Cancel
          </button>
          <button
            onClick={props.onConfirm}
            class="btn-sm-primary"
          >
            Merge Projects
          </button>
        </div>
      </div>
    </div>
  );
};
