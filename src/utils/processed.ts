// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Shared utility functions for processed database components
 */

import type { ProcessedDbType } from '../types/processed';
import { formatDateByPreference } from "../utils";

/** Ellipse path to show only filename with hover for full path */
export function ellipsePath(path: string, maxLen: number = 40): string {
  if (!path) return '';
  if (path.length <= maxLen) return path;
  const parts = path.split('/');
  const filename = parts.pop() || path;
  if (filename.length >= maxLen) {
    return '...' + filename.slice(-maxLen + 3);
  }
  return '.../' + filename;
}

/** Get display name for database type */
export function getDbTypeName(dbType: ProcessedDbType): string {
  const names: Record<ProcessedDbType, string> = {
    MagnetAxiom: 'Magnet AXIOM',
    CellebritePA: 'Cellebrite PA',
    XWays: 'X-Ways Forensics',
    Autopsy: 'Autopsy',
    EnCase: 'EnCase',
    FTK: 'FTK',
    GenericSqlite: 'SQLite Database',
    Unknown: 'Unknown Format',
  };
  return names[dbType] || dbType;
}

/** Get icon for database type */
export function getDbTypeIcon(dbType: ProcessedDbType): string {
  const icons: Record<ProcessedDbType, string> = {
    MagnetAxiom: '🧲',
    CellebritePA: '📱',
    XWays: '🔬',
    Autopsy: '🔍',
    EnCase: '📦',
    FTK: '🛠️',
    GenericSqlite: '🗄️',
    Unknown: '❓',
  };
  return icons[dbType] || '📁';
}

/** Get icon for artifact category */
export function getCategoryIcon(category: string): string {
  const icons: Record<string, string> = {
    WebHistory: '🌐',
    Web: '🌐',
    Email: '📧',
    'Email & Calendar': '📧',
    Communication: '💬',
    Chat: '💬',
    Media: '🖼️',
    Documents: '📄',
    FileSystem: '📂',
    'File System': '📂',
    System: '⚙️',
    Registry: '🗄️',
    Network: '🔗',
    Timeline: '📅',
    Artifacts: '🔎',
    Mobile: '📱',
    Location: '📍',
    Cloud: '☁️',
    Encryption: '🔒',
    Malware: '🦠',
    Data: '📊',
    'User Accounts': '👤',
    Applications: '📦',
    Identifiers: '🏷️',
    Other: '📋',
  };
  return icons[category] || '📋';
}

/** Format date for display */
export function formatDate(dateStr?: string): string {
  return formatDateByPreference(dateStr, true);
}
