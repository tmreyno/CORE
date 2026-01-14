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
import type { ProjectStateSignals, ProjectStateSetters, NoteManager, ActivityLogger } from "./types";

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
    const proj = signals.project();
    if (!proj) return;

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
    
    logger.logActivity('note', 'add', `Added note: ${note.title}`, note.target_path);
    markModified();
  };

  /**
   * Update a note
   */
  const updateNote = (noteId: string, updates: Partial<Pick<ProjectNote, 'title' | 'content' | 'tags' | 'priority'>>) => {
    const proj = signals.project();
    if (!proj) return;

    setters.setProject({
      ...proj,
      notes: proj.notes.map(n =>
        n.id === noteId
          ? { ...n, ...updates, modified_at: nowISO() }
          : n
      ),
    } as FFXProject);
    
    logger.logActivity('note', 'update', `Updated note: ${noteId}`);
    markModified();
  };

  return { addNote, updateNote };
}
