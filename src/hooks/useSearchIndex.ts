// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useSearchIndex — manages the Tantivy search index lifecycle.
 *
 * - Opens index when a project is opened
 * - Indexes discovered containers (metadata-only by default)
 * - Closes index when the project is closed
 * - Provides indexing progress and stats
 */

import { createSignal, createEffect, on, onCleanup } from "solid-js";
import {
  openSearchIndex,
  closeSearchIndex,
  indexContainer,
  indexAllContainers,
  rebuildSearchIndex,
  getSearchIndexStats,
  onIndexProgress,
  type IndexStats,
  type IndexProgress,
} from "../api/search";
import { logger } from "../utils/logger";

const log = logger.scope("SearchIndex");

export interface UseSearchIndexDeps {
  /** Whether a project is currently open */
  hasProject: () => boolean;
  /** The .cffx project file path (null when no project) */
  projectPath: () => string | null;
  /** Discovered evidence file paths */
  discoveredFilePaths: () => string[];
}

export function useSearchIndex(deps: UseSearchIndexDeps) {
  const [indexReady, setIndexReady] = createSignal(false);
  const [indexing, setIndexing] = createSignal(false);
  const [indexProgress, setIndexProgress] = createSignal<IndexProgress | null>(null);
  const [stats, setStats] = createSignal<IndexStats | null>(null);

  /** Derive .ffxdb path from .cffx path */
  const ffxdbPath = () => {
    const cffx = deps.projectPath();
    if (!cffx) return null;
    return cffx.replace(/\.cffx$/, ".ffxdb");
  };

  // --- Open/close index with project lifecycle ---
  createEffect(on(
    () => deps.hasProject(),
    async (hasProject, prevHasProject) => {
      if (hasProject && !prevHasProject) {
        // Project just opened — open search index
        const dbPath = ffxdbPath();
        if (dbPath) {
          try {
            const s = await openSearchIndex(dbPath);
            setStats(s);
            setIndexReady(true);
            log.info(`Search index opened: ${s.numDocs} docs, ${s.numSegments} segments`);
          } catch (err) {
            log.warn("Failed to open search index (non-fatal):", err);
            setIndexReady(false);
          }
        }
      } else if (!hasProject && prevHasProject) {
        // Project closed — close search index
        setIndexReady(false);
        setStats(null);
        setIndexProgress(null);
        try {
          await closeSearchIndex();
          log.info("Search index closed");
        } catch (err) {
          log.warn("Failed to close search index:", err);
        }
      }
    }
  ));

  // --- Auto-index when new files appear and index is ready ---
  let lastIndexedCount = 0;
  createEffect(on(
    () => [indexReady(), deps.discoveredFilePaths().length] as const,
    ([ready, fileCount]) => {
      if (ready && fileCount > 0 && fileCount !== lastIndexedCount && !indexing()) {
        lastIndexedCount = fileCount;
        // Auto-index metadata only (content extraction is opt-in via UI)
        const paths = deps.discoveredFilePaths();
        log.info(`Auto-indexing ${paths.length} containers (metadata only)`);
        setIndexing(true);
        indexAllContainers(paths, false).catch((err) => {
          log.warn("Auto-indexing failed:", err);
          setIndexing(false);
        });
      }
    }
  ));

  // --- Listen for indexing progress events ---
  let unlistenProgress: (() => void) | null = null;
  
  (async () => {
    unlistenProgress = await onIndexProgress((progress) => {
      setIndexProgress(progress);
      if (progress.phase === "done") {
        setIndexing(false);
        // Refresh stats after indexing completes
        getSearchIndexStats()
          .then(setStats)
          .catch(() => {});
      }
    });
  })();

  onCleanup(() => {
    unlistenProgress?.();
  });

  // --- Public API ---

  /**
   * Index a single container (metadata only by default).
   * Call this after a container is loaded.
   */
  const indexSingleContainer = async (containerPath: string, includeContent = false) => {
    if (!indexReady()) return;
    try {
      setIndexing(true);
      await indexContainer(containerPath, includeContent);
    } catch (err) {
      log.warn(`Failed to index container ${containerPath}:`, err);
      setIndexing(false);
    }
  };

  /**
   * Index all currently discovered containers.
   */
  const indexAllDiscovered = async (includeContent = false) => {
    if (!indexReady()) return;
    const paths = deps.discoveredFilePaths();
    if (paths.length === 0) return;
    try {
      setIndexing(true);
      await indexAllContainers(paths, includeContent);
    } catch (err) {
      log.warn("Failed to index all containers:", err);
      setIndexing(false);
    }
  };

  /**
   * Rebuild the entire search index from scratch.
   */
  const rebuildIndex = async (includeContent = false) => {
    if (!indexReady()) return;
    const paths = deps.discoveredFilePaths();
    try {
      setIndexing(true);
      await rebuildSearchIndex(paths, includeContent);
    } catch (err) {
      log.warn("Failed to rebuild search index:", err);
      setIndexing(false);
    }
  };

  /**
   * Refresh stats from the index.
   */
  const refreshStats = async () => {
    if (!indexReady()) return;
    try {
      const s = await getSearchIndexStats();
      setStats(s);
    } catch {
      // ignore
    }
  };

  return {
    /** Whether the search index is open and ready for queries */
    indexReady,
    /** Whether indexing is currently in progress */
    indexing,
    /** Current indexing progress (null when not indexing) */
    indexProgress,
    /** Index statistics (null when no index) */
    stats,
    /** Index a single container */
    indexSingleContainer,
    /** Index all discovered containers */
    indexAllDiscovered,
    /** Rebuild the entire index */
    rebuildIndex,
    /** Refresh index stats */
    refreshStats,
  };
}
