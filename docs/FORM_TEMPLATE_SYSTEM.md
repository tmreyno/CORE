# Form Template System — Developer Guide

> **Location:** `src/templates/`  
> **Purpose:** JSON-schema-driven forms for forensic reports, with auto-save to `.ffxdb`

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Data Flow](#data-flow)
3. [File Layout](#file-layout)
4. [Creating a New Form Template](#creating-a-new-form-template)
5. [Creating an Option Registry](#creating-an-option-registry)
6. [Creating a Schema Wrapper Component](#creating-a-schema-wrapper-component)
7. [Template JSON Reference](#template-json-reference)
8. [Field Type Reference](#field-type-reference)
9. [Conditional Visibility](#conditional-visibility)
10. [Repeatable Sections](#repeatable-sections)
11. [Auto-Fill](#auto-fill)
12. [Persistence (Auto-Save)](#persistence-auto-save)
13. [Database Schema](#database-schema)
14. [Hook API Reference](#hook-api-reference)
15. [Adding Fields to an Existing Form](#adding-fields-to-an-existing-form)

---

## Architecture Overview

The form system separates **structure** (JSON templates) from **rendering** (SolidJS components) from **storage** (SQLite). This means:

- **Non-developers can edit forms** by modifying JSON files — no TypeScript required.
- **All forms share one renderer** (`SchemaFormRenderer`) — consistent styling, no duplicated UI code.
- **All form data auto-saves** to the project database (`.ffxdb`) with configurable debounce.

```
┌─────────────────────────────────────────────────────────────────┐
│  JSON Definitions                                               │
│  ┌──────────────────┐  ┌──────────────────┐                    │
│  │ forms/*.json      │  │ options/*.json    │                   │
│  │ (form templates)  │  │ (shared dropdowns)│                   │
│  └────────┬─────────┘  └────────┬──────────┘                   │
│           │  dynamic import      │  cached import              │
│  ┌────────▼──────────────────────▼──────────┐                  │
│  │ useFormTemplate() hook                    │                  │
│  │  → loads JSON, resolves option refs,      │                  │
│  │    manages reactive FormData, conditions  │                  │
│  └────────┬─────────────────────────────────┘                  │
│           │                                                     │
│  ┌────────▼─────────────────────────────────┐                  │
│  │ SchemaFormRenderer                        │                  │
│  │  → renders sections → fields → inputs     │                  │
│  │  → handles grids, repeatable items,       │                  │
│  │    conditional visibility automatically   │                  │
│  └────────┬─────────────────────────────────┘                  │
│           │                                                     │
│  ┌────────▼─────────────────────────────────┐                  │
│  │ Schema Wrapper Component                  │                  │
│  │  → bridges form data ↔ WizardContext      │                  │
│  │  → calls useFormPersistence for auto-save │                  │
│  └────────┬─────────────────────────────────┘                  │
│           │                                                     │
│  ┌────────▼─────────────────────────────────┐                  │
│  │ useFormPersistence → dbSync → Tauri IPC   │                  │
│  │ → Rust project_db_upsert_form_submission  │                  │
│  │ → form_submissions table in .ffxdb        │                  │
│  └──────────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Data Flow

### Loading

1. Wrapper component calls `useFormTemplate({ templateId: "my_form" })`
2. Hook dynamically imports `./forms/my_form.json`
3. For any field with `options_ref: "device_types"`, hook imports `./options/device_types.json`
4. Option registries are **cached** — subsequent loads are instant
5. Hook builds default `FormData` from field definitions and `initialData`

### Editing

1. User types in a field → `SchemaFormRenderer` calls `form.setValue(fieldId, value)`
2. Hook updates reactive `FormData` signal
3. `createEffect` in the wrapper syncs changed data → WizardContext signals
4. `useFormPersistence` effect fires (debounced 2 seconds) → writes to `.ffxdb`

### Conditional Fields

1. Field has `show_when: { field: "type", operator: "eq", value: "physical" }`
2. On every render, `form.isFieldVisible(field)` evaluates the condition
3. If false, the field is hidden — its value is preserved in FormData but not rendered

---

## File Layout

```
src/templates/
├── types.ts                    # TypeScript type definitions (FieldSchema, FormTemplate, etc.)
├── index.ts                    # Barrel file — re-exports everything
├── useFormTemplate.ts          # Hook: loads templates, manages form state
├── useFormPersistence.ts       # Hook: auto-saves form data to .ffxdb
├── SchemaFormRenderer.tsx      # Component: renders any FormTemplate as a form
├── forms/                      # JSON form templates
│   ├── case_info.json          # Case information (4 sections, 12 fields)
│   ├── examiner_info.json      # Examiner details (1 section, 7 fields)
│   ├── evidence_collection.json # Evidence collection (3 sections, repeatable items)
│   ├── iar.json                # Investigative Activity Report
│   ├── user_activity.json      # User activity tracking
│   └── timeline.json           # Timeline of events
└── options/                    # Shared option registries (dropdowns)
    ├── device_types.json       # Desktop, Laptop, Mobile, etc.
    ├── evidence_types.json     # Physical, Digital, Documentary, etc.
    ├── evidence_conditions.json # Factory sealed, Good, Damaged, etc.
    ├── acquisition_methods.json # Physical image, Logical image, etc.
    ├── image_format_options.json # E01, DD/Raw, AFF4, etc.
    ├── classifications.json    # Case priority classifications
    ├── severities.json         # Critical, High, Medium, Low, Info
    ├── investigation_types.json # Computer forensics, Mobile, etc.
    ├── finding_categories.json  # Malware, User activity, etc.
    ├── custody_actions.json    # Received, Transferred, Returned, etc.
    ├── coc_dispositions.json   # Retained, Returned, Destroyed, etc.
    ├── coc_transfer_methods.json # In person, Courier, Secure mail, etc.
    ├── coc_transfer_purposes.json # Examination, Storage, Court, etc.
    ├── iar_event_categories.json # Analysis, Acquisition, Review, etc.
    ├── user_activity_categories.json # Web browsing, File access, etc.
    ├── report_types.json       # Report type options
    └── storage_interface_types.json # SATA, NVMe, USB, etc.
```

---

## Creating a New Form Template

### Step 1: Create the JSON file

Create `src/templates/forms/my_form.json`:

```json
{
  "id": "my_form",
  "name": "My Custom Form",
  "version": "1.0.0",
  "description": "What this form captures.",
  "icon": "📋",
  "report_types": ["forensic_examination"],
  "effective_date": "2025-01-01",
  "sections": [
    {
      "id": "main_info",
      "title": "Main Information",
      "icon": "ℹ️",
      "layout": { "columns": 2 },
      "fields": [
        {
          "id": "case_number",
          "label": "Case Number",
          "type": "text",
          "validation": { "required": true },
          "auto_fill": { "source": "case_info", "path": "case_number" }
        },
        {
          "id": "category",
          "label": "Category",
          "type": "select",
          "options_ref": "finding_categories"
        },
        {
          "id": "notes",
          "label": "Notes",
          "type": "textarea",
          "rows": 4,
          "col_span": 2
        }
      ]
    }
  ]
}
```

### Step 2: (Optional) Create option registries

If your form needs a new dropdown list, create `src/templates/options/my_options.json`:

```json
{
  "id": "my_options",
  "name": "My Options",
  "version": "1.0.0",
  "items": [
    { "value": "option_a", "label": "Option A" },
    { "value": "option_b", "label": "Option B" },
    { "value": "option_c", "label": "Option C", "group": "Advanced" }
  ]
}
```

Then reference it in the field: `"options_ref": "my_options"`.

### Step 3: Create the wrapper component

Create `src/components/report/wizard/steps/reportdata/MyFormSchemaSection.tsx`:

```tsx
import { createEffect, on } from "solid-js";
import { useWizard } from "../../WizardContext";
import { useFormTemplate } from "../../../../../templates/useFormTemplate";
import { useFormPersistence } from "../../../../../templates/useFormPersistence";
import { SchemaFormRenderer } from "../../../../../templates/SchemaFormRenderer";
import type { FormData } from "../../../../../templates/types";

export function MyFormSchemaSection() {
  const ctx = useWizard();

  // 1. Initialize the form from current WizardContext state
  const form = useFormTemplate({
    templateId: "my_form",
    initialData: {
      case_number: ctx.caseInfo().case_number || "",
      category: ctx.myData().category || "",
      notes: ctx.myData().notes || "",
    },
  });

  // 2. Sync form changes → WizardContext (one-way: form → wizard)
  createEffect(
    on(
      () => form.data(),
      (fd) => {
        if (!fd) return;
        ctx.setMyData({
          case_number: (fd.case_number as string) || "",
          category: (fd.category as string) || "",
          notes: (fd.notes as string) || "",
        });
      },
      { defer: true }
    )
  );

  // 3. Wire auto-save to .ffxdb
  useFormPersistence({
    templateId: "my_form",
    templateVersion: "1.0.0",
    caseNumber: () => ctx.caseInfo().case_number,
    data: form.data,
  });

  // 4. Render — that's it
  return <SchemaFormRenderer form={form} />;
}
```

### Step 4: Wire into the wizard

Add the component to `ReportDataStep.tsx` (or wherever appropriate):

```tsx
import { MyFormSchemaSection } from "./MyFormSchemaSection";

// Inside the report data JSX:
<MyFormSchemaSection />
```

---

## Creating an Option Registry

Option registries are shared dropdown menus that can be referenced by any form template.

### JSON Structure

```json
{
  "id": "registry_id",
  "name": "Human-Readable Name",
  "version": "1.0.0",
  "items": [
    { "value": "internal_value", "label": "Display Label" },
    { "value": "grouped_item", "label": "Grouped Item", "group": "Group Name" },
    { "value": "disabled_item", "label": "Not Available", "disabled": true }
  ]
}
```

### Rules

| Rule | Details |
|------|---------|
| Filename = ID | `device_types.json` → `"id": "device_types"` |
| Values are strings | Even for numeric concepts, use string values |
| Groups are optional | Only needed when the dropdown should have `<optgroup>` headers |
| Version is optional | Include it if the registry may evolve |
| Caching | Registries are cached in memory after first load — no duplicate network requests |

### Referencing from a Field

Use `options_ref` (NOT `options`) when pointing to a registry:

```json
{
  "id": "device_type",
  "label": "Device Type",
  "type": "select",
  "options_ref": "device_types"
}
```

For small, form-specific lists, use inline `options` instead:

```json
{
  "id": "priority",
  "label": "Priority",
  "type": "select",
  "options": [
    { "value": "high", "label": "High" },
    { "value": "medium", "label": "Medium" },
    { "value": "low", "label": "Low" }
  ]
}
```

---

## Template JSON Reference

### Top-Level FormTemplate

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `id` | `string` | ✅ | Unique identifier (matches filename without `.json`) |
| `name` | `string` | ✅ | Display name |
| `version` | `string` | ✅ | Semver version (e.g., `"1.0.0"`) |
| `description` | `string` | | Purpose of the form |
| `icon` | `string` | | Emoji/icon prefix |
| `report_types` | `string[]` | | Which report types use this form |
| `supersedes` | `string` | | Previous template ID this version replaces |
| `effective_date` | `string` | | When this version takes effect |
| `changelog` | `string` | | What changed in this version |
| `sections` | `SectionSchema[]` | ✅ | Ordered list of form sections |

### SectionSchema

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `id` | `string` | ✅ | Unique section ID |
| `title` | `string` | ✅ | Section header text |
| `icon` | `string` | | Emoji/icon for header |
| `description` | `string` | | Subtitle text |
| `fields` | `FieldSchema[]` | ✅ | Fields in this section |
| `layout` | `{ columns: 1-4, gap?: string }` | | Grid layout (default: 2 columns) |
| `repeatable` | `boolean` | | Section contains a list of items |
| `repeatable_config` | `RepeatableConfig` | | Config for repeatable sections |
| `collapsible` | `boolean` | | Section can be collapsed |
| `collapsed` | `boolean` | | Starts collapsed |
| `show_when` | `FieldCondition` | | Conditional section visibility |

### RepeatableConfig

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `item_label` | `string` | (required) | Singular noun for "Add {item_label}" button |
| `min_items` | `number` | `0` | Minimum items required |
| `max_items` | `number` | `∞` | Maximum items allowed |
| `summary_fields` | `string[]` | | Field IDs shown in collapsed item header |
| `collapsible` | `boolean` | `true` | Whether items can be collapsed |
| `number_field` | `string` | | Field to auto-populate with sequential number |
| `number_format` | `"sequential" \| "case_prefix"` | | Auto-number format |

---

## Field Type Reference

### Input Fields

| Type | Renders As | Value Type | Notes |
|------|-----------|------------|-------|
| `text` | `<input type="text">` | `string` | Standard text input |
| `textarea` | `<textarea>` | `string` | Multi-line, configurable `rows` (default 3) |
| `number` | `<input type="number">` | `number` | Supports `min`, `max` validation |
| `date` | `<input type="date">` | `string` | ISO date format |
| `datetime` | `<input type="datetime-local">` | `string` | ISO datetime format |
| `time` | `<input type="time">` | `string` | Time string |
| `email` | `<input type="email">` | `string` | Email validation |
| `phone` | `<input type="tel">` | `string` | Phone number |
| `url` | `<input type="url">` | `string` | URL validation |
| `file` | File picker | `string` | File path |
| `hidden` | None (hidden) | `string` | Hidden field, not rendered |

### Choice Fields

| Type | Renders As | Value Type | Notes |
|------|-----------|------------|-------|
| `select` | `<select>` dropdown | `string` | Single selection; uses `options` or `options_ref` |
| `multiselect` | Checkbox list | `string[]` | Multiple selection |
| `checkbox` | `<input type="checkbox">` | `boolean` | Single true/false toggle |
| `radio` | Radio button group | `string` | Single selection; uses `options` or `options_ref` |
| `comma_list` | Tag input | `string[]` | Values separated by commas |

### Display Fields (Non-Input)

| Type | Renders As | Notes |
|------|-----------|-------|
| `heading` | `<h3>` | Section sub-heading; uses `content` property |
| `separator` | `<hr>` | Visual divider |
| `static` | `<p>` | Static informational text; uses `content` property |

### FieldSchema Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `id` | `string` | ✅ | Unique field identifier |
| `label` | `string` | ✅ | Display label |
| `type` | `FieldType` | ✅ | One of the 20 types above |
| `placeholder` | `string` | | Placeholder text |
| `help` | `string` | | Helper text below the field |
| `default_value` | `FormValue` | | Initial value |
| `validation` | `FieldValidation` | | Validation rules |
| `options` | `InlineOption[]` | | Inline dropdown options |
| `options_ref` | `string` | | Reference to an option registry file |
| `show_when` | `FieldCondition` | | Conditional visibility |
| `col_span` | `number` | | Grid column span (1-4) |
| `content` | `string` | | Text for heading/static/separator types |
| `icon` | `string` | | Icon for headings |
| `rows` | `number` | | Textarea row count (default 3) |
| `auto_fill` | `AutoFillSource` | | Auto-populate from project data |

### FieldValidation

| Property | Type | Description |
|----------|------|-------------|
| `required` | `boolean` | Field must have a value |
| `min` | `number` | Minimum numeric value |
| `max` | `number` | Maximum numeric value |
| `min_length` | `number` | Minimum string length |
| `max_length` | `number` | Maximum string length |
| `pattern` | `string` | Regex pattern for validation |
| `pattern_message` | `string` | Error message when pattern fails |

---

## Conditional Visibility

Fields and sections can be shown/hidden based on other field values using `show_when`.

### Syntax

```json
{
  "show_when": {
    "field": "field_id_to_check",
    "operator": "eq",
    "value": "expected_value"
  }
}
```

### Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `eq` | Equals | `{ "field": "type", "operator": "eq", "value": "physical" }` |
| `neq` | Not equals | `{ "field": "status", "operator": "neq", "value": "draft" }` |
| `in` | Value in array | `{ "field": "format", "operator": "in", "value": ["E01", "DD"] }` |
| `not_in` | Value not in array | `{ "field": "type", "operator": "not_in", "value": ["note"] }` |
| `truthy` | Value is truthy | `{ "field": "has_notes", "operator": "truthy" }` |
| `falsy` | Value is falsy | `{ "field": "is_deleted", "operator": "falsy" }` |

### Example: Conditional Field

```json
{
  "id": "physical_description",
  "label": "Physical Description",
  "type": "textarea",
  "show_when": {
    "field": "evidence_type",
    "operator": "eq",
    "value": "physical"
  }
}
```

The "Physical Description" field only appears when `evidence_type` is set to `"physical"`.

---

## Repeatable Sections

Repeatable sections render as a list of item cards. Each item contains the full set of fields defined in the section. Users can add/remove items dynamically.

### JSON Example

```json
{
  "id": "collected_items",
  "title": "Collected Items",
  "repeatable": true,
  "repeatable_config": {
    "item_label": "Evidence Item",
    "min_items": 0,
    "max_items": 100,
    "summary_fields": ["device_type", "serial_number"],
    "collapsible": true,
    "number_field": "item_number",
    "number_format": "sequential"
  },
  "fields": [
    { "id": "item_number", "label": "#", "type": "text" },
    { "id": "device_type", "label": "Device", "type": "select", "options_ref": "device_types" },
    { "id": "serial_number", "label": "Serial", "type": "text" }
  ]
}
```

### Data Shape at Runtime

Repeatable sections store their data as an array under the section's `id`:

```typescript
formData = {
  // Regular fields
  case_number: "2025-001",
  
  // Repeatable section → array of objects
  collected_items: [
    { item_number: "1", device_type: "laptop", serial_number: "ABC123" },
    { item_number: "2", device_type: "mobile_phone", serial_number: "DEF456" }
  ]
}
```

### Auto-Numbering

When `number_field` is set, new items automatically get sequential numbers. With `"number_format": "case_prefix"`, the wrapper component can generate case-prefixed numbers (e.g., `2025-001-SH-1`).

---

## Auto-Fill

Fields can be auto-populated from project data using `auto_fill`:

```json
{
  "id": "examiner_name",
  "label": "Examiner",
  "type": "text",
  "auto_fill": {
    "source": "examiner",
    "path": "name"
  }
}
```

### Sources

| Source | Description |
|--------|-------------|
| `project` | Project metadata |
| `case_info` | Current case information |
| `examiner` | Examiner profile |
| `preferences` | User preferences |
| `evidence` | Evidence file data |

The `path` is a dot-notation string into the source object (e.g., `"case_info.case_number"`).

> **Note:** Auto-fill is currently defined in the schema but requires wrapper components to implement the actual value resolution from WizardContext signals.

---

## Persistence (Auto-Save)

### How It Works

1. `useFormPersistence` watches the form's `data()` signal
2. On change, it **debounces** for 2 seconds (configurable)
3. After debounce, it calls `dbSync.upsertFormSubmission()` (fire-and-forget)
4. The Tauri backend upserts into the `form_submissions` table in `.ffxdb`

### Submission IDs

Each template+case combination gets a **deterministic ID**:

```
form-{templateId}-{hash(templateId::caseNumber)}
```

This means:
- Reopening the same form for the same case overwrites (upserts) the same row
- Different cases get different rows
- Different templates always get different rows

### Usage in a Wrapper

```tsx
useFormPersistence({
  templateId: "my_form",
  templateVersion: "1.0.0",
  caseNumber: () => ctx.caseInfo().case_number,
  data: form.data,
  debounceMs: 2000,        // optional, default 2000
  enabled: () => true,     // optional, default true
});
```

### Options

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `templateId` | `string` | (required) | Template identifier |
| `templateVersion` | `string` | (required) | Current template version |
| `caseNumber` | `Accessor<string \| undefined>` | (required) | Reactive case number |
| `data` | `Accessor<FormData>` | (required) | The form data to persist |
| `debounceMs` | `number` | `2000` | Debounce interval in ms |
| `enabled` | `Accessor<boolean>` | `() => true` | Toggle persistence on/off |

---

## Database Schema

### Rust Side (`src-tauri/src/project_db.rs`, schema v6)

```sql
CREATE TABLE IF NOT EXISTS form_submissions (
  id TEXT PRIMARY KEY,
  template_id TEXT NOT NULL,
  template_version TEXT NOT NULL,
  case_number TEXT,
  data_json TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'draft',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

### Tauri Commands

| Command | Purpose |
|---------|---------|
| `project_db_upsert_form_submission` | Insert or update a form submission |
| `project_db_get_form_submission` | Get a single submission by ID |
| `project_db_list_form_submissions` | List submissions, optionally filtered by template_id |
| `project_db_delete_form_submission` | Delete a submission by ID |

### TypeScript Interface (`src/types/projectDb.ts`)

```typescript
interface DbFormSubmission {
  id: string;
  templateId: string;
  templateVersion: string;
  caseNumber?: string;
  dataJson: string;           // JSON-stringified FormData
  status: "draft" | "complete" | "locked";
  createdAt: string;          // ISO 8601
  updatedAt: string;          // ISO 8601
}
```

---

## Hook API Reference

### `useFormTemplate(options)`

The core hook that loads a template and manages reactive form state.

**Parameters:**

```typescript
interface UseFormTemplateOptions {
  templateId: string;         // JSON filename (without .json)
  initialData?: FormData;     // Pre-populate fields
}
```

**Returns:**

| Property | Type | Description |
|----------|------|-------------|
| `template` | `Accessor<FormTemplate \| undefined>` | Loaded template definition |
| `loading` | `Accessor<boolean>` | Whether template is still loading |
| `error` | `Accessor<string \| undefined>` | Error message if load failed |
| `registries` | `Accessor<Map<string, OptionRegistry>>` | Loaded option registries |
| `data` | `Accessor<FormData>` | Current form data (reactive) |
| `setData` | `(data: FormData) => void` | Replace all form data |
| `getValue` | `(fieldId: string) => FormValue` | Get a single field value |
| `setValue` | `(fieldId: string, value: FormValue) => void` | Set a single field value |
| `getOptions` | `(field: FieldSchema) => InlineOption[]` | Resolved options for a field |
| `isFieldVisible` | `(field: FieldSchema) => boolean` | Check field visibility (evaluates `show_when`) |
| `isSectionVisible` | `(section: SectionSchema) => boolean` | Check section visibility |
| `getRepeatableItems` | `(sectionId: string) => FormData[]` | Get items in a repeatable section |
| `addRepeatableItem` | `(sectionId: string) => void` | Add new item to repeatable section |
| `removeRepeatableItem` | `(sectionId: string, index: number) => void` | Remove item by index |
| `setRepeatableItemValue` | `(sectionId: string, index: number, fieldId: string, value: FormValue) => void` | Update a field in a repeatable item |
| `mergeData` | `(partial: Partial<FormData>) => void` | Merge partial data into current |

### `SchemaFormRenderer`

**Props:**

```typescript
interface SchemaFormRendererProps {
  form: ReturnType<typeof useFormTemplate>;  // The form hook instance
  class?: string;                            // Additional CSS class
}
```

Usage: `<SchemaFormRenderer form={form} />`

That's it — one prop. The renderer reads everything from the hook.

---

## Adding Fields to an Existing Form

### Step 1: Edit the JSON template

Open the form file (e.g., `src/templates/forms/case_info.json`) and add a field to the appropriate section:

```json
{
  "id": "jurisdiction",
  "label": "Jurisdiction",
  "type": "text",
  "placeholder": "e.g., Federal, State of California"
}
```

### Step 2: Update the wrapper component

If the new field maps to an existing WizardContext signal, add it to the `wizardToFormData()` and sync effect. If it's a new field not in the context, it will still be stored in FormData and auto-saved — no wrapper changes needed.

### Step 3: Bump the version

Update the template's `version` field and optionally add a `changelog` entry:

```json
{
  "version": "1.1.0",
  "changelog": "Added jurisdiction field"
}
```

---

## Existing Forms

| Template | File | Sections | Key Features |
|----------|------|----------|--------------|
| `case_info` | `forms/case_info.json` | 4 (case details, classification, parties, dates) | 12 fields, auto-fill from project |
| `examiner_info` | `forms/examiner_info.json` | 1 (examiner profile) | 7 fields, agency/certs/phone |
| `evidence_collection` | `forms/evidence_collection.json` | 3 (header, items, docs) | Repeatable items with auto-numbering |
| `iar` | `forms/iar.json` | 2 (summary, entries) | Repeatable activity entries |
| `user_activity` | `forms/user_activity.json` | 2 (target info, entries) | Repeatable activity entries with categories |
| `timeline` | `forms/timeline.json` | 2 (config, events) | Multiselect sources, repeatable events |

## Existing Option Registries

| Registry | File | Items | Used By |
|----------|------|-------|---------|
| `device_types` | `options/device_types.json` | 22 | evidence_collection |
| `storage_interface_types` | `options/storage_interface_types.json` | — | evidence_collection |
| `image_format_options` | `options/image_format_options.json` | — | evidence_collection |
| `acquisition_methods` | `options/acquisition_methods.json` | — | evidence_collection |
| `evidence_conditions` | `options/evidence_conditions.json` | — | evidence_collection |
| `evidence_types` | `options/evidence_types.json` | — | case_info |
| `investigation_types` | `options/investigation_types.json` | — | case_info |
| `classifications` | `options/classifications.json` | — | case_info |
| `severities` | `options/severities.json` | — | case_info |
| `finding_categories` | `options/finding_categories.json` | — | iar |
| `custody_actions` | `options/custody_actions.json` | — | — |
| `coc_dispositions` | `options/coc_dispositions.json` | — | — |
| `coc_transfer_methods` | `options/coc_transfer_methods.json` | — | — |
| `coc_transfer_purposes` | `options/coc_transfer_purposes.json` | — | — |
| `iar_event_categories` | `options/iar_event_categories.json` | — | iar |
| `user_activity_categories` | `options/user_activity_categories.json` | — | user_activity |
| `report_types` | `options/report_types.json` | — | — |
