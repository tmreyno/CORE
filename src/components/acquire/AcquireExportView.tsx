// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AcquireExportView — Unified acquire & export panel for CORE Acquire edition.
 *
 * Split layout: left panel with drives/sources browser, right panel with
 * export configuration. Sources selected in the left panel are pushed to
 * the ExportPanel via pending source signals.
 */

import {
  Component,
  lazy,
  Suspense,
  Show,
  For,
  createSignal,
  createMemo,
  type Accessor,
} from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import {
  HiOutlineArrowDownTray,
  HiOutlineDocumentPlus,
  HiOutlineFolderPlus,
  HiOutlineXMark,
  HiOutlineComputerDesktop,
} from "../icons";
import SystemInfoPanel from "./SystemInfoPanel";
import { DriveTreeBrowser } from "../export-panel/DriveTreeBrowser";
import { usePanelResize } from "../../hooks/usePanelResize";
import type { Activity } from "../../types/activity";
import type { ExportMode } from "../../hooks/export/types";
import type { SystemStats } from "../../hooks";
import type { DriveInfo } from "../../api/drives";
import { logger } from "../../utils/logger";
import AcquireProcessShell from "./AcquireProcessShell";

const ExportPanel = lazy(() =>
  import("../export-panel").then((m) => ({
    default: m.ExportPanel,
  })),
);

// =============================================================================
// Types
// =============================================================================

export interface AcquireExportViewProps {
  onBack: () => void;
  /** When true, hides the top back-bar (used for inline dashboard expansion) */
  inline?: boolean;
  initialSources: Accessor<string[]>;
  initialExaminerName: Accessor<string | undefined>;
  caseNumber?: Accessor<string | undefined>;
  /** Project name for auto-generating evidence filenames (optional) */
  projectName?: Accessor<string | undefined>;
  initialMode?: Accessor<ExportMode>;
  initialDestination?: string;
  onComplete: (destination: string) => void;
  onActivityCreate: (activity: Activity) => void;
  onActivityUpdate: (id: string, updates: Partial<Activity>) => void;
  // System identification data (for right panel)
  systemStats?: Accessor<SystemStats | null>;
  systemDrives?: Accessor<DriveInfo[]>;
  /** Active triage activity from App-level (survives panel remount) */
  activeTriageActivity?: Accessor<Activity | undefined>;
}

// =============================================================================
// Helpers
// =============================================================================

const basename = (p: string): string => {
  const parts = p.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || p;
};

// =============================================================================
// Component
// =============================================================================

const AcquireExportView: Component<AcquireExportViewProps> = (props) => {
  const log = logger.scope("AcquireExport");
  const mode = createMemo(() => props.initialMode?.() ?? "native");

  // Right panel toggle for system info
  const [showSystemPanel, setShowSystemPanel] = createSignal(false);

  // ── Left panel state ────────────────────────────────────────────────────
  const panel = usePanelResize({
    initialWidth: 288,
    minWidth: 180,
    maxWidth: 500,
    side: "left",
  });
  const [pendingSources, setPendingSources] = createSignal<string[]>([]);
  const [pendingRemovals, setPendingRemovals] = createSignal<string[]>([]);
  const [pendingMode, setPendingMode] = createSignal<ExportMode | null>(null);
  const [selectedPaths, setSelectedPaths] = createSignal<Set<string>>(new Set());
  const [isDragOver, setIsDragOver] = createSignal(false);
  let dragCounter = 0;

  // Add a source from the drive tree browser
  const handleSourceSelect = (path: string) => {
    // Toggle: if already selected, remove it
    const cur = selectedPaths();
    if (cur.has(path)) {
      log.debug(`Source deselected: ${path}`);
      const next = new Set(cur);
      next.delete(path);
      setSelectedPaths(next);
      setPendingRemovals((prev) => [...prev, path]);
      return;
    }
    log.debug(`Source selected: ${path}`);
    const next = new Set(cur);
    next.add(path);
    setSelectedPaths(next);
    setPendingSources((prev) => [...prev, path]);
  };

  // Add files via file dialog
  const handleAddFiles = async () => {
    log.debug("Opening file dialog for source files");
    const result = await open({ multiple: true, title: "Select Files" });
    if (!result) return;
    const paths = Array.isArray(result) ? result : [result];
    for (const p of paths) {
      if (!selectedPaths().has(p as string)) {
        handleSourceSelect(p as string);
      }
    }
  };

  // Add folder via folder dialog
  const handleAddFolder = async () => {
    const result = await open({ directory: true, title: "Select Folder" });
    if (!result) return;
    const p = result as string;
    if (!selectedPaths().has(p)) {
      handleSourceSelect(p);
    }
  };

  // Remove a single queued source
  const handleRemoveQueued = (path: string) => {
    const next = new Set(selectedPaths());
    next.delete(path);
    setSelectedPaths(next);
    setPendingRemovals((prev) => [...prev, path]);
  };

  // Handle acquire/export from context menu — selects source + sets mode
  const handleAcquireSource = (_path: string, acquireMode: ExportMode) => {
    log.info(`Acquire source via context menu: mode=${acquireMode}`);
    // Source is already added by DriveTreeBrowser's onAcquireSource handler
    setPendingMode(acquireMode);
  };

  // --- Drag-and-drop handlers (left panel) ---
  const handleLeftDragEnter = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter++;
    setIsDragOver(true);
  };

  const handleLeftDragOver = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
  };

  const handleLeftDragLeave = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter--;
    if (dragCounter <= 0) {
      dragCounter = 0;
      setIsDragOver(false);
    }
  };

  const handleLeftDrop = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter = 0;
    setIsDragOver(false);

    const dt = e.dataTransfer;
    if (!dt?.files?.length) return;

    for (let i = 0; i < dt.files.length; i++) {
      const file = dt.files[i];
      // @ts-expect-error — path is available in Tauri/Electron runtimes
      const path: string | undefined = file.path;
      if (path && !selectedPaths().has(path)) {
        handleSourceSelect(path);
      }
    }
  };

  return (
    <AcquireProcessShell
      title="Acquire & Export"
      onBack={props.onBack}
      inline={props.inline}
      headerActions={(
        <>
          <button
            class="icon-btn-sm"
            classList={{ "text-accent": showSystemPanel(), "text-txt-muted": !showSystemPanel() }}
            onClick={() => setShowSystemPanel(p => !p)}
            title={showSystemPanel() ? "Hide System Info" : "Show System Info"}
          >
            <HiOutlineComputerDesktop class="w-icon-sm h-icon-sm" />
          </button>
          <button
            class="btn btn-ghost gap-1 text-xs py-1 px-2"
            onClick={() => panel.toggleCollapsed()}
            title={panel.collapsed() ? "Show sources panel" : "Hide sources panel"}
            aria-label={panel.collapsed() ? "Show sources panel" : "Hide sources panel"}
            aria-expanded={!panel.collapsed()}
          >
            Sources
          </button>
        </>
      )}
    >
      <div
        class="flex-1 min-h-0 overflow-hidden flex flex-row"
        classList={{ "is-resizing": panel.isDragging() }}
      >
        {/* Left panel — drives & sources browser */}
        <Show when={!panel.collapsed()}>
          <div
            class="flex flex-col shrink-0 border-r border-border bg-bg overflow-hidden"
            style={{ width: `${panel.width()}px` }}
          >
            {/* Action bar */}
            <div class="flex items-center gap-1 px-2 py-1.5 border-b border-border/50 shrink-0">
              <button
                class="btn-sm gap-1 text-xs flex items-center"
                onClick={handleAddFiles}
                title="Add files"
              >
                <HiOutlineDocumentPlus class="w-icon-sm h-icon-sm" />
                Files
              </button>
              <button
                class="btn-sm gap-1 text-xs flex items-center"
                onClick={handleAddFolder}
                title="Add folder"
              >
                <HiOutlineFolderPlus class="w-icon-sm h-icon-sm" />
                Folder
              </button>
            </div>

            {/* Drive tree — fills available space */}
            <div
              class={`flex-1 min-h-0 flex flex-col overflow-hidden relative transition-colors duration-150 ${
                isDragOver() ? "bg-accent/5" : ""
              }`}
              onDragEnter={handleLeftDragEnter}
              onDragOver={handleLeftDragOver}
              onDragLeave={handleLeftDragLeave}
              onDrop={handleLeftDrop}
            >
              {/* Drop overlay */}
              <Show when={isDragOver()}>
                <div class="absolute inset-0 z-10 flex items-center justify-center pointer-events-none">
                  <div class="p-4 rounded-lg border-2 border-dashed border-accent bg-bg-panel/90 flex items-center gap-2">
                    <HiOutlineArrowDownTray class="w-5 h-5 text-accent" />
                    <span class="text-xs font-medium text-accent">Drop to add sources</span>
                  </div>
                </div>
              </Show>
              <div class="flex-1 min-h-0 overflow-y-auto p-2 flex flex-col">
                <DriveTreeBrowser
                  onSelectSource={handleSourceSelect}
                  onAcquireSource={handleAcquireSource}
                  selectedPaths={selectedPaths}
                  fillHeight
                  initialDrives={props.systemDrives?.()}
                />
              </div>
            </div>

            {/* Queued sources — fixed at bottom */}
            <Show when={selectedPaths().size > 0}>
              <div class="shrink-0 border-t border-border bg-bg-secondary/50 px-2 py-2 space-y-1.5 max-h-44 overflow-y-auto">
                <div class="flex items-center justify-between px-1">
                  <span class="text-2xs font-medium text-txt-muted uppercase tracking-wider">
                    Queued ({selectedPaths().size})
                  </span>
                </div>
                <div class="space-y-0.5">
                  <For each={[...selectedPaths()]}>
                    {(path) => (
                      <div class="flex items-center gap-1.5 py-0.5 px-2 rounded text-xs hover:bg-bg-hover group">
                        <span class="flex-1 truncate text-txt" title={path}>
                          {basename(path)}
                        </span>
                        <button
                          class="icon-btn-sm opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                          onClick={() => handleRemoveQueued(path)}
                          title="Remove"
                          aria-label="Remove source from queue"
                        >
                          <HiOutlineXMark class="w-3 h-3 text-error" />
                        </button>
                      </div>
                    )}
                  </For>
                </div>
              </div>
            </Show>

            {/* Empty state hint */}
            <Show when={selectedPaths().size === 0}>
              <div class="shrink-0 border-t border-border/50 px-3 py-2.5 text-center">
                <p class="text-xs text-txt-muted">Select drives above, or drag & drop files</p>
              </div>
            </Show>
          </div>
        </Show>

        {/* Resize handle */}
        <div
          class="resize-handle"
          classList={{ collapsed: panel.collapsed() }}
          onMouseDown={panel.startDrag}
          onClick={() => panel.collapsed() && panel.setCollapsed(false)}
          onDblClick={panel.toggleCollapsed}
        >
          <Show when={panel.collapsed()}>
            <span class="expand-icon">›</span>
          </Show>
        </div>

        {/* Center panel — export configuration */}
        <div class="flex-1 min-h-0 overflow-hidden flex flex-col">
          <Suspense
            fallback={
              <div class="flex items-center justify-center h-full text-txt-muted text-sm">
                Loading export panel…
              </div>
            }
          >
            <ExportPanel
              initialSources={props.initialSources()}
              initialExaminerName={props.initialExaminerName()}
              caseNumber={props.caseNumber?.()}
              projectName={props.projectName?.()}
              systemStats={props.systemStats?.() ?? null}
              initialMode={mode()}
              initialDestination={props.initialDestination}
              onComplete={props.onComplete}
              onActivityCreate={props.onActivityCreate}
              onActivityUpdate={props.onActivityUpdate}
              pendingDriveSources={pendingSources}
              pendingExportMode={pendingMode}
              onPendingSourcesConsumed={() => {
                setPendingSources([]);
                setPendingMode(null);
              }}
              pendingRemoveSources={pendingRemovals}
              onPendingRemoveConsumed={() => setPendingRemovals([])}
              activeTriageActivity={props.activeTriageActivity}
              hideTriageMode
            />
          </Suspense>
        </div>

        {/* Right panel — system info (toggled via top bar button) */}
        <Show when={showSystemPanel()}>
          <div class="w-72 shrink-0 border-l border-border overflow-hidden">
            <SystemInfoPanel
              systemStats={props.systemStats?.() ?? null}
              drives={props.systemDrives?.()}
            />
          </div>
        </Show>
      </div>
    </AcquireProcessShell>
  );
};

export default AcquireExportView;
