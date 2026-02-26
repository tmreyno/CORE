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
 */

import { createSignal, Show, For } from "solid-js";
import {
  HiOutlineArchiveBoxArrowDown,
  HiOutlineArchiveBox,
  HiOutlineDocumentText,
  HiOutlineLink,
  HiOutlineChevronRight,
  HiOutlineChevronDown,
  HiOutlineFingerPrint,
} from "./icons";

// =============================================================================
// Types
// =============================================================================

export interface LinkedDataNode {
  id: string;
  label: string;
  sublabel?: string;
  type: "collection" | "collected-item" | "coc" | "evidence-file";
  children?: LinkedDataNode[];
  linkedId?: string; // FK reference
}

// =============================================================================
// Tree Components
// =============================================================================

export function LinkedDataTree(props: {
  nodes: LinkedDataNode[];
  onNodeClick?: (node: LinkedDataNode) => void;
}) {
  return (
    <div class="flex flex-col gap-0.5">
      <For each={props.nodes}>
        {(node) => <LinkedDataTreeNode node={node} onNodeClick={props.onNodeClick} depth={0} />}
      </For>
    </div>
  );
}

function LinkedDataTreeNode(props: {
  node: LinkedDataNode;
  onNodeClick?: (node: LinkedDataNode) => void;
  depth: number;
}) {
  const [expanded, setExpanded] = createSignal(true);
  const hasChildren = () => (props.node.children?.length ?? 0) > 0;

  const iconForType = () => {
    switch (props.node.type) {
      case "collection": return <HiOutlineArchiveBoxArrowDown class="w-3.5 h-3.5 text-accent" />;
      case "collected-item": return <HiOutlineDocumentText class="w-3.5 h-3.5 text-type-ad1" />;
      case "coc": return <HiOutlineFingerPrint class="w-3.5 h-3.5 text-warning" />;
      case "evidence-file": return <HiOutlineArchiveBox class="w-3.5 h-3.5 text-type-e01" />;
    }
  };

  return (
    <div>
      <div
        class="flex items-center gap-1 px-2 py-1 rounded hover:bg-bg-hover cursor-pointer text-sm select-none"
        style={{ "padding-left": `${props.depth * 12 + 8}px` }}
        onClick={() => {
          if (hasChildren()) setExpanded(!expanded());
          props.onNodeClick?.(props.node);
        }}
      >
        <Show when={hasChildren()} fallback={<span class="w-3.5" />}>
          <Show when={expanded()} fallback={<HiOutlineChevronRight class="w-3.5 h-3.5 text-txt-muted" />}>
            <HiOutlineChevronDown class="w-3.5 h-3.5 text-txt-muted" />
          </Show>
        </Show>
        {iconForType()}
        <span class="text-txt truncate flex-1">{props.node.label}</span>
        <Show when={props.node.sublabel}>
          <span class="text-xs text-txt-muted">{props.node.sublabel}</span>
        </Show>
        <Show when={props.node.linkedId}>
          <HiOutlineLink class="w-3 h-3 text-accent opacity-50" title="Linked record" />
        </Show>
      </div>
      <Show when={hasChildren() && expanded()}>
        <For each={props.node.children!}>
          {(child) => <LinkedDataTreeNode node={child} onNodeClick={props.onNodeClick} depth={props.depth + 1} />}
        </For>
      </Show>
    </div>
  );
}
