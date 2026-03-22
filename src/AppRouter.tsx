// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { onMount, onCleanup } from "solid-js";
import App from "./App";
import { ToastProvider } from "./components/Toast";
import { HistoryProvider } from "./hooks/useHistory";
import { initAnnouncer } from "./utils/accessibility";
import { initGlobalErrorHandlers, removeGlobalErrorHandlers } from "./utils/telemetry";
import { logger } from "./utils/logger";

/**
 * AppRouter - Root component with global providers
 * 
 * Wraps the application with:
 * - ToastProvider for notifications
 * - HistoryProvider for undo/redo
 * - Error handlers and accessibility features
 */
export function AppRouter() {
  const log = logger.scope("AppRouter");
  log.debug("AppRouter rendering — setting up global providers");
  
  // Initialize global features on mount
  onMount(() => {
    const t0 = performance.now();
    
    // Initialize screen reader announcer
    initAnnouncer();
    log.debug(`Screen reader announcer initialized (+${(performance.now() - t0).toFixed(1)}ms)`);
    
    // Initialize global error handlers for uncaught exceptions
    initGlobalErrorHandlers();
    log.debug(`Global error handlers registered (+${(performance.now() - t0).toFixed(1)}ms)`);
    
    log.info(`Global providers initialized (+${(performance.now() - t0).toFixed(1)}ms)`);
  });
  
  onCleanup(() => {
    // Cleanup error handlers
    removeGlobalErrorHandlers();
  });

  return (
    <ToastProvider>
      <HistoryProvider>
        <div class="min-h-screen">
          <App />
        </div>
      </HistoryProvider>
    </ToastProvider>
  );
}
