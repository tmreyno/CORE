// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, For, Show, Accessor } from 'solid-js';
import { HiOutlineKey, HiOutlineDocument, HiOutlinePencil, HiOutlineScale } from '../icons';
import type { AxiomCaseInfo } from '../../types/processed';

interface KeywordsViewProps {
  caseInfo: Accessor<AxiomCaseInfo | null>;
}

export const KeywordsView: Component<KeywordsViewProps> = (props) => {
  const allKeywords = () => props.caseInfo()?.keyword_info?.keywords || [];
  const manualKeywords = () => allKeywords().filter(kw => !kw.from_file);
  const fileKeywords = () => allKeywords().filter(kw => kw.from_file);
  const regexKeywords = () => allKeywords().filter(kw => kw.is_regex);
  
  return (
    <div class="p-6 max-w-[900px]">
      <h2 class="text-xl font-semibold mb-6 text-txt flex items-center gap-2.5">
        <HiOutlineKey class="w-5 h-5" /> Keyword Search Configuration
      </h2>
      
      {/* Keyword Summary */}
      <section class="section">
        <h3 class="section-title">Summary</h3>
        <div class="grid grid-cols-[repeat(auto-fit,minmax(140px,1fr))] gap-3.5">
          <div class="stat-card">
            <span class="stat-value">{allKeywords().length.toLocaleString()}</span>
            <span class="stat-label">Total Keywords</span>
          </div>
          <div class="stat-card">
            <span class="stat-value">{manualKeywords().length.toLocaleString()}</span>
            <span class="stat-label">Manual Entry</span>
          </div>
          <div class="stat-card">
            <span class="stat-value">{fileKeywords().length.toLocaleString()}</span>
            <span class="stat-label">From Files</span>
          </div>
          <div class="stat-card">
            <span class="stat-value">{regexKeywords().length.toLocaleString()}</span>
            <span class="stat-label">Regex Patterns</span>
          </div>
          <div class="stat-card">
            <span class="stat-value">
              {props.caseInfo()?.keyword_info?.keyword_files?.length || 0}
            </span>
            <span class="stat-label">Keyword Files</span>
          </div>
          <div class="stat-card">
            <span class="stat-value">
              {props.caseInfo()?.keyword_info?.privileged_content_keywords?.length || 0}
            </span>
            <span class="stat-label">Privileged Terms</span>
          </div>
        </div>
      </section>
      
      <Show when={props.caseInfo()?.keyword_info?.privileged_content_mode}>
        <div class="field mb-4">
          <span class="field-label">Privileged Content Mode:</span>
          <span class="field-value">
            {props.caseInfo()?.keyword_info?.privileged_content_mode}
          </span>
        </div>
      </Show>
      
      {/* All Keywords Table */}
      <Show when={allKeywords().length > 0}>
        <section class="section">
          <h3 class="section-title">All Keywords ({allKeywords().length.toLocaleString()})</h3>
          <div class="bg-bg-panel rounded-lg border border-border overflow-hidden max-h-[500px] overflow-y-auto">
            <table class="w-full border-collapse text-base">
              <thead class="sticky top-0 z-[1]">
                <tr>
                  <th class="th-center w-[50px]">#</th>
                  <th class="th min-w-[200px]">Keyword</th>
                  <th class="th w-[100px]">Type</th>
                  <th class="th-center w-[80px]">Case</th>
                  <th class="th w-[140px]">Source</th>
                  <th class="th w-[150px]">Encodings</th>
                </tr>
              </thead>
              <tbody>
                <For each={allKeywords()}>
                  {(kw, idx) => (
                    <tr class="hover:bg-bg-hover border-b border-border/30 last:border-b-0">
                      <td class="px-3 py-2 text-center text-txt-faint font-mono text-xs">
                        {idx() + 1}
                      </td>
                      <td class="px-3 py-2">
                        <code class={`inline-block bg-bg px-2 py-0.5 rounded font-mono text-sm break-all max-w-[400px] ${
                          kw.is_regex 
                            ? 'bg-warning-soft text-warning border border-warning/30' 
                            : ''
                        }`}>
                          {kw.value}
                        </code>
                      </td>
                      <td class="px-3 py-2 text-sm">{kw.is_regex ? 'Regex' : 'Plain'}</td>
                      <td class="px-3 py-2 text-center">{kw.is_case_sensitive ? 'Yes' : 'No'}</td>
                      <td class="px-3 py-2 text-sm overflow-hidden text-ellipsis whitespace-nowrap">
                        {kw.from_file ? (
                          <span class="flex items-center gap-1" title={kw.file_name || 'Unknown file'}>
                            <HiOutlineDocument class="w-4 h-4 inline" /> {kw.file_name || 'File'}
                          </span>
                        ) : (
                          <span class="flex items-center gap-1">
                            <HiOutlinePencil class="w-4 h-4 inline" /> Manual
                          </span>
                        )}
                      </td>
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
      
      {/* Privileged Content Keywords */}
      <Show when={
        props.caseInfo()?.keyword_info?.privileged_content_keywords && 
        props.caseInfo()!.keyword_info!.privileged_content_keywords.length > 0
      }>
        <section class="section">
          <h3 class="section-title-with-icon">
            <HiOutlineScale class="w-4 h-4" /> Privileged Content Keywords (
            {props.caseInfo()!.keyword_info!.privileged_content_keywords.length})
          </h3>
          <div class="bg-bg-panel rounded-lg border border-border overflow-hidden max-h-[500px] overflow-y-auto">
            <table class="w-full border-collapse text-base">
              <thead class="sticky top-0 z-[1]">
                <tr>
                  <th class="th-center w-[50px]">#</th>
                  <th class="th min-w-[200px]">Keyword</th>
                  <th class="th w-[100px]">Type</th>
                  <th class="th w-[120px]">Tag/Category</th>
                </tr>
              </thead>
              <tbody>
                <For each={props.caseInfo()?.keyword_info?.privileged_content_keywords || []}>
                  {(kw, idx) => (
                    <tr class="hover:bg-bg-hover border-b border-border/30 last:border-b-0">
                      <td class="px-3 py-2 text-center text-txt-faint font-mono text-xs">
                        {idx() + 1}
                      </td>
                      <td class="px-3 py-2">
                        <code class={`inline-block bg-bg px-2 py-0.5 rounded font-mono text-sm break-all max-w-[400px] ${
                          kw.is_regex 
                            ? 'bg-warning-soft text-warning border border-warning/30' 
                            : ''
                        }`}>
                          {kw.value}
                        </code>
                      </td>
                      <td class="px-3 py-2 text-sm">{kw.is_regex ? 'Regex' : 'Plain'}</td>
                      <td class="px-3 py-2 text-sm text-txt-muted">{kw.file_name || '—'}</td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </section>
      </Show>
    </div>
  );
};
