// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Note management functionality for project
 */

import type { FFXProject, ProjectNote } from "../../types/project";
import { generateId, nowISO } from "../../types/project";
import { logger } from "../../utils/logger";
import type { ProjectStateSignals, ProjectStateSetters, NoteManager, ActivityLogger } from "./types";
import { dbSync } from "./useProjectDbSync";

const log = logger.scope("Notes");

/**
 * Create note management functions
 */
export function createNoteManager(
  signals: ProjectStateSignals,
  setters: ProjectStateSetters,
  markModified: () => void,
  logger: ActivityLogger
): NoteManager {
  /**
   * Add a note
   */
  const addNote = (note: Omit<ProjectNote, 'id' | 'created_by' | 'created_at' | 'modified_at'>) => {
    log.debug(`addNote called, title="${note.title}", targetPath=${note.target_path}`);
    const proj = signals.project();
    if (!proj) {
      log.debug("No project, skipping");
      return;
    }

    const now = nowISO();
    const newNote: ProjectNote = {
      ...note,
      id: generateId(),
      created_by: signals.currentUser(),
      created_at: now,
      modified_at: now,
    };

    setters.setProject({
      ...proj,
      notes: [...proj.notes, newNote],
    } as FFXProject);
    
    dbSync.upsertNote(newNote);
    logger.logActivity('note', 'add', `Added note: ${note.title}`, note.target_path);
    markModified();
  };

  /**
   * Update a note
   */
  const updateNote = (noteId: string, updates: Partial<Pick<ProjectNote, 'title' | 'content' | 'tags' | 'priority'>>) => {
    log.debug(`updateNote called, noteId=${noteId}, updates=`, Object.keys(updates));
    const proj = signals.project();
    if (!proj) {
      log.debug("No project, skipping update");
      return;
    }

    const existing = proj.notes.find(n => n.id === noteId);
    if (!existing) return;

    const updatedNote = { ...existing, ...updates, modified_at: nowISO() };
    setters.setProject({
      ...proj,
      notes: proj.notes.map(n =>
        n.id === noteId ? updatedNote : n
      ),
    } as FFXProject);
    
    dbSync.upsertNote(updatedNote);
    logger.logActivity('note', 'update', `Updated note: ${noteId}`);
    markModified();
  };

  /**
   * Remove a note by its ID
   */
  const removeNote = (noteId: string) => {
    log.debug(`removeNote called, noteId=${noteId}`);
    const proj = signals.project();
    if (!proj) {
      log.debug("No project, skipping remove");
      return;
    }

    const note = proj.notes.find(n => n.id === noteId);
    const noteTitle = note?.title || noteId;

    setters.setProject({
      ...proj,
      notes: proj.notes.filter(n => n.id !== noteId),
    } as FFXProject);

    dbSync.deleteNote(noteId);
    logger.logActivity('note', 'delete', `Deleted note: ${noteTitle}`, note?.target_path);
    markModified();
  };

  return { addNote, updateNote, removeNote };
}
