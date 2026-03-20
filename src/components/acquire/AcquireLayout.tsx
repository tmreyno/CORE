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
import type { ExportMode } from "../../hooks/export/types";
import type { Activity } from "../../types/activity";
import type { PortableConfig } from "../../api/portable";
import type { DiscoveredFile, ContainerInfo } from "../../types";
import type { DriveInfo } from "../../api/drives";
import type { SystemStats } from "../../hooks";
import { logger } from "../../utils/logger";

const AcquireExportView = lazy(() => import("./AcquireExportView"));
const AcquireCollectionView = lazy(() => import("./AcquireCollectionView"));
const AcquireIdentifyView = lazy(() => import("./AcquireIdentifyView"));
const AcquireTriageView = lazy(() => import("./AcquireTriageView"));

// =============================================================================
// Types
// =============================================================================

export type AcquireView =
  | "dashboard"
  | "identify"
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

  // Pre-filled files for the verify view (from dashboard quick-verify)
  const [pendingVerifyFiles, setPendingVerifyFiles] = createSignal<string[] | null>(null);

  // Right panel toggle for the collection view
  const [showSystemPanel, setShowSystemPanel] = createSignal(true);

  // Evidence item folder created by Identify System workflow
  const [evidenceItemFolder, setEvidenceItemFolder] = createSignal<string>("");
  const [activeCollectionId, setActiveCollectionId] = createSignal<string | undefined>(undefined);
  const [activeCollectionReadOnly, setActiveCollectionReadOnly] = createSignal(false);

  // Effective export destination: evidence folder overrides project exports path
  const effectiveDestination = () => evidenceItemFolder() || props.initialDestination || "";

  const openCollection = (collectionId?: string, readOnly = false) => {
    setActiveCollectionId(collectionId);
    setActiveCollectionReadOnly(readOnly);
    props.setAcquireView("collection");
  };

  const handleAction = (action: AcquireAction) => {
    log.info(`Action: ${action}`);
    switch (action) {
      case "physical":
        props.setInitialExportMode("physical");
        props.setAcquireView("export");
        break;
      case "identify":
        props.setAcquireView("identify");
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
        openCollection();
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



  return (
    <main class="flex flex-col flex-1 min-h-0 overflow-hidden bg-bg">
      {/* Dashboard view (default) */}
      <Show when={props.acquireView() === "dashboard"}>
        <AcquireDashboard
          onAction={handleAction}
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
          initialSystemStats={props.systemStatsData()}
          initialDrives={props.systemDrivesData()}
          evidenceItemFolder={evidenceItemFolder}
          initialDestination={props.initialDestination}
          onViewCollection={(_id) => {
            openCollection(_id, true);
          }}
          caseNumber={props.caseNumber}
          examinerName={props.initialExaminerName}
          discoveredFiles={props.discoveredFiles}
          fileInfoMap={props.fileInfoMap}
          onExportComplete={props.onExportComplete}
        />
      </Show>

      {/* Non-dashboard views */}
      <Show when={props.acquireView() !== "dashboard"}>
        <div class="flex flex-col flex-1 min-h-0 overflow-hidden">
          <Show when={props.acquireView() === "identify"}>
            <Suspense
              fallback={
                <div class="flex items-center justify-center flex-1 text-txt-muted text-sm">
                  Loading identify view…
                </div>
              }
            >
              <AcquireIdentifyView
                onBack={handleBack}
                hasProject={props.hasProject}
                projectName={props.projectName}
                currentUsername={props.currentUsername}
                evidenceBasePath={props.evidenceBasePath}
                systemStatsData={props.systemStatsData}
                setSystemStatsData={props.setSystemStatsData}
                systemDrivesData={props.systemDrivesData}
                setSystemDrivesData={props.setSystemDrivesData}
                evidenceItemFolder={evidenceItemFolder}
                setEvidenceItemFolder={setEvidenceItemFolder}
                onOpenCollection={() => openCollection()}
                onOpenBrowse={() => props.setAcquireView("browse")}
              />
            </Suspense>
          </Show>

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
              initialSources={props.initialSources}
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
              <AcquireCollectionView
                onBack={handleBack}
                caseNumber={props.caseNumber}
                projectName={props.projectName}
                examinerName={props.initialExaminerName}
                collectionId={activeCollectionId}
                readOnly={activeCollectionReadOnly}
                discoveredFiles={props.discoveredFiles}
                fileInfoMap={props.fileInfoMap}
                systemDrivesData={props.systemDrivesData}
                systemStatsData={props.systemStatsData}
                evidenceItemFolder={evidenceItemFolder}
                showSystemPanel={showSystemPanel}
                setShowSystemPanel={setShowSystemPanel}
              />
            </Suspense>
          </Show>
        </div>
      </Show>
    </main>
  );
};

export default AcquireLayout;
