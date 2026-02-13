// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { JSX, splitProps, ParentComponent, Component, Show, createEffect } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";

// =============================================================================
// SPINNER COMPONENT
// =============================================================================

interface SpinnerProps {
  /** Spinner size */
  size?: "sm" | "md" | "lg";
  /** Additional class names */
  class?: string;
}

/**
 * Standardized loading spinner
 */
export const Spinner: Component<SpinnerProps> = (props) => {
  const sizeClasses: Record<string, string> = {
    sm: "w-3 h-3 border",
    md: "w-5 h-5 border-2",
    lg: "w-8 h-8 border-2",
  };
  
  const size = props.size || "md";
  
  return (
    <div 
      class={`${sizeClasses[size]} border-current border-t-transparent rounded-full animate-spin ${props.class || ""}`}
      role="status"
      aria-label="Loading"
    />
  );
};

// =============================================================================
// BUTTON COMPONENT
// =============================================================================

type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";
type ButtonSize = "sm" | "md" | "lg" | "icon";

interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  /** Button style variant */
  variant?: ButtonVariant;
  /** Button size */
  size?: ButtonSize;
  /** Loading state - shows spinner */
  loading?: boolean;
  /** Additional class names */
  class?: string;
}

/** 
 * Standardized button component 
 */
export const Button: ParentComponent<ButtonProps> = (props) => {
  const [local, rest] = splitProps(props, ["variant", "size", "loading", "class", "disabled", "children"]);
  
  const variantClasses: Record<ButtonVariant, string> = {
    primary: "bg-accent text-white hover:bg-accent-hover",
    secondary: "bg-bg-panel border border-border text-txt hover:bg-bg-hover",
    ghost: "text-txt-secondary hover:text-txt hover:bg-bg-hover",
    danger: "bg-error text-white hover:bg-error/80",
  };
  
  const sizeClasses: Record<ButtonSize, string> = {
    sm: "px-2.5 py-1 text-xs",
    md: "px-3 py-1.5 text-sm",
    lg: "px-4 py-2 text-base",
    icon: "p-1.5",
  };
  
  const classes = () => {
    const variant = local.variant || "secondary";
    const size = local.size || "md";
    let cls = `inline-flex items-center justify-center gap-1.5 rounded font-medium transition-colors ${variantClasses[variant]} ${sizeClasses[size]}`;
    if (local.disabled || local.loading) cls += " opacity-50 cursor-not-allowed";
    if (local.class) cls += ` ${local.class}`;
    return cls;
  };
  
  return (
    <button class={classes()} disabled={local.disabled || local.loading} {...rest}>
      {local.loading && <Spinner size="sm" />}
      {local.children}
    </button>
  );
};

// =============================================================================
// ICON BUTTON COMPONENT
// =============================================================================

interface IconButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  /** Icon element */
  icon: JSX.Element;
  /** Tooltip/title text */
  label: string;
  /** Size variant */
  size?: "sm" | "md" | "lg";
  /** Active/pressed state */
  active?: boolean;
  /** Additional class names */
  class?: string;
}

/**
 * Icon-only button with tooltip
 */
export const IconButton: Component<IconButtonProps> = (props) => {
  const [local, rest] = splitProps(props, ["icon", "label", "size", "active", "class", "disabled"]);
  
  const sizeClasses: Record<string, string> = {
    sm: "p-1",
    md: "p-1.5",
    lg: "p-2",
  };
  
  const size = local.size || "md";
  
  const classes = () => {
    let cls = `${sizeClasses[size]} rounded text-txt-secondary hover:text-txt hover:bg-bg-hover transition-colors`;
    if (local.active) cls = cls.replace("text-txt-secondary", "text-accent bg-bg-hover");
    if (local.disabled) cls += " opacity-50 cursor-not-allowed";
    if (local.class) cls += ` ${local.class}`;
    return cls;
  };
  
  return (
    <button 
      class={classes()} 
      title={local.label} 
      aria-label={local.label}
      disabled={local.disabled}
      {...rest}
    >
      {local.icon}
    </button>
  );
};

// =============================================================================
// MODAL COMPONENT
// =============================================================================

interface ModalProps {
  /** Whether modal is open */
  isOpen: boolean;
  /** Close handler */
  onClose: () => void;
  /** Modal title */
  title?: string;
  /** Modal size */
  size?: "sm" | "md" | "lg" | "xl" | "full";
  /** Whether clicking backdrop closes modal */
  closeOnBackdrop?: boolean;
  /** Whether pressing Escape closes modal */
  closeOnEscape?: boolean;
  /** Additional class names for modal content */
  class?: string;
}

/**
 * Standardized modal/dialog component
 */
export const Modal: ParentComponent<ModalProps> = (props) => {
  const sizeClasses: Record<string, string> = {
    sm: "max-w-sm",
    md: "max-w-md",
    lg: "max-w-lg",
    xl: "max-w-2xl",
    full: "max-w-[90vw] max-h-[90vh]",
  };
  
  // Handle Escape key
  createEffect(() => {
    if (!props.isOpen || props.closeOnEscape === false) return;
    
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        props.onClose();
      }
    };
    
    makeEventListener(document, "keydown", handleKeyDown);
  });
  
  const handleBackdropClick = (e: MouseEvent) => {
    if (props.closeOnBackdrop !== false && e.target === e.currentTarget) {
      props.onClose();
    }
  };
  
  const size = props.size || "md";
  
  return (
    <Show when={props.isOpen}>
      <div 
        class="modal-overlay p-4"
        onClick={handleBackdropClick}
      >
        <div class={`modal-content w-full ${sizeClasses[size]} ${props.class || ""}`}>
          <Show when={props.title}>
            <div class="modal-header">
              <h2 class="text-base font-semibold text-txt">{props.title}</h2>
              <IconButton 
                icon={<span class="text-lg">×</span>} 
                label="Close" 
                size="sm"
                onClick={props.onClose}
              />
            </div>
          </Show>
          <div class="overflow-y-auto max-h-[calc(90vh-4rem)]">
            {props.children}
          </div>
        </div>
      </div>
    </Show>
  );
};
