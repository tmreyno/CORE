// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For } from "solid-js";
import {
  HiOutlineFolderOpen,
  HiOutlineServer,
  HiOutlineXMark,
  HiOutlineLockClosed,
} from "../icons";
import { getBasename } from "../../utils/pathUtils";
import type { ExportMode } from "../../hooks/useExportState";
import type { Accessor } from "solid-js";

interface ExportSourceSectionProps {
  mode: Accessor<ExportMode>;
  sources: Accessor<string[]>;
  destination: Accessor<string>;
  driveSources: Accessor<Set<string>>;
  mountDrivesReadOnly: Accessor<boolean>;
  onRemoveSource: (index: number) => void;
  onSelectDestination: () => void;
}

export function ExportSourceSection(props: ExportSourceSectionProps) {
  return (
    <>
      {/* Source List */}
      <div class="space-y-2">
        <label class="label">
          {props.mode() === "physical" || props.mode() === "logical" ? "Source" : "Source Files"}
        </label>

        <Show when={props.sources().length > 0}>
          <div class="space-y-1">
            <For each={props.sources()}>
              {(source, index) => {
                const isDrive = () => props.driveSources().has(source);
                return (
                  <div class="flex items-center gap-2 p-2 bg-bg-secondary rounded text-sm">
                    <Show when={isDrive()}>
                      <HiOutlineServer class="w-4 h-4 text-accent shrink-0" />
                    </Show>
                    <span class="flex-1 truncate text-txt" title={source}>
                      {isDrive() ? source : (getBasename(source) || source)}
                    </span>
                    <Show when={isDrive()}>
                      <span class="badge badge-warning text-2xs shrink-0">Drive</span>
                      <Show when={props.mountDrivesReadOnly()}>
                        <span
                          class="badge badge-success text-2xs shrink-0 flex items-center gap-0.5"
                          title="Will be remounted read-only before imaging"
                        >
                          <HiOutlineLockClosed class="w-2.5 h-2.5" />
                          RO
                        </span>
                      </Show>
                    </Show>
                    <button
                      class="icon-btn-sm"
                      onClick={() => props.onRemoveSource(index())}
                      title="Remove"
                    >
                      <HiOutlineXMark class="w-3 h-3" />
                    </button>
                  </div>
                );
              }}
            </For>
          </div>
        </Show>

        <Show when={props.sources().length === 0}>
          <p class="text-xs text-txt-muted italic">
            Use the Sources panel to add files, folders, or drives.
          </p>
        </Show>
      </div>

      {/* Destination */}
      <div class="space-y-2">
        <label class="label">Destination</label>
        <div class="flex gap-2">
          <input
            class="input-sm flex-1"
            type="text"
            value={props.destination()}
            placeholder="Select destination folder..."
            readOnly
          />
          <button class="btn-sm" onClick={props.onSelectDestination}>
            <HiOutlineFolderOpen class="w-4 h-4" />
            Browse
          </button>
        </div>
      </div>
    </>
  );
}
