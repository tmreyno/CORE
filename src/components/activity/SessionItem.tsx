// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import {
  HiOutlineUser,
  HiOutlineCalendar,
  HiOutlineClock,
} from "../icons";
import type { ProjectSession } from "../../types/project";
import { formatDuration, formatTimestamp } from "./helpers";

interface SessionItemProps {
  session: ProjectSession;
  isActive?: boolean;
}

export const SessionItem: Component<SessionItemProps> = (props) => {
  return (
    <div class={`px-3 py-2 border-b border-border/50 ${props.isActive ? "bg-accent/10" : "hover:bg-bg-hover"}`}>
      <div class="flex items-center gap-2 text-sm">
        <HiOutlineUser class="w-3.5 h-3.5 text-txt-muted" />
        <span class="text-txt font-medium">{props.session.user}</span>
        <Show when={props.isActive}>
          <span class="text-2xs px-1.5 py-0.5 bg-success/20 text-success rounded">ACTIVE</span>
        </Show>
      </div>
      <div class="flex items-center gap-3 mt-1 text-xs text-txt-muted">
        <span class="flex items-center gap-1">
          <HiOutlineCalendar class="w-3 h-3" />
          {formatTimestamp(props.session.started_at, false)}
        </span>
        <span class="flex items-center gap-1">
          <HiOutlineClock class="w-3 h-3" />
          {props.session.ended_at 
            ? formatDuration(props.session.duration_seconds) 
            : "In progress"}
        </span>
      </div>
      <Show when={props.session.app_version}>
        <div class="text-2xs text-txt-muted mt-1">
          v{props.session.app_version}
        </div>
      </Show>
    </div>
  );
};
