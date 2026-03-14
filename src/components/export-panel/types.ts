// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Activity } from "../../types/activity";
import type { ExportMode } from "../../hooks/export/types";
import type { Accessor } from "solid-js";

/** Export panel props */
export interface ExportPanelProps {
  /** Pre-selected source files (optional) */
  initialSources?: string[];
  /** Pre-fill examiner name from project owner (optional) */
  initialExaminerName?: string;
  /** Initial export mode (physical/logical/native/tools). Defaults to "native". */
  initialMode?: ExportMode;
  /** Callback when export completes */
  onComplete?: (destination: string) => void;
  /** Callback when panel is closed */
  onClose?: () => void;
  /** Callback when an activity is created */
  onActivityCreate?: (activity: Activity) => void;
  /** Callback when an activity is updated */
  onActivityUpdate?: (id: string, updates: Partial<Activity>) => void;
  /** Reactive signal of sources to add from the Drives panel */
  pendingDriveSources?: Accessor<string[]>;
  /** Reactive signal of export mode to switch to (from Drives panel context menu) */
  pendingExportMode?: Accessor<ExportMode | null>;
  /** Called after pending sources/mode have been consumed */
  onPendingSourcesConsumed?: () => void;
}
