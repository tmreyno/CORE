// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For } from "solid-js";
import { open, save } from "@tauri-apps/plugin-dialog";
import { HiOutlineInformationCircle } from "../../icons";
import { LZMA_COMPRESSION_LEVELS } from "../../../api/lzmaApi";
import type { Accessor } from "solid-js";

interface CompressTabProps {
  algorithm: Accessor<"lzma" | "lzma2">;
  setAlgorithm: (algo: "lzma" | "lzma2") => void;
  level: Accessor<number>;
  setLevel: (level: number) => void;
  inputPath: Accessor<string>;
  setInputPath: (path: string) => void;
  outputPath: Accessor<string>;
  setOutputPath: (path: string) => void;
}

export function CompressTab(props: CompressTabProps) {
  return (
    <div class="space-y-3">
      <div class="info-card">
        <HiOutlineInformationCircle class="w-5 h-5 text-info" />
        <div>
          <div class="font-medium text-txt">LZMA/LZMA2 Compression</div>
          <div class="text-xs text-txt-muted mt-1">
            Compress a single file using LZMA (.lzma) or LZMA2 (.xz) algorithm. Uses LZMA SDK 24.09.
          </div>
        </div>
      </div>

      <div class="space-y-2">
        <label class="label">Algorithm</label>
        <div class="flex gap-2">
          <button
            class={`px-3 py-1.5 rounded-lg text-xs transition-colors ${
              props.algorithm() === "lzma"
                ? "bg-accent text-white"
                : "bg-bg-secondary text-txt-secondary hover:text-txt"
            }`}
            onClick={() => props.setAlgorithm("lzma")}
          >
            LZMA (.lzma)
          </button>
          <button
            class={`px-3 py-1.5 rounded-lg text-xs transition-colors ${
              props.algorithm() === "lzma2"
                ? "bg-accent text-white"
                : "bg-bg-secondary text-txt-secondary hover:text-txt"
            }`}
            onClick={() => props.setAlgorithm("lzma2")}
          >
            LZMA2 (.xz)
          </button>
        </div>
      </div>

      <div class="space-y-2">
        <label class="label">Compression Level</label>
        <select
          class="input-sm"
          value={props.level()}
          onChange={(e) => props.setLevel(Number(e.currentTarget.value))}
        >
          <For each={LZMA_COMPRESSION_LEVELS}>
            {(level) => (
              <option value={level.value}>
                {level.label} (dict: {level.dictSize})
              </option>
            )}
          </For>
        </select>
      </div>

      <div class="space-y-2">
        <label class="label">Input File</label>
        <div class="flex gap-2">
          <input
            type="text"
            class="input-inline"
            value={props.inputPath()}
            onInput={(e) => props.setInputPath(e.currentTarget.value)}
            placeholder="Select file to compress..."
          />
          <button class="btn-sm" onClick={async () => {
            const selected = await open({ directory: false, multiple: false });
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
            placeholder={`Output path (.${props.algorithm() === "lzma" ? "lzma" : "xz"})...`}
          />
          <button class="btn-sm" onClick={async () => {
            const ext = props.algorithm() === "lzma" ? "lzma" : "xz";
            const selected = await save({ filters: [{ name: `${ext.toUpperCase()} File`, extensions: [ext] }] });
            if (selected) props.setOutputPath(selected as string);
          }}>
            Browse
          </button>
        </div>
      </div>
    </div>
  );
}
