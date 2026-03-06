// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, createMemo, type Accessor } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { getBasename } from "../../utils/pathUtils";
import { logger } from "../../utils/logger";
import type { EmailMetadataSection } from "../../types/viewerMetadata";
import type { EmailInfo, EmailViewerProps } from "./types";
import { isEml, isMbox, isMsg } from "./helpers";

const log = logger.scope("EmailViewer");

export interface UseEmailDataReturn {
  loading: Accessor<boolean>;
  error: Accessor<string | null>;
  emails: Accessor<EmailInfo[]>;
  selectedIndex: Accessor<number>;
  setSelectedIndex: (v: number) => void;
  showHeaders: Accessor<boolean>;
  setShowHeaders: (v: boolean) => void;
  showHtml: Accessor<boolean>;
  setShowHtml: (v: boolean) => void;
  isSingleEmail: Accessor<boolean>;
  selectedEmail: Accessor<EmailInfo | null>;
  filename: Accessor<string>;
  loadEmail: () => Promise<void>;
}

export function useEmailData(props: EmailViewerProps): UseEmailDataReturn {
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
        const info = await invoke<EmailInfo>("email_parse_msg", { path: props.path });
        setEmails([info]);
      } else if (isEml(props.path)) {
        const info = await invoke<EmailInfo>("email_parse_eml", { path: props.path });
        setEmails([info]);
      } else if (isMbox(props.path)) {
        const infos = await invoke<EmailInfo[]>("email_parse_mbox", {
          path: props.path,
          maxMessages: 200,
        });
        setEmails(infos);
      } else {
        try {
          const info = await invoke<EmailInfo>("email_parse_eml", { path: props.path });
          setEmails([info]);
        } catch {
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

  const filename = createMemo(() => getBasename(props.path) || props.path);

  // Emit metadata section
  createEffect(() => {
    const emailList = emails();
    if (emailList.length === 0 || !props.onMetadata) return;
    const current = selectedEmail();
    const section: EmailMetadataSection = {
      kind: "email",
      subject: current?.subject || undefined,
      from: current?.from?.[0]?.address || undefined,
      to: current?.to?.map((a) => a.address),
      cc: current?.cc?.length ? current.cc.map((a) => a.address) : undefined,
      date: current?.date || undefined,
      messageId: current?.message_id || undefined,
      contentType: current?.body_html ? "text/html" : "text/plain",
      attachmentCount: current?.attachments?.length ?? 0,
      messageCount: emailList.length > 1 ? emailList.length : undefined,
      selectedMessageIndex: emailList.length > 1 ? selectedIndex() : undefined,
    };
    props.onMetadata(section);
  });

  return {
    loading,
    error,
    emails,
    selectedIndex,
    setSelectedIndex,
    showHeaders,
    setShowHeaders,
    showHtml,
    setShowHtml,
    isSingleEmail,
    selectedEmail,
    filename,
    loadEmail,
  };
}
