// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ImageViewer - Simple image viewer that loads images via Tauri backend
 * 
 * Uses base64 encoding to bypass file:// protocol restrictions in Tauri 2
 */

import { createSignal, createEffect, Show, createMemo } from "solid-js";
import { getBasename } from "../utils/pathUtils";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineMagnifyingGlassPlus,
  HiOutlineMagnifyingGlassMinus,
  HiOutlineExclamationTriangle,
  HiOutlineArrowsPointingOut,
} from "./icons";
import { logger } from "../utils/logger";
const log = logger.scope("ImageViewer");

// ============================================================================
// Types
// ============================================================================

interface ImageViewerProps {
  /** Path to the image file */
  path: string;
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

// ============================================================================
// Component
// ============================================================================

export function ImageViewer(props: ImageViewerProps) {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [imageSrc, setImageSrc] = createSignal<string | null>(null);
  const [scale, setScale] = createSignal(1.0);
  const [naturalSize, setNaturalSize] = createSignal<{ width: number; height: number } | null>(null);

  // Memoized values to avoid recalculation
  const filename = createMemo(() => getBasename(props.path) || props.path);
  const extension = createMemo(() => props.path.split('.').pop()?.toLowerCase() || '');
  const mimeType = createMemo(() => getMimeType(props.path));
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
    setLoading(true);
    setError(null);

    try {
      const base64Data = await invoke<string>("viewer_read_binary_base64", { path: props.path });
      setImageSrc(`data:${mimeType()};base64,${base64Data}`);
    } catch (e) {
      log.error("Failed to load image:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  // Load image when path changes
  createEffect(() => {
    if (props.path) {
      loadImage();
    }
  });

  // Zoom controls
  const zoomIn = () => setScale(s => Math.min(s + 0.25, 5.0));
  const zoomOut = () => setScale(s => Math.max(s - 0.25, 0.1));
  const resetZoom = () => setScale(1.0);
  const fitToView = () => {
    // Calculate scale to fit the image in view
    const size = naturalSize();
    if (size) {
      const containerWidth = 800; // approximate
      const containerHeight = 600;
      const scaleX = containerWidth / size.width;
      const scaleY = containerHeight / size.height;
      setScale(Math.min(scaleX, scaleY, 1.0));
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
      <div class="flex-1 overflow-auto bg-bg-dark flex items-center justify-center">
        <Show
          when={!loading()}
          fallback={
            <div class="flex flex-col items-center gap-2">
              <div class="animate-spin w-8 h-8 border-2 border-accent border-t-transparent rounded-full" />
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
                setNaturalSize({ width: img.naturalWidth, height: img.naturalHeight });
              }}
              onError={() => {
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
