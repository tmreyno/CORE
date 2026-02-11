// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, type Accessor } from "solid-js";
import {
  HiOutlineArchiveBox,
  HiOutlineFolderOpen,
  HiOutlinePlay,
  HiOutlineCheckCircle,
} from "../icons";

interface ExtractTabProps {
  extractFirstVolume: Accessor<string>;
  setExtractFirstVolume: (value: string) => void;
  extractOutputDir: Accessor<string>;
  setExtractOutputDir: (value: string) => void;
  extractPassword: Accessor<string>;
  setExtractPassword: (value: string) => void;
  extractProgress: Accessor<number>;
  extractStatus: Accessor<string>;
  extractInProgress: Accessor<boolean>;
  extractResult: Accessor<string>;
  onExtract: () => void;
  onSelectFirstVolume: () => void;
  onSelectOutput: () => void;
}

export const ExtractTab: Component<ExtractTabProps> = (props) => {
  return (
    <div class="col gap-4">
      <div class="info-card">
        <HiOutlineArchiveBox class="w-5 h-5 text-accent" />
        <div>
          <div class="font-medium text-txt">Extract Split Archive</div>
          <div class="text-sm text-txt-secondary">
            Extract multi-volume archives (.001, .002, etc.) with automatic reassembly.
          </div>
        </div>
      </div>

      <div class="form-group">
        <label class="label">First Volume (.001)</label>
        <div class="flex gap-2">
          <input
            class="input flex-1"
            placeholder="/path/to/archive.7z.001"
            value={props.extractFirstVolume()}
            onInput={(e) => props.setExtractFirstVolume(e.currentTarget.value)}
          />
          <button class="btn-sm" onClick={props.onSelectFirstVolume}>
            <HiOutlineFolderOpen class="w-4 h-4" />
          </button>
        </div>
      </div>

      <div class="form-group">
        <label class="label">Output Directory</label>
        <div class="flex gap-2">
          <input
            class="input flex-1"
            placeholder="/path/to/output/"
            value={props.extractOutputDir()}
            onInput={(e) => props.setExtractOutputDir(e.currentTarget.value)}
          />
          <button class="btn-sm" onClick={props.onSelectOutput}>
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
          value={props.extractPassword()}
          onInput={(e) => props.setExtractPassword(e.currentTarget.value)}
        />
      </div>

      <button
        class="btn-sm-primary"
        onClick={props.onExtract}
        disabled={!props.extractFirstVolume() || !props.extractOutputDir() || props.extractInProgress()}
      >
        <HiOutlinePlay class="w-4 h-4" />
        {props.extractInProgress() ? "Extracting..." : "Extract Archive"}
      </button>

      <Show when={props.extractInProgress()}>
        <div class="card">
          <div class="text-sm text-txt-secondary mb-2">{props.extractStatus()}</div>
          <div class="w-full bg-bg-secondary rounded-full h-2">
            <div
              class="bg-accent h-2 rounded-full transition-all"
              style={{ width: `${props.extractProgress()}%` }}
            />
          </div>
          <div class="text-sm text-txt-muted mt-1 text-center">
            {props.extractProgress().toFixed(1)}%
          </div>
        </div>
      </Show>

      <Show when={props.extractResult()}>
        <div class="card bg-success/10 border-success">
          <div class="flex items-start gap-3">
            <HiOutlineCheckCircle class="w-6 h-6 text-success flex-shrink-0 mt-0.5" />
            <div>
              <div class="font-semibold text-txt">Extraction Complete</div>
              <div class="text-sm text-txt-secondary mt-1">
                Files extracted to: {props.extractResult()}
              </div>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
