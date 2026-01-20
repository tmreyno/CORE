// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * TransferOptions Component
 * 
 * Collapsible options panel for transfer settings including:
 * - Verification toggle
 * - Hash algorithm selection
 * - Timestamp preservation
 * - Overwrite behavior
 * - Recursive copying
 */

import { For } from "solid-js";
import { HASH_ALGORITHMS } from "../../types";
import type { TransferOptionsProps } from "./types";

export function TransferOptions(props: TransferOptionsProps) {
  return (
    <div class={`bg-bg-secondary/50 rounded border border-border/50 p-2.5 space-y-2`}>
      <div class="grid grid-cols-2 gap-x-4 gap-y-2">
        {/* Verify */}
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={props.verify}
            onChange={(e) => props.setVerify(e.currentTarget.checked)}
            class={`w-3 h-3 rounded border-border bg-bg-hover text-accent focus:ring-accent/50`}
          />
          <span class={`text-[11px] leading-tight text-txt-tertiary`}>Verify after copy</span>
        </label>

        {/* Preserve Timestamps */}
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={props.preserveTimestamps}
            onChange={(e) => props.setPreserveTimestamps(e.currentTarget.checked)}
            class={`w-3 h-3 rounded border-border bg-bg-hover text-accent focus:ring-accent/50`}
          />
          <span class={`text-[11px] leading-tight text-txt-tertiary`}>Preserve timestamps</span>
        </label>

        {/* Recursive */}
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={props.recursive}
            onChange={(e) => props.setRecursive(e.currentTarget.checked)}
            class={`w-3 h-3 rounded border-border bg-bg-hover text-accent focus:ring-accent/50`}
          />
          <span class={`text-[11px] leading-tight text-txt-tertiary`}>Include subfolders</span>
        </label>

        {/* Overwrite */}
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={props.overwrite}
            onChange={(e) => props.setOverwrite(e.currentTarget.checked)}
            class={`w-3 h-3 rounded border-border bg-bg-hover text-accent focus:ring-accent/50`}
          />
          <span class={`text-[11px] leading-tight text-txt-tertiary`}>Overwrite existing</span>
        </label>
      </div>

      {/* Hash Algorithm */}
      <div class="flex items-center gap-2">
        <label class={`text-[11px] leading-tight text-txt-secondary`}>Hash:</label>
        <select
          value={props.hashAlgorithm}
          onChange={(e) => props.setHashAlgorithm(e.currentTarget.value)}
          class={`flex-1 w-full px-2 py-1 text-xs bg-bg-tertiary border border-border rounded text-txt-primary placeholder:text-txt-muted focus:outline-none focus:ring-1 focus:ring-accent max-w-[140px]`}
        >
          <For each={HASH_ALGORITHMS}>
            {(alg) => <option value={alg.value}>{alg.label}</option>}
          </For>
        </select>
      </div>
    </div>
  );
}
