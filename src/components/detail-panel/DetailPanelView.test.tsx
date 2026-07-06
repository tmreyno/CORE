import { describe, expect, it } from "vitest";
import { createSignal } from "solid-js";
import { render } from "solid-js/web";
import { DetailPanel } from "./DetailPanelView";
import type { DiscoveredFile } from "../../types";

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

function makeFile(path: string, filename: string, containerType: string): DiscoveredFile {
  return {
    path,
    filename,
    container_type: containerType,
    size: 1024,
    segment_count: 1,
  };
}

describe("DetailPanel", () => {
  it("updates rendered file details when the active evidence file changes", async () => {
    const e01 = makeFile("/evidence/disk.E01", "disk.E01", "EnCase (E01)");
    const tar = makeFile("/evidence/logical.tar", "logical.tar", "TAR (Logical)");
    const [activeFile, setActiveFile] = createSignal<DiscoveredFile | null>(tar);

    const { container, dispose } = renderComponent(() => (
      <DetailPanel
        activeFile={activeFile()}
        fileInfoMap={() => new Map()}
        fileStatusMap={() => new Map()}
        fileHashMap={() => new Map()}
        hashHistory={() => new Map()}
        tree={[]}
        filteredTree={[]}
        treeFilter=""
        onTreeFilterChange={() => {}}
        selectedHashAlgorithm="SHA-256"
        storedHashesGetter={() => []}
        busy={false}
        onLoadInfo={() => {}}
        formatHashDate={(timestamp) => timestamp}
        onTabSelect={() => {}}
      />
    ));

    expect(container.querySelector("h2")?.textContent).toBe("logical.tar");

    setActiveFile(e01);
    await Promise.resolve();

    expect(container.querySelector("h2")?.textContent).toBe("disk.E01");
    dispose();
  });
});
