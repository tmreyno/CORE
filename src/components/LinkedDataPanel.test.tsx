import { describe, expect, it, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { save } from "@tauri-apps/plugin-dialog";
import { LinkedDataPanel } from "./LinkedDataPanel";
import type { LinkedDataNode } from "./LinkedDataTree";

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

function toolbarButton(container: HTMLElement, title: string): HTMLButtonElement {
  const button = container.querySelector(`button[title="${title}"]`);
  expect(button).toBeDefined();
  return button as HTMLButtonElement;
}

const nodes: LinkedDataNode[] = [
  {
    id: "collection-1",
    label: "Collection 1",
    sublabel: "Case evidence",
    type: "collection",
    children: [
      {
        id: "item-1",
        label: "Phone",
        sublabel: "Mobile device",
        type: "collected-item",
      },
    ],
  },
];

describe("LinkedDataPanel", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("shows a browser-preview message instead of opening a native CSV export dialog", () => {
    const { container, dispose } = renderComponent(() => <LinkedDataPanel nodes={nodes} />);

    toolbarButton(container, "Export as CSV").click();

    expect(save).not.toHaveBeenCalled();
    expect(container.textContent).toContain(
      "Linked data CSV export is available in the desktop app.",
    );
    dispose();
  });
});
