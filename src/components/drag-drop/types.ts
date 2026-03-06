// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Accessor, JSX } from "solid-js";

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

export interface GlobalDropOverlayProps {
  /** Whether the overlay is active */
  active: boolean;
  /** Called when files are dropped */
  onDrop: (files: File[], paths?: string[]) => void;
  /** Accepted file extensions */
  accept?: string[];
}
