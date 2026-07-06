// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ImageViewer - Simple image viewer that loads images via Tauri backend
 * 
 * Uses ranged backend reads and Blob URLs to bypass file:// protocol restrictions in Tauri 2
 */

import { createSignal, createEffect, Show, createMemo, onCleanup } from "solid-js";
import { CoreSpinner } from "@core-suite/icons";
import { getBasename } from "../utils/pathUtils";
import { commands, type HashSourceInput } from "../api/commands";
import {
  HiOutlineMagnifyingGlassPlus,
  HiOutlineMagnifyingGlassMinus,
  HiOutlineExclamationTriangle,
  HiOutlineArrowsPointingOut,
} from "./icons";
import { logger } from "../utils/logger";
import { isTauri } from "../utils/platform";
const log = logger.scope("ImageViewer");

// ============================================================================
// Types
// ============================================================================

interface ImageViewerProps {
  /** Path to the image file */
  path: string;
  /** Optional evidence source for container or nested image entries */
  source?: HashSourceInput | null;
  /** Optional class name */
  class?: string;
}

/** Get mime type from extension */
function getMimeType(path: string): string {
  const ext = path.split('.').pop()?.toLowerCase() || '';
  const mimeTypes: Record<string, string> = {
    'jpg': 'image/jpeg',
    'jpeg': 'image/jpeg',
    'png': 'image/png',
    'gif': 'image/gif',
    'bmp': 'image/bmp',
    'webp': 'image/webp',
    'svg': 'image/svg+xml',
    'ico': 'image/x-icon',
    'tiff': 'image/tiff',
    'tif': 'image/tiff',
    'heic': 'image/heic',
    'heif': 'image/heif',
    'avif': 'image/avif',
    // RAW camera formats (limited browser support)
    'raw': 'image/x-raw',
    'cr2': 'image/x-canon-cr2',
    'nef': 'image/x-nikon-nef',
    'arw': 'image/x-sony-arw',
    'dng': 'image/x-adobe-dng',
    'orf': 'image/x-olympus-orf',
    'rw2': 'image/x-panasonic-rw2',
  };
  return mimeTypes[ext] || 'image/png';
}

/**
 * Extensions whose MIME types may not be natively supported by the WebView
 * engine. HEIC/HEIF require platform-specific codecs, TIFF support varies,
 * and RAW camera formats are generally not renderable in browsers.
 */
const LIMITED_SUPPORT_EXTENSIONS = new Set([
  'heic', 'heif', 'tiff', 'tif',
  'raw', 'cr2', 'nef', 'arw', 'dng', 'orf', 'rw2',
]);
const MAX_INLINE_IMAGE_BYTES = 100 * 1024 * 1024;
const IMAGE_READ_CHUNK_SIZE = 3 * 1024 * 1024;

interface BinaryInfo {
  size: number;
  maxInlineBytes?: number;
}

async function getImageBinaryInfo(path: string, source?: HashSourceInput | null): Promise<BinaryInfo> {
  if (typeof source?.size === "number") {
    return {
      size: source.size,
      maxInlineBytes: MAX_INLINE_IMAGE_BYTES,
    };
  }
  return source
    ? await commands.viewer.getBinaryInfoSource(source)
    : await commands.viewer.getBinaryInfo(path);
}

async function readImageBlobUrl(
  path: string,
  source: HashSourceInput | null | undefined,
  mimeType: string,
): Promise<{ url: string; size: number }> {
  const info = await getImageBinaryInfo(path, source);
  if (!Number.isSafeInteger(info.size)) {
    throw new Error(`Image is too large to load in this browser process: ${info.size} bytes`);
  }

  const maxInlineBytes = info.maxInlineBytes ?? MAX_INLINE_IMAGE_BYTES;
  if (info.size > maxInlineBytes) {
    throw new Error(
      `Image is too large for inline preview: ${info.size} bytes > ${maxInlineBytes} bytes. Use hex or export the file for external viewing.`,
    );
  }

  const chunks: Uint8Array[] = [];
  let offset = 0;
  while (offset < info.size) {
    const size = Math.min(IMAGE_READ_CHUNK_SIZE, info.size - offset);
    const chunk = source
      ? await commands.viewer.readBinarySourceBase64Chunk(source, offset, size)
      : await commands.viewer.readBinaryBase64Chunk(path, offset, size);

    if (chunk.offset !== offset) {
      throw new Error(`Unexpected image chunk offset: expected ${offset}, received ${chunk.offset}`);
    }
    if (chunk.bytesRead === 0 && !chunk.eof) {
      throw new Error(`Image read stalled at offset ${offset}`);
    }

    const bytes = base64ToBytes(chunk.data);
    if (bytes.length !== chunk.bytesRead) {
      throw new Error(`Image chunk size mismatch at offset ${offset}`);
    }
    chunks.push(bytes);
    offset += chunk.bytesRead;
    if (chunk.eof) break;
  }

  return {
    url: URL.createObjectURL(new Blob(chunks, { type: mimeType })),
    size: info.size,
  };
}

function base64ToBytes(data: string): Uint8Array {
  const binaryString = atob(data);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes;
}

// ============================================================================
// Component
// ============================================================================

export function ImageViewer(props: ImageViewerProps) {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [imageSrc, setImageSrc] = createSignal<string | null>(null);
  const [scale, setScale] = createSignal(1.0);
  const [naturalSize, setNaturalSize] = createSignal<{ width: number; height: number } | null>(null);

  let containerRef: HTMLDivElement | undefined;
  let loadGeneration = 0;
  let objectUrl: string | null = null;

  const setImageObjectUrl = (url: string | null) => {
    if (objectUrl && objectUrl !== url) {
      URL.revokeObjectURL(objectUrl);
    }
    objectUrl = url;
    setImageSrc(url);
  };

  const isCurrentImageEvent = (img: HTMLImageElement) => {
    const currentSrc = imageSrc();
    if (!currentSrc) return false;
    const eventSrc = img.currentSrc || img.src;
    return eventSrc === currentSrc;
  };

  onCleanup(() => {
    if (objectUrl) {
      URL.revokeObjectURL(objectUrl);
      objectUrl = null;
    }
  });

  // Memoized values to avoid recalculation
  const filename = createMemo(() => getBasename(props.path) || props.path);
  const extension = createMemo(() => props.path.split('.').pop()?.toLowerCase() || '');
  const hasLimitedSupport = createMemo(() => LIMITED_SUPPORT_EXTENSIONS.has(extension()));
  const zoomPercent = createMemo(() => Math.round(scale() * 100));
  const dimensionText = createMemo(() => {
    const size = naturalSize();
    return size ? `${size.width} × ${size.height}` : null;
  });
  const transformStyle = createMemo(() => ({
    transform: `scale(${scale()})`,
    "transform-origin": "center center" as const,
  }));

  // Load image as base64
  const loadImage = async () => {
    const generation = ++loadGeneration;
    const requestedPath = props.path;
    const requestedSource = props.source;
    const requestedMimeType = getMimeType(requestedPath);

    setLoading(true);
    setError(null);
    setScale(1.0);
    setNaturalSize(null);

    try {
      if (!isTauri) {
        throw new Error("Image evidence viewing is available in the desktop app.");
      }

      const { url } = await readImageBlobUrl(requestedPath, requestedSource, requestedMimeType);

      if (generation !== loadGeneration) {
        URL.revokeObjectURL(url);
        return;
      }
      setImageObjectUrl(url);
    } catch (e) {
      if (generation !== loadGeneration) return;
      log.error("Failed to load image:", e);
      setError(e instanceof Error ? e.message : String(e));
      setImageObjectUrl(null);
    } finally {
      if (generation !== loadGeneration) return;
      setLoading(false);
    }
  };

  // Load image when path changes
  createEffect(() => {
    const path = props.path;
    const source = props.source;
    if (path || source) {
      void loadImage();
    }
  });

  // Zoom controls
  const zoomIn = () => setScale(s => Math.min(s + 0.25, 5.0));
  const zoomOut = () => setScale(s => Math.max(s - 0.25, 0.1));
  const resetZoom = () => setScale(1.0);
  const fitToView = () => {
    const size = naturalSize();
    if (size && containerRef) {
      const rect = containerRef.getBoundingClientRect();
      const containerWidth = rect.width - 32; // account for padding
      const containerHeight = rect.height - 32;
      const scaleX = containerWidth / size.width;
      const scaleY = containerHeight / size.height;
      setScale(Math.min(scaleX, scaleY, 1.0));
    }
  };

  // Wheel zoom handler
  const handleWheel = (e: WheelEvent) => {
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      const delta = e.deltaY > 0 ? -0.1 : 0.1;
      setScale(s => Math.max(0.1, Math.min(s + delta, 5.0)));
    }
  };

  return (
    <div class={`image-viewer flex flex-col h-full ${props.class || ""}`}>
      {/* Toolbar */}
      <div class="image-toolbar flex items-center gap-2 p-2 border-b border-border bg-bg-secondary">
        {/* File info */}
        <div class="flex items-center gap-2 text-sm">
          <span class="font-medium truncate max-w-[200px]" title={filename()}>{filename()}</span>
          <Show when={dimensionText()}>
            <span class="text-txt-muted">
              {dimensionText()}
            </span>
          </Show>
          <Show when={hasLimitedSupport()}>
            <span class="text-xs text-warning" title={`${extension().toUpperCase()} format has limited browser support`}>
              ⚠ Limited format support
            </span>
          </Show>
        </div>

        <div class="flex-1" />

        {/* Zoom controls */}
        <div class="flex items-center gap-1">
          <button
            onClick={zoomOut}
            class="p-1.5 rounded hover:bg-bg-hover"
            title="Zoom out"
          >
            <HiOutlineMagnifyingGlassMinus class="w-5 h-5" />
          </button>
          <span class="text-sm w-14 text-center">{zoomPercent()}%</span>
          <button
            onClick={zoomIn}
            class="p-1.5 rounded hover:bg-bg-hover"
            title="Zoom in"
          >
            <HiOutlineMagnifyingGlassPlus class="w-5 h-5" />
          </button>
          <button
            onClick={resetZoom}
            class="text-xs px-2 py-1 rounded hover:bg-bg-hover"
          >
            100%
          </button>
          <button
            onClick={fitToView}
            class="p-1.5 rounded hover:bg-bg-hover"
            title="Fit to view"
          >
            <HiOutlineArrowsPointingOut class="w-5 h-5" />
          </button>
        </div>
      </div>

      {/* Content */}
      <div
        ref={containerRef}
        class="flex-1 overflow-auto bg-bg-dark flex items-center justify-center"
        onWheel={handleWheel}
      >
        <Show
          when={!loading()}
          fallback={
            <div class="flex flex-col items-center gap-2">
              <CoreSpinner size={32} />
              <span class="text-txt-muted">Loading image...</span>
            </div>
          }
        >
          <Show
            when={!error()}
            fallback={
              <div class="flex flex-col items-center gap-2 text-error p-4">
                <HiOutlineExclamationTriangle class="w-12 h-12" />
                <span class="font-medium">Failed to load image</span>
                <span class="text-sm text-txt-muted">{error()}</span>
                <button
                  onClick={loadImage}
                  class="btn-sm-primary mt-2"
                >
                  Retry
                </button>
              </div>
            }
          >
            <img
              src={imageSrc() || ''}
              alt={filename()}
              class="max-w-none"
              style={transformStyle()}
              onLoad={(e) => {
                const img = e.currentTarget as HTMLImageElement;
                if (!isCurrentImageEvent(img)) return;
                setNaturalSize({ width: img.naturalWidth, height: img.naturalHeight });
              }}
              onError={(e) => {
                const img = e.currentTarget as HTMLImageElement;
                if (!isCurrentImageEvent(img)) return;
                if (hasLimitedSupport()) {
                  setError(`This image format (.${extension()}) may not be supported by the built-in viewer. Try exporting and opening with an external application.`);
                } else {
                  setError("Failed to decode image data");
                }
              }}
              draggable={false}
            />
          </Show>
        </Show>
      </div>
    </div>
  );
}
