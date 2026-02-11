// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * CaseDocumentsHeader Component
 * 
 * Header bar displaying:
 * - Panel title with icon
 * - Total document count badge
 * - COC count badge (if any)
 * - Refresh button
 */

import { Component, Show } from "solid-js";
import { HiOutlineClipboardDocumentList, HiOutlineArrowPath } from "../icons";

interface CaseDocumentsHeaderProps {
  documentCount: number;
  cocCount: number;
  searchPath: string | null;
  loading: boolean;
  onRefresh: () => void;
}

export const CaseDocumentsHeader: Component<CaseDocumentsHeaderProps> = (props) => {
  return (
    <div class="flex items-center justify-between px-3 py-2 border-b border-border">
      <div class="flex items-center gap-2">
        <HiOutlineClipboardDocumentList class="w-4 h-4 text-accent" />
        <span class="text-sm font-medium text-txt">Case Documents</span>
        <Show when={props.documentCount > 0}>
          <span class="px-1.5 py-0.5 text-xs bg-bg-panel text-txt-secondary rounded">
            {props.documentCount}
          </span>
        </Show>
        <Show when={props.cocCount > 0}>
          <span class="px-1.5 py-0.5 text-xs bg-accent/20 text-accent rounded">
            {props.cocCount} COC
          </span>
        </Show>
      </div>
      
      <Show when={props.searchPath}>
        <button
          onClick={props.onRefresh}
          class="p-1 text-txt-secondary hover:text-txt hover:bg-bg-panel rounded"
          title="Refresh"
        >
          <HiOutlineArrowPath class={`w-4 h-4 ${props.loading ? "animate-spin" : ""}`} />
        </button>
      </Show>
    </div>
  );
};
