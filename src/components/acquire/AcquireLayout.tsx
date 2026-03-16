// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AcquireLayout — Full replacement layout for the CORE Acquire edition.
 *
 * When acquire edition is active, this replaces the three-panel layout
 * (sidebar + center + right panel) with a focused acquisition UI.
 *
 * Views:
 *   "dashboard"  — 8-card action grid (Physical, Logical, Export, Browse,
 *                   Verify, Collection, Memory, Triage)
 *   "export"     — Unified acquire & export panel (all modes via ExportHeader tabs)
 *   "browse"     — Full evidence browser (renders App.tsx three-panel layout)
 *   "verify"     — Hash verification panel (add files, pick algorithm, compute)
 *   "collection" — Evidence Collection form (inline, not center-pane tab)
 */

import {
  Component,
  Show,
  lazy,
  Suspense,
  createSignal,
  type Accessor,
  type Setter,
} from "solid-js";
import AcquireDashboard, { type AcquireAction } from "./AcquireDashboard";
import AcquireVerifyView from "./AcquireVerifyView";
import { HiOutlineArrowLeft } from "../icons";
import type { ExportMode } from "../../hooks/export/types";
import type { Activity } from "../../types/activity";
import type { PortableConfig } from "../../api/portable";
import type { DiscoveredFile, ContainerInfo } from "../../types";
import "./acquire.css";

const AcquireExportView = lazy(() => import("./AcquireExportView"));
const EvidenceCollectionPanel = lazy(() =>
  import("../EvidenceCollectionPanel").then(m => ({ default: m.EvidenceCollectionPanel }))
);

// =============================================================================
// Types
// =============================================================================

export type AcquireView =
  | "dashboard"
  | "export"
  | "browse"
  | "verify"
  | "collection";

export interface AcquireLayoutProps {
  // ---- Dashboard handlers ----
  onSettings: () => void;
  onHelp: () => void;
  onCommandPalette: () => void;
  onOpenProject: () => void;
  onNewProject: () => void;
  projectName: Accessor<string | undefined>;
  hasProject: Accessor<boolean>;
  evidenceCount: Accessor<number>;

  // ---- Export panel props ----
  initialSources: Accessor<string[]>;
  initialExaminerName: Accessor<string | undefined>;
  onExportComplete: (destination: string) => void;
  onActivityCreate: (activity: Activity) => void;
  onActivityUpdate: (id: string, updates: Partial<Activity>) => void;

  // ---- Evidence collection (inline view) ----
  caseNumber?: Accessor<string | undefined>;
  discoveredFiles?: Accessor<DiscoveredFile[]>;
  fileInfoMap?: Accessor<Map<string, ContainerInfo>>;

  // ---- Verify ----
  onVerifyHashes: () => void;

  // ---- External view/mode control ----
  acquireView: Accessor<AcquireView>;
  setAcquireView: Setter<AcquireView>;

  // ---- Initial export mode when switching to imaging ----
  initialExportMode: Accessor<ExportMode>;
  setInitialExportMode: Setter<ExportMode>;

  // ---- Portable mode ----
  isPortable: () => boolean;
  portableConfig: () => PortableConfig | null;
}

// =============================================================================
// Component
// =============================================================================

const AcquireLayout: Component<AcquireLayoutProps> = (props) => {
  // Pre-filled sources for the export view
  const [prefilledSources] = createSignal<string[] | null>(null);

  // Pre-filled files for the verify view (from dashboard quick-verify)
  const [pendingVerifyFiles, setPendingVerifyFiles] = createSignal<string[] | null>(null);

  const handleAction = (action: AcquireAction) => {
    switch (action) {
      case "physical":
        props.setInitialExportMode("physical");
        props.setAcquireView("export");
        break;
      case "logical":
        props.setInitialExportMode("logical");
        props.setAcquireView("export");
        break;
      case "export":
        props.setInitialExportMode("native");
        props.setAcquireView("export");
        break;
      case "memory":
        props.setInitialExportMode("memory");
        props.setAcquireView("export");
        break;
      case "triage":
        props.setInitialExportMode("triage");
        props.setAcquireView("export");
        break;
      case "browse":
        props.setAcquireView("browse");
        break;
      case "verify":
        props.setAcquireView("verify");
        break;
      case "collection":
        props.setAcquireView("collection");
        break;
    }
  };

  const handleBack = () => {
    props.setAcquireView("dashboard");
  };

  const handleQuickVerify = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: true,
        title: "Select files to verify",
      });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        if (paths.length > 0) {
          setPendingVerifyFiles(paths);
          props.setAcquireView("verify");
        }
      }
    } catch { /* user cancelled */ }
  };

  return (
    <main class="acquire-layout">
      {/* Dashboard view (default) */}
      <Show when={props.acquireView() === "dashboard"}>
        <AcquireDashboard
          onAction={handleAction}
          onSettings={props.onSettings}
          onHelp={props.onHelp}
          onCommandPalette={props.onCommandPalette}
          onOpenProject={props.onOpenProject}
          onNewProject={props.onNewProject}
          projectName={props.projectName}
          hasProject={props.hasProject}
          evidenceCount={props.evidenceCount}
          isPortable={props.isPortable}
          portableConfig={props.portableConfig}
          onQuickVerify={handleQuickVerify}
        />
      </Show>

      {/* Non-dashboard views */}
      <Show when={props.acquireView() !== "dashboard"}>
        <div class="acquire-main-content">
          {/* Unified acquire & export view — Physical (E01), Logical (L01), Native, Memory, Triage */}
          <Show when={props.acquireView() === "export"}>
            <Suspense
              fallback={
                <div class="flex items-center justify-center flex-1 text-txt-muted text-sm">
                  Loading export panel…
                </div>
              }
            >
              <AcquireExportView
                onBack={handleBack}
                initialSources={() => prefilledSources() ?? props.initialSources()}
                initialExaminerName={props.initialExaminerName}
                caseNumber={props.caseNumber}
                initialMode={props.initialExportMode}
                onComplete={props.onExportComplete}
                onActivityCreate={props.onActivityCreate}
                onActivityUpdate={props.onActivityUpdate}
              />
            </Suspense>
          </Show>

          {/* Verify view — Hash verification panel */}
          <Show when={props.acquireView() === "verify"}>
            <AcquireVerifyView
              onBack={handleBack}
              onHashAll={props.onVerifyHashes}
              evidenceCount={props.evidenceCount}
              hasProject={props.hasProject}
              initialFiles={() => pendingVerifyFiles()}
              onInitialFilesConsumed={() => setPendingVerifyFiles(null)}
            />
          </Show>

          {/* Evidence Collection view — inline form */}
          <Show when={props.acquireView() === "collection"}>
            <Suspense
              fallback={
                <div class="flex items-center justify-center flex-1 text-txt-muted text-sm">
                  Loading evidence collection…
                </div>
              }
            >
              <div class="flex flex-col flex-1 min-h-0 overflow-hidden">
                <div class="flex items-center px-3 py-1.5 border-b border-border bg-bg-secondary shrink-0">
                  <button class="btn btn-ghost gap-1 text-xs py-1 px-2" onClick={handleBack}>
                    <HiOutlineArrowLeft class="w-3.5 h-3.5" />
                    Dashboard
                  </button>
                  <span class="text-xs text-txt-muted ml-2">Evidence Collection</span>
                </div>
                <div class="flex-1 min-h-0 overflow-auto">
                  <EvidenceCollectionPanel
                    caseNumber={props.caseNumber?.()}
                    projectName={props.projectName?.()}
                    examinerName={props.initialExaminerName?.()}
                    discoveredFiles={props.discoveredFiles?.() ?? []}
                    fileInfoMap={props.fileInfoMap?.() ?? new Map()}
                    onClose={handleBack}
                  />
                </div>
              </div>
            </Suspense>
          </Show>
        </div>
      </Show>
    </main>
  );
};

export default AcquireLayout;
