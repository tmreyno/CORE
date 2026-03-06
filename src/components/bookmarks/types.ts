// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { ProjectBookmark } from "../../types/project";

// =============================================================================
// Props interfaces
// =============================================================================

export interface BookmarksPanelProps {
  /** Bookmarks from the project */
  bookmarks: ProjectBookmark[];
  /** Handler to navigate to a bookmarked item */
  onNavigate?: (bookmark: ProjectBookmark) => void;
  /** Handler to remove a bookmark */
  onRemove?: (bookmarkId: string) => void;
  /** Handler to edit a bookmark (legacy - opens inline editor) */
  onEdit?: (bookmark: ProjectBookmark) => void;
  /** Handler to update a bookmark's properties */
  onUpdate?: (bookmarkId: string, updates: Partial<Pick<ProjectBookmark, "name" | "color" | "tags" | "notes">>) => void;
  /** Loading state */
  loading?: boolean;
  /** Whether the panel is compact (sidebar mode) */
  compact?: boolean;
}

export interface BookmarkItemProps {
  bookmark: ProjectBookmark;
  onNavigate?: (bookmark: ProjectBookmark) => void;
  onRemove?: (bookmarkId: string) => void;
  onEdit?: (bookmark: ProjectBookmark) => void;
  compact?: boolean;
}

export interface BookmarkGroupProps {
  type: BookmarkTargetType;
  bookmarks: ProjectBookmark[];
  onNavigate?: (bookmark: ProjectBookmark) => void;
  onRemove?: (bookmarkId: string) => void;
  onEdit?: (bookmark: ProjectBookmark) => void;
  compact?: boolean;
  defaultExpanded?: boolean;
}

export interface BookmarkEditDialogProps {
  bookmark: ProjectBookmark;
  onSave: (bookmarkId: string, updates: Partial<Pick<ProjectBookmark, "name" | "color" | "tags" | "notes">>) => void;
  onCancel: () => void;
}

export type BookmarkTargetType = ProjectBookmark["target_type"];
