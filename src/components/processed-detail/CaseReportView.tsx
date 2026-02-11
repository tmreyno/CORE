// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, Accessor } from 'solid-js';
import { HiOutlineClipboardDocument } from '../icons';
import type { ProcessedDatabase, AxiomCaseInfo, ArtifactCategorySummary } from '../../types/processed';
import { getDbTypeName, formatDate } from '../../utils/processed';
import { formatBytes } from '../../utils';

interface CaseReportViewProps {
  db: Accessor<ProcessedDatabase | null>;
  caseInfo: Accessor<AxiomCaseInfo | null>;
  categories: Accessor<ArtifactCategorySummary[]>;
}

export const CaseReportView: Component<CaseReportViewProps> = (props) => {
  return (
    <div class="p-6 max-w-[900px]">
      <h2 class="text-xl font-semibold mb-6 text-txt flex items-center gap-2.5">
        <HiOutlineClipboardDocument class="w-5 h-5" /> Case Report
      </h2>
      
      {/* Case Information Section */}
      <section class="section">
        <h3 class="section-title">Case Information</h3>
        <div class="grid grid-cols-2 gap-3.5">
          <div class="field">
            <span class="field-label">Case Name</span>
            <span class="field-value">
              {props.caseInfo()?.case_name || props.db()?.case_name || props.db()?.name || 'N/A'}
            </span>
          </div>
          <Show when={props.caseInfo()?.case_number}>
            <div class="field">
              <span class="field-label">Case Number</span>
              <span class="field-value">{props.caseInfo()?.case_number}</span>
            </div>
          </Show>
          <Show when={props.caseInfo()?.case_type}>
            <div class="field">
              <span class="field-label">Case Type</span>
              <span class="field-value">{props.caseInfo()?.case_type}</span>
            </div>
          </Show>
          <Show when={props.caseInfo()?.description}>
            <div class="field col-span-2">
              <span class="field-label">Description</span>
              <span class="field-value">{props.caseInfo()?.description}</span>
            </div>
          </Show>
        </div>
      </section>
      
      {/* Examiner Information Section */}
      <section class="section">
        <h3 class="section-title">Examiner Information</h3>
        <div class="grid grid-cols-2 gap-3.5">
          <div class="field">
            <span class="field-label">Examiner</span>
            <span class="field-value">
              {props.caseInfo()?.examiner || props.db()?.examiner || 'N/A'}
            </span>
          </div>
          <Show when={props.caseInfo()?.agency}>
            <div class="field">
              <span class="field-label">Agency</span>
              <span class="field-value">{props.caseInfo()?.agency}</span>
            </div>
          </Show>
          <Show when={props.caseInfo()?.user}>
            <div class="field">
              <span class="field-label">User Account</span>
              <span class="field-value">{props.caseInfo()?.user}</span>
            </div>
          </Show>
          <Show when={props.caseInfo()?.host_name}>
            <div class="field">
              <span class="field-label">Workstation</span>
              <span class="field-value">{props.caseInfo()?.host_name}</span>
            </div>
          </Show>
          <Show when={props.caseInfo()?.operating_system}>
            <div class="field">
              <span class="field-label">Operating System</span>
              <span class="field-value">{props.caseInfo()?.operating_system}</span>
            </div>
          </Show>
        </div>
      </section>
      
      {/* Processing Information Section */}
      <section class="section">
        <h3 class="section-title">Processing Details</h3>
        <div class="grid grid-cols-2 gap-3.5">
          <div class="field">
            <span class="field-label">Tool</span>
            <span class="field-value">{getDbTypeName(props.db()?.db_type || 'Unknown')}</span>
          </div>
          <Show when={props.caseInfo()?.axiom_version || props.db()?.version}>
            <div class="field">
              <span class="field-label">Version</span>
              <span class="field-value">
                {props.caseInfo()?.axiom_version || props.db()?.version}
              </span>
            </div>
          </Show>
          <Show when={props.caseInfo()?.search_start}>
            <div class="field">
              <span class="field-label">Processing Started</span>
              <span class="field-value">{formatDate(props.caseInfo()?.search_start)}</span>
            </div>
          </Show>
          <Show when={props.caseInfo()?.search_end}>
            <div class="field">
              <span class="field-label">Processing Ended</span>
              <span class="field-value">{formatDate(props.caseInfo()?.search_end)}</span>
            </div>
          </Show>
          <Show when={props.caseInfo()?.search_duration}>
            <div class="field">
              <span class="field-label">Duration</span>
              <span class="field-value">{props.caseInfo()?.search_duration}</span>
            </div>
          </Show>
          <Show when={props.caseInfo()?.search_outcome}>
            <div class="field">
              <span class="field-label">Outcome</span>
              <span class={`field-value ${
                props.caseInfo()?.search_outcome?.toLowerCase() === 'completed' 
                  ? 'text-success' 
                  : 'text-error'
              }`}>
                {props.caseInfo()?.search_outcome}
              </span>
            </div>
          </Show>
          <Show when={props.caseInfo()?.created || props.db()?.created_date}>
            <div class="field">
              <span class="field-label">Created</span>
              <span class="field-value">
                {formatDate(props.caseInfo()?.created || props.db()?.created_date)}
              </span>
            </div>
          </Show>
          <Show when={props.caseInfo()?.modified}>
            <div class="field">
              <span class="field-label">Last Modified</span>
              <span class="field-value">{formatDate(props.caseInfo()?.modified)}</span>
            </div>
          </Show>
        </div>
      </section>
      
      {/* Statistics Summary */}
      <section class="section">
        <h3 class="section-title">Statistics Summary</h3>
        <div class="grid grid-cols-[repeat(auto-fit,minmax(140px,1fr))] gap-3.5">
          <div class="stat-card">
            <span class="stat-value">
              {props.caseInfo()?.total_artifacts?.toLocaleString() || 0}
            </span>
            <span class="stat-label">Total Artifacts</span>
          </div>
          <div class="stat-card">
            <span class="stat-value">{props.caseInfo()?.evidence_sources?.length || 0}</span>
            <span class="stat-label">Evidence Sources</span>
          </div>
          <div class="stat-card">
            <span class="stat-value">
              {props.caseInfo()?.keyword_info?.keywords_entered || 0}
            </span>
            <span class="stat-label">Keywords</span>
          </div>
          <div class="stat-card">
            <span class="stat-value">
              {props.caseInfo()?.search_results?.length || props.categories().length || 0}
            </span>
            <span class="stat-label">Artifact Types</span>
          </div>
        </div>
      </section>
      
      {/* File Location */}
      <section class="section">
        <h3 class="section-title">File Location</h3>
        <div class="grid grid-cols-2 gap-3.5">
          <div class="field col-span-2">
            <span class="field-label">Database Path</span>
            <span class="field-value-mono" title={props.db()?.path}>{props.db()?.path}</span>
          </div>
          <Show when={props.db()?.total_size}>
            <div class="field">
              <span class="field-label">Database Size</span>
              <span class="field-value">{formatBytes(props.db()!.total_size!)}</span>
            </div>
          </Show>
        </div>
      </section>
    </div>
  );
};
