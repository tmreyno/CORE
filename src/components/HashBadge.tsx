// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * HashBadge - Reusable hash status indicator component
 * 
 * Displays hash verification status with consistent styling across the app.
 * Shows verification state (verified/mismatch/computed/stored/none),
 * progress during hashing, and hash count badges.
 */

import { Show, type JSX } from "solid-js";
import type { ContainerInfo, HashHistoryEntry } from "../types";
import type { FileStatus, FileHashInfo } from "../hooks";
import {
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
  HiOutlineHashtag,
} from "./icons";

/** Hash verification state */
export type HashState = "verified" | "failed" | "computed" | "stored" | "incomplete" | "none";

/** Props for HashBadge component */
export interface HashBadgeProps {
  /** Current file hash info from computation */
  fileHash?: FileHashInfo | null;
  /** Hash computation status */
  fileStatus?: FileStatus | null;
  /** Container info with stored hashes */
  containerInfo?: ContainerInfo | null;
  /** Hash history entries */
  hashHistory?: HashHistoryEntry[];
  /** Whether the file is busy (disable click) */
  busy?: boolean;
  /** Callback when badge is clicked (for hash/re-hash) */
  onHash?: () => void;
  /** Size variant */
  size?: "sm" | "md";
  /** Show count badge */
  showCount?: boolean;
}

/** Calculate hash state from props */
export function getHashState(props: {
  fileHash?: FileHashInfo | null;
  containerInfo?: ContainerInfo | null;
  hashHistory?: HashHistoryEntry[];
}): HashState {
  const { fileHash, containerInfo, hashHistory = [] } = props;
  
  // Check for incomplete container first
  if ((containerInfo?.ad1?.missing_segments?.length ?? 0) > 0) {
    return "incomplete";
  }
  
  const hash = fileHash;
  if (hash?.verified === true) return "verified";
  if (hash?.verified === false) return "failed";
  if (hash) return "computed";
  
  // Check for verified matches even without current fileHash
  if (hasVerifiedMatch(containerInfo, hashHistory)) return "verified";
  
  const storedCount = getStoredHashCount(containerInfo);
  if (storedCount > 0) return "stored";
  if (hashHistory.length > 0) return "computed"; // Has history but no stored to verify against
  
  return "none";
}

/** Check if any stored hash matches history */
export function hasVerifiedMatch(
  containerInfo?: ContainerInfo | null,
  hashHistory?: HashHistoryEntry[]
): boolean {
  const storedHashes = [
    ...(containerInfo?.e01?.stored_hashes ?? []),
    ...(containerInfo?.ufed?.stored_hashes ?? []),
    ...(containerInfo?.companion_log?.stored_hashes ?? [])
  ];
  const history = hashHistory ?? [];
  
  // Check if any stored hash matches any history hash
  for (const stored of storedHashes) {
    const match = history.find(h =>
      h.algorithm.toLowerCase() === stored.algorithm.toLowerCase() &&
      h.hash.toLowerCase() === stored.hash.toLowerCase()
    );
    if (match) return true;
  }
  
  // Check if any history entries match each other (same algorithm, same hash, different times)
  for (let i = 0; i < history.length; i++) {
    for (let j = i + 1; j < history.length; j++) {
      if (history[i].algorithm.toLowerCase() === history[j].algorithm.toLowerCase() &&
          history[i].hash.toLowerCase() === history[j].hash.toLowerCase()) {
        return true;
      }
    }
  }
  
  return false;
}

/** Count stored hashes */
export function getStoredHashCount(containerInfo?: ContainerInfo | null): number {
  return (containerInfo?.e01?.stored_hashes?.length ?? 0) +
         (containerInfo?.companion_log?.stored_hashes?.length ?? 0);
}

/** Get total hash count */
export function getTotalHashCount(props: {
  fileHash?: FileHashInfo | null;
  containerInfo?: ContainerInfo | null;
  hashHistory?: HashHistoryEntry[];
}): number {
  const storedCount = getStoredHashCount(props.containerInfo);
  const historyCount = props.hashHistory?.length ?? 0;
  return storedCount + (props.fileHash ? 1 : 0) + historyCount;
}

/** Check if currently hashing */
export function isHashing(fileStatus?: FileStatus | null, fileHash?: FileHashInfo | null): boolean {
  return fileStatus?.status === "hashing" && !fileHash && (fileStatus?.progress ?? 0) < 95;
}

/** Check if hash is completing (95%+ but not done) */
export function isCompleting(fileStatus?: FileStatus | null, fileHash?: FileHashInfo | null): boolean {
  return fileStatus?.status === "hashing" && (fileStatus?.progress ?? 0) >= 95 && !fileHash;
}

/** Format chunk count for display */
export function formatChunks(count: number): string {
  if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M`;
  if (count >= 1000) return `${(count / 1000).toFixed(0)}k`;
  return count.toString();
}

/** Count badge component */
function CountBadge(props: { count: number; colorClass: string }) {
  return (
    <Show when={props.count > 0}>
      <span class={`count-badge ${props.colorClass}`}>{props.count}</span>
    </Show>
  );
}

/**
 * HashBadge Component
 * 
 * Displays hash status with appropriate styling:
 * - Green checkmarks for verified
 * - Red for failed/mismatch
 * - Cyan for computed (no stored to verify)
 * - Purple for stored (awaiting verification)
 * - Orange for incomplete containers
 * - Gray for no hash
 * - Amber for in-progress hashing
 */
export function HashBadge(props: HashBadgeProps): JSX.Element {
  const hashState = () => getHashState({
    fileHash: props.fileHash ?? undefined,
    containerInfo: props.containerInfo ?? undefined,
    hashHistory: props.hashHistory,
  });
  
  const totalCount = () => getTotalHashCount({
    fileHash: props.fileHash ?? undefined,
    containerInfo: props.containerInfo ?? undefined,
    hashHistory: props.hashHistory,
  });
  
  const storedCount = () => getStoredHashCount(props.containerInfo);
  const hashProgress = () => props.fileStatus?.progress ?? 0;
  const chunksProcessed = () => props.fileStatus?.chunksProcessed;
  const chunksTotal = () => props.fileStatus?.chunksTotal;
  const hasChunkProgress = () => chunksTotal() !== undefined && chunksTotal()! > 0 && hashProgress() < 100;
  const hashing = () => isHashing(props.fileStatus, props.fileHash);
  const completing = () => isCompleting(props.fileStatus, props.fileHash);
  
  // Hashing in progress
  if (hashing()) {
    return (
      <span
        class="chip chip-amber animate-pulse"
        title={hasChunkProgress()
          ? `Hashing... ${hashProgress().toFixed(0)}%\nDecompressing: ${formatChunks(chunksProcessed() ?? 0)}/${formatChunks(chunksTotal() ?? 0)} chunks`
          : `Hashing... ${hashProgress().toFixed(0)}%`
        }
      >
        <span>{hashProgress().toFixed(0)}%</span>
        <span class="relative">
          <span class="font-bold">#</span>
          <Show when={props.showCount !== false}>
            <CountBadge count={totalCount()} colorClass="bg-amber-500" />
          </Show>
        </span>
      </span>
    );
  }
  
  // Completing (95%+ but not done)
  if (completing()) {
    return (
      <span class="chip chip-green" title="Finalizing hash...">
        <HiOutlineCheckCircle class="w-3 h-3" />
        <span class="relative">
          <HiOutlineHashtag class={`w-3 h-3 font-bold`} />
          <Show when={props.showCount !== false}>
            <CountBadge count={totalCount()} colorClass="bg-green-500" />
          </Show>
        </span>
      </span>
    );
  }
  
  // State-based rendering
  const state = hashState();
  
  switch (state) {
    case "verified":
      return (
        <button
          class="chip chip-green-hover disabled:opacity-50"
          onClick={(e) => { e.stopPropagation(); props.onHash?.(); }}
          disabled={props.busy}
          title={`VERIFIED: Hash matches ${props.fileHash ? "stored hash" : "in history"}\n${totalCount()} hash(es) • Click to re-hash`}
        >
          <span class="relative inline-flex items-center text-green-400">
            <span>✓</span>
            <span class="absolute left-[3px]">✓</span>
          </span>
          <span class="relative">
            <span class="font-bold">#</span>
            <Show when={props.showCount !== false}>
              <CountBadge count={totalCount()} colorClass="bg-green-500" />
            </Show>
          </span>
        </button>
      );
      
    case "failed":
      return (
        <button
          class="chip chip-red disabled:opacity-50"
          onClick={(e) => { e.stopPropagation(); props.onHash?.(); }}
          disabled={props.busy}
          title={`✗ MISMATCH: ${props.fileHash?.algorithm ?? "hash"} does NOT match stored hash\n${totalCount()} hash(es) • Click to re-hash`}
        >
          <span class="text-red-400 font-bold">✗</span>
          <span class="relative">
            <span class="font-bold">#</span>
            <Show when={props.showCount !== false}>
              <CountBadge count={totalCount()} colorClass="bg-red-500" />
            </Show>
          </span>
        </button>
      );
      
    case "computed":
      return (
        <button
          class="chip chip-cyan disabled:opacity-50"
          onClick={(e) => { e.stopPropagation(); props.onHash?.(); }}
          disabled={props.busy}
          title={`✓ Computed: ${props.fileHash?.algorithm ?? "hash"} (no stored hash to verify against)\n${totalCount()} hash(es) • Click to re-hash`}
        >
          <span class="relative">
            <span class="font-bold">#</span>
            <Show when={props.showCount !== false}>
              <CountBadge count={totalCount()} colorClass="bg-accent" />
            </Show>
          </span>
        </button>
      );
      
    case "incomplete":
      return (
        <span
          class="chip chip-orange"
          title={`Incomplete: Missing ${props.containerInfo?.ad1?.missing_segments?.length ?? 0} segment(s)\nCannot hash - segments are missing`}
        >
          <HiOutlineExclamationTriangle class="w-3 h-3" />
          <HiOutlineHashtag class="w-3 h-3" />
        </span>
      );
      
    case "stored":
      return (
        <button
          class="chip chip-purple disabled:opacity-50"
          onClick={(e) => { e.stopPropagation(); props.onHash?.(); }}
          disabled={props.busy}
          title={`${storedCount()} stored hash(es) • Click to verify`}
        >
          <span class="relative">
            <span class="font-bold">#</span>
            <Show when={props.showCount !== false}>
              <CountBadge count={storedCount()} colorClass="bg-purple-500" />
            </Show>
          </span>
        </button>
      );
      
    case "none":
    default:
      return (
        <button
          class="chip chip-neutral disabled:opacity-50"
          onClick={(e) => { e.stopPropagation(); props.onHash?.(); }}
          disabled={props.busy}
          title="Click to hash this file"
        >
          <span class="font-bold">#</span>
        </button>
      );
  }
}

/**
 * Simple hash verification indicator
 * Used in places where just the status icon/text is needed
 */
export function HashVerificationIndicator(props: {
  verified: boolean | null | undefined;
  class?: string;
}) {
  const colorClass = () =>
    props.verified === true ? "text-success" :
    props.verified === false ? "text-error" : "text-txt/60";
  
  const indicator = () =>
    props.verified === true ? " ✓" :
    props.verified === false ? " ✗" : "";
  
  return (
    <span class={`${colorClass()} ${props.class ?? ""}`}>
      {indicator()}
    </span>
  );
}
