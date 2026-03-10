// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * LinkedDataTree — Reusable tree component showing relationships between
 * evidence collections, collected items, COC records, and evidence files.
 *
 * Used in the right panel when a collection tab is active.
 * Each node displays metadata badges (container type, hash status, device IDs)
 * and clicking a node shows its full detail in the LinkedDataPanel detail pane.
 */

import { createSignal, Show, For } from "solid-js";
import {
  HiOutlineArchiveBoxArrowDown,
  HiOutlineArchiveBox,
  HiOutlineDocumentText,
  HiOutlineChevronRight,
  HiOutlineChevronDown,
  HiOutlineFingerPrint,
  HiOutlineCheckCircle,
  HiOutlineXCircle,
  HiOutlineShieldCheck,
} from "./icons";

// =============================================================================
// Types
// =============================================================================

/** Optional metadata attached to tree nodes for richer display */
export interface NodeMetadata {
  // Evidence file metadata
  containerType?: string;      // "e01", "ad1", "l01", "ufed", etc.
  totalSize?: number;          // bytes
  segmentCount?: number;
  discoveredAt?: string;       // ISO date

  // Device identification (from collected item or container)
  deviceType?: string;
  brand?: string;
  make?: string;
  model?: string;
  serialNumber?: string;
  imei?: string;

  // Forensic acquisition
  imageFormat?: string;
  acquisitionMethod?: string;
  acquisitionDate?: string;

  // Hash status
  hashStatus?: "verified" | "mismatch" | "computed" | "none";
  hashAlgorithm?: string;
  hashValue?: string;

  // COC metadata
  cocStatus?: string;           // "draft", "locked", "voided"
  submittedBy?: string;
  receivedBy?: string;
  collectionMethod?: string;
  storageLocation?: string;

  // Collection-level metadata
  collectionDate?: string;
  collectionLocation?: string;
  collectingOfficer?: string;
  authorization?: string;
  status?: string;              // "draft", "complete", "locked"

  // Item metadata
  condition?: string;
  packaging?: string;
  foundLocation?: string;
  itemType?: string;
  notes?: string;
}

export interface LinkedDataNode {
  id: string;
  label: string;
  sublabel?: string;
  type: "collection" | "collected-item" | "coc" | "evidence-file";
  children?: LinkedDataNode[];
  linkedId?: string; // FK reference
  /** Rich metadata for inline badges and detail pane */
  metadata?: NodeMetadata;
}

// =============================================================================
// Tree Components
// =============================================================================

export function LinkedDataTree(props: {
  nodes: LinkedDataNode[];
  onNodeClick?: (node: LinkedDataNode) => void;
  /** Currently selected node ID (for highlight) */
  selectedNodeId?: string;
}) {
  return (
    <div class="flex flex-col gap-0.5">
      <For each={props.nodes}>
        {(node) => (
          <LinkedDataTreeNode
            node={node}
            onNodeClick={props.onNodeClick}
            selectedNodeId={props.selectedNodeId}
            depth={0}
          />
        )}
      </For>
    </div>
  );
}

function LinkedDataTreeNode(props: {
  node: LinkedDataNode;
  onNodeClick?: (node: LinkedDataNode) => void;
  selectedNodeId?: string;
  depth: number;
}) {
  const [expanded, setExpanded] = createSignal(true);
  const hasChildren = () => (props.node.children?.length ?? 0) > 0;
  const isSelected = () => props.selectedNodeId === props.node.id;

  const iconForType = () => {
    switch (props.node.type) {
      case "collection": return <HiOutlineArchiveBoxArrowDown class="w-3.5 h-3.5 text-accent shrink-0" />;
      case "collected-item": return <HiOutlineDocumentText class="w-3.5 h-3.5 text-type-ad1 shrink-0" />;
      case "coc": return <HiOutlineShieldCheck class="w-3.5 h-3.5 text-warning shrink-0" />;
      case "evidence-file": return <HiOutlineArchiveBox class="w-3.5 h-3.5 text-type-e01 shrink-0" />;
    }
  };

  /** Render inline metadata badges based on node type */
  const metadataBadges = () => {
    const meta = props.node.metadata;
    if (!meta) return null;

    return (
      <div class="flex items-center gap-1 shrink-0">
        {/* Container type badge */}
        <Show when={meta.containerType}>
          <span
            class="text-[9px] font-bold uppercase px-1 py-0 rounded"
            classList={{
              "text-type-e01 bg-type-e01/10": meta.containerType === "e01" || meta.containerType === "ex01",
              "text-type-ad1 bg-type-ad1/10": meta.containerType === "ad1",
              "text-type-l01 bg-type-l01/10": meta.containerType === "l01" || meta.containerType === "lx01",
              "text-type-ufed bg-type-ufed/10": meta.containerType === "ufed",
              "text-type-raw bg-type-raw/10": !["e01","ex01","ad1","l01","lx01","ufed"].includes(meta.containerType!),
            }}
          >
            {meta.containerType}
          </span>
        </Show>

        {/* Hash status indicator */}
        <Show when={meta.hashStatus && meta.hashStatus !== "none"}>
          {(() => {
            switch (meta.hashStatus) {
              case "verified":
                return <HiOutlineCheckCircle class="w-3 h-3 text-success" title={`Hash verified (${meta.hashAlgorithm})`} />;
              case "mismatch":
                return <HiOutlineXCircle class="w-3 h-3 text-error" title="Hash mismatch!" />;
              case "computed":
                return <HiOutlineFingerPrint class="w-3 h-3 text-info" title={`Hash computed (${meta.hashAlgorithm})`} />;
              default:
                return null;
            }
          })()}
        </Show>

        {/* Serial number badge */}
        <Show when={meta.serialNumber}>
          <span class="text-[9px] text-txt-muted bg-bg-hover px-1 rounded truncate max-w-[80px]" title={`S/N: ${meta.serialNumber}`}>
            S/N
          </span>
        </Show>

        {/* COC status badge */}
        <Show when={meta.cocStatus}>
          <span
            class="text-[9px] px-1 rounded"
            classList={{
              "text-success bg-success/10": meta.cocStatus === "locked",
              "text-warning bg-warning/10": meta.cocStatus === "draft",
              "text-error bg-error/10": meta.cocStatus === "voided",
            }}
          >
            {meta.cocStatus}
          </span>
        </Show>

        {/* Size badge for evidence files */}
        <Show when={meta.totalSize && meta.totalSize > 0}>
          <span class="text-[9px] text-txt-muted">{formatSizeCompact(meta.totalSize!)}</span>
        </Show>
      </div>
    );
  };

  return (
    <div>
      <div
        class="flex items-center gap-1 px-2 py-1 rounded cursor-pointer text-sm select-none transition-colors"
        classList={{
          "bg-accent/10 hover:bg-accent/15": isSelected(),
          "hover:bg-bg-hover": !isSelected(),
        }}
        style={{ "padding-left": `${props.depth * 12 + 8}px` }}
        onClick={() => {
          if (hasChildren()) setExpanded(!expanded());
          props.onNodeClick?.(props.node);
        }}
      >
        <Show when={hasChildren()} fallback={<span class="w-3.5 shrink-0" />}>
          <Show when={expanded()} fallback={<HiOutlineChevronRight class="w-3.5 h-3.5 text-txt-muted shrink-0" />}>
            <HiOutlineChevronDown class="w-3.5 h-3.5 text-txt-muted shrink-0" />
          </Show>
        </Show>
        {iconForType()}
        <span class="text-txt truncate flex-1 min-w-0">{props.node.label}</span>
        {metadataBadges()}
      </div>
      <Show when={hasChildren() && expanded()}>
        <For each={props.node.children!}>
          {(child) => (
            <LinkedDataTreeNode
              node={child}
              onNodeClick={props.onNodeClick}
              selectedNodeId={props.selectedNodeId}
              depth={props.depth + 1}
            />
          )}
        </For>
      </Show>
    </div>
  );
}

// =============================================================================
// Helpers
// =============================================================================

function formatSizeCompact(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)}K`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)}M`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)}G`;
}
