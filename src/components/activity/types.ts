// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { FFXProject } from "../../types/project";

export interface ActivityPanelProps {
  project: FFXProject | null;
}

export type ActivityFilter = "all" | "project" | "file" | "hash" | "export" | "search" | "bookmark" | "note";
export type SortDirection = "newest" | "oldest";
