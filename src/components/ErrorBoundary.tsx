// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, Show, type JSX, type ParentComponent, ErrorBoundary as SolidErrorBoundary } from "solid-js";
import { HiOutlineExclamationTriangle } from "./icons";
import { logger } from "../utils/logger";
const log = logger.scope("ErrorBoundary");

interface ErrorFallbackProps {
  error: Error;
  reset: () => void;
}

/**
 * Default error fallback UI
 */
function DefaultErrorFallback(props: ErrorFallbackProps) {
  const [showDetails, setShowDetails] = createSignal(false);

  return (
    <div class="flex flex-col items-center justify-center p-8 text-center bg-red-900/10 border border-red-500/30 rounded-lg">
      <HiOutlineExclamationTriangle class="w-10 h-10 text-red-400 mb-4" />
      <h3 class="text-lg font-semibold text-red-400 m-0 mb-2">Something went wrong</h3>
      <p class="text-sm text-txt-secondary m-0 mb-4">
        An unexpected error occurred in this section.
      </p>
      
      <div class="flex gap-2">
        <button 
          class="btn-sm-primary"
          onClick={props.reset}
        >
          Try Again
        </button>
        <button 
          class="btn-sm"
          onClick={() => setShowDetails((v) => !v)}
        >
          {showDetails() ? "Hide Details" : "Show Details"}
        </button>
      </div>

      <Show when={showDetails()}>
        <div class="mt-4 w-full text-left">
          <div class="text-sm font-mono text-red-400 mb-2">{props.error.name}</div>
          <pre class="text-xs bg-bg-panel p-3 rounded overflow-x-auto text-txt-tertiary">{props.error.message}</pre>
          <Show when={props.error.stack}>
            <details class="mt-2">
              <summary class="text-xs text-txt-muted cursor-pointer hover:text-txt-tertiary">Stack Trace</summary>
              <pre class="text-xs bg-bg-panel p-3 rounded overflow-x-auto text-txt-secondary mt-1">{props.error.stack}</pre>
            </details>
          </Show>
        </div>
      </Show>
    </div>
  );
}

interface ErrorBoundaryProps {
  /** Custom fallback component */
  fallback?: (error: Error, reset: () => void) => JSX.Element;
  /** Called when error is caught */
  onError?: (error: Error) => void;
  /** Name/label for this boundary (for logging) */
  name?: string;
}

/**
 * Error Boundary component - catches errors in child components
 * 
 * Usage:
 * ```tsx
 * <ErrorBoundary name="FilePanel">
 *   <FilePanel ... />
 * </ErrorBoundary>
 * ```
 */
export const ErrorBoundary: ParentComponent<ErrorBoundaryProps> = (props) => {
  return (
    <SolidErrorBoundary
      fallback={(err, reset) => {
        // Log the error
        const error = err instanceof Error ? err : new Error(String(err));
        log.error(`[ErrorBoundary${props.name ? `: ${props.name}` : ""}]`, error);
        
        // Call custom error handler
        props.onError?.(error);

        // Render custom or default fallback
        if (props.fallback) {
          return props.fallback(error, reset);
        }
        return <DefaultErrorFallback error={error} reset={reset} />;
      }}
    >
      {props.children}
    </SolidErrorBoundary>
  );
};

/**
 * Compact error boundary for smaller UI sections
 */
export function CompactErrorBoundary(props: { children: JSX.Element; name?: string }) {
  return (
    <SolidErrorBoundary
      fallback={(err, reset) => {
        const error = err instanceof Error ? err : new Error(String(err));
        log.error(`[CompactErrorBoundary${props.name ? `: ${props.name}` : ""}]`, error);
        
        return (
          <div class="flex items-center gap-2 px-3 py-2 bg-red-900/10 border border-red-500/30 rounded text-sm">
            <HiOutlineExclamationTriangle class="w-4 h-4 text-red-400" />
            <span class="text-red-400">Error</span>
            <button 
              class="text-xs text-accent hover:underline focus:outline-none"
              onClick={reset}
            >
              Retry
            </button>
          </div>
        );
      }}
    >
      {props.children}
    </SolidErrorBoundary>
  );
}
