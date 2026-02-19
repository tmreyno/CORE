// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * PstViewer - Viewer for Outlook PST and OST email archives
 *
 * Provides a three-panel forensic viewer:
 * - Left: Folder tree (Inbox, Sent Items, etc.)
 * - Center: Message list for the selected folder
 * - Right: Full message detail with body and attachments
 *
 * All operations are read-only — no modifications to the source PST file.
 */

import { createSignal, createEffect, Show, For, on, type Component } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineExclamationTriangle,
  HiOutlinePaperClip,
  HiOutlineEnvelope,
  HiOutlineEnvelopeOpen,
} from "./icons";
import { FolderIcon, ChevronDownIcon, ChevronRightIcon, TimeIcon } from "./icons";
import { formatBytes } from "../utils";
import { logger } from "../utils/logger";
import type { PstMetadataSection } from "../types/viewerMetadata";
import type {
  PstInfo,
  PstFolderInfo,
  PstMessageSummary,
  PstMessageDetail,
} from "../types/pst";

const log = logger.scope("PstViewer");

// ============================================================================
// Props
// ============================================================================

interface PstViewerProps {
  /** Path to the PST/OST file */
  path: string;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: PstMetadataSection) => void;
}

// ============================================================================
// Helpers
// ============================================================================

function formatEmailDate(dateStr: string | null): string {
  if (!dateStr) return "";
  try {
    const d = new Date(dateStr);
    return d.toLocaleString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return dateStr;
  }
}

function importanceLabel(importance: number | null): string | null {
  if (importance === 0) return "Low";
  if (importance === 2) return "High";
  return null;
}

// ============================================================================
// Sub-components
// ============================================================================

/** Recursive folder tree node */
const FolderNode: Component<{
  folder: PstFolderInfo;
  selectedId: number | null;
  depth: number;
  onSelect: (folder: PstFolderInfo) => void;
}> = (props) => {
  const [expanded, setExpanded] = createSignal(props.depth < 2);
  const isSelected = () => props.selectedId === props.folder.nodeId;
  const hasChildren = () => props.folder.children.length > 0;

  return (
    <div>
      <button
        class="w-full flex items-center gap-1 px-2 py-1 text-sm rounded hover:bg-bg-hover transition-colors"
        classList={{
          "bg-bg-active text-accent": isSelected(),
          "text-txt": !isSelected(),
        }}
        style={{ "padding-left": `${props.depth * 16 + 8}px` }}
        onClick={() => {
          props.onSelect(props.folder);
          if (hasChildren()) setExpanded(!expanded());
        }}
      >
        <Show
          when={hasChildren()}
          fallback={<span class="w-4 inline-block" />}
        >
          <span class="w-4 h-4 flex items-center justify-center text-txt-muted">
            <Show when={expanded()} fallback={<ChevronRightIcon class="w-3 h-3" />}>
              <ChevronDownIcon class="w-3 h-3" />
            </Show>
          </span>
        </Show>
        <FolderIcon class="w-icon-sm h-icon-sm text-txt-muted flex-shrink-0" />
        <span class="truncate flex-1 text-left">{props.folder.name}</span>
        <Show when={props.folder.contentCount > 0}>
          <span class="text-xs text-txt-muted ml-1">
            {props.folder.contentCount}
          </span>
        </Show>
        <Show when={props.folder.unreadCount > 0}>
          <span class="text-xs text-accent font-semibold ml-1">
            ({props.folder.unreadCount})
          </span>
        </Show>
      </button>
      <Show when={expanded() && hasChildren()}>
        <For each={props.folder.children}>
          {(child) => (
            <FolderNode
              folder={child}
              selectedId={props.selectedId}
              depth={props.depth + 1}
              onSelect={props.onSelect}
            />
          )}
        </For>
      </Show>
    </div>
  );
};

// ============================================================================
// Main Component
// ============================================================================

export const PstViewer: Component<PstViewerProps> = (props) => {
  // ── State ──
  const [pstInfo, setPstInfo] = createSignal<PstInfo | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);

  const [selectedFolder, setSelectedFolder] = createSignal<PstFolderInfo | null>(null);
  const [messages, setMessages] = createSignal<PstMessageSummary[]>([]);
  const [messagesLoading, setMessagesLoading] = createSignal(false);

  const [selectedMessage, setSelectedMessage] = createSignal<PstMessageDetail | null>(null);
  const [messageLoading, setMessageLoading] = createSignal(false);
  const [selectedMessageId, setSelectedMessageId] = createSignal<number | null>(null);

  // ── Load folder tree when path changes ──
  createEffect(on(
    () => props.path,
    async (path) => {
      setLoading(true);
      setError(null);
      setPstInfo(null);
      setSelectedFolder(null);
      setMessages([]);
      setSelectedMessage(null);
      setSelectedMessageId(null);

      try {
        log.info(`Loading PST folders: ${path}`);
        const info = await invoke<PstInfo>("pst_get_folders", { path });
        setPstInfo(info);
        log.info(`Loaded ${info.totalFolders} folders from "${info.displayName}"`);

        // Emit metadata
        if (props.onMetadata) {
          props.onMetadata({
            kind: "pst",
            displayName: info.displayName,
            totalFolders: info.totalFolders,
          });
        }
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        log.error(`Failed to load PST: ${msg}`);
        setError(msg);
      } finally {
        setLoading(false);
      }
    }
  ));

  // ── Load messages when folder changes ──
  createEffect(on(
    () => selectedFolder(),
    async (folder) => {
      if (!folder) {
        setMessages([]);
        setSelectedMessage(null);
        setSelectedMessageId(null);
        return;
      }

      setMessagesLoading(true);
      setSelectedMessage(null);
      setSelectedMessageId(null);

      try {
        log.info(`Loading messages for folder: ${folder.name} (nodeId=${folder.nodeId})`);
        const msgs = await invoke<PstMessageSummary[]>("pst_get_messages", {
          path: props.path,
          folderNodeId: folder.nodeId,
          limit: 500,
        });
        setMessages(msgs);
        log.info(`Loaded ${msgs.length} messages`);

        // Update metadata
        if (props.onMetadata) {
          props.onMetadata({
            kind: "pst",
            displayName: pstInfo()?.displayName,
            totalFolders: pstInfo()?.totalFolders,
            selectedFolder: folder.name,
            messageCount: msgs.length,
          });
        }
      } catch (err) {
        log.error(`Failed to load messages: ${err}`);
        setMessages([]);
      } finally {
        setMessagesLoading(false);
      }
    }
  ));

  // ── Load message detail when message is selected ──
  const handleMessageSelect = async (msg: PstMessageSummary) => {
    setSelectedMessageId(msg.nodeId);
    setMessageLoading(true);

    try {
      log.info(`Loading message detail: nodeId=${msg.nodeId}`);
      const detail = await invoke<PstMessageDetail>("pst_get_message_detail", {
        path: props.path,
        messageNodeId: msg.nodeId,
      });
      setSelectedMessage(detail);

      // Update metadata
      if (props.onMetadata) {
        props.onMetadata({
          kind: "pst",
          displayName: pstInfo()?.displayName,
          totalFolders: pstInfo()?.totalFolders,
          selectedFolder: selectedFolder()?.name,
          messageCount: messages().length,
          selectedMessage: detail.subject || "(no subject)",
        });
      }
    } catch (err) {
      log.error(`Failed to load message: ${err}`);
      setSelectedMessage(null);
    } finally {
      setMessageLoading(false);
    }
  };

  // ── Render ──
  return (
    <div class={`flex flex-col h-full overflow-hidden bg-bg ${props.class || ""}`}>
      {/* Error state */}
      <Show when={error()}>
        <div class="flex items-center gap-2 p-4 bg-red-500/10 text-error border-b border-border">
          <HiOutlineExclamationTriangle class="w-icon-base h-icon-base flex-shrink-0" />
          <span class="text-sm">{error()}</span>
        </div>
      </Show>

      {/* Loading state */}
      <Show when={loading()}>
        <div class="flex items-center justify-center h-full text-txt-muted">
          <div class="flex flex-col items-center gap-2">
            <div class="animate-pulse-slow text-accent">
              <HiOutlineEnvelope class="w-8 h-8" />
            </div>
            <span class="text-sm">Loading PST archive...</span>
          </div>
        </div>
      </Show>

      {/* Main three-panel layout */}
      <Show when={!loading() && !error() && pstInfo()}>
        <div class="flex flex-1 overflow-hidden">
          {/* Left: Folder tree */}
          <div class="w-56 flex-shrink-0 border-r border-border overflow-y-auto bg-bg-secondary">
            <div class="panel-header text-xs font-semibold text-txt-muted uppercase tracking-wider">
              {pstInfo()!.displayName}
            </div>
            <div class="py-1">
              <For each={pstInfo()!.folders}>
                {(folder) => (
                  <FolderNode
                    folder={folder}
                    selectedId={selectedFolder()?.nodeId ?? null}
                    depth={0}
                    onSelect={(f) => setSelectedFolder(f)}
                  />
                )}
              </For>
            </div>
          </div>

          {/* Center: Message list */}
          <div class="w-80 flex-shrink-0 border-r border-border overflow-y-auto bg-bg">
            <Show when={selectedFolder()} fallback={
              <div class="flex items-center justify-center h-full text-txt-muted text-sm">
                Select a folder
              </div>
            }>
              <Show when={messagesLoading()}>
                <div class="flex items-center justify-center py-8 text-txt-muted text-sm">
                  Loading messages...
                </div>
              </Show>
              <Show when={!messagesLoading() && messages().length === 0}>
                <div class="flex items-center justify-center py-8 text-txt-muted text-sm">
                  No messages in this folder
                </div>
              </Show>
              <Show when={!messagesLoading() && messages().length > 0}>
                <For each={messages()}>
                  {(msg) => {
                    const isActive = () => selectedMessageId() === msg.nodeId;
                    const imp = importanceLabel(msg.importance);
                    return (
                      <button
                        class="w-full text-left px-3 py-2 border-b border-border/50 hover:bg-bg-hover transition-colors"
                        classList={{
                          "bg-bg-active": isActive(),
                        }}
                        onClick={() => handleMessageSelect(msg)}
                      >
                        <div class="flex items-center gap-1.5 mb-0.5">
                          <Show when={!msg.isRead} fallback={
                            <HiOutlineEnvelopeOpen class="w-3.5 h-3.5 text-txt-muted flex-shrink-0" />
                          }>
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
                            <span class={`text-[10px] font-medium ${imp === "High" ? "text-error" : "text-txt-muted"}`}>
                              {imp}
                            </span>
                          </Show>
                        </div>
                        <div class="flex items-center gap-1 text-xs text-txt-muted">
                          <span class="truncate flex-1">
                            {msg.senderName || msg.senderEmail || "Unknown"}
                          </span>
                          <Show when={msg.date}>
                            <span class="flex-shrink-0">{formatEmailDate(msg.date)}</span>
                          </Show>
                        </div>
                      </button>
                    );
                  }}
                </For>
              </Show>
            </Show>
          </div>

          {/* Right: Message detail */}
          <div class="flex-1 overflow-y-auto bg-bg">
            <Show when={messageLoading()}>
              <div class="flex items-center justify-center py-8 text-txt-muted text-sm">
                Loading message...
              </div>
            </Show>
            <Show when={!messageLoading() && !selectedMessage() && selectedFolder()}>
              <div class="flex items-center justify-center h-full text-txt-muted text-sm">
                Select a message to view
              </div>
            </Show>
            <Show when={!messageLoading() && selectedMessage()}>
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
                        <span>{detail().attachments.length} attachment{detail().attachments.length > 1 ? "s" : ""}</span>
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
                    <Show when={detail().bodyHtml} fallback={
                      <Show when={detail().bodyText} fallback={
                        <div class="text-txt-muted text-sm italic">No message body</div>
                      }>
                        <pre class="whitespace-pre-wrap text-sm text-txt font-sans leading-relaxed">
                          {detail().bodyText}
                        </pre>
                      </Show>
                    }>
                      <div
                        class="text-sm text-txt [&_a]:text-accent [&_img]:max-w-full"
                        innerHTML={detail().bodyHtml!}
                      />
                    </Show>
                  </div>
                </div>
              )}
            </Show>
            <Show when={!selectedFolder()}>
              <div class="flex items-center justify-center h-full text-txt-muted text-sm">
                <div class="flex flex-col items-center gap-2">
                  <HiOutlineEnvelope class="w-8 h-8 text-txt-muted/50" />
                  <span>Select a folder to browse messages</span>
                </div>
              </div>
            </Show>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default PstViewer;
