// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, type Accessor } from "solid-js";
import {
  HiOutlineInformationCircle,
  HiOutlineFolderOpen,
  HiOutlinePlay,
  HiOutlineCheckCircle,
  HiOutlineXCircle,
} from "../icons";
import type { DetailedArchiveError } from "../../api/archiveCreate";

interface TestTabProps {
  testArchivePath: Accessor<string>;
  setTestArchivePath: (value: string) => void;
  testPassword: Accessor<string>;
  setTestPassword: (value: string) => void;
  testResult: Accessor<boolean | null>;
  testInProgress: Accessor<boolean>;
  lastError: Accessor<DetailedArchiveError | null>;
  onTest: () => void;
  onSelectArchive: () => void;
}

export const TestTab: Component<TestTabProps> = (props) => {
  return (
    <div class="col gap-4">
      <div class="info-card">
        <HiOutlineInformationCircle class="w-5 h-5 text-info" />
        <div>
          <div class="font-medium text-txt">Test Archive Integrity</div>
          <div class="text-sm text-txt-secondary">
            Verify archive integrity without extracting files. Fast and non-destructive.
          </div>
        </div>
      </div>

      <div class="form-group">
        <label class="label">Archive Path</label>
        <div class="flex gap-2">
          <input
            class="input flex-1"
            placeholder="/path/to/archive.7z"
            value={props.testArchivePath()}
            onInput={(e) => props.setTestArchivePath(e.currentTarget.value)}
          />
          <button class="btn-sm" onClick={props.onSelectArchive}>
            <HiOutlineFolderOpen class="w-4 h-4" />
          </button>
        </div>
      </div>

      <div class="form-group">
        <label class="label">Password (optional)</label>
        <input
          type="password"
          class="input"
          placeholder="Enter password if encrypted"
          value={props.testPassword()}
          onInput={(e) => props.setTestPassword(e.currentTarget.value)}
        />
      </div>

      <button
        class="btn-sm-primary"
        onClick={props.onTest}
        disabled={!props.testArchivePath() || props.testInProgress()}
      >
        <HiOutlinePlay class="w-4 h-4" />
        {props.testInProgress() ? "Testing..." : "Test Archive"}
      </button>

      <Show when={props.testResult() !== null}>
        <div
          class={`card ${
            props.testResult() ? "bg-success/10 border-success" : "bg-error/10 border-error"
          }`}
        >
          <div class="flex items-start gap-3">
            {props.testResult() ? (
              <HiOutlineCheckCircle class="w-6 h-6 text-success flex-shrink-0 mt-0.5" />
            ) : (
              <HiOutlineXCircle class="w-6 h-6 text-error flex-shrink-0 mt-0.5" />
            )}
            <div>
              <div class="font-semibold text-txt">
                {props.testResult() ? "Test Passed" : "Test Failed"}
              </div>
              <div class="text-sm text-txt-secondary mt-1">
                {props.testResult()
                  ? "Archive integrity verified successfully"
                  : "Archive integrity check failed - try Repair or Validate"}
              </div>
            </div>
          </div>
        </div>
      </Show>

      <Show when={props.lastError()}>
        <div class="card bg-warning/10 border-warning">
          <div class="font-medium text-warning mb-2">Detailed Error Information</div>
          <div class="text-sm space-y-1">
            <div><strong>Code:</strong> {props.lastError()?.code}</div>
            <div><strong>Message:</strong> {props.lastError()?.message}</div>
            <div><strong>Context:</strong> {props.lastError()?.fileContext}</div>
            <div><strong>Position:</strong> {props.lastError()?.position}</div>
            <div class="pt-2 border-t border-warning/20">
              <strong>Suggestion:</strong> {props.lastError()?.suggestion}
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
