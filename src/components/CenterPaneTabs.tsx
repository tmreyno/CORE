// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * CenterPaneTabs - Tabbed interface for the center detail pane
 * 
 * Provides tabs to switch between:
 * 1. Evidence Files - Shows the traditional evidence file detail panel
 * 2. Case Documents - Shows case document viewer for selected documents
 */

import { Component, Show, For, createMemo } from "solid-js";
import {
  HiOutlineDocumentText,
  HiOutlineClipboardDocumentList,
  HiOutlineXMark,
} from "./icons";
import type { CenterPaneTab, OpenDocumentTab } from "../hooks";

interface CenterPaneTabsProps {
  /** Currently active tab category */
  activeTab: CenterPaneTab;
  /** Callback when tab changes */
  onTabChange: (tab: CenterPaneTab) => void;
  /** Open document tabs (case documents) */
  openDocumentTabs: OpenDocumentTab[];
  /** Active document tab ID */
  activeDocumentTabId: string | null;
  /** Callback when document tab is selected */
  onDocumentTabSelect: (tab: OpenDocumentTab) => void;
  /** Callback when document tab is closed */
  onDocumentTabClose: (tabId: string) => void;
  /** Whether there's an active evidence file */
  hasActiveEvidence: boolean;
  /** Evidence file name (for display) */
  activeEvidenceName?: string;
  /** Children to render (the actual panel content) */
  children: any;
}

export const CenterPaneTabs: Component<CenterPaneTabsProps> = (props) => {
  // Count of open document tabs
  const documentTabCount = createMemo(() => props.openDocumentTabs.length);

  return (
    <div class="flex flex-col h-full">
      {/* Tab bar */}
      <div class="flex items-center bg-bg-secondary border-b border-border px-1 gap-1 shrink-0 h-8">
        {/* Evidence Tab */}
        <button
          class="flex items-center gap-1.5 px-3 py-1 text-xs rounded-t transition-colors"
          classList={{
            "bg-bg text-txt border-t border-l border-r border-border -mb-px": props.activeTab === "evidence",
            "text-txt-muted hover:text-txt hover:bg-bg-hover": props.activeTab !== "evidence",
          }}
          onClick={() => props.onTabChange("evidence")}
          title="Evidence Files"
        >
          <HiOutlineDocumentText class="w-3.5 h-3.5" />
          <span>Evidence</span>
          <Show when={props.hasActiveEvidence && props.activeEvidenceName}>
            <span class="text-txt-muted truncate max-w-[120px]">
              — {props.activeEvidenceName}
            </span>
          </Show>
        </button>

        {/* Document Tabs */}
        <Show when={documentTabCount() > 0}>
          <div class="flex items-center gap-0.5 ml-1 border-l border-border pl-1">
            <For each={props.openDocumentTabs}>
              {(tab) => (
                <div
                  class="flex items-center gap-1 px-2 py-1 text-xs rounded-t transition-colors group cursor-pointer"
                  classList={{
                    "bg-bg text-txt border-t border-l border-r border-border -mb-px": 
                      props.activeTab === "document" && props.activeDocumentTabId === tab.id,
                    "text-txt-muted hover:text-txt hover:bg-bg-hover": 
                      props.activeTab !== "document" || props.activeDocumentTabId !== tab.id,
                  }}
                  onClick={() => {
                    props.onTabChange("document");
                    props.onDocumentTabSelect(tab);
                  }}
                  title={tab.path}
                >
                  <HiOutlineClipboardDocumentList class="w-3 h-3 shrink-0" />
                  <span class="truncate max-w-[100px]">{tab.filename}</span>
                  <button
                    class="ml-1 p-0.5 rounded hover:bg-bg-hover opacity-50 group-hover:opacity-100"
                    onClick={(e) => {
                      e.stopPropagation();
                      props.onDocumentTabClose(tab.id);
                    }}
                    title="Close"
                  >
                    <HiOutlineXMark class="w-3 h-3" />
                  </button>
                </div>
              )}
            </For>
          </div>
        </Show>

        {/* Spacer */}
        <div class="flex-1" />
      </div>

      {/* Content area */}
      <div class="flex-1 overflow-hidden">
        {props.children}
      </div>
    </div>
  );
};

export default CenterPaneTabs;
