// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, type Accessor } from "solid-js";
import {
  HiOutlineDocumentMagnifyingGlass,
  HiOutlineFolderOpen,
  HiOutlinePlay,
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
} from "../icons";
import type { ArchiveValidationResult, DetailedArchiveError } from "../../api/archiveCreate";

interface ValidateTabProps {
  validateArchivePath: Accessor<string>;
  setValidateArchivePath: (value: string) => void;
  validateResult: Accessor<ArchiveValidationResult | null>;
  validateInProgress: Accessor<boolean>;
  lastError: Accessor<DetailedArchiveError | null>;
  onValidate: () => void;
  onSelectArchive: () => void;
}

export const ValidateTab: Component<ValidateTabProps> = (props) => {
  return (
    <div class="col gap-4">
      <div class="info-card">
        <HiOutlineDocumentMagnifyingGlass class="w-5 h-5 text-info" />
        <div>
          <div class="font-medium text-txt">Validate Archive</div>
          <div class="text-sm text-txt-secondary">
            Perform thorough validation with detailed error context and suggestions.
          </div>
        </div>
      </div>

      <div class="form-group">
        <label class="label">Archive Path</label>
        <div class="flex gap-2">
          <input
            class="input flex-1"
            placeholder="/path/to/archive.7z"
            value={props.validateArchivePath()}
            onInput={(e) => props.setValidateArchivePath(e.currentTarget.value)}
          />
          <button class="btn-sm" onClick={props.onSelectArchive}>
            <HiOutlineFolderOpen class="w-4 h-4" />
          </button>
        </div>
      </div>

      <button
        class="btn-sm-primary"
        onClick={props.onValidate}
        disabled={!props.validateArchivePath() || props.validateInProgress()}
      >
        <HiOutlinePlay class="w-4 h-4" />
        {props.validateInProgress() ? "Validating..." : "Validate Archive"}
      </button>

      <Show when={props.validateResult()}>
        <div
          class={`card ${
            props.validateResult()!.isValid
              ? "bg-success/10 border-success"
              : "bg-warning/10 border-warning"
          }`}
        >
          <div class="flex items-start gap-3">
            {props.validateResult()!.isValid ? (
              <HiOutlineCheckCircle class="w-6 h-6 text-success flex-shrink-0 mt-0.5" />
            ) : (
              <HiOutlineExclamationTriangle class="w-6 h-6 text-warning flex-shrink-0 mt-0.5" />
            )}
            <div class="flex-1">
              <div class="font-semibold text-txt">
                {props.validateResult()!.isValid ? "Validation Passed" : "Validation Issues"}
              </div>
              <Show when={props.validateResult()!.errorMessage}>
                <div class="text-sm text-txt-secondary mt-1">
                  {props.validateResult()!.errorMessage}
                </div>
              </Show>
              <Show when={props.validateResult()!.fileContext}>
                <div class="text-sm text-txt-muted mt-1">
                  Context: {props.validateResult()!.fileContext}
                </div>
              </Show>
              <Show when={props.validateResult()!.suggestion}>
                <div class="text-sm text-accent mt-2 pt-2 border-t border-warning/20">
                  💡 {props.validateResult()!.suggestion}
                </div>
              </Show>
            </div>
          </div>
        </div>
      </Show>

      <Show when={props.lastError()}>
        <div class="card bg-error/10 border-error">
          <div class="font-medium text-error mb-2">Detailed Error Information</div>
          <div class="text-sm space-y-1">
            <div><strong>Code:</strong> {props.lastError()?.code}</div>
            <div><strong>Message:</strong> {props.lastError()?.message}</div>
            <div><strong>Context:</strong> {props.lastError()?.fileContext}</div>
            <div><strong>Position:</strong> {props.lastError()?.position}</div>
            <div class="pt-2 border-t border-error/20">
              <strong>Suggestion:</strong> {props.lastError()?.suggestion}
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
