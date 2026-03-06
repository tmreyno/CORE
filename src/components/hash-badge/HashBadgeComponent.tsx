// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

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

import { Show, type JSX } from "solid-js";
import type { HashBadgeProps } from "./types";
import {
  getHashState,
  getStoredHashCount,
  getTotalHashCount,
  isHashing,
  isCompleting,
  formatChunks,
} from "./hashHelpers";
import { CountBadge } from "./CountBadge";
import {
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
  HiOutlineHashtag,
} from "../icons";

export function HashBadge(props: HashBadgeProps): JSX.Element {
  const hashState = () =>
    getHashState({
      fileHash: props.fileHash ?? undefined,
      containerInfo: props.containerInfo ?? undefined,
      hashHistory: props.hashHistory,
    });

  const totalCount = () =>
    getTotalHashCount({
      fileHash: props.fileHash ?? undefined,
      containerInfo: props.containerInfo ?? undefined,
      hashHistory: props.hashHistory,
    });

  const storedCount = () => getStoredHashCount(props.containerInfo);
  const hashProgress = () => props.fileStatus?.progress ?? 0;
  const chunksProcessed = () => props.fileStatus?.chunksProcessed;
  const chunksTotal = () => props.fileStatus?.chunksTotal;
  const hasChunkProgress = () =>
    chunksTotal() !== undefined && chunksTotal()! > 0 && hashProgress() < 100;
  const hashing = () => isHashing(props.fileStatus, props.fileHash);
  const completing = () => isCompleting(props.fileStatus, props.fileHash);

  // Hashing in progress
  if (hashing()) {
    return (
      <span
        class="chip chip-amber animate-pulse"
        title={
          hasChunkProgress()
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
          onClick={(e) => {
            e.stopPropagation();
            props.onHash?.();
          }}
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
          onClick={(e) => {
            e.stopPropagation();
            props.onHash?.();
          }}
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
          onClick={(e) => {
            e.stopPropagation();
            props.onHash?.();
          }}
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
          onClick={(e) => {
            e.stopPropagation();
            props.onHash?.();
          }}
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
          onClick={(e) => {
            e.stopPropagation();
            props.onHash?.();
          }}
          disabled={props.busy}
          title="Click to hash this file"
        >
          <span class="font-bold">#</span>
        </button>
      );
  }
}
