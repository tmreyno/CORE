// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AcquireCollectionSummary — Right-panel summary of evidence collections
 * for the Acquire dashboard. Shows all collections and their items as
 * compact cards, providing a live overview as collections are created
 * from acquisitions.
 */

import {
  Component,
  Show,
  For,
  createSignal,
  createEffect,
  createMemo,
  on,
  type Accessor,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineArchiveBoxArrowDown,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineClock,
  HiOutlineDocumentText,
} from "../icons";
import type { DbEvidenceCollection, DbCollectedItem } from "../../types/projectDb";
import { logger } from "../../utils/logger";
import { isTauri } from "../../utils/platform";

const log = logger.scope("AcquireCollectionSummary");

// =============================================================================
// Types
// =============================================================================

export interface AcquireCollectionSummaryProps {
  hasProject: Accessor<boolean>;
  onViewCollection?: (collectionId: string) => void;
  onNewCollection?: () => void;
  /** When true, skips the outer panel wrapper (used when embedded in the right panel) */
  embedded?: boolean;
}

interface CollectionWithItems {
  collection: DbEvidenceCollection;
  items: DbCollectedItem[];
}

// =============================================================================
// Helpers
// =============================================================================

function formatDate(iso: string): string {
  if (!iso) return "";
  try {
    const d = new Date(iso);
    return d.toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" });
  } catch {
    return iso;
  }
}

function statusColor(status: string): string {
  switch (status) {
    case "complete":
    case "locked":
      return "text-success";
    case "draft":
      return "text-amber-400";
    default:
      return "text-txt-muted";
  }
}

// =============================================================================
// Component
// =============================================================================

const AcquireCollectionSummary: Component<AcquireCollectionSummaryProps> = (props) => {
  const [collections, setCollections] = createSignal<DbEvidenceCollection[]>([]);
  const [allItems, setAllItems] = createSignal<DbCollectedItem[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [expandedIds, setExpandedIds] = createSignal<Set<string>>(new Set());

  const loadData = async (retryCount: number = 0) => {
    if (!props.hasProject()) {
      setCollections([]);
      setAllItems([]);
      return;
    }
    if (!isTauri) {
      setCollections([]);
      setAllItems([]);
      setLoading(false);
      return;
    }
    setLoading(true);
    try {
      const [cols, items] = await Promise.all([
        invoke<DbEvidenceCollection[]>("project_db_get_evidence_collections", {
          caseNumber: null,
        }),
        invoke<DbCollectedItem[]>("project_db_get_all_collected_items"),
      ]);
      setCollections(cols);
      setAllItems(items);
    } catch (e) {
      const msg = String(e);
      if (msg.includes("No project database") && retryCount < 3) {
        // DB not open yet — project signal fires before project_db_open completes
        setTimeout(() => loadData(retryCount + 1), 500);
        return;
      }
      log.warn("Failed to load evidence collections:", e);
      setCollections([]);
      setAllItems([]);
    } finally {
      setLoading(false);
    }
  };

  const refreshData = () => loadData(0);

  // Reload when project loads/unloads
  createEffect(on(() => props.hasProject(), () => loadData(0)));

  // Group items by collection
  const grouped = createMemo<CollectionWithItems[]>(() => {
    const itemMap = new Map<string, DbCollectedItem[]>();
    for (const item of allItems()) {
      const list = itemMap.get(item.collectionId) ?? [];
      list.push(item);
      itemMap.set(item.collectionId, list);
    }
    return collections()
      .sort((a, b) => (b.modifiedAt || "").localeCompare(a.modifiedAt || ""))
      .map((c) => ({ collection: c, items: itemMap.get(c.id) ?? [] }));
  });

  const toggleExpand = (id: string) => {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  return (
    <div class={props.embedded ? "flex flex-col" : "flex flex-col h-full bg-bg"}>
      {/* Header */}
      <div class={props.embedded ? "flex items-center gap-small px-2 py-1.5" : "flex items-center gap-small px-3 py-2 border-b border-border bg-bg-secondary shrink-0"}>
        <HiOutlineArchiveBoxArrowDown class="w-icon-sm h-icon-sm text-accent shrink-0" />
        <span class="text-xs font-medium text-txt flex-1">Evidence Collections</span>
        <Show when={grouped().length > 0}>
          <span class="text-2xs text-txt-muted">{grouped().length}</span>
        </Show>
      </div>

      {/* Body */}
      <div class={props.embedded ? "flex-1 overflow-y-auto px-2 py-1.5" : "flex-1 overflow-y-auto"}>
        <Show
          when={props.hasProject()}
          fallback={
            <div class="flex flex-col items-center justify-center py-8 text-txt-muted text-sm gap-2">
              <HiOutlineArchiveBoxArrowDown class="w-8 h-8 opacity-20" />
              <p>Open a project to see evidence collections</p>
            </div>
          }
        >
          <Show
            when={!loading()}
            fallback={
              <div class="flex flex-col items-center justify-center py-8 text-txt-muted text-sm gap-2">
                <p>Loading…</p>
              </div>
            }
          >
            <Show
              when={grouped().length > 0}
              fallback={
                <div class="flex flex-col items-center justify-center py-8 text-txt-muted text-sm gap-2">
                  <HiOutlineDocumentText class="w-8 h-8 opacity-20" />
                  <p>No evidence collections yet</p>
                  <p class="text-2xs">Collections are created automatically when you complete acquisitions</p>
                </div>
              }
            >
              <div class="flex flex-col">
                <For each={grouped()}>
                  {(group) => {
                    const isExpanded = () => expandedIds().has(group.collection.id);
                    const col = group.collection;
                    const itemCount = () => group.items.length;

                    return (
                      <div class="border-b border-border last:border-b-0">
                        {/* Collection header — clickable to expand */}
                        <button
                          class="flex items-center gap-small w-full text-left px-2 py-1.5 hover:bg-bg-hover cursor-pointer"
                          onClick={() => toggleExpand(col.id)}
                        >
                          <div class="shrink-0 w-icon-compact">
                            <Show when={isExpanded()} fallback={
                              <HiOutlineChevronRight class="w-icon-micro h-icon-micro text-txt-muted" />
                            }>
                              <HiOutlineChevronDown class="w-icon-micro h-icon-micro text-txt-muted" />
                            </Show>
                          </div>
                          <div class="flex-1 min-w-0">
                            <div class="flex items-center gap-1.5">
                              <span class={`text-2xs font-medium uppercase ${statusColor(col.status)}`}>
                                {col.status}
                              </span>
                              <Show when={col.caseNumber}>
                                <span class="text-2xs text-txt-muted truncate">
                                  #{col.caseNumber}
                                </span>
                              </Show>
                            </div>
                            <div class="flex items-center gap-1 mt-0.5">
                              <Show when={col.collectingOfficer}>
                                <span class="text-xs text-txt truncate">{col.collectingOfficer}</span>
                              </Show>
                              <Show when={!col.collectingOfficer && col.collectionDate}>
                                <span class="text-xs text-txt truncate">{formatDate(col.collectionDate)}</span>
                              </Show>
                              <Show when={!col.collectingOfficer && !col.collectionDate}>
                                <span class="text-xs text-txt-muted italic">No details</span>
                              </Show>
                            </div>
                          </div>
                          <span class="text-2xs text-txt-muted shrink-0">
                            {itemCount()} item{itemCount() !== 1 ? "s" : ""}
                          </span>
                        </button>

                        {/* Expanded items */}
                        <Show when={isExpanded()}>
                          <div class="flex flex-col gap-0.5 px-2 pb-1.5">
                            <Show when={col.collectionDate}>
                              <div class="flex items-center gap-1 px-1">
                                <HiOutlineClock class="w-icon-micro h-icon-micro text-txt-muted shrink-0" />
                                <span class="text-2xs text-txt-muted">{formatDate(col.collectionDate)}</span>
                              </div>
                            </Show>
                            <Show when={col.collectionLocation}>
                              <div class="flex items-center gap-1 px-1">
                                <span class="text-2xs text-txt-muted truncate">📍 {col.collectionLocation}</span>
                              </div>
                            </Show>

                            <For each={group.items}>
                              {(item) => (
                                <div
                                  class="flex flex-col px-2 py-1 rounded hover:bg-bg-hover cursor-pointer"
                                  onClick={() => props.onViewCollection?.(col.id)}
                                >
                                  <div class="flex items-start gap-1.5 min-w-0">
                                    <Show when={item.itemNumber}>
                                      <span class="text-2xs font-mono text-accent shrink-0">
                                        #{item.itemNumber}
                                      </span>
                                    </Show>
                                    <span class="text-xs text-txt truncate">{item.description || "Untitled item"}</span>
                                  </div>
                                  <div class="flex items-center gap-2 mt-0.5">
                                    <Show when={item.itemType}>
                                      <span class="text-2xs text-txt-muted">{item.itemType}</span>
                                    </Show>
                                    <Show when={item.make || item.model}>
                                      <span class="text-2xs text-txt-muted truncate">
                                        {[item.make, item.model].filter(Boolean).join(" ")}
                                      </span>
                                    </Show>
                                    <Show when={item.serialNumber}>
                                      <span class="text-2xs font-mono text-txt-muted truncate">S/N: {item.serialNumber}</span>
                                    </Show>
                                  </div>
                                  <Show when={item.imageFormat || item.acquisitionMethod}>
                                    <div class="flex items-center gap-2 mt-0.5">
                                      <Show when={item.imageFormat}>
                                        <span class="text-2xs text-accent/70">{item.imageFormat}</span>
                                      </Show>
                                      <Show when={item.acquisitionMethod}>
                                        <span class="text-2xs text-txt-muted">{item.acquisitionMethod}</span>
                                      </Show>
                                    </div>
                                  </Show>
                                </div>
                              )}
                            </For>

                            {/* View full form */}
                            <Show when={props.onViewCollection}>
                              <button
                                class="btn-text text-2xs text-accent w-full text-center py-1"
                                onClick={() => props.onViewCollection?.(col.id)}
                              >
                                View Full Collection
                              </button>
                            </Show>
                          </div>
                        </Show>
                      </div>
                    );
                  }}
                </For>
              </div>
            </Show>
          </Show>
        </Show>
      </div>

      {/* Footer — refresh */}
      <Show when={props.hasProject()}>
        <div class="flex items-center justify-end gap-2 px-3 py-1 border-t border-border shrink-0">
          <button class="btn-text text-2xs" onClick={refreshData}>
            Refresh
          </button>
          <Show when={props.onNewCollection}>
            <button
              class="btn-text text-2xs text-accent"
              onClick={props.onNewCollection}
            >
              + New Collection
            </button>
          </Show>
        </div>
      </Show>
    </div>
  );
};

export default AcquireCollectionSummary;
