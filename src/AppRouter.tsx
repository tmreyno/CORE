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

/**
 * AppRouter - Root component with global providers
 * 
 * Wraps the application with:
 * - ToastProvider for notifications
 * - HistoryProvider for undo/redo
 * - Error handlers and accessibility features
 */
export function AppRouter() {
  // Initialize global features on mount
  onMount(() => {
    // Initialize screen reader announcer
    initAnnouncer();
    
    // Initialize global error handlers for uncaught exceptions
    initGlobalErrorHandlers();
    
    console.log("[CORE-FFX] Global providers initialized");
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
