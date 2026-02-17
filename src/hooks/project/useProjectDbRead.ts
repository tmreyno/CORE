// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Database read/seed layer: populates the .ffxdb from the loaded .cffx
 * project state so FTS5 search, querying, and audit trails are available
 * for projects created before .ffxdb support was added.
 *
 * Called once after `project_db_open` succeeds during project load.
 * Uses fire-and-forget semantics — seeding failures are non-fatal.
 */

import { invoke } from "@tauri-apps/api/core";
import { logger } from "../../utils/logger";
import { dbSync } from "./useProjectDbSync";
import type { FFXProject } from "../../types/project";
import type { ProjectDbStats } from "../../types/projectDb";

const log = logger.scope("DbRead");

/**
 * Seed the .ffxdb with data from the loaded .cffx project.
 *
 * Strategy: check the current DB stats first — if tables are already populated,
 * skip seeding to avoid duplicate inserts. This makes the operation idempotent.
 */
export async function seedDatabaseFromProject(project: FFXProject): Promise<void> {
  try {
    // Get current DB stats to decide what needs seeding
    const stats = await invoke<ProjectDbStats>("project_db_get_stats");
    log.info("DB stats before seeding:", stats);

    // Seed bookmarks if DB is empty but project has them
    if (stats.totalBookmarks === 0 && project.bookmarks?.length > 0) {
      log.info(`Seeding ${project.bookmarks.length} bookmarks into .ffxdb`);
      for (const bookmark of project.bookmarks) {
        dbSync.upsertBookmark(bookmark);
      }
    }

    // Seed notes if DB is empty but project has them
    if (stats.totalNotes === 0 && project.notes?.length > 0) {
      log.info(`Seeding ${project.notes.length} notes into .ffxdb`);
      for (const note of project.notes) {
        dbSync.upsertNote(note);
      }
    }

    // Seed activity log if DB is empty but project has entries
    if (stats.totalActivities === 0 && project.activity_log?.length > 0) {
      log.info(`Seeding ${project.activity_log.length} activity entries into .ffxdb`);
      for (const entry of project.activity_log) {
        dbSync.insertActivity(entry);
      }
    }

    // Seed tags if DB is empty but project has them
    if (stats.totalTags === 0 && project.tags?.length > 0) {
      log.info(`Seeding ${project.tags.length} tags into .ffxdb`);
      for (const tag of project.tags) {
        dbSync.upsertTag(tag);
      }
    }

    // Seed sessions if DB is empty but project has them
    if (stats.totalSessions === 0 && project.sessions?.length > 0) {
      log.info(`Seeding ${project.sessions.length} sessions into .ffxdb`);
      for (const session of project.sessions) {
        dbSync.upsertSession({
          sessionId: session.session_id,
          user: session.user,
          startedAt: session.started_at,
          endedAt: session.ended_at ?? undefined,
          durationSeconds: session.duration_seconds,
          hostname: session.hostname,
          appVersion: session.app_version,
          summary: session.summary,
        });
      }
    }

    // Seed saved searches if DB is empty but project has them
    if (stats.totalSavedSearches === 0 && project.saved_searches?.length > 0) {
      log.info(`Seeding ${project.saved_searches.length} saved searches into .ffxdb`);
      for (const search of project.saved_searches) {
        dbSync.upsertSavedSearch(search);
      }
    }

    // Seed reports if DB is empty but project has them
    if (stats.totalReports === 0 && project.reports?.length > 0) {
      log.info(`Seeding ${project.reports.length} reports into .ffxdb`);
      for (const report of project.reports) {
        dbSync.insertReport(report);
      }
    }

    log.info("Database seeding complete");
  } catch (err) {
    log.warn("Database seeding failed (non-fatal):", err);
    // Non-fatal: the .cffx is still the primary data source
  }
}
