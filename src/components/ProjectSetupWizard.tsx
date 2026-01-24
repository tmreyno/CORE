// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, createSignal, createEffect, createMemo, Show, For, on, onMount } from 'solid-js';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { makeEventListener } from "@solid-primitives/event-listener";
import { getBasename } from '../utils';
import {
  HiOutlineFolder,
  HiOutlineCircleStack,
  HiOutlineXMark,
  HiOutlineArchiveBox,
  HiOutlineClipboardDocumentList,
  HiOutlineFingerPrint,
  HiOutlineCheckCircle,
} from './icons';
import { useFocusTrap } from '../hooks/useFocusTrap';
import type { ProcessedDatabase } from '../types/processed';
import type { StoredHash } from '../types';

export interface ProjectLocations {
  /** Root project directory */
  projectRoot: string;
  /** Path to evidence files directory */
  evidencePath: string;
  /** Path to processed databases directory */
  processedDbPath: string;
  /** Path to case documents directory (COC, forms, etc.) */
  caseDocumentsPath: string;
  /** Auto-discovered evidence files */
  discoveredEvidence: string[];
  /** Auto-discovered processed databases */
  discoveredDatabases: ProcessedDatabase[];
  /** Whether to load stored hashes on project open */
  loadStoredHashes: boolean;
  /** Pre-loaded stored hashes map (path -> StoredHash[]) - fast hash-only extraction */
  loadedStoredHashes?: Map<string, StoredHash[]>;
}

interface ProjectSetupWizardProps {
  /** The selected project root directory */
  projectRoot: string;
  /** Whether the wizard is visible */
  isOpen: boolean;
  /** Called when wizard is closed/cancelled */
  onClose: () => void;
  /** Called when setup is complete with locations */
  onComplete: (locations: ProjectLocations) => void;
}

/**
 * Project Setup Wizard - Prompts user to select evidence and processed database locations
 * after opening a project directory.
 */
const ProjectSetupWizard: Component<ProjectSetupWizardProps> = (props) => {
  // Focus trap for modal accessibility
  let modalRef: HTMLDivElement | undefined;
  useFocusTrap(() => modalRef, () => props.isOpen);
  
  // Close on Escape key
  onMount(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && props.isOpen) {
        props.onClose();
      }
    };
    // makeEventListener auto-cleans up on component unmount
    makeEventListener(document, 'keydown', handleEscape);
  });
  
  // Step state: 0 = scanning, 1 = configure locations, 2 = complete
  const [step, setStep] = createSignal(0);
  
  // Paths state
  const [evidencePath, setEvidencePath] = createSignal('');
  const [processedDbPath, setProcessedDbPath] = createSignal('');
  const [caseDocumentsPath, setCaseDocumentsPath] = createSignal('');
  
  // Options state
  const [loadStoredHashes, setLoadStoredHashes] = createSignal(true);
  
  // Auto-discovery results
  const [discoveredEvidence, setDiscoveredEvidence] = createSignal<string[]>([]);
  const [discoveredDatabases, setDiscoveredDatabases] = createSignal<ProcessedDatabase[]>([]);
  
  // Scanning state
  const [scanning, setScanning] = createSignal(false);
  const [scanMessage, setScanMessage] = createSignal('Scanning project directory...');
  const [error, setError] = createSignal<string | null>(null);
  
  // Suggested paths from auto-discovery
  const [suggestedEvidence, setSuggestedEvidence] = createSignal<string[]>([]);
  const [suggestedProcessed, setSuggestedProcessed] = createSignal<string[]>([]);
  const [suggestedCaseDocs, setSuggestedCaseDocs] = createSignal<string[]>([]);
  
  // Discovered case documents count
  const [discoveredCaseDocCount, setDiscoveredCaseDocCount] = createSignal(0);
  
  // Track if we've started discovery for this open
  const [discoveryStarted, setDiscoveryStarted] = createSignal(false);

  // Hash loading state (Step 2)
  const [hashLoadingProgress, setHashLoadingProgress] = createSignal({ current: 0, total: 0, currentFile: '', hashCount: 0 });
  const [loadedStoredHashes, setLoadedStoredHashes] = createSignal<Map<string, StoredHash[]>>(new Map());
  const [hashLoadingCancelled, setHashLoadingCancelled] = createSignal(false);

  // === Derived/Memoized Values ===
  
  // Whether hash loading step should be shown
  const showHashLoadingStep = createMemo(() => 
    loadStoredHashes() && discoveredEvidence().length > 0
  );
  
  // Hash loading progress percentage
  const hashProgressPercent = createMemo(() => {
    const progress = hashLoadingProgress();
    return progress.total > 0 ? (progress.current / progress.total) * 100 : 0;
  });
  
  // Evidence count for display
  const evidenceCount = createMemo(() => discoveredEvidence().length);
  
  // Database count for display
  const databaseCount = createMemo(() => discoveredDatabases().length);
  
  // Suggested paths (limited to 3 for chips display)
  const evidenceChips = createMemo(() => suggestedEvidence().slice(0, 3));
  const processedChips = createMemo(() => suggestedProcessed().slice(0, 3));
  const caseDocsChips = createMemo(() => suggestedCaseDocs().slice(0, 3));

  // Discover evidence files in a directory
  const discoverEvidence = async (path: string): Promise<string[]> => {
    try {
      console.log('[Wizard] Discovering evidence files in:', path);
      const files = await invoke<string[]>('discover_evidence_files', { dirPath: path, recursive: true });
      console.log('[Wizard] Found evidence files:', files.length);
      setDiscoveredEvidence(files);
      return files;
    } catch (err) {
      console.warn('[Wizard] Failed to discover evidence:', err);
      setDiscoveredEvidence([]);
      return [];
    }
  };
  
  // Discover processed databases in a directory
  const discoverDatabases = async (path: string): Promise<ProcessedDatabase[]> => {
    try {
      console.log('[Wizard] Discovering processed databases in:', path);
      const dbs = await invoke<ProcessedDatabase[]>('scan_for_processed_databases', { dirPath: path });
      console.log('[Wizard] Found processed databases:', dbs.length);
      setDiscoveredDatabases(dbs);
      return dbs;
    } catch (err) {
      console.warn('[Wizard] Failed to discover processed databases:', err);
      setDiscoveredDatabases([]);
      return [];
    }
  };

  // Auto-discovery function
  const startAutoDiscovery = async (projectRoot: string) => {
    console.log('[Wizard] Starting auto-discovery for:', projectRoot);
    setStep(0);
    setScanning(true);
    setError(null);
    setScanMessage('Looking for common directory structures...');
    
    try {
      // Try to find common evidence directories
      const commonEvidencePaths = [
        '1.Evidence',
        'Evidence', 
        'evidence',
        'Images',
        'Forensic Images',
        'Source',
      ];
      
      const commonProcessedPaths = [
        '2.Processed',
        '2.Processed.Database',
        'Processed',
        'processed', 
        'AXIOM',
        'Cellebrite',
        'Exports',
        'Analysis',
      ];
      
      const commonCaseDocPaths = [
        '4.Case.Documents',
        'Case.Documents',
        'Case Documents',
        'CaseDocuments',
        'Documents',
        'Paperwork',
        'Forms',
        'COC',
        'Chain of Custody',
      ];
      
      // Check for existing directories
      const evidenceMatches: string[] = [];
      const processedMatches: string[] = [];
      const caseDocMatches: string[] = [];
      
      setScanMessage('Checking for evidence directories...');
      
      for (const subdir of commonEvidencePaths) {
        const testPath = `${projectRoot}/${subdir}`;
        try {
          const exists = await invoke<boolean>('path_exists', { path: testPath });
          console.log('[Wizard] Checking path:', testPath, '- exists:', exists);
          if (exists) {
            const isDir = await invoke<boolean>('path_is_directory', { path: testPath });
            if (isDir) {
              evidenceMatches.push(testPath);
            }
          }
        } catch (e) {
          console.warn('[Wizard] Error checking path:', testPath, e);
        }
      }
      
      // Always add project root as fallback
      evidenceMatches.push(projectRoot);
      
      setScanMessage('Checking for processed database directories...');
      
      for (const subdir of commonProcessedPaths) {
        const testPath = `${projectRoot}/${subdir}`;
        try {
          const exists = await invoke<boolean>('path_exists', { path: testPath });
          console.log('[Wizard] Checking processed path:', testPath, '- exists:', exists);
          if (exists) {
            const isDir = await invoke<boolean>('path_is_directory', { path: testPath });
            if (isDir) {
              processedMatches.push(testPath);
            }
          }
        } catch (e) {
          console.warn('[Wizard] Error checking processed path:', testPath, e);
        }
      }
      
      // Always add project root as fallback
      processedMatches.push(projectRoot);
      
      setScanMessage('Checking for case document directories...');
      
      for (const subdir of commonCaseDocPaths) {
        const testPath = `${projectRoot}/${subdir}`;
        try {
          const exists = await invoke<boolean>('path_exists', { path: testPath });
          console.log('[Wizard] Checking case doc path:', testPath, '- exists:', exists);
          if (exists) {
            const isDir = await invoke<boolean>('path_is_directory', { path: testPath });
            if (isDir) {
              caseDocMatches.push(testPath);
            }
          }
        } catch (e) {
          console.warn('[Wizard] Error checking case doc path:', testPath, e);
        }
      }
      
      // Always add project root as fallback
      caseDocMatches.push(projectRoot);
      
      console.log('[Wizard] Evidence matches:', evidenceMatches);
      console.log('[Wizard] Processed matches:', processedMatches);
      console.log('[Wizard] Case doc matches:', caseDocMatches);
      
      setSuggestedEvidence(evidenceMatches);
      setSuggestedProcessed(processedMatches);
      setSuggestedCaseDocs(caseDocMatches);
      
      // Set defaults to first matches
      const defaultEvidence = evidenceMatches[0] || projectRoot;
      const defaultProcessed = processedMatches[0] || projectRoot;
      const defaultCaseDocs = caseDocMatches[0] || projectRoot;
      
      setEvidencePath(defaultEvidence);
      setProcessedDbPath(defaultProcessed);
      setCaseDocumentsPath(defaultCaseDocs);
      
      // Now scan the selected evidence path for files
      setScanMessage('Discovering evidence files...');
      await discoverEvidence(defaultEvidence);
      
      // And scan for processed databases
      setScanMessage('Discovering processed databases...');
      await discoverDatabases(defaultProcessed);
      
      // Discover case documents (optional, just count for display)
      setScanMessage('Looking for case documents...');
      try {
        const docs = await invoke<{ length: number }[]>('discover_case_documents', { evidencePath: defaultCaseDocs });
        setDiscoveredCaseDocCount(Array.isArray(docs) ? docs.length : 0);
      } catch {
        setDiscoveredCaseDocCount(0);
      }
      
      console.log('[Wizard] Auto-discovery complete, moving to step 1');
      setStep(1);
    } catch (err) {
      console.error('[Wizard] Auto-discovery error:', err);
      setError(String(err));
      // Still allow manual configuration
      setEvidencePath(projectRoot);
      setProcessedDbPath(projectRoot);
      setCaseDocumentsPath(projectRoot);
      setStep(1);
    } finally {
      setScanning(false);
    }
  };

  // Use createEffect with explicit dependency tracking to start discovery when wizard opens
  createEffect(on(
    () => [props.isOpen, props.projectRoot] as const,
    ([isOpen, projectRoot]) => {
      if (isOpen && projectRoot && !discoveryStarted()) {
        console.log('[Wizard] Effect triggered - starting discovery');
        setDiscoveryStarted(true);
        startAutoDiscovery(projectRoot);
      } else if (!isOpen) {
        // Reset state when closed
        setDiscoveryStarted(false);
        setStep(0);
        setDiscoveredEvidence([]);
        setDiscoveredDatabases([]);
        setSuggestedEvidence([]);
        setSuggestedProcessed([]);
        setSuggestedCaseDocs([]);
        setDiscoveredCaseDocCount(0);
        setLoadStoredHashes(true);
        setError(null);
      }
    }
  ));
  
  // Browse for evidence directory
  const browseEvidence = async () => {
    try {
      const selected = await open({
        title: 'Select Evidence Directory',
        directory: true,
        multiple: false,
        defaultPath: props.projectRoot,
      });
      if (selected) {
        setEvidencePath(selected);
        setScanning(true);
        setScanMessage('Scanning for evidence files...');
        await discoverEvidence(selected);
        setScanning(false);
      }
    } catch (err) {
      setError(String(err));
    }
  };
  
  // Browse for processed database directory
  const browseProcessed = async () => {
    try {
      const selected = await open({
        title: 'Select Processed Database Directory',
        directory: true,
        multiple: false,
        defaultPath: props.projectRoot,
      });
      if (selected) {
        setProcessedDbPath(selected);
        setScanning(true);
        setScanMessage('Scanning for processed databases...');
        await discoverDatabases(selected);
        setScanning(false);
      }
    } catch (err) {
      setError(String(err));
    }
  };
  
  // Browse for case documents directory
  const browseCaseDocs = async () => {
    try {
      const selected = await open({
        title: 'Select Case Documents Directory',
        directory: true,
        multiple: false,
        defaultPath: props.projectRoot,
      });
      if (selected) {
        setCaseDocumentsPath(selected);
        setScanning(true);
        setScanMessage('Looking for case documents...');
        try {
          const docs = await invoke<{ length: number }[]>('discover_case_documents', { evidencePath: selected });
          setDiscoveredCaseDocCount(Array.isArray(docs) ? docs.length : 0);
        } catch {
          setDiscoveredCaseDocCount(0);
        }
        setScanning(false);
      }
    } catch (err) {
      setError(String(err));
    }
  };
  
  // Load stored hashes for all discovered evidence files (Step 2)
  // Uses the fast get_stored_hashes_only command - minimal parsing
  const loadHashesForEvidence = async () => {
    const files = discoveredEvidence();
    if (files.length === 0) {
      // No files, skip to completion
      finalizeSetup();
      return;
    }
    
    setHashLoadingCancelled(false);
    setHashLoadingProgress({ current: 0, total: files.length, currentFile: '', hashCount: 0 });
    const hashMap = new Map<string, StoredHash[]>();
    let totalHashCount = 0;
    
    for (let i = 0; i < files.length; i++) {
      if (hashLoadingCancelled()) {
        // User cancelled, complete with what we have
        setLoadedStoredHashes(hashMap);
        finalizeSetup();
        return;
      }
      
      const filePath = files[i];
      const filename = getBasename(filePath) || filePath;
      setHashLoadingProgress({ current: i + 1, total: files.length, currentFile: filename, hashCount: totalHashCount });
      
      try {
        // Use the fast hash-only extraction command
        const hashes = await invoke<StoredHash[]>("get_stored_hashes_only", { inputPath: filePath });
        if (hashes && hashes.length > 0) {
          hashMap.set(filePath, hashes);
          totalHashCount += hashes.length;
          setHashLoadingProgress(prev => ({ ...prev, hashCount: totalHashCount }));
        }
      } catch (err) {
        console.warn(`[Wizard] Failed to load hashes for ${filename}:`, err);
        // Continue with other files
      }
    }
    
    setLoadedStoredHashes(hashMap);
    finalizeSetup();
  };
  
  // Finalize and call onComplete
  const finalizeSetup = () => {
    const locations: ProjectLocations = {
      projectRoot: props.projectRoot,
      evidencePath: evidencePath(),
      processedDbPath: processedDbPath(),
      caseDocumentsPath: caseDocumentsPath(),
      discoveredEvidence: discoveredEvidence(),
      discoveredDatabases: discoveredDatabases(),
      loadStoredHashes: loadStoredHashes(),
      loadedStoredHashes: loadStoredHashes() ? loadedStoredHashes() : undefined,
    };
    props.onComplete(locations);
  };
  
  // Handle Continue button - either go to step 2 or finalize
  const handleContinue = () => {
    if (showHashLoadingStep()) {
      // Go to step 2 to load hashes
      setStep(2);
      // Start loading hashes
      loadHashesForEvidence();
    } else {
      // Skip hash loading, finalize directly
      finalizeSetup();
    }
  };
  
  // Cancel hash loading
  const cancelHashLoading = () => {
    setHashLoadingCancelled(true);
  };
  
  // Skip setup (use defaults)
  const handleSkip = () => {
    const locations: ProjectLocations = {
      projectRoot: props.projectRoot,
      evidencePath: props.projectRoot,
      processedDbPath: props.projectRoot,
      caseDocumentsPath: props.projectRoot,
      discoveredEvidence: [],
      discoveredDatabases: [],
      loadStoredHashes: true,
    };
    props.onComplete(locations);
  };
  
  // Start discovery when opened - handled by the createEffect above
  // (removed synchronous call that was breaking reactivity)
  
  return (
    <Show when={props.isOpen}>
      <div class="wizard-overlay" onClick={(e) => e.target === e.currentTarget && props.onClose()}>
        <div 
          ref={modalRef}
          class="wizard-modal"
          role="dialog"
          aria-modal="true"
          aria-labelledby="project-wizard-title"
        >
          {/* Header */}
          <div class="wizard-header">
            <h2 id="project-wizard-title" class="flex items-center gap-2">
              <HiOutlineFolder class="w-5 h-5" /> Project Setup
            </h2>
            <button class="wizard-close flex items-center justify-center" onClick={props.onClose} title="Close" aria-label="Close project setup">
              <HiOutlineXMark class="w-4 h-4" />
            </button>
          </div>
          
          {/* Step Indicator */}
          <div class="wizard-steps">
            <div class="step" classList={{ active: step() === 0, complete: step() > 0 }}>
              <span class="step-number">1</span>
              <span class="step-label">Scan</span>
            </div>
            <div class="step-connector" />
            <div class="step" classList={{ active: step() === 1, complete: step() > 1 }}>
              <span class="step-number">2</span>
              <span class="step-label">Configure</span>
            </div>
            <Show when={showHashLoadingStep()}>
              <div class="step-connector" />
              <div class="step" classList={{ active: step() === 2, complete: step() > 2 }}>
                <span class="step-number">3</span>
                <span class="step-label">Load Hashes</span>
              </div>
            </Show>
          </div>
          
          {/* Content */}
          <div class="wizard-content">
            {/* Step 0: Scanning */}
            <Show when={step() === 0}>
              <div class="scanning-state">
                <div class="spinner" />
                <p>{scanMessage()}</p>
                <Show when={error()}>
                  <p class="error-text">{error()}</p>
                </Show>
              </div>
            </Show>
            
            {/* Step 1: Configure Locations */}
            <Show when={step() === 1}>
              <div class="config-section compact">
                {/* Evidence Location - Compact */}
                <div class="location-group-compact">
                  <div class="location-header">
                    <HiOutlineArchiveBox class="w-4 h-4 text-accent" />
                    <span class="location-title">Evidence</span>
                    <Show when={evidenceCount() > 0}>
                      <span class="badge-sm success">{evidenceCount()} files</span>
                    </Show>
                  </div>
                  <div class="location-input-compact">
                    <input
                      type="text"
                      value={evidencePath()}
                      onInput={(e) => setEvidencePath(e.currentTarget.value)}
                      placeholder="Evidence path..."
                    />
                    <button class="btn-browse" onClick={browseEvidence}>...</button>
                  </div>
                  <Show when={suggestedEvidence().length > 1}>
                    <div class="path-chips">
                      <For each={evidenceChips()}>
                        {(path) => (
                          <button
                            class="chip"
                            classList={{ active: evidencePath() === path }}
                            onClick={async () => {
                              setEvidencePath(path);
                              await discoverEvidence(path);
                            }}
                          >
                            {getBasename(path)}
                          </button>
                        )}
                      </For>
                    </div>
                  </Show>
                </div>

                {/* Processed Database Location - Compact */}
                <div class="location-group-compact">
                  <div class="location-header">
                    <HiOutlineCircleStack class="w-4 h-4 text-purple-400" />
                    <span class="location-title">Processed Databases</span>
                    <Show when={databaseCount() > 0}>
                      <span class="badge-sm success">{databaseCount()} DBs</span>
                    </Show>
                  </div>
                  <div class="location-input-compact">
                    <input
                      type="text"
                      value={processedDbPath()}
                      onInput={(e) => setProcessedDbPath(e.currentTarget.value)}
                      placeholder="Processed DB path..."
                    />
                    <button class="btn-browse" onClick={browseProcessed}>...</button>
                  </div>
                  <Show when={suggestedProcessed().length > 1}>
                    <div class="path-chips">
                      <For each={processedChips()}>
                        {(path) => (
                          <button
                            class="chip"
                            classList={{ active: processedDbPath() === path }}
                            onClick={async () => {
                              setProcessedDbPath(path);
                              await discoverDatabases(path);
                            }}
                          >
                            {getBasename(path)}
                          </button>
                        )}
                      </For>
                    </div>
                  </Show>
                </div>

                {/* Case Documents Location - Compact */}
                <div class="location-group-compact">
                  <div class="location-header">
                    <HiOutlineClipboardDocumentList class="w-4 h-4 text-green-400" />
                    <span class="location-title">Case Documents</span>
                    <Show when={discoveredCaseDocCount() > 0}>
                      <span class="badge-sm success">{discoveredCaseDocCount()} docs</span>
                    </Show>
                  </div>
                  <div class="location-input-compact">
                    <input
                      type="text"
                      value={caseDocumentsPath()}
                      onInput={(e) => setCaseDocumentsPath(e.currentTarget.value)}
                      placeholder="Case documents path..."
                    />
                    <button class="btn-browse" onClick={browseCaseDocs}>...</button>
                  </div>
                  <Show when={suggestedCaseDocs().length > 1}>
                    <div class="path-chips">
                      <For each={caseDocsChips()}>
                        {(path) => (
                          <button
                            class="chip"
                            classList={{ active: caseDocumentsPath() === path }}
                            onClick={async () => {
                              setCaseDocumentsPath(path);
                              try {
                                const docs = await invoke<{ length: number }[]>('discover_case_documents', { evidencePath: path });
                                setDiscoveredCaseDocCount(Array.isArray(docs) ? docs.length : 0);
                              } catch {
                                setDiscoveredCaseDocCount(0);
                              }
                            }}
                          >
                            {getBasename(path)}
                          </button>
                        )}
                      </For>
                    </div>
                  </Show>
                </div>

                {/* Options Section */}
                <div class="options-section">
                  <label class="option-row">
                    <input
                      type="checkbox"
                      checked={loadStoredHashes()}
                      onChange={(e) => setLoadStoredHashes(e.currentTarget.checked)}
                    />
                    <HiOutlineFingerPrint class="w-4 h-4 text-amber-400" />
                    <span>Load stored hashes from containers</span>
                  </label>
                </div>
              </div>
            </Show>
            
            {/* Step 2: Loading Stored Hashes */}
            <Show when={step() === 2}>
              <div class="hash-loading-state">
                <div class="hash-loading-header">
                  <HiOutlineFingerPrint class="w-6 h-6 text-amber-400" />
                  <h3>Loading Stored Hashes</h3>
                </div>
                <p class="hash-loading-description">
                  Extracting stored hash values from container metadata...
                </p>
                
                {/* Progress Bar */}
                <div class="hash-progress-container">
                  <div class="hash-progress-bar">
                    <div 
                      class="hash-progress-fill"
                      style={{ width: `${hashProgressPercent()}%` }}
                    />
                  </div>
                  <div class="hash-progress-info">
                    <span class="hash-progress-file">{hashLoadingProgress().currentFile}</span>
                    <span class="hash-progress-count">
                      {hashLoadingProgress().current} / {hashLoadingProgress().total}
                    </span>
                  </div>
                </div>
                
                {/* Hash count summary */}
                <Show when={hashLoadingProgress().hashCount > 0}>
                  <div class="hash-loaded-summary">
                    <HiOutlineCheckCircle class="w-4 h-4 text-success" />
                    <span>{hashLoadingProgress().hashCount} hash(es) found from {loadedStoredHashes().size} container(s)</span>
                  </div>
                </Show>
              </div>
            </Show>
          </div>
          
          {/* Footer */}
          <div class="wizard-footer">
            <Show when={step() === 1}>
              <button class="btn-action-ghost" onClick={handleSkip}>
                Skip
              </button>
              <div class="footer-spacer" />
              <button class="btn-action-secondary" onClick={props.onClose}>
                Cancel
              </button>
              <button class="btn-action-primary" onClick={handleContinue} disabled={scanning()}>
                {scanning() ? 'Scanning...' : 'Continue'}
              </button>
            </Show>
            <Show when={step() === 2}>
              <div class="footer-spacer" />
              <button class="btn-action-secondary" onClick={cancelHashLoading}>
                Skip Loading
              </button>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default ProjectSetupWizard;
