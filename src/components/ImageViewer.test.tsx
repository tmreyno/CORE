// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { ImageViewer } from "./ImageViewer";
import { mockInvoke } from "../__tests__/setup";

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

describe("ImageViewer", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
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

    it("calls viewer_read_binary_base64 with the correct path", async () => {
      mockInvoke.mockResolvedValueOnce(MOCK_BASE64);

      renderComponent(() => <ImageViewer path="/evidence/photo.jpg" />);
      await tick();

      expect(mockInvoke).toHaveBeenCalledWith("viewer_read_binary_base64", {
        path: "/evidence/photo.jpg",
      });
    });
  });

  describe("successful render", () => {
    it("renders an img element with data URI src", async () => {
      mockInvoke.mockResolvedValueOnce(MOCK_BASE64);

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.jpg" />
      ));
      await tick();

      const img = container.querySelector("img");
      expect(img).not.toBeNull();
      expect(img!.src).toContain("data:image/jpeg;base64,");
      expect(img!.src).toContain(MOCK_BASE64);
    });

    it("displays the filename in the toolbar", async () => {
      mockInvoke.mockResolvedValueOnce(MOCK_BASE64);

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photos/photo.jpg" />
      ));
      await tick();

      expect(container.textContent).toContain("photo.jpg");
    });

    it("shows zoom controls at 100%", async () => {
      mockInvoke.mockResolvedValueOnce(MOCK_BASE64);

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.png" />
      ));
      await tick();

      expect(container.textContent).toContain("100%");
    });

    it("applies correct mime type for png files", async () => {
      mockInvoke.mockResolvedValueOnce(MOCK_BASE64);

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/image.png" />
      ));
      await tick();

      const img = container.querySelector("img");
      expect(img).not.toBeNull();
      expect(img!.src).toContain("data:image/png;base64,");
    });

    it("applies correct mime type for gif files", async () => {
      mockInvoke.mockResolvedValueOnce(MOCK_BASE64);

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/anim.gif" />
      ));
      await tick();

      const img = container.querySelector("img");
      expect(img!.src).toContain("data:image/gif;base64,");
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
      mockInvoke.mockRejectedValueOnce(new Error("Read error"));

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/corrupt.jpg" />
      ));
      await tick();

      expect(container.textContent).toContain("Retry");
    });

    it("retries loading when retry button clicked", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Temporary error"));

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.jpg" />
      ));
      await tick();

      expect(container.textContent).toContain("Failed to load image");

      // Now mock success
      mockInvoke.mockResolvedValueOnce(MOCK_BASE64);
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
      mockInvoke.mockResolvedValueOnce(MOCK_BASE64);

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
      mockInvoke.mockResolvedValueOnce(MOCK_BASE64);

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
      mockInvoke.mockResolvedValueOnce(MOCK_BASE64);

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
      mockInvoke.mockResolvedValueOnce(MOCK_BASE64);

      const { container } = renderComponent(() => (
        <ImageViewer path="/evidence/photo.jpg" class="custom-class" />
      ));
      await tick();

      const viewer = container.querySelector(".image-viewer");
      expect(viewer?.classList.contains("custom-class")).toBe(true);
    });
  });
});
