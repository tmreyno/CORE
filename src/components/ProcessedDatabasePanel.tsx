// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, createSignal, For, Show } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import {
  HiOutlineCircleStack,
  HiOutlineMagnifyingGlass,
  HiOutlineTrash,
  HiOutlinePlus,
  HiOutlineExclamationTriangle,
  HiOutlineArrowPath,
  HiOutlineFolder,
  HiOutlineDocument,
  HiOutlineClipboardDocument,
  HiOutlineKey,
  HiOutlineXMark,
} from './icons';
import type { 
  ProcessedDatabase, ArtifactInfo,
  ArtifactCategorySummary, AxiomCaseInfo, AxiomKeywordFile
} from '../types/processed';
import type { ProcessedDatabasesManager } from '../hooks/useProcessedDatabases';
import { ellipsePath, getDbTypeName, getDbTypeIcon, formatSize } from '../utils/processed';
import {
  UI_FONT_COMPACT,
  UI_ICON_COMPACT,
  UI_GAP_COMPACT,
  UI_GAP_SMALL,
} from './ui/constants';

interface ProcessedDatabasePanelProps {
  /** Manager hook for shared state (optional for backward compatibility) */
  manager?: ProcessedDatabasesManager;
  onSelectDatabase?: (db: ProcessedDatabase) => void;
  onSelectArtifact?: (db: ProcessedDatabase, artifact: ArtifactInfo) => void;
}

const ProcessedDatabasePanel: Component<ProcessedDatabasePanelProps> = (props) => {
  // Local state for databases
  const [localDatabases, setLocalDatabases] = createSignal<ProcessedDatabase[]>([]);
  const [localSelectedDb, setLocalSelectedDb] = createSignal<string | null>(null);
  const [localAxiomCaseInfo, setLocalAxiomCaseInfo] = createSignal<Record<string, AxiomCaseInfo>>({});
  const [localArtifactCategories, setLocalArtifactCategories] = createSignal<Record<string, ArtifactCategorySummary[]>>({});
  const [localLoadingDetails, setLocalLoadingDetails] = createSignal<Set<string>>(new Set());
  
  // Use manager when provided, otherwise use local state
  const databases = () => props.manager ? props.manager.databases() : localDatabases();
  
  const selectedDb = () => props.manager ? props.manager.selectedDatabase()?.path ?? null : localSelectedDb();
  const setSelectedDb = (path: string | null) => {
    if (props.manager) {
      const db = databases().find(d => d.path === path) || null;
      props.manager.selectDatabase(db);
    } else {
      setLocalSelectedDb(path);
    }
  };
  
  const axiomCaseInfo = () => props.manager ? props.manager.axiomCaseInfo() : localAxiomCaseInfo();
  const loadingDetails = () => props.manager ? props.manager.loadingDetails() : localLoadingDetails();
  
  // UI-only state (always local)
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  /** Load AXIOM case info when database is selected */
  const loadAxiomDetails = async (db: ProcessedDatabase) => {
    // Use manager's loadAxiomDetails when available
    if (props.manager) {
      await props.manager.loadAxiomDetails(db);
      return;
    }
    
    // Fallback to local loading
    if (db.db_type !== 'MagnetAxiom') return;
    if (axiomCaseInfo()[db.path]) return; // Already loaded
    
    const loadingSet = new Set(localLoadingDetails());
    loadingSet.add(db.path);
    setLocalLoadingDetails(loadingSet);
    
    try {
      // Load case info
      const caseInfo = await invoke<AxiomCaseInfo>('get_axiom_case_info', { path: db.path });
      setLocalAxiomCaseInfo({ ...axiomCaseInfo(), [db.path]: caseInfo });
      
      // Load artifact categories
      const categories = await invoke<ArtifactCategorySummary[]>('get_axiom_artifact_categories', { path: db.path });
      setLocalArtifactCategories({ ...localArtifactCategories(), [db.path]: categories });
    } catch (err) {
      console.warn(`Failed to load AXIOM details for ${db.path}:`, err);
    } finally {
      const newLoading = new Set(localLoadingDetails());
      newLoading.delete(db.path);
      setLocalLoadingDetails(newLoading);
    }
  };

  /** Select a database */
  const selectDatabase = async (db: ProcessedDatabase) => {
    setSelectedDb(db.path);
    props.onSelectDatabase?.(db);
    // Load details when selecting
    await loadAxiomDetails(db);
  };

  /** Scan a directory for processed databases */
  const scanDirectory = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select folder to scan for processed databases',
      });

      if (!selected) return;

      setLoading(true);
      setError(null);

      const results = await invoke<ProcessedDatabase[]>('scan_processed_databases', {
        path: selected,
        recursive: true,
      });

      // Add found databases using manager or local state
      if (results.length > 0) {
        if (props.manager) {
          props.manager.addDatabases(results);
        } else {
          const existingPaths = new Set(databases().map(db => db.path));
          const newDbs = results.filter(db => !existingPaths.has(db.path));
          if (newDbs.length > 0) {
            setLocalDatabases([...localDatabases(), ...newDbs]);
          }
        }
      }

      if (results.length === 0) {
        setError('No processed databases found in selected directory');
      }
    } catch (err) {
      console.error('Scan error:', err);
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  /** Add a specific database file */
  const addDatabaseFile = async () => {
    try {
      const selected = await open({
        multiple: true,
        title: 'Select processed database file(s)',
        filters: [
          {
            name: 'Forensic Databases',
            extensions: ['db', 'sqlite', 'sqlite3', 'case', 'ufdr', 'xml', 'json'],
          },
          {
            name: 'All Files',
            extensions: ['*'],
          },
        ],
      });

      if (!selected || (Array.isArray(selected) && selected.length === 0)) return;

      setLoading(true);
      setError(null);

      const paths = Array.isArray(selected) ? selected : [selected];
      
      for (const path of paths) {
        try {
          const dbInfo = await invoke<ProcessedDatabase | null>('get_processed_db_details', {
            path,
          });

          if (dbInfo) {
            if (props.manager) {
              props.manager.addDatabase(dbInfo);
            } else {
              const existingPaths = new Set(databases().map(db => db.path));
              if (!existingPaths.has(dbInfo.path)) {
                setLocalDatabases([...localDatabases(), dbInfo]);
              }
            }
          }
        } catch (err) {
          console.warn(`Could not parse ${path}:`, err);
        }
      }
    } catch (err) {
      console.error('Add database error:', err);
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  /** Remove a database from the list */
  const removeDatabaseItem = (path: string) => {
    if (props.manager) {
      props.manager.removeDatabase(path);
    } else {
      setLocalDatabases(localDatabases().filter(db => db.path !== path));
      if (selectedDb() === path) {
        setLocalSelectedDb(null);
      }
    }
  };

  /** Clear all databases */
  const clearAll = () => {
    if (props.manager) {
      props.manager.clearAll();
    } else {
      setLocalDatabases([]);
      setLocalSelectedDb(null);
    }
  };

  return (
    <div class="flex flex-col h-full bg-bg-panel rounded overflow-hidden">
      {/* Header with actions */}
      <div class="flex items-center justify-between px-2 py-1 bg-bg-card border-b border-border shrink-0">
        <h3 class={`m-0 ${UI_FONT_COMPACT} font-semibold text-txt flex items-center ${UI_GAP_SMALL}`}>
          <HiOutlineCircleStack class={UI_ICON_COMPACT} /> Processed Databases
        </h3>
        <div class={`flex ${UI_GAP_SMALL}`}>
          <button 
            class={`bg-transparent border border-transparent rounded px-1 py-0.5 cursor-pointer ${UI_FONT_COMPACT} transition-all duration-150 hover:bg-bg-hover hover:border-border disabled:opacity-50 disabled:cursor-not-allowed flex items-center`}
            onClick={scanDirectory} 
            disabled={loading()}
            title="Scan folder for databases"
          >
            <HiOutlineMagnifyingGlass class={UI_ICON_COMPACT} />
          </button>
          <button 
            class={`bg-transparent border border-transparent rounded px-1 py-0.5 cursor-pointer ${UI_FONT_COMPACT} transition-all duration-150 hover:bg-bg-hover hover:border-border disabled:opacity-50 disabled:cursor-not-allowed flex items-center`}
            onClick={addDatabaseFile} 
            disabled={loading()}
            title="Add database file"
          >
            <HiOutlinePlus class={UI_ICON_COMPACT} />
          </button>
          <Show when={databases().length > 0}>
            <button 
              class={`bg-transparent border border-transparent rounded px-1 py-0.5 cursor-pointer ${UI_FONT_COMPACT} transition-all duration-150 hover:bg-bg-hover hover:border-border flex items-center`}
              onClick={clearAll}
              title="Clear all"
            >
              <HiOutlineTrash class={UI_ICON_COMPACT} />
            </button>
          </Show>
        </div>
      </div>

      {/* Loading indicator */}
      <Show when={loading()}>
        <div class={`flex items-center justify-center ${UI_GAP_SMALL} p-2 text-txt-muted ${UI_FONT_COMPACT}`}>
          <HiOutlineArrowPath class={`${UI_ICON_COMPACT} animate-spin`} /> Scanning...
        </div>
      </Show>

      {/* Error message */}
      <Show when={error()}>
        <div class={`flex items-center justify-between px-2 py-1 m-1 bg-error-soft border border-error/30 rounded text-error ${UI_FONT_COMPACT}`}>
          <span class={`flex items-center ${UI_GAP_SMALL}`}><HiOutlineExclamationTriangle class={UI_ICON_COMPACT} /> {error()}</span>
          <button class={`bg-transparent border-none px-1 py-0.5 cursor-pointer ${UI_FONT_COMPACT} opacity-70 hover:opacity-100 transition-opacity flex items-center`} onClick={() => setError(null)}>
            <HiOutlineXMark class={UI_ICON_COMPACT} />
          </button>
        </div>
      </Show>

      {/* Empty state */}
      <Show when={databases().length === 0 && !loading()}>
        <div class={`flex flex-col items-center justify-center px-2 py-4 text-center text-txt-muted flex-1 ${UI_FONT_COMPACT}`}>
          <p class="my-0.5">No processed databases loaded</p>
          <p class="text-txt-faint">Click scan to find databases or add files</p>
          <p class="text-txt-faint">Supports: AXIOM, Cellebrite PA, X-Ways, Autopsy, EnCase, FTK</p>
        </div>
      </Show>

      {/* Database tree - expandable items like FilePanel */}
      <div class="flex-1 overflow-y-auto py-1">
        <For each={databases()}>
          {(db) => {
            const [expanded, setExpanded] = createSignal(true);
            const [keywordsExpanded, setKeywordsExpanded] = createSignal(false);
            const isSelected = () => selectedDb() === db.path;
            const isLoading = () => loadingDetails().has(db.path);
            const caseInfo = () => axiomCaseInfo()[db.path];
            const currentDetailView = () => props.manager?.detailView();
            
            // Get display name
            const displayName = () => caseInfo()?.case_name || db.case_name || db.name || ellipsePath(db.path, 30);
            
            // Keyword files from case info
            const keywordFiles = () => caseInfo()?.keyword_info?.keyword_files || [];
            const totalKeywords = () => caseInfo()?.keyword_info?.keywords?.length || 0;

            return (
              <div class={`border-b border-border last:border-b-0 ${isSelected() ? 'bg-accent-soft' : ''}`}>
                {/* Main database header - expandable */}
                <div 
                  class={`flex items-center ${UI_GAP_SMALL} py-0.5 px-1 cursor-pointer transition-colors duration-150 hover:bg-bg-hover`}
                  onClick={() => {
                    selectDatabase(db);
                    setExpanded(!expanded());
                  }}
                >
                  <span class={`${UI_FONT_COMPACT} text-txt-faint transition-transform duration-150 shrink-0 w-2.5 text-center ${expanded() ? 'rotate-90' : ''}`}>
                    ▶
                  </span>
                  <span class={`${UI_FONT_COMPACT} shrink-0 leading-none`}>{getDbTypeIcon(db.db_type)}</span>
                  <div class={`flex-1 min-w-0 flex flex-col ${UI_GAP_COMPACT}`}>
                    <div class={`${UI_FONT_COMPACT} font-semibold text-txt whitespace-nowrap overflow-hidden text-ellipsis`}>{displayName()}</div>
                    <div class={`flex flex-wrap ${UI_GAP_SMALL} ${UI_FONT_COMPACT} text-txt-muted`}>
                      <span class="text-accent font-medium">{getDbTypeName(db.db_type)}</span>
                      <Show when={caseInfo()?.total_artifacts}>
                        <span class="text-success">{caseInfo()!.total_artifacts.toLocaleString()} artifacts</span>
                      </Show>
                      <Show when={db.total_size}>
                        <span class="text-txt-faint">{formatSize(db.total_size)}</span>
                      </Show>
                    </div>
                    <Show when={caseInfo()?.examiner || db.examiner}>
                      <div class={`${UI_FONT_COMPACT} text-txt-faint`}>👤 {caseInfo()?.examiner || db.examiner}</div>
                    </Show>
                  </div>
                  <div class={`flex items-center ${UI_GAP_SMALL} opacity-0 transition-opacity group-hover:opacity-100 ${isLoading() ? 'opacity-100' : 'hover:opacity-100'}`}>
                    <Show when={isLoading()}>
                      <HiOutlineArrowPath class={`${UI_ICON_COMPACT} animate-spin text-accent`} />
                    </Show>
                    <button 
                      class={`bg-transparent border-none px-1 py-0.5 cursor-pointer ${UI_FONT_COMPACT} text-txt-faint opacity-70 hover:opacity-100 hover:text-error transition-all flex items-center`}
                      onClick={(e) => {
                        e.stopPropagation();
                        removeDatabaseItem(db.path);
                      }}
                      title="Remove from list"
                    >
                      <HiOutlineXMark class={UI_ICON_COMPACT} />
                    </button>
                  </div>
                </div>

                {/* Expanded content - sub-items */}
                <Show when={expanded() && caseInfo()}>
                  <div class="pl-3 pb-0.5 border-l border-border ml-[14px]">
                    {/* Case Report */}
                    <div 
                      class={`flex items-center ${UI_GAP_SMALL} px-1 py-0.5 cursor-pointer rounded ${UI_FONT_COMPACT} text-txt transition-all duration-150 hover:bg-bg-hover ${currentDetailView()?.type === 'case' ? 'bg-accent-soft text-accent' : ''}`}
                      onClick={(e) => {
                        e.stopPropagation();
                        props.manager?.setDetailView({ type: 'case' });
                      }}
                    >
                      <HiOutlineClipboardDocument class={`${UI_ICON_COMPACT} shrink-0`} />
                      <span class="flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">Case Report</span>
                    </div>

                    {/* Evidence Sources */}
                    <div 
                      class={`flex items-center ${UI_GAP_SMALL} px-1 py-0.5 cursor-pointer rounded ${UI_FONT_COMPACT} text-txt transition-all duration-150 hover:bg-bg-hover ${currentDetailView()?.type === 'evidence' ? 'bg-accent-soft text-accent' : ''}`}
                      onClick={(e) => {
                        e.stopPropagation();
                        props.manager?.setDetailView({ type: 'evidence' });
                      }}
                    >
                      <HiOutlineFolder class={`${UI_ICON_COMPACT} shrink-0`} />
                      <span class="flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">Evidence</span>
                      <Show when={caseInfo()?.evidence_sources?.length}>
                        <span class={`${UI_FONT_COMPACT} text-txt-faint`}>({caseInfo()!.evidence_sources.length})</span>
                      </Show>
                    </div>

                    {/* Keywords Section - collapsible */}
                    <div>
                      <div 
                        class={`flex items-center ${UI_GAP_SMALL} pl-0.5 pr-1 py-0.5 cursor-pointer rounded ${UI_FONT_COMPACT} text-txt transition-all duration-150 hover:bg-bg-hover ${currentDetailView()?.type === 'keywords' ? 'bg-accent-soft text-accent' : ''}`}
                        onClick={(e) => {
                          e.stopPropagation();
                          props.manager?.setDetailView({ type: 'keywords' });
                          setKeywordsExpanded(!keywordsExpanded());
                        }}
                      >
                        <span class={`${UI_FONT_COMPACT} text-txt-faint transition-transform duration-150 w-2 text-center ${keywordsExpanded() ? 'rotate-90' : ''}`}>
                          ▶
                        </span>
                        <HiOutlineKey class={`${UI_ICON_COMPACT} shrink-0`} />
                        <span class="flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">Keywords</span>
                        <span class={`${UI_FONT_COMPACT} text-txt-faint`}>({totalKeywords()})</span>
                      </div>

                      {/* Keyword Files - nested under Keywords */}
                      <Show when={keywordsExpanded() && keywordFiles().length > 0}>
                        <div class="pl-3 mt-0.5">
                          <For each={keywordFiles()}>
                            {(file: AxiomKeywordFile) => {
                              const isActive = () => {
                                const view = currentDetailView();
                                return view?.type === 'keyword-file' && view.file?.file_name === file.file_name;
                              };
                              return (
                                <div 
                                  class={`flex items-center ${UI_GAP_SMALL} px-1 py-0.5 cursor-pointer rounded-sm ${UI_FONT_COMPACT} text-txt-muted transition-all duration-150 hover:bg-bg-hover hover:text-txt ${isActive() ? 'bg-accent-soft text-accent' : ''}`}
                                  onClick={(e) => {
                                    e.stopPropagation();
                                    props.manager?.setDetailView({ type: 'keyword-file', file });
                                  }}
                                  title={file.file_path}
                                >
                                  <HiOutlineDocument class={`${UI_ICON_COMPACT} shrink-0`} />
                                  <span class="flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">{ellipsePath(file.file_name, 20)}</span>
                                  <span class={`${UI_FONT_COMPACT} text-txt-faint font-mono`}>{file.record_count}</span>
                                </div>
                              );
                            }}
                          </For>
                        </div>
                      </Show>
                    </div>

                    {/* Artifacts */}
                    <div 
                      class={`flex items-center ${UI_GAP_SMALL} px-1 py-0.5 cursor-pointer rounded ${UI_FONT_COMPACT} text-txt transition-all duration-150 hover:bg-bg-hover ${currentDetailView()?.type === 'artifacts' ? 'bg-accent-soft text-accent' : ''}`}
                      onClick={(e) => {
                        e.stopPropagation();
                        props.manager?.setDetailView({ type: 'artifacts' });
                      }}
                    >
                      <HiOutlineMagnifyingGlass class={`${UI_ICON_COMPACT} shrink-0`} />
                      <span class="flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">Artifacts</span>
                      <Show when={caseInfo()?.total_artifacts}>
                        <span class={`${UI_FONT_COMPACT} text-txt-faint`}>({caseInfo()!.total_artifacts.toLocaleString()})</span>
                      </Show>
                    </div>
                  </div>
                </Show>
              </div>
            );
          }}
        </For>
      </div>

      {/* Summary footer */}
      <Show when={databases().length > 0}>
        <div class={`px-2 py-0.5 bg-bg-card border-t border-border ${UI_FONT_COMPACT} text-txt-faint shrink-0`}>
          <span>{databases().length} database{databases().length !== 1 ? 's' : ''}</span>
        </div>
      </Show>
    </div>
  );
};

export default ProcessedDatabasePanel;
