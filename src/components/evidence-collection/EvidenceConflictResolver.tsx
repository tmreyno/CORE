// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceConflictResolver — Modal UI for resolving field-level conflicts
 * between user-entered evidence collection data and container-extracted metadata.
 *
 * Shows a side-by-side comparison of conflicting fields and lets the user
 * choose which value to keep. Non-selected values are archived as
 * "evidence data alternatives" for forensic auditability.
 */

import {
  Component,
  createSignal,
  Show,
  For,
  createMemo,
} from "solid-js";
import {
  HiOutlineExclamationTriangle,
  HiOutlineCheckCircle,
  HiOutlineArchiveBox,
  HiOutlineArrowPath,
  HiOutlineInformationCircle,
} from "../icons";
import type {
  EvidenceMatch,
  FieldConflict,
  MatchingResult,
} from "./evidenceMatching";
import { buildAlternativeRecords, applyResolutions } from "./evidenceMatching";
import type { DbCollectedItem, DbEvidenceDataAlternative } from "../../types/projectDb";

// =============================================================================
// Types
// =============================================================================

export interface ConflictResolverProps {
  /** Results from the matching engine */
  matchingResult: MatchingResult;
  /** Current examiner name for audit trail */
  examinerName?: string;
  /** Called when user applies resolutions */
  onApply: (
    updatedItems: DbCollectedItem[],
    alternatives: DbEvidenceDataAlternative[],
  ) => void;
  /** Called when user cancels */
  onCancel: () => void;
}

// =============================================================================
// Main Component
// =============================================================================

export const EvidenceConflictResolver: Component<ConflictResolverProps> = (props) => {
  // Deep-clone the match results so user choices are tracked locally
  const [matches, setMatches] = createSignal<EvidenceMatch[]>(
    JSON.parse(JSON.stringify(props.matchingResult.matched)),
  );

  // Track which container-only enrichments the user wants to accept
  const [acceptedEnrichments, setAcceptedEnrichments] = createSignal<Set<string>>(
    // Default: accept all enrichments
    new Set(
      props.matchingResult.matched.flatMap((m) =>
        m.containerOnlyFields.map((e) => `${m.collectedItem.id}:${e.fieldName}`),
      ),
    ),
  );

  const totalConflicts = createMemo(() =>
    matches().reduce((sum, m) => sum + m.conflicts.length, 0),
  );

  const totalEnrichments = createMemo(() =>
    matches().reduce((sum, m) => sum + m.containerOnlyFields.length, 0),
  );

  // Toggle a single conflict's chosen source
  const toggleConflictSource = (matchIdx: number, conflictIdx: number) => {
    setMatches((prev) => {
      const updated = [...prev];
      const match = { ...updated[matchIdx] };
      const conflicts = [...match.conflicts];
      const c = { ...conflicts[conflictIdx] };
      c.chosenSource = c.chosenSource === "user" ? "container" : "user";
      conflicts[conflictIdx] = c;
      match.conflicts = conflicts;
      updated[matchIdx] = match;
      return updated;
    });
  };

  // Toggle an enrichment acceptance
  const toggleEnrichment = (itemId: string, fieldName: string) => {
    const key = `${itemId}:${fieldName}`;
    setAcceptedEnrichments((prev) => {
      const next = new Set(prev);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
      }
      return next;
    });
  };

  // Bulk: choose all user or all container for a match
  const chooseAllForMatch = (matchIdx: number, source: "user" | "container") => {
    setMatches((prev) => {
      const updated = [...prev];
      const match = { ...updated[matchIdx] };
      match.conflicts = match.conflicts.map((c) => ({ ...c, chosenSource: source }));
      updated[matchIdx] = match;
      return updated;
    });
  };

  // Apply all resolutions
  const handleApply = () => {
    const updatedItems: DbCollectedItem[] = [];
    const allAlternatives: DbEvidenceDataAlternative[] = [];
    const accepted = acceptedEnrichments();

    for (const match of matches()) {
      // Filter containerOnlyFields to only accepted enrichments
      const filteredMatch = {
        ...match,
        containerOnlyFields: match.containerOnlyFields.filter((e) =>
          accepted.has(`${match.collectedItem.id}:${e.fieldName}`),
        ),
      };

      const updated = applyResolutions(match.collectedItem, filteredMatch);
      updatedItems.push(updated);

      if (match.conflicts.length > 0) {
        const alternatives = buildAlternativeRecords(
          filteredMatch,
          props.examinerName || "unknown",
        );
        allAlternatives.push(...alternatives);
      }
    }

    props.onApply(updatedItems, allAlternatives);
  };

  // Confidence label helper
  const confidenceLabel = (confidence: string) => {
    switch (confidence) {
      case "high":
        return <span class="text-2xs px-1.5 py-0.5 rounded bg-green-900/30 text-success">High</span>;
      case "medium":
        return <span class="text-2xs px-1.5 py-0.5 rounded bg-yellow-900/30 text-warning">Medium</span>;
      default:
        return <span class="text-2xs px-1.5 py-0.5 rounded bg-red-900/30 text-error">Low</span>;
    }
  };

  const matchTypeLabel = (type: string) => {
    switch (type) {
      case "evidence_file_id":
        return "Direct Link";
      case "item_number":
        return "Item Number";
      case "serial_number":
        return "Serial Number";
      case "filename":
        return "Filename";
      default:
        return type;
    }
  };

  return (
    <div class="modal-overlay" onClick={(e) => e.target === e.currentTarget && props.onCancel()}>
      <div class="modal-content" style={{ width: "720px", "max-height": "85vh" }}>
        {/* Header */}
        <div class="modal-header">
          <div class="flex items-center gap-2">
            <HiOutlineArrowPath class="w-5 h-5 text-accent" />
            <div>
              <h2 class="text-sm font-semibold">Evidence Data Reconciliation</h2>
              <p class="text-compact text-txt-muted">
                {matches().length} matched · {totalConflicts()} conflict{totalConflicts() !== 1 ? "s" : ""}
                {totalEnrichments() > 0 && ` · ${totalEnrichments()} enrichment${totalEnrichments() !== 1 ? "s" : ""}`}
              </p>
            </div>
          </div>
          <button class="icon-btn-sm" onClick={props.onCancel}>✕</button>
        </div>

        {/* Body */}
        <div class="modal-body overflow-y-auto" style={{ "max-height": "calc(85vh - 140px)" }}>
          {/* Summary banner */}
          <Show when={props.matchingResult.unmatchedItems.length > 0 || props.matchingResult.unmatchedFiles.length > 0}>
            <div class="info-card mb-4">
              <div class="info-card-title">
                <HiOutlineInformationCircle class="w-4 h-4" /> Match Summary
              </div>
              <div class="text-compact text-txt-muted space-y-1">
                <Show when={props.matchingResult.unmatchedItems.length > 0}>
                  <p>{props.matchingResult.unmatchedItems.length} collected item{props.matchingResult.unmatchedItems.length !== 1 ? "s" : ""} had no matching container</p>
                </Show>
                <Show when={props.matchingResult.unmatchedFiles.length > 0}>
                  <p>{props.matchingResult.unmatchedFiles.length} evidence file{props.matchingResult.unmatchedFiles.length !== 1 ? "s" : ""} had no matching collected item</p>
                </Show>
              </div>
            </div>
          </Show>

          {/* No conflicts case */}
          <Show when={totalConflicts() === 0 && totalEnrichments() === 0}>
            <div class="flex flex-col items-center justify-center py-8 text-txt-muted gap-2">
              <HiOutlineCheckCircle class="w-8 h-8 text-success" />
              <p class="text-sm font-medium text-txt">All data matches</p>
              <p class="text-xs">No conflicts found between user-entered and container data.</p>
            </div>
          </Show>

          {/* Match cards */}
          <div class="space-y-4">
            <For each={matches()}>
              {(match, matchIdx) => (
                <Show when={match.conflicts.length > 0 || match.containerOnlyFields.length > 0}>
                  <div class="card">
                    {/* Match header */}
                    <div class="flex items-center justify-between mb-3">
                      <div class="flex items-center gap-2">
                        <HiOutlineArchiveBox class="w-4 h-4 text-txt-muted" />
                        <span class="text-xs font-semibold text-txt truncate" style={{ "max-width": "300px" }}>
                          {match.collectedItem.description || match.collectedItem.itemNumber}
                        </span>
                        <span class="text-2xs text-txt-muted">↔</span>
                        <span class="text-xs text-txt-muted truncate" style={{ "max-width": "200px" }}>
                          {match.evidenceFile.filename}
                        </span>
                      </div>
                      <div class="flex items-center gap-2">
                        {confidenceLabel(match.confidence)}
                        <span class="text-2xs text-txt-muted">{matchTypeLabel(match.matchType)}</span>
                      </div>
                    </div>

                    {/* Field conflicts */}
                    <Show when={match.conflicts.length > 0}>
                      <div class="mb-3">
                        <div class="flex items-center justify-between mb-1.5">
                          <div class="flex items-center gap-1.5">
                            <HiOutlineExclamationTriangle class="w-3.5 h-3.5 text-warning" />
                            <span class="text-compact font-medium text-txt">
                              {match.conflicts.length} Conflict{match.conflicts.length !== 1 ? "s" : ""}
                            </span>
                          </div>
                          <div class="flex items-center gap-1">
                            <button
                              class="text-2xs px-1.5 py-0.5 rounded hover:bg-bg-hover text-txt-muted"
                              onClick={() => chooseAllForMatch(matchIdx(), "user")}
                            >
                              All User
                            </button>
                            <button
                              class="text-2xs px-1.5 py-0.5 rounded hover:bg-bg-hover text-txt-muted"
                              onClick={() => chooseAllForMatch(matchIdx(), "container")}
                            >
                              All Container
                            </button>
                          </div>
                        </div>

                        <div class="space-y-1">
                          <For each={match.conflicts}>
                            {(conflict, conflictIdx) => (
                              <ConflictRow
                                conflict={conflict}
                                onToggle={() => toggleConflictSource(matchIdx(), conflictIdx())}
                              />
                            )}
                          </For>
                        </div>
                      </div>
                    </Show>

                    {/* Container-only enrichments */}
                    <Show when={match.containerOnlyFields.length > 0}>
                      <div>
                        <div class="flex items-center gap-1.5 mb-1.5">
                          <HiOutlineCheckCircle class="w-3.5 h-3.5 text-accent" />
                          <span class="text-compact font-medium text-txt">
                            {match.containerOnlyFields.length} Enrichment{match.containerOnlyFields.length !== 1 ? "s" : ""}
                          </span>
                          <span class="text-2xs text-txt-muted">— container has data not in form</span>
                        </div>

                        <div class="space-y-0.5">
                          <For each={match.containerOnlyFields}>
                            {(field) => {
                              const key = `${match.collectedItem.id}:${field.fieldName}`;
                              const isAccepted = () => acceptedEnrichments().has(key);
                              return (
                                <div
                                  class="flex items-center gap-2 py-1 px-2 rounded text-compact cursor-pointer hover:bg-bg-hover"
                                  classList={{ "bg-bg-hover": isAccepted() }}
                                  onClick={() => toggleEnrichment(match.collectedItem.id, field.fieldName)}
                                >
                                  <input
                                    type="checkbox"
                                    checked={isAccepted()}
                                    class="w-3 h-3 accent-accent"
                                    onClick={(e) => e.stopPropagation()}
                                    onChange={() => toggleEnrichment(match.collectedItem.id, field.fieldName)}
                                  />
                                  <span class="text-txt-muted w-32 shrink-0">{field.fieldLabel}</span>
                                  <span class="text-txt truncate">{field.value}</span>
                                </div>
                              );
                            }}
                          </For>
                        </div>
                      </div>
                    </Show>

                    {/* Matching fields count */}
                    <Show when={match.matchingFields.length > 0}>
                      <div class="flex items-center gap-1.5 mt-2 pt-2 border-t border-border">
                        <HiOutlineCheckCircle class="w-3 h-3 text-success" />
                        <span class="text-2xs text-txt-muted">
                          {match.matchingFields.length} field{match.matchingFields.length !== 1 ? "s" : ""} already match
                        </span>
                      </div>
                    </Show>
                  </div>
                </Show>
              )}
            </For>
          </div>
        </div>

        {/* Footer */}
        <div class="modal-footer justify-end">
          <button class="btn btn-secondary" onClick={props.onCancel}>
            Cancel
          </button>
          <button
            class="btn btn-primary"
            onClick={handleApply}
            disabled={totalConflicts() === 0 && totalEnrichments() === 0}
          >
            Apply {totalConflicts() > 0 ? `${totalConflicts()} Resolution${totalConflicts() !== 1 ? "s" : ""}` : "Enrichments"}
          </button>
        </div>
      </div>
    </div>
  );
};

// =============================================================================
// ConflictRow Sub-component
// =============================================================================

interface ConflictRowProps {
  conflict: FieldConflict;
  onToggle: () => void;
}

const ConflictRow: Component<ConflictRowProps> = (props) => {
  const isUser = () => props.conflict.chosenSource === "user";

  return (
    <div class="flex items-start gap-2 py-1.5 px-2 rounded bg-bg text-compact">
      <span class="text-txt-muted w-32 shrink-0 pt-0.5">{props.conflict.fieldLabel}</span>

      <div class="flex flex-col gap-1 flex-1 min-w-0">
        {/* User value */}
        <button
          class="flex items-center gap-1.5 px-2 py-1 rounded text-left w-full transition-colors"
          classList={{
            "bg-accent/15 border border-accent/30": isUser(),
            "bg-bg-hover border border-transparent hover:border-border": !isUser(),
          }}
          onClick={props.onToggle}
        >
          <span class="text-2xs text-txt-muted w-14 shrink-0">User</span>
          <span
            class="truncate"
            classList={{ "text-txt font-medium": isUser(), "text-txt-muted": !isUser() }}
            title={props.conflict.userValue}
          >
            {props.conflict.userValue || <span class="italic text-txt-muted">(empty)</span>}
          </span>
          <Show when={isUser()}>
            <HiOutlineCheckCircle class="w-3.5 h-3.5 text-accent shrink-0 ml-auto" />
          </Show>
        </button>

        {/* Container value */}
        <button
          class="flex items-center gap-1.5 px-2 py-1 rounded text-left w-full transition-colors"
          classList={{
            "bg-accent/15 border border-accent/30": !isUser(),
            "bg-bg-hover border border-transparent hover:border-border": isUser(),
          }}
          onClick={props.onToggle}
        >
          <span class="text-2xs text-txt-muted w-14 shrink-0">Container</span>
          <span
            class="truncate"
            classList={{ "text-txt font-medium": !isUser(), "text-txt-muted": isUser() }}
            title={props.conflict.containerValue}
          >
            {props.conflict.containerValue || <span class="italic text-txt-muted">(empty)</span>}
          </span>
          <Show when={!isUser()}>
            <HiOutlineCheckCircle class="w-3.5 h-3.5 text-accent shrink-0 ml-auto" />
          </Show>
        </button>
      </div>
    </div>
  );
};
