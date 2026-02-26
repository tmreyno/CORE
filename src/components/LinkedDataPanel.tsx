// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * LinkedDataPanel — Right-panel component for evidence collection tabs.
 *
 * Renders a tabbed panel styled consistently with ViewerMetadataPanel,
 * showing the linked data tree (collections → collected items → COC → evidence files)
 * and a summary tab with collection statistics.
 */

import { Show, createSignal, createMemo } from "solid-js";
import { LinkedDataTree, type LinkedDataNode } from "./LinkedDataTree";
import {
  HiOutlineArchiveBoxArrowDown,
  HiOutlineDocumentText,
  HiOutlineFingerPrint,
  HiOutlineArchiveBox,
} from "./icons";

// =============================================================================
// Props
// =============================================================================

export interface LinkedDataPanelProps {
  /** Linked data tree nodes */
  nodes: LinkedDataNode[];
  /** Called when user clicks a tree node */
  onNodeClick?: (node: LinkedDataNode) => void;
}

// =============================================================================
// Tab definitions
// =============================================================================

type LinkedDataTabId = "tree" | "summary";

// =============================================================================
// Component
// =============================================================================

export function LinkedDataPanel(props: LinkedDataPanelProps) {
  const [activeTab, setActiveTab] = createSignal<LinkedDataTabId>("tree");

  /** Count nodes by type recursively */
  const stats = createMemo(() => {
    let collections = 0;
    let items = 0;
    let coc = 0;
    let evidenceFiles = 0;

    const walk = (nodes: LinkedDataNode[]) => {
      for (const node of nodes) {
        switch (node.type) {
          case "collection": collections++; break;
          case "collected-item": items++; break;
          case "coc": coc++; break;
          case "evidence-file": evidenceFiles++; break;
        }
        if (node.children) walk(node.children);
      }
    };

    walk(props.nodes);
    return { collections, items, coc, evidenceFiles };
  });

  return (
    <div class="flex flex-col h-full bg-bg">
      {/* Tab header — matches ViewerMetadataPanel style */}
      <div class="flex items-center border-b border-border bg-bg-secondary">
        <button
          class={`px-3 py-2 text-xs font-medium transition-colors ${
            activeTab() === "tree"
              ? "text-accent border-b-2 border-accent"
              : "text-txt-muted hover:text-txt"
          }`}
          onClick={() => setActiveTab("tree")}
        >
          Linked Data
        </button>
        <button
          class={`px-3 py-2 text-xs font-medium transition-colors ${
            activeTab() === "summary"
              ? "text-accent border-b-2 border-accent"
              : "text-txt-muted hover:text-txt"
          }`}
          onClick={() => setActiveTab("summary")}
        >
          Summary
        </button>
      </div>

      {/* Tab content */}
      <div class="flex-1 overflow-y-auto">
        {/* Tree tab */}
        <Show when={activeTab() === "tree"}>
          <Show when={props.nodes.length > 0} fallback={
            <div class="flex flex-col items-center justify-center py-8 text-txt-muted text-sm gap-2">
              <HiOutlineArchiveBoxArrowDown class="w-8 h-8 opacity-30" />
              <p>No linked data yet</p>
              <p class="text-xs">Save the collection to see relationships</p>
            </div>
          }>
            <div class="p-1">
              <LinkedDataTree nodes={props.nodes} onNodeClick={props.onNodeClick} />
            </div>
          </Show>
        </Show>

        {/* Summary tab */}
        <Show when={activeTab() === "summary"}>
          <div class="p-3 flex flex-col gap-3">
            {/* Statistics */}
            <div class="text-xs font-medium text-txt-muted uppercase tracking-wide mb-1">
              Collection Statistics
            </div>
            <div class="flex flex-col gap-2">
              <SummaryRow
                icon={<HiOutlineArchiveBoxArrowDown class="w-4 h-4 text-accent" />}
                label="Collections"
                value={stats().collections}
              />
              <SummaryRow
                icon={<HiOutlineDocumentText class="w-4 h-4 text-type-ad1" />}
                label="Collected Items"
                value={stats().items}
              />
              <SummaryRow
                icon={<HiOutlineFingerPrint class="w-4 h-4 text-warning" />}
                label="COC Records"
                value={stats().coc}
              />
              <SummaryRow
                icon={<HiOutlineArchiveBox class="w-4 h-4 text-type-e01" />}
                label="Evidence Files"
                value={stats().evidenceFiles}
              />
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
}

// =============================================================================
// Helpers
// =============================================================================

function SummaryRow(props: { icon: any; label: string; value: number }) {
  return (
    <div class="flex items-center gap-2 px-2 py-1.5 rounded bg-bg-secondary">
      {props.icon}
      <span class="flex-1 text-sm text-txt">{props.label}</span>
      <span class="text-sm font-medium text-txt">{props.value}</span>
    </div>
  );
}
