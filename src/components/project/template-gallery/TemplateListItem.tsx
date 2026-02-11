// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import { HiOutlineArrowDownTray } from "../../icons";
import type { TemplateSummary, TemplateCategory } from "../../../hooks/useProjectTemplates";

interface TemplateListItemProps {
  template: TemplateSummary;
  isBuiltin: boolean;
  onPreview: (template: TemplateSummary) => void;
  onApply: (templateId: string) => void;
  onExport: (templateId: string) => void;
  getCategoryColor: (category: TemplateCategory) => string;
  getCategoryLabel: (category: TemplateCategory) => string;
}

export const TemplateListItem: Component<TemplateListItemProps> = (props) => {
  return (
    <div class="bg-bg border border-border rounded-md p-4 hover:bg-bg-hover flex items-center gap-4">
      <div class="flex-1">
        <div class="flex items-center gap-2 mb-1">
          <h3 class="font-medium text-txt">{props.template.name}</h3>
          <Show when={props.isBuiltin}>
            <span class="px-2 py-0.5 bg-accent/20 text-accent text-xs rounded">
              Built-in
            </span>
          </Show>
          <span
            class={`px-2 py-0.5 text-xs rounded ${props.getCategoryColor(
              props.template.category
            )}`}
          >
            {props.getCategoryLabel(props.template.category)}
          </span>
        </div>
        <p class="text-sm text-txt-secondary">{props.template.description}</p>
        <div class="flex items-center gap-4 mt-2 text-xs text-txt-muted">
          <span class="flex items-center gap-1">
            Used {props.template.usage_count} times
          </span>
          <span class="flex items-center gap-1">
            {props.template.tags.length} tags
          </span>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <button
          onClick={() => props.onPreview(props.template)}
          class="px-3 py-1.5 bg-bg-secondary hover:bg-bg-hover text-txt text-sm rounded border border-border"
        >
          Preview
        </button>
        <button
          onClick={() => props.onApply(props.template.id)}
          class="px-3 py-1.5 bg-accent hover:bg-accent-hover text-white text-sm rounded"
        >
          Apply
        </button>
        <Show when={!props.isBuiltin}>
          <button
            onClick={() => props.onExport(props.template.id)}
            class="p-1.5 hover:bg-bg-hover text-txt-secondary hover:text-txt rounded"
          >
            <HiOutlineArrowDownTray class="w-icon-sm h-icon-sm" />
          </button>
        </Show>
      </div>
    </div>
  );
};
