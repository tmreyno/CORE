// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import {
  HiOutlineInformationCircle,
  HiOutlineCodeBracket,
  HiOutlineDocumentText,
  HiOutlineDocument,
  HiOutlineArrowUpTray,
  HiOutlineXMark,
} from "../icons";
import type { TabViewMode } from "./types";

interface ViewModeSelectorProps {
  viewMode: TabViewMode;
  onViewModeChange: (mode: TabViewMode) => void;
  tabCount: number;
  onCloseAll: () => void;
}

const VIEW_MODES: { mode: TabViewMode; label: string; title: string; icon: typeof HiOutlineInformationCircle }[] = [
  { mode: "info", label: "Info", title: "Container Info", icon: HiOutlineInformationCircle },
  { mode: "hex", label: "Hex", title: "Hex Viewer", icon: HiOutlineCodeBracket },
  { mode: "text", label: "Text", title: "Text Viewer", icon: HiOutlineDocumentText },
  { mode: "pdf", label: "PDF", title: "PDF Viewer", icon: HiOutlineDocument },
  { mode: "export", label: "Export", title: "Export Files", icon: HiOutlineArrowUpTray },
];

export function ViewModeSelector(props: ViewModeSelectorProps) {
  return (
    <div class="flex items-center gap-1 px-1.5 shrink-0 border-l border-border/50">
      <div class="flex items-center rounded-md bg-bg-panel/50 p-0.5">
        {VIEW_MODES.map(({ mode, label, title, icon: Icon }) => (
          <button
            class={`flex items-center gap-0.5 px-1.5 py-0.5 text-[10px] leading-tight rounded transition-colors ${
              props.viewMode === mode
                ? "bg-accent text-white"
                : "text-txt-secondary hover:text-txt"
            }`}
            onClick={() => props.onViewModeChange(mode)}
            title={title}
          >
            <Icon class="w-[10px] h-[10px]" /> {label}
          </button>
        ))}
      </div>
      <button
        class="btn-text-danger"
        onClick={props.onCloseAll}
        title="Close all tabs"
        disabled={props.tabCount === 0}
      >
        <HiOutlineXMark class="w-[10px] h-[10px]" /> All
      </button>
    </div>
  );
}
