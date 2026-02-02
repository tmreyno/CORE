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

import { For, Show, createMemo, createSignal } from "solid-js";
import type { Accessor } from "solid-js";
import { HASH_ALGORITHMS } from "../types";
import type { HashAlgorithmInfo } from "../types";
import type { HashAlgorithmName } from "../types/hash";
import {
  HiOutlineFolderOpen,
  HiOutlineMagnifyingGlass,
  HiOutlineArrowPath,
  HiOutlineFingerPrint,
  HiOutlineInformationCircle,
  HiOutlineChevronDown,
  HiOutlineDocumentText,
  HiOutlineDocumentArrowDown,
  HiOutlineCheck,
} from "./icons";

/** Project location entry for dropdown */
interface ProjectLocation {
  id: string;
  label: string;
  path: string | null;
  icon: "evidence" | "database" | "documents";
}

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
  onOpenProject: () => void;
  onSave: () => void;
  onSaveAs: () => void;
  onScan: () => void;
  onHashSelected: () => void;
  onLoadAll: () => void;
  // Auto-save state
  autoSaveEnabled?: Accessor<boolean>;
  onAutoSaveToggle?: () => void;
  // Project modified state
  projectModified?: Accessor<boolean>;
  // Responsive mode - show only icons when true
  compact?: boolean;
  // Project locations for dropdown
  evidencePath?: Accessor<string | null>;
  processedDbPath?: Accessor<string | null>;
  caseDocumentsPath?: Accessor<string | null>;
  projectName?: Accessor<string | null>;
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
  
  // Dropdown menu states
  const [showOpenMenu, setShowOpenMenu] = createSignal(false);
  const [showSaveMenu, setShowSaveMenu] = createSignal(false);
  
  // Derived state for better UX feedback
  const hasSelection = () => props.selectedCount > 0;
  const hasEvidence = () => props.discoveredCount > 0;
  const hasPath = () => props.scanDir.length > 0;
  const isModified = () => props.projectModified?.() ?? false;
  
  // Button style classes - organized by visual hierarchy
  const btnBase = "flex items-center justify-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-md transition-all duration-150 disabled:opacity-40 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-accent/50 focus:ring-offset-1 focus:ring-offset-bg";
  const btnPrimary = `${btnBase} bg-accent text-white hover:bg-accent-hover active:scale-[0.98] shadow-sm`;
  const btnSecondary = `${btnBase} bg-bg-secondary text-txt hover:bg-bg-hover active:bg-bg-active border border-border hover:border-border-strong`;
  const btnGhost = `${btnBase} bg-transparent text-txt-secondary hover:text-txt hover:bg-bg-hover`;
  const btnIcon = "flex items-center justify-center p-2 rounded-md transition-all duration-150 disabled:opacity-40 focus:outline-none focus:ring-2 focus:ring-accent/50";
  
  // Build project locations for dropdown
  const projectLocations = createMemo((): ProjectLocation[] => {
    const locations: ProjectLocation[] = [];
    
    const evidence = props.evidencePath?.();
    const processed = props.processedDbPath?.();
    const caseDocs = props.caseDocumentsPath?.();
    
    if (evidence) {
      locations.push({ id: "evidence", label: "Evidence", path: evidence, icon: "evidence" });
    }
    if (processed) {
      locations.push({ id: "processed", label: "Processed Database", path: processed, icon: "database" });
    }
    if (caseDocs) {
      locations.push({ id: "documents", label: "Case Documents", path: caseDocs, icon: "documents" });
    }
    
    return locations;
  });
  
  // Check if we have any project locations
  const hasProjectLocations = () => projectLocations().length > 0;
  
  // Get the folder name from a path
  const getFolderName = (path: string | null): string => {
    if (!path) return "";
    const parts = path.split("/").filter(Boolean);
    return parts[parts.length - 1] || path;
  };
  
  return (
    <nav 
      class="flex items-center gap-3 px-4 py-2.5 bg-bg-toolbar border-b border-border shrink-0 h-12 flex-nowrap overflow-visible" 
      role="toolbar" 
      aria-label="Evidence tools"
    >
      {/* === Evidence Section === */}
      <div class="flex items-center gap-2">
        {/* Open Button with Dropdown Menu */}
        <div class="relative">
          <button 
            class={btnPrimary}
            onClick={() => setShowOpenMenu(!showOpenMenu())}
            disabled={props.busy}
            title="Open Directory or Project"
            aria-label="Open menu"
            aria-expanded={showOpenMenu()}
            aria-haspopup="menu"
          >
            <HiOutlineFolderOpen class="w-4 h-4" />
            <Show when={!compact()}>
              <span>Open</span>
            </Show>
            <HiOutlineChevronDown class={`w-3 h-3 transition-transform ${showOpenMenu() ? 'rotate-180' : ''}`} />
          </button>
          
          {/* Dropdown Menu */}
          <Show when={showOpenMenu()}>
            {/* Click outside overlay to close */}
            <div 
              class="fixed inset-0 z-[9]"
              onClick={() => setShowOpenMenu(false)}
            />
            <div 
              class="absolute top-full left-0 mt-1 w-48 bg-bg-panel border border-border rounded-lg shadow-lg z-dropdown py-1"
            >
              <button
                class="w-full flex items-center gap-2 px-3 py-2 text-sm text-txt hover:bg-bg-hover transition-colors text-left"
                onClick={() => {
                  setShowOpenMenu(false);
                  props.onBrowse();
                }}
              >
                <HiOutlineFolderOpen class="w-4 h-4 text-txt-muted" />
                <span>Open Directory</span>
              </button>
              <button
                class="w-full flex items-center gap-2 px-3 py-2 text-sm text-txt hover:bg-bg-hover transition-colors text-left"
                onClick={() => {
                  setShowOpenMenu(false);
                  props.onOpenProject();
                }}
              >
                <HiOutlineDocumentText class="w-4 h-4 text-txt-muted" />
                <span>Open Project</span>
              </button>
            </div>
          </Show>
        </div>
        
        {/* Save Button with Dropdown Menu */}
        <div class="relative">
          <button 
            class={`${btnSecondary} ${isModified() ? 'border-warning text-warning' : ''}`}
            onClick={() => setShowSaveMenu(!showSaveMenu())}
            disabled={props.busy}
            title={isModified() ? "Save Project (unsaved changes)" : "Save Project"}
            aria-label="Save menu"
            aria-expanded={showSaveMenu()}
            aria-haspopup="menu"
          >
            <HiOutlineDocumentArrowDown class="w-4 h-4" />
            <Show when={!compact()}>
              <span>Save</span>
            </Show>
            <Show when={isModified()}>
              <span class="w-2 h-2 rounded-full bg-warning" />
            </Show>
            <HiOutlineChevronDown class={`w-3 h-3 transition-transform ${showSaveMenu() ? 'rotate-180' : ''}`} />
          </button>
          
          {/* Save Dropdown Menu */}
          <Show when={showSaveMenu()}>
            {/* Click outside overlay to close */}
            <div 
              class="fixed inset-0 z-[9]"
              onClick={() => setShowSaveMenu(false)}
            />
            <div 
              class="absolute top-full left-0 mt-1 w-52 bg-bg-panel border border-border rounded-lg shadow-lg z-dropdown py-1"
            >
              <button
                class="w-full flex items-center gap-2 px-3 py-2 text-sm text-txt hover:bg-bg-hover transition-colors text-left"
                onClick={() => {
                  setShowSaveMenu(false);
                  props.onSave();
                }}
              >
                <HiOutlineDocumentArrowDown class="w-4 h-4 text-txt-muted" />
                <div class="flex-1">
                  <span>Save</span>
                  <span class="text-txt-muted text-xs ml-2">⌘S</span>
                </div>
              </button>
              <button
                class="w-full flex items-center gap-2 px-3 py-2 text-sm text-txt hover:bg-bg-hover transition-colors text-left"
                onClick={() => {
                  setShowSaveMenu(false);
                  props.onSaveAs();
                }}
              >
                <HiOutlineDocumentText class="w-4 h-4 text-txt-muted" />
                <div class="flex-1">
                  <span>Save As...</span>
                  <span class="text-txt-muted text-xs ml-2">⌘⇧S</span>
                </div>
              </button>
              
              {/* Divider */}
              <div class="h-px bg-border my-1" />
              
              {/* Auto-save toggle */}
              <Show when={props.onAutoSaveToggle}>
                <button
                  class="w-full flex items-center gap-2 px-3 py-2 text-sm text-txt hover:bg-bg-hover transition-colors text-left"
                  onClick={() => {
                    props.onAutoSaveToggle?.();
                  }}
                >
                  <Show 
                    when={props.autoSaveEnabled?.()} 
                    fallback={<div class="w-4 h-4" />}
                  >
                    <HiOutlineCheck class="w-4 h-4 text-success" />
                  </Show>
                  <span>Auto-save</span>
                </button>
              </Show>
            </div>
          </Show>
        </div>
        
        {/* Project Location Dropdown or Empty State */}
        <Show 
          when={hasProjectLocations()} 
          fallback={
            <div class={`flex items-center gap-2 px-3 py-1.5 text-sm text-txt-muted border border-border rounded-md bg-bg ${compact() ? 'w-[140px]' : 'w-[240px]'}`}>
              <HiOutlineFolderOpen class="w-4 h-4 shrink-0" />
              <span class="truncate">{compact() ? "No project" : "No project open"}</span>
            </div>
          }
        >
          <div class={`relative ${compact() ? 'w-[180px]' : 'w-[280px]'}`}>
            <select
              class="w-full appearance-none px-3 py-1.5 pr-8 text-sm bg-bg border border-border rounded-md text-txt focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-accent transition-colors cursor-pointer"
              value={props.scanDir || ""}
              onChange={(e) => {
                const newPath = e.currentTarget.value;
                if (newPath) {
                  props.onScanDirChange(newPath);
                  props.onScan();
                }
              }}
              title={props.scanDir || "Select project location"}
            >
              <option value="" disabled>Select location...</option>
              <For each={projectLocations()}>
                {(location) => (
                  <option value={location.path || ""} title={location.path || ""}>
                    {location.label}: {getFolderName(location.path)}
                  </option>
                )}
              </For>
            </select>
            <HiOutlineChevronDown class="absolute right-2 top-1/2 -translate-y-1/2 w-4 h-4 text-txt-muted pointer-events-none" />
          </div>
        </Show>
        
        {/* Scan button - shown when path is selected */}
        <Show when={hasPath()}>
          <button 
            class={btnSecondary}
            onClick={props.onScan} 
            disabled={props.busy}
            title="Rescan Directory"
            aria-label="Rescan directory"
          >
            <HiOutlineMagnifyingGlass class="w-4 h-4" />
            <Show when={!compact()}>
              <span>Scan</span>
            </Show>
          </button>
        </Show>
        
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
