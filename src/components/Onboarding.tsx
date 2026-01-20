// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, Show, onMount, onCleanup, JSX, createEffect } from "solid-js";
import { Portal } from "solid-js/web";
import {
  HiOutlineFolder,
  HiOutlineLockClosed,
  HiOutlineChartBar,
  HiOutlineDocumentText,
} from "./icons";

// ============================================================================
// Types
// ============================================================================

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

// ============================================================================
// Onboarding Tour Hook
// ============================================================================

const TOUR_STORAGE_KEY = "ffx-tour-completed";

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

export function useTour(options: UseTourOptions) {
  const storageKey = options.storageKey ?? TOUR_STORAGE_KEY;
  
  const [isActive, setIsActive] = createSignal(false);
  const [currentStepIndex, setCurrentStepIndex] = createSignal(0);
  const [hasCompleted, setHasCompleted] = createSignal(false);

  // Check if tour was already completed
  onMount(() => {
    try {
      const completed = localStorage.getItem(storageKey);
      if (completed === "true") {
        setHasCompleted(true);
      } else if (options.autoStart) {
        start();
      }
    } catch (e) {}
  });

  const currentStep = () => options.steps[currentStepIndex()];
  const isFirstStep = () => currentStepIndex() === 0;
  const isLastStep = () => currentStepIndex() === options.steps.length - 1;
  const progress = () => ((currentStepIndex() + 1) / options.steps.length) * 100;

  const start = () => {
    setCurrentStepIndex(0);
    setIsActive(true);
    currentStep()?.onShow?.();
  };

  const next = () => {
    if (isLastStep()) {
      complete();
    } else {
      setCurrentStepIndex((i) => i + 1);
      currentStep()?.onShow?.();
    }
  };

  const previous = () => {
    if (!isFirstStep()) {
      setCurrentStepIndex((i) => i - 1);
      currentStep()?.onShow?.();
    }
  };

  const goTo = (index: number) => {
    if (index >= 0 && index < options.steps.length) {
      setCurrentStepIndex(index);
      currentStep()?.onShow?.();
    }
  };

  const skip = () => {
    setIsActive(false);
    options.onSkip?.();
  };

  const complete = () => {
    setIsActive(false);
    setHasCompleted(true);
    try {
      localStorage.setItem(storageKey, "true");
    } catch (e) {}
    options.onComplete?.();
  };

  const reset = () => {
    setHasCompleted(false);
    try {
      localStorage.removeItem(storageKey);
    } catch (e) {}
  };

  return {
    isActive,
    currentStep,
    currentStepIndex,
    isFirstStep,
    isLastStep,
    progress,
    hasCompleted,
    start,
    next,
    previous,
    goTo,
    skip,
    complete,
    reset,
  };
}

// ============================================================================
// Tour Overlay Component
// ============================================================================

export interface TourOverlayProps {
  /** Whether the tour is active */
  isActive: boolean;
  /** Current step */
  step: TourStep | undefined;
  /** Current step index */
  stepIndex: number;
  /** Total steps */
  totalSteps: number;
  /** Progress percentage */
  progress: number;
  /** Is first step */
  isFirst: boolean;
  /** Is last step */
  isLast: boolean;
  /** Callbacks */
  onNext: () => void;
  onPrevious: () => void;
  onSkip: () => void;
}

export function TourOverlay(props: TourOverlayProps) {
  const [targetRect, setTargetRect] = createSignal<DOMRect | null>(null);
  const [tooltipPosition, setTooltipPosition] = createSignal({ top: 0, left: 0 });

  // Find and highlight target element
  createEffect(() => {
    if (!props.isActive || !props.step?.target) {
      setTargetRect(null);
      return;
    }

    const target = document.querySelector(props.step.target);
    if (target) {
      const rect = target.getBoundingClientRect();
      setTargetRect(rect);
      
      // Calculate tooltip position
      const pos = calculateTooltipPosition(rect, props.step.position ?? "bottom");
      setTooltipPosition(pos);
    }
  });

  const calculateTooltipPosition = (
    rect: DOMRect,
    position: string
  ): { top: number; left: number } => {
    const margin = 16;
    const tooltipWidth = 320;
    const tooltipHeight = 200;

    switch (position) {
      case "top":
        return {
          top: rect.top - tooltipHeight - margin,
          left: rect.left + rect.width / 2 - tooltipWidth / 2,
        };
      case "bottom":
        return {
          top: rect.bottom + margin,
          left: rect.left + rect.width / 2 - tooltipWidth / 2,
        };
      case "left":
        return {
          top: rect.top + rect.height / 2 - tooltipHeight / 2,
          left: rect.left - tooltipWidth - margin,
        };
      case "right":
        return {
          top: rect.top + rect.height / 2 - tooltipHeight / 2,
          left: rect.right + margin,
        };
      default:
        return {
          top: window.innerHeight / 2 - tooltipHeight / 2,
          left: window.innerWidth / 2 - tooltipWidth / 2,
        };
    }
  };

  return (
    <Show when={props.isActive && props.step}>
      <Portal>
        {/* Backdrop */}
        <div class="fixed inset-0 z-[9998]">
          {/* Dark overlay with cutout for target */}
          <svg class="absolute inset-0 w-full h-full">
            <defs>
              <mask id="tour-mask">
                <rect width="100%" height="100%" fill="white" />
                <Show when={targetRect()}>
                  <rect
                    x={targetRect()!.left - 8}
                    y={targetRect()!.top - 8}
                    width={targetRect()!.width + 16}
                    height={targetRect()!.height + 16}
                    rx="8"
                    fill="black"
                  />
                </Show>
              </mask>
            </defs>
            <rect
              width="100%"
              height="100%"
              fill="rgba(0, 0, 0, 0.75)"
              mask="url(#tour-mask)"
            />
          </svg>

          {/* Highlight ring around target */}
          <Show when={targetRect()}>
            <div
              class="absolute border-2 border-accent rounded-lg animate-pulse pointer-events-none"
              style={{
                top: `${targetRect()!.top - 8}px`,
                left: `${targetRect()!.left - 8}px`,
                width: `${targetRect()!.width + 16}px`,
                height: `${targetRect()!.height + 16}px`,
              }}
            />
          </Show>
        </div>

        {/* Tooltip */}
        <div
          class="fixed z-[9999] w-80 bg-bg border border-border rounded-xl shadow-2xl overflow-hidden"
          style={{
            top: `${tooltipPosition().top}px`,
            left: `${tooltipPosition().left}px`,
          }}
        >
          {/* Progress bar */}
          <div class="h-1 bg-bg-panel">
            <div
              class="h-full bg-accent transition-all duration-300"
              style={{ width: `${props.progress}%` }}
            />
          </div>

          {/* Content */}
          <div class="p-4">
            <div class="flex items-start justify-between mb-2">
              <h3 class="text-lg font-semibold text-txt">
                {props.step!.title}
              </h3>
              <span class="text-xs text-txt-muted">
                {props.stepIndex + 1} / {props.totalSteps}
              </span>
            </div>

            <div class="text-sm text-txt-tertiary mb-4">
              {props.step!.content}
            </div>

            {/* Navigation buttons */}
            <div class="flex items-center justify-between">
              <button
                class="text-sm text-txt-muted hover:text-txt-tertiary transition-colors"
                onClick={props.onSkip}
              >
                Skip tour
              </button>

              <div class="flex gap-2">
                <Show when={!props.isFirst}>
                  <button
                    class="px-3 py-1.5 text-sm bg-bg-hover hover:bg-bg-active text-txt rounded transition-colors"
                    onClick={props.onPrevious}
                  >
                    Back
                  </button>
                </Show>
                <button
                  class="px-3 py-1.5 text-sm bg-accent hover:bg-accent text-white rounded transition-colors"
                  onClick={props.onNext}
                >
                  {props.isLast ? "Finish" : "Next"}
                </button>
              </div>
            </div>
          </div>
        </div>
      </Portal>
    </Show>
  );
}

// ============================================================================
// Tooltip Component
// ============================================================================

export interface TooltipProps {
  /** Content of the tooltip */
  content: string | JSX.Element;
  /** Position relative to children */
  position?: "top" | "bottom" | "left" | "right";
  /** Delay before showing (ms) */
  delay?: number;
  /** Disabled state */
  disabled?: boolean;
  /** Children to wrap */
  children: JSX.Element;
}

export function Tooltip(props: TooltipProps) {
  const [isVisible, setIsVisible] = createSignal(false);
  const [position, setPosition] = createSignal({ top: 0, left: 0 });
  let triggerRef: HTMLDivElement | undefined;
  let timeoutId: number | undefined;

  const showTooltip = () => {
    if (props.disabled) return;
    
    timeoutId = window.setTimeout(() => {
      if (triggerRef) {
        const rect = triggerRef.getBoundingClientRect();
        const pos = calculatePosition(rect);
        setPosition(pos);
        setIsVisible(true);
      }
    }, props.delay ?? 300);
  };

  const hideTooltip = () => {
    clearTimeout(timeoutId);
    setIsVisible(false);
  };

  const calculatePosition = (rect: DOMRect): { top: number; left: number } => {
    const margin = 8;
    
    switch (props.position ?? "top") {
      case "top":
        return {
          top: rect.top - margin,
          left: rect.left + rect.width / 2,
        };
      case "bottom":
        return {
          top: rect.bottom + margin,
          left: rect.left + rect.width / 2,
        };
      case "left":
        return {
          top: rect.top + rect.height / 2,
          left: rect.left - margin,
        };
      case "right":
        return {
          top: rect.top + rect.height / 2,
          left: rect.right + margin,
        };
    }
  };

  const tooltipTransform = () => {
    switch (props.position ?? "top") {
      case "top":
        return "translate(-50%, -100%)";
      case "bottom":
        return "translate(-50%, 0)";
      case "left":
        return "translate(-100%, -50%)";
      case "right":
        return "translate(0, -50%)";
    }
  };

  onCleanup(() => clearTimeout(timeoutId));

  return (
    <div
      ref={triggerRef}
      class="inline-block"
      onMouseEnter={showTooltip}
      onMouseLeave={hideTooltip}
      onFocus={showTooltip}
      onBlur={hideTooltip}
    >
      {props.children}

      <Show when={isVisible()}>
        <Portal>
          <div
            class="tooltip animate-[fadeIn_0.15s_ease-out]"
            style={{
              top: `${position().top}px`,
              left: `${position().left}px`,
              transform: tooltipTransform(),
            }}
          >
            {props.content}
          </div>
        </Portal>
      </Show>
    </div>
  );
}

// ============================================================================
// Help Button Component
// ============================================================================

export interface HelpButtonProps {
  /** Help content */
  content: string | JSX.Element;
  /** Size */
  size?: "sm" | "md" | "lg";
  /** Position of tooltip */
  position?: "top" | "bottom" | "left" | "right";
}

export function HelpButton(props: HelpButtonProps) {
  const sizes = {
    sm: "w-4 h-4 text-xs",
    md: "w-5 h-5 text-sm",
    lg: "w-6 h-6 text-base",
  };

  return (
    <Tooltip content={props.content} position={props.position ?? "top"}>
      <button
        class={`
          inline-flex items-center justify-center rounded-full
          bg-bg-hover hover:bg-bg-active text-txt-secondary hover:text-txt
          transition-colors cursor-help
          ${sizes[props.size ?? "sm"]}
        `}
        aria-label="Help"
      >
        ?
      </button>
    </Tooltip>
  );
}

// ============================================================================
// Welcome Modal
// ============================================================================

export interface WelcomeModalProps {
  isOpen: boolean;
  onClose: () => void;
  onStartTour: () => void;
  title?: string;
  description?: string | JSX.Element;
}

export function WelcomeModal(props: WelcomeModalProps) {
  return (
    <Show when={props.isOpen}>
      <Portal>
        <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
          <div class="bg-bg border border-border rounded-xl shadow-2xl w-[480px] overflow-hidden">
            {/* Header with illustration */}
            <div class="bg-gradient-to-br from-accent to-accent p-8 text-center">
              <div class="text-6xl mb-4">🔍</div>
              <h2 class="text-2xl font-bold text-white">
                {props.title ?? "Welcome to FFX"}
              </h2>
            </div>

            {/* Content */}
            <div class="p-6">
              <div class="text-txt-tertiary text-sm mb-6">
                {props.description ?? (
                  <>
                    <p class="mb-3">
                      FFX is a powerful forensic file explorer for analyzing digital evidence.
                    </p>
                    <p>
                      Would you like a quick tour to learn the basics?
                    </p>
                  </>
                )}
              </div>

              {/* Features preview */}
              <div class="grid grid-cols-2 gap-3 mb-6">
                <div class="flex items-center gap-2 text-sm text-txt-secondary">
                  <HiOutlineFolder class="w-4 h-4" /> Evidence Tree
                </div>
                <div class="flex items-center gap-2 text-sm text-txt-secondary">
                  <HiOutlineLockClosed class="w-4 h-4" /> Hash Verification
                </div>
                <div class="flex items-center gap-2 text-sm text-txt-secondary">
                  <HiOutlineChartBar class="w-4 h-4" /> Hex Viewer
                </div>
                <div class="flex items-center gap-2 text-sm text-txt-secondary">
                  <HiOutlineDocumentText class="w-4 h-4" /> Report Generation
                </div>
              </div>

              {/* Actions */}
              <div class="flex gap-3">
                <button
                  class="flex-1 px-4 py-2.5 bg-bg-hover hover:bg-bg-active text-txt rounded-lg transition-colors"
                  onClick={props.onClose}
                >
                  Skip for now
                </button>
                <button
                  class="flex-1 px-4 py-2.5 bg-accent hover:bg-accent text-white rounded-lg transition-colors font-medium"
                  onClick={() => {
                    props.onClose();
                    props.onStartTour();
                  }}
                >
                  Start Tour
                </button>
              </div>
            </div>
          </div>
        </div>
      </Portal>
    </Show>
  );
}

// ============================================================================
// Default Tour Steps
// ============================================================================

export const DEFAULT_TOUR_STEPS: TourStep[] = [
  {
    id: "welcome",
    title: "Welcome to FFX!",
    content: "Let's take a quick tour of the main features.",
    position: "center",
  },
  {
    id: "file-panel",
    title: "Evidence Files",
    content: "This panel shows all loaded evidence files. Click 'Add Files' to load forensic images like E01, AD1, or other supported formats.",
    target: ".evidence-panel",
    position: "right",
  },
  {
    id: "tree-view",
    title: "File Tree",
    content: "Browse the contents of your evidence files here. Click folders to expand them and see their contents.",
    target: ".tree-panel",
    position: "right",
  },
  {
    id: "hex-viewer",
    title: "Hex Viewer",
    content: "View the raw bytes of selected files in hexadecimal format. Great for analyzing file headers and binary data.",
    target: ".hex-viewer",
    position: "left",
  },
  {
    id: "hash-verification",
    title: "Hash Verification",
    content: "Verify the integrity of evidence files by computing and comparing cryptographic hashes.",
    target: ".hash-panel",
    position: "left",
  },
  {
    id: "keyboard-shortcuts",
    title: "Keyboard Shortcuts",
    content: "Press ? anytime to see all available keyboard shortcuts. Use ⌘K to open the command palette for quick actions.",
    position: "center",
  },
];

// Animation keyframes
const styleSheet = document.createElement("style");
styleSheet.textContent = `
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }
`;
document.head.appendChild(styleSheet);

export default Tooltip;
