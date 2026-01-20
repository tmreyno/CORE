// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, createSignal, createEffect, Show, For, on, onMount, onCleanup } from 'solid-js';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import {
  HiOutlineFolder,
  HiOutlineCircleStack,
  HiOutlineXMark,
  HiOutlineArchiveBox,
  HiOutlineClipboardDocumentList,
  HiOutlineFingerPrint,
} from './icons';
import { useFocusTrap } from '../hooks/useFocusTrap';
import type { ProcessedDatabase } from '../types/processed';

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
    document.addEventListener('keydown', handleEscape);
    onCleanup(() => document.removeEventListener('keydown', handleEscape));
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
  
  // Complete setup
  const handleComplete = () => {
    const locations: ProjectLocations = {
      projectRoot: props.projectRoot,
      evidencePath: evidencePath(),
      processedDbPath: processedDbPath(),
      caseDocumentsPath: caseDocumentsPath(),
      discoveredEvidence: discoveredEvidence(),
      discoveredDatabases: discoveredDatabases(),
      loadStoredHashes: loadStoredHashes(),
    };
    props.onComplete(locations);
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
                    <Show when={discoveredEvidence().length > 0}>
                      <span class="badge-sm success">{discoveredEvidence().length} files</span>
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
                      <For each={suggestedEvidence().slice(0, 3)}>
                        {(path) => (
                          <button
                            class="chip"
                            classList={{ active: evidencePath() === path }}
                            onClick={async () => {
                              setEvidencePath(path);
                              await discoverEvidence(path);
                            }}
                          >
                            {path.split('/').pop()}
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
                    <Show when={discoveredDatabases().length > 0}>
                      <span class="badge-sm success">{discoveredDatabases().length} DBs</span>
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
                      <For each={suggestedProcessed().slice(0, 3)}>
                        {(path) => (
                          <button
                            class="chip"
                            classList={{ active: processedDbPath() === path }}
                            onClick={async () => {
                              setProcessedDbPath(path);
                              await discoverDatabases(path);
                            }}
                          >
                            {path.split('/').pop()}
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
                      <For each={suggestedCaseDocs().slice(0, 3)}>
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
                            {path.split('/').pop()}
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
              <button class="btn-action-primary" onClick={handleComplete} disabled={scanning()}>
                {scanning() ? 'Scanning...' : 'Continue'}
              </button>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default ProjectSetupWizard;
