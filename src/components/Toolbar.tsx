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
  HiOutlineMagnifyingGlass,
  HiOutlineFingerPrint,
  HiOutlineInformationCircle,
  HiOutlineChevronDown,
  HiOutlineDocumentArrowDown,
  HiOutlineDocumentText,
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
  selectedHashAlgorithm: HashAlgorithmName;
  onHashAlgorithmChange: (algorithm: HashAlgorithmName) => void;
  selectedCount: number;
  discoveredCount: number;
  busy: boolean;
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
  
  // Dropdown menu state (save only — open dropdown removed)
  const [showSaveMenu, setShowSaveMenu] = createSignal(false);
  
  // Derived state for better UX feedback
  const hasSelection = () => props.selectedCount > 0;
  const hasEvidence = () => props.discoveredCount > 0;
  const hasPath = () => props.scanDir.length > 0;
  const isModified = () => props.projectModified?.() ?? false;
  
  // Button style classes - organized by visual hierarchy
  const btnBase = "flex items-center justify-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-md transition-all duration-150 disabled:opacity-40 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-accent/50 focus:ring-offset-1 focus:ring-offset-bg";
  const btnSecondary = `${btnBase} bg-bg-secondary text-txt hover:bg-bg-hover active:bg-bg-active border border-border hover:border-border-strong`;
  const btnGhost = `${btnBase} bg-transparent text-txt-secondary hover:text-txt hover:bg-bg-hover`;
  
  // Build project locations for dropdown
  // Only pass scanDir as fallback when a project is actually open,
  // to prevent session-restored paths from populating the dropdown
  const projectLocations = createMemo(() => 
    buildProjectLocations(props.evidencePath, props.processedDbPath, props.caseDocumentsPath, props.projectName?.() ? props.scanDir : undefined)
  );
  
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
      {/* === Save & Location Section === */}
      <div class="flex items-center gap-2">
        {/* Save Button with Dropdown Menu */}
        <div class="relative">
          <button 
            class={`${btnSecondary} ${isModified() ? 'border-warning text-warning' : ''}`}
            onClick={() => setShowSaveMenu(!showSaveMenu())}
            title={isModified() ? "Save Project (unsaved changes)" : "Save Project"}
            aria-label="Save menu"
            aria-expanded={showSaveMenu()}
            aria-haspopup="menu"
          >
            <HiOutlineDocumentArrowDown class="w-4 h-4" />
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
        />
        
        {/* Hash action button with count badge — NOT guarded by busy, so users can queue batches while hashing */}
        <button 
          class={`${btnSecondary} relative ${hasSelection() ? 'pr-8' : ''}`}
          onClick={props.onHashSelected} 
          disabled={!hasSelection()} 
          title={hasSelection() ? `Hash ${props.selectedCount} selected file${props.selectedCount > 1 ? 's' : ''}` : "Select files to hash"}
          aria-label={`Hash ${props.selectedCount} selected files`}
        >
          <HiOutlineFingerPrint class="w-4 h-4" />
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
