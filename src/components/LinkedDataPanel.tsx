// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * LinkedDataPanel — Right-panel component for evidence collection tabs.
 *
 * Renders a tabbed panel styled consistently with ViewerMetadataPanel,
 * showing:
 * - Tree tab: linked data tree with metadata badges + collapsible detail pane
 * - Summary tab: enriched collection statistics with hash/COC breakdowns
 *
 * When a tree node is clicked, a detail pane below the tree shows all metadata
 * for that node (device IDs, forensic acquisition info, hash details, COC, etc.)
 */

import { Show, For, createSignal, createMemo, type JSX } from "solid-js";
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
  HiOutlineCheckCircle,
  HiOutlineXCircle,
  HiOutlineShieldCheck,
  HiOutlineXMark,
} from "./icons";
import { CollapsibleGroup, OptionalMetadataRow, SectionHeader, SummaryRow } from "./viewerMetadata/shared";
import { printDocument } from "./document/documentHelpers";
import { logger } from "../utils/logger";

const log = logger.scope("LinkedDataPanel");

// =============================================================================
// Props
// =============================================================================

export interface LinkedDataPanelProps {
  /** Linked data tree nodes */
  nodes: LinkedDataNode[];
  /** Called when user clicks a tree node (external handler) */
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
  const [selectedNode, setSelectedNode] = createSignal<LinkedDataNode | null>(null);

  // ─── Node click handler ──────────────────────────────────────────────
  const handleNodeClick = (node: LinkedDataNode) => {
    setSelectedNode((prev) => (prev?.id === node.id ? null : node));
    props.onNodeClick?.(node);
  };

  // ─── Statistics ──────────────────────────────────────────────────────

  /** Walk all nodes and collect typed statistics */
  const enrichedStats = createMemo(() => {
    let collections = 0;
    let items = 0;
    let coc = 0;
    let evidenceFiles = 0;
    let hashVerified = 0;
    let hashMismatch = 0;
    let hashComputed = 0;
    let hashNone = 0;
    let cocLocked = 0;
    let cocDraft = 0;
    let cocVoided = 0;
    let totalSizeBytes = 0;
    const containerTypes = new Map<string, number>();

    const walk = (nodes: LinkedDataNode[]) => {
      for (const node of nodes) {
        switch (node.type) {
          case "collection": collections++; break;
          case "collected-item": items++; break;
          case "coc": {
            coc++;
            const cs = node.metadata?.cocStatus;
            if (cs === "locked") cocLocked++;
            else if (cs === "voided") cocVoided++;
            else cocDraft++;
            break;
          }
          case "evidence-file": {
            evidenceFiles++;
            const hs = node.metadata?.hashStatus;
            if (hs === "verified") hashVerified++;
            else if (hs === "mismatch") hashMismatch++;
            else if (hs === "computed") hashComputed++;
            else hashNone++;
            if (node.metadata?.totalSize) totalSizeBytes += node.metadata.totalSize;
            if (node.metadata?.containerType) {
              const ct = node.metadata.containerType.toUpperCase();
              containerTypes.set(ct, (containerTypes.get(ct) ?? 0) + 1);
            }
            break;
          }
        }
        if (node.children) walk(node.children);
      }
    };

    walk(props.nodes);
    return {
      collections, items, coc, evidenceFiles,
      hashVerified, hashMismatch, hashComputed, hashNone,
      cocLocked, cocDraft, cocVoided,
      totalSizeBytes, containerTypes,
    };
  });

  // ─── CSV / Print helpers ────────────────────────────────────────────

  const flattenTree = (
    nodes: LinkedDataNode[],
    depth = 0,
  ): { depth: number; type: string; label: string; sublabel: string }[] => {
    const rows: { depth: number; type: string; label: string; sublabel: string }[] = [];
    for (const node of nodes) {
      rows.push({ depth, type: node.type, label: node.label, sublabel: node.sublabel || "" });
      if (node.children) rows.push(...flattenTree(node.children, depth + 1));
    }
    return rows;
  };

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
        (r) => `${r.depth},${csvEsc(r.type)},${csvEsc(r.label)},${csvEsc(r.sublabel)}`,
      );
      await invoke("write_text_file", { path, content: [header, ...lines].join("\n") });
    } catch (e) {
      log.error("Export linked data failed:", e);
    }
  };

  const handlePrint = () => {
    const flat = flattenTree(props.nodes);
    const s = enrichedStats();
    const treeRows = flat
      .map(
        (r) =>
          `<tr><td style="border:1px solid #eee;padding:2px 8px;font-size:12px;padding-left:${8 + r.depth * 16}px">${escH(r.label)}</td><td style="border:1px solid #eee;padding:2px 8px;font-size:12px;">${escH(r.type)}</td><td style="border:1px solid #eee;padding:2px 8px;font-size:12px;">${escH(r.sublabel)}</td></tr>`,
      )
      .join("");
    const html = `<!DOCTYPE html><html><head><title>Linked Data</title><style>body{font-family:system-ui,sans-serif;margin:20px}table{border-collapse:collapse;width:100%}th{background:#f5f5f5;text-align:left}@media print{body{margin:0}}</style></head><body><h2>Linked Data Summary</h2><p style="font-size:13px;color:#666;">Printed: ${new Date().toLocaleString()}</p><div style="display:flex;gap:24px;margin:12px 0"><div><strong>${s.collections}</strong> Collections</div><div><strong>${s.items}</strong> Items</div><div><strong>${s.coc}</strong> COC Records</div><div><strong>${s.evidenceFiles}</strong> Evidence Files</div></div><table><thead><tr><th style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">Label</th><th style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">Type</th><th style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">Details</th></tr></thead><tbody>${treeRows}</tbody></table></body></html>`;
    printDocument(html);
  };

  // ─── Render ─────────────────────────────────────────────────────────

  return (
    <div class="flex flex-col h-full bg-bg">
      {/* Tab header */}
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
          <button class="icon-btn-sm mr-1" title="Export as CSV" onClick={handleExportCsv}>
            <HiOutlineArrowDownTray class="w-3.5 h-3.5" />
          </button>
          <button class="icon-btn-sm mr-1" title="Print linked data" onClick={handlePrint}>
            <HiOutlinePrinter class="w-3.5 h-3.5" />
          </button>
        </Show>
      </div>

      {/* Tab content */}
      <div class="flex-1 overflow-y-auto">
        {/* ──── Tree tab ──── */}
        <Show when={activeTab() === "tree"}>
          <Show when={props.nodes.length > 0} fallback={
            <div class="flex flex-col items-center justify-center py-8 text-txt-muted text-sm gap-2">
              <HiOutlineArchiveBoxArrowDown class="w-8 h-8 opacity-30" />
              <p>No linked data yet</p>
              <p class="text-xs">Save the collection to see relationships</p>
            </div>
          }>
            <div class="p-1">
              <LinkedDataTree
                nodes={props.nodes}
                onNodeClick={handleNodeClick}
                selectedNodeId={selectedNode()?.id}
              />
            </div>

            {/* ── Detail pane (below tree, shown when a node is selected) ── */}
            <Show when={selectedNode()}>
              {(node) => (
                <div class="border-t border-border">
                  <div class="flex items-center gap-1 px-3 py-1.5 bg-bg-secondary">
                    <span class="text-xs font-medium text-txt truncate flex-1">
                      {node().label}
                    </span>
                    <button
                      class="icon-btn-sm"
                      title="Close detail"
                      onClick={() => setSelectedNode(null)}
                    >
                      <HiOutlineXMark class="w-3.5 h-3.5" />
                    </button>
                  </div>
                  <div class="p-2">
                    <NodeDetailView node={node()} />
                  </div>
                </div>
              )}
            </Show>
          </Show>
        </Show>

        {/* ──── Summary tab ──── */}
        <Show when={activeTab() === "summary"}>
          <div class="p-3 flex flex-col gap-3">
            {/* Basic counts */}
            <SectionHeader label="Collection Statistics" />
            <div class="flex flex-col gap-1.5">
              <SummaryRow
                icon={<HiOutlineArchiveBoxArrowDown class="w-4 h-4 text-accent" />}
                label="Collections"
                value={enrichedStats().collections}
              />
              <SummaryRow
                icon={<HiOutlineDocumentText class="w-4 h-4 text-type-ad1" />}
                label="Collected Items"
                value={enrichedStats().items}
              />
              <SummaryRow
                icon={<HiOutlineShieldCheck class="w-4 h-4 text-warning" />}
                label="COC Records"
                value={enrichedStats().coc}
              />
              <SummaryRow
                icon={<HiOutlineArchiveBox class="w-4 h-4 text-type-e01" />}
                label="Evidence Files"
                value={enrichedStats().evidenceFiles}
              />
            </div>

            {/* Hash integrity breakdown */}
            <Show when={enrichedStats().evidenceFiles > 0}>
              <SectionHeader label="Hash Integrity" />
              <div class="flex flex-col gap-1.5">
                <Show when={enrichedStats().hashVerified > 0}>
                  <SummaryRow
                    icon={<HiOutlineCheckCircle class="w-4 h-4 text-success" />}
                    label="Verified"
                    value={enrichedStats().hashVerified}
                  />
                </Show>
                <Show when={enrichedStats().hashComputed > 0}>
                  <SummaryRow
                    icon={<HiOutlineFingerPrint class="w-4 h-4 text-info" />}
                    label="Computed (unverified)"
                    value={enrichedStats().hashComputed}
                  />
                </Show>
                <Show when={enrichedStats().hashMismatch > 0}>
                  <SummaryRow
                    icon={<HiOutlineXCircle class="w-4 h-4 text-error" />}
                    label="Mismatch"
                    value={enrichedStats().hashMismatch}
                  />
                </Show>
                <Show when={enrichedStats().hashNone > 0}>
                  <SummaryRow
                    icon={<span class="w-4 h-4 text-txt-muted text-center text-xs">—</span>}
                    label="No hash"
                    value={enrichedStats().hashNone}
                  />
                </Show>
              </div>
            </Show>

            {/* COC status breakdown */}
            <Show when={enrichedStats().coc > 0}>
              <SectionHeader label="COC Status" />
              <div class="flex flex-col gap-1.5">
                <Show when={enrichedStats().cocLocked > 0}>
                  <SummaryRow
                    icon={<span class="w-4 h-4 text-center text-success text-xs font-bold">🔒</span>}
                    label="Locked"
                    value={enrichedStats().cocLocked}
                  />
                </Show>
                <Show when={enrichedStats().cocDraft > 0}>
                  <SummaryRow
                    icon={<span class="w-4 h-4 text-center text-warning text-xs">✏️</span>}
                    label="Draft"
                    value={enrichedStats().cocDraft}
                  />
                </Show>
                <Show when={enrichedStats().cocVoided > 0}>
                  <SummaryRow
                    icon={<span class="w-4 h-4 text-center text-error text-xs">✕</span>}
                    label="Voided"
                    value={enrichedStats().cocVoided}
                  />
                </Show>
              </div>
            </Show>

            {/* Container type breakdown */}
            <Show when={enrichedStats().containerTypes.size > 0}>
              <SectionHeader label="Container Types" />
              <div class="flex flex-wrap gap-2 px-2">
                <For each={Array.from(enrichedStats().containerTypes.entries())}>
                  {([ct, count]) => (
                    <span
                      class="text-[10px] font-bold uppercase px-1.5 py-0.5 rounded"
                      classList={{
                        "text-type-e01 bg-type-e01/10": ct === "E01" || ct === "EX01",
                        "text-type-ad1 bg-type-ad1/10": ct === "AD1",
                        "text-type-l01 bg-type-l01/10": ct === "L01" || ct === "LX01",
                        "text-type-ufed bg-type-ufed/10": ct === "UFED",
                        "text-type-raw bg-type-raw/10": !["E01","EX01","AD1","L01","LX01","UFED"].includes(ct),
                      }}
                    >
                      {ct} ×{count}
                    </span>
                  )}
                </For>
              </div>
            </Show>

            {/* Total evidence size */}
            <Show when={enrichedStats().totalSizeBytes > 0}>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded bg-bg-secondary mt-1">
                <span class="flex-1 text-xs text-txt-muted">Total Evidence Size</span>
                <span class="text-xs font-medium text-txt">{formatSize(enrichedStats().totalSizeBytes)}</span>
              </div>
            </Show>
          </div>
        </Show>
      </div>
    </div>
  );
}

// =============================================================================
// Node Detail View — shows full metadata for a selected tree node
// =============================================================================

function NodeDetailView(props: { node: LinkedDataNode }) {
  const meta = () => props.node.metadata;

  return (
    <div class="flex flex-col gap-2 text-xs">
      {/* Node type label */}
      <div class="flex items-center gap-1.5 text-txt-muted">
        <span class="uppercase tracking-wider font-medium text-[10px]">
          {props.node.type.replace("-", " ")}
        </span>
        <Show when={props.node.sublabel}>
          <span class="text-txt-muted">— {props.node.sublabel}</span>
        </Show>
      </div>

      <Show when={meta()} fallback={
        <p class="text-txt-muted italic">No metadata available for this node.</p>
      }>
        {(m) => (
          <>
            {/* Evidence file details */}
            <Show when={props.node.type === "evidence-file"}>
              <CollapsibleGroup title="Container">
                <OptionalMetadataRow label="Type" value={m().containerType?.toUpperCase()} />
                <OptionalMetadataRow label="Size" value={m().totalSize ? formatSize(m().totalSize!) : undefined} />
                <OptionalMetadataRow label="Segments" value={m().segmentCount ? String(m().segmentCount) : undefined} />
                <OptionalMetadataRow label="Discovered" value={formatDate(m().discoveredAt)} />
              </CollapsibleGroup>
              <Show when={m().hashStatus && m().hashStatus !== "none"}>
                <CollapsibleGroup title="Hash Integrity">
                  <div class="flex items-center gap-1.5">
                    {m().hashStatus === "verified" && <HiOutlineCheckCircle class="w-3.5 h-3.5 text-success shrink-0" />}
                    {m().hashStatus === "mismatch" && <HiOutlineXCircle class="w-3.5 h-3.5 text-error shrink-0" />}
                    {m().hashStatus === "computed" && <HiOutlineFingerPrint class="w-3.5 h-3.5 text-info shrink-0" />}
                    <span classList={{
                      "text-success": m().hashStatus === "verified",
                      "text-error": m().hashStatus === "mismatch",
                      "text-info": m().hashStatus === "computed",
                    }}>
                      {m().hashStatus === "verified" ? "Verified" : m().hashStatus === "mismatch" ? "MISMATCH" : "Computed"}
                    </span>
                  </div>
                  <OptionalMetadataRow label="Algorithm" value={m().hashAlgorithm} />
                  <OptionalMetadataRow label="Value" value={m().hashValue} mono />
                </CollapsibleGroup>
              </Show>
            </Show>

            {/* Collected item details */}
            <Show when={props.node.type === "collected-item"}>
              <CollapsibleGroup title="Device Identification">
                <OptionalMetadataRow label="Device Type" value={m().deviceType} />
                <OptionalMetadataRow label="Brand" value={m().brand} />
                <OptionalMetadataRow label="Make" value={m().make} />
                <OptionalMetadataRow label="Model" value={m().model} />
                <OptionalMetadataRow label="Serial Number" value={m().serialNumber} mono />
                <OptionalMetadataRow label="IMEI" value={m().imei} mono />
              </CollapsibleGroup>
              <CollapsibleGroup title="Forensic Acquisition">
                <OptionalMetadataRow label="Image Format" value={m().imageFormat} />
                <OptionalMetadataRow label="Method" value={m().acquisitionMethod} />
                <OptionalMetadataRow label="Date" value={formatDate(m().acquisitionDate)} />
              </CollapsibleGroup>
              <CollapsibleGroup title="Item Details">
                <OptionalMetadataRow label="Type" value={m().itemType} />
                <OptionalMetadataRow label="Condition" value={m().condition} />
                <OptionalMetadataRow label="Packaging" value={m().packaging} />
                <OptionalMetadataRow label="Found Location" value={m().foundLocation} />
              </CollapsibleGroup>
              <Show when={m().notes}>
                <CollapsibleGroup title="Notes">
                  <p class="text-txt-secondary whitespace-pre-wrap">{m().notes}</p>
                </CollapsibleGroup>
              </Show>
            </Show>

            {/* COC details */}
            <Show when={props.node.type === "coc"}>
              <CollapsibleGroup title="Chain of Custody">
                <OptionalMetadataRow label="Status" value={m().cocStatus} />
                <OptionalMetadataRow label="Submitted By" value={m().submittedBy} />
                <OptionalMetadataRow label="Received By" value={m().receivedBy} />
                <OptionalMetadataRow label="Collection Method" value={m().collectionMethod} />
                <OptionalMetadataRow label="Storage Location" value={m().storageLocation} />
                <OptionalMetadataRow label="Acquisition Date" value={formatDate(m().acquisitionDate)} />
              </CollapsibleGroup>
              <Show when={m().make || m().model || m().serialNumber}>
                <CollapsibleGroup title="Item Identification">
                  <OptionalMetadataRow label="Make" value={m().make} />
                  <OptionalMetadataRow label="Model" value={m().model} />
                  <OptionalMetadataRow label="Serial Number" value={m().serialNumber} mono />
                </CollapsibleGroup>
              </Show>
            </Show>

            {/* Collection root details */}
            <Show when={props.node.type === "collection"}>
              <CollapsibleGroup title="Collection Event">
                <OptionalMetadataRow label="Date" value={formatDate(m().collectionDate)} />
                <OptionalMetadataRow label="Location" value={m().collectionLocation} />
                <OptionalMetadataRow label="Officer" value={m().collectingOfficer} />
                <OptionalMetadataRow label="Authorization" value={m().authorization} />
                <OptionalMetadataRow label="Status" value={m().status} />
              </CollapsibleGroup>
            </Show>
          </>
        )}
      </Show>
    </div>
  );
}

// =============================================================================
// Shared UI sub-components
// =============================================================================

// =============================================================================
// Formatting helpers
// =============================================================================

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function formatDate(iso?: string): string | undefined {
  if (!iso) return undefined;
  try {
    const d = new Date(iso);
    if (isNaN(d.getTime())) return iso;
    return d.toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
  } catch {
    return iso;
  }
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
