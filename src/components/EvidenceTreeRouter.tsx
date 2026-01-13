// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { EvidenceTreeLazy } from "./EvidenceTreeLazy";
import type { SelectedEntry } from "./EvidenceTreeLazy";
import type { DiscoveredFile } from "../types";

/**
 * EvidenceTreeRouter - Unified evidence tree component
 * 
 * Uses EvidenceTreeLazy for ALL container types with consistent UI:
 * - AD1: V2 APIs for 50x faster startup (40ms vs 2000ms)
 * - VFS (E01/Raw/L01): Partition mounting with filesystem navigation
 * - Archives (ZIP/7z/RAR/TAR): Archive tree browsing
 * - UFED: Mobile extraction tree
 * 
 * All containers share:
 * - Standardized ContainerHeader component
 * - Consistent info bar styling
 * - Unified keyboard navigation
 * - Type filter bar for multi-container workspaces
 */

interface EvidenceTreeRouterProps {
  discoveredFiles: DiscoveredFile[];
  activeFile: DiscoveredFile | null;
  busy: boolean;
  onSelectContainer: (file: DiscoveredFile) => void;
  onSelectEntry: (entry: SelectedEntry) => void;
  typeFilter: string | null;
  onToggleTypeFilter: (type: string) => void;
  onClearTypeFilter: () => void;
  containerStats: Record<string, number> | null;
  /** Callback to open a nested container (container inside an archive) */
  onOpenNestedContainer?: (tempPath: string, originalName: string, containerType: string, parentPath: string) => void;
}

export function EvidenceTreeRouter(props: EvidenceTreeRouterProps) {
  return (
    <EvidenceTreeLazy
      discoveredFiles={props.discoveredFiles}
      activeFile={props.activeFile}
      busy={props.busy}
      onSelectContainer={props.onSelectContainer}
      onSelectEntry={props.onSelectEntry}
      typeFilter={props.typeFilter}
      onToggleTypeFilter={props.onToggleTypeFilter}
      onClearTypeFilter={props.onClearTypeFilter}
      containerStats={props.containerStats || {}}
      onOpenNestedContainer={props.onOpenNestedContainer}
    />
  );
}
