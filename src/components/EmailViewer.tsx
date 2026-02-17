// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EmailViewer - Viewer for EML and MBOX email files
 *
 * Parses email files via the Rust backend and renders:
 * - Header fields (From, To, CC, BCC, Date, Subject)
 * - Message body (HTML or plain text)
 * - Attachment listing
 * - Raw headers (collapsible)
 */

import { createSignal, createEffect, Show, For, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineExclamationTriangle,
  HiOutlinePaperClip,
} from "./icons";
import { EmailIcon, ChevronDownIcon, ChevronRightIcon, TimeIcon } from "./icons";
import { formatBytes } from "../utils";
import { logger } from "../utils/logger";
import type { EmailMetadataSection } from "../types/viewerMetadata";
const log = logger.scope("EmailViewer");

// ============================================================================
// Types (matching Rust structs)
// ============================================================================

interface EmailAddress {
  name: string | null;
  address: string;
}

interface EmailAttachment {
  filename: string | null;
  content_type: string;
  size: number;
  is_inline: boolean;
}

interface EmailHeader {
  name: string;
  value: string;
}

interface EmailInfo {
  path: string;
  message_id: string | null;
  subject: string | null;
  from: EmailAddress[];
  to: EmailAddress[];
  cc: EmailAddress[];
  bcc: EmailAddress[];
  date: string | null;
  body_text: string | null;
  body_html: string | null;
  attachments: EmailAttachment[];
  headers: EmailHeader[];
  size: number;
}

// ============================================================================
// Props
// ============================================================================

interface EmailViewerProps {
  /** Path to the email file (.eml or .mbox) */
  path: string;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: EmailMetadataSection) => void;
}

// ============================================================================
// Helpers
// ============================================================================

function formatEmailAddress(addr: EmailAddress): string {
  if (addr.name) {
    return `${addr.name} <${addr.address}>`;
  }
  return addr.address;
}

function formatAddressList(addrs: EmailAddress[]): string {
  return addrs.map(formatEmailAddress).join(", ");
}

function formatEmailDate(dateStr: string | null): string {
  if (!dateStr) return "Unknown";
  try {
    const d = new Date(dateStr);
    return d.toLocaleString(undefined, {
      weekday: "short",
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      timeZoneName: "short",
    });
  } catch {
    return dateStr;
  }
}

function isEml(path: string): boolean {
  return path.toLowerCase().endsWith(".eml");
}

function isMbox(path: string): boolean {
  return path.toLowerCase().endsWith(".mbox");
}

function isMsg(path: string): boolean {
  return path.toLowerCase().endsWith(".msg");
}

// ============================================================================
// Component
// ============================================================================

export function EmailViewer(props: EmailViewerProps) {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [emails, setEmails] = createSignal<EmailInfo[]>([]);
  const [selectedIndex, setSelectedIndex] = createSignal(0);
  const [showHeaders, setShowHeaders] = createSignal(false);
  const [showHtml, setShowHtml] = createSignal(true);

  const isSingleEmail = createMemo(() => isEml(props.path) || isMsg(props.path));
  const selectedEmail = createMemo(() => emails()[selectedIndex()] ?? null);

  const loadEmail = async () => {
    setLoading(true);
    setError(null);
    setEmails([]);
    setSelectedIndex(0);

    try {
      if (isMsg(props.path)) {
        // .msg (Outlook) format - parse with dedicated backend command
        const info = await invoke<EmailInfo>("email_parse_msg", { path: props.path });
        setEmails([info]);
      } else if (isEml(props.path)) {
        const info = await invoke<EmailInfo>("email_parse_eml", { path: props.path });
        setEmails([info]);
      } else if (isMbox(props.path)) {
        // MBOX — multiple messages
        const infos = await invoke<EmailInfo[]>("email_parse_mbox", {
          path: props.path,
          maxMessages: 200,
        });
        setEmails(infos);
      } else {
        // Unknown email format — try EML as default since it's the most common
        try {
          const info = await invoke<EmailInfo>("email_parse_eml", { path: props.path });
          setEmails([info]);
        } catch {
          // Fall back to MBOX parser
          const infos = await invoke<EmailInfo[]>("email_parse_mbox", {
            path: props.path,
            maxMessages: 200,
          });
          setEmails(infos);
        }
      }
    } catch (e) {
      log.error("Failed to parse email:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    if (props.path) {
      loadEmail();
    }
  });

  const filename = createMemo(() => props.path.split("/").pop() || props.path);

  // Emit metadata section when email data loads or selection changes
  createEffect(() => {
    const emailList = emails();
    if (emailList.length === 0 || !props.onMetadata) return;
    const current = selectedEmail();
    const section: EmailMetadataSection = {
      kind: "email",
      subject: current?.subject || undefined,
      from: current?.from?.[0]?.address || undefined,
      to: current?.to?.map(a => a.address),
      cc: current?.cc?.length ? current.cc.map(a => a.address) : undefined,
      date: current?.date || undefined,
      messageId: current?.message_id || undefined,
      contentType: current?.body_html ? "text/html" : "text/plain",
      attachmentCount: current?.attachments?.length ?? 0,
      messageCount: emailList.length > 1 ? emailList.length : undefined,
      selectedMessageIndex: emailList.length > 1 ? selectedIndex() : undefined,
    };
    props.onMetadata(section);
  });

  return (
    <div class={`email-viewer flex flex-col h-full ${props.class || ""}`}>
      {/* Toolbar */}
      <div class="flex items-center gap-2 p-2 border-b border-border bg-bg-secondary">
        <EmailIcon class="w-4 h-4 text-accent" />
        <span class="text-sm font-medium truncate" title={filename()}>{filename()}</span>
        <Show when={!isSingleEmail() && emails().length > 0}>
          <span class="text-xs text-txt-muted">
            ({emails().length} messages)
          </span>
        </Show>
        <div class="flex-1" />
        <Show when={selectedEmail()?.body_html && selectedEmail()?.body_text}>
          <button
            class={`text-xs px-2 py-1 rounded border ${showHtml() ? "bg-accent text-white border-accent" : "bg-bg-panel border-border text-txt-secondary hover:text-txt"}`}
            onClick={() => setShowHtml(!showHtml())}
          >
            {showHtml() ? "HTML" : "Text"}
          </button>
        </Show>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-hidden flex">
        {/* MBOX message list sidebar */}
        <Show when={!isSingleEmail() && emails().length > 1}>
          <div class="w-64 border-r border-border overflow-y-auto bg-bg-panel">
            <For each={emails()}>
              {(email, idx) => (
                <button
                  class={`w-full text-left p-2 border-b border-border/50 hover:bg-bg-hover ${selectedIndex() === idx() ? "bg-bg-active" : ""}`}
                  onClick={() => setSelectedIndex(idx())}
                >
                  <div class="text-xs font-medium text-txt truncate">
                    {email.subject || "(No Subject)"}
                  </div>
                  <div class="text-[11px] text-txt-muted truncate">
                    {email.from.length > 0 ? formatEmailAddress(email.from[0]) : "Unknown"}
                  </div>
                  <div class="text-[10px] text-txt-muted">
                    {formatEmailDate(email.date)}
                  </div>
                </button>
              )}
            </For>
          </div>
        </Show>

        {/* Email content */}
        <div class="flex-1 overflow-y-auto">
          <Show
            when={!loading()}
            fallback={
              <div class="flex flex-col items-center justify-center h-full gap-2">
                <div class="animate-spin w-8 h-8 border-2 border-accent border-t-transparent rounded-full" />
                <span class="text-txt-muted">Parsing email...</span>
              </div>
            }
          >
            <Show
              when={!error()}
              fallback={
                <div class="flex flex-col items-center gap-2 text-error p-4">
                  <HiOutlineExclamationTriangle class="w-12 h-12" />
                  <span class="font-medium">Failed to parse email</span>
                  <span class="text-sm text-txt-muted">{error()}</span>
                  <button onClick={loadEmail} class="btn btn-secondary mt-2">Retry</button>
                </div>
              }
            >
              <Show when={selectedEmail()}>
                {(email) => (
                  <div class="p-4 space-y-4">
                    {/* Subject */}
                    <h2 class="text-lg font-semibold text-txt">
                      {email().subject || "(No Subject)"}
                    </h2>

                    {/* Header fields */}
                    <div class="card space-y-2 text-sm">
                      {/* From */}
                      <Show when={email().from.length > 0}>
                        <div class="flex gap-2">
                          <span class="text-txt-muted w-12 shrink-0">From:</span>
                          <span class="text-txt">{formatAddressList(email().from)}</span>
                        </div>
                      </Show>
                      {/* To */}
                      <Show when={email().to.length > 0}>
                        <div class="flex gap-2">
                          <span class="text-txt-muted w-12 shrink-0">To:</span>
                          <span class="text-txt">{formatAddressList(email().to)}</span>
                        </div>
                      </Show>
                      {/* CC */}
                      <Show when={email().cc.length > 0}>
                        <div class="flex gap-2">
                          <span class="text-txt-muted w-12 shrink-0">CC:</span>
                          <span class="text-txt">{formatAddressList(email().cc)}</span>
                        </div>
                      </Show>
                      {/* BCC */}
                      <Show when={email().bcc.length > 0}>
                        <div class="flex gap-2">
                          <span class="text-txt-muted w-12 shrink-0">BCC:</span>
                          <span class="text-txt">{formatAddressList(email().bcc)}</span>
                        </div>
                      </Show>
                      {/* Date */}
                      <div class="flex gap-2">
                        <span class="text-txt-muted w-12 shrink-0">Date:</span>
                        <span class="text-txt flex items-center gap-1">
                          <TimeIcon class="w-3.5 h-3.5 text-txt-muted" />
                          {formatEmailDate(email().date)}
                        </span>
                      </div>
                      {/* Message ID */}
                      <Show when={email().message_id}>
                        <div class="flex gap-2">
                          <span class="text-txt-muted w-12 shrink-0">ID:</span>
                          <span class="text-[11px] text-txt-muted font-mono truncate">{email().message_id}</span>
                        </div>
                      </Show>
                      {/* Size */}
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
                          <span class="text-sm font-medium text-txt">
                            Attachments ({email().attachments.length})
                          </span>
                        </div>
                        <div class="space-y-1">
                          <For each={email().attachments}>
                            {(att) => (
                              <div class="flex items-center gap-2 p-1.5 rounded bg-bg-secondary text-xs">
                                <span class="text-txt truncate flex-1">
                                  {att.filename || "(unnamed)"}
                                </span>
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
                        when={showHtml() && email().body_html}
                        fallback={
                          <Show
                            when={email().body_text}
                            fallback={
                              <p class="text-txt-muted italic">No message body</p>
                            }
                          >
                            <pre class="text-sm text-txt whitespace-pre-wrap font-sans leading-relaxed">
                              {email().body_text}
                            </pre>
                          </Show>
                        }
                      >
                        <div
                          class="email-html-body text-sm text-txt prose prose-invert max-w-none"
                          innerHTML={email().body_html!}
                        />
                      </Show>
                    </div>

                    {/* Raw headers (collapsible) */}
                    <Show when={email().headers.length > 0}>
                      <div class="card">
                        <button
                          class="flex items-center gap-2 text-sm font-medium text-txt-secondary hover:text-txt w-full text-left"
                          onClick={() => setShowHeaders(!showHeaders())}
                        >
                          <Show when={showHeaders()} fallback={<ChevronRightIcon class="w-4 h-4" />}>
                            <ChevronDownIcon class="w-4 h-4" />
                          </Show>
                          Raw Headers ({email().headers.length})
                        </button>
                        <Show when={showHeaders()}>
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
                )}
              </Show>
            </Show>
          </Show>
        </div>
      </div>
    </div>
  );
}
