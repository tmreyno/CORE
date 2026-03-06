// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import {
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
  HiOutlineHashtag,
} from "../icons";
import type { ContainerInfo, HashHistoryEntry } from "../../types";
import type { FileStatus, FileHashInfo } from "../../hooks";
import type { HashState } from "./types";
import {
  isCurrentlyHashing,
  isCurrentlyCompleting,
  getTotalHashCount,
  getStoredHashCount,
  getHashState,
  formatChunks,
} from "./hashHelpers";

interface HashIndicatorsProps {
  fileStatus: FileStatus | undefined;
  fileInfo: ContainerInfo | undefined;
  fileHash: FileHashInfo | undefined;
  hashHistory: HashHistoryEntry[];
  busy: boolean;
  onHash: () => void;
}

export function HashIndicators(props: HashIndicatorsProps) {
  const hashing = () => isCurrentlyHashing(props.fileStatus, props.fileHash);
  const completing = () => isCurrentlyCompleting(props.fileStatus, props.fileHash);
  const state = (): HashState => getHashState(props.fileInfo, props.fileHash, props.hashHistory);
  const total = () => getTotalHashCount(props.fileInfo, props.fileHash, props.hashHistory);
  const stored = () => getStoredHashCount(props.fileInfo);
  const progress = () => props.fileStatus?.progress ?? 0;
  const chunksProc = () => props.fileStatus?.chunksProcessed;
  const chunksAll = () => props.fileStatus?.chunksTotal;
  const hasChunks = () => chunksAll() !== undefined && chunksAll()! > 0 && progress() < 100;

  return (
    <>
      {/* Hashing in progress */}
      <Show when={hashing()}>
        <span
          class="chip chip-amber animate-pulse"
          title={hasChunks()
            ? `Hashing... ${progress().toFixed(0)}%\nDecompressing: ${formatChunks(chunksProc() ?? 0)}/${formatChunks(chunksAll() ?? 0)} chunks`
            : `Hashing... ${progress().toFixed(0)}%`
          }
        >
          <span>{progress().toFixed(0)}%</span>
          <span class="relative">
            <span class="font-bold">#</span>
            <Show when={total() > 0}>
              <span class="count-badge bg-amber-500">{total()}</span>
            </Show>
          </span>
        </span>
      </Show>

      {/* Completing */}
      <Show when={completing()}>
        <span class="chip chip-green" title="Finalizing hash...">
          <HiOutlineCheckCircle class="w-3 h-3" />
          <span class="relative">
            <HiOutlineHashtag class="w-3 h-3 font-bold" />
            <Show when={total() > 0}>
              <span class="count-badge bg-green-500">{total()}</span>
            </Show>
          </span>
        </span>
      </Show>

      {/* Verified */}
      <Show when={!hashing() && !completing() && state() === "verified"}>
        <button
          class="chip chip-green-hover disabled:opacity-50"
          onClick={(e) => { e.stopPropagation(); props.onHash(); }}
          disabled={props.busy}
          title={`VERIFIED: Hash matches ${props.fileHash ? "stored hash" : "in history"}\n${total()} hash(es) • Click to re-hash`}
        >
          <span class="relative"><span class="absolute -left-0.5">✓</span><span class="ml-1">✓</span></span>
          <span class="relative">
            <span class="font-bold">#</span>
            <span class="count-badge bg-green-500">{total()}</span>
          </span>
        </button>
      </Show>

      {/* Failed */}
      <Show when={!hashing() && !completing() && state() === "failed"}>
        <button
          class="chip chip-red disabled:opacity-50"
          onClick={(e) => { e.stopPropagation(); props.onHash(); }}
          disabled={props.busy}
          title={`✗ MISMATCH: ${props.fileHash?.algorithm ?? "hash"} does NOT match stored hash\n${total()} hash(es) • Click to re-hash`}
        >
          <span class="relative">
            <span class="font-bold">#</span>
            <span class="count-badge bg-red-500">{total()}</span>
          </span>
        </button>
      </Show>

      {/* Computed (no stored to verify) */}
      <Show when={!hashing() && !completing() && state() === "computed"}>
        <button
          class="chip chip-cyan disabled:opacity-50"
          onClick={(e) => { e.stopPropagation(); props.onHash(); }}
          disabled={props.busy}
          title={`✓ Computed: ${props.fileHash?.algorithm ?? "hash"} (no stored hash to verify against)\n${total()} hash(es) • Click to re-hash`}
        >
          <span class="relative">
            <span class="font-bold">#</span>
            <span class="count-badge bg-accent">{total()}</span>
          </span>
        </button>
      </Show>

      {/* Incomplete */}
      <Show when={!hashing() && !completing() && state() === "incomplete"}>
        <span
          class="chip chip-orange"
          title={`Incomplete: Missing ${props.fileInfo?.ad1?.missing_segments?.length ?? 0} segment(s)\nCannot hash - segments are missing`}
        >
          <HiOutlineExclamationTriangle class="w-3 h-3" />
          <HiOutlineHashtag class="w-3 h-3" />
        </span>
      </Show>

      {/* Stored (not yet verified) */}
      <Show when={!hashing() && !completing() && state() === "stored"}>
        <button
          class="chip chip-purple disabled:opacity-50"
          onClick={(e) => { e.stopPropagation(); props.onHash(); }}
          disabled={props.busy}
          title={`${stored()} stored hash(es) • Click to verify`}
        >
          <span class="relative">
            <span class="font-bold">#</span>
            <span class="count-badge bg-purple-500">{stored()}</span>
          </span>
        </button>
      </Show>

      {/* No hash */}
      <Show when={!hashing() && !completing() && state() === "none"}>
        <button
          class="chip chip-neutral disabled:opacity-50"
          onClick={(e) => { e.stopPropagation(); props.onHash(); }}
          disabled={props.busy}
          title="Click to hash this file"
        >
          <span class="font-bold">#</span>
        </button>
      </Show>
    </>
  );
}
