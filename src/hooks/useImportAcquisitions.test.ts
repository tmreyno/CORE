import { createRoot } from "solid-js";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { scanForAcquisitions } from "../api/importAcquisitions";
import { dbSync } from "./project/useProjectDbSync";
import { useImportAcquisitions } from "./useImportAcquisitions";
import type { DiscoveredAcquisition } from "../api/importAcquisitions";

vi.mock("../api/importAcquisitions", () => ({
  scanForAcquisitions: vi.fn(),
}));

vi.mock("./project/useProjectDbSync", () => ({
  dbSync: {
    upsertEvidenceFileAsync: vi.fn(),
    insertHashAsync: vi.fn(),
    upsertEvidenceCollectionAsync: vi.fn(),
    upsertCollectedItemAsync: vi.fn(),
  },
}));

const scanForAcquisitionsMock = vi.mocked(scanForAcquisitions);
const dbSyncMock = vi.mocked(dbSync);

const acquisition: DiscoveredAcquisition = {
  companionPath: "/cases/acq.ffx-companion.json",
  outputExists: true,
  outputSize: 1024,
  companion: {
    version: "1",
    tool: "CORE Acquire",
    toolVersion: "1.0.0",
    createdAt: "2026-07-04T12:00:00.000Z",
    acquisitionType: "e01",
    case: {
      caseNumber: "CASE-1",
      evidenceNumber: "EVID-1",
      examiner: "Examiner",
      description: "Test acquisition",
      notes: "Imported by test",
    },
    source: {
      paths: ["/source/disk"],
      totalFiles: 1,
      totalBytes: 1024,
    },
    output: {
      primaryPath: "/cases/disk.E01",
      format: "E01",
      totalBytes: 1024,
    },
    hashes: {
      md5: "md5",
      sha1: "sha1",
      sha256: "sha256",
    },
    timing: {
      startedAt: "2026-07-04T12:00:00.000Z",
      completedAt: "2026-07-04T12:05:00.000Z",
      durationMs: 300000,
    },
    system: {
      hostname: "lab-host",
      username: "examiner",
      sourceDrive: "/dev/disk4",
      sourceFileSystem: "apfs",
      sourceCapacity: 1024,
      sourceDriveType: "disk",
      sourceRemovable: false,
    },
  },
};

describe("useImportAcquisitions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    scanForAcquisitionsMock.mockResolvedValue([acquisition]);
    dbSyncMock.upsertEvidenceFileAsync.mockResolvedValue(undefined);
    dbSyncMock.insertHashAsync.mockResolvedValue(undefined);
    dbSyncMock.upsertEvidenceCollectionAsync.mockResolvedValue(undefined);
    dbSyncMock.upsertCollectedItemAsync.mockResolvedValue(undefined);
  });

  it("awaits durable DB writes before reporting an acquisition as imported", async () => {
    await createRoot(async (dispose) => {
      const importer = useImportAcquisitions();

      await importer.scan("/cases");
      const result = await importer.importSelected(new Set());

      expect(result).toEqual({ imported: 1, skipped: 0, errors: [] });
      expect(dbSyncMock.upsertEvidenceFileAsync).toHaveBeenCalledOnce();
      expect(dbSyncMock.insertHashAsync).toHaveBeenCalledTimes(3);
      expect(dbSyncMock.upsertEvidenceCollectionAsync).toHaveBeenCalledOnce();
      expect(dbSyncMock.upsertCollectedItemAsync).toHaveBeenCalledOnce();
      dispose();
    });
  });

  it("reports DB write failures instead of counting the acquisition as imported", async () => {
    await createRoot(async (dispose) => {
      dbSyncMock.upsertCollectedItemAsync.mockRejectedValueOnce(new Error("DB unavailable"));
      const importer = useImportAcquisitions();

      await importer.scan("/cases");
      const result = await importer.importSelected(new Set());

      expect(result.imported).toBe(0);
      expect(result.errors).toEqual(["/cases/disk.E01: DB unavailable"]);
      dispose();
    });
  });
});
