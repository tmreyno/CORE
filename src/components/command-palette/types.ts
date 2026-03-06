// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { JSX } from "solid-js";

export interface CommandAction {
  id: string;
  label: string;
  shortcut?: string;
  icon?: string | JSX.Element;
  category?: string;
  onSelect: () => void;
  disabled?: boolean;
}

export interface CommandPaletteProps {
  actions: CommandAction[];
  isOpen: boolean;
  onClose: () => void;
  placeholder?: string;
}
