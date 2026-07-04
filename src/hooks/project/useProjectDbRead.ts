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
import { generateId } from "../../types/project";
import type { CaseDocument } from "../../types";
import type { FFXProject } from "../../types/project";
import type { DbEvidenceFile, ProjectDbStats } from "../../types/projectDb";
import { buildLocalFileHashSourceFields } from "../../utils/hashSourceIdentity";

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

    // Seed users if DB is empty but project has them
    // Users MUST be seeded before sessions (FK: sessions.user → users.username)
    if (stats.totalUsers === 0 && project.users?.length > 0) {
      log.info(`Seeding ${project.users.length} users into .ffxdb`);
      for (const user of project.users) {
        dbSync.upsertUser({
          username: user.username,
          displayName: user.display_name,
          hostname: user.hostname,
          firstAccess: user.first_access,
          lastAccess: user.last_access,
        });
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

    const evidenceFileIdsByPath = new Map<string, string>();

    // Seed evidence files from cache if DB is empty but cache has them
    const cachedFiles = project.evidence_cache?.discovered_files;
    if (stats.totalEvidenceFiles === 0 && cachedFiles && cachedFiles.length > 0) {
      log.info(`Seeding ${cachedFiles.length} evidence files from cache into .ffxdb`);
      const cachedAt = project.evidence_cache?.cached_at ?? new Date().toISOString();
      await dbSync.batchUpsertEvidenceFiles(cachedFiles.map((file) => {
        evidenceFileIdsByPath.set(file.path, file.path);
        return {
          id: file.path,
          path: file.path,
          filename: file.filename,
          containerType: file.container_type,
          totalSize: file.size,
          segmentCount: file.segment_count ?? 1,
          discoveredAt: cachedAt,
          created: file.created,
          modified: file.modified,
        };
      }));
    } else if (stats.totalEvidenceFiles > 0) {
      try {
        const evidenceFiles = await invoke<DbEvidenceFile[]>("project_db_get_evidence_files");
        for (const file of evidenceFiles) {
          evidenceFileIdsByPath.set(file.path, file.id);
        }
      } catch (err) {
        log.warn("Could not read evidence file IDs before hash seeding:", err);
      }
    }

    // Seed case documents from cache if DB is empty but cache has them.
    const cachedCaseDocuments = project.case_documents_cache?.documents;
    if (
      stats.totalCaseDocuments === 0 &&
      cachedCaseDocuments &&
      cachedCaseDocuments.length > 0
    ) {
      log.info(`Seeding ${cachedCaseDocuments.length} case documents from cache into .ffxdb`);
      for (const doc of cachedCaseDocuments) {
        dbSync.upsertCaseDocument(doc as CaseDocument);
      }
    }

    // Seed hashes from cache/history if DB has no hashes but the project has them.
    const cachedHashes = project.evidence_cache?.computed_hashes;
    const cachedHashEntries = Object.entries(cachedHashes ?? {}).map(([filePath, hash]) => ({
      filePath,
      algorithm: hash.algorithm,
      hashValue: hash.hash,
      computedAt: hash.computed_at ?? project.evidence_cache?.cached_at ?? new Date().toISOString(),
      verified: hash.verified,
      expectedHash: hash.verified_against ?? hash.hash,
      verificationResult: hash.verified ? "match" : "mismatch",
      verifiedAt: undefined,
    }));
    const historyHashEntries = Object.entries(project.hash_history?.files ?? {}).flatMap(
      ([filePath, hashes]) => hashes.map((hash) => ({
        filePath,
        algorithm: hash.algorithm,
        hashValue: hash.hash_value,
        computedAt: hash.computed_at ?? project.evidence_cache?.cached_at ?? new Date().toISOString(),
        verified: hash.verification ? hash.verification.result === "match" : undefined,
        expectedHash: hash.verification?.verified_against,
        verificationResult: hash.verification?.result,
        verifiedAt: hash.verification?.verified_at,
      })),
    );
    const seenHashes = new Set<string>();
    const hashEntries = [...cachedHashEntries, ...historyHashEntries].filter((hash) => {
      const key = `${hash.filePath}\0${hash.algorithm}\0${hash.hashValue}`;
      if (seenHashes.has(key)) return false;
      seenHashes.add(key);
      return true;
    });
    if (stats.totalHashes === 0 && hashEntries.length > 0) {
      log.info(`Seeding ${hashEntries.length} cached hashes into .ffxdb`);

      const missingEvidenceFiles = hashEntries
        .filter((hash) => !evidenceFileIdsByPath.has(hash.filePath))
        .map((hash) => ({
          filePath: hash.filePath,
          id: hash.filePath,
          path: hash.filePath,
          filename: hash.filePath.split(/[\\/]/).pop() || hash.filePath,
          containerType: "File",
          totalSize: 0,
          segmentCount: 1,
          discoveredAt: project.evidence_cache?.cached_at ?? new Date().toISOString(),
          created: undefined,
          modified: undefined,
        }));
      if (missingEvidenceFiles.length > 0) {
        await dbSync.batchUpsertEvidenceFiles(missingEvidenceFiles.map(({ filePath: _filePath, ...file }) => file));
        for (const file of missingEvidenceFiles) {
          evidenceFileIdsByPath.set(file.filePath, file.id);
        }
      }

      for (const hash of hashEntries) {
        const hashRecordId = generateId();
        const fileId = evidenceFileIdsByPath.get(hash.filePath) ?? hash.filePath;
        dbSync.insertHash({
          id: hashRecordId,
          fileId,
          ...buildLocalFileHashSourceFields(hash.filePath),
          algorithm: hash.algorithm,
          hashValue: hash.hashValue,
          computedAt: hash.computedAt,
          source: "cached",
        });

        // If the cached hash has a verification result, seed that too.
        if (hash.verified !== undefined && hash.expectedHash) {
          dbSync.insertVerification({
            id: generateId(),
            hashId: hashRecordId,
            verifiedAt: hash.verifiedAt ?? hash.computedAt ?? new Date().toISOString(),
            result: hash.verificationResult ?? (hash.verified ? "match" : "mismatch"),
            expectedHash: hash.expectedHash,
            actualHash: hash.hashValue,
          });
        }
      }
    }

    log.info("Database seeding complete");
  } catch (err) {
    log.warn("Database seeding failed (non-fatal):", err);
    // Non-fatal: the .cffx is still the primary data source
  }
}
