// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, Show, type JSX, type ParentComponent } from "solid-js";

export type TransitionType = 
  | "fade"
  | "slide-up"
  | "slide-down"
  | "slide-left"
  | "slide-right"
  | "scale"
  | "scale-fade";

interface TransitionProps {
  /** Whether the content should be visible */
  show: boolean;
  /** Type of transition animation */
  type?: TransitionType;
  /** Duration in milliseconds */
  duration?: number;
  /** Delay before animation starts */
  delay?: number;
  /** Called when enter animation completes */
  onEntered?: () => void;
  /** Called when exit animation completes */
  onExited?: () => void;
  /** Additional CSS class */
  class?: string;
}

// Transition state styles as Tailwind-compatible inline styles
const transitionStyles: Record<TransitionType, {
  enter: Record<string, string>;
  entered: Record<string, string>;
  exit: Record<string, string>;
}> = {
  fade: {
    enter: { opacity: "0" },
    entered: { opacity: "1" },
    exit: { opacity: "0" },
  },
  "slide-up": {
    enter: { opacity: "0", transform: "translateY(10px)" },
    entered: { opacity: "1", transform: "translateY(0)" },
    exit: { opacity: "0", transform: "translateY(10px)" },
  },
  "slide-down": {
    enter: { opacity: "0", transform: "translateY(-10px)" },
    entered: { opacity: "1", transform: "translateY(0)" },
    exit: { opacity: "0", transform: "translateY(-10px)" },
  },
  "slide-left": {
    enter: { opacity: "0", transform: "translateX(10px)" },
    entered: { opacity: "1", transform: "translateX(0)" },
    exit: { opacity: "0", transform: "translateX(10px)" },
  },
  "slide-right": {
    enter: { opacity: "0", transform: "translateX(-10px)" },
    entered: { opacity: "1", transform: "translateX(0)" },
    exit: { opacity: "0", transform: "translateX(-10px)" },
  },
  scale: {
    enter: { opacity: "0", transform: "scale(0.95)" },
    entered: { opacity: "1", transform: "scale(1)" },
    exit: { opacity: "0", transform: "scale(0.95)" },
  },
  "scale-fade": {
    enter: { opacity: "0", transform: "scale(0.9)" },
    entered: { opacity: "1", transform: "scale(1)" },
    exit: { opacity: "0", transform: "scale(0.9)" },
  },
};

/**
 * Transition component for entrance/exit animations
 * 
 * Usage:
 * ```tsx
 * <Transition show={isOpen()} type="fade">
 *   <Modal>...</Modal>
 * </Transition>
 * 
 * <Transition show={visible()} type="slide-up" duration={300}>
 *   <Panel>...</Panel>
 * </Transition>
 * ```
 */
export const Transition: ParentComponent<TransitionProps> = (props) => {
  const [mounted, setMounted] = createSignal(props.show);
  const [animating, setAnimating] = createSignal(false);
  const [entering, setEntering] = createSignal(false);

  const type = () => props.type ?? "fade";
  const duration = () => props.duration ?? 200;
  const delay = () => props.delay ?? 0;

  createEffect(() => {
    const shouldShow = props.show;

    if (shouldShow) {
      // Mount first, then animate in
      setMounted(true);
      // Use requestAnimationFrame to ensure DOM is ready
      requestAnimationFrame(() => {
        setTimeout(() => {
          setAnimating(true);
          setEntering(true);
          setTimeout(() => {
            setAnimating(false);
            props.onEntered?.();
          }, duration());
        }, delay());
      });
    } else if (mounted()) {
      // Animate out, then unmount
      setAnimating(true);
      setEntering(false);
      setTimeout(() => {
        setMounted(false);
        setAnimating(false);
        props.onExited?.();
      }, duration());
    }
  });

  const getTransitionStyle = () => {
    const t = type();
    const isEntering = entering();
    const styles = transitionStyles[t];
    
    if (!animating()) {
      return isEntering ? styles.entered : {};
    }
    
    return isEntering ? styles.enter : styles.exit;
  };

  return (
    <Show when={mounted()}>
      <div
        class={props.class ?? ""}
        style={{
          ...getTransitionStyle(),
          transition: `all ${duration()}ms ease-out`,
        }}
      >
        {props.children}
      </div>
    </Show>
  );
};

/**
 * Fade transition - simple opacity change
 */
export const Fade: ParentComponent<Omit<TransitionProps, "type">> = (props) => (
  <Transition {...props} type="fade">
    {props.children}
  </Transition>
);

/**
 * Slide up transition - slides in from bottom
 */
export const SlideUp: ParentComponent<Omit<TransitionProps, "type">> = (props) => (
  <Transition {...props} type="slide-up">
    {props.children}
  </Transition>
);

/**
 * Scale fade transition - scales and fades simultaneously
 */
export const ScaleFade: ParentComponent<Omit<TransitionProps, "type">> = (props) => (
  <Transition {...props} type="scale-fade">
    {props.children}
  </Transition>
);

/**
 * Collapse transition - height animation for expandable content
 */
interface CollapseProps {
  show: boolean;
  duration?: number;
  class?: string;
  children: JSX.Element;
}

export function Collapse(props: CollapseProps) {
  const [height, setHeight] = createSignal<string>("0px");
  let contentRef: HTMLDivElement | undefined;

  createEffect(() => {
    if (props.show && contentRef) {
      // Measure content height
      setHeight(`${contentRef.scrollHeight}px`);
    } else {
      setHeight("0px");
    }
  });

  const duration = () => props.duration ?? 200;

  return (
    <div
      class={`overflow-hidden ${props.class ?? ""}`}
      style={{
        height: height(),
        transition: `height ${duration()}ms ease-out`,
      }}
    >
      <div ref={contentRef}>
        {props.children}
      </div>
    </div>
  );
}
