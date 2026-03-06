// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, onMount, type Accessor } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";
import { logger } from "../../utils/logger";
import type { DragDropOptions, DragDropState } from "./types";

/**
 * Hook for drag and drop functionality
 */
export function useDragDrop(
  containerRef: Accessor<HTMLElement | undefined>,
  options: DragDropOptions = {},
): DragDropState {
  const [isDragging, setIsDragging] = createSignal(false);
  const [isOver, setIsOver] = createSignal(false);
  const [dragCount, setDragCount] = createSignal(0);

  let globalDragCount = 0;

  const handleDragEnter = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (options.disabled) return;

    globalDragCount++;
    setDragCount(globalDragCount);
    if (globalDragCount === 1) {
      setIsDragging(true);
    }
  };

  const handleDragLeave = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (options.disabled) return;

    globalDragCount--;
    setDragCount(globalDragCount);
    if (globalDragCount === 0) {
      setIsDragging(false);
      setIsOver(false);
    }
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (options.disabled) return;

    const container = containerRef();
    if (container && container.contains(e.target as Node)) {
      setIsOver(true);
      options.onDragEnter?.();
      if (e.dataTransfer) {
        e.dataTransfer.dropEffect = "copy";
      }
    } else {
      setIsOver(false);
      options.onDragLeave?.();
    }
  };

  const handleDrop = async (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();

    globalDragCount = 0;
    setDragCount(0);
    setIsDragging(false);
    setIsOver(false);

    if (options.disabled) return;

    const container = containerRef();
    if (!container || !container.contains(e.target as Node)) return;

    const dataTransfer = e.dataTransfer;
    if (!dataTransfer) return;

    const files: File[] = [];
    const paths: string[] = [];

    if (dataTransfer.files && dataTransfer.files.length > 0) {
      for (let i = 0; i < dataTransfer.files.length; i++) {
        const file = dataTransfer.files[i];

        if (options.accept && options.accept.length > 0) {
          const ext = `.${file.name.split(".").pop()?.toLowerCase()}`;
          if (!options.accept.some((a) => a.toLowerCase() === ext || a === "*")) {
            continue;
          }
        }

        files.push(file);

        // @ts-expect-error - path is available in some environments
        if (file.path) {
          // @ts-expect-error
          paths.push(file.path);
        }
      }
    }

    if (dataTransfer.items && options.allowDirectories) {
      for (let i = 0; i < dataTransfer.items.length; i++) {
        const item = dataTransfer.items[i];
        if (item.kind === "file") {
          const entry = item.webkitGetAsEntry?.();
          if (entry?.isDirectory) {
            logger.debug("Directory dropped:", entry.name);
          }
        }
      }
    }

    const finalFiles = options.multiple ? files : files.slice(0, 1);
    const finalPaths = options.multiple ? paths : paths.slice(0, 1);

    if (finalFiles.length > 0) {
      options.onDrop?.(finalFiles, finalPaths.length > 0 ? finalPaths : undefined);
    }
  };

  onMount(() => {
    makeEventListener(document, "dragenter", handleDragEnter);
    makeEventListener(document, "dragleave", handleDragLeave);
    makeEventListener(document, "dragover", handleDragOver);
    makeEventListener(document, "drop", handleDrop);
  });

  return { isDragging, isOver, dragCount };
}
