// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, For, Show, Accessor } from 'solid-js';
import { HiOutlineMagnifyingGlass } from '../icons';
import type { AxiomCaseInfo, ArtifactCategorySummary } from '../../types/processed';
import { getCategoryIcon } from '../../utils/processed';

interface ArtifactsViewProps {
  caseInfo: Accessor<AxiomCaseInfo | null>;
  categories: Accessor<ArtifactCategorySummary[]>;
}

export const ArtifactsView: Component<ArtifactsViewProps> = (props) => {
  return (
    <div class="p-6 max-w-[900px]">
      <h2 class="text-xl font-semibold mb-6 text-txt flex items-center gap-2.5">
        <HiOutlineMagnifyingGlass class="w-5 h-5" /> Artifact Summary
      </h2>
      
      {/* Search Results from XML */}
      <Show when={props.caseInfo()?.search_results && props.caseInfo()!.search_results.length > 0}>
        <section class="mb-7 pb-6 border-b border-border">
          <h3 class="text-sm font-semibold mb-4 text-txt-muted uppercase tracking-wide">
            Search Results ({props.caseInfo()!.search_results.length} types)
          </h3>
          <div class="bg-bg-panel rounded-lg border border-border overflow-hidden">
            <div class="flex px-4 py-2.5 bg-bg-card border-b border-border text-xs font-semibold uppercase tracking-wide text-txt-faint">
              <span class="flex-1">Artifact Type</span>
              <span class="w-[100px] text-right font-mono">Count</span>
            </div>
            <For each={props.caseInfo()?.search_results?.sort((a, b) => b.hit_count - a.hit_count) || []}>
              {(result) => (
                <div class="flex px-4 py-2.5 text-base border-b border-border/30 last:border-b-0 hover:bg-bg-hover">
                  <span class="flex-1">{result.artifact_type}</span>
                  <span class="w-[100px] text-right font-mono">{result.hit_count.toLocaleString()}</span>
                </div>
              )}
            </For>
          </div>
        </section>
      </Show>
      
      {/* Category breakdown */}
      <Show when={props.categories().length > 0}>
        <section class="mb-7 pb-6 border-b border-border last:border-b-0 last:mb-0">
          <h3 class="text-sm font-semibold mb-4 text-txt-muted uppercase tracking-wide">
            Categories ({props.categories().length})
          </h3>
          <div class="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-3">
            <For each={props.categories()}>
              {(cat) => (
                <div class="flex items-center gap-2.5 px-3.5 py-3 bg-bg-panel border border-border rounded-lg hover:bg-bg-hover hover:border-accent transition-colors">
                  <span class="text-xl">{getCategoryIcon(cat.category)}</span>
                  <span class="flex-1 text-base overflow-hidden text-ellipsis whitespace-nowrap">
                    {cat.artifact_type}
                  </span>
                  <span class="text-base font-semibold text-accent">{cat.count.toLocaleString()}</span>
                </div>
              )}
            </For>
          </div>
        </section>
      </Show>
    </div>
  );
};
