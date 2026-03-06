// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import type { GlobalDropOverlayProps } from "./types";

/**
 * Full-screen drop overlay that appears when dragging files over the app
 */
export function GlobalDropOverlay(props: GlobalDropOverlayProps) {
  return (
    <Show when={props.active}>
      <div class="fixed inset-0 z-[9999] bg-black/60 backdrop-blur-sm flex items-center justify-center pointer-events-none">
        <div class="bg-bg border-2 border-dashed border-accent rounded-2xl p-12 text-center shadow-2xl animate-pulse">
          <div class="text-6xl mb-4">📥</div>
          <h2 class="text-xl font-semibold text-txt">
            Drop files to add evidence
          </h2>
          <p class="text-sm text-txt-secondary mt-2">
            Release to add files to your project
          </p>
          <Show when={props.accept && props.accept.length > 0}>
            <p class="text-xs text-txt-muted mt-4">
              Supported formats: {props.accept!.join(", ")}
            </p>
          </Show>
        </div>
      </div>
    </Show>
  );
}
