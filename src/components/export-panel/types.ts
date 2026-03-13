// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Activity } from "../../types/activity";
import type { ExportMode } from "../../hooks/export/types";

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
}
