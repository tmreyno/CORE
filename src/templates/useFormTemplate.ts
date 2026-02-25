// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useFormTemplate — Hook for loading JSON form templates and managing form state.
 *
 * Responsibilities:
 *  1. Load a FormTemplate from templates/forms/{id}.json
 *  2. Load referenced OptionRegistry files from templates/options/{ref}.json
 *  3. Provide reactive FormData state with get/set helpers
 *  4. Evaluate show_when conditions for conditional visibility
 *  5. Generate auto-filled default values
 *  6. Persist form data to the .ffxdb via form_submissions table
 */

import { createSignal, createMemo, createResource, type Accessor } from "solid-js";
import type {
  FormTemplate,
  FormData,
  FormValue,
  OptionRegistry,
  InlineOption,
  FieldCondition,
  FieldSchema,
  SectionSchema,
} from "./types";

// =============================================================================
// TEMPLATE & REGISTRY LOADING
// =============================================================================

/** Cache for loaded option registries (avoids re-fetching) */
const registryCache = new Map<string, OptionRegistry>();

/** Load a form template JSON file by ID */
export async function loadFormTemplate(templateId: string): Promise<FormTemplate> {
  const mod = await import(`./forms/${templateId}.json`);
  return mod.default as FormTemplate;
}

/** Load an option registry JSON file by ID */
export async function loadOptionRegistry(registryId: string): Promise<OptionRegistry> {
  const cached = registryCache.get(registryId);
  if (cached) return cached;
  const mod = await import(`./options/${registryId}.json`);
  const registry = mod.default as OptionRegistry;
  registryCache.set(registryId, registry);
  return registry;
}

/** Collect all unique options_ref values from a template */
function collectRegistryRefs(template: FormTemplate): string[] {
  const refs = new Set<string>();
  for (const section of template.sections) {
    for (const field of section.fields) {
      if (field.options_ref) refs.add(field.options_ref);
    }
  }
  return [...refs];
}

/** Load all option registries referenced by a template */
async function loadAllRegistries(template: FormTemplate): Promise<Map<string, OptionRegistry>> {
  const refs = collectRegistryRefs(template);
  const entries = await Promise.all(
    refs.map(async (ref) => [ref, await loadOptionRegistry(ref)] as const)
  );
  return new Map(entries);
}

// =============================================================================
// CONDITION EVALUATION
// =============================================================================

/** Evaluate a FieldCondition against the current form data */
export function evaluateCondition(
  condition: FieldCondition,
  data: FormData,
  /** For repeatable item context — item-level data takes precedence */
  itemData?: FormData,
): boolean {
  // Resolve the field value — check item data first, then parent form data
  const raw = itemData?.[condition.field] ?? data[condition.field];

  switch (condition.op) {
    case "eq":
      return raw === condition.value;
    case "neq":
      return raw !== condition.value;
    case "in":
      if (Array.isArray(condition.value)) {
        return condition.value.includes(raw as string);
      }
      return false;
    case "not_in":
      if (Array.isArray(condition.value)) {
        return !condition.value.includes(raw as string);
      }
      return true;
    case "truthy":
      return !!raw;
    case "falsy":
      return !raw;
    default:
      return true;
  }
}

// =============================================================================
// DEFAULT VALUES
// =============================================================================

/** Build initial FormData from template defaults */
export function buildDefaults(template: FormTemplate): FormData {
  const data: FormData = {};
  for (const section of template.sections) {
    if (section.repeatable) {
      // Repeatable sections start with an empty array
      data[section.id] = [];
      continue;
    }
    for (const field of section.fields) {
      if (field.default !== undefined) {
        data[field.id] = field.default;
      }
    }
  }
  return data;
}

/** Create a new blank item for a repeatable section using field defaults */
export function createRepeatableItem(section: SectionSchema): FormData {
  const item: FormData = { id: crypto.randomUUID() };
  for (const field of section.fields) {
    if (field.type === "heading" || field.type === "separator" || field.type === "static") continue;
    if (field.default !== undefined) {
      item[field.id] = field.default;
    }
  }
  return item;
}

// =============================================================================
// HOOK: useFormTemplate
// =============================================================================

export interface UseFormTemplateOptions {
  /** Template ID to load */
  templateId: string;
  /** Optional initial data (overrides defaults) */
  initialData?: FormData;
  /** Context values injected into condition evaluation (e.g., _report_type) */
  context?: Record<string, FormValue>;
}

export interface UseFormTemplateReturn {
  /** The loaded template (undefined while loading) */
  template: Accessor<FormTemplate | undefined>;
  /** Whether the template is still loading */
  loading: Accessor<boolean>;
  /** Load error, if any */
  error: Accessor<string | undefined>;
  /** All loaded option registries (keyed by registry ID) */
  registries: Accessor<Map<string, OptionRegistry>>;
  /** Current form data */
  data: Accessor<FormData>;
  /** Set entire form data */
  setData: (data: FormData) => void;
  /** Get a single field value */
  getValue: (fieldId: string) => FormValue;
  /** Set a single field value */
  setValue: (fieldId: string, value: FormValue) => void;
  /** Get options for a field (from registry or inline) */
  getOptions: (field: FieldSchema) => InlineOption[];
  /** Check if a field should be visible (evaluates show_when) */
  isFieldVisible: (field: FieldSchema, itemData?: FormData) => boolean;
  /** Check if a section should be visible */
  isSectionVisible: (section: SectionSchema) => boolean;
  /** Get repeatable items for a section */
  getRepeatableItems: (sectionId: string) => FormData[];
  /** Add a new item to a repeatable section */
  addRepeatableItem: (section: SectionSchema) => void;
  /** Remove an item from a repeatable section */
  removeRepeatableItem: (sectionId: string, itemId: string) => void;
  /** Update a field within a repeatable item */
  setRepeatableItemValue: (sectionId: string, itemId: string, fieldId: string, value: FormValue) => void;
  /** Merge external data into form data (for auto-fill, project loading, etc.) */
  mergeData: (patch: Partial<FormData>) => void;
}

export function useFormTemplate(options: UseFormTemplateOptions): UseFormTemplateReturn {
  const [formData, setFormData] = createSignal<FormData>(options.initialData ?? {});

  // Load template + registries
  const [templateResource] = createResource(
    () => options.templateId,
    async (id) => {
      const tmpl = await loadFormTemplate(id);
      const regs = await loadAllRegistries(tmpl);
      return { template: tmpl, registries: regs };
    }
  );

  // Once template loads, seed defaults if no initial data was provided
  const templateReady = createMemo(() => {
    const res = templateResource();
    if (res && !options.initialData) {
      const defaults = buildDefaults(res.template);
      // Only set defaults for fields not already in formData
      setFormData((prev) => {
        const merged = { ...defaults };
        for (const [key, val] of Object.entries(prev)) {
          if (val !== undefined) merged[key] = val;
        }
        return merged;
      });
    }
    return res;
  });

  const template: Accessor<FormTemplate | undefined> = () => templateReady()?.template;
  const loading: Accessor<boolean> = () => templateResource.loading;
  const error: Accessor<string | undefined> = () =>
    templateResource.error ? String(templateResource.error) : undefined;
  const registries: Accessor<Map<string, OptionRegistry>> = () =>
    templateReady()?.registries ?? new Map();

  // --- Merged data = formData + context (for condition evaluation) ---
  const mergedData = createMemo<FormData>(() => ({
    ...formData(),
    ...(options.context ?? {}),
  }));

  // --- Field/section API ---

  const getValue = (fieldId: string): FormValue => formData()[fieldId];

  const setValue = (fieldId: string, value: FormValue) => {
    setFormData((prev) => ({ ...prev, [fieldId]: value }));
  };

  const getOptions = (field: FieldSchema): InlineOption[] => {
    if (field.options) return field.options;
    if (field.options_ref) {
      const reg = registries().get(field.options_ref);
      return reg?.items ?? [];
    }
    return [];
  };

  const isFieldVisible = (field: FieldSchema, itemData?: FormData): boolean => {
    if (!field.show_when) return true;
    return evaluateCondition(field.show_when, mergedData(), itemData);
  };

  const isSectionVisible = (section: SectionSchema): boolean => {
    if (!section.show_when) return true;
    return evaluateCondition(section.show_when, mergedData());
  };

  // --- Repeatable section API ---

  const getRepeatableItems = (sectionId: string): FormData[] => {
    const val = formData()[sectionId];
    return Array.isArray(val) ? (val as FormData[]) : [];
  };

  const addRepeatableItem = (section: SectionSchema) => {
    const newItem = createRepeatableItem(section);
    setFormData((prev) => ({
      ...prev,
      [section.id]: [...(Array.isArray(prev[section.id]) ? (prev[section.id] as FormData[]) : []), newItem],
    }));
  };

  const removeRepeatableItem = (sectionId: string, itemId: string) => {
    setFormData((prev) => ({
      ...prev,
      [sectionId]: (prev[sectionId] as FormData[]).filter((item) => item.id !== itemId),
    }));
  };

  const setRepeatableItemValue = (
    sectionId: string,
    itemId: string,
    fieldId: string,
    value: FormValue,
  ) => {
    setFormData((prev) => ({
      ...prev,
      [sectionId]: (prev[sectionId] as FormData[]).map((item) =>
        item.id === itemId ? { ...item, [fieldId]: value } : item
      ),
    }));
  };

  const mergeData = (patch: Partial<FormData>) => {
    setFormData((prev) => ({ ...prev, ...patch }));
  };

  return {
    template,
    loading,
    error,
    registries,
    data: formData,
    setData: setFormData,
    getValue,
    setValue,
    getOptions,
    isFieldVisible,
    isSectionVisible,
    getRepeatableItems,
    addRepeatableItem,
    removeRepeatableItem,
    setRepeatableItemValue,
    mergeData,
  };
}
