// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, For, Show } from 'solid-js';
import {
  HiOutlineCircleStack,
  HiOutlineClipboardDocument,
  HiOutlineDocument,
  HiOutlineArrowPath,
} from './icons';
import type { 
  ProcessedDatabase, AxiomCaseInfo, ArtifactCategorySummary,
  AxiomKeywordFile
} from '../types/processed';
import type { DetailViewType } from '../hooks/useProcessedDatabases';
import { formatDate } from '../utils/processed';
import { CaseReportView } from './processed-detail/CaseReportView';
import { EvidenceView } from './processed-detail/EvidenceView';
import { KeywordsView } from './processed-detail/KeywordsView';
import { ArtifactsView } from './processed-detail/ArtifactsView';

/** Detail view types for processed databases - re-export from hook */
export type ProcessedDetailView = DetailViewType;

interface ProcessedDetailPanelProps {
  database: ProcessedDatabase | null;
  caseInfo: AxiomCaseInfo | null;
  categories: ArtifactCategorySummary[];
  loading?: boolean;
  /** External detail view from manager (overrides internal state) */
  detailView?: DetailViewType;
  /** Callback when view changes internally */
  onDetailViewChange?: (view: DetailViewType) => void;
}

export const ProcessedDetailPanel: Component<ProcessedDetailPanelProps> = (props) => {
  // Use external detailView - navigation is handled in the left panel
  const detailView = () => props.detailView ?? { type: 'case' };

  // Reset view when database changes
  const db = () => props.database;
  const caseInfo = () => props.caseInfo;
  const categories = () => props.categories || [];

  return (
    <div class="flex flex-col h-full bg-bg text-txt">
      <Show when={!db()}>
        <div class="flex flex-col items-center justify-center h-full gap-3 opacity-60 p-10 text-center">
          <HiOutlineCircleStack class="w-12 h-12 text-txt-muted" />
          <h3 class="m-0 text-lg font-medium text-txt-muted">No Database Selected</h3>
          <p class="m-0 text-base text-txt-faint">Select a processed database from the left panel to view details</p>
        </div>
      </Show>

      <Show when={db()}>
        {/* Loading indicator */}
        <Show when={props.loading}>
          <div class="absolute inset-0 flex items-center justify-center gap-2 bg-bg/80 z-10">
            <HiOutlineArrowPath class="w-5 h-5 animate-spin" />
            <span>Loading...</span>
          </div>
        </Show>

        {/* Detail Content - full width, no sidebar */}
        <div class="flex-1 overflow-auto p-0">
          {/* Case Report View */}
          <Show when={detailView()?.type === 'case'}>
            <CaseReportView db={db} caseInfo={caseInfo} categories={categories} />
          </Show>

          {/* Evidence View */}
          <Show when={detailView()?.type === 'evidence'}>
            <EvidenceView caseInfo={caseInfo} />
          </Show>

          {/* Keywords Overview View */}
          <Show when={detailView()?.type === 'keywords'}>
            <KeywordsView caseInfo={caseInfo} />
          </Show>

          {/* Keyword File Detail View */}
            <Show when={detailView()?.type === 'keyword-file'}>
              {(() => {
                const view = detailView() as { type: 'keyword-file'; file: AxiomKeywordFile };
                const file = view?.file;
                // Find keywords from this file - check both exact match and case-insensitive
                const fileKeywords = () => {
                  const keywords = caseInfo()?.keyword_info?.keywords || [];
                  const fileName = file?.file_name;
                  if (!fileName) return [];
                  
                  return keywords.filter(kw => 
                    kw.from_file && kw.file_name && (
                      kw.file_name === fileName ||
                      kw.file_name.toLowerCase() === fileName.toLowerCase()
                    )
                  );
                };
                
                // Group keywords by type (regex vs plain)
                const regexKeywords = () => fileKeywords().filter(kw => kw.is_regex);
                const plainKeywords = () => fileKeywords().filter(kw => !kw.is_regex);
                
                return (
                  <div class="p-6 max-w-[900px]">
                    <h2 class="text-xl font-semibold mb-6 text-txt flex items-center gap-2.5"><HiOutlineDocument class="w-5 h-5" /> Keyword File Details</h2>
                    
                    <section class="mb-7 pb-6 border-b border-border">
                      <h3 class="text-sm font-semibold mb-4 text-txt-muted uppercase tracking-wide">File Information</h3>
                      <div class="grid grid-cols-2 gap-3.5">
                        <div class="flex flex-col gap-1 col-span-2">
                          <span class="text-xs font-medium text-txt-faint uppercase tracking-wide">File Name</span>
                          <span class="text-base text-txt break-words">{file?.file_name}</span>
                        </div>
                        <div class="flex flex-col gap-1 col-span-2">
                          <span class="text-xs font-medium text-txt-faint uppercase tracking-wide">Full Path</span>
                          <span class="text-sm text-txt font-mono bg-bg-panel px-3 py-2 rounded overflow-x-auto whitespace-nowrap" title={file?.file_path}>{file?.file_path}</span>
                        </div>
                        <div class="flex flex-col gap-1">
                          <span class="text-xs font-medium text-txt-faint uppercase tracking-wide">Keywords Count</span>
                          <span class="text-base text-txt break-words">{file?.record_count.toLocaleString()}</span>
                        </div>
                        <div class="flex flex-col gap-1">
                          <span class="text-xs font-medium text-txt-faint uppercase tracking-wide">Found in Case</span>
                          <span class="text-base text-txt break-words">{fileKeywords().length.toLocaleString()}</span>
                        </div>
                        <div class="flex flex-col gap-1">
                          <span class="text-xs font-medium text-txt-faint uppercase tracking-wide">Status</span>
                          <span class={`text-base break-words ${file?.enabled ? 'text-success' : 'text-txt-faint'}`}>
                            {file?.enabled ? '✓ Enabled' : '✗ Disabled'}
                          </span>
                        </div>
                        <div class="flex flex-col gap-1">
                          <span class="text-xs font-medium text-txt-faint uppercase tracking-wide">Case Sensitive</span>
                          <span class="text-base text-txt break-words">{file?.is_case_sensitive ? 'Yes' : 'No'}</span>
                        </div>
                        <Show when={file?.date_added}>
                          <div class="flex flex-col gap-1">
                            <span class="text-xs font-medium text-txt-faint uppercase tracking-wide">Date Added</span>
                            <span class="text-base text-txt break-words">{formatDate(file?.date_added)}</span>
                          </div>
                        </Show>
                      </div>
                    </section>
                    
                    {/* Complete keyword list from this file */}
                    <Show when={fileKeywords().length > 0}>
                      {/* Summary */}
                      <section class="mb-7 pb-6 border-b border-border">
                        <h3 class="text-sm font-semibold mb-4 text-txt-muted uppercase tracking-wide">Keyword Summary</h3>
                        <div class="flex gap-6 flex-wrap">
                          <div class="flex flex-col items-center px-6 py-4 bg-bg-panel border border-border rounded-lg min-w-[120px]">
                            <span class="text-2xl font-bold text-accent font-mono">{plainKeywords().length.toLocaleString()}</span>
                            <span class="text-sm text-txt-faint uppercase tracking-wide mt-1">Plain Keywords</span>
                          </div>
                          <div class="flex flex-col items-center px-6 py-4 bg-bg-panel border border-border rounded-lg min-w-[120px]">
                            <span class="text-2xl font-bold text-accent font-mono">{regexKeywords().length.toLocaleString()}</span>
                            <span class="text-sm text-txt-faint uppercase tracking-wide mt-1">Regex Patterns</span>
                          </div>
                          <div class="flex flex-col items-center px-6 py-4 bg-bg-panel border border-border rounded-lg min-w-[120px]">
                            <span class="text-2xl font-bold text-accent font-mono">{fileKeywords().filter(kw => kw.is_case_sensitive).length.toLocaleString()}</span>
                            <span class="text-sm text-txt-faint uppercase tracking-wide mt-1">Case Sensitive</span>
                          </div>
                        </div>
                      </section>
                      
                      {/* Plain Keywords List */}
                      <Show when={plainKeywords().length > 0}>
                        <section class="mb-7 pb-6 border-b border-border">
                        <h3 class="text-sm font-semibold mb-4 text-txt-muted uppercase tracking-wide">Plain Keywords ({plainKeywords().length.toLocaleString()})</h3>
                          <div class="bg-bg-panel rounded-lg border border-border overflow-hidden max-h-[500px] overflow-y-auto">
                            <table class="w-full border-collapse text-base">
                              <thead class="sticky top-0 z-[1]">
                                <tr>
                                  <th class="bg-bg-card px-3 py-2.5 text-center text-xs font-semibold uppercase tracking-wide text-txt-faint border-b border-border w-[50px]">#</th>
                                  <th class="bg-bg-card px-3 py-2.5 text-left text-xs font-semibold uppercase tracking-wide text-txt-faint border-b border-border min-w-[200px]">Keyword</th>
                                  <th class="bg-bg-card px-3 py-2.5 text-center text-xs font-semibold uppercase tracking-wide text-txt-faint border-b border-border w-[80px]">Case</th>
                                  <th class="bg-bg-card px-3 py-2.5 text-left text-xs font-semibold uppercase tracking-wide text-txt-faint border-b border-border w-[150px]">Encodings</th>
                                </tr>
                              </thead>
                              <tbody>
                                <For each={plainKeywords()}>
                                  {(kw, idx) => (
                                    <tr class="hover:bg-bg-hover border-b border-border/30 last:border-b-0">
                                      <td class="px-3 py-2 text-center text-txt-faint font-mono text-xs">{idx() + 1}</td>
                                      <td class="px-3 py-2">
                                        <code class="inline-block bg-bg px-2 py-0.5 rounded font-mono text-sm break-all max-w-[400px]">{kw.value}</code>
                                      </td>
                                      <td class="px-3 py-2 text-center">{kw.is_case_sensitive ? 'Yes' : 'No'}</td>
                                      <td class="px-3 py-2 text-xs text-txt-muted">
                                        {kw.encoding_types.length > 0 ? kw.encoding_types.join(', ') : 'Default'}
                                      </td>
                                    </tr>
                                  )}
                                </For>
                              </tbody>
                            </table>
                          </div>
                        </section>
                      </Show>
                      
                      {/* Regex Patterns List */}
                      <Show when={regexKeywords().length > 0}>
                        <section class="mb-7 pb-6 border-b border-border last:border-b-0 last:mb-0">
                        <h3 class="text-sm font-semibold mb-4 text-txt-muted uppercase tracking-wide">Regex Patterns ({regexKeywords().length.toLocaleString()})</h3>
                          <div class="bg-bg-panel rounded-lg border border-border overflow-hidden max-h-[500px] overflow-y-auto">
                            <table class="w-full border-collapse text-base">
                              <thead class="sticky top-0 z-[1]">
                                <tr>
                                  <th class="bg-bg-card px-3 py-2.5 text-center text-xs font-semibold uppercase tracking-wide text-txt-faint border-b border-border w-[50px]">#</th>
                                  <th class="bg-bg-card px-3 py-2.5 text-left text-xs font-semibold uppercase tracking-wide text-txt-faint border-b border-border min-w-[200px]">Pattern</th>
                                  <th class="bg-bg-card px-3 py-2.5 text-center text-xs font-semibold uppercase tracking-wide text-txt-faint border-b border-border w-[80px]">Case</th>
                                  <th class="bg-bg-card px-3 py-2.5 text-left text-xs font-semibold uppercase tracking-wide text-txt-faint border-b border-border w-[150px]">Encodings</th>
                                </tr>
                              </thead>
                              <tbody>
                                <For each={regexKeywords()}>
                                  {(kw, idx) => (
                                    <tr class="hover:bg-bg-hover border-b border-border/30 last:border-b-0">
                                      <td class="px-3 py-2 text-center text-txt-faint font-mono text-xs">{idx() + 1}</td>
                                      <td class="px-3 py-2">
                                        <code class="inline-block bg-warning-soft text-warning border border-warning/30 px-2 py-0.5 rounded font-mono text-sm break-all max-w-[400px]">{kw.value}</code>
                                      </td>
                                      <td class="px-3 py-2 text-center">{kw.is_case_sensitive ? 'Yes' : 'No'}</td>
                                      <td class="px-3 py-2 text-xs text-txt-muted">
                                        {kw.encoding_types.length > 0 ? kw.encoding_types.join(', ') : 'Default'}
                                      </td>
                                    </tr>
                                  )}
                                </For>
                              </tbody>
                            </table>
                          </div>
                        </section>
                      </Show>
                    </Show>
                    
                    {/* No keywords found fallback */}
                    <Show when={fileKeywords().length === 0}>
                      <section class="mb-7 pb-6 border-b border-border last:border-b-0 last:mb-0">
                        <h3 class="text-sm font-semibold mb-4 text-txt-muted uppercase tracking-wide">Keywords from this File</h3>
                        <div class="flex flex-col items-center justify-center py-10 px-5 text-center gap-3">
                          <HiOutlineClipboardDocument class="w-12 h-12 opacity-50 text-txt-muted" />
                          <p class="text-sm text-txt-muted m-0 max-w-[450px]">
                            Individual keywords from this file are not stored in the AXIOM case database.
                          </p>
                          <p class="text-base text-txt-faint m-0 mt-2 max-w-[450px] leading-relaxed">
                            The file contains <strong class="text-accent">{file?.record_count.toLocaleString()}</strong> keywords 
                            that were used during the search, but AXIOM only stores the file reference, 
                            not the individual keyword values.
                          </p>
                          <Show when={file?.file_path}>
                            <p class="text-xs text-txt-faint m-0 mt-3 bg-bg-panel px-4 py-2.5 rounded-md max-w-full break-all">
                              Original file location: <code class="font-mono bg-bg px-1.5 py-0.5 rounded text-xs">{file?.file_path}</code>
                            </p>
                          </Show>
                        </div>
                      </section>
                    </Show>
                  </div>
                );
              })()}
            </Show>

            {/* Artifacts View */}
            <Show when={detailView()?.type === 'artifacts'}>
              <ArtifactsView caseInfo={caseInfo} categories={categories} />
            </Show>
          </div>
        </Show>
    </div>
  );
};
