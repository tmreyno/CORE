// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * SelectStep — Step 1 of MergeProjectsWizard: select .cffx files.
 */

import { Component, For, Show } from "solid-js";
import {
  HiOutlineArchiveBox,
  HiOutlinePlus,
  HiOutlineTrash,
} from "../icons";
import { basename } from "./helpers";

export interface SelectStepProps {
  cffxPaths: string[];
  onAddProjects: () => void;
  onRemoveProject: (path: string) => void;
}

export const SelectStep: Component<SelectStepProps> = (props) => {
  return (
    <div class="col gap-4">
      <p class="text-sm text-txt-secondary">
        Select two or more .cffx project files to merge into a single project.
        All evidence data, sessions, hashes, bookmarks, and activity logs will be combined.
      </p>

      {/* Project list */}
      <div class="col gap-2">
        <For each={props.cffxPaths}>
          {(path) => (
            <div class="flex items-center gap-2 p-2.5 rounded-lg bg-bg-secondary border border-border">
              <HiOutlineArchiveBox class="w-4 h-4 text-accent shrink-0" />
              <div class="flex-1 min-w-0">
                <div class="text-sm font-medium text-txt truncate">{basename(path)}</div>
                <div class="text-xs text-txt-muted truncate">{path}</div>
              </div>
              <button
                class="icon-btn-sm text-txt-muted hover:text-error"
                onClick={() => props.onRemoveProject(path)}
                title="Remove"
              >
                <HiOutlineTrash class="w-3.5 h-3.5" />
              </button>
            </div>
          )}
        </For>
      </div>

      {/* Add button */}
      <button
        class="btn btn-secondary flex items-center gap-2"
        onClick={props.onAddProjects}
      >
        <HiOutlinePlus class="w-4 h-4" />
        Add Project Files
      </button>

      <Show when={props.cffxPaths.length > 0 && props.cffxPaths.length < 2}>
        <p class="text-xs text-warning">Select at least 2 projects to merge.</p>
      </Show>
    </div>
  );
};
