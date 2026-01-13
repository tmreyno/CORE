// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import type { 
  ProcessedDatabase, ArtifactCategorySummary, AxiomCaseInfo, AxiomKeywordFile
} from '../types/processed';

/** Detail view types for the center panel */
export type DetailViewType = 
  | { type: 'case' }
  | { type: 'evidence' }
  | { type: 'keywords' }
  | { type: 'keyword-file'; file: AxiomKeywordFile }
  | { type: 'artifacts' }
  | null;

/**
 * Hook to manage processed database state across components.
 * This enables the left panel to show the database list/tree,
 * and the center panel to show details of the selected database.
 */
export function useProcessedDatabases() {
  // List of loaded databases
  const [databases, setDatabases] = createSignal<ProcessedDatabase[]>([]);
  
  // Currently selected database
  const [selectedDatabase, setSelectedDatabase] = createSignal<ProcessedDatabase | null>(null);
  
  // AXIOM case info (cached by path)
  const [axiomCaseInfo, setAxiomCaseInfo] = createSignal<Record<string, AxiomCaseInfo>>({});
  
  // Artifact categories (cached by path)
  const [artifactCategories, setArtifactCategories] = createSignal<Record<string, ArtifactCategorySummary[]>>({});
  
  // Loading state for individual databases
  const [loadingDetails, setLoadingDetails] = createSignal<Set<string>>(new Set());

  // Current detail view for center panel
  const [detailView, setDetailView] = createSignal<DetailViewType>({ type: 'case' });

  /** Load AXIOM case info for a database */
  const loadAxiomDetails = async (db: ProcessedDatabase) => {
    if (db.db_type !== 'MagnetAxiom') return;
    if (axiomCaseInfo()[db.path]) return; // Already loaded
    
    const loadingSet = new Set(loadingDetails());
    loadingSet.add(db.path);
    setLoadingDetails(loadingSet);
    
    try {
      // Load case info
      const caseInfo = await invoke<AxiomCaseInfo>('get_axiom_case_info', { path: db.path });
      setAxiomCaseInfo(prev => ({ ...prev, [db.path]: caseInfo }));
      
      // Load artifact categories
      const categories = await invoke<ArtifactCategorySummary[]>('get_axiom_artifact_categories', { path: db.path });
      setArtifactCategories(prev => ({ ...prev, [db.path]: categories }));
    } catch (err) {
      console.warn(`Failed to load AXIOM details for ${db.path}:`, err);
    } finally {
      const newLoading = new Set(loadingDetails());
      newLoading.delete(db.path);
      setLoadingDetails(newLoading);
    }
  };

  /** Select a database and load its details */
  const selectDatabase = async (db: ProcessedDatabase | null) => {
    setSelectedDatabase(db);
    if (db) {
      await loadAxiomDetails(db);
    }
  };

  /** Add a database to the list */
  const addDatabase = (db: ProcessedDatabase) => {
    const existingPaths = new Set(databases().map(d => d.path));
    if (!existingPaths.has(db.path)) {
      setDatabases(prev => [...prev, db]);
    }
  };

  /** Add multiple databases to the list */
  const addDatabases = (dbs: ProcessedDatabase[]) => {
    const existingPaths = new Set(databases().map(d => d.path));
    const newDbs = dbs.filter(db => !existingPaths.has(db.path));
    if (newDbs.length > 0) {
      setDatabases(prev => [...prev, ...newDbs]);
    }
  };

  /** Remove a database from the list */
  const removeDatabase = (path: string) => {
    setDatabases(prev => prev.filter(db => db.path !== path));
    if (selectedDatabase()?.path === path) {
      setSelectedDatabase(null);
    }
  };

  /** Clear all databases */
  const clearAll = () => {
    setDatabases([]);
    setSelectedDatabase(null);
    setAxiomCaseInfo({});
    setArtifactCategories({});
  };

  /** Get case info for currently selected database */
  const selectedCaseInfo = () => {
    const db = selectedDatabase();
    if (!db) return null;
    return axiomCaseInfo()[db.path] || null;
  };

  /** Get artifact categories for currently selected database */
  const selectedCategories = () => {
    const db = selectedDatabase();
    if (!db) return [];
    return artifactCategories()[db.path] || [];
  };

  /** Check if details are loading for a specific database */
  const isLoadingDetails = (path: string) => {
    return loadingDetails().has(path);
  };

  /** Check if selected database is loading */
  const isSelectedLoading = () => {
    const db = selectedDatabase();
    return db ? loadingDetails().has(db.path) : false;
  };

  return {
    // State
    databases,
    selectedDatabase,
    axiomCaseInfo,
    artifactCategories,
    loadingDetails,
    detailView,
    
    // Computed
    selectedCaseInfo,
    selectedCategories,
    isLoadingDetails,
    isSelectedLoading,
    
    // Actions
    selectDatabase,
    addDatabase,
    addDatabases,
    removeDatabase,
    clearAll,
    loadAxiomDetails,
    setDetailView,
  };
}

export type ProcessedDatabasesManager = ReturnType<typeof useProcessedDatabases>;
