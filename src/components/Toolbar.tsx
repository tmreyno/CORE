// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, Show } from "solid-js";
import { HASH_ALGORITHMS } from "../types";
import type { HashAlgorithm, HashAlgorithmInfo } from "../types";
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
  selectedHashAlgorithm: HashAlgorithm;
  onHashAlgorithmChange: (algorithm: HashAlgorithm) => void;
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

// Get tooltip for hash algorithm
const getAlgorithmTooltip = (alg: HashAlgorithmInfo): string => {
  const parts: string[] = [alg.label.replace(/ ⚡+/g, '')];
  if (alg.speed === "fast") parts.push("Very Fast");
  else if (alg.speed === "medium") parts.push("Medium Speed");
  else parts.push("Slower");
  if (alg.forensic) parts.push("Court-accepted");
  if (alg.cryptographic) parts.push("Cryptographic");
  else parts.push("Non-cryptographic");
  return parts.join(" • ");
};

export function Toolbar(props: ToolbarProps) {
  // Get current algorithm info for tooltip
  const currentAlgoInfo = () => HASH_ALGORITHMS.find(a => a.value === props.selectedHashAlgorithm);
  const compact = () => props.compact ?? false;
  
  // Button styles
  const btnBase = "px-3 py-1.5 text-sm font-medium rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed";
  const btnPrimary = `${btnBase} bg-accent text-white hover:bg-accent-hover`;
  const btnDefault = `${btnBase} bg-bg-hover text-txt hover:bg-bg-active border border-border`;
  const btnIcon = `p-2 rounded transition-colors disabled:opacity-50`;
  
  return (
    <nav class="flex items-center gap-2 px-3 py-2 bg-bg-toolbar border-b border-border shrink-0 h-11 flex-nowrap overflow-x-auto" role="toolbar" aria-label="Evidence tools">
      <button 
        class={btnPrimary}
        onClick={props.onBrowse} 
        disabled={props.busy}
        title="Open Evidence Directory"
        aria-label="Open evidence directory"
      >
        <HiOutlineFolderOpen class="w-4 h-4" />
      </button>
      
      <div class={`flex ${compact() ? 'flex-1 min-w-[80px] max-w-[200px]' : 'flex-1 min-w-[120px] max-w-[400px]'}`}>
        <input 
          type="text" 
          class="flex-1 min-w-0 px-2.5 py-1.5 bg-bg border border-border rounded-l text-txt text-sm focus:outline-none focus:border-accent overflow-hidden text-ellipsis"
          style={{ direction: "rtl", "text-align": "left" }}
          value={props.scanDir} 
          onInput={(e) => props.onScanDirChange(e.currentTarget.value)} 
          placeholder={compact() ? "Path..." : "Evidence directory path..."} 
          onKeyDown={(e) => e.key === "Enter" && props.onScan()}
          title={props.scanDir || "Evidence directory path"}
        />
        <button 
          class="px-3 py-1.5 bg-bg-hover text-txt hover:bg-bg-active rounded-r border border-border border-l-0 disabled:opacity-50" 
          onClick={props.onScan} 
          disabled={props.busy || !props.scanDir}
          title="Scan Directory"
        >
          <HiOutlineMagnifyingGlass class="w-4 h-4" />
        </button>
      </div>
      
      <Show when={!compact()}>
        <label class="flex items-center gap-1.5 text-sm text-txt-secondary cursor-pointer whitespace-nowrap" title="Scan subdirectories">
          <input 
            type="checkbox" 
            class="accent-current text-accent"
            checked={props.recursiveScan} 
            onChange={(e) => props.onRecursiveScanChange(e.currentTarget.checked)} 
          />
          <span>Recursive</span>
        </label>
      </Show>
      <Show when={compact()}>
        <button
          class={`${btnIcon} ${props.recursiveScan ? 'bg-accent text-white' : 'bg-bg-hover text-txt hover:bg-bg-active'}`}
          onClick={() => props.onRecursiveScanChange(!props.recursiveScan)}
          title={props.recursiveScan ? "Recursive scan enabled" : "Recursive scan disabled"}
        >
          <HiOutlineArrowPath class="w-4 h-4" />
        </button>
      </Show>
      
      <div class="w-px h-6 bg-bg-hover mx-1" />
      
      <select 
        class={`px-2 py-1.5 text-sm rounded border bg-bg-panel text-txt focus:outline-none focus:border-accent ${compact() ? 'w-20' : ''} ${currentAlgoInfo()?.speed === 'fast' ? 'border-green-500 bg-gradient-to-br from-bg-panel to-green-900/30' : 'border-border'}`}
        value={props.selectedHashAlgorithm} 
        onChange={(e) => props.onHashAlgorithmChange(e.currentTarget.value as HashAlgorithm)} 
        title={currentAlgoInfo() ? getAlgorithmTooltip(currentAlgoInfo()!) : "Hash algorithm"}
      >
        <optgroup label="Forensic Standard">
          <For each={HASH_ALGORITHMS.filter(a => a.forensic)}>
            {(alg) => <option value={alg.value} title={getAlgorithmTooltip(alg)}>{compact() ? alg.value.toUpperCase() : alg.label}</option>}
          </For>
        </optgroup>
        <optgroup label="Fast (Non-forensic)">
          <For each={HASH_ALGORITHMS.filter(a => !a.forensic)}>
            {(alg) => <option value={alg.value} title={getAlgorithmTooltip(alg)}>{compact() ? alg.value.toUpperCase() : alg.label}</option>}
          </For>
        </optgroup>
      </select>
      
      <button 
        class={`${btnDefault} flex items-center gap-1`}
        onClick={props.onHashSelected} 
        disabled={props.busy || props.selectedCount === 0} 
        title={`Hash ${props.selectedCount} selected files`}
        aria-label={`Hash ${props.selectedCount} selected files`}
      >
        <HiOutlineFingerPrint class="w-4 h-4" />
        <span class="text-xs">({props.selectedCount})</span>
      </button>
      
      <button 
        class={btnDefault}
        onClick={props.onLoadAll} 
        disabled={props.busy || props.discoveredCount === 0} 
        title="Load metadata for all files"
        aria-label="Load all file metadata"
      >
        <HiOutlineInformationCircle class="w-4 h-4" />
      </button>
    </nav>
  );
}
