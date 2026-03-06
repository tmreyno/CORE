// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { open } from "@tauri-apps/plugin-dialog";
import { HiOutlineInformationCircle } from "../../icons";
import type { Accessor } from "solid-js";

interface ExtractTabProps {
  firstVolume: Accessor<string>;
  setFirstVolume: (path: string) => void;
  outputDir: Accessor<string>;
  setOutputDir: (path: string) => void;
}

export function ExtractTab(props: ExtractTabProps) {
  return (
    <div class="space-y-3">
      <div class="info-card">
        <HiOutlineInformationCircle class="w-5 h-5 text-info" />
        <div>
          <div class="font-medium text-txt">Extract Split Archive</div>
          <div class="text-xs text-txt-muted mt-1">
            Extract multi-volume archives (*.7z.001, *.7z.002, etc.)
          </div>
        </div>
      </div>
      <div class="space-y-2">
        <label class="label">First Volume</label>
        <div class="flex gap-2">
          <input
            type="text"
            class="input-inline"
            value={props.firstVolume()}
            onInput={(e) => props.setFirstVolume(e.currentTarget.value)}
            placeholder="Select first volume (.001)..."
          />
          <button class="btn-sm" onClick={async () => {
            const selected = await open({ directory: false, multiple: false });
            if (selected) props.setFirstVolume(selected as string);
          }}>
            Browse
          </button>
        </div>
      </div>
      <div class="space-y-2">
        <label class="label">Output Directory</label>
        <div class="flex gap-2">
          <input
            type="text"
            class="input-inline"
            value={props.outputDir()}
            onInput={(e) => props.setOutputDir(e.currentTarget.value)}
            placeholder="Extract to..."
          />
          <button class="btn-sm" onClick={async () => {
            const selected = await open({ directory: true, multiple: false });
            if (selected) props.setOutputDir(selected as string);
          }}>
            Browse
          </button>
        </div>
      </div>
    </div>
  );
}
