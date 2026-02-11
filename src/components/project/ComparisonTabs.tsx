// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ComparisonTabs Component
 * 
 * Tab navigation for comparison view:
 * - Bookmarks tab with sync button
 * - Notes tab with sync button
 * - Evidence tab
 */

import { Component, Show } from "solid-js";
import {
  HiOutlineBookmark,
  HiOutlineDocumentText,
  HiOutlineFolder,
} from "../icons";

type ComparisonTab = "bookmarks" | "notes" | "evidence";

interface ComparisonTabsProps {
  activeTab: ComparisonTab;
  onTabChange: (tab: ComparisonTab) => void;
  onSyncBookmarks: () => void;
  onSyncNotes: () => void;
}

export const ComparisonTabs: Component<ComparisonTabsProps> = (props) => {
  return (
    <div class="flex items-center gap-2 p-4 border-b border-border">
      <button
        onClick={() => props.onTabChange("bookmarks")}
        class={`px-4 py-2 rounded-md flex items-center gap-2 ${
          props.activeTab === "bookmarks"
            ? "bg-accent text-white"
            : "bg-bg text-txt-secondary hover:text-txt hover:bg-bg-hover"
        }`}
      >
        <HiOutlineBookmark class="w-icon-sm h-icon-sm" />
        Bookmarks
      </button>
      <button
        onClick={() => props.onTabChange("notes")}
        class={`px-4 py-2 rounded-md flex items-center gap-2 ${
          props.activeTab === "notes"
            ? "bg-accent text-white"
            : "bg-bg text-txt-secondary hover:text-txt hover:bg-bg-hover"
        }`}
      >
        <HiOutlineDocumentText class="w-icon-sm h-icon-sm" />
        Notes
      </button>
      <button
        onClick={() => props.onTabChange("evidence")}
        class={`px-4 py-2 rounded-md flex items-center gap-2 ${
          props.activeTab === "evidence"
            ? "bg-accent text-white"
            : "bg-bg text-txt-secondary hover:text-txt hover:bg-bg-hover"
        }`}
      >
        <HiOutlineFolder class="w-icon-sm h-icon-sm" />
        Evidence
      </button>
      <div class="flex-1" />
      <Show when={props.activeTab === "bookmarks"}>
        <button
          onClick={props.onSyncBookmarks}
          class="px-3 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md text-sm border border-border"
        >
          Sync Bookmarks →
        </button>
      </Show>
      <Show when={props.activeTab === "notes"}>
        <button
          onClick={props.onSyncNotes}
          class="px-3 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md text-sm border border-border"
        >
          Sync Notes →
        </button>
      </Show>
    </div>
  );
};
