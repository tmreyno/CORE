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
 * Usage:
 *   const form = useFormTemplate({ templateId: "evidence_collection" });
 *   <SchemaFormRenderer form={form} />
 */

import { createSignal, For, Index, Show, type Component } from "solid-js";
import {
  HiOutlineChevronDown,
  HiOutlineChevronUp,
  HiOutlinePlus,
  HiOutlineTrash,
  HiOutlineXMark,
} from "../components/icons";
import type { FieldSchema, SectionSchema, FormData, FormValue, InlineOption } from "./types";
import type { UseFormTemplateReturn } from "./useFormTemplate";

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

// =============================================================================
// SECTION RENDERER
// =============================================================================

interface DynamicSectionProps {
  section: SectionSchema;
  form: UseFormTemplateReturn;
  readOnly?: boolean;
}

const DynamicSection: Component<DynamicSectionProps> = (props) => {
  const [collapsed, setCollapsed] = createSignal(props.section.collapsed ?? false);
  const isCollapsible = props.section.collapsible ?? false;
  const isRepeatable = props.section.repeatable ?? false;

  return (
    <div class={isRepeatable ? "space-y-2" : "border border-border/50 rounded-lg p-3 space-y-2.5"}>
      {/* Section Header — skip for repeatable (RepeatableSection renders its own) */}
      <Show when={!isRepeatable}>
        <div
          class={`flex items-center gap-2 ${isCollapsible ? "cursor-pointer select-none" : ""}`}
          onClick={() => isCollapsible && setCollapsed((v) => !v)}
        >
          <Show when={props.section.icon}>
            <span class="text-base">{props.section.icon}</span>
          </Show>
          <h4 class="font-medium text-sm flex-1">{props.section.title}</h4>
          <Show when={isCollapsible}>
            <Show when={collapsed()} fallback={<HiOutlineChevronUp class="w-4 h-4 text-txt-muted" />}>
              <HiOutlineChevronDown class="w-4 h-4 text-txt-muted" />
            </Show>
          </Show>
        </div>

        <Show when={props.section.description}>
          <p class="text-xs text-txt/40 -mt-2">{props.section.description}</p>
        </Show>
      </Show>

      {/* Section Content */}
      <Show when={!collapsed()}>
        <Show
          when={props.section.repeatable}
          fallback={
            <FieldGrid section={props.section} form={props.form} readOnly={props.readOnly} />
          }
        >
          <RepeatableSection section={props.section} form={props.form} readOnly={props.readOnly} />
        </Show>
      </Show>
    </div>
  );
};

// =============================================================================
// FIELD GRID (non-repeatable section)
// =============================================================================

interface FieldGridProps {
  section: SectionSchema;
  form: UseFormTemplateReturn;
  readOnly?: boolean;
  /** For repeatable items — scoped item data */
  itemData?: FormData;
  /** For repeatable items — scoped setter */
  onItemFieldChange?: (fieldId: string, value: FormValue) => void;
}

const FieldGrid: Component<FieldGridProps> = (props) => {
  const cols = props.section.layout?.columns ?? 2;

  return (
    <div class="grid gap-x-3 gap-y-2" style={{ "grid-template-columns": `repeat(${cols}, minmax(0, 1fr))` }}>
      <For each={props.section.fields}>
        {(field) => (
          <Show when={props.form.isFieldVisible(field, props.itemData)}>
            <DynamicField
              field={field}
              form={props.form}
              readOnly={props.readOnly}
              value={props.itemData?.[field.id] ?? props.form.getValue(field.id)}
              onChange={(val) => {
                if (props.onItemFieldChange) {
                  props.onItemFieldChange(field.id, val as FormValue);
                } else {
                  props.form.setValue(field.id, val as FormValue);
                }
              }}
              itemData={props.itemData}
            />
          </Show>
        )}
      </For>
    </div>
  );
};

// =============================================================================
// REPEATABLE SECTION
// =============================================================================

interface RepeatableSectionProps {
  section: SectionSchema;
  form: UseFormTemplateReturn;
  readOnly?: boolean;
}

const RepeatableSection: Component<RepeatableSectionProps> = (props) => {
  const [expandedItems, setExpandedItems] = createSignal<Set<string>>(new Set());
  const config = props.section.repeatable_config;
  const itemLabel = config?.item_label ?? "Item";
  const isItemCollapsible = config?.collapsible !== false;

  const items = () => props.form.getRepeatableItems(props.section.id);

  const toggleItem = (id: string) => {
    setExpandedItems((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const addItem = () => {
    props.form.addRepeatableItem(props.section);
    // Auto-expand new item
    const all = props.form.getRepeatableItems(props.section.id);
    const last = all[all.length - 1];
    if (last?.id) {
      setExpandedItems((prev) => new Set(prev).add(last.id as string));
    }
  };

  /** Build summary text from summary_fields */
  const buildSummary = (item: FormData): string => {
    if (!config?.summary_fields?.length) return String(item.description ?? itemLabel);
    const parts: string[] = [];
    for (const fid of config.summary_fields) {
      const val = item[fid];
      if (!val) continue;
      // Try to resolve the display label from options
      const fieldDef = props.section.fields.find((f) => f.id === fid);
      if (fieldDef && (fieldDef.options_ref || fieldDef.options)) {
        const opts = props.form.getOptions(fieldDef);
        const match = opts.find((o) => o.value === val);
        if (match) {
          parts.push(match.label);
          continue;
        }
      }
      parts.push(String(val));
    }
    return parts.join(" — ") || itemLabel;
  };

  return (
    <div class="space-y-2">
      {/* Section heading bar with add button */}
      <div class="flex items-center justify-between">
        <h4 class="font-medium text-xs flex items-center gap-1.5">
          <Show when={props.section.icon}>
            <span class="text-sm">{props.section.icon}</span>
          </Show>
          {props.section.title}
          <span class="text-txt/50 font-normal">({items().length})</span>
        </h4>
        <Show when={!props.readOnly && (!config?.max_items || items().length < config.max_items)}>
          <button type="button" class="btn-sm flex items-center gap-1" onClick={addItem}>
            <HiOutlinePlus class="w-3.5 h-3.5" />
            Add {itemLabel}
          </button>
        </Show>
      </div>

      <Show
        when={items().length > 0}
        fallback={
          <div class="text-center py-5 text-txt/50 border border-dashed border-border rounded-lg">
            <p class="text-sm mb-0.5">No {itemLabel.toLowerCase()}s added</p>
            <p class="text-xs">Click "Add {itemLabel}" to begin.</p>
          </div>
        }
      >
        <div class="space-y-2">
          {/* Use <Index> instead of <For> so that DOM nodes are stable
              per-index and item data updates reactively via the signal.
              <For> would re-create DOM when the item reference changes
              (every keystroke), causing inputs to lose focus. */}
          <Index each={items()}>
            {(item, idx) => {
              // item is an Accessor<FormData> — read via item()
              const itemId = () => item().id as string;
              const isExpanded = () => expandedItems().has(itemId());
              const itemNumber = () => item()[config?.number_field ?? "item_number"] ?? `#${idx + 1}`;

              return (
                <div class="border border-border/50 rounded-lg bg-surface/50 overflow-hidden">
                  {/* Item Header */}
                  <button
                    type="button"
                    class="w-full flex items-center justify-between px-3 py-2 text-left hover:bg-bg-hover/50 transition-colors"
                    onClick={() => isItemCollapsible && toggleItem(itemId())}
                  >
                    <div class="flex items-center gap-2 min-w-0">
                      <span class="font-mono text-xs font-medium text-accent shrink-0">
                        #{String(itemNumber())}
                      </span>
                      <span class="text-xs text-txt/70 truncate">{buildSummary(item())}</span>
                    </div>
                    <div class="flex items-center gap-2 shrink-0">
                      <Show when={!props.readOnly}>
                        <button
                          type="button"
                          class="icon-btn-sm text-txt-muted hover:text-error"
                          onClick={(e) => {
                            e.stopPropagation();
                            props.form.removeRepeatableItem(props.section.id, itemId());
                          }}
                          title={`Remove ${itemLabel}`}
                        >
                          <HiOutlineTrash class="w-4 h-4" />
                        </button>
                      </Show>
                      <Show when={isItemCollapsible}>
                        <Show when={isExpanded()} fallback={<HiOutlineChevronDown class="w-4 h-4 text-txt-muted" />}>
                          <HiOutlineChevronUp class="w-4 h-4 text-txt-muted" />
                        </Show>
                      </Show>
                    </div>
                  </button>

                  {/* Item Fields */}
                  <Show when={isExpanded()}>
                    <div class="px-3 pb-3 pt-1.5 border-t border-border/30">
                      <FieldGrid
                        section={props.section}
                        form={props.form}
                        readOnly={props.readOnly}
                        itemData={item()}
                        onItemFieldChange={(fieldId, value) => {
                          props.form.setRepeatableItemValue(
                            props.section.id,
                            itemId(),
                            fieldId,
                            value
                          );
                        }}
                      />
                    </div>
                  </Show>
                </div>
              );
            }}
          </Index>
        </div>
      </Show>
    </div>
  );
};

// =============================================================================
// DYNAMIC FIELD RENDERER
// =============================================================================

interface DynamicFieldProps {
  field: FieldSchema;
  form: UseFormTemplateReturn;
  readOnly?: boolean;
  value: unknown;
  onChange: (value: unknown) => void;
  /** For repeatable items — scoped item data (passed to getOptions for filtering) */
  itemData?: FormData;
}

const DynamicField: Component<DynamicFieldProps> = (props) => {
  const field = props.field;
  const colSpan = field.col_span ? `col-span-${field.col_span}` : "";
  const spanStyle = field.col_span ? { "grid-column": `span ${field.col_span}` } : {};

  // --- Display-only types ---

  if (field.type === "heading") {
    return (
      <div class={colSpan} style={spanStyle}>
        <div class="flex items-center gap-2 mt-2 mb-1">
          <Show when={field.icon}><span class="text-base">{field.icon}</span></Show>
          <h4 class="text-sm font-medium text-txt/70">{field.label}</h4>
        </div>
      </div>
    );
  }

  if (field.type === "separator") {
    return <div class={`border-t border-border/30 my-2 ${colSpan}`} style={spanStyle} />;
  }

  if (field.type === "static") {
    return (
      <div class={colSpan} style={spanStyle}>
        <p class="text-sm text-txt-muted">{field.content ?? field.label}</p>
      </div>
    );
  }

  // --- Input types ---

  const val = () => props.value;
  const readOnly = () => props.readOnly ?? false;
  const isRequired = field.validation?.required ?? false;
  const inputClass = field.input_class ?? "";

  return (
    <div class={`form-group ${colSpan}`} style={spanStyle}>
      <label class="label">
        {field.label}
        <Show when={isRequired}><span class="text-error ml-0.5">*</span></Show>
      </label>

      {/* Switch on field type */}
      {(() => {
        switch (field.type) {
          case "text":
          case "email":
          case "phone":
          case "url":
          case "file":
            return (
              <input
                type={field.type === "phone" ? "tel" : field.type === "file" ? "text" : field.type}
                class={`input-sm ${inputClass}`}
                value={String(val() ?? "")}
                onInput={(e) => props.onChange(e.currentTarget.value || undefined)}
                placeholder={field.placeholder}
                readOnly={readOnly()}
              />
            );

          case "number":
            return (
              <input
                type="number"
                class={`input-sm ${inputClass}`}
                value={val() != null ? Number(val()) : ""}
                onInput={(e) => {
                  const v = e.currentTarget.value;
                  props.onChange(v ? Number(v) : undefined);
                }}
                placeholder={field.placeholder}
                readOnly={readOnly()}
                min={field.validation?.min}
                max={field.validation?.max}
              />
            );

          case "textarea":
            return (
              <textarea
                class={`textarea text-sm ${inputClass}`}
                value={String(val() ?? "")}
                onInput={(e) => props.onChange(e.currentTarget.value || undefined)}
                placeholder={field.placeholder}
                readOnly={readOnly()}
                rows={field.rows ?? 3}
              />
            );

          case "date":
            return (
              <input
                type="date"
                class={`input-sm ${inputClass}`}
                value={String(val() ?? "")}
                onInput={(e) => props.onChange(e.currentTarget.value || undefined)}
                readOnly={readOnly()}
              />
            );

          case "datetime":
            return (
              <input
                type="datetime-local"
                class={`input-sm ${inputClass}`}
                value={String(val() ?? "")}
                onInput={(e) => props.onChange(e.currentTarget.value || undefined)}
                readOnly={readOnly()}
              />
            );

          case "time":
            return (
              <input
                type="time"
                class={`input-sm ${inputClass}`}
                value={String(val() ?? "")}
                onInput={(e) => props.onChange(e.currentTarget.value || undefined)}
                readOnly={readOnly()}
              />
            );

          case "select":
            return (
              <SelectField
                field={field}
                value={String(val() ?? "")}
                options={props.form.getOptions(field, props.itemData)}
                onChange={props.onChange}
                readOnly={readOnly()}
              />
            );

          case "multiselect":
            return (
              <MultiSelectField
                field={field}
                value={(val() as string[]) ?? []}
                options={props.form.getOptions(field, props.itemData)}
                onChange={props.onChange}
                readOnly={readOnly()}
              />
            );

          case "radio":
            return (
              <RadioField
                field={field}
                value={String(val() ?? "")}
                options={props.form.getOptions(field, props.itemData)}
                onChange={props.onChange}
                readOnly={readOnly()}
              />
            );

          case "checkbox":
            return (
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={!!val()}
                  onChange={(e) => props.onChange(e.currentTarget.checked)}
                  disabled={readOnly()}
                  class="rounded border-border"
                />
                <span class="text-sm text-txt">{field.help ?? field.label}</span>
              </label>
            );

          case "comma_list":
            return (
              <CommaListField
                field={field}
                value={(val() as string[]) ?? []}
                onChange={props.onChange}
                readOnly={readOnly()}
              />
            );

          case "hidden":
            return null;

          default:
            return (
              <input
                type="text"
                class="input-sm"
                value={String(val() ?? "")}
                onInput={(e) => props.onChange(e.currentTarget.value || undefined)}
                placeholder={field.placeholder}
                readOnly={readOnly()}
              />
            );
        }
      })()}

      <Show when={field.help && field.type !== "checkbox"}>
        <p class="text-xs text-txt/40 mt-0.5">{field.help}</p>
      </Show>
    </div>
  );
};

// =============================================================================
// SUB-FIELD COMPONENTS
// =============================================================================

/** Select dropdown */
const SelectField: Component<{
  field: FieldSchema;
  value: string;
  options: InlineOption[];
  onChange: (value: unknown) => void;
  readOnly: boolean;
}> = (props) => (
  <select
    class={`input-sm ${props.field.input_class ?? ""}`}
    value={props.value}
    onChange={(e) => props.onChange(e.currentTarget.value || undefined)}
    disabled={props.readOnly}
  >
    <Show when={props.field.placeholder}>
      <option value="">{props.field.placeholder}</option>
    </Show>
    <For each={props.options}>
      {(opt) => (
        <option
          value={opt.disabled ? "" : opt.value}
          disabled={opt.disabled}
          class={opt.disabled ? "text-txt-muted font-semibold bg-bg-secondary" : ""}
        >
          {opt.label}
        </option>
      )}
    </For>
  </select>
);

/** Multi-select as chips/tags */
const MultiSelectField: Component<{
  field: FieldSchema;
  value: string[];
  options: InlineOption[];
  onChange: (value: unknown) => void;
  readOnly: boolean;
}> = (props) => {
  const available = () => props.options.filter((o) => !props.value.includes(o.value));

  const add = (val: string) => {
    if (val && !props.value.includes(val)) {
      props.onChange([...props.value, val]);
    }
  };

  const remove = (val: string) => {
    props.onChange(props.value.filter((v) => v !== val));
  };

  return (
    <div class="space-y-2">
      <Show when={!props.readOnly && available().length > 0}>
        <select
          class="input-sm"
          onChange={(e) => {
            add(e.currentTarget.value);
            e.currentTarget.value = "";
          }}
        >
          <option value="">Add {props.field.label}...</option>
          <For each={available()}>
            {(opt) => <option value={opt.value}>{opt.label}</option>}
          </For>
        </select>
      </Show>
      <div class="flex flex-wrap gap-2">
        <For each={props.value}>
          {(val) => {
            const opt = props.options.find((o) => o.value === val);
            return (
              <span class="inline-flex items-center gap-1 px-2 py-0.5 bg-accent/10 text-accent rounded-full text-xs font-medium">
                {opt?.label ?? val}
                <Show when={!props.readOnly}>
                  <button
                    class="w-3.5 h-3.5 rounded-full bg-accent/20 hover:bg-accent/30 flex items-center justify-center"
                    onClick={() => remove(val)}
                  >
                    <HiOutlineXMark class="w-2.5 h-2.5" />
                  </button>
                </Show>
              </span>
            );
          }}
        </For>
      </div>
    </div>
  );
};

/** Radio button group */
const RadioField: Component<{
  field: FieldSchema;
  value: string;
  options: InlineOption[];
  onChange: (value: unknown) => void;
  readOnly: boolean;
}> = (props) => (
  <div class="flex flex-wrap gap-3">
    <For each={props.options}>
      {(opt) => (
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="radio"
            name={props.field.id}
            value={opt.value}
            checked={props.value === opt.value}
            onChange={() => props.onChange(opt.value)}
            disabled={props.readOnly}
            class="border-border"
          />
          <span class="text-sm">{opt.label}</span>
        </label>
      )}
    </For>
  </div>
);

/** Comma-separated list rendered as tags with an input */
const CommaListField: Component<{
  field: FieldSchema;
  value: string[];
  onChange: (value: unknown) => void;
  readOnly: boolean;
}> = (props) => {
  const [input, setInput] = createSignal("");

  const addValue = () => {
    const trimmed = input().trim();
    if (trimmed && !props.value.includes(trimmed)) {
      props.onChange([...props.value, trimmed]);
      setInput("");
    }
  };

  const removeValue = (val: string) => {
    props.onChange(props.value.filter((v) => v !== val));
  };

  return (
    <div class="space-y-2">
      <Show when={!props.readOnly}>
        <div class="flex gap-2">
          <input
            type="text"
            class="input-sm flex-1"
            value={input()}
            onInput={(e) => setInput(e.currentTarget.value)}
            onKeyDown={(e) => e.key === "Enter" && (e.preventDefault(), addValue())}
            placeholder={props.field.placeholder}
          />
          <button
            class="px-2.5 py-1.5 rounded-md text-xs font-medium bg-accent/10 text-accent hover:bg-accent/20 transition-colors"
            onClick={addValue}
          >
            + Add
          </button>
        </div>
      </Show>
      <div class="flex flex-wrap gap-2">
        <For each={props.value}>
          {(val) => (
            <span class="inline-flex items-center gap-1 px-2 py-0.5 bg-accent/10 text-accent rounded-full text-xs font-medium">
              {val}
              <Show when={!props.readOnly}>
                <button
                  class="w-3.5 h-3.5 rounded-full bg-accent/20 hover:bg-accent/30 flex items-center justify-center"
                  onClick={() => removeValue(val)}
                >
                  <HiOutlineXMark class="w-2.5 h-2.5" />
                </button>
              </Show>
            </span>
          )}
        </For>
        <Show when={props.value.length === 0}>
          <span class="text-sm text-txt/40 italic">None added</span>
        </Show>
      </div>
    </div>
  );
};

// =============================================================================
// LOADING / ERROR STATES
// =============================================================================

const FormSkeleton: Component = () => (
  <div class="space-y-3 animate-pulse">
    <div class="h-5 w-40 bg-bg-hover rounded" />
    <div class="grid grid-cols-2 gap-3">
      <div class="h-8 bg-bg-hover rounded" />
      <div class="h-8 bg-bg-hover rounded" />
      <div class="h-8 bg-bg-hover rounded" />
      <div class="h-8 bg-bg-hover rounded" />
    </div>
  </div>
);

const FormError: Component<{ message?: string }> = (props) => (
  <div class="p-4 rounded-lg bg-error/10 text-error text-sm">
    Failed to load form template{props.message ? `: ${props.message}` : "."}
  </div>
);

export default SchemaFormRenderer;
