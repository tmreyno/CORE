// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { JSX, ParentComponent, Component } from "solid-js";
import { textStyles, cardStyles, badgeStyles } from "./styles";

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
// EMPTY STATE COMPONENT
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
