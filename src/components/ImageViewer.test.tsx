// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render } from "solid-js/web";
import { createSignal } from "solid-js";
import { ImageViewer } from "./ImageViewer";
import { mockInvoke } from "../__tests__/setup";

vi.mock("../utils/platform", () => ({
  isTauri: true,
}));

// Helper to render and return the container
function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

// Wait for async updates
const tick = (ms = 50) => new Promise(resolve => setTimeout(resolve, ms));

// A small 1x1 red PNG as base64
const MOCK_BASE64 =
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==";
const INLINE_IMAGE_LIMIT = 100 * 1024 * 1024;

function makeChunk(base64 = MOCK_BASE64, totalSize = 1024, offset = 0) {
  const bytesRead = atob(base64).length;
  return {
    path: "source",
    offset,
    bytesRead,
    totalSize,
    eof: offset + bytesRead >= totalSize,
    data: base64,
  };
}

function mockLocalImageLoad(base64 = MOCK_BASE64, size = 1024, path = "/evidence/photo.jpg") {
  mockInvoke
    .mockResolvedValueOnce({
      path,
      size,
      maxInlineBytes: INLINE_IMAGE_LIMIT,
      supportsRangeReads: true,
    })
    .mockResolvedValueOnce(makeChunk(base64, atob(base64).length));
}

describe("ImageViewer", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
    let urlCounter = 0;
    Object.defineProperty(URL, "createObjectURL", {
      configurable: true,
      value: vi.fn(() => `blob:core-ffx-image-${++urlCounter}`),
    });
    Object.defineProperty(URL, "revokeObjectURL", {
      configurable: true,
      value: vi.fn(),
    });
  });

  describe("loading state", () => {
    it("shows loading spinner initially", () => {
      // Never resolve so we stay in loading
      mockInvoke.mockReturnValue(new Promise(() => {}));

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.jpg" />
      ));

      expect(container.textContent).toContain("Loading image...");
    });

    it("calls ranged binary image reads with the correct path", async () => {
      mockLocalImageLoad(MOCK_BASE64, 1024, "/evidence/photo.jpg");

      renderComponent(() => <ImageViewer path="/evidence/photo.jpg" />);
      await tick();

      expect(mockInvoke).toHaveBeenCalledWith("viewer_get_binary_info", {
        path: "/evidence/photo.jpg",
      });
      expect(mockInvoke).toHaveBeenCalledWith("viewer_read_binary_base64_chunk", {
        path: "/evidence/photo.jpg",
        offset: 0,
        size: 1024,
      });
    });

    it("calls ranged source image reads when an evidence source is provided", async () => {
      const source = {
        containerPath: "/evidence/image.ad1",
        entryPath: "photos/photo.jpg",
        containerType: "ad1",
        size: atob(MOCK_BASE64).length,
      };
      mockInvoke.mockResolvedValueOnce(makeChunk(MOCK_BASE64, source.size));

      renderComponent(() => <ImageViewer path="/evidence/photo.jpg" source={source} />);
      await tick();

      expect(mockInvoke).toHaveBeenCalledWith("viewer_read_binary_source_base64_chunk", {
        source,
        offset: 0,
        size: source.size,
      });
    });
  });

  describe("successful render", () => {
    it("renders an img element with a blob URL src", async () => {
      mockLocalImageLoad();

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.jpg" />
      ));
      await tick();

      const img = container.querySelector("img");
      expect(img).not.toBeNull();
      expect(img!.src).toContain("blob:core-ffx-image-1");
      expect(URL.createObjectURL).toHaveBeenCalledWith(expect.any(Blob));
    });

    it("displays the filename in the toolbar", async () => {
      mockLocalImageLoad(MOCK_BASE64, 1024, "/evidence/photos/photo.jpg");

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photos/photo.jpg" />
      ));
      await tick();

      expect(container.textContent).toContain("photo.jpg");
    });

    it("shows zoom controls at 100%", async () => {
      mockLocalImageLoad(MOCK_BASE64, 1024, "/evidence/photo.png");

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.png" />
      ));
      await tick();

      expect(container.textContent).toContain("100%");
    });

    it("applies correct mime type for png files", async () => {
      mockLocalImageLoad(MOCK_BASE64, 1024, "/evidence/image.png");

      renderComponent(() => <ImageViewer path="/evidence/image.png" />);
      await tick();

      expect(URL.createObjectURL).toHaveBeenCalledWith(
        expect.objectContaining({ type: "image/png" }),
      );
    });

    it("applies correct mime type for gif files", async () => {
      mockLocalImageLoad(MOCK_BASE64, 1024, "/evidence/anim.gif");

      renderComponent(() => <ImageViewer path="/evidence/anim.gif" />);
      await tick();

      expect(URL.createObjectURL).toHaveBeenCalledWith(
        expect.objectContaining({ type: "image/gif" }),
      );
    });

    it("ignores stale image loads when the selected path changes", async () => {
      let resolveFirst: (value: ReturnType<typeof makeChunk>) => void = () => {};
      const firstLoad = new Promise<ReturnType<typeof makeChunk>>((resolve) => {
        resolveFirst = resolve;
      });
      const SECOND_BASE64 = "c2Vjb25k";
      const secondSize = atob(SECOND_BASE64).length;
      const [path, setPath] = createSignal("/evidence/first.jpg");
      mockInvoke.mockImplementation((command: string, args: any) => {
        if (command === "viewer_get_binary_info" && args.path === "/evidence/first.jpg") {
          return Promise.resolve({
            path: "/evidence/first.jpg",
            size: atob(MOCK_BASE64).length,
            maxInlineBytes: INLINE_IMAGE_LIMIT,
            supportsRangeReads: true,
          });
        }
        if (command === "viewer_read_binary_base64_chunk" && args.path === "/evidence/first.jpg") {
          return firstLoad;
        }
        if (command === "viewer_get_binary_info" && args.path === "/evidence/second.png") {
          return Promise.resolve({
            path: "/evidence/second.png",
            size: secondSize,
            maxInlineBytes: INLINE_IMAGE_LIMIT,
            supportsRangeReads: true,
          });
        }
        if (command === "viewer_read_binary_base64_chunk" && args.path === "/evidence/second.png") {
          return Promise.resolve(makeChunk(SECOND_BASE64, secondSize));
        }
        return Promise.reject(new Error(`Unexpected invoke: ${command}`));
      });

      const { container } = renderComponent(() => (
        <ImageViewer path={path()} />
      ));

      setPath("/evidence/second.png");
      await tick();
      resolveFirst(makeChunk(MOCK_BASE64, atob(MOCK_BASE64).length));
      await tick();

      const img = container.querySelector("img");
      expect(img).not.toBeNull();
      expect(img!.src).toContain("blob:core-ffx-image-1");
      expect(URL.revokeObjectURL).toHaveBeenCalledWith("blob:core-ffx-image-2");
    });

    it("ignores stale image decode errors after the selected path changes", async () => {
      const firstSize = atob(MOCK_BASE64).length;
      const SECOND_BASE64 = "c2Vjb25k";
      const secondSize = atob(SECOND_BASE64).length;
      const [path, setPath] = createSignal("/evidence/first.jpg");
      mockInvoke.mockImplementation((command: string, args: any) => {
        if (command === "viewer_get_binary_info" && args.path === "/evidence/first.jpg") {
          return Promise.resolve({
            path: "/evidence/first.jpg",
            size: firstSize,
            maxInlineBytes: INLINE_IMAGE_LIMIT,
            supportsRangeReads: true,
          });
        }
        if (command === "viewer_read_binary_base64_chunk" && args.path === "/evidence/first.jpg") {
          return Promise.resolve(makeChunk(MOCK_BASE64, firstSize));
        }
        if (command === "viewer_get_binary_info" && args.path === "/evidence/second.png") {
          return Promise.resolve({
            path: "/evidence/second.png",
            size: secondSize,
            maxInlineBytes: INLINE_IMAGE_LIMIT,
            supportsRangeReads: true,
          });
        }
        if (command === "viewer_read_binary_base64_chunk" && args.path === "/evidence/second.png") {
          return Promise.resolve(makeChunk(SECOND_BASE64, secondSize));
        }
        return Promise.reject(new Error(`Unexpected invoke: ${command}`));
      });

      const { container } = renderComponent(() => (
        <ImageViewer path={path()} />
      ));
      await tick();

      const img = container.querySelector("img")!;
      expect(img.src).toContain("blob:core-ffx-image-1");

      setPath("/evidence/second.png");
      await tick(100);
      const currentImg = container.querySelector("img")!;
      expect(currentImg.src).toContain("blob:core-ffx-image-2");

      Object.defineProperty(currentImg, "currentSrc", {
        configurable: true,
        value: "blob:core-ffx-image-1",
      });
      currentImg.dispatchEvent(new Event("error"));
      await tick();

      expect(container.textContent).not.toContain("Failed to decode image data");
      expect(container.textContent).not.toContain("Failed to load image");
    });
  });

  describe("error state", () => {
    it("shows error message when loading fails", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("File not found"));

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/missing.jpg" />
      ));
      await tick();

      expect(container.textContent).toContain("Failed to load image");
      expect(container.textContent).toContain("File not found");
    });

    it("shows retry button on error", async () => {
      mockInvoke
        .mockResolvedValueOnce({
          path: "/evidence/corrupt.jpg",
          size: 1024,
          maxInlineBytes: INLINE_IMAGE_LIMIT,
          supportsRangeReads: true,
        })
        .mockRejectedValueOnce(new Error("Read error"));

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/corrupt.jpg" />
      ));
      await tick();

      expect(container.textContent).toContain("Retry");
    });

    it("rejects oversized source images before full base64 read", async () => {
      const source = {
        containerPath: "/evidence/image.ad1",
        entryPath: "photos/huge.jpg",
        containerType: "ad1",
        size: 101 * 1024 * 1024,
      };

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/huge.jpg" source={source} />
      ));
      await tick();

      expect(container.textContent).toContain("Image is too large for inline preview");
      expect(mockInvoke).not.toHaveBeenCalledWith(
        "viewer_read_binary_source_base64_chunk",
        expect.anything(),
      );
    });

    it("rejects oversized local images before full base64 read", async () => {
      mockInvoke.mockResolvedValueOnce({
        path: "/evidence/huge.jpg",
        size: 101 * 1024 * 1024,
        maxInlineBytes: INLINE_IMAGE_LIMIT,
        supportsRangeReads: true,
      });

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/huge.jpg" />
      ));
      await tick();

      expect(container.textContent).toContain("Image is too large for inline preview");
      expect(mockInvoke).toHaveBeenCalledWith("viewer_get_binary_info", {
        path: "/evidence/huge.jpg",
      });
      expect(mockInvoke).not.toHaveBeenCalledWith(
        "viewer_read_binary_base64_chunk",
        expect.anything(),
      );
    });

    it("retries loading when retry button clicked", async () => {
      mockInvoke
        .mockResolvedValueOnce({
          path: "/evidence/photo.jpg",
          size: 1024,
          maxInlineBytes: INLINE_IMAGE_LIMIT,
          supportsRangeReads: true,
        })
        .mockRejectedValueOnce(new Error("Temporary error"));

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.jpg" />
      ));
      await tick();

      expect(container.textContent).toContain("Failed to load image");

      // Now mock success
      mockLocalImageLoad();
      const retryBtn = Array.from(container.querySelectorAll("button")).find(
        (b) => b.textContent?.includes("Retry")
      ) as HTMLButtonElement;
      expect(retryBtn).not.toBeNull();
      retryBtn.click();
      await tick();

      expect(container.textContent).not.toContain("Failed to load image");
    });
  });

  describe("zoom controls", () => {
    it("zoom in increases the displayed percentage", async () => {
      mockLocalImageLoad();

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.jpg" />
      ));
      await tick();

      // Find zoom in button by title
      const zoomInBtn = container.querySelector('button[title="Zoom in"]') as HTMLButtonElement;
      expect(zoomInBtn).not.toBeNull();
      zoomInBtn.click();
      await tick(10);

      expect(container.textContent).toContain("125%");
    });

    it("zoom out decreases the displayed percentage", async () => {
      mockLocalImageLoad();

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.jpg" />
      ));
      await tick();

      const zoomOutBtn = container.querySelector('button[title="Zoom out"]') as HTMLButtonElement;
      expect(zoomOutBtn).not.toBeNull();
      zoomOutBtn.click();
      await tick(10);

      expect(container.textContent).toContain("75%");
    });

    it("reset zoom button restores to 100%", async () => {
      mockLocalImageLoad();

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.jpg" />
      ));
      await tick();

      // Zoom in first
      const zoomInBtn = container.querySelector('button[title="Zoom in"]') as HTMLButtonElement;
      zoomInBtn.click();
      await tick(10);
      expect(container.textContent).toContain("125%");

      // Click 100% reset button
      const resetBtn = Array.from(container.querySelectorAll("button")).find(
        (b) => b.textContent === "100%"
      );
      expect(resetBtn).not.toBeNull();
      resetBtn!.click();
      await tick(10);

      // Should show 100% in the zoom display span
      const zoomSpan = container.querySelector(".w-14");
      expect(zoomSpan?.textContent).toContain("100%");
    });
  });

  describe("optional class", () => {
    it("applies custom class when provided", async () => {
      mockLocalImageLoad();

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.jpg" class="custom-class" />
      ));
      await tick();

      const viewer = container.querySelector(".image-viewer");
      expect(viewer?.classList.contains("custom-class")).toBe(true);
    });
  });

  describe("limited format support", () => {
    it("shows warning for HEIC files", async () => {
      mockLocalImageLoad(MOCK_BASE64, 1024, "/evidence/photo.heic");

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.heic" />
      ));
      await tick();

      expect(container.textContent).toContain("Limited format support");
    });

    it("shows warning for TIFF files", async () => {
      mockLocalImageLoad(MOCK_BASE64, 1024, "/evidence/scan.tiff");

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/scan.tiff" />
      ));
      await tick();

      expect(container.textContent).toContain("Limited format support");
    });

    it("does not show warning for standard formats", async () => {
      mockLocalImageLoad(MOCK_BASE64, 1024, "/evidence/photo.png");

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.png" />
      ));
      await tick();

      expect(container.textContent).not.toContain("Limited format support");
    });

    it("shows format-specific error when img decode fails for HEIC", async () => {
      mockLocalImageLoad(MOCK_BASE64, 1024, "/evidence/photo.heic");

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.heic" />
      ));
      await tick();

      // Simulate img onError
      const img = container.querySelector("img");
      expect(img).not.toBeNull();
      img!.dispatchEvent(new Event("error"));
      await tick();

      expect(container.textContent).toContain(".heic");
      expect(container.textContent).toContain("may not be supported");
    });

    it("shows generic error when img decode fails for standard format", async () => {
      mockLocalImageLoad(MOCK_BASE64, 1024, "/evidence/photo.png");

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.png" />
      ));
      await tick();

      const img = container.querySelector("img");
      expect(img).not.toBeNull();
      img!.dispatchEvent(new Event("error"));
      await tick();

      expect(container.textContent).toContain("Failed to decode image data");
    });
  });
});
