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
import { DriveTreeBrowser } from "./DriveTreeBrowser";
import type { ExportMode } from "../../hooks/useExportState";
import type { Accessor } from "solid-js";

interface ExportSourceSectionProps {
  mode: Accessor<ExportMode>;
  sources: Accessor<string[]>;
  destination: Accessor<string>;
  driveSources: Accessor<Set<string>>;
  mountDrivesReadOnly: Accessor<boolean>;
  onAddSources: () => void;
  onAddFolder: () => void;
  onRemoveSource: (index: number) => void;
  onSelectDestination: () => void;
  onShowDriveSelector: () => void;
  /** Called when a drive/folder is selected from the inline tree browser */
  onAddDriveSource?: (path: string) => void;
}

export function ExportSourceSection(props: ExportSourceSectionProps) {
  return (
    <>
      {/* Source Files */}
      <div class="space-y-2">
        <label class="label">
          {props.mode() === "physical" || props.mode() === "logical" ? "Source" : "Source Files"}
        </label>
        <div class="flex gap-2 flex-wrap">
          <button class="btn-sm" onClick={props.onAddSources}>
            <HiOutlineFolderOpen class="w-4 h-4" />
            Add Files
          </button>
          <button class="btn-sm" onClick={props.onAddFolder}>
            <HiOutlineFolderOpen class="w-4 h-4" />
            Add Folder
          </button>
          <Show when={props.mode() === "physical" || props.mode() === "logical"}>
            <button class="btn-sm" onClick={props.onShowDriveSelector}>
              <HiOutlineServer class="w-4 h-4" />
              Select Drive
            </button>
          </Show>
        </div>
        <Show when={props.mode() === "physical"}>
          <p class="text-xs text-txt-muted">
            Select raw disk images (.dd, .raw, .img), memory dumps (.mem),
            folders, drives/volumes, or other evidence files to wrap in an E01 container.
          </p>
        </Show>
        <Show when={props.mode() === "logical"}>
          <p class="text-xs text-txt-muted">
            Select files, folders, or drives/volumes to package into an L01 logical evidence container.
          </p>
        </Show>

        {/* Inline drive tree browser for physical/logical modes */}
        <Show when={(props.mode() === "physical" || props.mode() === "logical") && props.onAddDriveSource}>
          <DriveTreeBrowser
            onSelectSource={(path) => props.onAddDriveSource?.(path)}
            selectedPaths={() => props.driveSources()}
          />
        </Show>

        {/* Source List */}
        <Show when={props.sources().length > 0}>
          <div class="space-y-1 mt-2">
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
          <div class="text-sm text-txt-muted italic">No files selected</div>
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
