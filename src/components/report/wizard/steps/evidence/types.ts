// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Types for the EvidenceStep sub-components.
 */

import type { EvidenceType } from "../../../types";
import type { EvidenceGroup } from "../../types";

export interface EvidenceCardProps {
  group: EvidenceGroup;
  isExpanded: boolean;
  onToggleExpand: (e?: MouseEvent) => void;
  notes: string;
  onNotesChange: (notes: string) => void;
  evidenceType?: EvidenceType;
  onTypeChange: (type: EvidenceType) => void;
}
