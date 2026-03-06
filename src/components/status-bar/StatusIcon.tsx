// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import {
  HiOutlineCheckCircle,
  HiOutlineXCircle,
  HiOutlineArrowPath,
  HiOutlineMinusCircle,
} from "../icons";

/** Status icon with appropriate styling based on status kind */
export function StatusIcon(props: { kind: "idle" | "working" | "ok" | "error" }) {
  const baseClass = "w-3.5 h-3.5";
  switch (props.kind) {
    case "working":
      return (
        <span class="flex items-center justify-center w-5 h-5 rounded-full bg-warning/10">
          <HiOutlineArrowPath class={`${baseClass} animate-spin text-warning`} />
        </span>
      );
    case "ok":
      return (
        <span class="flex items-center justify-center w-5 h-5 rounded-full bg-success/10">
          <HiOutlineCheckCircle class={`${baseClass} text-success`} />
        </span>
      );
    case "error":
      return (
        <span class="flex items-center justify-center w-5 h-5 rounded-full bg-error/10">
          <HiOutlineXCircle class={`${baseClass} text-error`} />
        </span>
      );
    default:
      return (
        <span class="flex items-center justify-center w-5 h-5 rounded-full bg-bg-hover">
          <HiOutlineMinusCircle class={`${baseClass} text-txt-muted`} />
        </span>
      );
  }
}
