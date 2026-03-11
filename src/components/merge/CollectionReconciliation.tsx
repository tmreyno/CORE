// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * CollectionReconciliation — Review step section for comparing and reconciling
 * collection data when merging projects with overlapping evidence collections.
 *
 * Detects potential duplicate collections (same case number or overlapping dates)
 * and lets the user choose which version to keep for each conflict.
 *
 * Also shows non-conflicting incoming collections that can be included/excluded.
 */

import { Component, For, Show, createSignal, createMemo } from "solid-js";
import {
  HiOutlineArchiveBoxArrowDown,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
} from "../icons";
import type {
  ProjectMergeSummary,
  CollectionConflict,
  CollectionWithSource,
  ConflictResolution,
  ReconciliationChoices,
  MergeCollectionSummary,
} from "./types";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

export interface CollectionReconciliationProps {
  summaries: ProjectMergeSummary[];
  /** The pinned current project path (merge-into-open mode) */
  currentProjectPath: string;
  /** Current conflict resolution choices */
  choices: ReconciliationChoices;
  /** Called when user changes a resolution choice */
  onChoiceChange: (key: string, resolution: ConflictResolution) => void;
  /** Set of incoming collection IDs the user has excluded */
  excludedIds: Set<string>;
  /** Toggle include/exclude for a non-conflicting incoming collection */
  onToggleExcluded: (id: string) => void;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Detect potential duplicate collections between current and incoming projects */
export function detectConflicts(
  summaries: ProjectMergeSummary[],
  currentProjectPath: string,
): CollectionConflict[] {
  const currentSummary = summaries.find((s) => s.cffxPath === currentProjectPath);
  if (!currentSummary) return [];

  const incomingSummaries = summaries.filter((s) => s.cffxPath !== currentProjectPath);
  const conflicts: CollectionConflict[] = [];

  for (const currentCol of currentSummary.collections) {
    for (const incoming of incomingSummaries) {
      for (const incomingCol of incoming.collections) {
        const reason = matchCollections(currentCol, incomingCol);
        if (reason) {
          conflicts.push({
            key: `${currentCol.id}::${incomingCol.id}`,
            current: {
              collection: currentCol,
              projectName: currentSummary.name,
              cffxPath: currentSummary.cffxPath,
            },
            incoming: {
              collection: incomingCol,
              projectName: incoming.name,
              cffxPath: incoming.cffxPath,
            },
            matchReason: reason,
          });
        }
      }
    }
  }

  return conflicts;
}

/** Check if two collections likely represent the same real-world event */
function matchCollections(
  a: MergeCollectionSummary,
  b: MergeCollectionSummary,
): string | null {
  // Same ID — INSERT OR IGNORE handles this, no user action needed
  if (a.id === b.id) return null;

  // Same case number (non-empty, case-insensitive)
  if (
    a.caseNumber && b.caseNumber &&
    a.caseNumber.toLowerCase() === b.caseNumber.toLowerCase()
  ) {
    return `Same case number: ${a.caseNumber}`;
  }

  // Same date and officer (both non-empty)
  if (
    a.collectionDate && b.collectionDate &&
    a.collectingOfficer && b.collectingOfficer &&
    a.collectionDate.slice(0, 10) === b.collectionDate.slice(0, 10) &&
    a.collectingOfficer.toLowerCase() === b.collectingOfficer.toLowerCase()
  ) {
    return `Same date and officer: ${a.collectionDate.slice(0, 10)}, ${a.collectingOfficer}`;
  }

  // Same location and date
  if (
    a.collectionDate && b.collectionDate &&
    a.collectionLocation && b.collectionLocation &&
    a.collectionDate.slice(0, 10) === b.collectionDate.slice(0, 10) &&
    a.collectionLocation.toLowerCase() === b.collectionLocation.toLowerCase()
  ) {
    return `Same date and location: ${a.collectionDate.slice(0, 10)}, ${a.collectionLocation}`;
  }

  return null;
}

/** Get all incoming collection IDs that are involved in a conflict */
function getConflictedIncomingIds(conflicts: CollectionConflict[]): Set<string> {
  return new Set(conflicts.map((c) => c.incoming.collection.id));
}

/** Get all non-conflicting incoming collections */
export function getNonConflictingIncoming(
  summaries: ProjectMergeSummary[],
  currentProjectPath: string,
  conflicts: CollectionConflict[],
): CollectionWithSource[] {
  const conflictedIds = getConflictedIncomingIds(conflicts);
  const result: CollectionWithSource[] = [];

  for (const s of summaries) {
    if (s.cffxPath === currentProjectPath) continue;
    for (const col of s.collections) {
      if (!conflictedIds.has(col.id)) {
        result.push({ collection: col, projectName: s.name, cffxPath: s.cffxPath });
      }
    }
  }

  return result;
}

// ---------------------------------------------------------------------------
// Sub-component: Collection card (compact)
// ---------------------------------------------------------------------------

const CollectionCard: Component<{
  col: MergeCollectionSummary;
  projectName: string;
  highlight?: boolean;
}> = (props) => (
  <div
    class="p-2 rounded-lg border text-xs"
    classList={{
      "bg-accent/5 border-accent/30": props.highlight,
      "bg-bg-secondary border-border": !props.highlight,
    }}
  >
    <div class="flex items-center justify-between mb-1">
      <span class="font-medium text-txt">{props.col.caseNumber || "No case #"}</span>
      <span class="badge" style="font-size: 10px; padding: 1px 5px;">{props.col.status}</span>
    </div>
    <div class="text-txt-muted text-2xs mb-1">From: {props.projectName}</div>
    <div class="flex flex-wrap gap-x-3 gap-y-0.5 text-txt-muted">
      <Show when={props.col.collectingOfficer}>
        <span>Officer: {props.col.collectingOfficer}</span>
      </Show>
      <Show when={props.col.collectionDate}>
        <span>{props.col.collectionDate.slice(0, 10)}</span>
      </Show>
      <span>{props.col.itemCount} items</span>
      <Show when={props.col.collectionLocation}>
        <span>{props.col.collectionLocation}</span>
      </Show>
    </div>
  </div>
);

// ---------------------------------------------------------------------------
// Main Component
// ---------------------------------------------------------------------------

export const CollectionReconciliation: Component<CollectionReconciliationProps> = (props) => {
  const [expanded, setExpanded] = createSignal(true);

  const conflicts = createMemo(() =>
    detectConflicts(props.summaries, props.currentProjectPath),
  );

  const nonConflicting = createMemo(() =>
    getNonConflictingIncoming(props.summaries, props.currentProjectPath, conflicts()),
  );

  const hasContent = createMemo(() => conflicts().length > 0 || nonConflicting().length > 0);

  // Count incoming collections across all non-current projects
  const totalIncoming = createMemo(() => {
    let count = 0;
    for (const s of props.summaries) {
      if (s.cffxPath !== props.currentProjectPath) count += s.collections.length;
    }
    return count;
  });

  return (
    <Show when={totalIncoming() > 0}>
      <div class="p-3 rounded-lg bg-bg-panel border border-border">
        <button
          class="flex items-center gap-2 w-full text-left"
          onClick={() => setExpanded((e) => !e)}
        >
          <Show when={expanded()} fallback={<HiOutlineChevronRight class="w-3.5 h-3.5 text-txt-muted" />}>
            <HiOutlineChevronDown class="w-3.5 h-3.5 text-txt-muted" />
          </Show>
          <HiOutlineArchiveBoxArrowDown class="w-4 h-4 text-accent" />
          <h3 class="text-sm font-semibold text-txt flex-1">
            Collection Data Reconciliation
          </h3>
          <Show when={conflicts().length > 0}>
            <span class="badge badge-warning" style="font-size: 10px;">
              {conflicts().length} potential {conflicts().length === 1 ? "conflict" : "conflicts"}
            </span>
          </Show>
        </button>

        <Show when={expanded()}>
          <div class="mt-3 col gap-4">
            {/* Conflicts section */}
            <Show when={conflicts().length > 0}>
              <div class="col gap-3">
                <div class="flex items-center gap-2">
                  <HiOutlineExclamationTriangle class="w-3.5 h-3.5 text-warning" />
                  <span class="text-xs font-medium text-txt">
                    Potential Duplicate Collections
                  </span>
                </div>
                <p class="text-xs text-txt-muted ml-5">
                  These collections appear to describe the same event. Choose which version to keep
                  for each conflict. The other version will be excluded from the merge.
                </p>

                <For each={conflicts()}>
                  {(conflict) => {
                    const choice = () => props.choices[conflict.key] || "keep-current";
                    return (
                      <div class="ml-5 p-3 rounded-lg bg-bg border border-border col gap-2">
                        {/* Match reason */}
                        <div class="text-xs text-warning font-medium">
                          {conflict.matchReason}
                        </div>

                        {/* Side-by-side comparison */}
                        <div class="grid grid-cols-2 gap-3">
                          {/* Current project's collection */}
                          <div class="col gap-1.5">
                            <label class="flex items-center gap-2 text-xs font-medium text-txt cursor-pointer">
                              <input
                                type="radio"
                                name={`conflict-${conflict.key}`}
                                checked={choice() === "keep-current"}
                                onClick={() => props.onChoiceChange(conflict.key, "keep-current")}
                                class="accent-accent"
                              />
                              Keep Current
                            </label>
                            <CollectionCard
                              col={conflict.current.collection}
                              projectName={conflict.current.projectName}
                              highlight={choice() === "keep-current"}
                            />
                          </div>

                          {/* Incoming project's collection */}
                          <div class="col gap-1.5">
                            <label class="flex items-center gap-2 text-xs font-medium text-txt cursor-pointer">
                              <input
                                type="radio"
                                name={`conflict-${conflict.key}`}
                                checked={choice() === "use-incoming"}
                                onClick={() => props.onChoiceChange(conflict.key, "use-incoming")}
                                class="accent-accent"
                              />
                              Use Incoming
                            </label>
                            <CollectionCard
                              col={conflict.incoming.collection}
                              projectName={conflict.incoming.projectName}
                              highlight={choice() === "use-incoming"}
                            />
                          </div>
                        </div>
                      </div>
                    );
                  }}
                </For>
              </div>
            </Show>

            {/* Non-conflicting incoming collections */}
            <Show when={nonConflicting().length > 0}>
              <div class="col gap-2">
                <div class="flex items-center gap-2">
                  <HiOutlineCheckCircle class="w-3.5 h-3.5 text-success" />
                  <span class="text-xs font-medium text-txt">
                    Additional Incoming Collections ({nonConflicting().length})
                  </span>
                </div>
                <p class="text-xs text-txt-muted ml-5">
                  These collections don't conflict with existing data. Uncheck any you don't want to include.
                </p>

                <div class="ml-5 col gap-1.5">
                  <For each={nonConflicting()}>
                    {(item) => {
                      const isExcluded = () => props.excludedIds.has(item.collection.id);
                      return (
                        <label
                          class="flex items-center gap-2 p-2 rounded-lg border cursor-pointer transition-colors"
                          classList={{
                            "bg-bg-secondary border-border": !isExcluded(),
                            "bg-bg border-border/50 opacity-50": isExcluded(),
                          }}
                        >
                          <input
                            type="checkbox"
                            checked={!isExcluded()}
                            onClick={() => props.onToggleExcluded(item.collection.id)}
                            class="accent-accent"
                          />
                          <div class="flex-1 min-w-0">
                            <div class="flex items-center gap-2 text-xs">
                              <span class="font-medium text-txt">
                                {item.collection.caseNumber || "No case #"}
                              </span>
                              <span class="text-txt-muted">from {item.projectName}</span>
                              <span class="badge" style="font-size: 10px; padding: 1px 5px;">
                                {item.collection.status}
                              </span>
                            </div>
                            <div class="flex gap-3 text-2xs text-txt-muted mt-0.5">
                              <Show when={item.collection.collectingOfficer}>
                                <span>Officer: {item.collection.collectingOfficer}</span>
                              </Show>
                              <Show when={item.collection.collectionDate}>
                                <span>{item.collection.collectionDate.slice(0, 10)}</span>
                              </Show>
                              <span>{item.collection.itemCount} items</span>
                            </div>
                          </div>
                        </label>
                      );
                    }}
                  </For>
                </div>
              </div>
            </Show>

            {/* No content fallback — shouldn't happen due to outer Show */}
            <Show when={!hasContent()}>
              <p class="text-xs text-txt-muted ml-5">
                No collection data to reconcile.
              </p>
            </Show>
          </div>
        </Show>
      </div>
    </Show>
  );
};
