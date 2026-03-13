// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AcquireExportView — File export panel for CORE Acquire edition.
 *
 * Simplified wrapper around the ExportPanel component, focused on
 * native file/folder export (file copy or 7z archive creation).
 */

import {
  Component,
  lazy,
  Suspense,
  type Accessor,
} from "solid-js";
import { HiOutlineArrowLeft } from "../icons";
import type { Activity } from "../../types/activity";

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
  onComplete: (destination: string) => void;
  onActivityCreate: (activity: Activity) => void;
  onActivityUpdate: (id: string, updates: Partial<Activity>) => void;
}

// =============================================================================
// Component
// =============================================================================

const AcquireExportView: Component<AcquireExportViewProps> = (props) => {
  return (
    <div class="acquire-panel">
      <div class="acquire-panel-header">
        <button class="btn btn-ghost gap-1.5" onClick={props.onBack}>
          <HiOutlineArrowLeft class="w-4 h-4" />
          Back
        </button>
        <h2 class="text-lg font-medium text-txt">Export Files</h2>
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
            initialMode="native"
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
