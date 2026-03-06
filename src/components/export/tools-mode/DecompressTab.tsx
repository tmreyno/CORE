// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { open, save } from "@tauri-apps/plugin-dialog";
import { HiOutlineInformationCircle } from "../../icons";
import type { Accessor } from "solid-js";

interface DecompressTabProps {
  inputPath: Accessor<string>;
  setInputPath: (path: string) => void;
  outputPath: Accessor<string>;
  setOutputPath: (path: string) => void;
}

export function DecompressTab(props: DecompressTabProps) {
  return (
    <div class="space-y-3">
      <div class="info-card">
        <HiOutlineInformationCircle class="w-5 h-5 text-info" />
        <div>
          <div class="font-medium text-txt">LZMA/LZMA2 Decompression</div>
          <div class="text-xs text-txt-muted mt-1">
            Decompress a .lzma or .xz file back to its original format. Auto-detects algorithm from extension.
          </div>
        </div>
      </div>
      <div class="space-y-2">
        <label class="label">Compressed File</label>
        <div class="flex gap-2">
          <input
            type="text"
            class="input-inline"
            value={props.inputPath()}
            onInput={(e) => props.setInputPath(e.currentTarget.value)}
            placeholder="Select .lzma or .xz file..."
          />
          <button class="btn-sm" onClick={async () => {
            const selected = await open({ directory: false, multiple: false, filters: [{ name: "LZMA Files", extensions: ["lzma", "xz"] }] });
            if (selected) props.setInputPath(selected as string);
          }}>
            Browse
          </button>
        </div>
      </div>
      <div class="space-y-2">
        <label class="label">Output File</label>
        <div class="flex gap-2">
          <input
            type="text"
            class="input-inline"
            value={props.outputPath()}
            onInput={(e) => props.setOutputPath(e.currentTarget.value)}
            placeholder="Decompressed output path..."
          />
          <button class="btn-sm" onClick={async () => {
            const selected = await save({});
            if (selected) props.setOutputPath(selected as string);
          }}>
            Browse
          </button>
        </div>
      </div>
    </div>
  );
}
