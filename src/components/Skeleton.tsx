// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, Show, type JSX } from "solid-js";

interface SkeletonProps {
  /** Width of the skeleton (CSS value) */
  width?: string;
  /** Height of the skeleton (CSS value) */
  height?: string;
  /** Border radius variant */
  rounded?: "none" | "sm" | "md" | "lg" | "full";
  /** Additional CSS classes */
  class?: string;
}

/**
 * Basic skeleton placeholder with shimmer animation
 */
export function Skeleton(props: SkeletonProps) {
  const roundedClass = () => {
    switch (props.rounded) {
      case "none": return "";
      case "sm": return "rounded-sm";
      case "lg": return "rounded-lg";
      case "full": return "rounded-full";
      default: return "rounded";
    }
  };

  return (
    <div
      class={`bg-bg-hover relative overflow-hidden animate-pulse ${roundedClass()} ${props.class ?? ""}`}
      style={{
        width: props.width ?? "100%",
        height: props.height ?? "1rem",
      }}
    />
  );
}

/**
 * Skeleton for text content - multiple lines
 */
export function SkeletonText(props: { lines?: number; class?: string }) {
  const lines = () => props.lines ?? 3;
  
  return (
    <div class={`flex flex-col gap-2 ${props.class ?? ""}`}>
      <For each={Array(lines()).fill(0)}>
        {(_, i) => (
          <Skeleton
            width={i() === lines() - 1 ? "60%" : "100%"}
            height="0.875rem"
          />
        )}
      </For>
    </div>
  );
}

/**
 * Skeleton for file row in FilePanel
 */
export function SkeletonFileRow() {
  return (
    <div class="flex items-center gap-3 px-3 py-2.5 border-b border-border-muted">
      {/* Checkbox */}
      <Skeleton width="1rem" height="1rem" rounded="sm" />
      {/* Type icon */}
      <Skeleton width="1.5rem" height="1.5rem" rounded="md" />
      {/* Filename */}
      <div class="flex-1 flex flex-col gap-1">
        <Skeleton width="70%" height="0.875rem" />
        <Skeleton width="40%" height="0.75rem" />
      </div>
      {/* Size */}
      <Skeleton width="4rem" height="0.75rem" />
      {/* Hash button */}
      <Skeleton width="2rem" height="1.5rem" rounded="md" />
    </div>
  );
}

/**
 * Skeleton for tree node in TreePanel
 */
export function SkeletonTreeNode(props: { level?: number }) {
  const indent = () => (props.level ?? 0) * 16;
  
  return (
    <div 
      class="flex items-center gap-2 px-2 py-1.5"
      style={{ "padding-left": `${indent() + 8}px` }}
    >
      {/* Expand arrow */}
      <Skeleton width="0.75rem" height="0.75rem" rounded="sm" />
      {/* Icon */}
      <Skeleton width="1rem" height="1rem" rounded="sm" />
      {/* Name */}
      <Skeleton width="60%" height="0.875rem" />
    </div>
  );
}

/**
 * Skeleton for hex viewer content
 */
export function SkeletonHexView() {
  return (
    <div class="flex flex-col gap-1 p-3 font-mono">
      <For each={Array(16).fill(0)}>
        {() => (
          <div class="flex items-center gap-4">
            {/* Offset */}
            <Skeleton width="4rem" height="1rem" />
            {/* Hex bytes */}
            <div class="flex gap-1">
              <For each={Array(16).fill(0)}>
                {() => <Skeleton width="1.25rem" height="1rem" />}
              </For>
            </div>
            {/* ASCII */}
            <Skeleton width="8rem" height="1rem" />
          </div>
        )}
      </For>
    </div>
  );
}

/**
 * Skeleton for metadata panel
 */
export function SkeletonMetadata() {
  return (
    <div class="flex flex-col gap-4 p-4">
      {/* Header */}
      <div class="flex items-center gap-3">
        <Skeleton width="3rem" height="3rem" rounded="lg" />
        <div class="flex-1 flex flex-col gap-2">
          <Skeleton width="60%" height="1.25rem" />
          <Skeleton width="40%" height="0.875rem" />
        </div>
      </div>
      
      {/* Divider */}
      <div class="h-px bg-border" />
      
      {/* Metadata rows */}
      <For each={Array(6).fill(0)}>
        {() => (
          <div class="flex items-center justify-between py-1">
            <Skeleton width="30%" height="0.875rem" />
            <Skeleton width="50%" height="0.875rem" />
          </div>
        )}
      </For>
    </div>
  );
}

/**
 * Skeleton wrapper for panel loading state
 */
export function SkeletonPanel(props: { 
  children: JSX.Element;
  loading: boolean;
  fallback?: JSX.Element;
}) {
  return (
    <Show when={!props.loading} fallback={props.fallback}>
      {props.children}
    </Show>
  );
}

/**
 * Full-panel loading skeleton with centered spinner
 */
export function SkeletonLoader(props: { message?: string }) {
  return (
    <div class="flex flex-col items-center justify-center h-full gap-3 text-txt-muted">
      <div class="w-8 h-8 border-2 border-border rounded-full border-t-accent animate-spin" />
      <Show when={props.message}>
        <span class="text-sm">{props.message}</span>
      </Show>
    </div>
  );
}

/**
 * Tree skeleton - simulates a hierarchical tree view
 */
export function SkeletonTree(props: { items?: number; maxDepth?: number; class?: string }) {
  const itemCount = props.items ?? 10;
  const maxDepth = props.maxDepth ?? 3;
  
  // Generate pseudo-random depths
  const items = () => {
    const result: number[] = [];
    let currentDepth = 0;
    for (let i = 0; i < itemCount; i++) {
      if (Math.random() > 0.6 && currentDepth < maxDepth) currentDepth++;
      else if (Math.random() > 0.5 && currentDepth > 0) currentDepth--;
      result.push(currentDepth);
    }
    return result;
  };
  
  return (
    <div class={`space-y-1 ${props.class ?? ""}`}>
      <For each={items()}>
        {(depth) => <SkeletonTreeNode level={depth} />}
      </For>
    </div>
  );
}

/**
 * Table skeleton - simulates a data table
 */
export function SkeletonTable(props: { 
  rows?: number; 
  columns?: number; 
  showHeader?: boolean;
  class?: string;
}) {
  const rowCount = props.rows ?? 5;
  const colCount = props.columns ?? 4;
  
  return (
    <div class={`overflow-hidden rounded-lg border border-border ${props.class ?? ""}`}>
      {props.showHeader !== false && (
        <div class="flex gap-4 p-3 bg-bg-card border-b border-border">
          <For each={Array(colCount).fill(0)}>
            {() => <Skeleton width={`${100/colCount}%`} height="1rem" />}
          </For>
        </div>
      )}
      <For each={Array(rowCount).fill(0)}>
        {(_, i) => (
          <div class={`flex gap-4 p-3 ${i() % 2 === 0 ? 'bg-bg-card/50' : ''}`}>
            <For each={Array(colCount).fill(0)}>
              {() => <Skeleton width={`${70 + Math.random() * 30}%`} height="0.875rem" />}
            </For>
          </div>
        )}
      </For>
    </div>
  );
}

/**
 * Card skeleton
 */
export function SkeletonCard(props: { showImage?: boolean; lines?: number; class?: string }) {
  return (
    <div class={`bg-bg-card rounded-lg border border-border p-4 space-y-3 ${props.class ?? ""}`}>
      <Show when={props.showImage}>
        <Skeleton height="8rem" rounded="md" />
      </Show>
      <Skeleton width="60%" height="1.25rem" />
      <SkeletonText lines={props.lines ?? 2} />
    </div>
  );
}

/**
 * Progress skeleton - animated bar
 */
export function SkeletonProgress(props: { class?: string }) {
  return (
    <div class={`h-2 bg-bg-hover rounded-full overflow-hidden ${props.class ?? ""}`}>
      <div 
        class="h-full bg-gradient-to-r from-transparent via-accent/30 to-transparent animate-[shimmer_1.5s_infinite]"
        style={{ width: "50%", animation: "shimmer 1.5s infinite" }}
      />
    </div>
  );
}


