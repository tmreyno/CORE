// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For } from "solid-js";
import { HiOutlinePaperClip } from "../icons";
import { TimeIcon } from "../icons";
import { formatBytes } from "../../utils";
import type { EmailInfo } from "./types";
import { formatAddressList, formatEmailDate } from "./helpers";

interface EmailMessageProps {
  email: EmailInfo;
  showHtml: boolean;
  showHeaders: boolean;
  onToggleHeaders: () => void;
}

export function EmailMessage(props: EmailMessageProps) {
  const email = () => props.email;

  return (
    <div class="p-4 space-y-4">
      {/* Subject */}
      <h2 class="text-lg font-semibold text-txt">{email().subject || "(No Subject)"}</h2>

      {/* Header fields */}
      <div class="card space-y-2 text-sm">
        <Show when={email().from.length > 0}>
          <div class="flex gap-2">
            <span class="text-txt-muted w-12 shrink-0">From:</span>
            <span class="text-txt">{formatAddressList(email().from)}</span>
          </div>
        </Show>
        <Show when={email().to.length > 0}>
          <div class="flex gap-2">
            <span class="text-txt-muted w-12 shrink-0">To:</span>
            <span class="text-txt">{formatAddressList(email().to)}</span>
          </div>
        </Show>
        <Show when={email().cc.length > 0}>
          <div class="flex gap-2">
            <span class="text-txt-muted w-12 shrink-0">CC:</span>
            <span class="text-txt">{formatAddressList(email().cc)}</span>
          </div>
        </Show>
        <Show when={email().bcc.length > 0}>
          <div class="flex gap-2">
            <span class="text-txt-muted w-12 shrink-0">BCC:</span>
            <span class="text-txt">{formatAddressList(email().bcc)}</span>
          </div>
        </Show>
        <div class="flex gap-2">
          <span class="text-txt-muted w-12 shrink-0">Date:</span>
          <span class="text-txt flex items-center gap-1">
            <TimeIcon class="w-3.5 h-3.5 text-txt-muted" />
            {formatEmailDate(email().date)}
          </span>
        </div>
        <Show when={email().message_id}>
          <div class="flex gap-2">
            <span class="text-txt-muted w-12 shrink-0">ID:</span>
            <span class="text-[11px] text-txt-muted font-mono truncate">{email().message_id}</span>
          </div>
        </Show>
        <div class="flex gap-2">
          <span class="text-txt-muted w-12 shrink-0">Size:</span>
          <span class="text-txt-secondary text-xs">{formatBytes(email().size)}</span>
        </div>
      </div>

      {/* Attachments */}
      <Show when={email().attachments.length > 0}>
        <div class="card">
          <div class="flex items-center gap-2 mb-2">
            <HiOutlinePaperClip class="w-4 h-4 text-txt-muted" />
            <span class="text-sm font-medium text-txt">Attachments ({email().attachments.length})</span>
          </div>
          <div class="space-y-1">
            <For each={email().attachments}>
              {(att) => (
                <div class="flex items-center gap-2 p-1.5 rounded bg-bg-secondary text-xs">
                  <span class="text-txt truncate flex-1">{att.filename || "(unnamed)"}</span>
                  <span class="text-txt-muted">{att.content_type}</span>
                  <span class="text-txt-muted">{formatBytes(att.size)}</span>
                  <Show when={att.is_inline}>
                    <span class="badge badge-info text-[10px]">inline</span>
                  </Show>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>

      {/* Body */}
      <div class="card">
        <Show
          when={props.showHtml && email().body_html}
          fallback={
            <Show when={email().body_text} fallback={<p class="text-txt-muted italic">No message body</p>}>
              <pre class="text-sm text-txt whitespace-pre-wrap font-sans leading-relaxed">{email().body_text}</pre>
            </Show>
          }
        >
          <div class="email-html-body text-sm text-txt prose prose-invert max-w-none" innerHTML={email().body_html!} />
        </Show>
      </div>

      {/* Raw headers (collapsible) */}
      <Show when={email().headers.length > 0}>
        <div class="card">
          <button
            class="flex items-center gap-2 text-sm font-medium text-txt-secondary hover:text-txt w-full text-left"
            onClick={props.onToggleHeaders}
          >
            {/* Inline chevron to avoid importing separately */}
            <svg class={`w-4 h-4 transition-transform ${props.showHeaders ? "rotate-90" : ""}`} viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z" clip-rule="evenodd" />
            </svg>
            Raw Headers ({email().headers.length})
          </button>
          <Show when={props.showHeaders}>
            <div class="mt-2 space-y-1 max-h-64 overflow-y-auto">
              <For each={email().headers}>
                {(hdr) => (
                  <div class="flex gap-2 text-xs font-mono">
                    <span class="text-accent shrink-0">{hdr.name}:</span>
                    <span class="text-txt-muted break-all">{hdr.value}</span>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
}
