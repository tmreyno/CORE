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
import type { ImagingConfig } from "./AcquireImageWizard";
import AcquireVerifyView from "./AcquireVerifyView";
import AcquireProgressView from "./AcquireProgressView";
import AcquireSourcePanel from "./AcquireSourcePanel";
import { HiOutlineArrowLeft } from "../icons";
import type { ExportMode } from "../../hooks/export/types";
import type { Activity } from "../../types/activity";
import type { PortableConfig } from "../../api/portable";
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

  // ---- Portable mode ----
  isPortable: () => boolean;
  portableConfig: () => PortableConfig | null;
}

// =============================================================================
// Component
// =============================================================================

const AcquireLayout: Component<AcquireLayoutProps> = (props) => {
  // Imaging config is set when the wizard finishes — drives the progress view
  const [imagingConfig, setImagingConfig] = createSignal<ImagingConfig | null>(null);

  // Source panel: manually-added image files
  const [imageFiles, setImageFiles] = createSignal<string[]>([]);

  // Pre-filled sources for the wizard (set from source panel or queue)
  const [prefilledSources, setPrefilledSources] = createSignal<string[] | null>(null);

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

  // ── Source panel handlers ───────────────────────────────────────────────

  const handleAcquirePhysical = (sourcePaths: string[]) => {
    setPrefilledSources(sourcePaths);
    props.setInitialExportMode("physical");
    props.setAcquireView("export");
  };

  const handleAcquireLogical = (sourcePaths: string[]) => {
    setPrefilledSources(sourcePaths);
    props.setInitialExportMode("logical");
    props.setAcquireView("export");
  };

  const handleAddImageFiles = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: true,
        title: "Select forensic image files",
        filters: [{
          name: "Evidence Files",
          extensions: ["e01", "E01", "l01", "L01", "ad1", "AD1", "aff", "001", "dd", "raw", "img", "dmg", "iso"],
        }],
      });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        const newPaths = paths.filter(p => !imageFiles().includes(p));
        if (newPaths.length > 0) setImageFiles(prev => [...prev, ...newPaths]);
      }
    } catch { /* user cancelled */ }
  };

  const handleRemoveImageFile = (index: number) => {
    setImageFiles(prev => prev.filter((_, i) => i !== index));
  };

  // Whether the source panel should show (not on dashboard or progress views)
  const showSourcePanel = () => {
    const view = props.acquireView();
    return view !== "dashboard" && view !== "progress";
  };

  return (
    <div class="acquire-layout">
      {/* Dashboard view (default) — no sidebar */}
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
          isPortable={props.isPortable}
          portableConfig={props.portableConfig}
        />
      </Show>

      {/* Progress view — no sidebar (full width for progress bars) */}
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

      {/* All other views render with the source panel sidebar */}
      <Show when={showSourcePanel()}>
        <div class="acquire-with-sidebar">
          <AcquireSourcePanel
            onAcquirePhysical={handleAcquirePhysical}
            onAcquireLogical={handleAcquireLogical}
            imageFiles={imageFiles}
            onAddImageFiles={handleAddImageFiles}
            onRemoveImageFile={handleRemoveImageFile}
          />

          <div class="acquire-main-content">
            {/* Unified acquire & export view — Physical (E01), Logical (L01), and Native export */}
            <Show when={props.acquireView() === "export" || props.acquireView() === "imaging"}>
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
                  initialMode={props.initialExportMode}
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
        </div>
      </Show>
    </div>
  );
};

export default AcquireLayout;
