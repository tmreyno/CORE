// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { beforeEach, describe, expect, it, vi } from "vitest";
import { createSignal } from "solid-js";
import { render } from "solid-js/web";
import { mockInvoke } from "../__tests__/setup";
import type { DbNormalizedArtifact } from "../api/commands";
import type { DiscoveredFile } from "../types";
import { notifyArtifactsUpdated } from "../utils/artifactEvents";
import { ToastProvider } from "./Toast";

vi.mock("../utils/platform", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../utils/platform")>()),
  isTauri: true,
}));

const { SystemIdentitySummaryPanel } = await import("./SystemIdentitySummaryPanel");

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

const activeFile: DiscoveredFile = {
  path: "/evidence/disk.E01",
  filename: "disk.E01",
  container_type: "ewf",
  size: 1024,
};

function artifact(overrides: Partial<DbNormalizedArtifact> = {}): DbNormalizedArtifact {
  return {
    id: "artifact-1",
    evidenceFileId: activeFile.path,
    sourceId: `e01:${activeFile.path}:/Windows/System32/config/SOFTWARE`,
    sourceRefJson: JSON.stringify({
      kind: "vfsEntry",
      containerPath: activeFile.path,
      entryPath: "/Windows/System32/config/SOFTWARE",
      containerType: "e01",
    }),
    name: "SOFTWARE",
    extension: null,
    size: 65536,
    mimeType: null,
    typeDescription: "System Identification Artifact",
    category: "systeminfo",
    confidence: "high",
    isText: false,
    contentPreview: null,
    metadataJson: JSON.stringify({
      "system.machineGuid": "6f2d5a21-24e0-47cc-b9b2-7dc8c763f9c3",
      "system.osName": "Windows 11 Pro",
    }),
    extractedAt: "2026-07-06T10:00:00Z",
    extractor: "test-system-identity",
    ...overrides,
  };
}

describe("SystemIdentitySummaryPanel desktop mode", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
  });

  it("loads selected evidence artifacts directly before using category fallback", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "project_db_list_artifacts_for_evidence") return [artifact()];
      if (cmd === "project_db_list_artifacts_by_category") {
        throw new Error("category fallback should not run when direct evidence records exist");
      }
      return null;
    });

    const { container, dispose } = renderComponent(() => (
      <ToastProvider>
        <SystemIdentitySummaryPanel
          activeFile={() => activeFile}
          hasProject={() => true}
        />
      </ToastProvider>
    ));

    await tick();
    await tick();

    expect(mockInvoke).toHaveBeenCalledWith("project_db_list_artifacts_for_evidence", {
      evidenceFileId: activeFile.path,
    });
    expect(container.textContent).toContain("Machine GUID");
    expect(container.textContent).toContain("6f2d5a21-24e0-47cc-b9b2-7dc8c763f9c3");
    expect(
      mockInvoke.mock.calls.some(([cmd]) => cmd === "project_db_list_artifacts_by_category"),
    ).toBe(false);
    dispose();
  });

  it("ignores stale artifact results after selected evidence changes", async () => {
    const secondFile: DiscoveredFile = {
      ...activeFile,
      path: "/evidence/second.E01",
      filename: "second.E01",
    };
    let resolveFirst:
      | ((value: DbNormalizedArtifact[]) => void)
      | undefined;

    mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "project_db_list_artifacts_for_evidence") {
        if (args?.evidenceFileId === activeFile.path) {
          return new Promise((resolve) => {
            resolveFirst = resolve;
          });
        }
        if (args?.evidenceFileId === secondFile.path) {
          return Promise.resolve([
            artifact({
              id: "artifact-2",
              evidenceFileId: secondFile.path,
              sourceId: `e01:${secondFile.path}:/etc/os-release`,
              sourceRefJson: JSON.stringify({
                kind: "vfsEntry",
                containerPath: secondFile.path,
                entryPath: "/etc/os-release",
                containerType: "e01",
              }),
              metadataJson: JSON.stringify({
                "system.osName": "Second OS",
                "system.machineId": "second-machine-id",
              }),
            }),
          ]);
        }
      }
      if (cmd === "project_db_list_artifacts_by_category") return Promise.resolve([]);
      return Promise.resolve(null);
    });

    const [file, setFile] = createSignal(activeFile);
    const { container, dispose } = renderComponent(() => (
      <ToastProvider>
        <SystemIdentitySummaryPanel
          activeFile={file}
          hasProject={() => true}
        />
      </ToastProvider>
    ));

    await tick();
    setFile(secondFile);
    await tick();
    await tick();

    resolveFirst?.([
      artifact({
        metadataJson: JSON.stringify({
          "system.machineGuid": "stale-machine-guid",
          "system.osName": "Stale OS",
        }),
      }),
    ]);
    await tick();
    await tick();

    expect(container.textContent).toContain("Second OS");
    expect(container.textContent).toContain("second-machine-id");
    expect(container.textContent).not.toContain("stale-machine-guid");
    expect(container.textContent).not.toContain("Stale OS");
    dispose();
  });

  it("reloads when system identity artifacts update for the active evidence", async () => {
    let calls = 0;
    mockInvoke.mockImplementation(async (cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "project_db_list_artifacts_for_evidence") {
        calls += 1;
        if (args?.evidenceFileId !== activeFile.path) return [];
        return calls <= 2
          ? []
          : [
              artifact({
                metadataJson: JSON.stringify({
                  "system.osName": "Refreshed OS",
                  "system.machineId": "refreshed-machine-id",
                }),
              }),
            ];
      }
      if (cmd === "project_db_list_artifacts_by_category") return [];
      return null;
    });

    const { container, dispose } = renderComponent(() => (
      <ToastProvider>
        <SystemIdentitySummaryPanel
          activeFile={() => activeFile}
          hasProject={() => true}
        />
      </ToastProvider>
    ));

    await tick();
    await tick();
    expect(container.textContent).toContain("No system identity artifacts found");

    notifyArtifactsUpdated({
      evidenceFileId: "/evidence/other.E01",
      category: "systeminfo",
    });
    await tick();
    await tick();
    expect(calls).toBe(2);

    notifyArtifactsUpdated({
      evidenceFileId: activeFile.path,
      category: "systeminfo",
    });
    await tick();
    await tick();

    expect(calls).toBe(3);
    expect(container.textContent).toContain("Refreshed OS");
    expect(container.textContent).toContain("refreshed-machine-id");
    dispose();
  });
});
