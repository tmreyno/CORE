// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * SchemaFormRenderer — Generic SolidJS component that renders any FormTemplate.
 *
 * Renders sections (with optional collapsible/repeatable support) and fields
 * (text, select, date, checkbox, textarea, comma_list, etc.) driven entirely
 * by JSON schema. No form-specific code needed.
 *
 * Sub-components are organized into:
 *   - `templates/sections/` — DynamicSection, FieldGrid, RepeatableSection, DynamicField
 *   - `templates/fields/`   — SelectField, MultiSelectField, RadioField, CommaListField
 *
 * Usage:
 *   const form = useFormTemplate({ templateId: "evidence_collection" });
 *   <SchemaFormRenderer form={form} />
 */

import { For, Show, type Component } from "solid-js";
import type { UseFormTemplateReturn } from "./useFormTemplate";
import { DynamicSection, FormSkeleton, FormError } from "./sections";

// =============================================================================
// MAIN COMPONENT
// =============================================================================

export interface SchemaFormRendererProps {
  /** The form hook instance (from useFormTemplate) */
  form: UseFormTemplateReturn;
  /** Optional: make all fields read-only */
  readOnly?: boolean;
  /** Optional: extra CSS class on the root container */
  class?: string;
}

export const SchemaFormRenderer: Component<SchemaFormRendererProps> = (props) => {
  return (
    <Show when={!props.form.loading()} fallback={<FormSkeleton />}>
      <Show when={props.form.template()} fallback={<FormError message={props.form.error()} />}>
        {(template) => (
          <div class={`space-y-3 ${props.class ?? ""}`}>
            {/* Form Header */}
            <div class="flex items-center gap-2">
              <div class="w-7 h-7 rounded-lg bg-accent/10 flex items-center justify-center">
                <span class="text-sm">{template().icon ?? "📋"}</span>
              </div>
              <div>
                <h3 class="text-sm font-semibold">{template().name}</h3>
                <Show when={template().description}>
                  <p class="text-xs text-txt/50">{template().description}</p>
                </Show>
              </div>
            </div>

            <For each={template().sections}>
              {(section) => (
                <Show when={props.form.isSectionVisible(section)}>
                  <DynamicSection
                    section={section}
                    form={props.form}
                    readOnly={props.readOnly}
                  />
                </Show>
              )}
            </For>
          </div>
        )}
      </Show>
    </Show>
  );
};

export default SchemaFormRenderer;
