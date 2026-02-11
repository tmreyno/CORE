// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ImportProfileDialog - Dialog for importing profile from JSON
 */

import { Component, Show } from "solid-js";

interface ImportProfileDialogProps {
  isOpen: boolean;
  importJson: string;
  loading?: boolean;
  onJsonChange: (json: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
}

export const ImportProfileDialog: Component<ImportProfileDialogProps> = (props) => {
  return (
    <Show when={props.isOpen}>
      <div class="modal-overlay">
        <div class="modal-content max-w-lg w-full">
          <div class="modal-header">
            <h3 class="text-lg font-semibold text-txt">Import Profile</h3>
          </div>

          <div class="modal-body space-y-4">
            <div class="form-group">
              <label class="label">Profile JSON</label>
              <textarea
                value={props.importJson}
                onInput={(e) => props.onJsonChange(e.currentTarget.value)}
                placeholder="Paste exported profile JSON here..."
                rows={10}
                class="textarea font-mono text-sm"
              />
            </div>
          </div>

          <div class="modal-footer justify-end">
            <button
              onClick={props.onCancel}
              class="btn-sm"
            >
              Cancel
            </button>
            <button
              onClick={props.onSubmit}
              disabled={!props.importJson.trim() || props.loading}
              class="btn-sm-primary"
            >
              Import
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};
