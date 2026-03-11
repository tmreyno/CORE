// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { getBasename } from '../../utils';
import {
  HiOutlineArchiveBox,
  HiOutlineCircleStack,
  HiOutlineClipboardDocumentList,
  HiOutlineFingerPrint,
  HiOutlineBookmark,
} from '../icons';
import { ProfileSelector } from '../project/ProfileSelector';

interface ConfigureLocationsStepProps {
  // Project Name
  projectName: () => string;
  setProjectName: (value: string) => void;
  
  // Owner/Examiner Name
  ownerName?: () => string;
  setOwnerName?: (value: string) => void;
  
  // Case Identification
  caseNumber?: () => string;
  setCaseNumber?: (value: string) => void;
  caseName?: () => string;
  setCaseName?: (value: string) => void;
  
  // Evidence
  evidencePath: () => string;
  setEvidencePath: (value: string) => void;
  evidenceCount: () => number;
  evidenceChips: () => string[];
  suggestedEvidence: () => string[];
  browseEvidence: () => void;
  discoverEvidence: (path: string) => Promise<string[]>;
  
  // Processed Databases
  processedDbPath: () => string;
  setProcessedDbPath: (value: string) => void;
  databaseCount: () => number;
  processedChips: () => string[];
  suggestedProcessed: () => string[];
  browseProcessed: () => void;
  discoverDatabases: (path: string) => Promise<any[]>;
  
  // Case Documents
  caseDocumentsPath: () => string;
  setCaseDocumentsPath: (value: string) => void;
  discoveredCaseDocCount: () => number;
  caseDocsChips: () => string[];
  suggestedCaseDocs: () => string[];
  browseCaseDocs: () => void;
  setDiscoveredCaseDocCount: (count: number) => void;
  
  // Options
  loadStoredHashes: () => boolean;
  setLoadStoredHashes: (value: boolean) => void;
  
  // Profile
  onProfileChange?: (profileId: string) => void;
}

export const ConfigureLocationsStep: Component<ConfigureLocationsStepProps> = (props) => {
  return (
    <div class="config-section compact">
      {/* Project Name + Owner/Examiner */}
      <div class="location-group-compact">
        <div class="location-header">
          <HiOutlineClipboardDocumentList class="w-4 h-4 text-accent" />
          <span class="location-title">Project Info</span>
        </div>
        <div class="grid grid-cols-2 gap-2">
          <div class="location-input-compact">
            <input
              type="text"
              value={props.projectName()}
              onInput={(e) => props.setProjectName(e.currentTarget.value)}
              placeholder="Project name..."
            />
          </div>
          <Show when={props.ownerName && props.setOwnerName}>
            <div class="location-input-compact">
              <input
                type="text"
                value={props.ownerName?.() || ''}
                onInput={(e) => props.setOwnerName?.(e.currentTarget.value)}
                placeholder="Owner / Examiner name..."
              />
            </div>
          </Show>
        </div>
      </div>

      {/* Case Identification (optional) */}
      <Show when={props.caseNumber && props.setCaseNumber}>
        <div class="location-group-compact">
          <div class="location-header">
            <HiOutlineClipboardDocumentList class="w-4 h-4 text-accent" />
            <span class="location-title">Case Identification</span>
            <span class="text-2xs text-txt-muted">(optional)</span>
          </div>
          <div class="flex gap-2">
            <div class="location-input-compact flex-[0_0_40%]">
              <input
                type="text"
                value={props.caseNumber?.() || ''}
                onInput={(e) => props.setCaseNumber?.(e.currentTarget.value)}
                placeholder="Case number..."
              />
            </div>
            <div class="location-input-compact flex-1">
              <input
                type="text"
                value={props.caseName?.() || ''}
                onInput={(e) => props.setCaseName?.(e.currentTarget.value)}
                placeholder="Case name / title..."
              />
            </div>
          </div>
        </div>
      </Show>
      
      {/* Workspace Profile */}
      <div class="location-group-compact">
        <div class="location-header">
          <HiOutlineBookmark class="w-4 h-4 text-accent" />
          <span class="location-title">Workspace Profile</span>
        </div>
        <div class="mt-1">
          <ProfileSelector
            onProfileChange={props.onProfileChange}
          />
        </div>
      </div>
      
      {/* Evidence Location - Compact */}
      <div class="location-group-compact">
        <div class="location-header">
          <HiOutlineArchiveBox class="w-4 h-4 text-accent" />
          <span class="location-title">Evidence</span>
          <Show when={props.evidenceCount() > 0}>
            <span class="badge-sm success">{props.evidenceCount()} files</span>
          </Show>
        </div>
        <div class="location-input-compact">
          <input
            type="text"
            value={props.evidencePath()}
            onInput={(e) => props.setEvidencePath(e.currentTarget.value)}
            placeholder="Evidence path..."
          />
          <button class="btn-browse" onClick={props.browseEvidence}>...</button>
        </div>
        <Show when={props.suggestedEvidence().length > 1}>
          <div class="path-chips">
            <For each={props.evidenceChips()}>
              {(path) => (
                <button
                  class="chip"
                  classList={{ active: props.evidencePath() === path }}
                  onClick={async () => {
                    props.setEvidencePath(path);
                    await props.discoverEvidence(path);
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
          <Show when={props.databaseCount() > 0}>
            <span class="badge-sm success">{props.databaseCount()} DBs</span>
          </Show>
        </div>
        <div class="location-input-compact">
          <input
            type="text"
            value={props.processedDbPath()}
            onInput={(e) => props.setProcessedDbPath(e.currentTarget.value)}
            placeholder="Processed DB path..."
          />
          <button class="btn-browse" onClick={props.browseProcessed}>...</button>
        </div>
        <Show when={props.suggestedProcessed().length > 1}>
          <div class="path-chips">
            <For each={props.processedChips()}>
              {(path) => (
                <button
                  class="chip"
                  classList={{ active: props.processedDbPath() === path }}
                  onClick={async () => {
                    props.setProcessedDbPath(path);
                    await props.discoverDatabases(path);
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
          <Show when={props.discoveredCaseDocCount() > 0}>
            <span class="badge-sm success">{props.discoveredCaseDocCount()} docs</span>
          </Show>
        </div>
        <div class="location-input-compact">
          <input
            type="text"
            value={props.caseDocumentsPath()}
            onInput={(e) => props.setCaseDocumentsPath(e.currentTarget.value)}
            placeholder="Case documents path..."
          />
          <button class="btn-browse" onClick={props.browseCaseDocs}>...</button>
        </div>
        <Show when={props.suggestedCaseDocs().length > 1}>
          <div class="path-chips">
            <For each={props.caseDocsChips()}>
              {(path) => (
                <button
                  class="chip"
                  classList={{ active: props.caseDocumentsPath() === path }}
                  onClick={async () => {
                    props.setCaseDocumentsPath(path);
                    try {
                      const docs = await invoke<{ length: number }[]>('discover_case_documents', { evidencePath: path });
                      props.setDiscoveredCaseDocCount(Array.isArray(docs) ? docs.length : 0);
                    } catch {
                      props.setDiscoveredCaseDocCount(0);
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
            checked={props.loadStoredHashes()}
            onChange={(e) => props.setLoadStoredHashes(e.currentTarget.checked)}
          />
          <HiOutlineFingerPrint class="w-4 h-4 text-amber-400" />
          <span>Load stored hashes from containers</span>
        </label>
      </div>
    </div>
  );
};
