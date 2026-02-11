// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from 'solid-js';
import { HiOutlineFingerPrint, HiOutlineCheckCircle } from '../icons';
import type { StoredHash } from '../../types';

interface HashLoadingProgress {
  current: number;
  total: number;
  currentFile: string;
  hashCount: number;
}

interface LoadHashesStepProps {
  hashLoadingProgress: () => HashLoadingProgress;
  hashProgressPercent: () => number;
  loadedStoredHashes: () => Map<string, StoredHash[]>;
}

export const LoadHashesStep: Component<LoadHashesStepProps> = (props) => {
  return (
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
            style={{ width: `${props.hashProgressPercent()}%` }}
          />
        </div>
        <div class="hash-progress-info">
          <span class="hash-progress-file">{props.hashLoadingProgress().currentFile}</span>
          <span class="hash-progress-count">
            {props.hashLoadingProgress().current} / {props.hashLoadingProgress().total}
          </span>
        </div>
      </div>
      
      {/* Hash count summary */}
      <Show when={props.hashLoadingProgress().hashCount > 0}>
        <div class="hash-loaded-summary">
          <HiOutlineCheckCircle class="w-4 h-4 text-success" />
          <span>{props.hashLoadingProgress().hashCount} hash(es) found from {props.loadedStoredHashes().size} container(s)</span>
        </div>
      </Show>
    </div>
  );
};
