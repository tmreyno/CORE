// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * LoadingOverlay — lightweight toast-style indicator for async operations.
 * Use when an operation may take noticeable time but has no numeric progress.
 *
 * Usage:
 *   const [loading, withLoading] = createLoadingState();
 *   await withLoading("Scanning evidence…", () => scanForFiles());
 *   <LoadingOverlay {...loading} />
 */

import { Show, type Component, type Accessor } from "solid-js";

export interface LoadingState {
  isLoading: Accessor<boolean>;
  message: Accessor<string>;
  error: Accessor<string | null>;
}

export interface LoadingOverlayProps extends LoadingState {
  /** Position within the parent — default "bottom-right" */
  position?: "bottom-right" | "bottom-center" | "top-center";
}

const positionClasses: Record<string, string> = {
  "bottom-right": "fixed bottom-16 right-4",
  "bottom-center": "fixed bottom-16 left-1/2 -translate-x-1/2",
  "top-center": "fixed top-16 left-1/2 -translate-x-1/2",
};

export const LoadingOverlay: Component<LoadingOverlayProps> = (props) => {
  const pos = () => positionClasses[props.position || "bottom-right"];

  return (
    <>
      {/* Loading indicator */}
      <Show when={props.isLoading()}>
        <div
          class={`${pos()} z-notification flex items-center gap-2 px-3 py-2 rounded-lg bg-bg-panel border border-border shadow-lg animate-slide-up`}
          role="status"
          aria-live="polite"
        >
          {/* Spinner */}
          <div class="w-4 h-4 border-2 border-accent/30 border-t-accent rounded-full animate-spin" />
          <span class="text-xs text-txt">{props.message()}</span>
        </div>
      </Show>

      {/* Error indicator (auto-dismiss after 4s) */}
      <Show when={props.error()}>
        <div
          class={`${pos()} z-notification flex items-center gap-2 px-3 py-2 rounded-lg bg-error/10 border border-error/30 shadow-lg animate-slide-up`}
          role="alert"
        >
          <span class="text-xs text-error">{props.error()}</span>
        </div>
      </Show>
    </>
  );
};
