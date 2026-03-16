// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useDriveWatcher — Polls for drive/volume changes and exposes reactive signals.
 *
 * Since there is no OS-level event mechanism exposed through Tauri for drive
 * hot-plug detection, this hook polls `list_drives` at a configurable interval
 * and fires callbacks when drives are added or removed.
 */

import { createSignal, onCleanup, onMount } from "solid-js";
import { listDrives, type DriveInfo } from "../api/drives";

export interface DriveWatcherOptions {
  /** Poll interval in milliseconds (default: 5000) */
  intervalMs?: number;
  /** Called when new drives are detected */
  onDriveAdded?: (drives: DriveInfo[]) => void;
  /** Called when drives are removed */
  onDriveRemoved?: (mountPoints: string[]) => void;
}

export function useDriveWatcher(options: DriveWatcherOptions = {}) {
  const interval = options.intervalMs ?? 5000;
  const [drives, setDrives] = createSignal<DriveInfo[]>([]);
  const [lastPoll, setLastPoll] = createSignal<number>(0);

  let knownMounts = new Set<string>();
  let timer: ReturnType<typeof setInterval> | null = null;

  const poll = async () => {
    try {
      const current = await listDrives();
      setDrives(current);
      setLastPoll(Date.now());

      const currentMounts = new Set(current.map((d) => d.mountPoint));

      // Detect additions
      if (knownMounts.size > 0) {
        const added = current.filter((d) => !knownMounts.has(d.mountPoint));
        if (added.length > 0) options.onDriveAdded?.(added);

        const removedMounts = [...knownMounts].filter((m) => !currentMounts.has(m));
        if (removedMounts.length > 0) options.onDriveRemoved?.(removedMounts);
      }

      knownMounts = currentMounts;
    } catch {
      // Command may not be available — silently ignore
    }
  };

  onMount(() => {
    poll(); // Initial fetch
    timer = setInterval(poll, interval);
  });

  onCleanup(() => {
    if (timer) clearInterval(timer);
  });

  /** Force an immediate refresh */
  const refresh = () => poll();

  return { drives, lastPoll, refresh };
}
