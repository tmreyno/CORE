// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, createSignal } from 'solid-js';
import { HiOutlineFolder, HiOutlineDocumentDuplicate, HiOutlineArrowLeft } from '../icons';

interface SelectFolderStepProps {
  error: () => string | null;
  /** Browse for an existing project folder */
  onBrowse: () => void;
  /** Create template folders: called with (projectName, examiner) — handler opens location picker */
  onCreateFromTemplate: (projectName: string, examiner: string) => void;
}

export const SelectFolderStep: Component<SelectFolderStepProps> = (props) => {
  // Sub-view: 'choose' (initial) or 'template' (enter name then pick location)
  const [mode, setMode] = createSignal<'choose' | 'template'>('choose');
  const [templateName, setTemplateName] = createSignal('');
  const [templateExaminer, setTemplateExaminer] = createSignal('');

  const handleCreate = () => {
    const name = templateName().trim();
    if (!name) return;
    props.onCreateFromTemplate(name, templateExaminer().trim());
  };

  return (
    <div class="folder-select-state">
      <Show when={mode() === 'choose'}>
        <h3 class="text-lg font-medium text-txt mb-1">New Project</h3>
        <p class="text-txt-muted text-sm mb-5 text-center max-w-md">
          Open an existing case folder, or create a new case folder structure.
        </p>

        <div class="flex flex-col gap-3 w-full max-w-sm">
          {/* Option 1: Open existing folder */}
          <button
            class="flex items-center gap-3 w-full px-4 py-3 rounded-lg border border-border bg-bg-secondary hover:bg-bg-hover transition-colors text-left cursor-pointer"
            onClick={props.onBrowse}
          >
            <HiOutlineFolder class="w-6 h-6 text-accent shrink-0" />
            <div>
              <div class="text-sm font-medium text-txt">Open Existing Folder</div>
              <div class="text-xs text-txt-muted">Select a folder that already has case files</div>
            </div>
          </button>

          {/* Option 2: Create from template */}
          <button
            class="flex items-center gap-3 w-full px-4 py-3 rounded-lg border border-border bg-bg-secondary hover:bg-bg-hover transition-colors text-left cursor-pointer"
            onClick={() => setMode('template')}
          >
            <HiOutlineDocumentDuplicate class="w-6 h-6 text-accent shrink-0" />
            <div>
              <div class="text-sm font-medium text-txt">Create Case Folder Structure</div>
              <div class="text-xs text-txt-muted">
                Enter a case name, then pick where to create the folders
              </div>
            </div>
          </button>
        </div>
      </Show>

      <Show when={mode() === 'template'}>
        <button
          class="self-start mb-3 flex items-center gap-1 text-xs text-txt-muted hover:text-txt cursor-pointer"
          onClick={() => setMode('choose')}
        >
          <HiOutlineArrowLeft class="w-3.5 h-3.5" />
          Back
        </button>

        <HiOutlineDocumentDuplicate class="w-10 h-10 text-accent mb-2" />
        <h3 class="text-lg font-medium text-txt mb-1">Create Case Folder Structure</h3>
        <p class="text-txt-muted text-xs mb-4 text-center max-w-sm">
          A folder named after the project will be created at your chosen location,
          containing Evidence, Exports, Case Documents, and more.
        </p>

        <div class="flex flex-col gap-3 w-full max-w-sm">
          <div class="flex flex-col gap-1">
            <label class="text-xs font-medium text-txt-secondary">Project / Case Name *</label>
            <input
              type="text"
              class="input-sm"
              placeholder="e.g. 10115-0900"
              value={templateName()}
              onInput={(e) => setTemplateName(e.currentTarget.value)}
              autofocus
            />
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-xs font-medium text-txt-secondary">Examiner (optional)</label>
            <input
              type="text"
              class="input-sm"
              placeholder="e.g. SA Reynolds"
              value={templateExaminer()}
              onInput={(e) => setTemplateExaminer(e.currentTarget.value)}
            />
          </div>

          <button
            class="btn-sm-primary mt-1"
            disabled={!templateName().trim()}
            onClick={handleCreate}
          >
            <HiOutlineFolder class="w-4 h-4" />
            Select Location &amp; Create
          </button>
        </div>
      </Show>

      <Show when={props.error()}>
        <p class="error-text mt-4">{props.error()}</p>
      </Show>
    </div>
  );
};
