// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * MessageDetail — right panel showing the full message content, headers and attachments.
 */

import { Show, For, type Component, type Accessor } from "solid-js";
import { HiOutlineEnvelope, HiOutlinePaperClip } from "../icons";
import { TimeIcon } from "../icons";
import { formatBytes } from "../../utils";
import { formatEmailDate } from "./helpers";
import type { PstFolderInfo, PstMessageDetail as PstMessageDetailType } from "../../types/pst";

export interface MessageDetailProps {
  selectedMessage: Accessor<PstMessageDetailType | null>;
  messageLoading: Accessor<boolean>;
  selectedFolder: Accessor<PstFolderInfo | null>;
}

export const MessageDetail: Component<MessageDetailProps> = (props) => {
  return (
    <div class="flex-1 overflow-y-auto bg-bg">
      <Show when={props.messageLoading()}>
        <div class="flex items-center justify-center py-8 text-txt-muted text-sm">
          Loading message...
        </div>
      </Show>

      <Show when={!props.messageLoading() && !props.selectedMessage() && props.selectedFolder()}>
        <div class="flex items-center justify-center h-full text-txt-muted text-sm">
          Select a message to view
        </div>
      </Show>

      <Show when={!props.messageLoading() && props.selectedMessage()}>
        {(detail) => (
          <div class="flex flex-col h-full">
            {/* Message header */}
            <div class="border-b border-border p-4 bg-bg-secondary space-y-1">
              <h2 class="text-base font-semibold text-txt leading-tight">
                {detail().subject || "(no subject)"}
              </h2>
              <div class="text-sm text-txt-secondary space-y-0.5">
                <Show when={detail().senderName || detail().senderEmail}>
                  <div>
                    <span class="text-txt-muted">From: </span>
                    <span>
                      {detail().senderName}
                      <Show when={detail().senderEmail}>
                        {" "}&lt;{detail().senderEmail}&gt;
                      </Show>
                    </span>
                  </div>
                </Show>
                <Show when={detail().displayTo}>
                  <div>
                    <span class="text-txt-muted">To: </span>
                    <span>{detail().displayTo}</span>
                  </div>
                </Show>
                <Show when={detail().displayCc}>
                  <div>
                    <span class="text-txt-muted">CC: </span>
                    <span>{detail().displayCc}</span>
                  </div>
                </Show>
                <Show when={detail().displayBcc}>
                  <div>
                    <span class="text-txt-muted">BCC: </span>
                    <span>{detail().displayBcc}</span>
                  </div>
                </Show>
                <Show when={detail().date}>
                  <div class="flex items-center gap-1">
                    <TimeIcon class="w-3.5 h-3.5 text-txt-muted" />
                    <span>{formatEmailDate(detail().date)}</span>
                  </div>
                </Show>
              </div>
            </div>

            {/* Attachments */}
            <Show when={detail().attachments.length > 0}>
              <div class="border-b border-border px-4 py-2 bg-bg-secondary/50">
                <div class="flex items-center gap-1.5 text-xs text-txt-muted mb-1">
                  <HiOutlinePaperClip class="w-3.5 h-3.5" />
                  <span>
                    {detail().attachments.length} attachment
                    {detail().attachments.length > 1 ? "s" : ""}
                  </span>
                </div>
                <div class="flex flex-wrap gap-2">
                  <For each={detail().attachments}>
                    {(att) => (
                      <div class="chip chip-neutral text-xs flex items-center gap-1">
                        <span>{att.filename || "unnamed"}</span>
                        <Show when={att.size}>
                          <span class="text-txt-muted">({formatBytes(att.size!)})</span>
                        </Show>
                      </div>
                    )}
                  </For>
                </div>
              </div>
            </Show>

            {/* Message body */}
            <div class="flex-1 overflow-y-auto p-4">
              <Show
                when={detail().bodyHtml}
                fallback={
                  <Show
                    when={detail().bodyText}
                    fallback={
                      <div class="text-txt-muted text-sm italic">No message body</div>
                    }
                  >
                    <pre class="whitespace-pre-wrap text-sm text-txt font-sans leading-relaxed">
                      {detail().bodyText}
                    </pre>
                  </Show>
                }
              >
                <div
                  class="text-sm text-txt [&_a]:text-accent [&_img]:max-w-full"
                  innerHTML={detail().bodyHtml!}
                />
              </Show>
            </div>
          </div>
        )}
      </Show>

      <Show when={!props.selectedFolder()}>
        <div class="flex items-center justify-center h-full text-txt-muted text-sm">
          <div class="flex flex-col items-center gap-2">
            <HiOutlineEnvelope class="w-8 h-8 text-txt-muted/50" />
            <span>Select a folder to browse messages</span>
          </div>
        </div>
      </Show>
    </div>
  );
};
