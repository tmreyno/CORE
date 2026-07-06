// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal } from "solid-js";
import { render } from "solid-js/web";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { MetadataPanel } from "./metadata/MetadataPanel";

const mockReadEwfImageInfo = vi.fn();

vi.mock("../api/ewfExport", () => ({
  readEwfImageInfo: (...args: any[]) => mockReadEwfImageInfo(...args),
}));

function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

const tick = (ms = 50) => new Promise((resolve) => setTimeout(resolve, ms));

const fileInfo = (path: string) => ({
  path,
  filename: path.split("/").pop() ?? path,
  size: 4096,
  container_type: "e01",
});

const containerInfo = {
  e01: {
    format_version: "EWF-E01",
    segment_count: 1,
    sector_count: 8,
    bytes_per_sector: 512,
    chunk_count: 1,
    sectors_per_chunk: 64,
    total_size: 4096,
    compression: "deflate",
  },
};

const enhancedInfo = {
  format: "Stale EnCase",
  formatExtension: "E01",
  isLogical: false,
  isV2: false,
  mediaSize: 4096,
  bytesPerSector: 512,
  sectorsPerChunk: 64,
  compressionLevel: 1,
  compressionMethod: "Deflate",
  mediaType: 1,
  mediaFlags: 0,
  segmentFileVersion: "1.0",
  isCorrupted: false,
  isEncrypted: false,
  caseInfo: {
    caseNumber: "STALE-CASE",
    evidenceNumber: null,
    examinerName: null,
    description: null,
    notes: null,
    acquirySoftwareVersion: null,
    acquiryDate: null,
    acquiryOperatingSystem: null,
    model: null,
    serialNumber: null,
  },
  md5Hash: null,
  sha1Hash: null,
};

describe("MetadataPanel", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockReadEwfImageInfo.mockReset();
  });

  it("ignores stale enhanced EWF info after the selected file changes", async () => {
    let resolveSlow: (value: typeof enhancedInfo) => void = () => {};
    const slowInfo = new Promise<typeof enhancedInfo>((resolve) => {
      resolveSlow = resolve;
    });
    mockReadEwfImageInfo.mockReturnValue(slowInfo);

    const [currentFileInfo, setCurrentFileInfo] = createSignal(fileInfo("/case/slow.E01"));
    const { container } = renderComponent(() => (
      <MetadataPanel
        metadata={null}
        fileInfo={currentFileInfo()}
        containerInfo={containerInfo as any}
      />
    ));
    await tick();

    const loadButton = Array.from(container.querySelectorAll("button")).find((button) =>
      button.textContent?.includes("Load Enhanced Info"),
    ) as HTMLButtonElement;
    expect(loadButton).toBeDefined();
    loadButton.click();
    await tick();

    expect(container.textContent).toContain("Loading libewf info");

    setCurrentFileInfo(fileInfo("/case/current.E01"));
    await tick();

    expect(container.textContent).not.toContain("Loading libewf info");

    resolveSlow(enhancedInfo);
    await tick();

    expect(container.textContent).not.toContain("Stale EnCase");
    expect(container.textContent).not.toContain("STALE-CASE");
  });
});
