// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Toolbar - Primary action bar for evidence operations
 * 
 * Features:
 * - Open evidence directories
 * - Scan and discover forensic containers
 * - Hash computation with algorithm selection
 * - Responsive compact mode for narrow screens
 * - Keyboard shortcuts and accessibility
 */

import { For, Show } from "solid-js";
import { HASH_ALGORITHMS } from "../types";
import type { HashAlgorithmInfo } from "../types";
import type { HashAlgorithmName } from "../types/hash";
import {
  HiOutlineFolderOpen,
  HiOutlineMagnifyingGlass,
  HiOutlineArrowPath,
  HiOutlineFingerPrint,
  HiOutlineInformationCircle,
} from "./icons";

interface ToolbarProps {
  scanDir: string;
  onScanDirChange: (dir: string) => void;
  recursiveScan: boolean;
  onRecursiveScanChange: (recursive: boolean) => void;
  selectedHashAlgorithm: HashAlgorithmName;
  onHashAlgorithmChange: (algorithm: HashAlgorithmName) => void;
  selectedCount: number;
  discoveredCount: number;
  busy: boolean;
  onBrowse: () => void;
  onScan: () => void;
  onHashSelected: () => void;
  onLoadAll: () => void;
  // Responsive mode - show only icons when true
  compact?: boolean;
}

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

export function Toolbar(props: ToolbarProps) {
  // Get current algorithm info for tooltip and visual indicators
  const currentAlgoInfo = () => HASH_ALGORITHMS.find(a => a.value === props.selectedHashAlgorithm);
  const compact = () => props.compact ?? false;
  
  // Derived state for better UX feedback
  const hasSelection = () => props.selectedCount > 0;
  const hasEvidence = () => props.discoveredCount > 0;
  const hasPath = () => props.scanDir.length > 0;
  
  // Button style classes - organized by visual hierarchy
  const btnBase = "flex items-center justify-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-md transition-all duration-150 disabled:opacity-40 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-accent/50 focus:ring-offset-1 focus:ring-offset-bg";
  const btnPrimary = `${btnBase} bg-accent text-white hover:bg-accent-hover active:scale-[0.98] shadow-sm`;
  const btnSecondary = `${btnBase} bg-bg-secondary text-txt hover:bg-bg-hover active:bg-bg-active border border-border hover:border-border-strong`;
  const btnGhost = `${btnBase} bg-transparent text-txt-secondary hover:text-txt hover:bg-bg-hover`;
  const btnIcon = "flex items-center justify-center p-2 rounded-md transition-all duration-150 disabled:opacity-40 focus:outline-none focus:ring-2 focus:ring-accent/50";
  
  // Input group styling
  const inputGroup = "flex rounded-md shadow-sm";
  const inputField = "flex-1 min-w-0 px-3 py-1.5 bg-bg border border-border rounded-l-md text-txt text-sm placeholder:text-txt-muted focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-accent transition-colors";
  const inputAddon = "px-3 py-1.5 bg-bg-secondary text-txt-secondary hover:text-txt hover:bg-bg-hover rounded-r-md border border-l-0 border-border transition-colors disabled:opacity-40";
  
  return (
    <nav 
      class="flex items-center gap-3 px-4 py-2.5 bg-bg-toolbar border-b border-border shrink-0 h-12 flex-nowrap overflow-x-auto" 
      role="toolbar" 
      aria-label="Evidence tools"
    >
      {/* === Evidence Section === */}
      <div class="flex items-center gap-2">
        <button 
          class={btnPrimary}
          onClick={props.onBrowse} 
          disabled={props.busy}
          title="Open Evidence Directory (⌘O)"
          aria-label="Open evidence directory"
        >
          <HiOutlineFolderOpen class="w-4 h-4" />
          <Show when={!compact()}>
            <span>Open</span>
          </Show>
        </button>
        
        {/* Path input with scan button */}
        <div class={`${inputGroup} ${compact() ? 'w-[180px]' : 'w-[320px]'}`}>
          <input 
            type="text" 
            class={inputField}
            style={{ direction: "rtl", "text-align": "left" }}
            value={props.scanDir} 
            onInput={(e) => props.onScanDirChange(e.currentTarget.value)} 
            placeholder={compact() ? "Path..." : "Evidence directory path..."} 
            onKeyDown={(e) => e.key === "Enter" && props.onScan()}
            title={props.scanDir || "Enter evidence directory path"}
            aria-label="Evidence directory path"
          />
          <button 
            class={inputAddon}
            onClick={props.onScan} 
            disabled={props.busy || !hasPath()}
            title="Scan Directory (Enter)"
            aria-label="Scan directory"
          >
            <HiOutlineMagnifyingGlass class="w-4 h-4" />
          </button>
        </div>
        
        {/* Recursive scan toggle */}
        <Show when={!compact()}>
          <label 
            class="flex items-center gap-2 px-2 py-1.5 text-sm text-txt-secondary hover:text-txt cursor-pointer rounded-md hover:bg-bg-hover transition-colors select-none" 
            title="Include subdirectories in scan"
          >
            <input 
              type="checkbox" 
              class="w-4 h-4 rounded border-border bg-bg text-accent focus:ring-accent focus:ring-offset-bg"
              checked={props.recursiveScan} 
              onChange={(e) => props.onRecursiveScanChange(e.currentTarget.checked)} 
            />
            <span>Recursive</span>
          </label>
        </Show>
        <Show when={compact()}>
          <button
            class={`${btnIcon} ${props.recursiveScan ? 'bg-accent/20 text-accent' : 'text-txt-muted hover:text-txt hover:bg-bg-hover'}`}
            onClick={() => props.onRecursiveScanChange(!props.recursiveScan)}
            title={props.recursiveScan ? "Recursive scan: ON" : "Recursive scan: OFF"}
            aria-label={props.recursiveScan ? "Disable recursive scan" : "Enable recursive scan"}
            aria-pressed={props.recursiveScan}
          >
            <HiOutlineArrowPath class={`w-4 h-4 ${props.recursiveScan ? 'animate-spin-slow' : ''}`} />
          </button>
        </Show>
      </div>
      
      {/* Divider */}
      <div class="w-px h-6 bg-border/50 mx-1" aria-hidden="true" />
      
      {/* === Hash Section === */}
      <div class="flex items-center gap-2">
        {/* Algorithm selector with speed indicator */}
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
            value={props.selectedHashAlgorithm} 
            onChange={(e) => props.onHashAlgorithmChange(e.currentTarget.value as HashAlgorithmName)} 
            title={currentAlgoInfo() ? getAlgorithmTooltip(currentAlgoInfo()!) : "Select hash algorithm"}
            aria-label="Hash algorithm"
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
        
        {/* Hash action button with count badge */}
        <button 
          class={`${btnSecondary} relative ${hasSelection() ? 'pr-8' : ''}`}
          onClick={props.onHashSelected} 
          disabled={props.busy || !hasSelection()} 
          title={hasSelection() ? `Hash ${props.selectedCount} selected file${props.selectedCount > 1 ? 's' : ''}` : "Select files to hash"}
          aria-label={`Hash ${props.selectedCount} selected files`}
        >
          <HiOutlineFingerPrint class="w-4 h-4" />
          <Show when={!compact()}>
            <span>Hash</span>
          </Show>
          <Show when={hasSelection()}>
            <span class="absolute -top-1.5 -right-1.5 flex items-center justify-center min-w-[18px] h-[18px] px-1 text-[10px] font-bold bg-accent text-white rounded-full shadow-sm">
              {props.selectedCount}
            </span>
          </Show>
        </button>
        
        {/* Load metadata button with count */}
        <button 
          class={`${btnGhost} relative ${hasEvidence() ? 'pr-7' : ''}`}
          onClick={props.onLoadAll} 
          disabled={props.busy || !hasEvidence()} 
          title={hasEvidence() ? `Load metadata for ${props.discoveredCount} file${props.discoveredCount > 1 ? 's' : ''}` : "No evidence files discovered"}
          aria-label="Load all file metadata"
        >
          <HiOutlineInformationCircle class="w-4 h-4" />
          <Show when={!compact()}>
            <span>Info</span>
          </Show>
          <Show when={hasEvidence()}>
            <span class="absolute -top-1 -right-1 flex items-center justify-center min-w-[16px] h-4 px-0.5 text-[9px] font-medium bg-bg-active text-txt-secondary rounded-full">
              {props.discoveredCount}
            </span>
          </Show>
        </button>
      </div>
    </nav>
  );
}
