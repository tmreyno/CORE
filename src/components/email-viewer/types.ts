// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { EmailMetadataSection } from "../../types/viewerMetadata";

export interface EmailAddress {
  name: string | null;
  address: string;
}

export interface EmailAttachment {
  filename: string | null;
  content_type: string;
  size: number;
  is_inline: boolean;
}

export interface EmailHeader {
  name: string;
  value: string;
}

export interface EmailInfo {
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

export interface EmailViewerProps {
  /** Path to the email file (.eml or .mbox) */
  path: string;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: EmailMetadataSection) => void;
}
