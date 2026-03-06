// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Props for the DetailPanel component.
 */

import type { DiscoveredFile, ContainerInfo, TreeEntry, HashHistoryEntry, StoredHash } from "../../types";
import type { HashAlgorithmName } from "../../types/hash";
import type { FileStatus, FileHashInfo } from "../../hooks";
import type { ParsedMetadata } from "../HexViewer";
import type { TabViewMode, OpenTab } from "../TabBar";
import type { BreadcrumbItem } from "../Breadcrumb";

export interface DetailPanelProps {
  activeFile: DiscoveredFile | null;
  fileInfoMap: () => Map<string, ContainerInfo>;
  fileStatusMap: () => Map<string, FileStatus>;
  fileHashMap: () => Map<string, FileHashInfo>;
  hashHistory: () => Map<string, HashHistoryEntry[]>;
  tree: TreeEntry[];
  filteredTree: TreeEntry[];
  treeFilter: string;
  onTreeFilterChange: (filter: string) => void;
  selectedHashAlgorithm: HashAlgorithmName;
  storedHashesGetter: (info: ContainerInfo | undefined) => StoredHash[];
  busy: boolean;
  onLoadInfo: (file: DiscoveredFile) => void;
  formatHashDate: (timestamp: string) => string;
  onTabSelect: (file: DiscoveredFile | null) => void;
  onTabsChange?: (tabs: OpenTab[]) => void;
  onMetadataLoaded?: (metadata: ParsedMetadata | null) => void;
  onViewModeChange?: (mode: TabViewMode) => void;
  onHexNavigatorReady?: (navigateTo: (offset: number, size?: number) => void) => void;
  requestViewMode?: TabViewMode | null;
  onViewModeRequestHandled?: () => void;
  breadcrumbItems?: BreadcrumbItem[];
  onBreadcrumbNavigate?: (path: string) => void;
  scanDir?: string;
  selectedFiles?: DiscoveredFile[];
  onTransferStart?: () => void;
  onHashComputed?: (entries: HashHistoryEntry[]) => void;
}
