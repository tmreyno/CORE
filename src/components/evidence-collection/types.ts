// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { LinkedDataNode } from "../LinkedDataTree";
import type { DiscoveredFile, ContainerInfo } from "../../types";
import type { DriveInfo } from "../../api/drives";
import type { SystemStats } from "../../hooks";

export type CollectionStatus = "draft" | "complete" | "locked";

export interface EvidenceCollectionPanelProps {
  caseNumber?: string;
  projectName?: string;
  examinerName?: string;
  /** Open specific collection by ID */
  collectionId?: string;
  /** Open in read-only/review mode */
  readOnly?: boolean;
  /** Discovered evidence files from useFileManager */
  discoveredFiles?: DiscoveredFile[];
  /** Container info map from useFileManager (path → ContainerInfo) */
  fileInfoMap?: Map<string, ContainerInfo>;
  /** System drives for auto-filling device identification fields (brand, model, serial) */
  systemDrives?: DriveInfo[];
  /** System stats from Identify phase for auto-filling system identification fields */
  systemStats?: SystemStats | null;
  /** Pre-created evidence item folder path (from Identify phase) */
  evidenceItemFolder?: string;
  /** Called when user closes the tab */
  onClose?: () => void;
  /** Called when user wants to open a different collection */
  onOpenCollection?: (collectionId: string, readOnly: boolean) => void;
  /** Called when linked data nodes change (rendered in right panel) */
  onLinkedNodesChange?: (nodes: LinkedDataNode[]) => void;
}
