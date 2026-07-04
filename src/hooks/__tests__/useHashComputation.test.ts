// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createRoot, createSignal } from "solid-js";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { mockInvoke, mockListen } from "../../__tests__/setup";
import { useHashComputation } from "../useHashComputation";
import type { ContainerInfo, DiscoveredFile } from "../../types";
import type { SelectedEntry } from "../../components/EvidenceTree/types";
import type { FileHashInfo, HashHistoryEntry } from "../../types/hash";

vi.mock("../../components/preferences", () => ({
  getPreference: vi.fn((key: string) => {
    if (key === "confirmBeforeHash") return false;
    if (key === "copyHashToClipboard") return false;
    return 0;
  }),
}));

vi.mock("../../utils/telemetry", () => ({
  logAuditAction: vi.fn(),
}));

vi.mock("../project/useProjectDbSync", () => ({
  dbSync: {
    upsertEvidenceFile: vi.fn(),
  },
}));

function makeFile(overrides: Partial<DiscoveredFile> = {}): DiscoveredFile {
  return {
    path: "/case/evidence.ad1",
    filename: "evidence.ad1",
    size: 4096,
    container_type: "ad1",
    ...overrides,
  };
}

function makeEntry(overrides: Partial<SelectedEntry> = {}): SelectedEntry {
  return {
    containerPath: "/case/evidence.ad1",
    entryPath: "/Users/alice/report.pdf",
    name: "report.pdf",
    size: 512,
    isDir: false,
    containerType: "ad1",
    ...overrides,
  };
}

describe("useHashComputation.hashEntry", () => {
  beforeEach(() => {
    mockListen.mockResolvedValue(vi.fn());
    mockInvoke.mockImplementation((command: string, args?: unknown) => {
      if (command === "project_db_is_open") return Promise.resolve(true);
      if (command === "project_db_hash_source_and_insert") {
        return Promise.resolve({
          hashResult: {
            sourceRef: {
              kind: "containerEntry",
              containerPath: "/case/evidence.ad1",
              entryPath: "/Users/alice/report.pdf",
              containerType: "ad1",
            },
            sourceId: "ad1:/case/evidence.ad1:/Users/alice/report.pdf",
            containerPath: "/case/evidence.ad1",
            entryPath: "/Users/alice/report.pdf",
            containerType: "ad1",
            algorithm: "SHA-256",
            hash: "A".repeat(64),
            bytesHashed: 512,
            durationMs: 3,
            throughputMbs: 1.5,
          },
          hashRecord: {
            id: "hash-1",
            fileId: "/case/evidence.ad1",
            sourceId: "ad1:/case/evidence.ad1:/Users/alice/report.pdf",
            algorithm: "SHA-256",
            hashValue: "A".repeat(64),
            computedAt: "2026-07-04T00:00:00Z",
            source: "computed",
          },
        });
      }
      return Promise.reject(new Error(`unexpected command ${command}: ${JSON.stringify(args)}`));
    });
  });

  it("hashes selected AD1 entries through the source-aware project DB command", async () => {
    await createRoot(async (dispose) => {
      const [selectedFiles, setSelectedFiles] = createSignal<Set<string>>(new Set());
      const [fileHashMap, setFileHashMap] = createSignal<Map<string, FileHashInfo>>(new Map());
      const computation = useHashComputation({
        discoveredFiles: () => [makeFile()],
        selectedFiles,
        setSelectedFiles,
        fileInfoMap: () => new Map<string, ContainerInfo>(),
        setWorking: vi.fn(),
        setOk: vi.fn(),
        setError: vi.fn(),
        updateFileStatus: vi.fn(),
        updateFileStatusThrottled: vi.fn(),
        loadFileInfo: vi.fn(),
        selectedHashAlgorithm: () => "SHA-256",
        fileHashMap,
        setFileHashMap,
        hashHistory: () => new Map<string, HashHistoryEntry[]>(),
        recordHashToHistory: vi.fn(),
      });

      const hash = await computation.hashEntry(makeEntry(), makeFile());

      expect(hash).toBe("A".repeat(64));
      expect(mockInvoke).toHaveBeenCalledWith("project_db_hash_source_and_insert", {
        request: {
          source: {
            containerPath: "/case/evidence.ad1",
            entryPath: "/Users/alice/report.pdf",
            containerType: "ad1",
            size: 512,
          },
          algorithm: "SHA-256",
          evidenceFile: {
            id: "/case/evidence.ad1",
            path: "/case/evidence.ad1",
            filename: "evidence.ad1",
            containerType: "ad1",
            totalSize: 4096,
            segmentCount: 1,
            discoveredAt: expect.any(String),
            created: null,
            modified: null,
          },
          hashRecordSource: "computed",
        },
      });
      expect(fileHashMap().get("ad1:/case/evidence.ad1:/Users/alice/report.pdf")).toMatchObject({
        algorithm: "SHA-256",
        hash: "A".repeat(64),
        verified: null,
      });

      dispose();
    });
  });
});
