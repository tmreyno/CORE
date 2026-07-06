import { createRoot } from "solid-js";
import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../__tests__/setup";
import { useAcquisitionSession } from "./useAcquisitionSession";
import type { AcquisitionSession } from "../../types/acquisitionSession";

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

function createManager() {
  let manager!: ReturnType<typeof useAcquisitionSession>;
  let dispose!: () => void;

  createRoot((d) => {
    dispose = d;
    manager = useAcquisitionSession();
  });

  return { manager, dispose };
}

describe("useAcquisitionSession browser runtime guards", () => {
  it("creates an in-memory acquisition session without native file writes", async () => {
    const { manager, dispose } = createManager();

    await manager.create({
      caseNumber: "1827-1001",
      examiner: "Examiner",
      outputFolder: "/Cases/1827-1001",
      caseName: "Browser Case",
    });

    expect(manager.hasSession()).toBe(true);
    expect(manager.projectName()).toBe("Browser Case");
    expect(manager.sessionPath()).toContain("1827-1001");
    expect(mockInvoke).not.toHaveBeenCalled();
    dispose();
  });

  it("loads a parsed browser session without native file reads", () => {
    const { manager, dispose } = createManager();
    const session: AcquisitionSession = {
      version: "1.0",
      caseNumber: "1827-1001",
      caseName: "Loaded Case",
      examiner: "Examiner",
      organization: "",
      outputFolder: "/Cases/1827-1001",
      evidenceFolder: "",
      sessionFilePath: "1827-1001.acquisition.json",
      createdAt: "2026-07-04T00:00:00.000Z",
      modifiedAt: "2026-07-04T00:00:00.000Z",
      system: {
        hostname: "",
        username: "",
        osName: "",
        osVersion: "",
        systemModel: "",
        systemSerialNumber: "",
        systemManufacturer: "",
        drives: [],
      },
      acquisitions: [],
      collections: [],
      activity: [],
    };

    manager.loadParsed(session, "1827-1001.acquisition.json");

    expect(manager.hasSession()).toBe(true);
    expect(manager.projectName()).toBe("Loaded Case");
    expect(manager.sessionPath()).toBe("1827-1001.acquisition.json");
    expect(mockInvoke).not.toHaveBeenCalled();
    dispose();
  });
});
