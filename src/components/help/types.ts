// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";

export interface HelpSection {
  id: string;
  title: string;
  icon: Component<{ class?: string }>;
  content: Component;
}
