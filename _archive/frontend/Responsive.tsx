// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, onMount, onCleanup, Accessor, JSX, Show } from "solid-js";

// ============================================================================
// Types
// ============================================================================

export type ResizeDirection = "horizontal" | "vertical" | "both";

export interface ResizeConstraints {
  minWidth?: number;
  maxWidth?: number;
  minHeight?: number;
  maxHeight?: number;
}

// ============================================================================
// Resize Handle Hook
// ============================================================================

export interface UseResizeOptions {
  direction: ResizeDirection;
  initialSize: number;
  constraints?: ResizeConstraints;
  onResize?: (size: number) => void;
  onResizeEnd?: (size: number) => void;
  storageKey?: string;
}

export function useResize(options: UseResizeOptions) {
  const [size, setSize] = createSignal(options.initialSize);
  const [isResizing, setIsResizing] = createSignal(false);

  // Load from storage
  onMount(() => {
    if (options.storageKey) {
      try {
        const stored = localStorage.getItem(options.storageKey);
        if (stored) {
          const parsed = parseFloat(stored);
          if (!isNaN(parsed)) {
            setSize(clampSize(parsed));
          }
        }
      } catch (e) {
        console.warn("Failed to load resize state:", e);
      }
    }
  });

  const clampSize = (value: number): number => {
    const { minWidth, maxWidth, minHeight, maxHeight } = options.constraints ?? {};
    const min = options.direction === "vertical" ? minHeight : minWidth;
    const max = options.direction === "vertical" ? maxHeight : maxWidth;
    
    let clamped = value;
    if (min !== undefined) clamped = Math.max(clamped, min);
    if (max !== undefined) clamped = Math.min(clamped, max);
    return clamped;
  };

  const startResize = (e: MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);

    const startPos = options.direction === "vertical" ? e.clientY : e.clientX;
    const startSize = size();

    const handleMove = (moveEvent: MouseEvent) => {
      const currentPos = options.direction === "vertical" ? moveEvent.clientY : moveEvent.clientX;
      const delta = currentPos - startPos;
      const newSize = clampSize(startSize + delta);
      setSize(newSize);
      options.onResize?.(newSize);
    };

    const handleUp = () => {
      setIsResizing(false);
      document.removeEventListener("mousemove", handleMove);
      document.removeEventListener("mouseup", handleUp);
      
      options.onResizeEnd?.(size());
      
      // Save to storage
      if (options.storageKey) {
        try {
          localStorage.setItem(options.storageKey, String(size()));
        } catch (e) {
          console.warn("Failed to save resize state:", e);
        }
      }
    };

    document.addEventListener("mousemove", handleMove);
    document.addEventListener("mouseup", handleUp);
  };

  return {
    size,
    setSize,
    isResizing,
    startResize,
  };
}

// ============================================================================
// Resizable Panel Component
// ============================================================================

export interface ResizablePanelProps {
  /** Direction to resize */
  direction: ResizeDirection;
  /** Initial size in pixels */
  initialSize?: number;
  /** Constraints */
  constraints?: ResizeConstraints;
  /** Storage key for persistence */
  storageKey?: string;
  /** Handle position */
  handlePosition?: "start" | "end";
  /** Additional class names */
  class?: string;
  /** Children */
  children: JSX.Element;
}

export function ResizablePanel(props: ResizablePanelProps) {
  const { size, isResizing, startResize } = useResize({
    direction: props.direction,
    initialSize: props.initialSize ?? 300,
    constraints: props.constraints,
    storageKey: props.storageKey,
  });

  const isVertical = () => props.direction === "vertical";
  const handleAtStart = () => props.handlePosition === "start";

  const handleStyle = () => ({
    cursor: isVertical() ? "row-resize" : "col-resize",
  });

  const panelStyle = () => {
    if (isVertical()) {
      return { height: `${size()}px` };
    }
    return { width: `${size()}px` };
  };

  const Handle = () => (
    <div
      class={`
        flex items-center justify-center shrink-0 transition-colors
        ${isVertical() ? "h-1.5 w-full hover:bg-accent/30" : "w-1.5 h-full hover:bg-accent/30"}
        ${isResizing() ? "bg-accent/50" : "bg-bg-hover/50"}
      `}
      style={handleStyle()}
      onMouseDown={startResize}
    >
      <div
        class={`
          rounded-full bg-bg-muted transition-all
          ${isResizing() ? "bg-accent" : ""}
          ${isVertical() ? "w-8 h-1" : "w-1 h-8"}
        `}
      />
    </div>
  );

  return (
    <div
      class={`flex ${isVertical() ? "flex-col" : ""} ${props.class ?? ""}`}
      style={panelStyle()}
    >
      <Show when={handleAtStart()}>
        <Handle />
      </Show>
      
      <div class="flex-1 overflow-hidden">
        {props.children}
      </div>
      
      <Show when={!handleAtStart()}>
        <Handle />
      </Show>
    </div>
  );
}

// ============================================================================
// Collapsible Sidebar
// ============================================================================

export interface CollapsibleSidebarProps {
  /** Whether the sidebar is collapsed */
  collapsed: Accessor<boolean>;
  /** Toggle collapse */
  onToggle: () => void;
  /** Width when expanded */
  expandedWidth?: number;
  /** Width when collapsed */
  collapsedWidth?: number;
  /** Side of the screen */
  side?: "left" | "right";
  /** Additional class names */
  class?: string;
  /** Content */
  children: JSX.Element;
}

export function CollapsibleSidebar(props: CollapsibleSidebarProps) {
  const expandedWidth = props.expandedWidth ?? 280;
  const collapsedWidth = props.collapsedWidth ?? 48;
  const isLeft = () => props.side !== "right";

  return (
    <div
      class={`relative flex flex-col transition-all duration-300 ease-in-out overflow-hidden ${props.class ?? ""}`}
      style={{ width: `${props.collapsed() ? collapsedWidth : expandedWidth}px` }}
    >
      {/* Toggle button */}
      <button
        class={`
          absolute top-2 z-10 p-1.5 rounded-full bg-bg-hover hover:bg-bg-active 
          text-txt-tertiary transition-all duration-300 shadow-lg
          ${isLeft() ? "right-2" : "left-2"}
        `}
        onClick={props.onToggle}
        aria-label={props.collapsed() ? "Expand sidebar" : "Collapse sidebar"}
        title={props.collapsed() ? "Expand" : "Collapse"}
      >
        <span class={`block transition-transform duration-300 ${
          props.collapsed() 
            ? (isLeft() ? "rotate-180" : "")
            : (isLeft() ? "" : "rotate-180")
        }`}>
          ◀
        </span>
      </button>

      {/* Content */}
      <div class="flex-1 overflow-hidden">
        {props.children}
      </div>
    </div>
  );
}

// ============================================================================
// Touch-Friendly Button
// ============================================================================

export interface TouchButtonProps {
  onClick: () => void;
  disabled?: boolean;
  size?: "sm" | "md" | "lg";
  variant?: "default" | "primary" | "danger";
  class?: string;
  children: JSX.Element;
  "aria-label"?: string;
}

export function TouchButton(props: TouchButtonProps) {
  // Minimum touch target sizes
  const sizes = {
    sm: "min-w-[44px] min-h-[44px] p-2",
    md: "min-w-[48px] min-h-[48px] p-3",
    lg: "min-w-[56px] min-h-[56px] p-4",
  };

  const variants = {
    default: "bg-bg-hover hover:bg-bg-active text-txt",
    primary: "bg-accent hover:bg-accent-hover text-white",
    danger: "bg-red-600 hover:bg-red-500 text-white",
  };

  return (
    <button
      class={`
        inline-flex items-center justify-center rounded-lg transition-colors
        active:scale-95 touch-manipulation select-none
        disabled:opacity-50 disabled:cursor-not-allowed
        ${sizes[props.size ?? "md"]}
        ${variants[props.variant ?? "default"]}
        ${props.class ?? ""}
      `}
      onClick={props.onClick}
      disabled={props.disabled}
      aria-label={props["aria-label"]}
    >
      {props.children}
    </button>
  );
}

// ============================================================================
// Responsive Container
// ============================================================================

export type Breakpoint = "sm" | "md" | "lg" | "xl" | "2xl";

export interface UseBreakpointOptions {
  /** Default breakpoint if window is not available */
  defaultBreakpoint?: Breakpoint;
}

const breakpoints: Record<Breakpoint, number> = {
  sm: 640,
  md: 768,
  lg: 1024,
  xl: 1280,
  "2xl": 1536,
};

/**
 * Hook to detect current breakpoint
 */
export function useBreakpoint(options: UseBreakpointOptions = {}) {
  const [currentBreakpoint, setCurrentBreakpoint] = createSignal<Breakpoint>(
    options.defaultBreakpoint ?? "lg"
  );
  const [windowWidth, setWindowWidth] = createSignal(
    typeof window !== "undefined" ? window.innerWidth : 1024
  );

  const updateBreakpoint = () => {
    const width = window.innerWidth;
    setWindowWidth(width);

    if (width < breakpoints.sm) {
      setCurrentBreakpoint("sm");
    } else if (width < breakpoints.md) {
      setCurrentBreakpoint("md");
    } else if (width < breakpoints.lg) {
      setCurrentBreakpoint("lg");
    } else if (width < breakpoints.xl) {
      setCurrentBreakpoint("xl");
    } else {
      setCurrentBreakpoint("2xl");
    }
  };

  onMount(() => {
    updateBreakpoint();
    window.addEventListener("resize", updateBreakpoint);
  });

  onCleanup(() => {
    window.removeEventListener("resize", updateBreakpoint);
  });

  const isMobile = () => windowWidth() < breakpoints.md;
  const isTablet = () => windowWidth() >= breakpoints.md && windowWidth() < breakpoints.lg;
  const isDesktop = () => windowWidth() >= breakpoints.lg;
  const isAtLeast = (bp: Breakpoint) => windowWidth() >= breakpoints[bp];
  const isAtMost = (bp: Breakpoint) => windowWidth() < breakpoints[bp];

  return {
    currentBreakpoint,
    windowWidth,
    isMobile,
    isTablet,
    isDesktop,
    isAtLeast,
    isAtMost,
  };
}

// ============================================================================
// Split View
// ============================================================================

export interface SplitViewProps {
  /** Direction of split */
  direction?: "horizontal" | "vertical";
  /** Initial split ratio (0-1) */
  initialRatio?: number;
  /** Minimum size for first panel */
  minFirst?: number;
  /** Minimum size for second panel */
  minSecond?: number;
  /** Storage key */
  storageKey?: string;
  /** Class for container */
  class?: string;
  /** First panel content */
  first: JSX.Element;
  /** Second panel content */
  second: JSX.Element;
}

export function SplitView(props: SplitViewProps) {
  const [ratio, setRatio] = createSignal(props.initialRatio ?? 0.5);
  const [isDragging, setIsDragging] = createSignal(false);
  let containerRef: HTMLDivElement | undefined;

  const isVertical = () => props.direction === "vertical";

  // Load from storage
  onMount(() => {
    if (props.storageKey) {
      try {
        const stored = localStorage.getItem(`split-${props.storageKey}`);
        if (stored) {
          const parsed = parseFloat(stored);
          if (!isNaN(parsed) && parsed >= 0 && parsed <= 1) {
            setRatio(parsed);
          }
        }
      } catch (e) {}
    }
  });

  const handleMouseDown = (e: MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);

    const handleMove = (moveEvent: MouseEvent) => {
      if (!containerRef) return;
      
      const rect = containerRef.getBoundingClientRect();
      const position = isVertical()
        ? (moveEvent.clientY - rect.top) / rect.height
        : (moveEvent.clientX - rect.left) / rect.width;
      
      // Clamp based on min sizes
      const containerSize = isVertical() ? rect.height : rect.width;
      const minRatio = (props.minFirst ?? 100) / containerSize;
      const maxRatio = 1 - ((props.minSecond ?? 100) / containerSize);
      
      setRatio(Math.max(minRatio, Math.min(maxRatio, position)));
    };

    const handleUp = () => {
      setIsDragging(false);
      document.removeEventListener("mousemove", handleMove);
      document.removeEventListener("mouseup", handleUp);
      
      if (props.storageKey) {
        try {
          localStorage.setItem(`split-${props.storageKey}`, String(ratio()));
        } catch (e) {}
      }
    };

    document.addEventListener("mousemove", handleMove);
    document.addEventListener("mouseup", handleUp);
  };

  return (
    <div
      ref={containerRef}
      class={`flex ${isVertical() ? "flex-col" : ""} h-full ${props.class ?? ""}`}
    >
      {/* First panel */}
      <div
        class="overflow-hidden"
        style={{
          [isVertical() ? "height" : "width"]: `calc(${ratio() * 100}% - 3px)`,
        }}
      >
        {props.first}
      </div>

      {/* Divider */}
      <div
        class={`
          flex items-center justify-center shrink-0 transition-colors
          ${isVertical() ? "h-1.5 w-full cursor-row-resize" : "w-1.5 h-full cursor-col-resize"}
          ${isDragging() ? "bg-accent/50" : "bg-bg-hover hover:bg-bg-active"}
        `}
        onMouseDown={handleMouseDown}
      >
        <div
          class={`
            rounded-full transition-colors
            ${isDragging() ? "bg-accent" : "bg-bg-muted"}
            ${isVertical() ? "w-8 h-1" : "w-1 h-8"}
          `}
        />
      </div>

      {/* Second panel */}
      <div
        class="overflow-hidden flex-1"
        style={{
          [isVertical() ? "height" : "width"]: `calc(${(1 - ratio()) * 100}% - 3px)`,
        }}
      >
        {props.second}
      </div>
    </div>
  );
}

export default ResizablePanel;
