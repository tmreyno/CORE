// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render } from "solid-js/web";
import { PdfViewer } from "./PdfViewer";

// Mock pdfjs-dist before importing anything that depends on it
vi.mock("pdfjs-dist", () => ({
  GlobalWorkerOptions: { workerSrc: "" },
}));

// Mock the pdfHelpers module
const mockLoadPdfDocument = vi.fn();
const mockRenderPdfPage = vi.fn();
const mockGenerateThumbnailsBatch = vi.fn();

vi.mock("./pdf/pdfHelpers", () => ({
  loadPdfDocument: (...args: any[]) => mockLoadPdfDocument(...args),
  renderPdfPage: (...args: any[]) => mockRenderPdfPage(...args),
  generateThumbnailsBatch: (...args: any[]) => mockGenerateThumbnailsBatch(...args),
}));

// Mock solid-primitives modules used by PdfViewer
vi.mock("@solid-primitives/resize-observer", () => ({
  createResizeObserver: vi.fn(),
}));

vi.mock("@solid-primitives/scheduled", () => ({
  debounce: (fn: () => void) => fn,
}));

vi.mock("@solid-primitives/event-listener", () => ({
  makeEventListener: vi.fn(),
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

// Create a mock PDFDocumentProxy
function createMockPdfDoc(numPages: number = 5) {
  const mockPage = {
    getViewport: vi.fn(() => ({ width: 612, height: 792, scale: 1 })),
    render: vi.fn(() => ({ promise: Promise.resolve(), cancel: vi.fn() })),
  };

  return {
    numPages,
    getPage: vi.fn(() => Promise.resolve(mockPage)),
    destroy: vi.fn(),
  };
}

describe("PdfViewer", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockLoadPdfDocument.mockReset();
    mockRenderPdfPage.mockReset();
    mockGenerateThumbnailsBatch.mockReset();
    mockRenderPdfPage.mockResolvedValue(undefined);
    mockGenerateThumbnailsBatch.mockResolvedValue([]);
  });

  describe("loading state", () => {
    it("shows loading spinner initially", () => {
      mockLoadPdfDocument.mockReturnValue(new Promise(() => {}));

      const { container } = renderComponent(() => (
        <PdfViewer path="/evidence/report.pdf" />
      ));

      expect(container.textContent).toContain("Loading PDF...");
    });

    it("calls loadPdfDocument with correct path", async () => {
      const mockDoc = createMockPdfDoc(3);
      mockLoadPdfDocument.mockResolvedValueOnce(mockDoc);

      renderComponent(() => <PdfViewer path="/evidence/report.pdf" />);
      await tick();

      expect(mockLoadPdfDocument).toHaveBeenCalledWith("/evidence/report.pdf");
    });
  });

  describe("successful render", () => {
    it("displays page info after loading", async () => {
      const mockDoc = createMockPdfDoc(10);
      mockLoadPdfDocument.mockResolvedValueOnce(mockDoc);

      const { container } = renderComponent(() => (
        <PdfViewer path="/evidence/report.pdf" />
      ));
      await tick();

      // Should show current page and total pages somewhere
      expect(container.textContent).toContain("1");
      expect(container.textContent).toContain("10");
    });

    it("renders a canvas element for PDF pages", async () => {
      const mockDoc = createMockPdfDoc(5);
      mockLoadPdfDocument.mockResolvedValueOnce(mockDoc);

      const { container } = renderComponent(() => (
        <PdfViewer path="/evidence/report.pdf" />
      ));
      await tick();

      const canvas = container.querySelector("canvas");
      expect(canvas).not.toBeNull();
    });

    it("initializes with page 1 after loading", async () => {
      const mockDoc = createMockPdfDoc(3);
      mockLoadPdfDocument.mockResolvedValueOnce(mockDoc);

      const { container } = renderComponent(() => (
        <PdfViewer path="/evidence/report.pdf" />
      ));
      await tick();

      // After loading, should display page 1 info (canvas rendering may skip in jsdom)
      expect(container.textContent).toContain("1");
      expect(container.textContent).toContain("3");
    });

    it("generates thumbnails in background", async () => {
      const mockDoc = createMockPdfDoc(5);
      mockLoadPdfDocument.mockResolvedValueOnce(mockDoc);

      renderComponent(() => <PdfViewer path="/evidence/report.pdf" />);
      await tick();

      expect(mockGenerateThumbnailsBatch).toHaveBeenCalledWith(
        mockDoc,
        3,
        expect.any(Function)
      );
    });

    it("displays the file path in status bar", async () => {
      const mockDoc = createMockPdfDoc(3);
      mockLoadPdfDocument.mockResolvedValueOnce(mockDoc);

      const { container } = renderComponent(() => (
        <PdfViewer path="/evidence/docs/report.pdf" />
      ));
      await tick();

      expect(container.textContent).toContain("/evidence/docs/report.pdf");
    });
  });

  describe("error state", () => {
    it("shows error message when PDF loading fails", async () => {
      mockLoadPdfDocument.mockRejectedValueOnce(new Error("Invalid PDF format"));

      const { container } = renderComponent(() => (
        <PdfViewer path="/evidence/corrupt.pdf" />
      ));
      await tick();

      expect(container.textContent).toContain("Failed to load PDF");
      expect(container.textContent).toContain("Invalid PDF format");
    });

    it("shows try again button on error", async () => {
      mockLoadPdfDocument.mockRejectedValueOnce(new Error("Read error"));

      const { container } = renderComponent(() => (
        <PdfViewer path="/evidence/bad.pdf" />
      ));
      await tick();

      expect(container.textContent).toContain("Try Again");
    });

    it("retries loading when try again button clicked", async () => {
      mockLoadPdfDocument.mockRejectedValueOnce(new Error("Temporary error"));

      const { container } = renderComponent(() => (
        <PdfViewer path="/evidence/report.pdf" />
      ));
      await tick();

      expect(container.textContent).toContain("Failed to load PDF");

      // Now mock success
      const mockDoc = createMockPdfDoc(3);
      mockLoadPdfDocument.mockResolvedValueOnce(mockDoc);

      const retryBtn = Array.from(container.querySelectorAll("button")).find(
        (b) => b.textContent?.includes("Try Again")
      ) as HTMLButtonElement;
      expect(retryBtn).toBeDefined();
      retryBtn.click();
      await tick();

      // Should now have a canvas
      const canvas = container.querySelector("canvas");
      expect(canvas).not.toBeNull();
    });
  });

  describe("cleanup", () => {
    it("destroys PDF document on unmount", async () => {
      const mockDoc = createMockPdfDoc(3);
      mockLoadPdfDocument.mockResolvedValueOnce(mockDoc);

      const { dispose } = renderComponent(() => (
        <PdfViewer path="/evidence/report.pdf" />
      ));
      await tick();

      dispose();

      expect(mockDoc.destroy).toHaveBeenCalled();
    });
  });

  describe("optional class", () => {
    it("applies custom class when provided", async () => {
      const mockDoc = createMockPdfDoc(3);
      mockLoadPdfDocument.mockResolvedValueOnce(mockDoc);

      const { container } = renderComponent(() => (
        <PdfViewer path="/evidence/report.pdf" class="custom-pdf" />
      ));
      await tick();

      const viewer = container.firstElementChild;
      expect(viewer?.classList.contains("custom-pdf")).toBe(true);
    });
  });
});
