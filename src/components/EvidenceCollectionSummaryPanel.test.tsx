import { describe, expect, it, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { mockInvoke } from "../__tests__/setup";
import { ToastProvider } from "./Toast";
import { EvidenceCollectionSummaryPanel } from "./EvidenceCollectionSummaryPanel";
import type { DiscoveredFile } from "../types";

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

const activeFile: DiscoveredFile = {
  path: "/evidence/phone.E01",
  filename: "phone.E01",
  container_type: "ewf",
  size: 1024,
};

async function renderLoadedPanel() {
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

  it("does not query project collection tables in browser preview", async () => {
    const { container, dispose } = await renderLoadedPanel();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(container.querySelector(`button[title="Save as document"]`)).toBeNull();
    expect(container.querySelector(`button[title="Export all collections as CSV"]`)).toBeNull();
    dispose();
  });
});
