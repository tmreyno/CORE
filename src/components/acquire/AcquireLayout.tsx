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
 * States:
 *   "dashboard"  — Action card grid (default)
 *   "imaging"    — AcquireImageWizard (E01/L01 step-by-step wizard)
 *   "export"     — AcquireExportView (native file export via ExportPanel)
 *   "browse"     — Full evidence browser (existing three-panel layout)
 *   "verify"     — AcquireVerifyView (hash verification)
 *   "collection" — Evidence Collection (center-pane tab)
 */

import {
  Component,
  Show,
  lazy,
  Suspense,
  createSignal,
  type JSXElement,
  type Accessor,
  type Setter,
} from "solid-js";
import AcquireDashboard, { type AcquireAction } from "./AcquireDashboard";
import AcquireImageWizard, { type ImagingConfig } from "./AcquireImageWizard";
import AcquireVerifyView from "./AcquireVerifyView";
import AcquireProgressView from "./AcquireProgressView";
import { HiOutlineArrowLeft } from "../icons";
import type { ExportMode } from "../../hooks/export/types";
import type { Activity } from "../../types/activity";
import "./acquire.css";

const AcquireExportView = lazy(() => import("./AcquireExportView"));

// =============================================================================
// Types
// =============================================================================

export type AcquireView =
  | "dashboard"
  | "imaging"
  | "progress"
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
  onBookmarks: () => void;
  onSearch: () => void;
  projectName: Accessor<string | undefined>;
  hasProject: Accessor<boolean>;
  evidenceCount: Accessor<number>;

  // ---- Export panel props ----
  initialSources: Accessor<string[]>;
  initialExaminerName: Accessor<string | undefined>;
  onExportComplete: (destination: string) => void;
  onActivityCreate: (activity: Activity) => void;
  onActivityUpdate: (id: string, updates: Partial<Activity>) => void;

  // ---- Browse mode — renders the existing three-panel layout ----
  browseContent: JSXElement;

  // ---- Evidence collection ----
  onEvidenceCollection: () => void;

  // ---- Verify ----
  onVerifyHashes: () => void;

  // ---- External view/mode control ----
  acquireView: Accessor<AcquireView>;
  setAcquireView: Setter<AcquireView>;

  // ---- Initial export mode when switching to imaging ----
  initialExportMode: Accessor<ExportMode>;
  setInitialExportMode: Setter<ExportMode>;
}

// =============================================================================
// Component
// =============================================================================

const AcquireLayout: Component<AcquireLayoutProps> = (props) => {
  // Imaging config is set when the wizard finishes — drives the progress view
  const [imagingConfig, setImagingConfig] = createSignal<ImagingConfig | null>(null);

  const handleAction = (action: AcquireAction) => {
    switch (action) {
      case "physical":
        props.setInitialExportMode("physical");
        props.setAcquireView("imaging");
        break;
      case "logical":
        props.setInitialExportMode("logical");
        props.setAcquireView("imaging");
        break;
      case "export":
        props.setAcquireView("export");
        break;
      case "browse":
        props.setAcquireView("browse");
        break;
      case "verify":
        props.setAcquireView("verify");
        break;
      case "collection":
        props.onEvidenceCollection();
        break;
    }
  };

  const handleBack = () => {
    props.setAcquireView("dashboard");
  };

  return (
    <div class="acquire-layout">
      {/* Dashboard view (default) */}
      <Show when={props.acquireView() === "dashboard"}>
        <AcquireDashboard
          onAction={handleAction}
          onSettings={props.onSettings}
          onHelp={props.onHelp}
          onCommandPalette={props.onCommandPalette}
          onOpenProject={props.onOpenProject}
          onNewProject={props.onNewProject}
          onBookmarks={props.onBookmarks}
          onSearch={props.onSearch}
          projectName={props.projectName}
          hasProject={props.hasProject}
          evidenceCount={props.evidenceCount}
        />
      </Show>

      {/* Imaging view — Step-by-step wizard for E01/L01 */}
      <Show when={props.acquireView() === "imaging"}>
        <AcquireImageWizard
          mode={props.initialExportMode}
          onBack={handleBack}
          onStartImaging={(config) => {
            setImagingConfig(config);
            props.setAcquireView("progress");
          }}
        />
      </Show>

      {/* Progress view — Real-time imaging progress with cancel */}
      <Show when={props.acquireView() === "progress" && imagingConfig()}>
        <AcquireProgressView
          config={imagingConfig()!}
          onDone={handleBack}
          onNewImage={() => {
            setImagingConfig(null);
            props.setAcquireView("imaging");
          }}
        />
      </Show>

      {/* Export view — Native file export (7z / file copy) */}
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
            onComplete={props.onExportComplete}
            onActivityCreate={props.onActivityCreate}
            onActivityUpdate={props.onActivityUpdate}
          />
        </Suspense>
      </Show>

      {/* Browse view — renders the existing three-panel layout passed as children */}
      <Show when={props.acquireView() === "browse"}>
        <div class="acquire-panel">
          <div class="acquire-panel-header">
            <button class="btn btn-ghost gap-1.5" onClick={handleBack}>
              <HiOutlineArrowLeft class="w-4 h-4" />
              Back
            </button>
            <h2 class="text-lg font-medium text-txt">Browse Evidence</h2>
            <div class="w-20" />
          </div>
          <div class="acquire-panel-body">
            {props.browseContent}
          </div>
        </div>
      </Show>

      {/* Verify view — Hash verification panel */}
      <Show when={props.acquireView() === "verify"}>
        <AcquireVerifyView
          onBack={handleBack}
          onHashAll={props.onVerifyHashes}
          evidenceCount={props.evidenceCount}
          hasProject={props.hasProject}
        />
      </Show>
    </div>
  );
};

export default AcquireLayout;
