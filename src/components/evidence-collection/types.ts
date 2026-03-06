// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { LinkedDataNode } from "../LinkedDataTree";

export type CollectionStatus = "draft" | "complete" | "locked";

export interface EvidenceCollectionPanelProps {
  caseNumber?: string;
  projectName?: string;
  examinerName?: string;
  /** Open specific collection by ID */
  collectionId?: string;
  /** Open in read-only/review mode */
  readOnly?: boolean;
  /** Called when user closes the tab */
  onClose?: () => void;
  /** Called when user wants to open a different collection */
  onOpenCollection?: (collectionId: string, readOnly: boolean) => void;
  /** Called when linked data nodes change (rendered in right panel) */
  onLinkedNodesChange?: (nodes: LinkedDataNode[]) => void;
}
