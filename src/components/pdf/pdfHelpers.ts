// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { getDocument, type PDFDocumentProxy, type PDFPageProxy } from "pdfjs-dist";
import { invoke } from "@tauri-apps/api/core";
import { logger } from "../../utils/logger";
import type { HashSourceInput } from "../../api/commands";
const log = logger.scope("PdfHelpers");

/**
 * Load a PDF document from a file path via Tauri command
 * Reads in base64 chunks and converts to Uint8Array for PDF.js.
 */
export async function loadPdfDocument(
  path: string,
  source?: HashSourceInput | null
): Promise<PDFDocumentProxy> {
  const bytes = await readBinaryFileChunked(path, source);

  const loadingTask = getDocument({ data: bytes });
  return await loadingTask.promise;
}

interface BinaryInfo {
  size: number;
}

interface BinaryBase64Chunk {
  offset: number;
  bytesRead: number;
  totalSize: number;
  eof: boolean;
  data: string;
}

const PDF_READ_CHUNK_SIZE = 4 * 1024 * 1024;

async function readBinaryFileChunked(
  path: string,
  source?: HashSourceInput | null
): Promise<Uint8Array> {
  const info = source
    ? await invoke<BinaryInfo>("viewer_get_binary_info_source", { source })
    : await invoke<BinaryInfo>("viewer_get_binary_info", { path });
  if (!Number.isSafeInteger(info.size)) {
    throw new Error(`PDF is too large to load in this browser process: ${info.size} bytes`);
  }

  const bytes = new Uint8Array(info.size);
  let offset = 0;

  while (offset < info.size) {
    const chunkSize = Math.min(PDF_READ_CHUNK_SIZE, info.size - offset);
    const chunk = source
      ? await invoke<BinaryBase64Chunk>("viewer_read_binary_source_base64_chunk", {
          source,
          offset,
          size: chunkSize,
        })
      : await invoke<BinaryBase64Chunk>("viewer_read_binary_base64_chunk", {
          path,
          offset,
          size: chunkSize,
        });

    if (chunk.offset !== offset) {
      throw new Error(`Unexpected PDF chunk offset: expected ${offset}, received ${chunk.offset}`);
    }
    if (chunk.bytesRead === 0 && !chunk.eof) {
      throw new Error(`PDF read stalled at offset ${offset}`);
    }

    const binaryString = atob(chunk.data);
    if (binaryString.length !== chunk.bytesRead) {
      throw new Error(`PDF chunk size mismatch at offset ${offset}`);
    }

    for (let i = 0; i < binaryString.length; i++) {
      bytes[offset + i] = binaryString.charCodeAt(i);
    }

    offset += chunk.bytesRead;
    if (chunk.eof) break;
  }

  return bytes;
}

/**
 * Render a PDF page to a canvas with proper scaling and high DPI support
 */
export async function renderPdfPage(
  page: PDFPageProxy,
  canvas: HTMLCanvasElement,
  containerWidth: number,
  scale: number
): Promise<void> {
  // Calculate scale to fit container width
  const viewport = page.getViewport({ scale: 1 });
  const fitScale = Math.max(0.5, (containerWidth - 80) / viewport.width);
  const actualScale = scale * fitScale;
  
  const scaledViewport = page.getViewport({ scale: actualScale });

  // Set canvas dimensions with device pixel ratio for crisp rendering
  const pixelRatio = window.devicePixelRatio || 1;
  const canvasWidth = Math.floor(scaledViewport.width * pixelRatio);
  const canvasHeight = Math.floor(scaledViewport.height * pixelRatio);
  
  canvas.width = canvasWidth;
  canvas.height = canvasHeight;
  canvas.style.width = `${Math.floor(scaledViewport.width)}px`;
  canvas.style.height = `${Math.floor(scaledViewport.height)}px`;

  const context = canvas.getContext("2d");
  if (!context) {
    throw new Error("Failed to get canvas 2d context");
  }

  // Scale context for high DPI displays
  context.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);

  // Clear canvas before render
  context.fillStyle = "white";
  context.fillRect(0, 0, scaledViewport.width, scaledViewport.height);

  // Render PDF page
  const renderContext = {
    canvasContext: context,
    viewport: scaledViewport,
  };

  await page.render(renderContext).promise;
}

/**
 * Generate a thumbnail for a single PDF page
 */
export async function generateThumbnail(
  pdf: PDFDocumentProxy,
  pageNum: number,
  thumbScale: number = 0.15
): Promise<string> {
  try {
    const page = await pdf.getPage(pageNum);
    const viewport = page.getViewport({ scale: thumbScale });
    
    const canvas = document.createElement("canvas");
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    
    const context = canvas.getContext("2d");
    if (!context) {
      return "";
    }

    await page.render({
      canvasContext: context,
      viewport: viewport,
    }).promise;
    
    return canvas.toDataURL("image/jpeg", 0.6);
  } catch (e) {
    log.error(`Failed to generate thumbnail for page ${pageNum}:`, e);
    return "";
  }
}

/**
 * Generate thumbnails for all pages in batches to avoid blocking
 */
export async function generateThumbnailsBatch(
  pdf: PDFDocumentProxy,
  batchSize: number = 3,
  onBatchComplete?: (startIndex: number, thumbnails: string[]) => void
): Promise<string[]> {
  const thumbScale = 0.15;
  const allThumbnails: string[] = new Array(pdf.numPages).fill("");

  for (let batch = 0; batch < pdf.numPages; batch += batchSize) {
    const batchEnd = Math.min(batch + batchSize, pdf.numPages);
    const batchPromises: Promise<{ index: number; dataUrl: string }>[] = [];
    
    for (let i = batch; i < batchEnd; i++) {
      batchPromises.push(
        (async () => {
          const dataUrl = await generateThumbnail(pdf, i + 1, thumbScale);
          return { index: i, dataUrl };
        })()
      );
    }
    
    // Wait for batch to complete
    const results = await Promise.all(batchPromises);
    
    // Update thumbnails
    for (const result of results) {
      allThumbnails[result.index] = result.dataUrl;
    }
    
    // Notify callback with batch results
    if (onBatchComplete) {
      const batchThumbs = results.map(r => r.dataUrl);
      onBatchComplete(batch, batchThumbs);
    }
    
    // Yield to main thread between batches
    await new Promise((resolve) => setTimeout(resolve, 10));
  }

  return allThumbnails;
}
