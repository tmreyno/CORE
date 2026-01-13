# CORE-FFX Report Module (Frontend)

Frontend report configuration and API bindings.

## Files

- `api.ts` - Tauri command wrappers
- `generator.ts` - Report workflow helpers
- `types.ts` - Report schema definitions
- `index.ts` - Barrel exports

## Report Flow

```
UI Wizard -> Report Model -> Tauri Commands -> Rust Report Engine -> Output
```

## Output Formats (Backend)

- PDF (genpdf)
- DOCX (docx-rs)
- HTML
- Markdown
- Typst (optional build feature)

## Templates

Report templates are stored in `src-tauri/src/report/templates/`.

## AI Features

AI-assisted narrative generation is optional and requires the backend `ai-assistant` feature and provider configuration.
