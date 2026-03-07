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
import { logger } from "../../utils/logger";
import type { ProjectStateSignals, ProjectStateSetters, BookmarkManager, ActivityLogger } from "./types";
import { dbSync } from "./useProjectDbSync";

const log = logger.scope("Bookmarks");

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
    log.debug(`addBookmark: name=${bookmark.name}, path=${bookmark.target_path}`);
    const proj = signals.project();
    if (!proj) {
      log.debug("addBookmark: No project, skipping");
      return;
    }

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
    
    dbSync.upsertBookmark(newBookmark);
    logger.logActivity('bookmark', 'add', `Added bookmark: ${bookmark.name}`, bookmark.target_path);
    log.debug("addBookmark: Calling markModified...");
    markModified();
  };

  /**
   * Update a bookmark's name, color, tags, or notes
   */
  const updateBookmark = (bookmarkId: string, updates: Partial<Pick<ProjectBookmark, 'name' | 'color' | 'tags' | 'notes'>>) => {
    log.debug(`updateBookmark: id=${bookmarkId}, updates=${JSON.stringify(updates)}`);
    const proj = signals.project();
    if (!proj) return;

    const bookmark = proj.bookmarks.find(b => b.id === bookmarkId);
    if (!bookmark) {
      log.warn(`updateBookmark: Bookmark not found: ${bookmarkId}`);
      return;
    }

    const updatedBookmark = { ...bookmark, ...updates };
    setters.setProject({
      ...proj,
      bookmarks: proj.bookmarks.map(b =>
        b.id === bookmarkId ? updatedBookmark : b
      ),
    } as FFXProject);

    dbSync.upsertBookmark(updatedBookmark);
    logger.logActivity('bookmark', 'update', `Updated bookmark: ${updates.name || bookmark.name}`, bookmark.target_path);
    markModified();
  };

  /**
   * Remove a bookmark
   */
  const removeBookmark = (bookmarkId: string) => {
    log.debug(`removeBookmark: id=${bookmarkId}`);
    const proj = signals.project();
    if (!proj) return;

    const bookmark = proj.bookmarks.find(b => b.id === bookmarkId);
    setters.setProject({
      ...proj,
      bookmarks: proj.bookmarks.filter(b => b.id !== bookmarkId),
    } as FFXProject);
    
    dbSync.deleteBookmark(bookmarkId);
    if (bookmark) {
      logger.logActivity('bookmark', 'remove', `Removed bookmark: ${bookmark.name}`, bookmark.target_path);
    }
    markModified();
  };

  /**
   * Clear all bookmarks
   */
  const clearBookmarks = () => {
    log.debug("clearBookmarks: Removing all bookmarks");
    const proj = signals.project();
    if (!proj) return;

    const count = proj.bookmarks.length;
    if (count === 0) return;

    // Delete each from DB
    for (const bookmark of proj.bookmarks) {
      dbSync.deleteBookmark(bookmark.id);
    }

    setters.setProject({
      ...proj,
      bookmarks: [],
    } as FFXProject);

    logger.logActivity('bookmark', 'remove', `Cleared all bookmarks (${count})`);
    markModified();
  };

  return { addBookmark, updateBookmark, removeBookmark, clearBookmarks };
}
