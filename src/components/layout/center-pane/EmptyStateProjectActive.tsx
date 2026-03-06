// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, type Accessor } from "solid-js";
import {
  HiOutlineCheckCircle,
  HiOutlineArchiveBox,
  HiOutlineFolder,
} from "../../icons";
import { Shortcut, CommonShortcuts } from "../../ui/Kbd";

interface EmptyStateProjectActiveProps {
  projectName?: Accessor<string | null>;
  projectRoot?: Accessor<string | null>;
  evidenceCount?: Accessor<number>;
}

/** Shown when a project is loaded but no file is selected */
export function EmptyStateProjectActive(props: EmptyStateProjectActiveProps) {
  return (
    <div class="text-center p-8 max-w-lg">
      <div class="w-20 h-20 mx-auto mb-6 rounded-full bg-accent/10 flex items-center justify-center">
        <HiOutlineCheckCircle class="w-10 h-10 text-accent" />
      </div>
      <h3 class="text-txt font-semibold text-lg mb-1">
        {props.projectName?.() || "Project Ready"}
      </h3>
      <p class="text-txt-muted text-sm mb-6">
        <Show when={props.projectRoot?.()} fallback="Project loaded successfully">
          <span class="font-mono text-xs bg-bg-secondary px-2 py-1 rounded">
            {props.projectRoot!()}
          </span>
        </Show>
      </p>
      
      {/* Evidence count */}
      <Show when={props.evidenceCount && props.evidenceCount() > 0}>
        <div class="flex items-center justify-center gap-2 mb-6 text-sm">
          <HiOutlineArchiveBox class="w-5 h-5 text-accent" />
          <span class="text-txt">
            <span class="font-semibold">{props.evidenceCount!()}</span> evidence file{props.evidenceCount!() !== 1 ? 's' : ''} discovered
          </span>
        </div>
      </Show>
      
      <div class="p-4 bg-bg-secondary rounded-lg mb-6">
        <div class="flex items-center gap-3 text-left">
          <HiOutlineFolder class="w-6 h-6 text-accent shrink-0" />
          <div>
            <p class="text-txt text-sm font-medium">Select a file to begin</p>
            <p class="text-txt-muted text-xs">
              Choose an evidence container, case document, or processed database from the sidebar
            </p>
          </div>
        </div>
      </div>
      
      <div class="flex items-center justify-center gap-6 text-xs text-txt-muted">
        <span class="flex items-center gap-1.5">
          <Shortcut {...CommonShortcuts.open} />
          <span>Open File</span>
        </span>
        <span class="flex items-center gap-1.5">
          <Shortcut {...CommonShortcuts.commandPalette} />
          <span>Commands</span>
        </span>
      </div>
    </div>
  );
}
