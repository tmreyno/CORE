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
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { LinkedDataTree, type LinkedDataNode } from "./LinkedDataTree";
import {
  HiOutlineArchiveBoxArrowDown,
  HiOutlineDocumentText,
  HiOutlineFingerPrint,
  HiOutlineArchiveBox,
  HiOutlinePrinter,
  HiOutlineArrowDownTray,
} from "./icons";
import { printDocument } from "./document/documentHelpers";
import { logger } from "../utils/logger";

const log = logger.scope("LinkedDataPanel");

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

  /** Flatten tree into rows for CSV/print */
  const flattenTree = (
    nodes: LinkedDataNode[],
    depth = 0,
  ): { depth: number; type: string; label: string; sublabel: string }[] => {
    const rows: { depth: number; type: string; label: string; sublabel: string }[] = [];
    for (const node of nodes) {
      rows.push({
        depth,
        type: node.type,
        label: node.label,
        sublabel: node.sublabel || "",
      });
      if (node.children) {
        rows.push(...flattenTree(node.children, depth + 1));
      }
    }
    return rows;
  };

  /** Export linked data as CSV */
  const handleExportCsv = async () => {
    const flat = flattenTree(props.nodes);
    if (flat.length === 0) return;

    try {
      const path = await save({
        title: "Export Linked Data as CSV",
        defaultPath: "linked-data.csv",
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });
      if (!path) return;

      const header = "Depth,Type,Label,Details";
      const lines = flat.map(
        (r) =>
          `${r.depth},${csvEsc(r.type)},${csvEsc(r.label)},${csvEsc(r.sublabel)}`,
      );
      const csv = [header, ...lines].join("\n");
      await invoke("write_text_file", { path, content: csv });
    } catch (e) {
      log.error("Export linked data failed:", e);
    }
  };

  /** Print linked data tree + summary */
  const handlePrint = () => {
    const flat = flattenTree(props.nodes);
    const s = stats();

    const treeRows = flat
      .map(
        (r) =>
          `<tr><td style="border:1px solid #eee;padding:2px 8px;font-size:12px;padding-left:${8 + r.depth * 16}px">${escH(r.label)}</td><td style="border:1px solid #eee;padding:2px 8px;font-size:12px;">${escH(r.type)}</td><td style="border:1px solid #eee;padding:2px 8px;font-size:12px;">${escH(r.sublabel)}</td></tr>`,
      )
      .join("");

    const html = `<!DOCTYPE html><html><head><title>Linked Data</title><style>body{font-family:system-ui,sans-serif;margin:20px}table{border-collapse:collapse;width:100%}th{background:#f5f5f5;text-align:left}@media print{body{margin:0}}</style></head><body><h2>Linked Data Summary</h2><p style="font-size:13px;color:#666;">Printed: ${new Date().toLocaleString()}</p><div style="display:flex;gap:24px;margin:12px 0"><div><strong>${s.collections}</strong> Collections</div><div><strong>${s.items}</strong> Items</div><div><strong>${s.coc}</strong> COC Records</div><div><strong>${s.evidenceFiles}</strong> Evidence Files</div></div><table><thead><tr><th style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">Label</th><th style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">Type</th><th style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">Details</th></tr></thead><tbody>${treeRows}</tbody></table></body></html>`;
    printDocument(html);
  };

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
        <div class="flex-1" />
        <Show when={props.nodes.length > 0}>
          <button
            class="icon-btn-sm mr-1"
            title="Export as CSV"
            onClick={handleExportCsv}
          >
            <HiOutlineArrowDownTray class="w-3.5 h-3.5" />
          </button>
          <button
            class="icon-btn-sm mr-1"
            title="Print linked data"
            onClick={handlePrint}
          >
            <HiOutlinePrinter class="w-3.5 h-3.5" />
          </button>
        </Show>
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

function csvEsc(s: string): string {
  if (s.includes(",") || s.includes('"') || s.includes("\n")) {
    return `"${s.replace(/"/g, '""')}"`;
  }
  return s;
}

function escH(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
