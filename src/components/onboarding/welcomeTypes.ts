// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Accessor, JSX } from "solid-js";

/** Recent project info for the welcome modal */
export interface RecentProjectInfo {
  path: string;
  name: string;
  lastOpened: string;
}

export interface WelcomeModalProps {
  isOpen: boolean;
  onClose: () => void;
  onStartTour: () => void;
  title?: string;
  description?: string | JSX.Element;
  /** Callback to create a new project */
  onNewProject?: () => void;
  /** Callback to open an existing project */
  onOpenProject?: () => void;
  /** Recent projects to display */
  recentProjects?: Accessor<RecentProjectInfo[]>;
  /** Callback when a recent project is selected */
  onSelectRecentProject?: (path: string) => void;
}
