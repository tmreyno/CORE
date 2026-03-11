// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * MessageList — center panel showing the message list for a selected PST folder.
 */

import { Show, For, type Component, type Accessor } from "solid-js";
import {
  HiOutlinePaperClip,
  HiOutlineEnvelope,
  HiOutlineEnvelopeOpen,
} from "../icons";
import { formatEmailDate, importanceLabel } from "./helpers";
import type { PstFolderInfo, PstMessageSummary } from "../../types/pst";

export interface MessageListProps {
  selectedFolder: Accessor<PstFolderInfo | null>;
  messages: Accessor<PstMessageSummary[]>;
  messagesLoading: Accessor<boolean>;
  selectedMessageId: Accessor<number | null>;
  onMessageSelect: (msg: PstMessageSummary) => void;
}

export const MessageList: Component<MessageListProps> = (props) => {
  return (
    <div class="w-80 flex-shrink-0 border-r border-border overflow-y-auto bg-bg">
      <Show
        when={props.selectedFolder()}
        fallback={
          <div class="flex items-center justify-center h-full text-txt-muted text-sm">
            Select a folder
          </div>
        }
      >
        <Show when={props.messagesLoading()}>
          <div class="flex items-center justify-center py-8 text-txt-muted text-sm">
            Loading messages...
          </div>
        </Show>
        <Show when={!props.messagesLoading() && props.messages().length === 0}>
          <div class="flex items-center justify-center py-8 text-txt-muted text-sm">
            No messages in this folder
          </div>
        </Show>
        <Show when={!props.messagesLoading() && props.messages().length > 0}>
          <For each={props.messages()}>
            {(msg) => {
              const isActive = () => props.selectedMessageId() === msg.nodeId;
              const imp = importanceLabel(msg.importance);
              return (
                <button
                  class="w-full text-left px-3 py-2 border-b border-border/50 hover:bg-bg-hover transition-colors"
                  classList={{
                    "bg-bg-active": isActive(),
                  }}
                  onClick={() => props.onMessageSelect(msg)}
                >
                  <div class="flex items-center gap-1.5 mb-0.5">
                    <Show
                      when={!msg.isRead}
                      fallback={
                        <HiOutlineEnvelopeOpen class="w-3.5 h-3.5 text-txt-muted flex-shrink-0" />
                      }
                    >
                      <HiOutlineEnvelope class="w-3.5 h-3.5 text-accent flex-shrink-0" />
                    </Show>
                    <span
                      class="text-sm truncate flex-1"
                      classList={{
                        "font-semibold text-txt": !msg.isRead,
                        "text-txt-secondary": msg.isRead,
                      }}
                    >
                      {msg.subject || "(no subject)"}
                    </span>
                    <Show when={msg.hasAttachments}>
                      <HiOutlinePaperClip class="w-3 h-3 text-txt-muted flex-shrink-0" />
                    </Show>
                    <Show when={imp}>
                      <span
                        class={`text-2xs font-medium ${imp === "High" ? "text-error" : "text-txt-muted"}`}
                      >
                        {imp}
                      </span>
                    </Show>
                  </div>
                  <div class="flex items-center gap-1 text-xs text-txt-muted">
                    <span class="truncate flex-1">
                      {msg.senderName || msg.senderEmail || "Unknown"}
                    </span>
                    <Show when={msg.date}>
                      <span class="flex-shrink-0">
                        {formatEmailDate(msg.date)}
                      </span>
                    </Show>
                  </div>
                </button>
              );
            }}
          </For>
        </Show>
      </Show>
    </div>
  );
};
