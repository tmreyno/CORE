// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import { EmailIcon, HiOutlineExclamationTriangle } from "../icons";
import type { EmailViewerProps } from "./types";
import { useEmailData } from "./useEmailData";
import { EmailMessage } from "./EmailMessage";
import { MboxSidebar } from "./MboxSidebar";

export function EmailViewerComponent(props: EmailViewerProps) {
  const ev = useEmailData(props);

  return (
    <div class={`email-viewer flex flex-col h-full ${props.class || ""}`}>
      {/* Toolbar */}
      <div class="flex items-center gap-2 p-2 border-b border-border bg-bg-secondary">
        <EmailIcon class="w-4 h-4 text-accent" />
        <span class="text-sm font-medium truncate" title={ev.filename()}>{ev.filename()}</span>
        <Show when={!ev.isSingleEmail() && ev.emails().length > 0}>
          <span class="text-xs text-txt-muted">({ev.emails().length} messages)</span>
        </Show>
        <div class="flex-1" />
        <Show when={ev.selectedEmail()?.body_html && ev.selectedEmail()?.body_text}>
          <button
            class={`text-xs px-2 py-1 rounded border ${ev.showHtml() ? "bg-accent text-white border-accent" : "bg-bg-panel border-border text-txt-secondary hover:text-txt"}`}
            onClick={() => ev.setShowHtml(!ev.showHtml())}
          >
            {ev.showHtml() ? "HTML" : "Text"}
          </button>
        </Show>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-hidden flex">
        {/* MBOX message list sidebar */}
        <Show when={!ev.isSingleEmail() && ev.emails().length > 1}>
          <MboxSidebar
            emails={ev.emails()}
            selectedIndex={ev.selectedIndex}
            onSelect={ev.setSelectedIndex}
          />
        </Show>

        {/* Email content */}
        <div class="flex-1 overflow-y-auto">
          <Show
            when={!ev.loading()}
            fallback={
              <div class="flex flex-col items-center justify-center h-full gap-2">
                <div class="animate-spin w-8 h-8 border-2 border-accent border-t-transparent rounded-full" />
                <span class="text-txt-muted">Parsing email...</span>
              </div>
            }
          >
            <Show
              when={!ev.error()}
              fallback={
                <div class="flex flex-col items-center gap-2 text-error p-4">
                  <HiOutlineExclamationTriangle class="w-12 h-12" />
                  <span class="font-medium">Failed to parse email</span>
                  <span class="text-sm text-txt-muted">{ev.error()}</span>
                  <button onClick={ev.loadEmail} class="btn btn-secondary mt-2">Retry</button>
                </div>
              }
            >
              <Show when={ev.selectedEmail()}>
                {(email) => (
                  <EmailMessage
                    email={email()}
                    showHtml={ev.showHtml()}
                    showHeaders={ev.showHeaders()}
                    onToggleHeaders={() => ev.setShowHeaders(!ev.showHeaders())}
                  />
                )}
              </Show>
            </Show>
          </Show>
        </div>
      </div>
    </div>
  );
}
