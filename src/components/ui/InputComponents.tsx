// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { JSX, splitProps, Component } from "solid-js";
import { inputStyles, inputClass, inputClassSm } from "./styles";

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
