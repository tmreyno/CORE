// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component, Accessor } from "solid-js";
import type { FFXProject } from "../../types/project";
import type { DiscoveredFile } from "../../types";
import type { FileHashInfo } from "../../hooks";

export interface ProjectDashboardProps {
  project: Accessor<FFXProject | null>;
  discoveredFiles: Accessor<DiscoveredFile[]>;
  fileHashMap: Accessor<Map<string, FileHashInfo>>;
  bookmarkCount: Accessor<number>;
  noteCount: Accessor<number>;
  onNavigateTab?: (tab: string) => void;
}

export interface StatCardProps {
  icon: Component<{ class?: string }>;
  label: string;
  value: number | string;
  onClick?: () => void;
  accent?: boolean;
}
