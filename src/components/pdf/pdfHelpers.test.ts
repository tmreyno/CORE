import { beforeEach, describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../__tests__/setup";

const mockGetDocument = vi.fn();

vi.mock("pdfjs-dist", () => ({
  getDocument: (...args: any[]) => mockGetDocument(...args),
}));

import { loadPdfDocument } from "./pdfHelpers";

describe("pdfHelpers", () => {
  beforeEach(() => {
    mockGetDocument.mockReset();
    mockGetDocument.mockReturnValue({
      promise: Promise.resolve({ numPages: 1 }),
    });
  });

  it("loads PDFs through bounded range chunks", async () => {
    mockInvoke
      .mockResolvedValueOnce({
        path: "/evidence/report.pdf",
        size: 5,
        maxInlineBytes: 100,
        supportsRangeReads: true,
      })
      .mockResolvedValueOnce({
        offset: 0,
        bytesRead: 5,
        totalSize: 5,
        eof: true,
        data: "aGVsbG8=",
      });

    await expect(loadPdfDocument("/evidence/report.pdf")).resolves.toMatchObject({
      numPages: 1,
    });

    expect(mockInvoke).toHaveBeenCalledWith("viewer_get_binary_info", {
      path: "/evidence/report.pdf",
    });
    expect(mockInvoke).toHaveBeenCalledWith("viewer_read_binary_base64_chunk", {
      path: "/evidence/report.pdf",
      offset: 0,
      size: 5,
    });
    expect(mockGetDocument).toHaveBeenCalledWith({
      data: new Uint8Array([104, 101, 108, 108, 111]),
    });
  });

  it("rejects oversized local PDFs before allocating the byte buffer", async () => {
    mockInvoke.mockResolvedValueOnce({
      path: "/evidence/huge.pdf",
      size: 101 * 1024 * 1024,
      maxInlineBytes: 100 * 1024 * 1024,
      supportsRangeReads: true,
    });

    await expect(loadPdfDocument("/evidence/huge.pdf")).rejects.toThrow(
      "PDF is too large for inline preview",
    );

    expect(mockInvoke).toHaveBeenCalledWith("viewer_get_binary_info", {
      path: "/evidence/huge.pdf",
    });
    expect(mockInvoke).not.toHaveBeenCalledWith(
      "viewer_read_binary_base64_chunk",
      expect.anything(),
    );
    expect(mockGetDocument).not.toHaveBeenCalled();
  });

  it("rejects oversized source PDFs before range reads", async () => {
    const source = {
      containerPath: "/evidence/image.ad1",
      entryPath: "docs/huge.pdf",
      containerType: "ad1",
    };
    mockInvoke.mockResolvedValueOnce({
      path: "ad1:/evidence/image.ad1:docs/huge.pdf",
      size: 101 * 1024 * 1024,
      maxInlineBytes: 100 * 1024 * 1024,
      supportsRangeReads: true,
    });

    await expect(loadPdfDocument("/evidence/huge.pdf", source)).rejects.toThrow(
      "PDF is too large for inline preview",
    );

    expect(mockInvoke).toHaveBeenCalledWith("viewer_get_binary_info_source", {
      source,
    });
    expect(mockInvoke).not.toHaveBeenCalledWith(
      "viewer_read_binary_source_base64_chunk",
      expect.anything(),
    );
    expect(mockGetDocument).not.toHaveBeenCalled();
  });
});
