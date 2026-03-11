// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ProcessedDatabaseItem - Individual expandable database card
 */

import { Component, Show, For, createSignal } from 'solid-js';
import type { ProcessedDatabase, AxiomCaseInfo, AxiomKeywordFile } from '../../types/processed';
import type { DetailViewType } from '../../hooks/useProcessedDatabases';
import {
  HiOutlineArrowPath,
  HiOutlineXMark,
  HiOutlineClipboardDocument,
  HiOutlineFolder,
  HiOutlineKey,
  HiOutlineDocument,
  HiOutlineMagnifyingGlass,
} from '../icons';
import { getDbTypeName, getDbTypeIcon, ellipsePath } from '../../utils/processed';
import { formatBytes } from '../../utils';
import { getDatabaseDisplayName, getTotalKeywords, getKeywordFiles } from './processedDatabaseHelpers';

interface ProcessedDatabaseItemProps {
  db: ProcessedDatabase;
  isSelected: boolean;
  isLoading: boolean;
  caseInfo?: AxiomCaseInfo;
  currentDetailView?: { type: string; file?: AxiomKeywordFile } | null;
  onSelect: () => void;
  onRemove: () => void;
  onSetDetailView: (view: DetailViewType) => void;
}

export const ProcessedDatabaseItem: Component<ProcessedDatabaseItemProps> = (props) => {
  const [expanded, setExpanded] = createSignal(true);
  const [keywordsExpanded, setKeywordsExpanded] = createSignal(false);
  
  const displayName = () => getDatabaseDisplayName(props.db, props.caseInfo);
  const keywordFiles = () => getKeywordFiles(props.caseInfo);
  const totalKeywords = () => getTotalKeywords(props.caseInfo);
  
  return (
    <div class={`border-b border-border last:border-b-0 ${props.isSelected ? 'bg-accent-soft' : ''}`}>
      {/* Main database header - expandable */}
      <div 
        class={`flex items-center gap-1 py-0.5 px-1 cursor-pointer transition-colors duration-150 hover:bg-bg-hover`}
        onClick={() => {
          props.onSelect();
          setExpanded(!expanded());
        }}
      >
        <span class={`text-compact leading-tight text-txt-faint transition-transform duration-150 shrink-0 w-2.5 text-center ${expanded() ? 'rotate-90' : ''}`}>
          ▶
        </span>
        <span class={`text-compact leading-none shrink-0`}>{getDbTypeIcon(props.db.db_type)}</span>
        <div class={`flex-1 min-w-0 flex flex-col gap-0.5`}>
          <div class={`text-compact leading-tight font-semibold text-txt whitespace-nowrap overflow-hidden text-ellipsis`}>
            {displayName()}
          </div>
          <div class={`flex flex-wrap gap-1 text-compact leading-tight text-txt-muted`}>
            <span class="text-accent font-medium">{getDbTypeName(props.db.db_type)}</span>
            <Show when={props.caseInfo?.total_artifacts}>
              <span class="text-success">{props.caseInfo!.total_artifacts.toLocaleString()} artifacts</span>
            </Show>
            <Show when={props.db.total_size}>
              <span class="text-txt-faint">{formatBytes(props.db.total_size!)}</span>
            </Show>
          </div>
          <Show when={props.caseInfo?.examiner || props.db.examiner}>
            <div class={`text-compact leading-tight text-txt-faint`}>
              👤 {props.caseInfo?.examiner || props.db.examiner}
            </div>
          </Show>
        </div>
        <div class={`flex items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100 ${props.isLoading ? 'opacity-100' : 'hover:opacity-100'}`}>
          <Show when={props.isLoading}>
            <HiOutlineArrowPath class={`w-3 h-3 animate-spin text-accent`} />
          </Show>
          <button 
            class={`bg-transparent border-none px-1 py-0.5 cursor-pointer text-compact leading-tight text-txt-faint opacity-70 hover:opacity-100 hover:text-error transition-all flex items-center`}
            onClick={(e) => {
              e.stopPropagation();
              props.onRemove();
            }}
            title="Remove from list"
          >
            <HiOutlineXMark class="w-3 h-3" />
          </button>
        </div>
      </div>

      {/* Expanded content - sub-items */}
      <Show when={expanded() && props.caseInfo}>
        <div class="pl-3 pb-0.5 border-l border-border ml-[14px]">
          {/* Case Report */}
          <div 
            class={`flex items-center gap-1 px-1 py-0.5 cursor-pointer rounded text-compact leading-tight text-txt transition-all duration-150 hover:bg-bg-hover ${props.currentDetailView?.type === 'case' ? 'bg-accent-soft text-accent' : ''}`}
            onClick={(e) => {
              e.stopPropagation();
              props.onSetDetailView({ type: 'case' });
            }}
          >
            <HiOutlineClipboardDocument class={`w-3 h-3 shrink-0`} />
            <span class="flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">Case Report</span>
          </div>

          {/* Evidence Sources */}
          <div 
            class={`flex items-center gap-1 px-1 py-0.5 cursor-pointer rounded text-compact leading-tight text-txt transition-all duration-150 hover:bg-bg-hover ${props.currentDetailView?.type === 'evidence' ? 'bg-accent-soft text-accent' : ''}`}
            onClick={(e) => {
              e.stopPropagation();
              props.onSetDetailView({ type: 'evidence' });
            }}
          >
            <HiOutlineFolder class={`w-3 h-3 shrink-0`} />
            <span class="flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">Evidence</span>
            <Show when={props.caseInfo?.evidence_sources?.length}>
              <span class={`text-compact leading-tight text-txt-faint`}>
                ({props.caseInfo!.evidence_sources.length})
              </span>
            </Show>
          </div>

          {/* Keywords Section - collapsible */}
          <div>
            <div 
              class={`flex items-center gap-1 pl-0.5 pr-1 py-0.5 cursor-pointer rounded text-compact leading-tight text-txt transition-all duration-150 hover:bg-bg-hover ${props.currentDetailView?.type === 'keywords' ? 'bg-accent-soft text-accent' : ''}`}
              onClick={(e) => {
                e.stopPropagation();
                props.onSetDetailView({ type: 'keywords' });
                setKeywordsExpanded(!keywordsExpanded());
              }}
            >
              <span class={`text-compact leading-tight text-txt-faint transition-transform duration-150 w-2 text-center ${keywordsExpanded() ? 'rotate-90' : ''}`}>
                ▶
              </span>
              <HiOutlineKey class={`w-3 h-3 shrink-0`} />
              <span class="flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">Keywords</span>
              <span class={`text-compact leading-tight text-txt-faint`}>({totalKeywords()})</span>
            </div>

            {/* Keyword Files - nested under Keywords */}
            <Show when={keywordsExpanded() && keywordFiles().length > 0}>
              <div class="pl-3 mt-0.5">
                <For each={keywordFiles()}>
                  {(file: AxiomKeywordFile) => {
                    const isActive = () => {
                      const view = props.currentDetailView;
                      return view?.type === 'keyword-file' && view.file?.file_name === file.file_name;
                    };
                    return (
                      <div 
                        class={`flex items-center gap-1 px-1 py-0.5 cursor-pointer rounded-sm text-compact leading-tight text-txt-muted transition-all duration-150 hover:bg-bg-hover hover:text-txt ${isActive() ? 'bg-accent-soft text-accent' : ''}`}
                        onClick={(e) => {
                          e.stopPropagation();
                          props.onSetDetailView({ type: 'keyword-file', file });
                        }}
                        title={file.file_path}
                      >
                        <HiOutlineDocument class={`w-3 h-3 shrink-0`} />
                        <span class="flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">
                          {ellipsePath(file.file_name, 20)}
                        </span>
                        <span class={`text-compact leading-tight text-txt-faint font-mono`}>
                          {file.record_count}
                        </span>
                      </div>
                    );
                  }}
                </For>
              </div>
            </Show>
          </div>

          {/* Artifacts */}
          <div 
            class={`flex items-center gap-1 px-1 py-0.5 cursor-pointer rounded text-compact leading-tight text-txt transition-all duration-150 hover:bg-bg-hover ${props.currentDetailView?.type === 'artifacts' ? 'bg-accent-soft text-accent' : ''}`}
            onClick={(e) => {
              e.stopPropagation();
              props.onSetDetailView({ type: 'artifacts' });
            }}
          >
            <HiOutlineMagnifyingGlass class={`w-3 h-3 shrink-0`} />
            <span class="flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">Artifacts</span>
            <Show when={props.caseInfo?.total_artifacts}>
              <span class={`text-compact leading-tight text-txt-faint`}>
                ({props.caseInfo!.total_artifacts.toLocaleString()})
              </span>
            </Show>
          </div>
        </div>
      </Show>
    </div>
  );
};
