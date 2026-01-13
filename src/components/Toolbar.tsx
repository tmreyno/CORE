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
  HiOutlineDocumentArrowDown,
  HiOutlineFolderArrowDown,
  HiOutlineDocumentText,
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
  // Project management
  projectPath?: string | null;
  projectModified?: boolean;
  onSaveProject?: () => void;
  onLoadProject?: () => void;
  // Report generation
  onGenerateReport?: () => void;
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
  const btnPrimary = `${btnBase} bg-cyan-600 text-white hover:bg-cyan-500`;
  const btnDefault = `${btnBase} bg-zinc-700 text-zinc-200 hover:bg-zinc-600 border border-zinc-600`;
  const btnWarning = `${btnBase} bg-amber-600 text-white hover:bg-amber-500`;
  const btnIcon = `p-2 rounded transition-colors disabled:opacity-50`;
  
  return (
    <nav class="flex items-center gap-2 px-3 py-2 bg-zinc-900/50 border-b border-zinc-700 shrink-0 h-11 flex-nowrap overflow-x-auto" role="toolbar" aria-label="Evidence tools">
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
          class="flex-1 min-w-0 px-2.5 py-1.5 bg-zinc-900 border border-zinc-700 rounded-l text-zinc-200 text-sm focus:outline-none focus:border-cyan-500 overflow-hidden text-ellipsis"
          style={{ direction: "rtl", "text-align": "left" }}
          value={props.scanDir} 
          onInput={(e) => props.onScanDirChange(e.currentTarget.value)} 
          placeholder={compact() ? "Path..." : "Evidence directory path..."} 
          onKeyDown={(e) => e.key === "Enter" && props.onScan()}
          title={props.scanDir || "Evidence directory path"}
        />
        <button 
          class="px-3 py-1.5 bg-zinc-700 text-zinc-200 hover:bg-zinc-600 rounded-r border border-zinc-600 border-l-0 disabled:opacity-50" 
          onClick={props.onScan} 
          disabled={props.busy || !props.scanDir}
          title="Scan Directory"
        >
          <HiOutlineMagnifyingGlass class="w-4 h-4" />
        </button>
      </div>
      
      <Show when={!compact()}>
        <label class="flex items-center gap-1.5 text-sm text-zinc-400 cursor-pointer whitespace-nowrap" title="Scan subdirectories">
          <input 
            type="checkbox" 
            class="accent-cyan-500"
            checked={props.recursiveScan} 
            onChange={(e) => props.onRecursiveScanChange(e.currentTarget.checked)} 
          />
          <span>Recursive</span>
        </label>
      </Show>
      <Show when={compact()}>
        <button
          class={`${btnIcon} ${props.recursiveScan ? 'bg-cyan-600 text-white' : 'bg-zinc-700 text-zinc-200 hover:bg-zinc-600'}`}
          onClick={() => props.onRecursiveScanChange(!props.recursiveScan)}
          title={props.recursiveScan ? "Recursive scan enabled" : "Recursive scan disabled"}
        >
          <HiOutlineArrowPath class="w-4 h-4" />
        </button>
      </Show>
      
      <div class="w-px h-6 bg-zinc-700 mx-1" />
      
      <select 
        class={`px-2 py-1.5 text-sm rounded border bg-zinc-800 text-zinc-200 focus:outline-none focus:border-cyan-500 ${compact() ? 'w-20' : ''} ${currentAlgoInfo()?.speed === 'fast' ? 'border-green-500 bg-gradient-to-br from-zinc-800 to-green-900/30' : 'border-zinc-600'}`}
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
      
      <div class="w-px h-6 bg-zinc-700 mx-1" />
      
      {/* Project Management */}
      <Show when={props.onSaveProject}>
        <button 
          class={props.projectModified ? btnWarning : btnDefault}
          onClick={props.onSaveProject} 
          disabled={props.busy || !props.scanDir}
          title={props.projectPath 
            ? `Save project to ${props.projectPath}${props.projectModified ? ' (unsaved changes)' : ''}`
            : "Save project"}
          aria-label="Save project"
        >
          <HiOutlineDocumentArrowDown class="w-4 h-4" />
          <Show when={props.projectModified}>
            <span class="text-xs ml-0.5">*</span>
          </Show>
        </button>
      </Show>
      
      <Show when={props.onLoadProject}>
        <button 
          class={btnDefault}
          onClick={props.onLoadProject} 
          disabled={props.busy}
          title="Load project file"
          aria-label="Load project"
        >
          <HiOutlineFolderArrowDown class="w-4 h-4" />
        </button>
      </Show>
      
      <div class="w-px h-6 bg-zinc-700 mx-1" />
      
      {/* Report Generation */}
      <Show when={props.onGenerateReport}>
        <button 
          class={btnPrimary}
          onClick={props.onGenerateReport} 
          disabled={props.busy || props.discoveredCount === 0}
          title="Generate forensic report"
          aria-label="Generate report"
        >
          <HiOutlineDocumentText class="w-4 h-4" />
        </button>
      </Show>
    </nav>
  );
}
