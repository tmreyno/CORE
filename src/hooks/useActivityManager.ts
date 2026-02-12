// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useActivityManager - Manages activity lifecycle (cancel, pause, resume, clear)
 *
 * Tracks running backend operations and their cleanup functions so they can
 * be cancelled from the UI.  Previously inlined in App.tsx.
 */

import { createSignal, type Accessor, type Setter } from "solid-js";
import type { Activity } from "../types/activity";
import { logger } from "../utils/logger";

const log = logger.scope("ActivityManager");

export interface ActivityManager {
  /** Reactive list of activities */
  activities: Accessor<Activity[]>;
  setActivities: Setter<Activity[]>;

  /** Register a cleanup function for a running operation */
  registerCleanup: (activityId: string, cleanup: () => void) => void;

  /** Unregister a cleanup (e.g. when the operation completes normally) */
  unregisterCleanup: (activityId: string) => void;

  /** Cancel a running/pending/paused activity and invoke its cleanup */
  cancel: (id: string) => void;

  /** Pause a running activity */
  pause: (id: string) => void;

  /** Resume a paused activity */
  resume: (id: string) => void;

  /** Remove an activity from the list */
  clear: (id: string) => void;
}

export function useActivityManager(): ActivityManager {
  const [activities, setActivities] = createSignal<Activity[]>([]);

  // Maps activity IDs → cleanup functions (event unlisteners, abort controllers, etc.)
  const cleanups = new Map<string, () => void>();

  const registerCleanup = (activityId: string, cleanup: () => void) => {
    cleanups.set(activityId, cleanup);
  };

  const unregisterCleanup = (activityId: string) => {
    cleanups.delete(activityId);
  };

  const cancel = (id: string) => {
    log.debug(`Cancel activity: ${id}`);

    const cleanup = cleanups.get(id);
    if (cleanup) {
      try {
        cleanup();
      } catch (e) {
        log.warn("Error during operation cleanup:", e);
      }
      cleanups.delete(id);
    }

    setActivities((list) =>
      list.map((a) =>
        a.id === id &&
        (a.status === "running" || a.status === "pending" || a.status === "paused")
          ? { ...a, status: "cancelled" as const, endTime: new Date() }
          : a,
      ),
    );
  };

  const pause = (id: string) => {
    log.debug(`Pause activity: ${id}`);
    setActivities((list) =>
      list.map((a) =>
        a.id === id && a.status === "running"
          ? { ...a, status: "paused" as const }
          : a,
      ),
    );
  };

  const resume = (id: string) => {
    log.debug(`Resume activity: ${id}`);
    setActivities((list) =>
      list.map((a) =>
        a.id === id && a.status === "paused"
          ? { ...a, status: "running" as const }
          : a,
      ),
    );
  };

  const clear = (id: string) => {
    setActivities((list) => list.filter((a) => a.id !== id));
  };

  return {
    activities,
    setActivities,
    registerCleanup,
    unregisterCleanup,
    cancel,
    pause,
    resume,
    clear,
  };
}
