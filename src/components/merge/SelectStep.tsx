// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * SelectStep — Step 1 of MergeProjectsWizard: select .cffx files.
 *
 * When `currentProjectPath` is provided (merge-into-open mode):
 *  - The current project is shown as a pinned, non-removable entry
 *  - Instructions text changes to reflect single-project merge
 *  - Only 1 additional project is needed to proceed
 */

import { Component, For, Show } from "solid-js";
import {
  HiOutlineArchiveBox,
  HiOutlineLockClosed,
  HiOutlinePlus,
  HiOutlineTrash,
} from "../icons";
import { basename } from "./helpers";

export interface SelectStepProps {
  cffxPaths: string[];
  onAddProjects: () => void;
  onRemoveProject: (path: string) => void;
  /** When set, this project is pinned (merge-into-open mode) */
  currentProjectPath?: string;
}

export const SelectStep: Component<SelectStepProps> = (props) => {
  const isMergeIntoOpen = () => !!props.currentProjectPath;

  /** Additional (non-pinned) projects the user selected */
  const additionalPaths = () =>
    props.currentProjectPath
      ? props.cffxPaths.filter((p) => p !== props.currentProjectPath)
      : props.cffxPaths;

  const minRequired = () => (isMergeIntoOpen() ? 1 : 2);
  const totalCount = () => props.cffxPaths.length;

  return (
    <div class="col gap-4">
      <Show
        when={isMergeIntoOpen()}
        fallback={
          <p class="text-sm text-txt-secondary">
            Select two or more .cffx project files to merge into a single project.
            All evidence data, sessions, hashes, bookmarks, and activity logs will be combined.
          </p>
        }
      >
        <p class="text-sm text-txt-secondary">
          Select a project to merge <strong class="text-txt">into the currently open project</strong>.
          Evidence data, collections, COC items, and other records from the selected project will be
          merged into your open project. Duplicate records (same ID) are automatically skipped.
        </p>
      </Show>

      {/* Project list */}
      <div class="col gap-2">
        {/* Pinned current project (merge-into-open mode) */}
        <Show when={isMergeIntoOpen() && props.currentProjectPath}>
          <div class="flex items-center gap-2 p-2.5 rounded-lg bg-accent/10 border border-accent/30">
            <HiOutlineLockClosed class="w-4 h-4 text-accent shrink-0" />
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-sm font-medium text-txt truncate">{basename(props.currentProjectPath!)}</span>
                <span class="badge badge-success" style="font-size: 10px; padding: 1px 6px;">Current</span>
              </div>
              <div class="text-xs text-txt-muted truncate">{props.currentProjectPath}</div>
            </div>
          </div>
        </Show>

        {/* Additional projects (removable) */}
        <For each={additionalPaths()}>
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
        {isMergeIntoOpen() ? "Select Project to Merge" : "Add Project Files"}
      </button>

      <Show when={totalCount() > 0 && totalCount() < minRequired()}>
        <p class="text-xs text-warning">
          {isMergeIntoOpen()
            ? "Select a project to merge into the current project."
            : "Select at least 2 projects to merge."}
        </p>
      </Show>
    </div>
  );
};
