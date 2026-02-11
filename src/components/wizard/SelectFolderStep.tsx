// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from 'solid-js';
import { HiOutlineFolder } from '../icons';

interface SelectFolderStepProps {
  error: () => string | null;
  onBrowse: () => void;
}

export const SelectFolderStep: Component<SelectFolderStepProps> = (props) => {
  return (
    <div class="folder-select-state">
      <div class="folder-icon-container">
        <HiOutlineFolder class="w-16 h-16 text-accent" />
      </div>
      <h3 class="text-lg font-medium text-txt mb-2">Select Project Folder</h3>
      <p class="text-txt-muted text-sm mb-6 text-center max-w-md">
        Choose a folder to create your new forensic project. This folder will contain 
        your evidence files, processed databases, and case documents.
      </p>
      <button class="btn-sm-primary" onClick={props.onBrowse}>
        <HiOutlineFolder class="w-4 h-4" />
        Browse for Folder
      </button>
      <Show when={props.error()}>
        <p class="error-text mt-4">{props.error()}</p>
      </Show>
    </div>
  );
};
