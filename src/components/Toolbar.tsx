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

import { Show, createMemo, createSignal } from "solid-js";
import type { Accessor } from "solid-js";
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
import { DropdownMenu } from "./toolbar/DropdownMenu";
import type { DropdownMenuElement } from "./toolbar/DropdownMenu";
import { HashAlgorithmSelector } from "./toolbar/HashAlgorithmSelector";
import { ProjectLocationSelector } from "./toolbar/ProjectLocationSelector";
import { buildProjectLocations } from "./toolbar/toolbarHelpers";

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
  /** Called when a location is selected from the dropdown with its type */
  onLocationSelect?: (path: string, locationId: string) => void;
}

export function Toolbar(props: ToolbarProps) {
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
  // Only pass scanDir as fallback when a project is actually open,
  // to prevent session-restored paths from populating the dropdown
  const projectLocations = createMemo(() => 
    buildProjectLocations(props.evidencePath, props.processedDbPath, props.caseDocumentsPath, props.projectName?.() ? props.scanDir : undefined)
  );
  
  // Open menu items
  const openMenuItems = (): DropdownMenuElement[] => [
    {
      id: "open-directory",
      label: "Open Directory",
      icon: HiOutlineFolderOpen,
      onClick: props.onBrowse,
    },
    {
      id: "open-project",
      label: "Open Project",
      icon: HiOutlineDocumentText,
      onClick: props.onOpenProject,
    },
  ];
  
  // Save menu items
  const saveMenuItems = (): DropdownMenuElement[] => {
    const items: DropdownMenuElement[] = [
      {
        id: "save",
        label: "Save",
        icon: HiOutlineDocumentArrowDown,
        shortcut: "⌘S",
        onClick: props.onSave,
      },
      {
        id: "save-as",
        label: "Save As...",
        icon: HiOutlineDocumentText,
        shortcut: "⌘⇧S",
        onClick: props.onSaveAs,
      },
    ];
    
    if (props.onAutoSaveToggle) {
      items.push(
        { type: "divider" },
        {
          id: "auto-save",
          label: "Auto-save",
          icon: props.autoSaveEnabled?.() ? HiOutlineCheck : undefined,
          onClick: props.onAutoSaveToggle,
        }
      );
    }
    
    return items;
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
          
          <DropdownMenu
            isOpen={showOpenMenu()}
            onClose={() => setShowOpenMenu(false)}
            items={openMenuItems()}
          />
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
          
          <DropdownMenu
            isOpen={showSaveMenu()}
            onClose={() => setShowSaveMenu(false)}
            items={saveMenuItems()}
            width="w-52"
          />
        </div>
        
        {/* Project Name Badge */}
        <Show when={props.projectName?.()}>
          <div 
            class="flex items-center gap-1.5 px-2.5 py-1 text-sm font-medium text-accent bg-accent/10 rounded-md border border-accent/20 truncate max-w-[180px]"
            title={`Project: ${props.projectName!()}`}
          >
            <span class="truncate">{props.projectName!()}</span>
            <Show when={isModified()}>
              <span class="w-1.5 h-1.5 rounded-full bg-warning shrink-0" title="Unsaved changes" />
            </Show>
          </div>
        </Show>
        
        {/* Project Location Dropdown or Empty State */}
        <ProjectLocationSelector
          locations={projectLocations()}
          selectedPath={props.scanDir}
          onPathSelect={(path, locationId) => {
            if (locationId && props.onLocationSelect) {
              props.onLocationSelect(path, locationId);
            } else {
              props.onScanDirChange(path);
              props.onScan();
            }
          }}
          compact={compact()}
        />
        
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
        <HashAlgorithmSelector
          selectedAlgorithm={props.selectedHashAlgorithm}
          onAlgorithmChange={props.onHashAlgorithmChange}
          compact={compact()}
          disabled={props.busy}
        />
        
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
