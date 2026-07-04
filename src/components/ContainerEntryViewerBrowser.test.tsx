import { describe, expect, it, vi } from "vitest";
import { render } from "solid-js/web";
import { mockInvoke } from "../__tests__/setup";
import { ContainerEntryViewer } from "./ContainerEntryViewer";
import type { SelectedEntry } from "./EvidenceTree";

vi.mock("../utils/platform", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../utils/platform")>()),
  isTauri: false,
}));

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

function makeEntry(overrides: Partial<SelectedEntry> = {}): SelectedEntry {
  return {
    name: "report.pdf",
    containerPath: "/evidence/container.ad1",
    entryPath: "/files/report.pdf",
    size: 1024,
    isDir: false,
    isDiskFile: false,
    isVfsEntry: false,
    isArchiveEntry: false,
    ...overrides,
  };
}

describe("ContainerEntryViewer browser runtime guard", () => {
  it("does not invoke artifact, detection, or extraction commands", async () => {
    const entry = makeEntry();

    const { container, dispose } = renderComponent(() => (
      <ContainerEntryViewer entry={entry} viewMode="preview" />
    ));

    await tick();
    await tick();

    expect(mockInvoke).not.toHaveBeenCalledWith("project_db_is_open");
    expect(mockInvoke).not.toHaveBeenCalledWith("project_db_extract_artifact_source", expect.anything());
    expect(mockInvoke).not.toHaveBeenCalledWith("detect_content_format_source", expect.anything());
    expect(mockInvoke).not.toHaveBeenCalledWith("container_extract_entry_to_temp", expect.anything());
    expect(container.textContent).toContain("report.pdf");
    dispose();
  });
});
