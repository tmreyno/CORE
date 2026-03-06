// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceStepComponent — third wizard step for evidence selection.
 *
 * Features:
 * - Select/deselect evidence items for the report
 * - Expandable metadata panels showing container-specific details
 * - Editable evidence type per item
 * - Per-item notes for report annotations
 * - Chain of custody sub-section
 */

import { createSignal, For, Show } from "solid-js";
import { HiOutlineCircleStack } from "../../../../icons";
import type { EvidenceType } from "../../../types";
import { useWizard } from "../../WizardContext";
import { EvidenceCard } from "./EvidenceCard";
import { ChainOfCustodySection } from "./ChainOfCustodySection";

export function EvidenceStep() {
  const ctx = useWizard();

  const [expandedCards, setExpandedCards] = createSignal<Set<string>>(new Set());
  const [evidenceNotes, setEvidenceNotes] = createSignal<Map<string, string>>(new Map());
  const [evidenceTypes, setEvidenceTypes] = createSignal<Map<string, EvidenceType>>(new Map());

  const toggleExpanded = (path: string, e?: MouseEvent) => {
    e?.stopPropagation();
    setExpandedCards((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  };

  const setNoteForEvidence = (path: string, note: string) => {
    setEvidenceNotes((prev) => {
      const next = new Map(prev);
      if (note) next.set(path, note);
      else next.delete(path);
      return next;
    });
  };

  const setTypeForEvidence = (path: string, type: EvidenceType) => {
    setEvidenceTypes((prev) => {
      const next = new Map(prev);
      next.set(path, type);
      return next;
    });
  };

  return (
    <div class="space-y-5">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <HiOutlineCircleStack class="w-5 h-5 text-accent" />
          <h3 class="text-base font-semibold">Evidence Items</h3>
        </div>
        <div class="flex items-center gap-3">
          <span class="text-sm text-txt/60">
            <span class="font-medium text-accent">{ctx.selectedEvidence().size}</span> of {ctx.groupedEvidence().length} selected
          </span>
          <Show when={ctx.groupedEvidence().length > 0}>
            <button
              class="text-xs text-accent hover:underline"
              onClick={() => {
                if (ctx.selectedEvidence().size === ctx.groupedEvidence().length) {
                  ctx.setSelectedEvidence(new Set<string>());
                } else {
                  ctx.setSelectedEvidence(new Set<string>(ctx.groupedEvidence().map(g => g.primaryFile.path)));
                }
              }}
            >
              {ctx.selectedEvidence().size === ctx.groupedEvidence().length ? 'Deselect All' : 'Select All'}
            </button>
          </Show>
        </div>
      </div>

      <Show when={ctx.groupedEvidence().length === 0}>
        <div class="text-center py-12 bg-surface/30 rounded-xl border border-border/30">
          <div class="w-16 h-16 mx-auto mb-4 rounded-2xl bg-accent/10 flex items-center justify-center">
            <span class="text-3xl">📂</span>
          </div>
          <p class="font-medium text-txt/80">No evidence files discovered</p>
          <p class="text-sm text-txt/50 mt-1">Scan a directory first to discover forensic images</p>
        </div>
      </Show>

      <div class="space-y-2 max-h-[400px] overflow-y-auto pr-1">
        <For each={ctx.groupedEvidence()}>
          {(group) => (
            <EvidenceCard
              group={group}
              isExpanded={expandedCards().has(group.primaryFile.path)}
              onToggleExpand={(e) => toggleExpanded(group.primaryFile.path, e)}
              notes={evidenceNotes().get(group.primaryFile.path) || ""}
              onNotesChange={(n) => setNoteForEvidence(group.primaryFile.path, n)}
              evidenceType={evidenceTypes().get(group.primaryFile.path)}
              onTypeChange={(t) => setTypeForEvidence(group.primaryFile.path, t)}
            />
          )}
        </For>
      </div>

      {/* Chain of Custody Section */}
      <Show when={ctx.enabledSections().chainOfCustody}>
        <ChainOfCustodySection />
      </Show>
    </div>
  );
}
