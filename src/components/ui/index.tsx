// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Shared UI Components - Reusable form elements and containers
 * 
 * These components provide consistent styling across the application
 * and reduce code duplication.
 */

import { JSX, splitProps, ParentComponent, Component, Show, createEffect } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";

// =============================================================================
// STYLE CONSTANTS
// =============================================================================

/** Common input styling classes */
export const inputStyles = {
  base: "w-full px-3 py-2.5 bg-surface border border-border/50 rounded-lg transition-all",
  focus: "focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20",
  small: "px-2.5 py-2 text-sm",
  error: "border-error/50 focus:border-error focus:ring-error/20",
  disabled: "opacity-50 cursor-not-allowed",
} as const;

/** Combined input class string */
export const inputClass = `${inputStyles.base} ${inputStyles.focus}`;
export const inputClassSm = `${inputStyles.base} ${inputStyles.focus} ${inputStyles.small}`;

/** Card/panel styling classes */
export const cardStyles = {
  base: "bg-surface/50 border border-border/30 rounded-xl",
  padded: "p-4 bg-surface/50 border border-border/30 rounded-xl",
  large: "p-6 bg-surface/50 border border-border/30 rounded-2xl",
  interactive: "hover:border-accent/30 hover:bg-surface/60 transition-colors cursor-pointer",
} as const;

/** Text styling classes */
export const textStyles = {
  muted: "text-xs text-txt/50",
  mutedSm: "text-sm text-txt/60",
  secondary: "text-sm text-txt/70",
  label: "text-xs text-txt/50 font-medium",
  error: "text-xs text-error",
  success: "text-xs text-success",
} as const;

/** Badge styling classes */
export const badgeStyles = {
  default: "px-2 py-0.5 text-xs rounded-full font-medium",
  accent: "px-2 py-0.5 text-xs rounded-full font-medium bg-accent/10 text-accent",
  success: "px-2 py-0.5 text-xs rounded-full font-medium bg-success/10 text-success",
  warning: "px-2 py-0.5 text-xs rounded-full font-medium bg-warning/10 text-warning",
  error: "px-2 py-0.5 text-xs rounded-full font-medium bg-error/10 text-error",
  info: "px-2 py-0.5 text-xs rounded-full font-medium bg-blue-500/10 text-blue-400",
} as const;

/** Button styling classes */
export const buttonStyles = {
  base: "px-4 py-2 rounded-lg font-medium transition-all",
  primary: "px-4 py-2 rounded-lg font-medium bg-accent text-white hover:bg-accent/90 transition-all",
  secondary: "px-4 py-2 rounded-lg font-medium bg-surface border border-border/50 hover:bg-surface/80 transition-all",
  ghost: "px-4 py-2 rounded-lg font-medium hover:bg-surface/50 transition-all",
  small: "px-3 py-1.5 text-sm rounded-lg font-medium transition-all",
  icon: "p-2 rounded-lg hover:bg-surface/50 transition-colors",
} as const;

// =============================================================================
// INPUT COMPONENT
// =============================================================================

interface InputProps extends JSX.InputHTMLAttributes<HTMLInputElement> {
  /** Small size variant */
  small?: boolean;
  /** Error state */
  error?: boolean;
  /** Additional class names */
  class?: string;
}

/** Styled input component */
export const Input: Component<InputProps> = (props) => {
  const [local, rest] = splitProps(props, ["small", "error", "class", "disabled"]);
  
  const classes = () => {
    let cls = local.small ? inputClassSm : inputClass;
    if (local.error) cls += ` ${inputStyles.error}`;
    if (local.disabled) cls += ` ${inputStyles.disabled}`;
    if (local.class) cls += ` ${local.class}`;
    return cls;
  };
  
  return <input class={classes()} disabled={local.disabled} {...rest} />;
};

// =============================================================================
// TEXTAREA COMPONENT
// =============================================================================

interface TextareaProps extends JSX.TextareaHTMLAttributes<HTMLTextAreaElement> {
  /** Small size variant */
  small?: boolean;
  /** Error state */
  error?: boolean;
  /** Additional class names */
  class?: string;
}

/** Styled textarea component */
export const Textarea: Component<TextareaProps> = (props) => {
  const [local, rest] = splitProps(props, ["small", "error", "class", "disabled"]);
  
  const classes = () => {
    let cls = `${local.small ? inputClassSm : inputClass} resize-none`;
    if (local.error) cls += ` ${inputStyles.error}`;
    if (local.disabled) cls += ` ${inputStyles.disabled}`;
    if (local.class) cls += ` ${local.class}`;
    return cls;
  };
  
  return <textarea class={classes()} disabled={local.disabled} {...rest} />;
};

// =============================================================================
// SELECT COMPONENT
// =============================================================================

interface SelectProps extends JSX.SelectHTMLAttributes<HTMLSelectElement> {
  /** Small size variant */
  small?: boolean;
  /** Error state */
  error?: boolean;
  /** Additional class names */
  class?: string;
}

/** Styled select component */
export const Select: Component<SelectProps> = (props) => {
  const [local, rest] = splitProps(props, ["small", "error", "class", "disabled", "children"]);
  
  const classes = () => {
    let cls = local.small ? inputClassSm : inputClass;
    if (local.error) cls += ` ${inputStyles.error}`;
    if (local.disabled) cls += ` ${inputStyles.disabled}`;
    if (local.class) cls += ` ${local.class}`;
    return cls;
  };
  
  return (
    <select class={classes()} disabled={local.disabled} {...rest}>
      {local.children}
    </select>
  );
};

// =============================================================================
// FORM FIELD COMPONENT
// =============================================================================

interface FormFieldProps {
  /** Field label */
  label: string;
  /** Optional hint text below the field */
  hint?: string;
  /** Error message */
  error?: string;
  /** Required field indicator */
  required?: boolean;
  /** Additional class names for container */
  class?: string;
}

/** Form field wrapper with label */
export const FormField: ParentComponent<FormFieldProps> = (props) => {
  return (
    <div class={`space-y-1.5 ${props.class || ""}`}>
      <label class={textStyles.label}>
        {props.label}
        {props.required && <span class="text-error ml-0.5">*</span>}
      </label>
      {props.children}
      {props.hint && !props.error && (
        <p class={textStyles.muted}>{props.hint}</p>
      )}
      {props.error && (
        <p class={textStyles.error}>{props.error}</p>
      )}
    </div>
  );
};

// =============================================================================
// CARD COMPONENT
// =============================================================================

interface CardProps {
  /** Padding size variant */
  size?: "default" | "large" | "none";
  /** Interactive (hoverable) */
  interactive?: boolean;
  /** Additional class names */
  class?: string;
  /** Click handler */
  onClick?: () => void;
}

/** Card container component */
export const Card: ParentComponent<CardProps> = (props) => {
  const classes = () => {
    let cls: string = cardStyles.base;
    if (props.size === "large") {
      cls = cardStyles.large;
    } else if (props.size !== "none") {
      cls = cardStyles.padded;
    }
    if (props.interactive) cls += ` ${cardStyles.interactive}`;
    if (props.class) cls += ` ${props.class}`;
    return cls;
  };
  
  return (
    <div class={classes()} onClick={props.onClick}>
      {props.children}
    </div>
  );
};

// =============================================================================
// BADGE COMPONENT
// =============================================================================

interface BadgeProps {
  /** Badge color variant */
  variant?: "default" | "accent" | "success" | "warning" | "error" | "info";
  /** Additional class names */
  class?: string;
}

/** Badge/tag component */
export const Badge: ParentComponent<BadgeProps> = (props) => {
  const classes = () => {
    const variant = props.variant || "default";
    let cls = badgeStyles[variant] || badgeStyles.default;
    if (props.class) cls += ` ${props.class}`;
    return cls;
  };
  
  return <span class={classes()}>{props.children}</span>;
};

// =============================================================================
// SECTION HEADER COMPONENT
// =============================================================================

interface SectionHeaderProps {
  /** Section title */
  title: string;
  /** Optional icon */
  icon?: JSX.Element;
  /** Optional subtitle */
  subtitle?: string;
  /** Right-side action content */
  action?: JSX.Element;
  /** Additional class names */
  class?: string;
}

/** Section header with title, optional icon, and action */
export const SectionHeader: Component<SectionHeaderProps> = (props) => {
  return (
    <div class={`flex items-center justify-between ${props.class || ""}`}>
      <div class="flex items-center gap-2">
        {props.icon}
        <div>
          <h3 class="text-base font-semibold">{props.title}</h3>
          {props.subtitle && (
            <p class={textStyles.muted}>{props.subtitle}</p>
          )}
        </div>
      </div>
      {props.action}
    </div>
  );
};

// =============================================================================
// EMPTY STATE COMPONENT (LIGHTWEIGHT)
// =============================================================================

interface EmptyStateProps {
  /** Icon or emoji */
  icon?: string | JSX.Element;
  /** Title text */
  title: string;
  /** Description text */
  description?: string;
  /** Action button/content */
  action?: JSX.Element;
  /** Size variant */
  size?: "small" | "default" | "large";
  /** Additional class names */
  class?: string;
}

/** Empty state placeholder */
export const EmptyStateSimple: Component<EmptyStateProps> = (props) => {
  const sizeClasses = () => {
    switch (props.size) {
      case "small": return "py-6";
      case "large": return "py-16";
      default: return "py-12";
    }
  };
  
  const iconSize = () => {
    switch (props.size) {
      case "small": return "w-10 h-10 text-xl";
      case "large": return "w-20 h-20 text-4xl";
      default: return "w-16 h-16 text-3xl";
    }
  };
  
  return (
    <div class={`text-center ${sizeClasses()} ${props.class || ""}`}>
      {props.icon && (
        <div class={`mx-auto mb-4 rounded-2xl bg-accent/10 flex items-center justify-center ${iconSize()}`}>
          {typeof props.icon === "string" ? <span>{props.icon}</span> : props.icon}
        </div>
      )}
      <p class="font-medium text-txt/80">{props.title}</p>
      {props.description && (
        <p class="text-sm text-txt/50 mt-1">{props.description}</p>
      )}
      {props.action && (
        <div class="mt-4">{props.action}</div>
      )}
    </div>
  );
};

// =============================================================================
// CHECKBOX COMPONENT
// =============================================================================

interface CheckboxProps {
  /** Checked state */
  checked: boolean;
  /** Change handler */
  onChange: (checked: boolean) => void;
  /** Label text */
  label?: string;
  /** Disabled state */
  disabled?: boolean;
  /** Additional class names */
  class?: string;
}

/** Styled checkbox component */
export const Checkbox: Component<CheckboxProps> = (props) => {
  return (
    <label class={`flex items-center gap-2 cursor-pointer ${props.disabled ? "opacity-50 cursor-not-allowed" : ""} ${props.class || ""}`}>
      <div 
        class={`w-5 h-5 rounded-md border-2 flex items-center justify-center transition-colors ${
          props.checked 
            ? "bg-accent border-accent" 
            : "border-border/50 hover:border-accent/50"
        }`}
        onClick={() => !props.disabled && props.onChange(!props.checked)}
      >
        {props.checked && (
          <span class="text-white text-xs font-bold">✓</span>
        )}
      </div>
      {props.label && (
        <span class="text-sm">{props.label}</span>
      )}
    </label>
  );
};

// =============================================================================
// DIVIDER COMPONENT
// =============================================================================

interface DividerProps {
  /** Vertical or horizontal */
  vertical?: boolean;
  /** Additional class names */
  class?: string;
}

/** Simple divider line */
export const Divider: Component<DividerProps> = (props) => {
  if (props.vertical) {
    return <div class={`w-px bg-border/30 self-stretch ${props.class || ""}`} />;
  }
  return <div class={`h-px bg-border/30 w-full ${props.class || ""}`} />;
};

// =============================================================================
// Toggle Component
// =============================================================================
interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  class?: string;
}

/** Toggle switch component */
export const Toggle: Component<ToggleProps> = (props) => {
  return (
    <button
      role="switch"
      aria-checked={props.checked}
      disabled={props.disabled}
      class={`relative w-10 h-6 rounded-full transition-colors ${
        props.checked ? "bg-accent" : "bg-bg-active"
      } ${props.disabled ? "opacity-50 cursor-not-allowed" : ""} ${props.class || ""}`}
      onClick={() => !props.disabled && props.onChange(!props.checked)}
    >
      <span
        class={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${
          props.checked ? "translate-x-4" : ""
        }`}
      />
    </button>
  );
};

// =============================================================================
// Slider Component
// =============================================================================
interface SliderProps {
  value: number;
  min: number;
  max: number;
  step?: number;
  onChange: (value: number) => void;
  showValue?: boolean;
  class?: string;
}

/** Range slider with optional value display */
export const Slider: Component<SliderProps> = (props) => {
  return (
    <div class={`flex items-center gap-3 ${props.class || ""}`}>
      <input
        type="range"
        class="w-24 accent-accent"
        min={props.min}
        max={props.max}
        step={props.step ?? 1}
        value={props.value}
        onInput={(e) => props.onChange(Number(e.currentTarget.value))}
      />
      <Show when={props.showValue !== false}>
        <span class="text-sm text-txt-secondary w-8 text-right">{props.value}</span>
      </Show>
    </div>
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
 * Use this instead of inline button styles for consistency.
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
 * Use this instead of inconsistent spinner implementations.
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
 * Use this for toolbar buttons, close buttons, etc.
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
 * Use this instead of custom modal implementations.
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
    
    // makeEventListener auto-cleans up when effect re-runs or component unmounts
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

// =============================================================================
// MODAL FOOTER COMPONENT
// =============================================================================

interface ModalFooterProps {
  /** Align buttons */
  align?: "left" | "center" | "right" | "between";
  /** Additional class names */
  class?: string;
}

/**
 * Footer for modal dialogs with action buttons
 */
export const ModalFooter: ParentComponent<ModalFooterProps> = (props) => {
  const alignClasses: Record<string, string> = {
    left: "justify-start",
    center: "justify-center",
    right: "justify-end",
    between: "justify-between",
  };
  
  const align = props.align || "right";
  
  return (
    <div class={`flex items-center gap-2 px-4 py-3 border-t border-border ${alignClasses[align]} ${props.class || ""}`}>
      {props.children}
    </div>
  );
};

