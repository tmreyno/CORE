// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";

interface ProgressModalProps {
  show: boolean;
  title: string;
  message: string;
  current: number;
  total: number;
  onCancel?: () => void;
}

export function ProgressModal(props: ProgressModalProps) {
  const percent = () => props.total > 0 ? Math.round((props.current / props.total) * 100) : 0;
  
  return (
    <Show when={props.show}>
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-bg-card border border-border rounded-lg p-6 min-w-[320px] max-w-[480px] shadow-xl">
          <h3 class="text-lg font-semibold text-txt mb-2">{props.title}</h3>
          <p class="text-sm text-txt-muted mb-4">{props.message}</p>
          
          {/* Progress bar */}
          <div class="h-2 bg-bg rounded-full overflow-hidden mb-2">
            <div 
              class="h-full bg-accent transition-all duration-200 ease-out"
              style={{ width: `${percent()}%` }}
            />
          </div>
          
          {/* Progress text */}
          <div class="flex justify-between text-xs text-txt-muted mb-4">
            <span>{props.current} / {props.total}</span>
            <span>{percent()}%</span>
          </div>
          
          {/* Cancel button */}
          <Show when={props.onCancel}>
            <div class="flex justify-end">
              <button 
                class="px-4 py-1.5 text-sm bg-bg-hover border border-border rounded text-txt hover:bg-error hover:border-error hover:text-white transition-colors"
                onClick={props.onCancel}
              >
                Cancel
              </button>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
}
