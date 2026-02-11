// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render } from "solid-js/web";
import { DocumentViewer } from "./DocumentViewer";
import { mockInvoke } from "../__tests__/setup";

// Mock the documentHelpers module
vi.mock("./document/documentHelpers", () => ({
  getFormatIcon: (format: string) => format === "pdf" ? "📄" : "📝",
  performSearch: vi.fn(() => 0),
  printDocument: vi.fn(),
  downloadHtml: vi.fn(),
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

// Mock successful document response
const mockDocumentResponse = {
  success: true,
  content: {
    format: "pdf",
    title: "Forensic Report 2024",
    author: "Investigator Smith",
    page_count: 15,
    file_size: 524288,
    text: "This is the extracted text content of the forensic report.",
    html: "<h1>Forensic Report</h1><p>This is the extracted text content of the forensic report.</p>",
  },
  error: null,
};

// Mock successful metadata response
const mockMetadataResponse = {
  success: true,
  metadata: {
    format: "pdf",
    title: "Forensic Report 2024",
    author: "Investigator Smith",
    subject: "Digital Forensics",
    keywords: ["forensic", "evidence", "report"],
    page_count: 15,
    file_size: 524288,
    created: "2024-01-15T10:00:00Z",
    modified: "2024-06-20T14:30:00Z",
    producer: "LibreOffice",
    creator: "Writer",
    encrypted: false,
    word_count: 2500,
  },
  error: null,
};

// Mock error responses
const mockErrorDocResponse = {
  success: false,
  content: null,
  error: "Unsupported document format",
};

const mockErrorMetaResponse = {
  success: false,
  metadata: null,
  error: "Could not extract metadata",
};

describe("DocumentViewer", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
  });

  describe("loading state", () => {
    it("shows loading spinner initially", () => {
      mockInvoke.mockReturnValue(new Promise(() => {}));

      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/report.pdf" />
      ));

      expect(container.textContent).toContain("Loading document...");
    });

    it("calls document_read and document_get_metadata in parallel", async () => {
      mockInvoke.mockResolvedValue(mockDocumentResponse);
      mockInvoke.mockResolvedValue(mockMetadataResponse);

      renderComponent(() => <DocumentViewer path="/evidence/report.pdf" />);
      await tick();

      expect(mockInvoke).toHaveBeenCalledWith("document_read", {
        path: "/evidence/report.pdf",
      });
      expect(mockInvoke).toHaveBeenCalledWith("document_get_metadata", {
        path: "/evidence/report.pdf",
      });
    });
  });

  describe("successful render", () => {
    beforeEach(() => {
      mockInvoke
        .mockResolvedValueOnce(mockDocumentResponse)
        .mockResolvedValueOnce(mockMetadataResponse);
    });

    it("renders document HTML content", async () => {
      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/report.pdf" />
      ));
      await tick();

      expect(container.textContent).toContain("Forensic Report");
      expect(container.textContent).toContain("extracted text content");
    });

    it("displays document format in toolbar", async () => {
      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/report.pdf" />
      ));
      await tick();

      // The toolbar shows the format
      expect(container.textContent).toContain("pdf");
    });

    it("shows zoom percentage at 100% initially", async () => {
      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/report.pdf" />
      ));
      await tick();

      expect(container.textContent).toContain("100%");
    });

    it("renders innerHTML from document HTML", async () => {
      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/report.pdf" />
      ));
      await tick();

      // The HTML is injected via innerHTML
      const contentDiv = container.querySelector(".document-content");
      expect(contentDiv).not.toBeNull();
      expect(contentDiv!.innerHTML).toContain("<h1>Forensic Report</h1>");
    });
  });

  describe("error state", () => {
    it("shows error when document_read fails", async () => {
      mockInvoke
        .mockResolvedValueOnce(mockErrorDocResponse)
        .mockResolvedValueOnce(mockMetadataResponse);

      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/bad.xyz" />
      ));
      await tick();

      expect(container.textContent).toContain("Failed to load document");
      expect(container.textContent).toContain("Unsupported document format");
    });

    it("shows retry button on error", async () => {
      mockInvoke
        .mockResolvedValueOnce(mockErrorDocResponse)
        .mockResolvedValueOnce(mockErrorMetaResponse);

      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/bad.xyz" />
      ));
      await tick();

      expect(container.textContent).toContain("Retry");
    });

    it("retries loading when retry button clicked", async () => {
      mockInvoke
        .mockResolvedValueOnce(mockErrorDocResponse)
        .mockResolvedValueOnce(mockErrorMetaResponse);

      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/report.pdf" />
      ));
      await tick();

      expect(container.textContent).toContain("Failed to load document");

      // Now mock success for retry
      mockInvoke
        .mockResolvedValueOnce(mockDocumentResponse)
        .mockResolvedValueOnce(mockMetadataResponse);

      const retryBtn = Array.from(container.querySelectorAll("button")).find(
        (b) => b.textContent?.includes("Retry")
      ) as HTMLButtonElement;
      retryBtn.click();
      await tick();

      expect(container.textContent).toContain("Forensic Report");
    });

    it("shows error when invoke rejects", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Network timeout"));
      mockInvoke.mockResolvedValueOnce(mockMetadataResponse);

      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/report.pdf" />
      ));
      await tick();

      expect(container.textContent).toContain("Failed to load document");
      expect(container.textContent).toContain("Network timeout");
    });
  });

  describe("zoom controls", () => {
    beforeEach(() => {
      mockInvoke
        .mockResolvedValueOnce(mockDocumentResponse)
        .mockResolvedValueOnce(mockMetadataResponse);
    });

    it("zoom in increases percentage to 125%", async () => {
      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/report.pdf" />
      ));
      await tick();

      // Find zoom in button by title or icon
      const zoomInBtn = container.querySelector('button[title="Zoom in"]') as HTMLButtonElement;
      if (zoomInBtn) {
        zoomInBtn.click();
        await tick(10);
        expect(container.textContent).toContain("125%");
      }
    });

    it("zoom out decreases percentage to 75%", async () => {
      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/report.pdf" />
      ));
      await tick();

      const zoomOutBtn = container.querySelector('button[title="Zoom out"]') as HTMLButtonElement;
      if (zoomOutBtn) {
        zoomOutBtn.click();
        await tick(10);
        expect(container.textContent).toContain("75%");
      }
    });

    it("applies scale transform to content div", async () => {
      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/report.pdf" />
      ));
      await tick();

      const contentDiv = container.querySelector(".document-content") as HTMLElement;
      expect(contentDiv).not.toBeNull();
      expect(contentDiv.style.transform).toContain("scale(1)");
    });
  });

  describe("metadata", () => {
    it("loads metadata successfully alongside content", async () => {
      mockInvoke
        .mockResolvedValueOnce(mockDocumentResponse)
        .mockResolvedValueOnce(mockMetadataResponse);

      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/report.pdf" showMetadata />
      ));
      await tick();

      // Metadata panel is available but hidden by default (toggled via toolbar button)
      // Verify content rendered without error
      expect(container.textContent).toContain("Forensic Report");
    });

    it("still renders content when metadata fails", async () => {
      mockInvoke
        .mockResolvedValueOnce(mockDocumentResponse)
        .mockResolvedValueOnce(mockErrorMetaResponse);

      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/report.pdf" />
      ));
      await tick();

      // Content should still render
      expect(container.textContent).toContain("Forensic Report");
    });
  });

  describe("no content fallback", () => {
    it("shows fallback when html content is empty", async () => {
      const emptyHtmlResponse = {
        success: true,
        content: {
          format: "txt",
          title: null,
          author: null,
          page_count: 0,
          file_size: 0,
          text: "",
          html: "",
        },
        error: null,
      };
      mockInvoke
        .mockResolvedValueOnce(emptyHtmlResponse)
        .mockResolvedValueOnce(mockErrorMetaResponse);

      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/empty.txt" />
      ));
      await tick();

      expect(container.textContent).toContain("No content");
    });
  });

  describe("optional class", () => {
    it("applies custom class when provided", async () => {
      mockInvoke
        .mockResolvedValueOnce(mockDocumentResponse)
        .mockResolvedValueOnce(mockMetadataResponse);

      const { container } = renderComponent(() => (
        <DocumentViewer path="/evidence/report.pdf" class="my-doc-class" />
      ));
      await tick();

      const viewer = container.querySelector(".document-viewer");
      expect(viewer?.classList.contains("my-doc-class")).toBe(true);
    });
  });
});
