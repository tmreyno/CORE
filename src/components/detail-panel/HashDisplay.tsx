// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, Component, Accessor } from "solid-js";
import { HiOutlineLockClosed, HiOutlineDocumentDuplicate, HiOutlineCheck, HiOutlineExclamationTriangle } from "../icons";
import type { FileHashInfo } from "../../types/hash";
import type { StoredHash, HashAlgorithm } from "../../types";

interface HashDisplayProps {
  fileHash: FileHashInfo | undefined;
  isHashing: Accessor<boolean>;
  currentProgress: Accessor<number>;
  selectedHashAlgorithm: HashAlgorithm;
  storedHashes: StoredHash[];
}

export const HashDisplay: Component<HashDisplayProps> = (props) => {
  return (
    <>
      {/* Hash progress */}
      <Show when={props.isHashing()}>
        <div class="progress-card">
          <div class="progress-header">
            <span class="progress-title">
              <HiOutlineLockClosed class="w-4 h-4" /> Hashing with {props.selectedHashAlgorithm.toUpperCase()}...
            </span>
            <span class="progress-value">{props.currentProgress().toFixed(1)}%</span>
          </div>
          <div class="progress-bar">
            <div class="progress-fill" style={{ width: `${props.currentProgress()}%` }} />
          </div>
        </div>
      </Show>
      
      {/* Computed hash card */}
      <Show when={props.fileHash && !props.isHashing()}>
        <div class={`rounded-lg border p-3 ${
          props.fileHash!.verified === true 
            ? 'bg-green-900/20 border-green-500/50' 
            : props.fileHash!.verified === false 
              ? 'bg-red-900/20 border-red-500/50' 
              : 'bg-bg-panel/50 border-border/50'
        }`}>
          <div class="flex items-center gap-2 mb-2">
            <span class="text-sm text-accent font-medium">
              <HiOutlineLockClosed class="w-4 h-4 inline" /> {props.fileHash!.algorithm}
            </span>
            
            <Show when={props.fileHash!.verified === true}>
              <span class="inline-flex items-center gap-1 text-xs text-green-400 font-semibold">
                <span class="relative inline-flex">
                  <span>✓</span>
                  <span class="absolute left-[3px]">✓</span>
                </span>
                <span class="ml-1">VERIFIED</span>
              </span>
            </Show>
            
            <Show when={props.fileHash!.verified === false}>
              <span class="text-xs text-red-400 font-semibold flex items-center gap-1">
                <span class="font-bold">✗</span>
                MISMATCH
              </span>
            </Show>
            
            <Show when={props.fileHash!.verified === null}>
              <span class="text-xs text-txt-secondary flex items-center gap-1">
                <HiOutlineCheck class="w-3 h-3" /> Computed
              </span>
            </Show>
            
            <button 
              class="ml-auto text-sm hover:bg-bg-hover p-1 rounded flex items-center" 
              onClick={() => navigator.clipboard.writeText(props.fileHash!.hash)} 
              title="Copy hash"
            >
              <HiOutlineDocumentDuplicate class="w-4 h-4" />
            </button>
          </div>
          
          <code class="block text-xs font-mono text-txt-tertiary break-all bg-bg/50 p-2 rounded">
            {props.fileHash!.hash}
          </code>
          
          <Show when={
            props.fileHash!.verified === true && 
            props.storedHashes.some(sh => sh.algorithm.toLowerCase() === props.fileHash!.algorithm.toLowerCase())
          }>
            <div class="mt-2 text-xs text-green-400 flex items-center gap-1">
              <span class="relative inline-flex">
                <span>✓</span>
                <span class="absolute left-[3px]">✓</span>
              </span>
              <span class="ml-1">Hash matches stored value from container/companion</span>
            </div>
          </Show>
          
          <Show when={
            props.fileHash!.verified === true && 
            !props.storedHashes.some(sh => sh.algorithm.toLowerCase() === props.fileHash!.algorithm.toLowerCase())
          }>
            <div class="mt-2 text-xs text-green-400 flex items-center gap-1">
              <span class="relative inline-flex">
                <span>✓</span>
                <span class="absolute left-[3px]">✓</span>
              </span>
              <span class="ml-1">Hash matches previous computation (self-verified)</span>
            </div>
          </Show>
          
          <Show when={props.fileHash!.verified === false}>
            <div class="mt-2 text-xs text-red-400 flex items-center gap-1">
              <HiOutlineExclamationTriangle class="w-3 h-3" /> Computed hash does NOT match stored hash!
            </div>
          </Show>
          
          <Show when={props.fileHash!.verified === null && props.storedHashes.length === 0}>
            <div class="mt-2 text-xs text-txt-muted">No stored hash or history to verify against</div>
          </Show>
        </div>
      </Show>
    </>
  );
};
