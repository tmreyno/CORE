// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useAppStore - Store-based state management using solid-js/store.
 * 
 * Provides efficient nested state updates with fine-grained reactivity.
 * Use this for complex nested data structures like transfer jobs, tabs, etc.
 * 
 * Benefits over createSignal:
 * - Fine-grained reactivity for nested objects
 * - Efficient partial updates without recreating entire objects
 * - Better performance with large arrays/objects
 * - produce() helper for immutable-style mutations
 */

import { createStore, produce, reconcile } from "solid-js/store";
import type { TransferJob } from "../components";
import type { TransferProgress } from "../transfer";

// =============================================================================
// Types
// =============================================================================

export interface TransferJobStore {
  jobs: TransferJob[];
}

export interface TransferActions {
  addJob: (job: TransferJob) => void;
  updateJob: (id: string, updates: Partial<TransferJob>) => void;
  updateJobProgress: (id: string, progress: TransferProgress) => void;
  completeJob: (id: string, success: boolean) => void;
  removeJob: (id: string) => void;
  clearCompleted: () => void;
}

// =============================================================================
// Transfer Jobs Store
// =============================================================================

/**
 * Creates a store for managing transfer jobs with efficient updates.
 * Uses produce() for immutable-style mutations with fine-grained reactivity.
 */
export function createTransferStore(): [TransferJobStore, TransferActions] {
  const [store, setStore] = createStore<TransferJobStore>({ jobs: [] });

  const actions: TransferActions = {
    addJob: (job: TransferJob) => {
      setStore(produce((s) => {
        s.jobs.push(job);
      }));
    },

    updateJob: (id: string, updates: Partial<TransferJob>) => {
      setStore(produce((s) => {
        const idx = s.jobs.findIndex(j => j.id === id);
        if (idx !== -1) {
          Object.assign(s.jobs[idx], updates);
        }
      }));
    },

    updateJobProgress: (id: string, progress: TransferProgress) => {
      setStore(produce((s) => {
        const job = s.jobs.find(j => j.id === id);
        if (job) {
          job.progress = progress;
        }
      }));
    },

    completeJob: (id: string, success: boolean) => {
      setStore(produce((s) => {
        const job = s.jobs.find(j => j.id === id);
        if (job) {
          job.status = success ? "completed" : "failed";
        }
      }));
    },

    removeJob: (id: string) => {
      setStore(produce((s) => {
        const idx = s.jobs.findIndex(j => j.id === id);
        if (idx !== -1) {
          s.jobs.splice(idx, 1);
        }
      }));
    },

    clearCompleted: () => {
      setStore(produce((s) => {
        s.jobs = s.jobs.filter(j => j.status !== "completed" && j.status !== "failed");
      }));
    },
  };

  return [store, actions];
}

// =============================================================================
// Tab Store
// =============================================================================

export interface TabItem {
  id: string;
  label: string;
  path: string;
  type: "file" | "container" | "document";
  modified?: boolean;
  icon?: string;
}

export interface TabStore {
  tabs: TabItem[];
  activeTabId: string | null;
}

export interface TabActions {
  openTab: (tab: TabItem) => void;
  closeTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  updateTab: (id: string, updates: Partial<TabItem>) => void;
  reorderTabs: (fromIndex: number, toIndex: number) => void;
  closeAllTabs: () => void;
  closeOtherTabs: (keepId: string) => void;
}

/**
 * Creates a store for managing open tabs with efficient updates.
 */
export function createTabStore(): [TabStore, TabActions] {
  const [store, setStore] = createStore<TabStore>({
    tabs: [],
    activeTabId: null,
  });

  const actions: TabActions = {
    openTab: (tab: TabItem) => {
      setStore(produce((s) => {
        // Check if tab already exists
        const existing = s.tabs.find(t => t.id === tab.id);
        if (!existing) {
          s.tabs.push(tab);
        }
        s.activeTabId = tab.id;
      }));
    },

    closeTab: (id: string) => {
      setStore(produce((s) => {
        const idx = s.tabs.findIndex(t => t.id === id);
        if (idx !== -1) {
          s.tabs.splice(idx, 1);
          // If closing active tab, activate previous or next
          if (s.activeTabId === id) {
            if (s.tabs.length > 0) {
              const newIdx = Math.min(idx, s.tabs.length - 1);
              s.activeTabId = s.tabs[newIdx].id;
            } else {
              s.activeTabId = null;
            }
          }
        }
      }));
    },

    setActiveTab: (id: string) => {
      setStore("activeTabId", id);
    },

    updateTab: (id: string, updates: Partial<TabItem>) => {
      setStore(produce((s) => {
        const tab = s.tabs.find(t => t.id === id);
        if (tab) {
          Object.assign(tab, updates);
        }
      }));
    },

    reorderTabs: (fromIndex: number, toIndex: number) => {
      setStore(produce((s) => {
        const [removed] = s.tabs.splice(fromIndex, 1);
        if (removed) {
          s.tabs.splice(toIndex, 0, removed);
        }
      }));
    },

    closeAllTabs: () => {
      setStore({ tabs: [], activeTabId: null });
    },

    closeOtherTabs: (keepId: string) => {
      setStore(produce((s) => {
        s.tabs = s.tabs.filter(t => t.id === keepId);
        s.activeTabId = keepId;
      }));
    },
  };

  return [store, actions];
}

// =============================================================================
// Selection Store (for multi-select support)
// =============================================================================

export interface SelectionStore {
  selectedIds: Set<string>;
  lastSelectedId: string | null;
  anchorId: string | null;
}

export interface SelectionActions {
  select: (id: string, mode?: "single" | "toggle" | "range") => void;
  selectAll: (ids: string[]) => void;
  deselectAll: () => void;
  isSelected: (id: string) => boolean;
}

/**
 * Creates a store for managing multi-selection with shift+click range support.
 */
export function createSelectionStore(allIds: () => string[]): [SelectionStore, SelectionActions] {
  const [store, setStore] = createStore<SelectionStore>({
    selectedIds: new Set(),
    lastSelectedId: null,
    anchorId: null,
  });

  const actions: SelectionActions = {
    select: (id: string, mode: "single" | "toggle" | "range" = "single") => {
      setStore(produce((s) => {
        if (mode === "single") {
          s.selectedIds = new Set([id]);
          s.anchorId = id;
        } else if (mode === "toggle") {
          if (s.selectedIds.has(id)) {
            s.selectedIds.delete(id);
          } else {
            s.selectedIds.add(id);
          }
          s.anchorId = id;
        } else if (mode === "range" && s.anchorId) {
          const ids = allIds();
          const anchorIdx = ids.indexOf(s.anchorId);
          const targetIdx = ids.indexOf(id);
          if (anchorIdx !== -1 && targetIdx !== -1) {
            const start = Math.min(anchorIdx, targetIdx);
            const end = Math.max(anchorIdx, targetIdx);
            for (let i = start; i <= end; i++) {
              s.selectedIds.add(ids[i]);
            }
          }
        }
        s.lastSelectedId = id;
      }));
    },

    selectAll: (ids: string[]) => {
      setStore("selectedIds", new Set(ids));
    },

    deselectAll: () => {
      setStore({
        selectedIds: new Set(),
        lastSelectedId: null,
        anchorId: null,
      });
    },

    isSelected: (id: string) => store.selectedIds.has(id),
  };

  return [store, actions];
}

// =============================================================================
// Export
// =============================================================================

export { produce, reconcile };
