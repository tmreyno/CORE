// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, For, Show, Accessor } from 'solid-js';
import { HiOutlineFolder } from '../icons';
import type { AxiomCaseInfo } from '../../types/processed';
import { ellipsePath } from '../../utils/processed';
import { formatBytes } from '../../utils';

interface EvidenceViewProps {
  caseInfo: Accessor<AxiomCaseInfo | null>;
}

export const EvidenceView: Component<EvidenceViewProps> = (props) => {
  return (
    <div class="p-6 max-w-[900px]">
      <h2 class="text-xl font-semibold mb-6 text-txt flex items-center gap-2.5">
        <HiOutlineFolder class="w-5 h-5" /> Evidence Sources
      </h2>
      <div class="flex flex-col gap-4">
        <For each={props.caseInfo()?.evidence_sources || []}>
          {(source, idx) => (
            <div class="bg-bg-panel rounded-lg border border-border overflow-hidden">
              <div class="flex items-center gap-2.5 px-4 py-3 bg-bg-card border-b border-border">
                <span class="text-xs font-semibold text-accent bg-accent-soft px-2 py-0.5 rounded-full">
                  #{idx() + 1}
                </span>
                <span 
                  class="flex-1 font-medium text-sm overflow-hidden text-ellipsis whitespace-nowrap" 
                  title={source.name}
                >
                  {ellipsePath(source.name, 50)}
                </span>
                <Show when={source.evidence_number}>
                  <span class="text-sm text-txt-faint">[{source.evidence_number}]</span>
                </Show>
              </div>
              <div class="grid grid-cols-2 gap-2.5 p-4">
                <Show when={source.source_type}>
                  <div class="flex flex-col gap-0.5 text-sm">
                    <span class="text-2xs text-txt-faint uppercase">Type:</span>
                    <span class="text-sm text-txt">{source.source_type}</span>
                  </div>
                </Show>
                <Show when={source.path}>
                  <div class="flex flex-col gap-0.5 text-sm col-span-2">
                    <span class="text-2xs text-txt-faint uppercase">Path:</span>
                    <span class="text-sm text-txt" title={source.path}>
                      {ellipsePath(source.path || '', 60)}
                    </span>
                  </div>
                </Show>
                <Show when={source.hash}>
                  <div class="flex flex-col gap-0.5 text-sm">
                    <span class="text-2xs text-txt-faint uppercase">Hash:</span>
                    <span class="text-sm text-txt font-mono">{source.hash}</span>
                  </div>
                </Show>
                <Show when={source.size}>
                  <div class="flex flex-col gap-0.5 text-sm">
                    <span class="text-2xs text-txt-faint uppercase">Size:</span>
                    <span class="text-sm text-txt">{formatBytes(source.size!)}</span>
                  </div>
                </Show>
                <Show when={source.search_types && source.search_types.length > 0}>
                  <div class="flex flex-col gap-0.5 text-sm col-span-2">
                    <span class="text-2xs text-txt-faint uppercase">Search Types:</span>
                    <span class="text-sm text-txt">{source.search_types.join(', ')}</span>
                  </div>
                </Show>
              </div>
            </div>
          )}
        </For>
      </div>
    </div>
  );
};
