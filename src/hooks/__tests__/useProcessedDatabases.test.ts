import { describe, expect, it, vi } from "vitest";
import { createRoot } from "solid-js";

vi.mock("../../utils/platform", () => ({
  isTauri: true,
}));

import { invoke } from "@tauri-apps/api/core";
import { useProcessedDatabases } from "../useProcessedDatabases";
import type { AxiomCaseInfo, ProcessedDatabase } from "../../types/processed";
import { mockInvoke } from "../../__tests__/setup";

const mockedInvoke = vi.mocked(invoke);

function createHook() {
  let hook!: ReturnType<typeof useProcessedDatabases>;
  let dispose!: () => void;
  createRoot((d) => {
    dispose = d;
    hook = useProcessedDatabases();
  });
  return { hook, dispose };
}

const axiomDb: ProcessedDatabase = {
  path: "/case/processed/axiom.mfdb",
  db_type: "MagnetAxiom",
  name: "AXIOM case",
};

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((res) => {
    resolve = res;
  });
  return { promise, resolve };
}

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("useProcessedDatabases", () => {
  it("ignores stale AXIOM detail responses after clearing databases", async () => {
    const caseInfo = deferred<AxiomCaseInfo>();
    mockedInvoke.mockImplementation((command) => {
      if (command === "get_axiom_case_info") return caseInfo.promise;
      if (command === "get_axiom_artifact_categories") return Promise.resolve([]);
      return Promise.reject(new Error(`Unexpected invoke: ${command}`));
    });

    const { hook, dispose } = createHook();

    hook.addDatabase(axiomDb);
    const selection = hook.selectDatabase(axiomDb);
    expect(hook.isSelectedLoading()).toBe(true);

    hook.clearAll();
    expect(hook.databases()).toEqual([]);
    expect(hook.selectedDatabase()).toBeNull();
    expect(hook.isSelectedLoading()).toBe(false);

    caseInfo.resolve({
      case_name: "Stale AXIOM case",
      total_artifacts: 0,
      evidence_sources: [],
      search_results: [],
    } as AxiomCaseInfo);
    await selection;
    await tick();

    expect(hook.axiomCaseInfo()).toEqual({});
    expect(hook.artifactCategories()).toEqual({});
    expect(hook.selectedCaseInfo()).toBeNull();
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    dispose();
  });
});
