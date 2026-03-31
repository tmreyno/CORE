// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import { CoreProgressBar } from "@core-suite/icons";
import { useFocusTrap } from "../hooks/useFocusTrap";

interface ProgressModalProps {
  show: boolean;
  title: string;
  message: string;
  current: number;
  total: number;
  onCancel?: () => void;
}

export function ProgressModal(props: ProgressModalProps) {
  let modalRef: HTMLDivElement | undefined;
  const percent = () => props.total > 0 ? Math.round((props.current / props.total) * 100) : 0;
  
  // Focus trap for modal accessibility
  useFocusTrap(() => modalRef, () => props.show);
  
  return (
    <Show when={props.show}>
      <div class="modal-overlay" role="dialog" aria-modal="true" aria-labelledby="progress-modal-title">
        <div class="modal-content min-w-[320px] max-w-[480px]" ref={modalRef}>
          <div class="modal-body">
            <h3 id="progress-modal-title" class="text-lg font-semibold text-txt mb-2">{props.title}</h3>
            <p class="text-sm text-txt-muted mb-4">{props.message}</p>
            
            {/* Progress bar */}
            <CoreProgressBar progress={percent()} height={8} class="mb-2" />
            
            {/* Progress text */}
            <div class="flex justify-between text-xs text-txt-muted mb-4" aria-live="polite">
              <span>{props.current} / {props.total}</span>
              <span>{percent()}%</span>
            </div>
            
            {/* Cancel button */}
            <Show when={props.onCancel}>
              <div class="flex justify-end">
                <button 
                  class="btn-sm"
                  onClick={props.onCancel}
                >
                  Cancel
                </button>
              </div>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  );
}
