// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Templates Module — JSON schema-driven form system
 *
 * Public API:
 *   - Types: FormTemplate, FieldSchema, SectionSchema, OptionRegistry, FormData, FormSubmission
 *   - Hook:  useFormTemplate(options) → reactive form state
 *   - UI:    SchemaFormRenderer → renders any FormTemplate
 *
 * File layout:
 *   templates/
 *     types.ts               ← Schema type definitions
 *     useFormTemplate.ts     ← State management hook
 *     SchemaFormRenderer.tsx ← Generic renderer component
 *     index.ts              ← This barrel file
 *     forms/                ← Form template JSON files
 *       case_info.json
 *       examiner_info.json
 *       evidence_collection.json
 *       iar.json
 *       user_activity.json
 *       timeline.json
 *     options/              ← Shared option registry JSON files
 *       device_types.json
 *       classifications.json
 *       ... (17 total)
 */

// --- Types ---
export type {
  FieldType,
  FieldValidation,
  FieldCondition,
  InlineOption,
  FieldSchema,
  AutoFillSource,
  OptionsFilter,
  GridLayout,
  SectionSchema,
  RepeatableConfig,
  FormTemplate,
  OptionRegistry,
  FormData,
  FormValue,
  FormSubmission,
} from "./types";

// --- Hook ---
export {
  useFormTemplate,
  loadFormTemplate,
  loadOptionRegistry,
  evaluateCondition,
  buildDefaults,
  createRepeatableItem,
} from "./useFormTemplate";
export type { UseFormTemplateOptions, UseFormTemplateReturn } from "./useFormTemplate";

// --- Filters ---
export { getFilterMap, filterOptions } from "./deviceTypeFilters";
export type { FilterMap } from "./deviceTypeFilters";

// --- Component ---
export { SchemaFormRenderer } from "./SchemaFormRenderer";
export type { SchemaFormRendererProps } from "./SchemaFormRenderer";

// --- Persistence ---
export { useFormPersistence } from "./useFormPersistence";
export type { UseFormPersistenceOptions } from "./useFormPersistence";
