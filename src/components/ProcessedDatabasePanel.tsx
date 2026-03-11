// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, createSignal, For, Show } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import {
  HiOutlineArrowPath,
  HiOutlineExclamationTriangle,
  HiOutlineXMark,
} from './icons';
import type { 
  ProcessedDatabase, ArtifactInfo,
  ArtifactCategorySummary, AxiomCaseInfo
} from '../types/processed';
import type { ProcessedDatabasesManager } from '../hooks/useProcessedDatabases';
import { ProcessedDatabaseItem } from './processed/ProcessedDatabaseItem';
import { ProcessedDatabaseEmptyState } from './processed/ProcessedDatabaseEmptyState';
import { ProcessedDatabaseToolbar } from './processed/ProcessedDatabaseToolbar';
import { logger } from "../utils/logger";
const log = logger.scope("ProcessedDatabase");

interface ProcessedDatabasePanelProps {
  /** Manager hook for shared state (optional for backward compatibility) */
  manager?: ProcessedDatabasesManager;
  onSelectDatabase?: (db: ProcessedDatabase) => void;
  onSelectArtifact?: (db: ProcessedDatabase, artifact: ArtifactInfo) => void;
}

export const ProcessedDatabasePanel: Component<ProcessedDatabasePanelProps> = (props) => {
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
      log.warn(`Failed to load AXIOM details for ${db.path}:`, err);
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
      log.error('Scan error:', err);
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
          log.warn(`Could not parse ${path}:`, err);
        }
      }
    } catch (err) {
      log.error('Add database error:', err);
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
      <ProcessedDatabaseToolbar
        databaseCount={databases().length}
        loading={loading()}
        onScan={scanDirectory}
        onAdd={addDatabaseFile}
        onClearAll={clearAll}
      />

      {/* Loading indicator */}
      <Show when={loading()}>
        <div class={`flex items-center justify-center gap-1 p-2 text-txt-muted text-compact leading-tight`}>
          <HiOutlineArrowPath class={`w-3 h-3 animate-spin`} /> Scanning...
        </div>
      </Show>

      {/* Error message */}
      <Show when={error()}>
        <div class={`flex items-center justify-between px-2 py-1 m-1 bg-error-soft border border-error/30 rounded text-error text-compact leading-tight`}>
          <span class={`flex items-center gap-1`}><HiOutlineExclamationTriangle class="w-3 h-3" /> {error()}</span>
          <button class={`bg-transparent border-none px-1 py-0.5 cursor-pointer text-compact leading-tight opacity-70 hover:opacity-100 transition-opacity flex items-center`} onClick={() => setError(null)}>
            <HiOutlineXMark class="w-3 h-3" />
          </button>
        </div>
      </Show>

      {/* Empty state */}
      <Show when={databases().length === 0 && !loading()}>
        <ProcessedDatabaseEmptyState />
      </Show>

      {/* Database tree - expandable items like FilePanel */}
      <div class="flex-1 overflow-y-auto py-1">
        <For each={databases()}>
          {(db) => (
            <ProcessedDatabaseItem
              db={db}
              isSelected={selectedDb() === db.path}
              isLoading={loadingDetails().has(db.path)}
              caseInfo={axiomCaseInfo()[db.path]}
              currentDetailView={props.manager?.detailView()}
              onSelect={() => selectDatabase(db)}
              onRemove={() => removeDatabaseItem(db.path)}
              onSetDetailView={(view) => props.manager?.setDetailView(view)}
            />
          )}
        </For>
      </div>

      {/* Summary footer */}
      <Show when={databases().length > 0}>
        <div class={`px-2 py-0.5 bg-bg-card border-t border-border text-compact leading-tight text-txt-faint shrink-0`}>
          <span>{databases().length} database{databases().length !== 1 ? 's' : ''}</span>
        </div>
      </Show>
    </div>
  );
};
