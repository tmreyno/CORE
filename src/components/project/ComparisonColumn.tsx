// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ComparisonColumn Component
 * 
 * Single column in 3-column comparison view showing:
 * - Section header with icon and title
 * - List of items (bookmarks/notes/evidence)
 * - Item count footer
 * - Modified status highlighting for common items
 */

import { Component, Show, For } from "solid-js";

type ComparisonTab = "bookmarks" | "notes" | "evidence";

interface ComparisonColumnProps {
  title: string;
  icon: Component<{ class?: string }>;
  iconColor: string;
  activeTab: ComparisonTab;
  bookmarks: string[];
  notes: string[];
  evidence: string[];
  modifiedBookmarks?: string[];
  modifiedNotes?: string[];
  isCommonColumn?: boolean;
}

export const ComparisonColumn: Component<ComparisonColumnProps> = (props) => {
  const Icon = props.icon;
  
  return (
    <div class="bg-bg rounded-lg border border-border p-4">
      <h3 class="text-sm font-medium text-txt mb-3 flex items-center gap-2">
        <Icon class={`w-icon-sm h-icon-sm ${props.iconColor}`} />
        {props.title}
      </h3>
      <div class="space-y-2 max-h-96 overflow-auto">
        <Show when={props.activeTab === "bookmarks"}>
          <For each={props.bookmarks}>
            {(name) => {
              const isModified = props.isCommonColumn && 
                                 props.modifiedBookmarks?.includes(name);
              return (
                <div
                  class={`p-2 rounded text-sm ${
                    isModified
                      ? "bg-warning/20 border border-warning"
                      : "bg-bg-secondary"
                  }`}
                >
                  <div class="font-medium text-txt truncate">
                    {name}
                  </div>
                  <Show when={isModified}>
                    <div class="text-xs text-warning mt-1">
                      Modified in one project
                    </div>
                  </Show>
                </div>
              );
            }}
          </For>
        </Show>
        <Show when={props.activeTab === "notes"}>
          <For each={props.notes}>
            {(title) => {
              const isModified = props.isCommonColumn && 
                                 props.modifiedNotes?.includes(title);
              return (
                <div
                  class={`p-2 rounded text-sm ${
                    isModified
                      ? "bg-warning/20 border border-warning"
                      : "bg-bg-secondary"
                  }`}
                >
                  <div class="font-medium text-txt truncate">
                    {title}
                  </div>
                  <Show when={isModified}>
                    <div class="text-xs text-warning mt-1">
                      Modified in one project
                    </div>
                  </Show>
                </div>
              );
            }}
          </For>
        </Show>
        <Show when={props.activeTab === "evidence"}>
          <For each={props.evidence}>
            {(path) => (
              <div class="p-2 bg-bg-secondary rounded text-sm">
                <div class="font-medium text-txt truncate">
                  {path}
                </div>
              </div>
            )}
          </For>
        </Show>
      </div>
      <div class="mt-3 text-xs text-txt-muted text-center">
        {props.activeTab === "bookmarks" && `${props.bookmarks.length} ${props.isCommonColumn ? 'common' : 'unique'} items`}
        {props.activeTab === "notes" && `${props.notes.length} ${props.isCommonColumn ? 'common' : 'unique'} items`}
        {props.activeTab === "evidence" && `${props.evidence.length} ${props.isCommonColumn ? 'common' : 'unique'} items`}
        {props.isCommonColumn && props.modifiedBookmarks && props.modifiedBookmarks.length > 0 &&
          props.activeTab === "bookmarks" &&
          ` (${props.modifiedBookmarks.length} modified)`}
        {props.isCommonColumn && props.modifiedNotes && props.modifiedNotes.length > 0 &&
          props.activeTab === "notes" &&
          ` (${props.modifiedNotes.length} modified)`}
      </div>
    </div>
  );
};
