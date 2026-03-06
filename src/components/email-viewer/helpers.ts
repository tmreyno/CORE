// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { EmailAddress } from "./types";

export function formatEmailAddress(addr: EmailAddress): string {
  if (addr.name) {
    return `${addr.name} <${addr.address}>`;
  }
  return addr.address;
}

export function formatAddressList(addrs: EmailAddress[]): string {
  return addrs.map(formatEmailAddress).join(", ");
}

export function formatEmailDate(dateStr: string | null): string {
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

export function isEml(path: string): boolean {
  return path.toLowerCase().endsWith(".eml");
}

export function isMbox(path: string): boolean {
  return path.toLowerCase().endsWith(".mbox");
}

export function isMsg(path: string): boolean {
  return path.toLowerCase().endsWith(".msg");
}
