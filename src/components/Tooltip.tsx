// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, Show, onCleanup, type JSX } from "solid-js";
import { Portal } from "solid-js/web";

export type TooltipPosition = "top" | "bottom" | "left" | "right";

interface TooltipProps {
  /** Tooltip content */
  content: string | JSX.Element;
  /** Position relative to trigger */
  position?: TooltipPosition;
  /** Delay before showing (ms) */
  delay?: number;
  /** Whether tooltip is disabled */
  disabled?: boolean;
  /** Additional CSS class for tooltip */
  class?: string;
  /** Trigger element */
  children: JSX.Element;
}

// Position-specific transform classes
const positionClasses: Record<TooltipPosition, string> = {
  top: "-translate-x-1/2 -translate-y-full",
  bottom: "-translate-x-1/2",
  left: "-translate-x-full -translate-y-1/2",
  right: "-translate-y-1/2",
};

// Arrow classes by position
const arrowClasses: Record<TooltipPosition, string> = {
  top: "absolute left-1/2 -translate-x-1/2 top-full border-4 border-transparent border-t-zinc-800",
  bottom: "absolute left-1/2 -translate-x-1/2 bottom-full border-4 border-transparent border-b-zinc-800",
  left: "absolute top-1/2 -translate-y-1/2 left-full border-4 border-transparent border-l-zinc-800",
  right: "absolute top-1/2 -translate-y-1/2 right-full border-4 border-transparent border-r-zinc-800",
};

/**
 * Styled Tooltip component - replaces native title attributes
 * 
 * Usage:
 * ```tsx
 * <Tooltip content="Click to hash file" position="top">
 *   <button>🔐</button>
 * </Tooltip>
 * ```
 */
export function Tooltip(props: TooltipProps) {
  const [visible, setVisible] = createSignal(false);
  const [coords, setCoords] = createSignal({ x: 0, y: 0 });
  
  let triggerRef: HTMLDivElement | undefined;
  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  
  const position = () => props.position ?? "top";
  const delay = () => props.delay ?? 400;

  const show = () => {
    if (props.disabled) return;
    
    timeoutId = setTimeout(() => {
      if (triggerRef) {
        const rect = triggerRef.getBoundingClientRect();
        const pos = position();
        
        let x = rect.left + rect.width / 2;
        let y = rect.top;
        
        switch (pos) {
          case "bottom":
            y = rect.bottom + 8;
            break;
          case "left":
            x = rect.left - 8;
            y = rect.top + rect.height / 2;
            break;
          case "right":
            x = rect.right + 8;
            y = rect.top + rect.height / 2;
            break;
          default: // top
            y = rect.top - 8;
        }
        
        setCoords({ x, y });
        setVisible(true);
      }
    }, delay());
  };

  const hide = () => {
    if (timeoutId) {
      clearTimeout(timeoutId);
      timeoutId = null;
    }
    setVisible(false);
  };

  onCleanup(() => {
    if (timeoutId) clearTimeout(timeoutId);
  });

  return (
    <>
      <div
        ref={triggerRef}
        class="inline-flex"
        onMouseEnter={show}
        onMouseLeave={hide}
        onFocus={show}
        onBlur={hide}
      >
        {props.children}
      </div>
      <Show when={visible()}>
        <Portal>
          <div
            class={`fixed z-[9999] px-2 py-1 text-xs font-medium text-zinc-100 bg-zinc-800 rounded shadow-lg whitespace-nowrap animate-[fadeIn_0.15s_ease-out] ${positionClasses[position()]} ${props.class ?? ""}`}
            style={{
              left: `${coords().x}px`,
              top: `${coords().y}px`,
            }}
            role="tooltip"
          >
            {props.content}
            <div class={arrowClasses[position()]} />
          </div>
        </Portal>
      </Show>
    </>
  );
}

/**
 * Simple inline tooltip trigger - for quick usage
 */
export function TooltipText(props: { 
  content: string; 
  children: JSX.Element;
  position?: TooltipPosition;
}) {
  return (
    <Tooltip content={props.content} position={props.position}>
      {props.children}
    </Tooltip>
  );
}
