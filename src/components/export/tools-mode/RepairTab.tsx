// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { open, save } from "@tauri-apps/plugin-dialog";
import { HiOutlineInformationCircle } from "../../icons";
import type { Accessor } from "solid-js";

interface RepairTabProps {
  corruptedPath: Accessor<string>;
  setCorruptedPath: (path: string) => void;
  outputPath: Accessor<string>;
  setOutputPath: (path: string) => void;
}

export function RepairTab(props: RepairTabProps) {
  return (
    <div class="space-y-3">
      <div class="info-card">
        <HiOutlineInformationCircle class="w-5 h-5 text-info" />
        <div>
          <div class="font-medium text-txt">Repair Corrupted Archive</div>
          <div class="text-xs text-txt-muted mt-1">
            Attempt to recover data from damaged archives.
          </div>
        </div>
      </div>
      <div class="space-y-2">
        <label class="label">Corrupted Archive</label>
        <div class="flex gap-2">
          <input
            type="text"
            class="input-inline"
            value={props.corruptedPath()}
            onInput={(e) => props.setCorruptedPath(e.currentTarget.value)}
            placeholder="Select corrupted archive..."
          />
          <button class="btn-sm" onClick={async () => {
            const selected = await open({ directory: false, multiple: false, filters: [{ name: "Archives", extensions: ["7z"] }] });
            if (selected) props.setCorruptedPath(selected as string);
          }}>
            Browse
          </button>
        </div>
      </div>
      <div class="space-y-2">
        <label class="label">Output Archive</label>
        <div class="flex gap-2">
          <input
            type="text"
            class="input-inline"
            value={props.outputPath()}
            onInput={(e) => props.setOutputPath(e.currentTarget.value)}
            placeholder="Output path for repaired archive..."
          />
          <button class="btn-sm" onClick={async () => {
            const selected = await save({ filters: [{ name: "7z Archive", extensions: ["7z"] }] });
            if (selected) props.setOutputPath(selected as string);
          }}>
            Browse
          </button>
        </div>
      </div>
    </div>
  );
}
