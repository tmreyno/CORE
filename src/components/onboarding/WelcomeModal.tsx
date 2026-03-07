// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For } from "solid-js";
import { Portal } from "solid-js/web";
import {
  HiOutlineFolder,
  HiOutlineFingerPrint,
  HiOutlineChartBar,
  HiOutlineClipboardDocumentList,
  HiOutlinePlusCircle,
  HiOutlineFolderOpen,
  HiOutlineClock,
  HiOutlineDocumentDuplicate,
} from "../icons";
import { Kbd } from "../ui/Kbd";
import { formatRelativeTime } from "./welcomeHelpers";
import type { WelcomeModalProps } from "./welcomeTypes";

export function WelcomeModal(props: WelcomeModalProps) {
  const hasQuickActions = () => !!props.onNewProject || !!props.onOpenProject;
  const hasRecentProjects = () =>
    (props.recentProjects?.()?.length ?? 0) > 0;

  return (
    <Show when={props.isOpen}>
      <Portal>
        <div class="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50 animate-fade-in">
          <div class="bg-bg-panel border border-border rounded-2xl shadow-2xl w-[600px] max-h-[90vh] overflow-hidden animate-slide-up flex flex-col">
            {/* Header with gradient and branding */}
            <div class="relative bg-gradient-to-br from-accent via-accent to-accent/80 p-5 text-center overflow-hidden flex-shrink-0">
              {/* Decorative background elements */}
              <div class="absolute inset-0 opacity-10">
                <div class="absolute top-0 left-0 w-32 h-32 bg-white rounded-full blur-3xl -translate-x-1/2 -translate-y-1/2" />
                <div class="absolute bottom-0 right-0 w-40 h-40 bg-white rounded-full blur-3xl translate-x-1/2 translate-y-1/2" />
              </div>

              <div class="relative">
                <div class="inline-flex items-center justify-center w-12 h-12 bg-white/20 rounded-xl backdrop-blur mb-2">
                  <span class="text-2xl">🔍</span>
                </div>
                <h2 class="text-lg font-bold text-white mb-1">
                  {props.title ?? "Welcome to CORE-FFX"}
                </h2>
                <p class="text-white/80 text-sm">Forensic File Xplorer</p>
              </div>
            </div>

            {/* Content - scrollable */}
            <div class="p-4 overflow-y-auto flex-1">
              <div class="text-txt-secondary text-sm mb-4 leading-relaxed">
                {props.description ?? (
                  <p>
                    CORE-FFX is a powerful forensic file explorer for analyzing
                    digital evidence containers like E01, AD1, L01, and more.
                    Get started by creating a new project or opening an existing
                    one.
                  </p>
                )}
              </div>

              {/* Quick Actions */}
              <Show when={hasQuickActions()}>
                <div class="mb-4">
                  <h3 class="text-xs font-semibold text-txt-muted uppercase tracking-wider mb-2">
                    Quick Actions
                  </h3>
                  <div class="grid grid-cols-2 gap-2">
                    <Show when={props.onNewProject}>
                      <button
                        class="flex items-center gap-2.5 p-3 bg-accent/10 hover:bg-accent/20 border border-accent/30 rounded-lg transition-all duration-200 group"
                        onClick={() => {
                          props.onClose();
                          props.onNewProject?.();
                        }}
                      >
                        <div class="p-1.5 bg-accent/20 rounded-md text-accent group-hover:scale-110 transition-transform">
                          <HiOutlinePlusCircle class="w-5 h-5" />
                        </div>
                        <div class="text-left">
                          <div class="font-medium text-txt">New Project</div>
                          <div class="text-xs text-txt-muted">
                            Start a new case
                          </div>
                        </div>
                      </button>
                    </Show>
                    <Show when={props.onOpenProject}>
                      <button
                        class="flex items-center gap-2.5 p-3 bg-bg-secondary/50 hover:bg-bg-hover border border-border/50 rounded-lg transition-all duration-200 group"
                        onClick={() => {
                          props.onClose();
                          props.onOpenProject?.();
                        }}
                      >
                        <div class="p-1.5 bg-bg-hover rounded-md text-txt-secondary group-hover:scale-110 transition-transform">
                          <HiOutlineFolderOpen class="w-5 h-5" />
                        </div>
                        <div class="text-left">
                          <div class="font-medium text-txt">Open Project</div>
                          <div class="text-xs text-txt-muted">
                            Open existing case
                          </div>
                        </div>
                      </button>
                    </Show>
                  </div>
                </div>
              </Show>

              {/* Recent Projects */}
              <Show when={hasRecentProjects()}>
                <div class="mb-4">
                  <h3 class="text-xs font-semibold text-txt-muted uppercase tracking-wider mb-2 flex items-center gap-2">
                    <HiOutlineClock class="w-4 h-4" />
                    Recent Projects
                  </h3>
                  <div class="space-y-2 max-h-[140px] overflow-y-auto">
                    <For each={props.recentProjects?.().slice(0, 5)}>
                      {(project) => (
                        <button
                          class="w-full flex items-center gap-3 p-3 bg-bg-secondary/30 hover:bg-bg-hover border border-border/30 rounded-lg transition-all duration-200 text-left group"
                          onClick={() => {
                            props.onClose();
                            props.onSelectRecentProject?.(project.path);
                          }}
                        >
                          <HiOutlineDocumentDuplicate class="w-5 h-5 text-txt-muted group-hover:text-accent transition-colors flex-shrink-0" />
                          <div class="min-w-0 flex-1">
                            <div class="font-medium text-txt text-sm truncate">
                              {project.name}
                            </div>
                            <div class="text-xs text-txt-muted truncate">
                              {project.path}
                            </div>
                          </div>
                          <div class="text-xs text-txt-muted flex-shrink-0">
                            {formatRelativeTime(project.lastOpened)}
                          </div>
                        </button>
                      )}
                    </For>
                  </div>
                </div>
              </Show>

              {/* Features preview */}
              <div class="mb-4">
                <h3 class="text-xs font-semibold text-txt-muted uppercase tracking-wider mb-2">
                  Features
                </h3>
                <div class="grid grid-cols-2 gap-2">
                  <div class="flex items-center gap-2.5 p-2.5 bg-bg-secondary/50 rounded-md border border-border/30">
                    <div class="p-1.5 bg-type-e01/10 rounded-md text-type-e01">
                      <HiOutlineFolder class="w-4 h-4" />
                    </div>
                    <div class="text-xs">
                      <div class="font-medium text-txt">Evidence Browser</div>
                      <div class="text-xs text-txt-muted">
                        E01, AD1, L01 support
                      </div>
                    </div>
                  </div>
                  <div class="flex items-center gap-2.5 p-2.5 bg-bg-secondary/50 rounded-md border border-border/30">
                    <div class="p-1.5 bg-success/10 rounded-md text-success">
                      <HiOutlineFingerPrint class="w-4 h-4" />
                    </div>
                    <div class="text-xs">
                      <div class="font-medium text-txt">
                        Hash Verification
                      </div>
                      <div class="text-xs text-txt-muted">
                        MD5, SHA1, SHA256
                      </div>
                    </div>
                  </div>
                  <div class="flex items-center gap-2.5 p-2.5 bg-bg-secondary/50 rounded-md border border-border/30">
                    <div class="p-1.5 bg-accent/10 rounded-md text-accent">
                      <HiOutlineChartBar class="w-4 h-4" />
                    </div>
                    <div class="text-xs">
                      <div class="font-medium text-txt">Hex Analysis</div>
                      <div class="text-xs text-txt-muted">
                        Binary inspection
                      </div>
                    </div>
                  </div>
                  <div class="flex items-center gap-2.5 p-2.5 bg-bg-secondary/50 rounded-md border border-border/30">
                    <div class="p-1.5 bg-warning/10 rounded-md text-warning">
                      <HiOutlineClipboardDocumentList class="w-4 h-4" />
                    </div>
                    <div class="text-xs">
                      <div class="font-medium text-txt">
                        Report Generation
                      </div>
                      <div class="text-xs text-txt-muted">PDF export</div>
                    </div>
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div class="flex gap-2">
                <button
                  class="flex-1 px-4 py-2.5 bg-bg-hover hover:bg-bg-active text-txt rounded-lg transition-all duration-200 text-sm font-medium"
                  onClick={props.onClose}
                >
                  Skip for now
                </button>
                <button
                  class="flex-1 px-4 py-2.5 bg-accent hover:bg-accent-hover text-white rounded-lg transition-all duration-200 font-medium text-sm hover:shadow-lg hover:shadow-accent/20"
                  onClick={() => {
                    props.onClose();
                    props.onStartTour();
                  }}
                >
                  🎯 Start Tour
                </button>
              </div>

              {/* Hint */}
              <p class="text-center text-xs text-txt-muted mt-4 flex items-center justify-center gap-1.5">
                Press <Kbd keys="?" muted /> anytime for help
              </p>
            </div>
          </div>
        </div>
      </Portal>
    </Show>
  );
}
