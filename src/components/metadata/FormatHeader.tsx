// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import type { ParsedMetadata } from "../HexViewer";

interface FormatHeaderProps {
  metadata: ParsedMetadata | null;
}

export const FormatHeader: Component<FormatHeaderProps> = (props) => {
  return (
    <Show when={props.metadata}>
      {meta => (
        <div class="panel-header">
          <span class="text-txt-muted text-[10px] leading-tight">Format</span>
          <span class="font-semibold text-accent">{meta().format}</span>
          <Show when={meta().version}>
            <span class="text-[10px] leading-tight text-txt-muted">v{meta().version}</span>
          </Show>
        </div>
      )}
    </Show>
  );
};
