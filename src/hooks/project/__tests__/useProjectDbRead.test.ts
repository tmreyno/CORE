// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { beforeEach, describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../../__tests__/setup";
import type { FFXProject } from "../../../types/project";
import type { ProjectDbStats } from "../../../types/projectDb";

const dbSyncMocks = vi.hoisted(() => ({
  batchUpsertEvidenceFiles: vi.fn(),
  insertHash: vi.fn(),
  insertVerification: vi.fn(),
  upsertCaseDocument: vi.fn(),
  upsertBookmark: vi.fn(),
  upsertNote: vi.fn(),
  insertActivity: vi.fn(),
  upsertTag: vi.fn(),
  upsertUser: vi.fn(),
  upsertSession: vi.fn(),
  upsertSavedSearch: vi.fn(),
  insertReport: vi.fn(),
}));

vi.mock("../useProjectDbSync", () => ({
  dbSync: dbSyncMocks,
}));

import { seedDatabaseFromProject } from "../useProjectDbRead";

function makeStats(overrides: Partial<ProjectDbStats> = {}): ProjectDbStats {
  return {
    totalActivities: 0,
    totalSessions: 0,
    totalUsers: 0,
    totalEvidenceFiles: 0,
    totalHashes: 0,
    totalVerifications: 0,
    totalBookmarks: 0,
    totalNotes: 0,
    totalTags: 0,
    totalReports: 0,
    totalSavedSearches: 0,
    totalCaseDocuments: 0,
    totalProcessedDatabases: 0,
    totalAxiomCases: 0,
    totalArtifactCategories: 0,
    totalExports: 0,
    totalCustodyRecords: 0,
    totalClassifications: 0,
    totalExtractions: 0,
    totalViewerHistory: 0,
    totalAnnotations: 0,
    totalRelationships: 0,
    totalCocItems: 0,
    totalCocTransfers: 0,
    totalEvidenceCollections: 0,
    totalCollectedItems: 0,
    dbSizeBytes: 0,
    walExists: false,
    walSizeBytes: 0,
    schemaVersion: 1,
    ...overrides,
  };
}

describe("seedDatabaseFromProject", () => {
  beforeEach(() => {
    for (const mock of Object.values(dbSyncMocks)) {
      mock.mockReset();
    }
    dbSyncMocks.batchUpsertEvidenceFiles.mockResolvedValue(1);
  });

  it("restores cached documents and seeds hashes against existing evidence IDs", async () => {
    const existingEvidencePath = "/case/evidence.E01";
    const legacyOnlyPath = "/case/legacy.AD1";
    const project = {
      evidence_cache: {
        discovered_files: [
          {
            path: existingEvidencePath,
            filename: "evidence.E01",
            container_type: "E01",
            size: 1024,
            segment_count: 1,
          },
        ],
        computed_hashes: {
          [existingEvidencePath]: {
            algorithm: "SHA-256",
            hash: "a".repeat(64),
            verified: true,
            verified_against: "a".repeat(64),
            computed_at: "2026-02-16T10:00:00Z",
          },
        },
        cached_at: "2026-02-16T10:00:00Z",
      },
      hash_history: {
        files: {
          [legacyOnlyPath]: [
            {
              algorithm: "SHA-1",
              hash_value: "b".repeat(40),
              computed_at: "2026-02-16T10:01:00Z",
              verification: {
                result: "match",
                verified_against: "b".repeat(40),
                verified_at: "2026-02-16T10:02:00Z",
              },
            },
          ],
        },
      },
      case_documents_cache: {
        documents: [
          {
            path: "/case/docs/COC.pdf",
            filename: "COC.pdf",
            size: 4096,
            format: "PDF",
            document_type: "chain_of_custody",
            case_number: "CASE-001",
            evidence_id: "EV-001",
            modified: "2026-02-16T09:00:00Z",
          },
        ],
      },
    } as unknown as FFXProject;

    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "project_db_get_stats") {
        return Promise.resolve(makeStats({ totalEvidenceFiles: 1 }));
      }
      if (cmd === "project_db_get_evidence_files") {
        return Promise.resolve([
          {
            id: "existing-db-id",
            path: existingEvidencePath,
            filename: "evidence.E01",
            containerType: "E01",
            totalSize: 1024,
            segmentCount: 1,
            discoveredAt: "2026-02-16T09:00:00Z",
          },
        ]);
      }
      return Promise.reject(new Error(`unexpected command: ${cmd}`));
    });

    await seedDatabaseFromProject(project);

    expect(dbSyncMocks.upsertCaseDocument).toHaveBeenCalledWith(
      project.case_documents_cache!.documents[0],
    );
    expect(dbSyncMocks.batchUpsertEvidenceFiles).toHaveBeenCalledWith([
      expect.objectContaining({
        id: legacyOnlyPath,
        path: legacyOnlyPath,
        filename: "legacy.AD1",
        containerType: "File",
      }),
    ]);
    expect(dbSyncMocks.insertHash).toHaveBeenCalledTimes(2);
    expect(dbSyncMocks.insertHash).toHaveBeenCalledWith(
      expect.objectContaining({
        fileId: "existing-db-id",
        sourceId: existingEvidencePath,
        algorithm: "SHA-256",
        hashValue: "a".repeat(64),
      }),
    );
    expect(dbSyncMocks.insertHash).toHaveBeenCalledWith(
      expect.objectContaining({
        fileId: legacyOnlyPath,
        sourceId: legacyOnlyPath,
        algorithm: "SHA-1",
        hashValue: "b".repeat(40),
      }),
    );
    expect(dbSyncMocks.insertVerification).toHaveBeenCalledTimes(2);
  });
});
