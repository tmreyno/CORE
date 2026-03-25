// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * StartSessionDialog — Simple modal to create a new acquisition session.
 * Collects: case number, examiner name, output folder.
 */

import { Component, Show, createSignal } from "solid-js";
import {
  HiOutlineFolderOpen,
  HiOutlineArchiveBoxArrowDown,
} from "../icons";

interface StartSessionDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (opts: {
    caseNumber: string;
    examiner: string;
    outputFolder: string;
    caseName?: string;
  }) => void | Promise<void>;
  defaultExaminer?: string;
}

const StartSessionDialog: Component<StartSessionDialogProps> = (props) => {
  const [caseNumber, setCaseNumber] = createSignal("");
  const [examiner, setExaminer] = createSignal(props.defaultExaminer ?? "");
  const [caseName, setCaseName] = createSignal("");
  const [outputFolder, setOutputFolder] = createSignal("");
  const [creating, setCreating] = createSignal(false);

  const canCreate = () =>
    caseNumber().trim().length > 0 &&
    examiner().trim().length > 0 &&
    outputFolder().trim().length > 0;

  async function handleBrowse() {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({ directory: true, title: "Select Output Folder" });
    if (selected && typeof selected === "string") {
      setOutputFolder(selected);
    }
  }

  async function handleCreate() {
    if (!canCreate() || creating()) return;
    setCreating(true);
    try {
      await props.onCreate({
        caseNumber: caseNumber().trim(),
        examiner: examiner().trim(),
        outputFolder: outputFolder().trim(),
        caseName: caseName().trim() || undefined,
      });
      // Reset for next time only on success
      setCaseNumber("");
      setCaseName("");
      setOutputFolder("");
    } finally {
      setCreating(false);
    }
  }

  function handleClose() {
    if (!creating()) props.onClose();
  }

  return (
    <Show when={props.isOpen}>
      <div class="modal-overlay" onClick={handleClose}>
        <div
          class="modal-content w-[480px]"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div class="modal-header">
            <div class="flex items-center gap-2">
              <HiOutlineArchiveBoxArrowDown class="w-icon-base h-icon-base text-accent" />
              <h2 class="text-lg font-medium text-txt">New Acquisition Session</h2>
            </div>
            <button
              class="icon-btn-sm"
              onClick={handleClose}
              title="Close"
            >
              ✕
            </button>
          </div>

          {/* Body */}
          <div class="modal-body space-y-4">
            <p class="text-sm text-txt-muted">
              Create a session to track acquisitions, evidence, and chain of custody.
              All data is saved to a single portable JSON file.
            </p>

            {/* Case Number */}
            <div class="form-group">
              <label class="label">
                Case Number <span class="text-error">*</span>
              </label>
              <input
                class="input"
                type="text"
                placeholder="e.g. 2025-001"
                value={caseNumber()}
                onInput={(e) => setCaseNumber(e.currentTarget.value)}
                autofocus
              />
            </div>

            {/* Case Name (optional) */}
            <div class="form-group">
              <label class="label">Case Name</label>
              <input
                class="input"
                type="text"
                placeholder="Optional descriptive name"
                value={caseName()}
                onInput={(e) => setCaseName(e.currentTarget.value)}
              />
            </div>

            {/* Examiner */}
            <div class="form-group">
              <label class="label">
                Examiner <span class="text-error">*</span>
              </label>
              <input
                class="input"
                type="text"
                placeholder="Full name"
                value={examiner()}
                onInput={(e) => setExaminer(e.currentTarget.value)}
              />
            </div>

            {/* Output Folder */}
            <div class="form-group">
              <label class="label">
                Output Folder <span class="text-error">*</span>
              </label>
              <div class="flex items-center gap-2">
                <input
                  class="input-inline flex-1"
                  type="text"
                  placeholder="Select folder..."
                  value={outputFolder()}
                  onInput={(e) => setOutputFolder(e.currentTarget.value)}
                  readOnly
                />
                <button class="btn-sm" onClick={handleBrowse}>
                  <HiOutlineFolderOpen class="w-icon-sm h-icon-sm" />
                  Browse
                </button>
              </div>
              <p class="text-2xs text-txt-muted mt-1">
                The session file and all acquisitions will be saved here.
              </p>
            </div>
          </div>

          {/* Footer */}
          <div class="modal-footer justify-end">
            <button class="btn btn-secondary" onClick={handleClose}>
              Cancel
            </button>
            <button
              class="btn btn-primary"
              onClick={handleCreate}
              disabled={!canCreate() || creating()}
            >
              Create Session
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default StartSessionDialog;
