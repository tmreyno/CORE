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
  HiOutlineArrowLeft,
  HiOutlineDocumentPlus,
  HiOutlineFolderPlus,
  HiOutlineXMark,
  HiOutlineChevronDoubleLeft,
  HiOutlineChevronDoubleRight,
} from "../icons";
import { DriveTreeBrowser } from "../export-panel/DriveTreeBrowser";
import type { Activity } from "../../types/activity";
import type { ExportMode } from "../../hooks/export/types";

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
  initialSources: Accessor<string[]>;
  initialExaminerName: Accessor<string | undefined>;
  caseNumber?: Accessor<string | undefined>;
  initialMode?: Accessor<ExportMode>;
  onComplete: (destination: string) => void;
  onActivityCreate: (activity: Activity) => void;
  onActivityUpdate: (id: string, updates: Partial<Activity>) => void;
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
  const mode = createMemo(() => props.initialMode?.() ?? "native");

  // ── Left panel state ────────────────────────────────────────────────────
  const [panelOpen, setPanelOpen] = createSignal(true);
  const [pendingSources, setPendingSources] = createSignal<string[]>([]);
  const [pendingRemovals, setPendingRemovals] = createSignal<string[]>([]);
  const [pendingMode, setPendingMode] = createSignal<ExportMode | null>(null);
  const [selectedPaths, setSelectedPaths] = createSignal<Set<string>>(new Set());

  // Add a source from the drive tree browser
  const handleSourceSelect = (path: string) => {
    // Toggle: if already selected, remove it
    const cur = selectedPaths();
    if (cur.has(path)) {
      const next = new Set(cur);
      next.delete(path);
      setSelectedPaths(next);
      setPendingRemovals((prev) => [...prev, path]);
      return;
    }
    const next = new Set(cur);
    next.add(path);
    setSelectedPaths(next);
    setPendingSources((prev) => [...prev, path]);
  };

  // Add files via file dialog
  const handleAddFiles = async () => {
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
    // Source is already added by DriveTreeBrowser's onAcquireSource handler
    setPendingMode(acquireMode);
  };

  return (
    <div class="flex flex-col flex-1 min-h-0 overflow-hidden">
      {/* ── Top bar ────────────────────────────────────────────────────────── */}
      <div class="flex items-center px-3 py-1.5 border-b border-border bg-bg-secondary shrink-0">
        <button class="btn btn-ghost gap-1 text-xs py-1 px-2" onClick={props.onBack}>
          <HiOutlineArrowLeft class="w-3.5 h-3.5" />
          Dashboard
        </button>
        <button
          class="btn btn-ghost gap-1 text-xs py-1 px-2 ml-auto"
          onClick={() => setPanelOpen((v) => !v)}
          title={panelOpen() ? "Hide sources panel" : "Show sources panel"}
          aria-label={panelOpen() ? "Hide sources panel" : "Show sources panel"}
          aria-expanded={panelOpen()}
        >
          <Show when={panelOpen()} fallback={<HiOutlineChevronDoubleRight class="w-3.5 h-3.5" />}>
            <HiOutlineChevronDoubleLeft class="w-3.5 h-3.5" />
          </Show>
          Sources
        </button>
      </div>

      {/* ── Split layout ───────────────────────────────────────────────────── */}
      <div class="flex-1 min-h-0 overflow-hidden flex flex-row">
        {/* Left panel — drives & sources browser */}
        <Show when={panelOpen()}>
          <div class="flex flex-col w-72 shrink-0 border-r border-border bg-bg overflow-hidden">
            {/* Action bar */}
            <div class="flex items-center gap-1 px-2 py-1.5 border-b border-border/50 shrink-0">
              <button
                class="btn-sm gap-1 text-xs flex items-center"
                onClick={handleAddFiles}
                title="Add files"
              >
                <HiOutlineDocumentPlus class="w-3.5 h-3.5" />
                Files
              </button>
              <button
                class="btn-sm gap-1 text-xs flex items-center"
                onClick={handleAddFolder}
                title="Add folder"
              >
                <HiOutlineFolderPlus class="w-3.5 h-3.5" />
                Folder
              </button>
            </div>

            {/* Drive tree */}
            <div class="flex-1 overflow-y-auto p-2 space-y-3">
              <DriveTreeBrowser
                onSelectSource={handleSourceSelect}
                onAcquireSource={handleAcquireSource}
                selectedPaths={selectedPaths}
              />

              {/* Selected sources queue */}
              <Show when={selectedPaths().size > 0}>
                <div class="space-y-1">
                  <div class="text-2xs font-medium text-txt-muted uppercase tracking-wider px-1">
                    Queued Sources ({selectedPaths().size})
                  </div>
                  <div class="border border-border rounded-lg bg-bg-secondary max-h-48 overflow-y-auto">
                    <div class="p-1 space-y-0.5">
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
                </div>
              </Show>
            </div>
          </div>
        </Show>

        {/* Right panel — export configuration */}
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
              initialMode={mode()}
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
            />
          </Suspense>
        </div>
      </div>
    </div>
  );
};

export default AcquireExportView;
