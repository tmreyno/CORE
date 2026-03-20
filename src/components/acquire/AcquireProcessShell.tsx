// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, type JSX } from "solid-js";
import { HiOutlineArrowLeft } from "../icons";

export interface AcquireProcessShellProps {
  title: string;
  onBack: () => void;
  inline?: boolean;
  shellClass?: string;
  headerActions?: JSX.Element;
  children: JSX.Element;
}

const joinClasses = (...values: Array<string | undefined>) => values.filter(Boolean).join(" ");

const AcquireProcessShell: Component<AcquireProcessShellProps> = (props) => {
  return (
    <div class={joinClasses("flex flex-col flex-1 min-h-0 overflow-hidden bg-bg", props.shellClass)}>
      <Show when={!props.inline}>
        <div class="flex items-center gap-small px-3 py-1.5 shrink-0 bg-bg-secondary border-b border-border">
          <button class="btn btn-ghost gap-1 text-xs py-1 px-2" onClick={props.onBack}>
            <HiOutlineArrowLeft class="w-icon-sm h-icon-sm" />
            Dashboard
          </button>
          <span class="text-2xs font-medium text-txt-muted uppercase tracking-wider">{props.title}</span>
          <Show when={props.headerActions}>
            <div class="flex items-center gap-small ml-auto">{props.headerActions}</div>
          </Show>
        </div>
      </Show>
      {props.children}
    </div>
  );
};

export default AcquireProcessShell;