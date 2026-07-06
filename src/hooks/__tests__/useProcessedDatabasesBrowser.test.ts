import { describe, expect, it, vi } from "vitest";
import { createRoot } from "solid-js";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

import { invoke } from "@tauri-apps/api/core";
import { useProcessedDatabases } from "../useProcessedDatabases";
import type { ProcessedDatabase } from "../../types/processed";

const mockInvoke = vi.mocked(invoke);

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
  path: "/case/processed/axiom.db",
  db_type: "MagnetAxiom",
  name: "AXIOM case",
};

describe("useProcessedDatabases browser runtime guards", () => {
  it("does not invoke AXIOM detail backend commands outside Tauri", async () => {
    const { hook, dispose } = createHook();

    await hook.selectDatabase(axiomDb);

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(hook.selectedDatabase()).toBe(axiomDb);
    expect(hook.selectedCaseInfo()).toBeNull();
    expect(hook.selectedCategories()).toEqual([]);
    expect(hook.isSelectedLoading()).toBe(false);
    dispose();
  });
});
