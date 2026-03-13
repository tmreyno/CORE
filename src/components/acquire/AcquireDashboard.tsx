// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AcquireDashboard — Primary UI for CORE Acquire edition.
 *
 * Replaces the full CORE-FFX three-panel layout with a streamlined
 * acquisition-focused interface inspired by FTK Imager and Magnet Acquire.
 *
 * Action cards:
 *   1. Create Disk Image (E01)
 *   2. Create Logical Image (L01)
 *   3. Export Files / Archive (7z)
 *   4. Browse Evidence
 *   5. Verify Hash
 */

import {
  Component,
  Show,
  For,
  createSignal,
  Accessor,
} from "solid-js";
import {
  HiOutlineCircleStack,
  HiOutlineFolder,
  HiOutlineArchiveBox,
  HiOutlineFingerPrint,
  HiOutlineArrowUpTray,
  HiOutlineFolderOpen,
  HiOutlineCog6Tooth,
  HiOutlineQuestionMarkCircle,
  HiOutlineCommandLine,
  HiOutlineBookmark,
  HiOutlineMagnifyingGlass,
} from "../icons";
import { APP_NAME } from "../../utils/edition";

// =============================================================================
// Types
// =============================================================================

export type AcquireAction =
  | "physical"
  | "logical"
  | "export"
  | "browse"
  | "verify"
  | "collection";

interface ActionCard {
  id: AcquireAction;
  title: string;
  description: string;
  icon: Component<{ class?: string }>;
  accent: string;
}

export interface AcquireDashboardProps {
  /** Handler when an action card is clicked */
  onAction: (action: AcquireAction) => void;
  /** Open settings */
  onSettings: () => void;
  /** Open help */
  onHelp: () => void;
  /** Open command palette */
  onCommandPalette: () => void;
  /** Open a project */
  onOpenProject: () => void;
  /** Create a new project */
  onNewProject: () => void;
  /** Bookmarks */
  onBookmarks: () => void;
  /** Search */
  onSearch: () => void;
  /** Project name if one is loaded */
  projectName: Accessor<string | undefined>;
  /** Whether a project is loaded */
  hasProject: Accessor<boolean>;
  /** Number of evidence files discovered */
  evidenceCount: Accessor<number>;
}

// =============================================================================
// Constants
// =============================================================================

const ACTION_CARDS: ActionCard[] = [
  {
    id: "physical",
    title: "Create Disk Image",
    description: "Acquire a physical or logical drive as an E01 forensic image with hash verification",
    icon: HiOutlineCircleStack,
    accent: "text-blue-400",
  },
  {
    id: "logical",
    title: "Create Logical Image",
    description: "Package files and folders into an L01 logical evidence container",
    icon: HiOutlineFolder,
    accent: "text-emerald-400",
  },
  {
    id: "export",
    title: "Export Files",
    description: "Copy or archive selected files to a 7z container or folder with manifests",
    icon: HiOutlineArrowUpTray,
    accent: "text-amber-400",
  },
  {
    id: "browse",
    title: "Browse Evidence",
    description: "Open and explore E01, AD1, L01, and archive containers",
    icon: HiOutlineArchiveBox,
    accent: "text-purple-400",
  },
  {
    id: "verify",
    title: "Verify Hashes",
    description: "Compute and verify MD5, SHA-1, or SHA-256 hashes of evidence files",
    icon: HiOutlineFingerPrint,
    accent: "text-rose-400",
  },
  {
    id: "collection",
    title: "Evidence Collection",
    description: "Document on-site evidence collection with chain of custody tracking",
    icon: HiOutlineFolderOpen,
    accent: "text-cyan-400",
  },
];

// =============================================================================
// Component
// =============================================================================

const AcquireDashboard: Component<AcquireDashboardProps> = (props) => {
  const [hoveredCard, setHoveredCard] = createSignal<string | null>(null);

  return (
    <div class="acquire-dashboard">
      {/* Top bar — project info + utility buttons */}
      <header class="acquire-topbar">
        <div class="flex items-center gap-3">
          <Show when={props.hasProject()}>
            <div class="flex items-center gap-2 px-3 py-1.5 bg-accent/10 border border-accent/20 rounded-lg">
              <span class="text-sm font-medium text-accent truncate max-w-[200px]">
                {props.projectName()}
              </span>
              <Show when={props.evidenceCount() > 0}>
                <span class="badge badge-success">{props.evidenceCount()} files</span>
              </Show>
            </div>
          </Show>
        </div>

        <div class="flex items-center gap-1">
          <button
            class="icon-btn"
            onClick={props.onSearch}
            title="Search"
          >
            <HiOutlineMagnifyingGlass class="w-5 h-5" />
          </button>
          <button
            class="icon-btn"
            onClick={props.onBookmarks}
            title="Bookmarks"
          >
            <HiOutlineBookmark class="w-5 h-5" />
          </button>
          <button
            class="icon-btn"
            onClick={props.onCommandPalette}
            title="Command Palette (⌘K)"
          >
            <HiOutlineCommandLine class="w-5 h-5" />
          </button>
          <button
            class="icon-btn"
            onClick={props.onSettings}
            title="Settings"
          >
            <HiOutlineCog6Tooth class="w-5 h-5" />
          </button>
          <button
            class="icon-btn"
            onClick={props.onHelp}
            title="Help"
          >
            <HiOutlineQuestionMarkCircle class="w-5 h-5" />
          </button>
        </div>
      </header>

      {/* Hero section */}
      <div class="acquire-hero">
        <h1 class="acquire-title">{APP_NAME}</h1>
        <p class="acquire-subtitle">Evidence Acquisition & Imaging</p>

        {/* Project quick-start row */}
        <Show when={!props.hasProject()}>
          <div class="flex items-center gap-3 mt-4">
            <button
              class="btn btn-primary"
              onClick={props.onNewProject}
            >
              New Project
            </button>
            <button
              class="btn btn-secondary"
              onClick={props.onOpenProject}
            >
              Open Project
            </button>
          </div>
        </Show>
      </div>

      {/* Action Cards Grid */}
      <div class="acquire-grid">
        <For each={ACTION_CARDS}>
          {(card) => {
            const Icon = card.icon;
            return (
              <button
                class="acquire-card"
                classList={{
                  "acquire-card-hover": hoveredCard() === card.id,
                }}
                onMouseEnter={() => setHoveredCard(card.id)}
                onMouseLeave={() => setHoveredCard(null)}
                onClick={() => props.onAction(card.id)}
              >
                <div class={`acquire-card-icon ${card.accent}`}>
                  <Icon class="w-8 h-8" />
                </div>
                <div class="acquire-card-content">
                  <h3 class="acquire-card-title">{card.title}</h3>
                  <p class="acquire-card-desc">{card.description}</p>
                </div>
              </button>
            );
          }}
        </For>
      </div>
    </div>
  );
};

export default AcquireDashboard;
