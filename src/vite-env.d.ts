// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/// <reference types="vite/client" />

/** App version injected at build time from package.json */
declare const __APP_VERSION__: string;

/** Build edition: "full" (default), "acquire", or "review" */
declare const __APP_EDITION__: string;

/** GitHub PAT for private repo update checks (build-time, optional) */
declare const __GITHUB_UPDATE_TOKEN__: string;
