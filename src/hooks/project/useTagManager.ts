// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Tag management functionality for project
 *
 * Manages project-level tag definitions (ProjectTag) and handles
 * write-through to .ffxdb via dbSync. Tag *assignments* to bookmarks
 * and notes are managed by their respective hooks via string arrays;
 * this hook manages the canonical tag registry.
 */

import type { FFXProject, ProjectTag } from "../../types/project";
import { generateId, nowISO } from "../../types/project";
import { logger } from "../../utils/logger";
import type { ProjectStateSignals, ProjectStateSetters, ActivityLogger } from "./types";
import { dbSync } from "./useProjectDbSync";

const log = logger.scope("TagManager");

/** Tag management interface */
export interface TagManager {
  addTag: (tag: Omit<ProjectTag, "id" | "created_at">) => ProjectTag | undefined;
  updateTag: (tagId: string, updates: Partial<Pick<ProjectTag, "name" | "color" | "description">>) => void;
  removeTag: (tagId: string) => void;
}

/**
 * Create tag management functions
 */
export function createTagManager(
  signals: ProjectStateSignals,
  setters: ProjectStateSetters,
  markModified: () => void,
  logger: ActivityLogger,
): TagManager {
  /**
   * Add a new tag to the project tag registry
   */
  const addTag = (tag: Omit<ProjectTag, "id" | "created_at">): ProjectTag | undefined => {
    log.debug(`addTag: name=${tag.name}, color=${tag.color}`);
    const proj = signals.project();
    if (!proj) {
      log.debug("addTag: No project, skipping");
      return undefined;
    }

    // Prevent duplicate tag names
    if (proj.tags.some((t) => t.name.toLowerCase() === tag.name.toLowerCase())) {
      log.warn(`addTag: Tag "${tag.name}" already exists`);
      return undefined;
    }

    const newTag: ProjectTag = {
      ...tag,
      id: generateId(),
      created_at: nowISO(),
    };

    setters.setProject({
      ...proj,
      tags: [...proj.tags, newTag],
    } as FFXProject);

    dbSync.upsertTag(newTag);
    logger.logActivity("tag", "add", `Created tag: ${tag.name}`);
    markModified();
    return newTag;
  };

  /**
   * Update a tag's name, color, or description
   */
  const updateTag = (
    tagId: string,
    updates: Partial<Pick<ProjectTag, "name" | "color" | "description">>,
  ) => {
    log.debug(`updateTag: id=${tagId}, updates=${JSON.stringify(updates)}`);
    const proj = signals.project();
    if (!proj) return;

    const tag = proj.tags.find((t) => t.id === tagId);
    if (!tag) {
      log.warn(`updateTag: Tag not found: ${tagId}`);
      return;
    }

    const updatedTag = { ...tag, ...updates };
    setters.setProject({
      ...proj,
      tags: proj.tags.map((t) => (t.id === tagId ? updatedTag : t)),
    } as FFXProject);

    dbSync.upsertTag(updatedTag);
    logger.logActivity("tag", "update", `Updated tag: ${updates.name || tag.name}`);
    markModified();
  };

  /**
   * Remove a tag from the project registry
   */
  const removeTag = (tagId: string) => {
    log.debug(`removeTag: id=${tagId}`);
    const proj = signals.project();
    if (!proj) return;

    const tag = proj.tags.find((t) => t.id === tagId);
    setters.setProject({
      ...proj,
      tags: proj.tags.filter((t) => t.id !== tagId),
    } as FFXProject);

    dbSync.deleteTag(tagId);
    if (tag) {
      logger.logActivity("tag", "remove", `Removed tag: ${tag.name}`);
    }
    markModified();
  };

  return { addTag, updateTag, removeTag };
}
