// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, onMount, onCleanup, Accessor, JSX, Show } from "solid-js";
import { HiOutlineFolder, HiOutlineArrowDownTray } from "./icons";

export interface DragDropOptions {
  /** File type filters (e.g., ['.E01', '.ad1', '.zip']) */
  accept?: string[];
  /** Allow multiple files */
  multiple?: boolean;
  /** Allow directory drops */
  allowDirectories?: boolean;
  /** Called when files are dropped */
  onDrop?: (files: File[], paths?: string[]) => void;
  /** Called when drag enters the zone */
  onDragEnter?: () => void;
  /** Called when drag leaves the zone */
  onDragLeave?: () => void;
  /** Disabled state */
  disabled?: boolean;
}

export interface DragDropState {
  isDragging: Accessor<boolean>;
  isOver: Accessor<boolean>;
  dragCount: Accessor<number>;
}

/**
 * Hook for drag and drop functionality
 */
export function useDragDrop(
  containerRef: Accessor<HTMLElement | undefined>,
  options: DragDropOptions = {}
): DragDropState {
  const [isDragging, setIsDragging] = createSignal(false);
  const [isOver, setIsOver] = createSignal(false);
  const [dragCount, setDragCount] = createSignal(0);

  // Track global drag state
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

    // Check if over our container
    const container = containerRef();
    if (container && container.contains(e.target as Node)) {
      setIsOver(true);
      options.onDragEnter?.();
      
      // Set drop effect
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

    // Handle files
    if (dataTransfer.files && dataTransfer.files.length > 0) {
      for (let i = 0; i < dataTransfer.files.length; i++) {
        const file = dataTransfer.files[i];
        
        // Filter by accepted types
        if (options.accept && options.accept.length > 0) {
          const ext = `.${file.name.split(".").pop()?.toLowerCase()}`;
          if (!options.accept.some(a => a.toLowerCase() === ext || a === "*")) {
            continue;
          }
        }
        
        files.push(file);
        
        // Try to get file path (Tauri/Electron)
        // @ts-expect-error - path is available in some environments
        if (file.path) {
          // @ts-expect-error
          paths.push(file.path);
        }
      }
    }

    // Handle items (for directories in some browsers)
    if (dataTransfer.items && options.allowDirectories) {
      for (let i = 0; i < dataTransfer.items.length; i++) {
        const item = dataTransfer.items[i];
        if (item.kind === "file") {
          const entry = item.webkitGetAsEntry?.();
          if (entry?.isDirectory) {
            // Handle directory - would need recursive reading
            console.log("Directory dropped:", entry.name);
          }
        }
      }
    }

    // Limit to single file if not multiple
    const finalFiles = options.multiple ? files : files.slice(0, 1);
    const finalPaths = options.multiple ? paths : paths.slice(0, 1);

    if (finalFiles.length > 0) {
      options.onDrop?.(finalFiles, finalPaths.length > 0 ? finalPaths : undefined);
    }
  };

  onMount(() => {
    // Use document for global drag tracking
    document.addEventListener("dragenter", handleDragEnter);
    document.addEventListener("dragleave", handleDragLeave);
    document.addEventListener("dragover", handleDragOver);
    document.addEventListener("drop", handleDrop);
  });

  onCleanup(() => {
    document.removeEventListener("dragenter", handleDragEnter);
    document.removeEventListener("dragleave", handleDragLeave);
    document.removeEventListener("dragover", handleDragOver);
    document.removeEventListener("drop", handleDrop);
  });

  return {
    isDragging,
    isOver,
    dragCount,
  };
}

// ============================================================================
// DropZone Component
// ============================================================================

export interface DropZoneProps {
  /** Called when files are dropped */
  onDrop: (files: File[], paths?: string[]) => void;
  /** Accepted file extensions */
  accept?: string[];
  /** Allow multiple files */
  multiple?: boolean;
  /** Disabled state */
  disabled?: boolean;
  /** Custom class for the container */
  class?: string;
  /** Custom content when not dragging */
  children?: JSX.Element;
  /** Icon to show */
  icon?: string;
  /** Title text */
  title?: string;
  /** Description text */
  description?: string;
}

/**
 * Drop Zone component for file uploads
 */
export function DropZone(props: DropZoneProps) {
  let containerRef: HTMLDivElement | undefined;
  
  const { isDragging, isOver } = useDragDrop(
    () => containerRef,
    {
      accept: props.accept,
      multiple: props.multiple,
      disabled: props.disabled,
      onDrop: props.onDrop,
    }
  );

  // Also support click to browse
  let inputRef: HTMLInputElement | undefined;

  const handleClick = () => {
    if (!props.disabled) {
      inputRef?.click();
    }
  };

  const handleFileSelect = (e: Event) => {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      const files = Array.from(input.files);
      props.onDrop(files);
      input.value = ""; // Reset for next selection
    }
  };

  return (
    <div
      ref={containerRef}
      class={`
        relative rounded-lg border-2 border-dashed transition-all duration-200 cursor-pointer
        ${isOver() 
          ? "border-cyan-500 bg-cyan-500/10 scale-[1.02]" 
          : isDragging()
            ? "border-cyan-600/50 bg-cyan-600/5"
            : "border-zinc-600 hover:border-zinc-500 bg-zinc-800/30 hover:bg-zinc-800/50"
        }
        ${props.disabled ? "opacity-50 cursor-not-allowed" : ""}
        ${props.class ?? ""}
      `}
      onClick={handleClick}
      role="button"
      aria-label="Drop files here or click to browse"
      tabIndex={props.disabled ? -1 : 0}
      onKeyDown={(e) => e.key === "Enter" && handleClick()}
    >
      {/* Hidden file input */}
      <input
        ref={inputRef}
        type="file"
        class="hidden"
        accept={props.accept?.join(",")}
        multiple={props.multiple}
        onChange={handleFileSelect}
        disabled={props.disabled}
      />

      {/* Content */}
      <Show when={props.children} fallback={
        <div class="flex flex-col items-center justify-center gap-3 p-8 text-center">
          <div class={`text-4xl transition-transform duration-200 ${isOver() ? "scale-125" : ""}`}>
            {props.icon ?? <HiOutlineFolder class="w-10 h-10 text-cyan-400" />}
          </div>
          <div>
            <p class="text-sm font-medium text-zinc-200">
              {props.title ?? "Drop files here"}
            </p>
            <p class="text-xs text-zinc-400 mt-1">
              {props.description ?? "or click to browse"}
            </p>
          </div>
          <Show when={props.accept && props.accept.length > 0}>
            <p class="text-xs text-zinc-500">
              Accepted: {props.accept!.join(", ")}
            </p>
          </Show>
        </div>
      }>
        {props.children}
      </Show>

      {/* Drag overlay */}
      <Show when={isOver()}>
        <div class="absolute inset-0 rounded-lg bg-cyan-500/20 flex items-center justify-center pointer-events-none">
          <div class="text-cyan-400 font-medium flex items-center gap-2">
            <HiOutlineArrowDownTray class="w-6 h-6" />
            <span>Drop to add</span>
          </div>
        </div>
      </Show>
    </div>
  );
}

// ============================================================================
// Global Drop Overlay
// ============================================================================

export interface GlobalDropOverlayProps {
  /** Whether the overlay is active */
  active: boolean;
  /** Called when files are dropped */
  onDrop: (files: File[], paths?: string[]) => void;
  /** Accepted file extensions */
  accept?: string[];
}

/**
 * Full-screen drop overlay that appears when dragging files over the app
 */
export function GlobalDropOverlay(props: GlobalDropOverlayProps) {
  return (
    <Show when={props.active}>
      <div class="fixed inset-0 z-[9999] bg-black/60 backdrop-blur-sm flex items-center justify-center pointer-events-none">
        <div class="bg-zinc-900 border-2 border-dashed border-cyan-500 rounded-2xl p-12 text-center shadow-2xl animate-pulse">
          <div class="text-6xl mb-4">📥</div>
          <h2 class="text-xl font-semibold text-zinc-100">Drop files to add evidence</h2>
          <p class="text-sm text-zinc-400 mt-2">
            Release to add files to your project
          </p>
          <Show when={props.accept && props.accept.length > 0}>
            <p class="text-xs text-zinc-500 mt-4">
              Supported formats: {props.accept!.join(", ")}
            </p>
          </Show>
        </div>
      </div>
    </Show>
  );
}

export default DropZone;
