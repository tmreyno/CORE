import { describe, expect, it, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { createSignal } from "solid-js";
import { open, save } from "@tauri-apps/plugin-dialog";
import { TestTab } from "./TestTab";
import { RepairTab } from "./RepairTab";
import { ValidateTab } from "./ValidateTab";
import { ExtractTab } from "./ExtractTab";
import { CompressTab } from "./CompressTab";
import { DecompressTab } from "./DecompressTab";

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

function clickAllBrowseButtons(container: HTMLElement) {
  const buttons = Array.from(container.querySelectorAll("button")).filter((button) =>
    button.textContent?.includes("Browse"),
  );
  expect(buttons.length).toBeGreaterThan(0);
  buttons.forEach((button) => button.click());
}

function expectBrowserGuard(container: HTMLElement) {
  expect(open).not.toHaveBeenCalled();
  expect(save).not.toHaveBeenCalled();
  expect(container.textContent).toContain(
    "Archive tool file browsing is available in the desktop app.",
  );
}

describe("tools mode browser dialog guards", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("guards test archive browsing", () => {
    const [archivePath, setArchivePath] = createSignal("");
    const { container, dispose } = renderComponent(() => (
      <TestTab archivePath={archivePath} setArchivePath={setArchivePath} />
    ));

    clickAllBrowseButtons(container);
    expectBrowserGuard(container);
    dispose();
  });

  it("guards repair archive input and output browsing", () => {
    const [corruptedPath, setCorruptedPath] = createSignal("");
    const [outputPath, setOutputPath] = createSignal("");
    const { container, dispose } = renderComponent(() => (
      <RepairTab
        corruptedPath={corruptedPath}
        setCorruptedPath={setCorruptedPath}
        outputPath={outputPath}
        setOutputPath={setOutputPath}
      />
    ));

    clickAllBrowseButtons(container);
    expectBrowserGuard(container);
    dispose();
  });

  it("guards validate archive browsing", () => {
    const [archivePath, setArchivePath] = createSignal("");
    const { container, dispose } = renderComponent(() => (
      <ValidateTab archivePath={archivePath} setArchivePath={setArchivePath} />
    ));

    clickAllBrowseButtons(container);
    expectBrowserGuard(container);
    dispose();
  });

  it("guards split archive extraction browsing", () => {
    const [firstVolume, setFirstVolume] = createSignal("");
    const [outputDir, setOutputDir] = createSignal("");
    const { container, dispose } = renderComponent(() => (
      <ExtractTab
        firstVolume={firstVolume}
        setFirstVolume={setFirstVolume}
        outputDir={outputDir}
        setOutputDir={setOutputDir}
      />
    ));

    clickAllBrowseButtons(container);
    expectBrowserGuard(container);
    dispose();
  });

  it("guards compression input and output browsing", () => {
    const [algorithm, setAlgorithm] = createSignal<"lzma" | "lzma2">("lzma");
    const [level, setLevel] = createSignal(5);
    const [inputPath, setInputPath] = createSignal("");
    const [outputPath, setOutputPath] = createSignal("");
    const { container, dispose } = renderComponent(() => (
      <CompressTab
        algorithm={algorithm}
        setAlgorithm={setAlgorithm}
        level={level}
        setLevel={setLevel}
        inputPath={inputPath}
        setInputPath={setInputPath}
        outputPath={outputPath}
        setOutputPath={setOutputPath}
      />
    ));

    clickAllBrowseButtons(container);
    expectBrowserGuard(container);
    dispose();
  });

  it("guards decompression input and output browsing", () => {
    const [inputPath, setInputPath] = createSignal("");
    const [outputPath, setOutputPath] = createSignal("");
    const { container, dispose } = renderComponent(() => (
      <DecompressTab
        inputPath={inputPath}
        setInputPath={setInputPath}
        outputPath={outputPath}
        setOutputPath={setOutputPath}
      />
    ));

    clickAllBrowseButtons(container);
    expectBrowserGuard(container);
    dispose();
  });
});
