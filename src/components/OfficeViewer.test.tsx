// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render } from "solid-js/web";
import { OfficeViewer } from "./OfficeViewer";
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

// Mock office document data (matches Rust OfficeDocumentInfo)
const mockDocxData = {
  path: "/tmp/report.docx",
  format: "docx",
  formatDescription: "Office Open XML Document (.docx)",
  metadata: {
    title: "Quarterly Report",
    creator: "Jane Doe",
    lastModifiedBy: "John Smith",
    subject: "Finance",
    description: "Q3 financial summary",
    created: "2024-01-15T08:00:00Z",
    modified: "2024-06-20T14:30:00Z",
    application: "Microsoft Word 365",
    pageCount: 12,
    wordCount: 5400,
    charCount: 32000,
  },
  sections: [
    { label: "Body", paragraphs: ["Introduction paragraph.", "Second paragraph with details."] },
    { label: "Appendix", paragraphs: ["Appendix content here."] },
  ],
  totalChars: 32000,
  totalWords: 5400,
  extractionComplete: true,
  warnings: [],
};

const mockPptxData = {
  ...mockDocxData,
  path: "/tmp/slides.pptx",
  format: "pptx",
  formatDescription: "Office Open XML Presentation (.pptx)",
  sections: [
    { label: "Slide 1", paragraphs: ["Welcome to the presentation"] },
    { label: "Slide 2", paragraphs: ["Key findings", "Bullet point 1"] },
  ],
};

describe("OfficeViewer", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
  });

  describe("DOCX rendering", () => {
    it("calls office_read_document with correct path", async () => {
      mockInvoke.mockResolvedValueOnce(mockDocxData);

      renderComponent(() => <OfficeViewer path="/tmp/report.docx" />);
      await tick();

      expect(mockInvoke).toHaveBeenCalledWith("office_read_document", {
        path: "/tmp/report.docx",
      });
    });

    it("renders document text content", async () => {
      mockInvoke.mockResolvedValueOnce(mockDocxData);

      const { container } = renderComponent(() => (
        <OfficeViewer path="/tmp/report.docx" />
      ));
      await tick();

      expect(container.textContent).toContain("Introduction paragraph.");
      expect(container.textContent).toContain("Second paragraph with details.");
    });

    it("displays format label", async () => {
      mockInvoke.mockResolvedValueOnce(mockDocxData);

      const { container } = renderComponent(() => (
        <OfficeViewer path="/tmp/report.docx" />
      ));
      await tick();

      expect(container.textContent).toContain("DOCX");
    });

    it("renders section labels", async () => {
      mockInvoke.mockResolvedValueOnce(mockDocxData);

      const { container } = renderComponent(() => (
        <OfficeViewer path="/tmp/report.docx" />
      ));
      await tick();

      expect(container.textContent).toContain("Body");
      expect(container.textContent).toContain("Appendix");
    });
  });

  describe("PPTX rendering", () => {
    it("renders slide content for presentations", async () => {
      mockInvoke.mockResolvedValueOnce(mockPptxData);

      const { container } = renderComponent(() => (
        <OfficeViewer path="/tmp/slides.pptx" />
      ));
      await tick();

      expect(container.textContent).toContain("Welcome to the presentation");
      expect(container.textContent).toContain("Key findings");
    });

    it("displays PPTX format label", async () => {
      mockInvoke.mockResolvedValueOnce(mockPptxData);

      const { container } = renderComponent(() => (
        <OfficeViewer path="/tmp/slides.pptx" />
      ));
      await tick();

      expect(container.textContent).toContain("PPTX");
    });
  });

  describe("Loading and error states", () => {
    it("shows loading state initially", () => {
      mockInvoke.mockReturnValue(new Promise(() => {}));

      const { container } = renderComponent(() => (
        <OfficeViewer path="/tmp/report.docx" />
      ));

      // Loading spinner or indicator should be present
      expect(container.textContent).toContain("Loading");
    });

    it("shows error when parsing fails", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Corrupted DOCX file"));

      const { container } = renderComponent(() => (
        <OfficeViewer path="/tmp/bad.docx" />
      ));
      await tick();

      expect(container.textContent).toContain("Corrupted DOCX file");
    });
  });

  describe("Edge cases", () => {
    it("handles document with no metadata", async () => {
      mockInvoke.mockResolvedValueOnce({
        ...mockDocxData,
        metadata: {
          title: null,
          creator: null,
          lastModifiedBy: null,
          subject: null,
          description: null,
          created: null,
          modified: null,
          application: null,
          pageCount: null,
          wordCount: null,
          charCount: null,
        },
      });

      const { container } = renderComponent(() => (
        <OfficeViewer path="/tmp/empty-meta.docx" />
      ));
      await tick();

      // Should still render without crashing
      expect(container.innerHTML).toBeTruthy();
    });

    it("handles document with warnings", async () => {
      mockInvoke.mockResolvedValueOnce({
        ...mockDocxData,
        warnings: ["Some images could not be extracted"],
      });

      const { container } = renderComponent(() => (
        <OfficeViewer path="/tmp/warn.docx" />
      ));
      await tick();

      expect(container.textContent).toContain("Some images could not be extracted");
    });

    it("handles empty sections", async () => {
      mockInvoke.mockResolvedValueOnce({
        ...mockDocxData,
        sections: [],
        totalChars: 0,
        totalWords: 0,
      });

      const { container } = renderComponent(() => (
        <OfficeViewer path="/tmp/empty.docx" />
      ));
      await tick();

      // Should render without crashing
      expect(container.innerHTML).toBeTruthy();
    });

    it("emits metadata callback when provided", async () => {
      mockInvoke.mockResolvedValueOnce(mockDocxData);
      const onMetadata = vi.fn();

      renderComponent(() => (
        <OfficeViewer path="/tmp/report.docx" onMetadata={onMetadata} />
      ));
      await tick();

      expect(onMetadata).toHaveBeenCalled();
    });
  });
});
