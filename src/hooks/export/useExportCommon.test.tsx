import { describe, expect, it, beforeEach, vi } from "vitest";
import { render } from "solid-js/web";
import { open } from "@tauri-apps/plugin-dialog";
import { useExportCommon } from "./useExportCommon";

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

function createToast() {
  return {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  };
}

describe("useExportCommon", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("does not open a native source file picker in browser preview", () => {
    const toast = createToast();
    const { container, dispose } = renderComponent(() => {
      const state = useExportCommon({ toast });
      return <button onClick={state.handleAddSources}>Add files</button>;
    });

    container.querySelector("button")!.click();

    expect(open).not.toHaveBeenCalled();
    expect(toast.error).toHaveBeenCalledWith(
      "Selection Unavailable",
      "Export source and destination browsing is available in the desktop app.",
    );
    dispose();
  });

  it("does not open a native source folder picker in browser preview", () => {
    const toast = createToast();
    const { container, dispose } = renderComponent(() => {
      const state = useExportCommon({ toast });
      return <button onClick={state.handleAddFolder}>Add folder</button>;
    });

    container.querySelector("button")!.click();

    expect(open).not.toHaveBeenCalled();
    expect(toast.error).toHaveBeenCalledWith(
      "Selection Unavailable",
      "Export source and destination browsing is available in the desktop app.",
    );
    dispose();
  });

  it("does not open a native destination picker in browser preview", () => {
    const toast = createToast();
    const { container, dispose } = renderComponent(() => {
      const state = useExportCommon({ toast });
      return <button onClick={state.handleSelectDestination}>Destination</button>;
    });

    container.querySelector("button")!.click();

    expect(open).not.toHaveBeenCalled();
    expect(toast.error).toHaveBeenCalledWith(
      "Selection Unavailable",
      "Export source and destination browsing is available in the desktop app.",
    );
    dispose();
  });
});
