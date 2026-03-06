// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Activity } from "../../types/activity";

export interface SimpleActivityPanelProps {
  activities: Activity[];
  onCancel?: (id: string) => void;
  onClear?: (id: string) => void;
  onPause?: (id: string) => void;
  onResume?: (id: string) => void;
}

export interface ActivityCardProps {
  activity: Activity;
  onCancel?: (id: string) => void;
  onClear?: (id: string) => void;
  onPause?: (id: string) => void;
  onResume?: (id: string) => void;
  onOpenDestination: (path: string) => void;
}
