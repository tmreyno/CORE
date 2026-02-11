// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * @fileoverview Project Template Selector Component
 * 
 * Displays available project templates organized by category
 * for quick project initialization with pre-configured settings.
 */

import { Component, createSignal, createResource, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineDevicePhoneMobile,
  HiOutlineComputerDesktop,
  HiOutlineGlobeAlt,
  HiOutlineCloud,
  HiOutlineShieldExclamation,
  HiOutlineCpuChip,
  HiOutlineExclamationTriangle,
  HiOutlineDocumentMagnifyingGlass,
  HiOutlineFolder,
  HiOutlineSparkles,
  HiOutlineCheckCircle,
  HiOutlineTag,
} from "./icons";
import { logger } from "../utils/logger";
const log = logger.scope("ProjectTemplateSelector");

// ============================================================================

/** Template category enum (matches Rust) */
export type TemplateCategory = 
  | "Mobile"
  | "Computer"
  | "Network"
  | "Cloud"
  | "IncidentResponse"
  | "Memory"
  | "Malware"
  | "EDiscovery"
  | "General"
  | "Custom";

/** Template summary from backend */
export interface TemplateSummary {
  id: string;
  name: string;
  category: TemplateCategory;
  description: string;
  tags: string[];
  usage_count: number;
}

// ============================================================================
// Category Icons & Colors
// ============================================================================

const categoryConfig: Record<TemplateCategory, { icon: typeof HiOutlineFolder; color: string; label: string }> = {
  Mobile: { icon: HiOutlineDevicePhoneMobile, color: "text-type-ufed", label: "Mobile Forensics" },
  Computer: { icon: HiOutlineComputerDesktop, color: "text-type-e01", label: "Computer Forensics" },
  Network: { icon: HiOutlineGlobeAlt, color: "text-accent", label: "Network Forensics" },
  Cloud: { icon: HiOutlineCloud, color: "text-info", label: "Cloud Forensics" },
  IncidentResponse: { icon: HiOutlineShieldExclamation, color: "text-warning", label: "Incident Response" },
  Memory: { icon: HiOutlineCpuChip, color: "text-type-raw", label: "Memory Analysis" },
  Malware: { icon: HiOutlineExclamationTriangle, color: "text-error", label: "Malware Analysis" },
  EDiscovery: { icon: HiOutlineDocumentMagnifyingGlass, color: "text-success", label: "E-Discovery" },
  General: { icon: HiOutlineFolder, color: "text-txt-secondary", label: "General Purpose" },
  Custom: { icon: HiOutlineSparkles, color: "text-accent", label: "Custom" },
};

// ============================================================================
// Component
// ============================================================================

export interface ProjectTemplateSelectorProps {
  /** Called when a template is selected */
  onSelect: (templateId: string | null) => void;
  /** Currently selected template ID */
  selectedId?: string | null;
  /** Compact display mode */
  compact?: boolean;
  /** Class name */
  class?: string;
}

/**
 * Fetches available templates from backend
 */
async function fetchTemplates(): Promise<TemplateSummary[]> {
  try {
    return await invoke<TemplateSummary[]>("template_list");
  } catch (err) {
    log.warn("Failed to load templates:", err);
    return [];
  }
}

/**
 * Project Template Selector - displays templates by category
 */
export const ProjectTemplateSelector: Component<ProjectTemplateSelectorProps> = (props) => {
  
  // Fetch templates
  const [templates] = createResource(fetchTemplates);
  
  // Selected category filter
  const [selectedCategory, setSelectedCategory] = createSignal<TemplateCategory | null>(null);
  
  // Filter templates by category
  const filteredTemplates = () => {
    const all = templates() ?? [];
    const cat = selectedCategory();
    return cat ? all.filter(t => t.category === cat) : all;
  };
  
  // Get unique categories from templates
  const categories = () => {
    const all = templates() ?? [];
    const cats = new Set(all.map(t => t.category));
    return Array.from(cats).sort();
  };

  const getCategoryConfig = (cat: TemplateCategory) => 
    categoryConfig[cat] || categoryConfig.General;

  return (
    <div class={`${props.class ?? ""}`}>
      {/* Category Filter Chips */}
      <div class="flex flex-wrap gap-2 mb-4">
        <button
          class={`px-3 py-1.5 rounded-full text-sm font-medium transition-colors ${
            selectedCategory() === null
              ? "bg-accent text-white"
              : "bg-bg-secondary text-txt-secondary hover:bg-bg-hover"
          }`}
          onClick={() => setSelectedCategory(null)}
        >
          All Templates
        </button>
        <For each={categories()}>
          {(cat) => {
            const config = getCategoryConfig(cat);
            return (
              <button
                class={`flex items-center gap-1.5 px-3 py-1.5 rounded-full text-sm font-medium transition-colors ${
                  selectedCategory() === cat
                    ? "bg-accent text-white"
                    : "bg-bg-secondary text-txt-secondary hover:bg-bg-hover"
                }`}
                onClick={() => setSelectedCategory(cat)}
              >
                <config.icon class="w-4 h-4" />
                {config.label}
              </button>
            );
          }}
        </For>
      </div>

      {/* Loading State */}
      <Show when={templates.loading}>
        <div class="flex items-center justify-center py-8 text-txt-muted">
          <div class="animate-spin mr-2">⏳</div>
          Loading templates...
        </div>
      </Show>

      {/* No Project / Skip Option */}
      <button
        class={`w-full mb-3 p-4 rounded-xl border-2 transition-all ${
          props.selectedId === null
            ? "border-accent bg-accent/10"
            : "border-border hover:border-border-active bg-bg-secondary/50 hover:bg-bg-hover"
        }`}
        onClick={() => props.onSelect(null)}
      >
        <div class="flex items-center gap-3">
          <div class={`p-2 rounded-lg ${props.selectedId === null ? "bg-accent/20 text-accent" : "bg-bg-hover text-txt-muted"}`}>
            <HiOutlineFolder class="w-6 h-6" />
          </div>
          <div class="text-left flex-1">
            <div class="font-medium text-txt">Start from Scratch</div>
            <div class="text-sm text-txt-muted">Create an empty project without a template</div>
          </div>
          <Show when={props.selectedId === null}>
            <HiOutlineCheckCircle class="w-5 h-5 text-accent" />
          </Show>
        </div>
      </button>

      {/* Template Grid */}
      <Show when={!templates.loading && filteredTemplates().length > 0}>
        <div class={props.compact ? "space-y-2" : "grid gap-3 grid-cols-1 md:grid-cols-2"}>
          <For each={filteredTemplates()}>
            {(template) => {
              const config = getCategoryConfig(template.category);
              const isSelected = () => props.selectedId === template.id;
              
              return (
                <button
                  class={`w-full p-4 rounded-xl border-2 transition-all text-left ${
                    isSelected()
                      ? "border-accent bg-accent/10"
                      : "border-border hover:border-border-active bg-bg-secondary/50 hover:bg-bg-hover"
                  }`}
                  onClick={() => props.onSelect(template.id)}
                >
                  <div class="flex items-start gap-3">
                    <div class={`p-2 rounded-lg ${isSelected() ? "bg-accent/20 text-accent" : `bg-bg-hover ${config.color}`}`}>
                      <config.icon class="w-6 h-6" />
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center justify-between gap-2">
                        <span class="font-medium text-txt truncate">{template.name}</span>
                        <Show when={isSelected()}>
                          <HiOutlineCheckCircle class="w-5 h-5 text-accent flex-shrink-0" />
                        </Show>
                      </div>
                      <p class="text-sm text-txt-muted mt-1 line-clamp-2">{template.description}</p>
                      
                      {/* Tags */}
                      <Show when={template.tags.length > 0 && !props.compact}>
                        <div class="flex flex-wrap gap-1 mt-2">
                          <For each={template.tags.slice(0, 3)}>
                            {(tag) => (
                              <span class="inline-flex items-center gap-1 px-2 py-0.5 bg-bg-hover rounded text-xs text-txt-muted">
                                <HiOutlineTag class="w-3 h-3" />
                                {tag}
                              </span>
                            )}
                          </For>
                          <Show when={template.tags.length > 3}>
                            <span class="px-2 py-0.5 text-xs text-txt-muted">
                              +{template.tags.length - 3}
                            </span>
                          </Show>
                        </div>
                      </Show>
                    </div>
                  </div>
                </button>
              );
            }}
          </For>
        </div>
      </Show>

      {/* Empty State */}
      <Show when={!templates.loading && filteredTemplates().length === 0 && selectedCategory() !== null}>
        <div class="text-center py-8 text-txt-muted">
          No templates found in this category.
        </div>
      </Show>
    </div>
  );
};

export default ProjectTemplateSelector;
