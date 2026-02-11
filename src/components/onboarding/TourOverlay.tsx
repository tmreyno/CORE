// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, Show } from "solid-js";
import { Portal } from "solid-js/web";
import { Kbd, ModifierKeys } from "../ui/Kbd";
import type { TourStep } from "./types";

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
