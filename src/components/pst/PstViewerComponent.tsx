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
import { HiOutlineExclamationTriangle, HiOutlineEnvelope } from "../icons";
import { commands } from "../../api/commands";
import { logger } from "../../utils/logger";
import { FolderNode } from "./FolderNode";
import { MessageList } from "./MessageList";
import { MessageDetail } from "./MessageDetail";
import { isTauri } from "../../utils/platform";
import type { PstViewerProps } from "./types";
import type {
  PstInfo,
  PstFolderInfo,
  PstMessageSummary,
  PstMessageDetail as PstMessageDetailType,
} from "../../types/pst";

const log = logger.scope("PstViewer");
const BROWSER_PST_VIEW_MESSAGE =
  "PST evidence viewing is available in the desktop app.";

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

  const [selectedMessage, setSelectedMessage] = createSignal<PstMessageDetailType | null>(null);
  const [messageLoading, setMessageLoading] = createSignal(false);
  const [selectedMessageId, setSelectedMessageId] = createSignal<number | null>(null);
  let archiveGeneration = 0;
  let messagesGeneration = 0;
  let detailGeneration = 0;

  // ── Load folder tree when path changes ──
  createEffect(
    on(
      () => `${props.path}|${JSON.stringify(props.source ?? null)}`,
      async () => {
        const generation = ++archiveGeneration;
        messagesGeneration++;
        detailGeneration++;
        const path = props.path;
        const source = props.source;
        setLoading(true);
        setError(null);
        setPstInfo(null);
        setSelectedFolder(null);
        setMessages([]);
        setSelectedMessage(null);
        setSelectedMessageId(null);

        try {
          if (!isTauri) {
            throw new Error(BROWSER_PST_VIEW_MESSAGE);
          }

          log.info(`Loading PST folders: ${path}`);
          const info = source
            ? await commands.pst.getFoldersSource<PstInfo>(source)
            : await commands.pst.getFolders<PstInfo>(path);
          if (generation !== archiveGeneration) return;
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
          if (generation !== archiveGeneration) return;
          const msg = err instanceof Error ? err.message : String(err);
          log.error(`Failed to load PST: ${msg}`);
          setError(msg);
        } finally {
          if (generation === archiveGeneration) {
            setLoading(false);
          }
        }
      },
    ),
  );

  // ── Load messages when folder changes ──
  createEffect(
    on(
      () => selectedFolder(),
      async (folder) => {
        if (!folder) {
          messagesGeneration++;
          detailGeneration++;
          setMessages([]);
          setSelectedMessage(null);
          setSelectedMessageId(null);
          return;
        }

        const archiveToken = archiveGeneration;
        const generation = ++messagesGeneration;
        detailGeneration++;
        setMessagesLoading(true);
        setSelectedMessage(null);
        setSelectedMessageId(null);

        try {
          if (!isTauri) {
            throw new Error(BROWSER_PST_VIEW_MESSAGE);
          }

          log.info(`Loading messages for folder: ${folder.name} (nodeId=${folder.nodeId})`);
          const source = props.source;
          const msgs = source
            ? await commands.pst.getMessagesSource<PstMessageSummary[]>(
                source,
                folder.nodeId,
                undefined,
                500,
              )
            : await commands.pst.getMessages<PstMessageSummary[]>(
                props.path,
                folder.nodeId,
                undefined,
                500,
              );
          if (archiveToken !== archiveGeneration || generation !== messagesGeneration) return;
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
          if (archiveToken !== archiveGeneration || generation !== messagesGeneration) return;
          log.error(`Failed to load messages: ${err}`);
          setMessages([]);
        } finally {
          if (archiveToken === archiveGeneration && generation === messagesGeneration) {
            setMessagesLoading(false);
          }
        }
      },
    ),
  );

  // ── Load message detail when message is selected ──
  const handleMessageSelect = async (msg: PstMessageSummary) => {
    const archiveToken = archiveGeneration;
    const messagesToken = messagesGeneration;
    const generation = ++detailGeneration;
    setSelectedMessageId(msg.nodeId);
    setMessageLoading(true);

    try {
      if (!isTauri) {
        throw new Error(BROWSER_PST_VIEW_MESSAGE);
      }

      log.info(`Loading message detail: nodeId=${msg.nodeId}`);
      const source = props.source;
      const detail = source
        ? await commands.pst.getMessageDetailSource<PstMessageDetailType>(
            source,
            msg.nodeId,
          )
        : await commands.pst.getMessageDetail<PstMessageDetailType>(
            props.path,
            msg.nodeId,
          );
      if (
        archiveToken !== archiveGeneration ||
        messagesToken !== messagesGeneration ||
        generation !== detailGeneration
      ) return;
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
      if (
        archiveToken !== archiveGeneration ||
        messagesToken !== messagesGeneration ||
        generation !== detailGeneration
      ) return;
      log.error(`Failed to load message: ${err}`);
      setSelectedMessage(null);
    } finally {
      if (
        archiveToken === archiveGeneration &&
        messagesToken === messagesGeneration &&
        generation === detailGeneration
      ) {
        setMessageLoading(false);
      }
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
          <MessageList
            selectedFolder={selectedFolder}
            messages={messages}
            messagesLoading={messagesLoading}
            selectedMessageId={selectedMessageId}
            onMessageSelect={handleMessageSelect}
          />

          {/* Right: Message detail */}
          <MessageDetail
            selectedMessage={selectedMessage}
            messageLoading={messageLoading}
            selectedFolder={selectedFolder}
          />
        </div>
      </Show>
    </div>
  );
};

export default PstViewer;
