// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * HashAlgorithmSelector - Hash algorithm selector with visual indicators
 * 
 * Features:
 * - Speed badge (fast/medium/slow)
 * - Grouped by forensic vs non-forensic
 * - Visual indicators for forensic algorithms
 * - Tooltips with algorithm details
 */

import { Component, For, Show } from "solid-js";
import { HASH_ALGORITHMS } from "../../types";
import type { HashAlgorithmName } from "../../types/hash";
import type { HashAlgorithmInfo } from "../../types";

interface HashAlgorithmSelectorProps {
  selectedAlgorithm: HashAlgorithmName;
  onAlgorithmChange: (algorithm: HashAlgorithmName) => void;
  compact?: boolean;
  disabled?: boolean;
}

// Speed indicator badge component
const SpeedBadge = (props: { speed: "fast" | "medium" | "slow" }) => {
  const colors = {
    fast: "bg-success/20 text-success border-success/30",
    medium: "bg-warning/20 text-warning border-warning/30", 
    slow: "bg-txt-muted/20 text-txt-muted border-txt-muted/30",
  };
  const labels = { fast: "⚡", medium: "◑", slow: "○" };
  
  return (
    <span class={`inline-flex items-center justify-center w-4 h-4 text-[10px] rounded border ${colors[props.speed]}`}>
      {labels[props.speed]}
    </span>
  );
};

// Get tooltip for hash algorithm with visual indicators
const getAlgorithmTooltip = (alg: HashAlgorithmInfo): string => {
  const parts: string[] = [alg.label.replace(/ ⚡+/g, '')];
  if (alg.speed === "fast") parts.push("⚡ Very Fast");
  else if (alg.speed === "medium") parts.push("Medium Speed");
  else parts.push("Slower");
  if (alg.forensic) parts.push("✓ Court-accepted");
  if (alg.cryptographic) parts.push("🔒 Cryptographic");
  else parts.push("Non-cryptographic");
  return parts.join(" • ");
};

export const HashAlgorithmSelector: Component<HashAlgorithmSelectorProps> = (props) => {
  const compact = () => props.compact ?? false;
  const currentAlgoInfo = () => HASH_ALGORITHMS.find(a => a.value === props.selectedAlgorithm);
  
  return (
    <div class="flex items-center gap-1.5">
      <Show when={currentAlgoInfo()}>
        <SpeedBadge speed={currentAlgoInfo()!.speed} />
      </Show>
      <select 
        class={`px-2.5 py-1.5 text-sm rounded-md border bg-bg-secondary text-txt focus:outline-none focus:ring-2 focus:ring-accent/50 cursor-pointer transition-colors ${
          compact() ? 'w-24' : 'min-w-[140px]'
        } ${
          currentAlgoInfo()?.forensic 
            ? 'border-success/40 bg-success/5' 
            : 'border-border'
        }`}
        value={props.selectedAlgorithm} 
        onChange={(e) => props.onAlgorithmChange(e.currentTarget.value as HashAlgorithmName)} 
        title={currentAlgoInfo() ? getAlgorithmTooltip(currentAlgoInfo()!) : "Select hash algorithm"}
        aria-label="Hash algorithm"
        disabled={props.disabled}
      >
        <optgroup label="📋 Forensic Standard">
          <For each={HASH_ALGORITHMS.filter(a => a.forensic)}>
            {(alg) => (
              <option value={alg.value} title={getAlgorithmTooltip(alg)}>
                {compact() ? alg.value.toUpperCase() : alg.label}
              </option>
            )}
          </For>
        </optgroup>
        <optgroup label="⚡ Fast (Non-forensic)">
          <For each={HASH_ALGORITHMS.filter(a => !a.forensic)}>
            {(alg) => (
              <option value={alg.value} title={getAlgorithmTooltip(alg)}>
                {compact() ? alg.value.toUpperCase() : alg.label}
              </option>
            )}
          </For>
        </optgroup>
      </select>
    </div>
  );
};
