// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { defineConfig } from "vitest/config";
import solidPlugin from "vite-plugin-solid";

export default defineConfig({
  plugins: [solidPlugin() as any],
  resolve: {
    conditions: ["development", "browser"],
  },
  test: {
    // Use jsdom for DOM testing
    environment: "jsdom",
    // Ensure solid-js uses browser build
    deps: {
      optimizer: {
        web: {
          include: ["solid-js", "solid-icons"],
        },
      },
    },
    
    // Enable globals (describe, it, expect, etc.)
    globals: true,
    
    // Setup files to run before tests
    setupFiles: ["./src/__tests__/setup.ts"],
    
    // Test file patterns
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
    
    // Coverage configuration
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html"],
      include: ["src/**/*.{ts,tsx}"],
      exclude: [
        "src/**/*.test.{ts,tsx}",
        "src/**/*.spec.{ts,tsx}",
        "src/__tests__/**",
        "src/vite-env.d.ts",
      ],
    },
  },
});
