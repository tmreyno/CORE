// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * TreeErrorState - Error display for tree views
 */

import { Show } from 'solid-js';
import { HiOutlineExclamationTriangle, HiOutlineLockClosed, HiOutlineArrowPath } from '../icons';

export interface TreeErrorStateProps {
  /** Error message to display */
  message: string;
  /** Whether this is an encryption error */
  isEncrypted?: boolean;
  /** Retry handler */
  onRetry?: () => void;
  /** Indent depth for alignment */
  depth?: number;
}

export function TreeErrorState(props: TreeErrorStateProps) {
  const paddingLeft = () => props.depth ? `${(props.depth + 1) * 16}px` : '32px';
  
  // Check if this looks like an encryption error
  const isEncrypted = () => 
    props.isEncrypted || 
    props.message.toLowerCase().includes('encrypted') ||
    props.message.toLowerCase().includes('password');
  
  return (
    <div 
      class="flex flex-col gap-2 py-3"
      style={{ 'padding-left': paddingLeft() }}
    >
      <Show when={isEncrypted()} fallback={
        // Standard error display
        <div class="flex items-start gap-2">
          <HiOutlineExclamationTriangle class="w-5 h-5 text-amber-400 shrink-0 mt-0.5" />
          <div class="flex flex-col gap-1">
            <span class="text-sm text-amber-400">Error loading contents</span>
            <span class="text-xs text-txt-muted">{props.message}</span>
          </div>
        </div>
      }>
        {/* Encryption error display */}
        <div class="flex items-start gap-2">
          <HiOutlineLockClosed class="w-5 h-5 text-amber-400 shrink-0 mt-0.5" />
          <div class="flex flex-col gap-1.5">
            <span class="text-sm font-medium text-amber-400">Encrypted Container</span>
            <span class="text-xs text-txt-secondary">
              This container is encrypted and requires a password to open.
            </span>
            <div class="text-xs text-txt-muted mt-1">
              <p>To decrypt with FTK Imager:</p>
              <ol class="list-decimal list-inside mt-1 space-y-0.5 text-txt-muted">
                <li>File → Decrypt AD1 Image</li>
                <li>Select this file</li>
                <li>Enter the password</li>
              </ol>
            </div>
          </div>
        </div>
      </Show>
      
      <Show when={props.onRetry}>
        <button
          class="flex items-center gap-1.5 px-2 py-1 text-xs text-txt-secondary hover:text-txt hover:bg-bg-hover/50 rounded transition-colors w-fit"
          onClick={props.onRetry}
        >
          <HiOutlineArrowPath class="w-3.5 h-3.5" />
          Retry
        </button>
      </Show>
    </div>
  );
}
