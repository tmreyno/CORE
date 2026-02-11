// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, type Accessor } from "solid-js";
import {
  HiOutlineWrench,
  HiOutlineFolderOpen,
  HiOutlineDocumentText,
  HiOutlinePlay,
  HiOutlineCheckCircle,
} from "../icons";

interface RepairTabProps {
  repairCorruptedPath: Accessor<string>;
  setRepairCorruptedPath: (value: string) => void;
  repairOutputPath: Accessor<string>;
  setRepairOutputPath: (value: string) => void;
  repairProgress: Accessor<number>;
  repairStatus: Accessor<string>;
  repairInProgress: Accessor<boolean>;
  repairResult: Accessor<string>;
  onRepair: () => void;
  onSelectInput: () => void;
  onSelectOutput: () => void;
}

export const RepairTab: Component<RepairTabProps> = (props) => {
  return (
    <div class="col gap-4">
      <div class="info-card">
        <HiOutlineWrench class="w-5 h-5 text-warning" />
        <div>
          <div class="font-medium text-txt">Repair Corrupted Archive</div>
          <div class="text-sm text-txt-secondary">
            Attempt to recover data from damaged or incomplete archives.
          </div>
        </div>
      </div>

      <div class="form-group">
        <label class="label">Corrupted Archive</label>
        <div class="flex gap-2">
          <input
            class="input flex-1"
            placeholder="/path/to/corrupted.7z"
            value={props.repairCorruptedPath()}
            onInput={(e) => props.setRepairCorruptedPath(e.currentTarget.value)}
          />
          <button class="btn-sm" onClick={props.onSelectInput}>
            <HiOutlineFolderOpen class="w-4 h-4" />
          </button>
        </div>
      </div>

      <div class="form-group">
        <label class="label">Output Path</label>
        <div class="flex gap-2">
          <input
            class="input flex-1"
            placeholder="/path/to/repaired.7z"
            value={props.repairOutputPath()}
            onInput={(e) => props.setRepairOutputPath(e.currentTarget.value)}
          />
          <button class="btn-sm" onClick={props.onSelectOutput}>
            <HiOutlineDocumentText class="w-4 h-4" />
          </button>
        </div>
      </div>

      <button
        class="btn-sm-primary"
        onClick={props.onRepair}
        disabled={!props.repairCorruptedPath() || !props.repairOutputPath() || props.repairInProgress()}
      >
        <HiOutlinePlay class="w-4 h-4" />
        {props.repairInProgress() ? "Repairing..." : "Repair Archive"}
      </button>

      <Show when={props.repairInProgress()}>
        <div class="card">
          <div class="text-sm text-txt-secondary mb-2">{props.repairStatus()}</div>
          <div class="w-full bg-bg-secondary rounded-full h-2">
            <div
              class="bg-accent h-2 rounded-full transition-all"
              style={{ width: `${props.repairProgress()}%` }}
            />
          </div>
          <div class="text-sm text-txt-muted mt-1 text-center">
            {props.repairProgress().toFixed(1)}%
          </div>
        </div>
      </Show>

      <Show when={props.repairResult()}>
        <div class="card bg-success/10 border-success">
          <div class="flex items-start gap-3">
            <HiOutlineCheckCircle class="w-6 h-6 text-success flex-shrink-0 mt-0.5" />
            <div>
              <div class="font-semibold text-txt">Repair Complete</div>
              <div class="text-sm text-txt-secondary mt-1">
                Repaired archive saved to: {props.repairResult()}
              </div>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
