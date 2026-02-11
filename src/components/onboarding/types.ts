// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { JSX } from "solid-js";

export interface TourStep {
  /** Unique ID for the step */
  id: string;
  /** Title of the step */
  title: string;
  /** Description/content */
  content: string | JSX.Element;
  /** CSS selector for the target element to highlight */
  target?: string;
  /** Position of the tooltip relative to target */
  position?: "top" | "bottom" | "left" | "right" | "center";
  /** Action buttons */
  actions?: {
    primary?: { label: string; onClick: () => void };
    secondary?: { label: string; onClick: () => void };
  };
  /** Callback when this step is shown */
  onShow?: () => void;
  /** Allow interaction with target element */
  allowInteraction?: boolean;
}

export interface TooltipConfig {
  /** Tooltip content */
  content: string | JSX.Element;
  /** Trigger events */
  trigger?: "hover" | "click" | "focus";
  /** Position relative to trigger */
  position?: "top" | "bottom" | "left" | "right";
  /** Delay before showing (ms) */
  delay?: number;
  /** Additional class names */
  class?: string;
}

export interface UseTourOptions {
  /** Tour steps */
  steps: TourStep[];
  /** Storage key for completion status */
  storageKey?: string;
  /** Auto-start on first visit */
  autoStart?: boolean;
  /** Callback when tour completes */
  onComplete?: () => void;
  /** Callback when tour is skipped */
  onSkip?: () => void;
}
