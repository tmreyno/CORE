// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { FilterPreset } from "../../hooks/useWorkspaceProfiles";

export interface FilterPresetsDropdownProps {
  /** Available filter presets from profile */
  presets?: FilterPreset[];
  /** Currently active preset ID */
  activePresetId?: string | null;
  /** Handler when preset is selected */
  onSelectPreset?: (preset: FilterPreset | null) => void;
  /** Custom quick filters (in addition to presets) */
  quickFilters?: QuickFilter[];
  /** Show quick filters section */
  showQuickFilters?: boolean;
  /** Compact mode (icon only) */
  compact?: boolean;
  /** Disabled state */
  disabled?: boolean;
}

export interface QuickFilter {
  id: string;
  name: string;
  extensions: string[];
  icon?: string;
}
