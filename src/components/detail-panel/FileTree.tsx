// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For, Component, Accessor } from "solid-js";
import { HiOutlineFolder, HiOutlineDocument } from "../icons";
import type { TreeEntry } from "../../types";
import { formatBytes } from "../../utils";

interface FileTreeProps {
  tree: TreeEntry[];
  filteredTree: TreeEntry[];
  localTreeFilter: Accessor<string>;
  treeCount: Accessor<number>;
  hasTree: Accessor<boolean>;
  treeExceedsLimit: Accessor<boolean>;
  handleTreeFilterInput: (value: string) => void;
}

export const FileTree: Component<FileTreeProps> = (props) => {
  return (
    <Show when={props.hasTree()}>
      <div class="info-card">
        <div class="flex items-center justify-between mb-2">
          <span class="info-card-title">
            <HiOutlineFolder class="w-4 h-4 text-yellow-500" /> File Tree ({props.treeCount()})
          </span>
          <input 
            type="text" 
            class="text-xs px-2 py-1 rounded bg-bg border border-border text-txt placeholder-txt-muted focus:border-accent focus:outline-none w-32" 
            placeholder="Filter..." 
            value={props.localTreeFilter()} 
            onInput={(e) => props.handleTreeFilterInput(e.currentTarget.value)} 
          />
        </div>
        <div class="flex flex-col gap-0.5 max-h-64 overflow-y-auto bg-bg/50 rounded p-1">
          <For each={props.filteredTree}>
            {(entry) => (
              <div 
                class={`flex items-center gap-1.5 px-2 py-1 text-xs rounded hover:bg-bg-panel ${
                  entry.is_dir ? "text-accent" : "text-txt-tertiary"
                }`} 
                title={entry.path}
              >
                <span class="w-4 shrink-0 flex items-center justify-center">
                  {entry.is_dir 
                    ? <HiOutlineFolder class={`w-3 h-3 text-yellow-500`} /> 
                    : <HiOutlineDocument class="w-3 h-3" />
                  }
                </span>
                <span class="truncate flex-1" title={entry.path}>{entry.path}</span>
                <span class="text-txt-muted w-16 text-right shrink-0">
                  {entry.is_dir ? "" : formatBytes(entry.size)}
                </span>
              </div>
            )}
          </For>
          <Show when={props.treeExceedsLimit()}>
            <div class="text-center text-xs text-txt-muted py-2">
              Showing first 500 of {props.treeCount()} items
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
};
