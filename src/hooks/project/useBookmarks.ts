// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Bookmark management functionality for project
 */

import type { FFXProject, ProjectBookmark } from "../../types/project";
import { generateId, nowISO } from "../../types/project";
import type { ProjectStateSignals, ProjectStateSetters, BookmarkManager, ActivityLogger } from "./types";

/**
 * Create bookmark management functions
 */
export function createBookmarkManager(
  signals: ProjectStateSignals,
  setters: ProjectStateSetters,
  markModified: () => void,
  logger: ActivityLogger
): BookmarkManager {
  /**
   * Add a bookmark
   */
  const addBookmark = (bookmark: Omit<ProjectBookmark, 'id' | 'created_by' | 'created_at'>) => {
    const proj = signals.project();
    if (!proj) return;

    const newBookmark: ProjectBookmark = {
      ...bookmark,
      id: generateId(),
      created_by: signals.currentUser(),
      created_at: nowISO(),
    };

    setters.setProject({
      ...proj,
      bookmarks: [...proj.bookmarks, newBookmark],
    } as FFXProject);
    
    logger.logActivity('bookmark', 'add', `Added bookmark: ${bookmark.name}`, bookmark.target_path);
    markModified();
  };

  /**
   * Remove a bookmark
   */
  const removeBookmark = (bookmarkId: string) => {
    const proj = signals.project();
    if (!proj) return;

    const bookmark = proj.bookmarks.find(b => b.id === bookmarkId);
    setters.setProject({
      ...proj,
      bookmarks: proj.bookmarks.filter(b => b.id !== bookmarkId),
    } as FFXProject);
    
    if (bookmark) {
      logger.logActivity('bookmark', 'remove', `Removed bookmark: ${bookmark.name}`, bookmark.target_path);
    }
    markModified();
  };

  return { addBookmark, removeBookmark };
}
