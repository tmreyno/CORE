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
  type JSX,
} from "solid-js";
import AcquireDashboard, { type AcquireAction } from "./AcquireDashboard";
import AcquireVerifyView from "./AcquireVerifyView";
import { HiOutlineArrowLeft, HiOutlineComputerDesktop } from "../icons";
import type { ExportMode } from "../../hooks/export/types";
import type { Activity } from "../../types/activity";
import type { PortableConfig } from "../../api/portable";
import type { DiscoveredFile, ContainerInfo } from "../../types";
import type { DriveInfo } from "../../api/drives";
import type { SystemStats } from "../../hooks";
import { logger } from "../../utils/logger";
import "./acquire.css";

const AcquireExportView = lazy(() => import("./AcquireExportView"));
const AcquireTriageView = lazy(() => import("./AcquireTriageView"));
const EvidenceCollectionPanel = lazy(() =>
  import("../EvidenceCollectionPanel").then(m => ({ default: m.EvidenceCollectionPanel }))
);
import SystemInfoPanel from "./SystemInfoPanel";

// =============================================================================
// Types
// =============================================================================

export type AcquireView =
  | "dashboard"
  | "export"
  | "browse"
  | "verify"
  | "collection"
  | "triage";

export interface AcquireLayoutProps {
  // ---- Dashboard handlers ----
  onSettings: () => void;
  onHelp: () => void;
  onCommandPalette: () => void;
  onOpenProject: () => void;
  onOpenRecentProject?: (path: string) => void;
  onNewProject: () => void;
  projectName: Accessor<string | undefined>;
  hasProject: Accessor<boolean>;
  evidenceCount: Accessor<number>;

  // ---- Export panel props ----
  initialSources: Accessor<string[]>;
  initialExaminerName: Accessor<string | undefined>;
  initialDestination?: string;
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

  // ---- Lifted system identification state (persists across layout remounts) ----
  systemStatsData: Accessor<SystemStats | null>;
  setSystemStatsData: Setter<SystemStats | null>;
  systemDrivesData: Accessor<DriveInfo[]>;
  setSystemDrivesData: Setter<DriveInfo[]>;

  // ---- Active triage activity (survives view changes) ----
  activeTriageActivity?: Accessor<Activity | undefined>;

  // ---- Evidence item folder (created by Identify System) ----
  evidenceBasePath?: string;
  currentUsername?: string;
}

// =============================================================================
// Component
// =============================================================================

const AcquireLayout: Component<AcquireLayoutProps> = (props) => {
  const log = logger.scope("AcquireLayout");
  // Pre-filled sources for the export view
  const [prefilledSources] = createSignal<string[] | null>(null);

  // Pre-filled files for the verify view (from dashboard quick-verify)
  const [pendingVerifyFiles, setPendingVerifyFiles] = createSignal<string[] | null>(null);

  // Right panel toggle for the collection view
  const [showSystemPanel, setShowSystemPanel] = createSignal(true);

  // Evidence item folder created by Identify System workflow
  const [evidenceItemFolder, setEvidenceItemFolder] = createSignal<string>("");

  // Effective export destination: evidence folder overrides project exports path
  const effectiveDestination = () => evidenceItemFolder() || props.initialDestination || "";

  const handleAction = (action: AcquireAction) => {
    log.info(`Action: ${action}`);
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
        props.setAcquireView("triage");
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
    log.debug("Navigating back to dashboard");
    props.setAcquireView("dashboard");
  };

  const handleQuickVerify = async () => {
    log.debug("Opening quick verify file picker");
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: true,
        title: "Select files to verify",
      });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        if (paths.length > 0) {
          log.info(`Quick verify: ${paths.length} file(s) selected`);
          setPendingVerifyFiles(paths);
          props.setAcquireView("verify");
        }
      }
    } catch { /* user cancelled */ }
  };

  // Map action → inline component for collapsible card expansion
  const EXPORT_ACTION_MODES: Partial<Record<AcquireAction, ExportMode>> = {
    physical: "physical",
    logical: "logical",
    export: "native",
    memory: "memory",
  };

  const renderExpandedContent = (action: AcquireAction, onCollapse: () => void): JSX.Element => {
    const exportMode = EXPORT_ACTION_MODES[action];
    if (exportMode) {
      return (
        <Suspense fallback={<div class="p-4 text-xs text-txt-muted">Loading…</div>}>
          <AcquireExportView
            inline
            onBack={onCollapse}
            initialSources={() => prefilledSources() ?? props.initialSources()}
            initialExaminerName={props.initialExaminerName}
            caseNumber={props.caseNumber}
            projectName={props.projectName}
            initialMode={() => exportMode}
            initialDestination={effectiveDestination()}
            onComplete={props.onExportComplete}
            onActivityCreate={props.onActivityCreate}
            onActivityUpdate={props.onActivityUpdate}
            systemStats={props.systemStatsData}
            systemDrives={props.systemDrivesData}
            activeTriageActivity={props.activeTriageActivity}
          />
        </Suspense>
      );
    }

    if (action === "verify") {
      return (
        <AcquireVerifyView
          inline
          onBack={onCollapse}
          onHashAll={props.onVerifyHashes}
          evidenceCount={props.evidenceCount}
          hasProject={props.hasProject}
          initialFiles={() => pendingVerifyFiles()}
          onInitialFilesConsumed={() => setPendingVerifyFiles(null)}
        />
      );
    }

    if (action === "triage") {
      return (
        <Suspense fallback={<div class="p-4 text-xs text-txt-muted">Loading…</div>}>
          <AcquireTriageView
            inline
            onBack={onCollapse}
            initialDestination={effectiveDestination()}
            onComplete={props.onExportComplete}
            onActivityCreate={props.onActivityCreate}
            onActivityUpdate={props.onActivityUpdate}
            caseNumber={props.caseNumber}
            examinerName={props.initialExaminerName}
            systemStats={props.systemStatsData}
            activeTriageActivity={props.activeTriageActivity}
          />
        </Suspense>
      );
    }

    if (action === "collection") {
      return (
        <Suspense fallback={<div class="p-4 text-xs text-txt-muted">Loading…</div>}>
          <EvidenceCollectionPanel
            caseNumber={props.caseNumber?.()}
            projectName={props.projectName?.()}
            examinerName={props.initialExaminerName?.()}
            discoveredFiles={props.discoveredFiles?.() ?? []}
            fileInfoMap={props.fileInfoMap?.() ?? new Map()}
            systemDrives={props.systemDrivesData()}
            systemStats={props.systemStatsData?.()}
            evidenceItemFolder={evidenceItemFolder()}
            onClose={onCollapse}
          />
        </Suspense>
      );
    }

    return <></>;
  };

  return (
    <main class="acquire-layout">
      {/* Dashboard view (default) */}
      <Show when={props.acquireView() === "dashboard"}>
        <AcquireDashboard
          onAction={handleAction}
          renderExpandedContent={renderExpandedContent}
          onSettings={props.onSettings}
          onHelp={props.onHelp}
          onCommandPalette={props.onCommandPalette}
          onOpenProject={props.onOpenProject}
          onOpenRecentProject={props.onOpenRecentProject}
          onNewProject={props.onNewProject}
          projectName={props.projectName}
          hasProject={props.hasProject}
          evidenceCount={props.evidenceCount}
          isPortable={props.isPortable}
          portableConfig={props.portableConfig}
          onQuickVerify={handleQuickVerify}
          onDrivesLoaded={props.setSystemDrivesData}
          onSystemStatsLoaded={props.setSystemStatsData}
          initialSystemStats={props.systemStatsData()}
          initialDrives={props.systemDrivesData()}
          evidenceBasePath={props.evidenceBasePath}
          onEvidenceItemFolderCreated={setEvidenceItemFolder}
          currentUsername={props.currentUsername}
          onViewCollection={(_id) => {
            // Navigate to collection view
            handleAction("collection");
          }}
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
                projectName={props.projectName}
                initialMode={props.initialExportMode}
              initialDestination={effectiveDestination()}
                onComplete={props.onExportComplete}
                onActivityCreate={props.onActivityCreate}
                onActivityUpdate={props.onActivityUpdate}
                systemStats={props.systemStatsData}
                systemDrives={props.systemDrivesData}
                activeTriageActivity={props.activeTriageActivity}
              />
            </Suspense>
          </Show>

          {/* Triage view — standalone quick triage (no export panel overhead) */}
          <Show when={props.acquireView() === "triage"}>
            <Suspense
              fallback={
                <div class="flex items-center justify-center flex-1 text-txt-muted text-sm">
                  Loading triage…
                </div>
              }
            >
              <AcquireTriageView
                onBack={handleBack}
                initialDestination={effectiveDestination()}
                onComplete={props.onExportComplete}
                onActivityCreate={props.onActivityCreate}
                onActivityUpdate={props.onActivityUpdate}
                caseNumber={props.caseNumber}
                examinerName={props.initialExaminerName}
                systemStats={props.systemStatsData}
                activeTriageActivity={props.activeTriageActivity}
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
                <div class="flex items-center gap-2 px-3 py-1.5 border-b border-border bg-bg-secondary shrink-0">
                  <button class="btn btn-ghost gap-1 text-xs py-1 px-2" onClick={handleBack}>
                    <HiOutlineArrowLeft class="w-icon-sm h-icon-sm" />
                    Dashboard
                  </button>
                  <span class="text-2xs text-txt-muted uppercase tracking-wider font-medium">Evidence Collection</span>
                  <div class="ml-auto">
                    <button
                      class="icon-btn-sm"
                      classList={{ "text-accent": showSystemPanel(), "text-txt-muted": !showSystemPanel() }}
                      onClick={() => setShowSystemPanel(p => !p)}
                      title={showSystemPanel() ? "Hide System Info" : "Show System Info"}
                    >
                      <HiOutlineComputerDesktop class="w-icon-sm h-icon-sm" />
                    </button>
                  </div>
                </div>
                <div class="flex flex-1 min-h-0">
                  {/* Evidence collection form — main content */}
                  <div class="flex-1 min-h-0 overflow-auto">
                    <EvidenceCollectionPanel
                      caseNumber={props.caseNumber?.()}
                      projectName={props.projectName?.()}
                      examinerName={props.initialExaminerName?.()}
                      discoveredFiles={props.discoveredFiles?.() ?? []}
                      fileInfoMap={props.fileInfoMap?.() ?? new Map()}
                      systemDrives={props.systemDrivesData()}
                      systemStats={props.systemStatsData?.()}
                      evidenceItemFolder={evidenceItemFolder()}
                      onClose={handleBack}
                    />
                  </div>
                  {/* System info right panel — toggled via header button */}
                  <Show when={showSystemPanel()}>
                    <div class="w-72 shrink-0 border-l border-border overflow-hidden">
                      <SystemInfoPanel
                        systemStats={props.systemStatsData()}
                        drives={props.systemDrivesData()}
                      />
                    </div>
                  </Show>
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
