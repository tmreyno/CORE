// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * TypeScript types for PST/OST email archive viewer.
 * Must stay in sync with src-tauri/src/viewer/document/pst.rs
 */

/** Top-level PST store information with folder tree */
export interface PstInfo {
  /** File path */
  path: string;
  /** Store display name */
  displayName: string;
  /** Folder tree (recursive) */
  folders: PstFolderInfo[];
  /** Total folder count */
  totalFolders: number;
}

/** A single folder in the PST hierarchy */
export interface PstFolderInfo {
  /** Folder display name */
  name: string;
  /** Node ID (used as identifier for subsequent queries) */
  nodeId: number;
  /** Number of messages in this folder */
  contentCount: number;
  /** Number of unread messages */
  unreadCount: number;
  /** Whether this folder has subfolders */
  hasSubFolders: boolean;
  /** Child folders (recursive) */
  children: PstFolderInfo[];
}

/** Lightweight message summary for listing in a folder */
export interface PstMessageSummary {
  /** Node ID (used as identifier for full message fetch) */
  nodeId: number;
  /** Subject line */
  subject: string | null;
  /** Sender display name */
  senderName: string | null;
  /** Sender email */
  senderEmail: string | null;
  /** Display To field */
  displayTo: string | null;
  /** Date as ISO 8601 string */
  date: string | null;
  /** Message size in bytes */
  size: number | null;
  /** Whether message has attachments */
  hasAttachments: boolean;
  /** Importance level (0=low, 1=normal, 2=high) */
  importance: number | null;
  /** Whether message has been read */
  isRead: boolean;
  /** Message class (IPM.Note, etc.) */
  messageClass: string | null;
}

/** Full message detail with body content */
export interface PstMessageDetail {
  /** Node ID */
  nodeId: number;
  /** Subject line */
  subject: string | null;
  /** Sender display name */
  senderName: string | null;
  /** Sender email */
  senderEmail: string | null;
  /** Display To */
  displayTo: string | null;
  /** Display CC */
  displayCc: string | null;
  /** Display BCC */
  displayBcc: string | null;
  /** Date as ISO 8601 string */
  date: string | null;
  /** Plain text body */
  bodyText: string | null;
  /** HTML body */
  bodyHtml: string | null;
  /** Message size in bytes */
  size: number | null;
  /** Whether message has attachments */
  hasAttachments: boolean;
  /** Importance level */
  importance: number | null;
  /** Whether message has been read */
  isRead: boolean;
  /** Message class */
  messageClass: string | null;
  /** Conversation topic */
  conversationTopic: string | null;
  /** Attachment metadata */
  attachments: PstAttachmentInfo[];
}

/** Attachment metadata (no binary content — forensic read-only) */
export interface PstAttachmentInfo {
  /** Display name or filename */
  filename: string | null;
  /** Size in bytes */
  size: number | null;
  /** MIME content type */
  contentType: string | null;
}
