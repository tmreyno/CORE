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
  HiOutlineDocument,
  HiOutlineCircleStack,
  HiOutlineDevicePhoneMobile,
  HiOutlineCpuChip,
  HiOutlineXMark,
  HiOutlineArchiveBox,
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
  /** Auto-discovered evidence files */
  discoveredEvidence: string[];
  /** Auto-discovered processed databases */
  discoveredDatabases: ProcessedDatabase[];
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
      
      // Check for existing directories
      const evidenceMatches: string[] = [];
      const processedMatches: string[] = [];
      
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
      
      console.log('[Wizard] Evidence matches:', evidenceMatches);
      console.log('[Wizard] Processed matches:', processedMatches);
      
      setSuggestedEvidence(evidenceMatches);
      setSuggestedProcessed(processedMatches);
      
      // Set defaults to first matches
      const defaultEvidence = evidenceMatches[0] || projectRoot;
      const defaultProcessed = processedMatches[0] || projectRoot;
      
      setEvidencePath(defaultEvidence);
      setProcessedDbPath(defaultProcessed);
      
      // Now scan the selected evidence path for files
      setScanMessage('Discovering evidence files...');
      await discoverEvidence(defaultEvidence);
      
      // And scan for processed databases
      setScanMessage('Discovering processed databases...');
      await discoverDatabases(defaultProcessed);
      
      console.log('[Wizard] Auto-discovery complete, moving to step 1');
      setStep(1);
    } catch (err) {
      console.error('[Wizard] Auto-discovery error:', err);
      setError(String(err));
      // Still allow manual configuration
      setEvidencePath(projectRoot);
      setProcessedDbPath(projectRoot);
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
  
  // Complete setup
  const handleComplete = () => {
    const locations: ProjectLocations = {
      projectRoot: props.projectRoot,
      evidencePath: evidencePath(),
      processedDbPath: processedDbPath(),
      discoveredEvidence: discoveredEvidence(),
      discoveredDatabases: discoveredDatabases(),
    };
    props.onComplete(locations);
  };
  
  // Skip setup (use defaults)
  const handleSkip = () => {
    const locations: ProjectLocations = {
      projectRoot: props.projectRoot,
      evidencePath: props.projectRoot,
      processedDbPath: props.projectRoot,
      discoveredEvidence: [],
      discoveredDatabases: [],
    };
    props.onComplete(locations);
  };
  
  // Start discovery when opened - handled by the createEffect above
  // (removed synchronous call that was breaking reactivity)
  
  // Helper to truncate path for display
  const truncatePath = (path: string, maxLen = 50) => {
    if (path.length <= maxLen) return path;
    return '...' + path.slice(-maxLen + 3);
  };
  
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
              <div class="config-section">
                <p class="section-intro">
                  Configure the locations for evidence files and processed databases.
                  The project root is: <code>{truncatePath(props.projectRoot)}</code>
                </p>
                
                {/* Evidence Location */}
                <div class="location-group">
                  <label class="location-label">
                    <span class="label-icon flex items-center"><HiOutlineArchiveBox class="w-4 h-4" /></span>
                    <span class="label-text">Evidence Location</span>
                    <Show when={discoveredEvidence().length > 0}>
                      <span class="discovery-badge">{discoveredEvidence().length} files found</span>
                    </Show>
                  </label>
                  
                  <div class="location-input">
                    <input
                      type="text"
                      value={evidencePath()}
                      onInput={(e) => setEvidencePath(e.currentTarget.value)}
                      placeholder="Path to evidence files..."
                    />
                    <button class="btn-action-secondary" onClick={browseEvidence}>Browse</button>
                  </div>
                  
                  {/* Suggested paths dropdown */}
                  <Show when={suggestedEvidence().length > 1}>
                    <div class="suggested-paths">
                      <span class="suggested-label">Suggested:</span>
                      <For each={suggestedEvidence().slice(0, 5)}>
                        {(path) => (
                          <button
                            class="path-chip"
                            classList={{ "bg-cyan-600/15 border-cyan-600 text-cyan-400": evidencePath() === path }}
                            onClick={async () => {
                              setEvidencePath(path);
                              setScanning(true);
                              setScanMessage('Scanning...');
                              await discoverEvidence(path);
                              setScanning(false);
                            }}
                          >
                            {truncatePath(path, 30)}
                          </button>
                        )}
                      </For>
                    </div>
                  </Show>
                  
                  {/* Show discovered evidence preview */}
                  <Show when={discoveredEvidence().length > 0}>
                    <div class="discovery-preview">
                      <For each={discoveredEvidence().slice(0, 5)}>
                        {(file) => (
                          <div class="discovered-item">
                            <span class="item-icon flex items-center"><HiOutlineDocument class="w-4 h-4" /></span>
                            <span class="item-name">{file.split('/').pop()}</span>
                          </div>
                        )}
                      </For>
                      <Show when={discoveredEvidence().length > 5}>
                        <div class="discovered-more">
                          +{discoveredEvidence().length - 5} more files
                        </div>
                      </Show>
                    </div>
                  </Show>
                </div>
                
                {/* Processed Database Location */}
                <div class="location-group">
                  <label class="location-label">
                    <span class="label-icon flex items-center"><HiOutlineCircleStack class="w-4 h-4" /></span>
                    <span class="label-text">Processed Database Location</span>
                    <Show when={discoveredDatabases().length > 0}>
                      <span class="discovery-badge">{discoveredDatabases().length} databases found</span>
                    </Show>
                  </label>
                  
                  <div class="location-input">
                    <input
                      type="text"
                      value={processedDbPath()}
                      onInput={(e) => setProcessedDbPath(e.currentTarget.value)}
                      placeholder="Path to processed databases..."
                    />
                    <button class="btn-action-secondary" onClick={browseProcessed}>Browse</button>
                  </div>
                  
                  {/* Suggested paths dropdown */}
                  <Show when={suggestedProcessed().length > 1}>
                    <div class="suggested-paths">
                      <span class="suggested-label">Suggested:</span>
                      <For each={suggestedProcessed().slice(0, 5)}>
                        {(path) => (
                          <button
                            class="path-chip"
                            classList={{ "bg-cyan-600/15 border-cyan-600 text-cyan-400": processedDbPath() === path }}
                            onClick={async () => {
                              setProcessedDbPath(path);
                              setScanning(true);
                              setScanMessage('Scanning...');
                              await discoverDatabases(path);
                              setScanning(false);
                            }}
                          >
                            {truncatePath(path, 30)}
                          </button>
                        )}
                      </For>
                    </div>
                  </Show>
                  
                  {/* Show discovered databases preview */}
                  <Show when={discoveredDatabases().length > 0}>
                    <div class="discovery-preview">
                      <For each={discoveredDatabases().slice(0, 5)}>
                        {(db) => (
                          <div class="discovered-item">
                            <span class="item-icon flex items-center">
                              {db.db_type === 'MagnetAxiom' 
                                ? <HiOutlineCpuChip class="w-4 h-4" /> 
                                : db.db_type === 'CellebritePA' 
                                  ? <HiOutlineDevicePhoneMobile class="w-4 h-4" /> 
                                  : <HiOutlineFolder class="w-4 h-4" />
                              }
                            </span>
                            <span class="item-name">{db.name || db.path.split('/').pop()}</span>
                            <span class="item-type">{db.db_type}</span>
                          </div>
                        )}
                      </For>
                      <Show when={discoveredDatabases().length > 5}>
                        <div class="discovered-more">
                          +{discoveredDatabases().length - 5} more databases
                        </div>
                      </Show>
                    </div>
                  </Show>
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
