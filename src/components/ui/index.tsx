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

import { JSX, splitProps, ParentComponent, Component, Show } from "solid-js";

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
  muted: "text-xs text-text/50",
  mutedSm: "text-sm text-text/60",
  secondary: "text-sm text-text/70",
  label: "text-xs text-text/50 font-medium",
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
      <p class="font-medium text-text/80">{props.title}</p>
      {props.description && (
        <p class="text-sm text-text/50 mt-1">{props.description}</p>
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
        props.checked ? "bg-cyan-600" : "bg-zinc-600"
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
        class="w-24 accent-cyan-500"
        min={props.min}
        max={props.max}
        step={props.step ?? 1}
        value={props.value}
        onInput={(e) => props.onChange(Number(e.currentTarget.value))}
      />
      <Show when={props.showValue !== false}>
        <span class="text-sm text-zinc-400 w-8 text-right">{props.value}</span>
      </Show>
    </div>
  );
};
