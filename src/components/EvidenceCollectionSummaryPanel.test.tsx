import { describe, expect, it, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { save } from "@tauri-apps/plugin-dialog";
import { mockInvoke } from "../__tests__/setup";
import { ToastProvider } from "./Toast";
import { EvidenceCollectionSummaryPanel } from "./EvidenceCollectionSummaryPanel";
import type { DiscoveredFile } from "../types";

const BROWSER_EXPORT_MESSAGE = "Evidence collection export is available in the desktop app.";

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

function buttonByTitle(container: HTMLElement, title: string): HTMLButtonElement {
  const button = container.querySelector(`button[title="${title}"]`);
  expect(button).toBeDefined();
  return button as HTMLButtonElement;
}

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

const activeFile: DiscoveredFile = {
  path: "/evidence/phone.E01",
  filename: "phone.E01",
  container_type: "ewf",
  size: 1024,
};

const collection = {
  id: "collection-1",
  caseNumber: "1827-1001",
  status: "draft",
  collectionDate: "2026-07-04",
  collectingOfficer: "Examiner",
};

const collectedItem = {
  id: "item-1",
  collectionId: "collection-1",
  evidenceFileId: "/evidence/phone.E01",
  itemNumber: "1",
  description: "Phone image",
  itemType: "EWF",
};

async function renderLoadedPanel() {
  mockInvoke.mockImplementation(async (command: string) => {
    if (command === "project_db_get_evidence_collections") return [collection];
    if (command === "project_db_get_all_collected_items") return [collectedItem];
    return undefined;
  });

  const result = renderComponent(() => (
    <ToastProvider>
      <EvidenceCollectionSummaryPanel
        activeFile={() => activeFile}
        hasProject={() => true}
      />
    </ToastProvider>
  ));

  await tick();
  await tick();
  return result;
}

describe("EvidenceCollectionSummaryPanel", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
  });

  it("shows a browser-preview message instead of opening the native save dialog for document export", async () => {
    const { container, dispose } = await renderLoadedPanel();

    buttonByTitle(container, "Save as document").click();
    await tick();

    expect(save).not.toHaveBeenCalled();
    expect(document.body.textContent).toContain(BROWSER_EXPORT_MESSAGE);
    dispose();
  });

  it("shows a browser-preview message instead of opening the native save dialog for CSV export", async () => {
    const { container, dispose } = await renderLoadedPanel();

    buttonByTitle(container, "Export all collections as CSV").click();
    await tick();

    expect(save).not.toHaveBeenCalled();
    expect(document.body.textContent).toContain(BROWSER_EXPORT_MESSAGE);
    dispose();
  });
});
