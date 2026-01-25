// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, Show, onMount, onCleanup, JSX, createEffect, For, type Accessor } from "solid-js";
import { Portal } from "solid-js/web";
import {
  HiOutlineFolder,
  HiOutlineLockClosed,
  HiOutlineChartBar,
  HiOutlineDocumentText,
  HiOutlinePlusCircle,
  HiOutlineFolderOpen,
  HiOutlineClock,
  HiOutlineDocumentDuplicate,
} from "./icons";
import { Kbd, ModifierKeys } from "./ui/Kbd";

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
  const [isTransitioning, setIsTransitioning] = createSignal(false);

  // Find and highlight target element with smooth transition
  createEffect(() => {
    if (!props.isActive || !props.step?.target) {
      setTargetRect(null);
      return;
    }

    // Add transition effect when changing steps
    setIsTransitioning(true);
    setTimeout(() => setIsTransitioning(false), 200);

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
    const margin = 20;
    const tooltipWidth = 360;
    const tooltipHeight = 240;

    switch (position) {
      case "top":
        return {
          top: Math.max(16, rect.top - tooltipHeight - margin),
          left: Math.max(16, Math.min(window.innerWidth - tooltipWidth - 16, rect.left + rect.width / 2 - tooltipWidth / 2)),
        };
      case "bottom":
        return {
          top: Math.min(window.innerHeight - tooltipHeight - 16, rect.bottom + margin),
          left: Math.max(16, Math.min(window.innerWidth - tooltipWidth - 16, rect.left + rect.width / 2 - tooltipWidth / 2)),
        };
      case "left":
        return {
          top: Math.max(16, Math.min(window.innerHeight - tooltipHeight - 16, rect.top + rect.height / 2 - tooltipHeight / 2)),
          left: Math.max(16, rect.left - tooltipWidth - margin),
        };
      case "right":
        return {
          top: Math.max(16, Math.min(window.innerHeight - tooltipHeight - 16, rect.top + rect.height / 2 - tooltipHeight / 2)),
          left: Math.min(window.innerWidth - tooltipWidth - 16, rect.right + margin),
        };
      default: // center
        return {
          top: window.innerHeight / 2 - tooltipHeight / 2,
          left: window.innerWidth / 2 - tooltipWidth / 2,
        };
    }
  };

  // Create progress dots array
  const progressDots = () => Array.from({ length: props.totalSteps }, (_, i) => i);

  return (
    <Show when={props.isActive && props.step}>
      <Portal>
        {/* Backdrop with gradient */}
        <div class="fixed inset-0 z-[9998] animate-fade-in">
          {/* Dark overlay with cutout for target */}
          <svg class="absolute inset-0 w-full h-full pointer-events-none">
            <defs>
              <mask id="tour-mask">
                <rect width="100%" height="100%" fill="white" />
                <Show when={targetRect()}>
                  <rect
                    x={targetRect()!.left - 12}
                    y={targetRect()!.top - 12}
                    width={targetRect()!.width + 24}
                    height={targetRect()!.height + 24}
                    rx="12"
                    fill="black"
                    class="transition-all duration-300"
                  />
                </Show>
              </mask>
              {/* Glow effect for highlight */}
              <filter id="tour-glow">
                <feGaussianBlur stdDeviation="8" result="coloredBlur"/>
                <feMerge>
                  <feMergeNode in="coloredBlur"/>
                  <feMergeNode in="SourceGraphic"/>
                </feMerge>
              </filter>
            </defs>
            <rect
              width="100%"
              height="100%"
              fill="rgba(0, 0, 0, 0.8)"
              mask="url(#tour-mask)"
            />
          </svg>

          {/* Highlight ring around target with glow */}
          <Show when={targetRect()}>
            <div
              class="absolute rounded-xl transition-all duration-300 ease-out pointer-events-none"
              classList={{ "opacity-0 scale-95": isTransitioning() }}
              style={{
                top: `${targetRect()!.top - 12}px`,
                left: `${targetRect()!.left - 12}px`,
                width: `${targetRect()!.width + 24}px`,
                height: `${targetRect()!.height + 24}px`,
                "box-shadow": "0 0 0 2px var(--color-accent), 0 0 20px 4px var(--color-accent), inset 0 0 20px 4px rgba(var(--color-accent-rgb), 0.1)",
              }}
            />
          </Show>
        </div>

        {/* Tooltip Card */}
        <div
          class="fixed z-[9999] w-[360px] bg-bg-panel border border-border rounded-2xl shadow-2xl overflow-hidden transition-all duration-300 ease-out"
          classList={{ "opacity-0 scale-95": isTransitioning() }}
          style={{
            top: `${tooltipPosition().top}px`,
            left: `${tooltipPosition().left}px`,
            "box-shadow": "0 25px 50px -12px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(var(--color-accent-rgb), 0.1)",
          }}
        >
          {/* Header with step indicator */}
          <div class="px-5 py-4 bg-gradient-to-r from-accent/10 to-transparent border-b border-border/50">
            <div class="flex items-center justify-between mb-1">
              <span class="text-xs font-medium text-accent uppercase tracking-wider">
                Step {props.stepIndex + 1} of {props.totalSteps}
              </span>
              <button
                class="p-1 rounded hover:bg-bg-hover text-txt-muted hover:text-txt transition-colors"
                onClick={props.onSkip}
                title="Skip tour"
              >
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            <h3 class="text-lg font-semibold text-txt">
              {props.step!.title}
            </h3>
          </div>

          {/* Content */}
          <div class="p-5">
            <div class="text-sm text-txt-secondary leading-relaxed mb-5">
              {props.step!.content}
            </div>

            {/* Progress Dots */}
            <div class="flex items-center justify-center gap-1.5 mb-5">
              {progressDots().map((_, index) => (
                <button
                  class={`w-2 h-2 rounded-full transition-all duration-200 ${
                    index === props.stepIndex 
                      ? "w-6 bg-accent" 
                      : index < props.stepIndex 
                        ? "bg-accent/50" 
                        : "bg-border hover:bg-txt-muted"
                  }`}
                  onClick={() => {
                    // Navigate to specific step if available
                    if (index < props.stepIndex) {
                      for (let i = props.stepIndex; i > index; i--) props.onPrevious();
                    } else if (index > props.stepIndex) {
                      for (let i = props.stepIndex; i < index; i++) props.onNext();
                    }
                  }}
                  aria-label={`Go to step ${index + 1}`}
                />
              ))}
            </div>

            {/* Navigation buttons */}
            <div class="flex items-center justify-between gap-3">
              <button
                class={`px-4 py-2 text-sm rounded-lg transition-all duration-200 ${
                  props.isFirst 
                    ? "opacity-0 pointer-events-none" 
                    : "bg-bg-hover hover:bg-bg-active text-txt-secondary hover:text-txt"
                }`}
                onClick={props.onPrevious}
                disabled={props.isFirst}
              >
                ← Back
              </button>
              
              <button
                class="flex-1 max-w-[140px] px-4 py-2 text-sm bg-accent hover:bg-accent-hover text-white font-medium rounded-lg transition-all duration-200 hover:shadow-lg hover:shadow-accent/20"
                onClick={props.onNext}
              >
                {props.isLast ? "✓ Finish" : "Next →"}
              </button>
            </div>
          </div>
          
          {/* Progress bar at bottom */}
          <div class="h-1 bg-bg-secondary">
            <div
              class="h-full bg-gradient-to-r from-accent to-accent-hover transition-all duration-500 ease-out"
              style={{ width: `${props.progress}%` }}
            />
          </div>
        </div>

        {/* Keyboard hints */}
        <div class="fixed bottom-4 left-1/2 -translate-x-1/2 z-[9999] flex items-center gap-4 px-4 py-2 bg-bg-panel/90 backdrop-blur border border-border rounded-full text-xs text-txt-muted">
          <span class="flex items-center gap-1.5">
            <Kbd keys={[ModifierKeys.left, ModifierKeys.right]} muted />
            <span>Navigate</span>
          </span>
          <span class="flex items-center gap-1.5">
            <Kbd keys={ModifierKeys.enter} muted />
            <span>Continue</span>
          </span>
          <span class="flex items-center gap-1.5">
            <Kbd keys={ModifierKeys.esc} muted />
            <span>Skip</span>
          </span>
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

/** Recent project info for the welcome modal */
export interface RecentProjectInfo {
  path: string;
  name: string;
  lastOpened: string;
}

export interface WelcomeModalProps {
  isOpen: boolean;
  onClose: () => void;
  onStartTour: () => void;
  title?: string;
  description?: string | JSX.Element;
  /** Callback to create a new project */
  onNewProject?: () => void;
  /** Callback to open an existing project */
  onOpenProject?: () => void;
  /** Recent projects to display */
  recentProjects?: Accessor<RecentProjectInfo[]>;
  /** Callback when a recent project is selected */
  onSelectRecentProject?: (path: string) => void;
}

export function WelcomeModal(props: WelcomeModalProps) {
  const hasQuickActions = () => !!props.onNewProject || !!props.onOpenProject;
  const hasRecentProjects = () => (props.recentProjects?.()?.length ?? 0) > 0;
  
  // Debug logging
  console.log(`[DEBUG] WelcomeModal render: isOpen=${props.isOpen}`);
  
  return (
    <Show when={props.isOpen}>
      {(() => { console.log("[DEBUG] WelcomeModal: Rendering modal content"); return null; })()}
      <Portal>
        <div class="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50 animate-fade-in">
          <div class="bg-bg-panel border border-border rounded-2xl shadow-2xl w-[600px] max-h-[90vh] overflow-hidden animate-slide-up flex flex-col">
            {/* Header with gradient and branding */}
            <div class="relative bg-gradient-to-br from-accent via-accent to-accent/80 p-8 text-center overflow-hidden flex-shrink-0">
              {/* Decorative background elements */}
              <div class="absolute inset-0 opacity-10">
                <div class="absolute top-0 left-0 w-32 h-32 bg-white rounded-full blur-3xl -translate-x-1/2 -translate-y-1/2" />
                <div class="absolute bottom-0 right-0 w-40 h-40 bg-white rounded-full blur-3xl translate-x-1/2 translate-y-1/2" />
              </div>
              
              <div class="relative">
                <div class="inline-flex items-center justify-center w-16 h-16 bg-white/20 rounded-2xl backdrop-blur mb-3">
                  <span class="text-4xl">🔍</span>
                </div>
                <h2 class="text-2xl font-bold text-white mb-1">
                  {props.title ?? "Welcome to CORE-FFX"}
                </h2>
                <p class="text-white/80 text-sm">
                  Forensic File Xplorer
                </p>
              </div>
            </div>

            {/* Content - scrollable */}
            <div class="p-5 overflow-y-auto flex-1">
              <div class="text-txt-secondary text-sm mb-5 leading-relaxed">
                {props.description ?? (
                  <p>
                    CORE-FFX is a powerful forensic file explorer for analyzing digital evidence containers 
                    like E01, AD1, L01, and more. Get started by creating a new project or opening an existing one.
                  </p>
                )}
              </div>

              {/* Quick Actions */}
              <Show when={hasQuickActions()}>
                <div class="mb-5">
                  <h3 class="text-xs font-semibold text-txt-muted uppercase tracking-wider mb-3">Quick Actions</h3>
                  <div class="grid grid-cols-2 gap-3">
                    <Show when={props.onNewProject}>
                      <button
                        class="flex items-center gap-3 p-4 bg-accent/10 hover:bg-accent/20 border border-accent/30 rounded-xl transition-all duration-200 group"
                        onClick={() => {
                          props.onClose();
                          props.onNewProject?.();
                        }}
                      >
                        <div class="p-2 bg-accent/20 rounded-lg text-accent group-hover:scale-110 transition-transform">
                          <HiOutlinePlusCircle class="w-6 h-6" />
                        </div>
                        <div class="text-left">
                          <div class="font-medium text-txt">New Project</div>
                          <div class="text-xs text-txt-muted">Start a new case</div>
                        </div>
                      </button>
                    </Show>
                    <Show when={props.onOpenProject}>
                      <button
                        class="flex items-center gap-3 p-4 bg-bg-secondary/50 hover:bg-bg-hover border border-border/50 rounded-xl transition-all duration-200 group"
                        onClick={() => {
                          props.onClose();
                          props.onOpenProject?.();
                        }}
                      >
                        <div class="p-2 bg-bg-hover rounded-lg text-txt-secondary group-hover:scale-110 transition-transform">
                          <HiOutlineFolderOpen class="w-6 h-6" />
                        </div>
                        <div class="text-left">
                          <div class="font-medium text-txt">Open Project</div>
                          <div class="text-xs text-txt-muted">Open existing case</div>
                        </div>
                      </button>
                    </Show>
                  </div>
                </div>
              </Show>

              {/* Recent Projects */}
              <Show when={hasRecentProjects()}>
                <div class="mb-5">
                  <h3 class="text-xs font-semibold text-txt-muted uppercase tracking-wider mb-3 flex items-center gap-2">
                    <HiOutlineClock class="w-4 h-4" />
                    Recent Projects
                  </h3>
                  <div class="space-y-2 max-h-[140px] overflow-y-auto">
                    <For each={props.recentProjects?.().slice(0, 5)}>
                      {(project) => (
                        <button
                          class="w-full flex items-center gap-3 p-3 bg-bg-secondary/30 hover:bg-bg-hover border border-border/30 rounded-lg transition-all duration-200 text-left group"
                          onClick={() => {
                            props.onClose();
                            props.onSelectRecentProject?.(project.path);
                          }}
                        >
                          <HiOutlineDocumentDuplicate class="w-5 h-5 text-txt-muted group-hover:text-accent transition-colors flex-shrink-0" />
                          <div class="min-w-0 flex-1">
                            <div class="font-medium text-txt text-sm truncate">{project.name}</div>
                            <div class="text-xs text-txt-muted truncate">{project.path}</div>
                          </div>
                          <div class="text-xs text-txt-muted flex-shrink-0">
                            {formatRelativeTime(project.lastOpened)}
                          </div>
                        </button>
                      )}
                    </For>
                  </div>
                </div>
              </Show>

              {/* Features preview */}
              <div class="mb-5">
                <h3 class="text-xs font-semibold text-txt-muted uppercase tracking-wider mb-3">Features</h3>
                <div class="grid grid-cols-2 gap-2">
                  <div class="flex items-center gap-3 p-3 bg-bg-secondary/50 rounded-lg border border-border/30">
                    <div class="p-2 bg-type-e01/10 rounded-lg text-type-e01">
                      <HiOutlineFolder class="w-5 h-5" />
                    </div>
                    <div class="text-sm">
                      <div class="font-medium text-txt">Evidence Browser</div>
                      <div class="text-xs text-txt-muted">E01, AD1, L01 support</div>
                    </div>
                  </div>
                  <div class="flex items-center gap-3 p-3 bg-bg-secondary/50 rounded-lg border border-border/30">
                    <div class="p-2 bg-success/10 rounded-lg text-success">
                      <HiOutlineLockClosed class="w-5 h-5" />
                    </div>
                    <div class="text-sm">
                      <div class="font-medium text-txt">Hash Verification</div>
                      <div class="text-xs text-txt-muted">MD5, SHA1, SHA256</div>
                    </div>
                  </div>
                  <div class="flex items-center gap-3 p-3 bg-bg-secondary/50 rounded-lg border border-border/30">
                    <div class="p-2 bg-accent/10 rounded-lg text-accent">
                      <HiOutlineChartBar class="w-5 h-5" />
                    </div>
                    <div class="text-sm">
                      <div class="font-medium text-txt">Hex Analysis</div>
                      <div class="text-xs text-txt-muted">Binary inspection</div>
                    </div>
                  </div>
                  <div class="flex items-center gap-3 p-3 bg-bg-secondary/50 rounded-lg border border-border/30">
                    <div class="p-2 bg-warning/10 rounded-lg text-warning">
                      <HiOutlineDocumentText class="w-5 h-5" />
                    </div>
                    <div class="text-sm">
                      <div class="font-medium text-txt">Report Generation</div>
                      <div class="text-xs text-txt-muted">PDF export</div>
                    </div>
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div class="flex gap-3">
                <button
                  class="flex-1 px-4 py-3 bg-bg-hover hover:bg-bg-active text-txt rounded-xl transition-all duration-200 text-sm font-medium"
                  onClick={props.onClose}
                >
                  Skip for now
                </button>
                <button
                  class="flex-1 px-4 py-3 bg-accent hover:bg-accent-hover text-white rounded-xl transition-all duration-200 font-medium text-sm hover:shadow-lg hover:shadow-accent/20"
                  onClick={() => {
                    props.onClose();
                    props.onStartTour();
                  }}
                >
                  🎯 Start Tour
                </button>
              </div>
              
              {/* Hint */}
              <p class="text-center text-xs text-txt-muted mt-4 flex items-center justify-center gap-1.5">
                Press <Kbd keys="?" muted /> anytime for help
              </p>
            </div>
          </div>
        </div>
      </Portal>
    </Show>
  );
}

/** Format a date string as relative time (e.g., "2 hours ago") */
function formatRelativeTime(dateString: string): string {
  try {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);
    
    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    
    return date.toLocaleDateString();
  } catch {
    return "";
  }
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
