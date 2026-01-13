// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Component Playground - Visual testing and demonstration of UI components
 * 
 * This module provides a development-only playground for testing and
 * demonstrating UI components in isolation.
 */

import { Component, createSignal, For, Show } from "solid-js";
import { 
  Skeleton, 
  SkeletonFileRow, 
  SkeletonLoader,
  SkeletonText,
  SkeletonTree,
} from "./Skeleton";
import { Toast, ToastProvider, useToast } from "./Toast";
import { Tooltip } from "./Tooltip";
import { Fade, SlideUp, Collapse } from "./Transition";
import { ThemeSwitcher } from "./ThemeSwitcher";
import { EmptyState, NoFilesEmptyState, ErrorEmptyState } from "./EmptyState";
import { Breadcrumb, type BreadcrumbItem } from "./Breadcrumb";
import DragDrop from "./DragDrop";
import { 
  HiOutlineFolder, 
  HiOutlineDocument, 
  HiOutlineCog6Tooth,
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
} from "./icons";

// ============================================================================
// Playground Component Types
// ============================================================================

interface PlaygroundSection {
  id: string;
  title: string;
  description: string;
  component: Component;
}

// ============================================================================
// Demo Components
// ============================================================================

const SkeletonDemo: Component = () => {
  return (
    <div class="space-y-4">
      <h4 class="text-sm font-medium text-txt-muted">Basic Skeleton</h4>
      <div class="space-y-2">
        <Skeleton width="100%" height="1rem" />
        <Skeleton width="80%" height="1rem" />
        <Skeleton width="60%" height="1rem" />
      </div>

      <h4 class="text-sm font-medium text-txt-muted mt-6">Skeleton Text</h4>
      <SkeletonText lines={3} />

      <h4 class="text-sm font-medium text-txt-muted mt-6">File Row Skeleton</h4>
      <div class="space-y-1">
        <SkeletonFileRow />
        <SkeletonFileRow />
        <SkeletonFileRow />
      </div>

      <h4 class="text-sm font-medium text-txt-muted mt-6">Skeleton Loader</h4>
      <div class="h-32 border border-border rounded">
        <SkeletonLoader message="Loading files..." />
      </div>

      <h4 class="text-sm font-medium text-txt-muted mt-6">Tree Skeleton</h4>
      <SkeletonTree items={5} maxDepth={3} />
    </div>
  );
};

const ToastDemo: Component = () => {
  const toast = useToast();

  return (
    <div class="space-y-4">
      <p class="text-sm text-txt-muted">Click buttons to trigger toast notifications:</p>
      <div class="flex flex-wrap gap-2">
        <button
          class="px-3 py-1.5 text-sm bg-green-600 text-white rounded hover:bg-green-500"
          onClick={() => toast.success("Success!", "Operation completed successfully.")}
        >
          Success Toast
        </button>
        <button
          class="px-3 py-1.5 text-sm bg-red-600 text-white rounded hover:bg-red-500"
          onClick={() => toast.error("Error!", "Something went wrong.")}
        >
          Error Toast
        </button>
        <button
          class="px-3 py-1.5 text-sm bg-yellow-600 text-white rounded hover:bg-yellow-500"
          onClick={() => toast.warning("Warning!", "Please review before continuing.")}
        >
          Warning Toast
        </button>
        <button
          class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-500"
          onClick={() => toast.info("Info", "Here's some useful information.")}
        >
          Info Toast
        </button>
      </div>
    </div>
  );
};

const TooltipDemo: Component = () => {
  return (
    <div class="space-y-4">
      <p class="text-sm text-txt-muted">Hover over buttons to see tooltips:</p>
      <div class="flex flex-wrap gap-4">
        <Tooltip content="This is a top tooltip" position="top">
          <button class="px-3 py-1.5 text-sm bg-bg-hover rounded">Top</button>
        </Tooltip>
        <Tooltip content="This is a bottom tooltip" position="bottom">
          <button class="px-3 py-1.5 text-sm bg-bg-hover rounded">Bottom</button>
        </Tooltip>
        <Tooltip content="This is a left tooltip" position="left">
          <button class="px-3 py-1.5 text-sm bg-bg-hover rounded">Left</button>
        </Tooltip>
        <Tooltip content="This is a right tooltip" position="right">
          <button class="px-3 py-1.5 text-sm bg-bg-hover rounded">Right</button>
        </Tooltip>
      </div>
    </div>
  );
};

const TransitionDemo: Component = () => {
  const [showFade, setShowFade] = createSignal(false);
  const [showSlide, setShowSlide] = createSignal(false);
  const [showCollapse, setShowCollapse] = createSignal(false);

  return (
    <div class="space-y-6">
      <div>
        <button
          class="px-3 py-1.5 text-sm bg-accent text-white rounded mb-2"
          onClick={() => setShowFade(!showFade())}
        >
          Toggle Fade
        </button>
        <Fade show={showFade()}>
          <div class="p-4 bg-bg-hover rounded">Fade content</div>
        </Fade>
      </div>

      <div>
        <button
          class="px-3 py-1.5 text-sm bg-accent text-white rounded mb-2"
          onClick={() => setShowSlide(!showSlide())}
        >
          Toggle Slide Up
        </button>
        <SlideUp show={showSlide()}>
          <div class="p-4 bg-bg-hover rounded">Slide up content</div>
        </SlideUp>
      </div>

      <div>
        <button
          class="px-3 py-1.5 text-sm bg-accent text-white rounded mb-2"
          onClick={() => setShowCollapse(!showCollapse())}
        >
          Toggle Collapse
        </button>
        <Collapse show={showCollapse()}>
          <div class="p-4 bg-bg-hover rounded">
            <p>Collapsible content</p>
            <p>With multiple lines</p>
            <p>That expand and collapse</p>
          </div>
        </Collapse>
      </div>
    </div>
  );
};

const EmptyStateDemo: Component = () => {
  return (
    <div class="space-y-6">
      <div class="border border-border rounded p-4">
        <EmptyState
          icon="📁"
          title="No files found"
          description="Open an evidence directory to get started"
          action={{ label: "Open Directory", onClick: () => console.log("Open clicked") }}
        />
      </div>

      <div class="border border-border rounded p-4">
        <NoFilesEmptyState onBrowse={() => console.log("Open files")} />
      </div>

      <div class="border border-border rounded p-4">
        <ErrorEmptyState 
          error="Failed to load file: Connection timeout" 
          onRetry={() => console.log("Retry")} 
        />
      </div>
    </div>
  );
};

const BreadcrumbDemo: Component = () => {
  const [path, setPath] = createSignal<BreadcrumbItem[]>([
    { label: "Root", path: "/" },
    { label: "Documents", path: "/Documents" },
    { label: "Evidence", path: "/Documents/Evidence" },
    { label: "Case-001", path: "/Documents/Evidence/Case-001" },
  ]);

  return (
    <div class="space-y-4">
      <Breadcrumb
        items={path()}
        onNavigate={(p) => console.log("Navigate to:", p)}
        maxItems={4}
      />

      <div class="mt-4">
        <button
          class="px-3 py-1.5 text-sm bg-bg-hover rounded"
          onClick={() => setPath(p => [...p, { 
            label: `Folder-${p.length}`, 
            path: `${p[p.length-1].path}/Folder-${p.length}` 
          }])}
        >
          Add Level
        </button>
      </div>
    </div>
  );
};

const DragDropDemo: Component = () => {
  const [files, setFiles] = createSignal<string[]>([]);

  return (
    <div class="space-y-4">
      <DragDrop
        onDrop={(dropped) => {
          setFiles(prev => [...prev, ...dropped.map(f => f.name)]);
        }}
        accept={[".ad1", ".e01", ".zip"]}
      >
        <div class="p-8 text-center">
          <p class="text-txt-muted">Drop evidence files here</p>
          <p class="text-xs text-txt-muted mt-1">Supported: .ad1, .e01, .zip</p>
        </div>
      </DragDrop>

      <Show when={files().length > 0}>
        <div class="p-3 bg-bg-hover rounded">
          <h4 class="text-sm font-medium mb-2">Dropped files:</h4>
          <ul class="text-sm text-txt-muted">
            <For each={files()}>
              {(file) => <li>• {file}</li>}
            </For>
          </ul>
        </div>
      </Show>
    </div>
  );
};

const IconsDemo: Component = () => {
  return (
    <div class="space-y-4">
      <p class="text-sm text-txt-muted">Icons from Heroicons (solid-icons/hi):</p>
      <div class="flex flex-wrap gap-4">
        <div class="flex items-center gap-2 p-2 bg-bg-hover rounded">
          <HiOutlineFolder class="w-5 h-5" />
          <span class="text-sm">Folder</span>
        </div>
        <div class="flex items-center gap-2 p-2 bg-bg-hover rounded">
          <HiOutlineDocument class="w-5 h-5" />
          <span class="text-sm">Document</span>
        </div>
        <div class="flex items-center gap-2 p-2 bg-bg-hover rounded">
          <HiOutlineCog6Tooth class="w-5 h-5" />
          <span class="text-sm">Settings</span>
        </div>
        <div class="flex items-center gap-2 p-2 bg-bg-hover rounded text-green-400">
          <HiOutlineCheckCircle class="w-5 h-5" />
          <span class="text-sm">Success</span>
        </div>
        <div class="flex items-center gap-2 p-2 bg-bg-hover rounded text-yellow-400">
          <HiOutlineExclamationTriangle class="w-5 h-5" />
          <span class="text-sm">Warning</span>
        </div>
      </div>
    </div>
  );
};

const ThemeDemo: Component = () => {
  return (
    <div class="space-y-4">
      <p class="text-sm text-txt-muted">Toggle between light and dark themes:</p>
      <ThemeSwitcher />
    </div>
  );
};

// ============================================================================
// Main Playground Component
// ============================================================================

const sections: PlaygroundSection[] = [
  { id: "skeleton", title: "Skeleton Loading", description: "Loading placeholder components", component: SkeletonDemo },
  { id: "toast", title: "Toast Notifications", description: "Notification system", component: ToastDemo },
  { id: "tooltip", title: "Tooltips", description: "Hover tooltips", component: TooltipDemo },
  { id: "transition", title: "Transitions", description: "Animation transitions", component: TransitionDemo },
  { id: "empty", title: "Empty States", description: "Placeholder states", component: EmptyStateDemo },
  { id: "breadcrumb", title: "Breadcrumb", description: "Navigation breadcrumbs", component: BreadcrumbDemo },
  { id: "dragdrop", title: "Drag & Drop", description: "File drop zone", component: DragDropDemo },
  { id: "icons", title: "Icons", description: "Icon library", component: IconsDemo },
  { id: "theme", title: "Theme", description: "Theme switcher", component: ThemeDemo },
];

export const ComponentPlayground: Component = () => {
  const [activeSection, setActiveSection] = createSignal("skeleton");

  const ActiveComponent = () => {
    const section = sections.find(s => s.id === activeSection());
    if (!section) return null;
    return <section.component />;
  };

  return (
    <ToastProvider>
      <div class="flex h-full bg-bg text-txt">
        {/* Sidebar */}
        <div class="w-64 border-r border-border p-4 overflow-y-auto">
          <h2 class="text-lg font-semibold mb-4">Component Playground</h2>
          <nav class="space-y-1">
            <For each={sections}>
              {(section) => (
                <button
                  class={`w-full text-left px-3 py-2 rounded text-sm transition-colors ${
                    activeSection() === section.id
                      ? "bg-accent text-white"
                      : "hover:bg-bg-hover"
                  }`}
                  onClick={() => setActiveSection(section.id)}
                >
                  <div class="font-medium">{section.title}</div>
                  <div class="text-xs opacity-70">{section.description}</div>
                </button>
              )}
            </For>
          </nav>
        </div>

        {/* Content */}
        <div class="flex-1 p-6 overflow-y-auto">
          <div class="max-w-3xl">
            <h3 class="text-xl font-semibold mb-4">
              {sections.find(s => s.id === activeSection())?.title}
            </h3>
            <div class="bg-bg-secondary p-6 rounded-lg border border-border">
              <ActiveComponent />
            </div>
          </div>
        </div>
      </div>
    </ToastProvider>
  );
};

export default ComponentPlayground;
