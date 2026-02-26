// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Form Template Schema Types
 *
 * These types define the JSON schema format for config-driven forms.
 * Form templates are JSON files that describe form structure, field types,
 * validation rules, and layout — no code changes needed to add/modify fields.
 *
 * Architecture:
 *   templates/forms/*.json   → form structure (sections, fields, layout)
 *   templates/options/*.json → shared dropdown option registries
 *   SchemaFormRenderer       → generic component that renders any template
 *   form_submissions table   → generic JSON blob storage in .ffxdb
 */

// =============================================================================
// FIELD TYPES
// =============================================================================

/** Supported field input types */
export type FieldType =
  | "text"          // Single-line text input
  | "textarea"      // Multi-line text input
  | "number"        // Numeric input
  | "date"          // Date picker (YYYY-MM-DD)
  | "datetime"      // Date-time picker (datetime-local)
  | "time"          // Time picker
  | "select"        // Dropdown select
  | "multiselect"   // Multi-select (tags/chips)
  | "checkbox"      // Boolean checkbox
  | "radio"         // Radio button group
  | "email"         // Email input with validation
  | "phone"         // Phone number input
  | "url"           // URL input
  | "file"          // File reference (path)
  | "hidden"        // Hidden field (auto-populated)
  | "heading"       // Section heading (not an input — display only)
  | "separator"     // Visual separator line
  | "static"        // Static text / instructions (display only)
  | "comma_list";   // Text input that stores as string[] (comma-separated)

// =============================================================================
// VALIDATION
// =============================================================================

/** Validation rule for a field */
export interface FieldValidation {
  /** Field is required */
  required?: boolean;
  /** Minimum length (text) or minimum value (number) */
  min?: number;
  /** Maximum length (text) or maximum value (number) */
  max?: number;
  /** Regex pattern the value must match */
  pattern?: string;
  /** Custom error message when validation fails */
  message?: string;
}

// =============================================================================
// FIELD SCHEMA
// =============================================================================

/** Condition for showing/hiding a field based on another field's value */
export interface FieldCondition {
  /** Field ID to check */
  field: string;
  /** Operator for comparison */
  op: "eq" | "neq" | "in" | "not_in" | "truthy" | "falsy";
  /** Value(s) to compare against */
  value?: string | string[] | boolean | number;
}

/** Inline option for select/radio fields (when not using a shared registry) */
export interface InlineOption {
  value: string;
  label: string;
  icon?: string;
  color?: string;
  /** When true, this option is non-selectable (used for group separator headers) */
  disabled?: boolean;
}

/** Schema definition for a single form field */
export interface FieldSchema {
  /** Unique field ID — used as the data key in form values */
  id: string;
  /** Display label */
  label: string;
  /** Field input type */
  type: FieldType;
  /** Placeholder text */
  placeholder?: string;
  /** Help text shown below the field */
  help?: string;
  /** Default value */
  default?: string | number | boolean | string[];
  /** Validation rules */
  validation?: FieldValidation;
  /** Conditional visibility — field only shown when condition is met */
  show_when?: FieldCondition;
  /**
   * Reference to a shared option registry (for select/radio/multiselect).
   * The renderer loads options from templates/options/{options_ref}.json
   */
  options_ref?: string;
  /** Inline options (alternative to options_ref for small fixed lists) */
  options?: InlineOption[];
  /**
   * When this select has value "other", show a write-in field.
   * The value is the ID of the write-in field.
   */
  other_field?: string;
  /** CSS class additions for the input element */
  input_class?: string;
  /** Grid column span (1-4, default 1) */
  col_span?: number;
  /** Static text content (for type "heading", "static", "separator") */
  content?: string;
  /** Icon/emoji prefix for headings */
  icon?: string;
  /** Number of rows for textarea (default 3) */
  rows?: number;
  /** Whether this field value should be auto-populated from project data */
  auto_fill?: AutoFillSource;
  /**
   * Filter displayed options based on another field's current value.
   * When present, `getOptions()` uses the filter map to show only
   * relevant options for the selected value plus "other" and "n_a".
   */
  options_filter?: OptionsFilter;
}

/**
 * Configuration for filtering a field's options based on another field's value.
 * Example: filter storage_interface options based on device_type selection.
 */
export interface OptionsFilter {
  /** Field ID whose value determines which options to show */
  field: string;
  /** Filter map key — references a named map in deviceTypeFilters.ts */
  filter_map: string;
}

/** Source for auto-filling field values from project data */
export interface AutoFillSource {
  /** Where to pull the value from */
  source: "project" | "case_info" | "examiner" | "preferences" | "evidence" | "section";
  /** Dot-notation path to the value (e.g., "case_info.case_number" or "collection_header.collecting_officer") */
  path: string;
}

// =============================================================================
// SECTION & LAYOUT
// =============================================================================

/** Grid layout configuration for a section */
export interface GridLayout {
  /** Number of columns (1-4, default 2) */
  columns: number;
  /** Gap size class */
  gap?: string;
}

/** Schema definition for a form section (group of fields) */
export interface SectionSchema {
  /** Unique section ID */
  id: string;
  /** Section title */
  title: string;
  /** Icon/emoji for the section header */
  icon?: string;
  /** Description text below the title */
  description?: string;
  /** Fields in this section */
  fields: FieldSchema[];
  /** Grid layout for fields (default: 2 columns) */
  layout?: GridLayout;
  /**
   * Whether this section contains repeatable items (e.g., collected evidence items).
   * When true, the section renders as a list of item cards, each containing
   * the defined fields. An "Add Item" button creates new entries.
   */
  repeatable?: boolean;
  /** Configuration for repeatable sections */
  repeatable_config?: RepeatableConfig;
  /** Whether the section is collapsible (default false) */
  collapsible?: boolean;
  /** Whether the section starts collapsed (default false) */
  collapsed?: boolean;
  /** Conditional visibility — section only shown when condition is met */
  show_when?: FieldCondition;
}

/** Configuration for repeatable (list) sections */
export interface RepeatableConfig {
  /** Singular noun for the add button ("Add {item_label}") */
  item_label: string;
  /** Minimum number of items required */
  min_items?: number;
  /** Maximum number of items allowed */
  max_items?: number;
  /** Field IDs to show in the collapsed summary line */
  summary_fields?: string[];
  /** Whether each item card is collapsible (default true) */
  collapsible?: boolean;
  /** Field to use for auto-numbering (e.g., "item_number") */
  number_field?: string;
  /** Auto-number format: "sequential" or "case_prefix" */
  number_format?: "sequential" | "case_prefix";
}

// =============================================================================
// FORM TEMPLATE
// =============================================================================

/** Complete form template — loaded from templates/forms/*.json */
export interface FormTemplate {
  /** Unique template ID (e.g., "evidence_collection") */
  id: string;
  /** Human-readable template name */
  name: string;
  /** Template version (semver, e.g., "3.0.0") */
  version: string;
  /** Description of the form's purpose */
  description?: string;
  /** Icon/emoji for the form */
  icon?: string;
  /** Which report type(s) this form is used with (if any) */
  report_types?: string[];
  /** Optional: ID of the previous version this supersedes */
  supersedes?: string;
  /** Effective date for this template version */
  effective_date?: string;
  /** Changelog entry for this version */
  changelog?: string;
  /** Sections that make up this form */
  sections: SectionSchema[];
}

// =============================================================================
// OPTION REGISTRY
// =============================================================================

/** Option registry — loaded from templates/options/*.json */
export interface OptionRegistry {
  /** Registry ID (matches the filename, e.g., "device_types") */
  id: string;
  /** Human-readable name */
  name: string;
  /** Version */
  version?: string;
  /** The options list */
  items: InlineOption[];
}

// =============================================================================
// FORM DATA (Runtime)
// =============================================================================

/**
 * Form data at runtime — a flat or nested Record of field values.
 * Keys are field IDs, values are the field's current value.
 * For repeatable sections, the value is an array of objects.
 */
export type FormData = Record<string, FormValue>;

/** Possible values stored for a single field */
export type FormValue =
  | string
  | number
  | boolean
  | string[]
  | FormData[]       // repeatable section items
  | null
  | undefined;

// =============================================================================
// FORM SUBMISSION (DB Storage)
// =============================================================================

/** Stored form submission in the project DB */
export interface FormSubmission {
  /** Unique submission ID */
  id: string;
  /** Template ID this submission was created with */
  template_id: string;
  /** Template version at time of submission */
  template_version: string;
  /** Associated case number (if applicable) */
  case_number?: string;
  /** The form data as a JSON object */
  data: FormData;
  /** Submission status */
  status: "draft" | "complete" | "locked";
  /** Creation timestamp (ISO 8601) */
  created_at: string;
  /** Last modification timestamp (ISO 8601) */
  updated_at: string;
}
