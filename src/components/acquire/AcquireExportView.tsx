// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AcquireExportView — Unified acquire & export panel for CORE Acquire edition.
 *
 * Wraps the ExportPanel component for all acquisition and export modes:
 * physical imaging (E01), logical imaging (L01), and native file export.
 */

import {
  Component,
  lazy,
  Suspense,
  createMemo,
  type Accessor,
} from "solid-js";
import { HiOutlineArrowLeft } from "../icons";
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
  initialMode?: Accessor<ExportMode>;
  onComplete: (destination: string) => void;
  onActivityCreate: (activity: Activity) => void;
  onActivityUpdate: (id: string, updates: Partial<Activity>) => void;
}

// =============================================================================
// Component
// =============================================================================

const AcquireExportView: Component<AcquireExportViewProps> = (props) => {
  const mode = createMemo(() => props.initialMode?.() ?? "native");
  const headerTitle = createMemo(() => {
    switch (mode()) {
      case "physical": return "Acquire Physical Image";
      case "logical": return "Acquire Logical Image";
      default: return "Export Files";
    }
  });

  return (
    <div class="acquire-panel">
      <div class="acquire-panel-header">
        <button class="btn btn-ghost gap-1.5" onClick={props.onBack}>
          <HiOutlineArrowLeft class="w-4 h-4" />
          Back
        </button>
        <h2 class="text-lg font-medium text-txt">{headerTitle()}</h2>
        <div class="w-20" />
      </div>
      <div class="acquire-panel-body">
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
            initialMode={mode()}
            onComplete={props.onComplete}
            onActivityCreate={props.onActivityCreate}
            onActivityUpdate={props.onActivityUpdate}
          />
        </Suspense>
      </div>
    </div>
  );
};

export default AcquireExportView;
