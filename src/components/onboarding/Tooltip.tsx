// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, Show, onCleanup, type JSX } from "solid-js";
import { Portal } from "solid-js/web";

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
