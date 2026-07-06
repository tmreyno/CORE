import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../__tests__/setup";
import {
  getFilesForSession,
  getHashesForFile,
  getLastSession,
  getLatestHash,
  getOpenTabs,
  getOrCreateSession,
  getRecentSessions,
  getSetting,
  getSettingWithDefault,
  getVerificationsForFile,
  insertHash,
  insertVerification,
  saveOpenTabs,
  setSetting,
  upsertFile,
} from "../useDatabase";
import type { DbFileRecord, DbHashRecord, DbOpenTabRecord, DbVerificationRecord } from "../../types";

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

describe("useDatabase browser runtime guards", () => {
  it("creates a synthetic session and returns empty read models", async () => {
    await expect(getOrCreateSession("/Cases/1827-1001")).resolves.toMatchObject({
      id: "browser-session:/Cases/1827-1001",
      name: "1827-1001",
      root_path: "/Cases/1827-1001",
    });
    await expect(getRecentSessions()).resolves.toEqual([]);
    await expect(getLastSession()).resolves.toBeNull();
    await expect(getFilesForSession("session-1")).resolves.toEqual([]);
    await expect(getHashesForFile("file-1")).resolves.toEqual([]);
    await expect(getLatestHash("file-1", "SHA-256")).resolves.toBeNull();
    await expect(getVerificationsForFile("file-1")).resolves.toEqual([]);
    await expect(getOpenTabs("session-1")).resolves.toEqual([]);

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("no-ops writes without native SQLite commands", async () => {
    const file: DbFileRecord = {
      id: "file-1",
      session_id: "session-1",
      path: "/evidence/case.E01",
      filename: "case.E01",
      container_type: "e01",
      total_size: 1024,
      segment_count: 1,
      discovered_at: "2026-07-04T00:00:00.000Z",
    };
    const hash: DbHashRecord = {
      id: "hash-1",
      file_id: "file-1",
      algorithm: "SHA-256",
      hash_value: "abc",
      computed_at: "2026-07-04T00:00:00.000Z",
      source: "computed",
    };
    const verification: DbVerificationRecord = {
      id: "verify-1",
      hash_id: "hash-1",
      verified_at: "2026-07-04T00:00:00.000Z",
      result: "match",
      expected_hash: "abc",
      actual_hash: "abc",
    };
    const tabs: DbOpenTabRecord[] = [
      {
        id: "tab-1",
        session_id: "session-1",
        file_path: "/evidence/case.E01",
        tab_order: 0,
        is_active: true,
      },
    ];

    await expect(upsertFile(file)).resolves.toBeUndefined();
    await expect(insertHash(hash)).resolves.toBeUndefined();
    await expect(insertVerification(verification)).resolves.toBeUndefined();
    await expect(saveOpenTabs("session-1", tabs)).resolves.toBeUndefined();

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("keeps browser settings in memory", async () => {
    await expect(getSetting("theme")).resolves.toBeNull();
    await expect(getSettingWithDefault("theme", "dark")).resolves.toBe("dark");

    await setSetting("theme", "light");

    await expect(getSetting("theme")).resolves.toBe("light");
    await expect(getSettingWithDefault("theme", "dark")).resolves.toBe("light");
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
